//! Pure-Rust inference for the tiny char-level transformer G2P used for
//! out-of-vocabulary words. Mirrors `train_g2p.py` exactly: learned positions,
//! pre-LayerNorm blocks, ReLU FFN, greedy decoding with a KV cache.
//!
//! Weight file: b"G2PW" | u32 header_len | JSON header | f32 LE tensors.

use std::collections::HashMap;
use std::io;
use std::path::Path;

#[derive(serde::Deserialize)]
struct Cfg {
    d: usize,
    heads: usize,
    layers: usize,
    ff: usize,
    max_in: usize,
    max_out: usize,
}

#[derive(serde::Deserialize)]
struct TensorMeta {
    name: String,
    shape: Vec<usize>,
    offset: usize,
}

#[derive(serde::Deserialize)]
struct Header {
    config: Cfg,
    src_vocab: HashMap<String, usize>,
    tgt_vocab: Vec<String>,
    tensors: Vec<TensorMeta>,
}

/// Linear layer, weight stored transposed ([in, out]) for a cache-friendly matvec.
struct Lin {
    wt: Vec<f32>,
    b: Vec<f32>,
    i: usize,
    o: usize,
}

#[inline(always)]
fn matvec_generic(wt: &[f32], b: &[f32], o: usize, x: &[f32], y: &mut [f32]) {
    y.copy_from_slice(b);
    for (k, &xk) in x.iter().enumerate() {
        if xk == 0.0 {
            continue;
        }
        let row = &wt[k * o..(k + 1) * o];
        for (yj, &w) in y.iter_mut().zip(row) {
            *yj += xk * w;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn matvec_avx2(wt: &[f32], b: &[f32], o: usize, x: &[f32], y: &mut [f32]) {
    matvec_generic(wt, b, o, x, y)
}

#[cfg(target_arch = "x86_64")]
fn has_avx2() -> bool {
    use std::sync::OnceLock;
    static F: OnceLock<bool> = OnceLock::new();
    *F.get_or_init(|| is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma"))
}

impl Lin {
    fn apply(&self, x: &[f32], y: &mut [f32]) {
        #[cfg(target_arch = "x86_64")]
        if has_avx2() {
            // SAFETY: the CPU supports avx2+fma (checked at runtime).
            unsafe { matvec_avx2(&self.wt, &self.b, self.o, x, y) };
            return;
        }
        matvec_generic(&self.wt, &self.b, self.o, x, y)
    }
}

struct Ln {
    g: Vec<f32>,
    b: Vec<f32>,
}

impl Ln {
    fn apply(&self, x: &[f32], y: &mut [f32]) {
        let n = x.len() as f32;
        let mean = x.iter().sum::<f32>() / n;
        let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
        let inv = 1.0 / (var + 1e-5).sqrt();
        for i in 0..x.len() {
            y[i] = (x[i] - mean) * inv * self.g[i] + self.b[i];
        }
    }
}

struct Mha {
    q: Lin,
    k: Lin,
    v: Lin,
    o: Lin,
}

struct Block {
    ln1: Ln,
    sa: Mha,
    ln2: Option<Ln>,
    ca: Option<Mha>,
    ln3: Ln,
    ff1: Lin,
    ff2: Lin,
}

pub struct Model {
    cfg: Cfg,
    src_vocab: HashMap<char, usize>,
    tgt_vocab: Vec<String>,
    src_emb: Vec<f32>,
    tgt_emb: Vec<f32>,
    src_pos: Vec<f32>,
    tgt_pos: Vec<f32>,
    enc: Vec<Block>,
    dec: Vec<Block>,
    enc_norm: Ln,
    dec_norm: Ln,
    out: Lin,
}

impl Model {
    pub fn open(path: &Path) -> io::Result<Model> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Model> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        if bytes.len() < 8 || &bytes[..4] != b"G2PW" {
            return Err(bad("not a G2PW file"));
        }
        let hl = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let h: Header = serde_json::from_slice(&bytes[8..8 + hl]).map_err(|e| bad(&e.to_string()))?;
        let data = &bytes[8 + hl..];
        let mut t: HashMap<String, (Vec<usize>, Vec<f32>)> = HashMap::new();
        for m in &h.tensors {
            let n: usize = m.shape.iter().product();
            let raw = &data[m.offset..m.offset + n * 4];
            let v: Vec<f32> = raw.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
            t.insert(m.name.clone(), (m.shape.clone(), v));
        }
        let mut take = |name: &str| -> io::Result<(Vec<usize>, Vec<f32>)> {
            t.remove(name).ok_or_else(|| bad(&format!("missing tensor {name}")))
        };
        let lin = |take: &mut dyn FnMut(&str) -> io::Result<(Vec<usize>, Vec<f32>)>, p: &str| -> io::Result<Lin> {
            let (s, w) = take(&format!("{p}.weight"))?; // [out, in]
            let (_, b) = take(&format!("{p}.bias"))?;
            let (o, i) = (s[0], s[1]);
            let mut wt = vec![0f32; i * o];
            for r in 0..o {
                for c in 0..i {
                    wt[c * o + r] = w[r * i + c];
                }
            }
            Ok(Lin { wt, b, i, o })
        };
        let ln = |take: &mut dyn FnMut(&str) -> io::Result<(Vec<usize>, Vec<f32>)>, p: &str| -> io::Result<Ln> {
            Ok(Ln { g: take(&format!("{p}.weight"))?.1, b: take(&format!("{p}.bias"))?.1 })
        };
        let mha = |take: &mut dyn FnMut(&str) -> io::Result<(Vec<usize>, Vec<f32>)>, p: &str| -> io::Result<Mha> {
            Ok(Mha {
                q: lin(take, &format!("{p}.q"))?,
                k: lin(take, &format!("{p}.k"))?,
                v: lin(take, &format!("{p}.v"))?,
                o: lin(take, &format!("{p}.o"))?,
            })
        };
        let block = |take: &mut dyn FnMut(&str) -> io::Result<(Vec<usize>, Vec<f32>)>, p: &str, cross: bool| -> io::Result<Block> {
            Ok(Block {
                ln1: ln(take, &format!("{p}.ln1"))?,
                sa: mha(take, &format!("{p}.sa"))?,
                ln2: if cross { Some(ln(take, &format!("{p}.ln2"))?) } else { None },
                ca: if cross { Some(mha(take, &format!("{p}.ca"))?) } else { None },
                ln3: ln(take, &format!("{p}.ln3"))?,
                ff1: lin(take, &format!("{p}.ff1"))?,
                ff2: lin(take, &format!("{p}.ff2"))?,
            })
        };
        let cfg = h.config;
        let mut enc = Vec::new();
        let mut dec = Vec::new();
        for l in 0..cfg.layers {
            enc.push(block(&mut take, &format!("enc.{l}"), false)?);
            dec.push(block(&mut take, &format!("dec.{l}"), true)?);
        }
        let src_vocab = h.src_vocab.into_iter().filter_map(|(k, v)| k.chars().next().map(|c| (c, v))).collect();
        Ok(Model {
            src_emb: take("src_emb.weight")?.1,
            tgt_emb: take("tgt_emb.weight")?.1,
            src_pos: take("src_pos.weight")?.1,
            tgt_pos: take("tgt_pos.weight")?.1,
            enc_norm: ln(&mut take, "enc_norm")?,
            dec_norm: ln(&mut take, "dec_norm")?,
            out: lin(&mut take, "out")?,
            enc,
            dec,
            cfg,
            src_vocab,
            tgt_vocab: h.tgt_vocab,
        })
    }

    /// Greedy-decode the phoneme string for `word`.
    pub fn predict(&self, word: &str) -> String {
        let d = self.cfg.d;
        let ids: Vec<usize> = word.chars().take(self.cfg.max_in).map(|c| *self.src_vocab.get(&c).unwrap_or(&3)).collect();
        if ids.is_empty() {
            return String::new();
        }
        let s = ids.len();
        // ---- encoder
        let mut x = vec![0f32; s * d];
        for (p, &id) in ids.iter().enumerate() {
            for j in 0..d {
                x[p * d + j] = self.src_emb[id * d + j] + self.src_pos[p * d + j];
            }
        }
        let mut h = vec![0f32; s * d];
        for b in &self.enc {
            for p in 0..s {
                b.ln1.apply(&x[p * d..(p + 1) * d], &mut h[p * d..(p + 1) * d]);
            }
            let (k, v) = project_kv(&b.sa, &h, s, d);
            let mut att = vec![0f32; d];
            let mut q = vec![0f32; d];
            let mut o = vec![0f32; d];
            for p in 0..s {
                b.sa.q.apply(&h[p * d..(p + 1) * d], &mut q);
                attend(&q, &k, &v, s, self.cfg.heads, d, &mut att);
                b.sa.o.apply(&att, &mut o);
                for j in 0..d {
                    x[p * d + j] += o[j];
                }
            }
            ffn(b, &mut x, s, d, self.cfg.ff);
        }
        let mut mem = vec![0f32; s * d];
        for p in 0..s {
            self.enc_norm.apply(&x[p * d..(p + 1) * d], &mut mem[p * d..(p + 1) * d]);
        }
        // cross-attention K/V per decoder layer
        let cross: Vec<(Vec<f32>, Vec<f32>)> =
            self.dec.iter().map(|b| project_kv(b.ca.as_ref().unwrap(), &mem, s, d)).collect();
        // ---- decoder with self-attention KV cache
        let max_t = self.cfg.max_out - 1;
        let nl = self.dec.len();
        let mut kc: Vec<Vec<f32>> = vec![Vec::with_capacity(max_t * d); nl];
        let mut vc: Vec<Vec<f32>> = vec![Vec::with_capacity(max_t * d); nl];
        let mut out = String::new();
        let mut tok = 1usize; // BOS
        let mut y = vec![0f32; d];
        let mut hh = vec![0f32; d];
        let mut q = vec![0f32; d];
        let mut att = vec![0f32; d];
        let mut o = vec![0f32; d];
        let mut kk = vec![0f32; d];
        let mut vv = vec![0f32; d];
        let mut f1 = vec![0f32; self.cfg.ff];
        let mut logits = vec![0f32; self.tgt_vocab.len()];
        for t in 0..max_t {
            for j in 0..d {
                y[j] = self.tgt_emb[tok * d + j] + self.tgt_pos[t * d + j];
            }
            for (l, b) in self.dec.iter().enumerate() {
                b.ln1.apply(&y, &mut hh);
                b.sa.q.apply(&hh, &mut q);
                b.sa.k.apply(&hh, &mut kk);
                b.sa.v.apply(&hh, &mut vv);
                kc[l].extend_from_slice(&kk);
                vc[l].extend_from_slice(&vv);
                attend(&q, &kc[l], &vc[l], t + 1, self.cfg.heads, d, &mut att);
                b.sa.o.apply(&att, &mut o);
                for j in 0..d {
                    y[j] += o[j];
                }
                let ca = b.ca.as_ref().unwrap();
                b.ln2.as_ref().unwrap().apply(&y, &mut hh);
                ca.q.apply(&hh, &mut q);
                attend(&q, &cross[l].0, &cross[l].1, s, self.cfg.heads, d, &mut att);
                ca.o.apply(&att, &mut o);
                for j in 0..d {
                    y[j] += o[j];
                }
                b.ln3.apply(&y, &mut hh);
                b.ff1.apply(&hh, &mut f1);
                for v in f1.iter_mut() {
                    *v = v.max(0.0);
                }
                b.ff2.apply(&f1, &mut o);
                for j in 0..d {
                    y[j] += o[j];
                }
            }
            self.dec_norm.apply(&y, &mut hh);
            self.out.apply(&hh, &mut logits);
            let (best, _) = logits.iter().enumerate().fold((0, f32::MIN), |a, (i, &v)| if v > a.1 { (i, v) } else { a });
            if best == 2 || best == 0 {
                break;
            }
            out.push_str(&self.tgt_vocab[best]);
            tok = best;
        }
        out
    }
}

fn project_kv(m: &Mha, h: &[f32], s: usize, d: usize) -> (Vec<f32>, Vec<f32>) {
    let mut k = vec![0f32; s * d];
    let mut v = vec![0f32; s * d];
    for p in 0..s {
        m.k.apply(&h[p * d..(p + 1) * d], &mut k[p * d..(p + 1) * d]);
        m.v.apply(&h[p * d..(p + 1) * d], &mut v[p * d..(p + 1) * d]);
    }
    (k, v)
}

/// Multi-head attention of one query over `n` keys/values.
fn attend(q: &[f32], k: &[f32], v: &[f32], n: usize, heads: usize, d: usize, out: &mut [f32]) {
    let hd = d / heads;
    let scale = 1.0 / (hd as f32).sqrt();
    let mut w = vec![0f32; n];
    for h in 0..heads {
        let qs = &q[h * hd..(h + 1) * hd];
        let mut mx = f32::MIN;
        for p in 0..n {
            let ks = &k[p * d + h * hd..p * d + (h + 1) * hd];
            let dot: f32 = qs.iter().zip(ks).map(|(a, b)| a * b).sum::<f32>() * scale;
            w[p] = dot;
            mx = mx.max(dot);
        }
        let mut sum = 0.0;
        for x in w.iter_mut() {
            *x = (*x - mx).exp();
            sum += *x;
        }
        let os = &mut out[h * hd..(h + 1) * hd];
        os.iter_mut().for_each(|x| *x = 0.0);
        for p in 0..n {
            let a = w[p] / sum;
            let vs = &v[p * d + h * hd..p * d + (h + 1) * hd];
            for (o, &vv) in os.iter_mut().zip(vs) {
                *o += a * vv;
            }
        }
    }
}

fn ffn(b: &Block, x: &mut [f32], s: usize, d: usize, ff: usize) {
    let mut h = vec![0f32; d];
    let mut f1 = vec![0f32; ff];
    let mut o = vec![0f32; d];
    for p in 0..s {
        b.ln3.apply(&x[p * d..(p + 1) * d], &mut h);
        b.ff1.apply(&h, &mut f1);
        for v in f1.iter_mut() {
            *v = v.max(0.0);
        }
        b.ff2.apply(&f1, &mut o);
        for j in 0..d {
            x[p * d + j] += o[j];
        }
    }
}

// silence unused-field warnings for fields kept for clarity
#[allow(dead_code)]
fn _unused(l: &Lin) -> usize {
    l.i + l.o
}
