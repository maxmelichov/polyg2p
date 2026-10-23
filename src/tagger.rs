//! Pure-Rust inference for the character-level transformer **encoder tagger**
//! used by the Hebrew engine (trained by `tools/train_he.py`). Mirrors it
//! exactly: learned positions, pre-LayerNorm blocks, ReLU FFN, final LayerNorm,
//! three linear heads per position (consonant / vowel / stress).
//!
//! Weight file (`G2PT`): b"G2PT" | u32 header_len | JSON header | tensors.
//! Tensors are little-endian f32, or int8 with a per-output-row f32 scale
//! (`"dtype": "i8"`, scale tensor `<name>.scale`) which is dequantized at load.
//!
//! The matmul kernel works on the whole sequence at once (4 input rows × 2
//! weight rows per tile) so one sentence costs a few milliseconds.

use std::collections::HashMap;
use std::io;
use std::path::Path;

#[derive(serde::Deserialize, Clone)]
pub struct Cfg {
    pub d: usize,
    pub heads: usize,
    pub layers: usize,
    pub ff: usize,
    pub max_len: usize,
    pub n_cons: usize,
    pub n_vow: usize,
}

#[derive(serde::Deserialize)]
struct TensorMeta {
    name: String,
    shape: Vec<usize>,
    offset: usize,
    #[serde(default)]
    dtype: Option<String>,
}

#[derive(serde::Deserialize)]
struct Header {
    config: Cfg,
    vocab: HashMap<String, usize>,
    cons_vocab: Vec<String>,
    vow_vocab: Vec<String>,
    letter_cons: HashMap<String, Vec<usize>>,
    tensors: Vec<TensorMeta>,
}

/// Linear layer, weight in PyTorch layout `[out, in]`.
struct Lin {
    w: Vec<f32>,
    b: Vec<f32>,
    i: usize,
    o: usize,
}

const LANES: usize = 8;

#[inline(always)]
fn dot_tail(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

impl Lin {
    /// `y[l, o] = x[l, i] · Wᵀ + b` for `l` rows.
    fn apply(&self, x: &[f32], l: usize, y: &mut [f32]) {
        let (ni, no) = (self.i, self.o);
        debug_assert_eq!(x.len(), l * ni);
        debug_assert_eq!(y.len(), l * no);
        let kmain = ni - ni % LANES;
        let mut r = 0;
        while r + 4 <= l {
            let x0 = &x[r * ni..(r + 1) * ni];
            let x1 = &x[(r + 1) * ni..(r + 2) * ni];
            let x2 = &x[(r + 2) * ni..(r + 3) * ni];
            let x3 = &x[(r + 3) * ni..(r + 4) * ni];
            let mut c = 0;
            while c + 2 <= no {
                let w0 = &self.w[c * ni..(c + 1) * ni];
                let w1 = &self.w[(c + 1) * ni..(c + 2) * ni];
                let mut a = [[0f32; LANES]; 8];
                let mut k = 0;
                while k < kmain {
                    let (p0, p1, p2, p3) = (&x0[k..k + LANES], &x1[k..k + LANES], &x2[k..k + LANES], &x3[k..k + LANES]);
                    let (q0, q1) = (&w0[k..k + LANES], &w1[k..k + LANES]);
                    for j in 0..LANES {
                        a[0][j] += p0[j] * q0[j];
                        a[1][j] += p0[j] * q1[j];
                        a[2][j] += p1[j] * q0[j];
                        a[3][j] += p1[j] * q1[j];
                        a[4][j] += p2[j] * q0[j];
                        a[5][j] += p2[j] * q1[j];
                        a[6][j] += p3[j] * q0[j];
                        a[7][j] += p3[j] * q1[j];
                    }
                    k += LANES;
                }
                let s: Vec<f32> = a.iter().map(|v| v.iter().sum::<f32>()).collect();
                let (t0, t1) = (&w0[kmain..], &w1[kmain..]);
                y[r * no + c] = s[0] + dot_tail(&x0[kmain..], t0) + self.b[c];
                y[r * no + c + 1] = s[1] + dot_tail(&x0[kmain..], t1) + self.b[c + 1];
                y[(r + 1) * no + c] = s[2] + dot_tail(&x1[kmain..], t0) + self.b[c];
                y[(r + 1) * no + c + 1] = s[3] + dot_tail(&x1[kmain..], t1) + self.b[c + 1];
                y[(r + 2) * no + c] = s[4] + dot_tail(&x2[kmain..], t0) + self.b[c];
                y[(r + 2) * no + c + 1] = s[5] + dot_tail(&x2[kmain..], t1) + self.b[c + 1];
                y[(r + 3) * no + c] = s[6] + dot_tail(&x3[kmain..], t0) + self.b[c];
                y[(r + 3) * no + c + 1] = s[7] + dot_tail(&x3[kmain..], t1) + self.b[c + 1];
                c += 2;
            }
            if c < no {
                for rr in r..r + 4 {
                    y[rr * no + c] = dot(&x[rr * ni..(rr + 1) * ni], &self.w[c * ni..(c + 1) * ni]) + self.b[c];
                }
            }
            r += 4;
        }
        for rr in r..l {
            let xr = &x[rr * ni..(rr + 1) * ni];
            for c in 0..no {
                y[rr * no + c] = dot(xr, &self.w[c * ni..(c + 1) * ni]) + self.b[c];
            }
        }
    }
}

#[inline(always)]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() - a.len() % LANES;
    let mut acc = [0f32; LANES];
    let mut k = 0;
    while k < n {
        for j in 0..LANES {
            acc[j] += a[k + j] * b[k + j];
        }
        k += LANES;
    }
    acc.iter().sum::<f32>() + dot_tail(&a[n..], &b[n..])
}

struct Ln {
    g: Vec<f32>,
    b: Vec<f32>,
}

impl Ln {
    fn apply(&self, x: &[f32], l: usize, d: usize, y: &mut [f32]) {
        for p in 0..l {
            let xs = &x[p * d..(p + 1) * d];
            let mean = xs.iter().sum::<f32>() / d as f32;
            let var = xs.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / d as f32;
            let inv = 1.0 / (var + 1e-5).sqrt();
            let ys = &mut y[p * d..(p + 1) * d];
            for i in 0..d {
                ys[i] = (xs[i] - mean) * inv * self.g[i] + self.b[i];
            }
        }
    }
}

struct Block {
    ln1: Ln,
    q: Lin,
    k: Lin,
    v: Lin,
    o: Lin,
    ln3: Ln,
    ff1: Lin,
    ff2: Lin,
}

/// Logits for one window: `[len, n]` row-major per head.
pub struct Logits {
    pub cons: Vec<f32>,
    pub vow: Vec<f32>,
    pub stress: Vec<f32>,
    pub len: usize,
}

pub struct Tagger {
    pub cfg: Cfg,
    vocab: HashMap<char, usize>,
    pub cons_vocab: Vec<String>,
    pub vow_vocab: Vec<String>,
    /// legal consonant classes per Hebrew letter
    pub letter_cons: HashMap<char, Vec<usize>>,
    emb: Vec<f32>,
    pos: Vec<f32>,
    blocks: Vec<Block>,
    norm: Ln,
    h_cons: Lin,
    h_vow: Lin,
    h_stress: Lin,
}

type Tensors = HashMap<String, (Vec<usize>, Vec<f32>)>;

impl Tagger {
    pub fn open(path: &Path) -> io::Result<Tagger> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Tagger> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        if bytes.len() < 8 || &bytes[..4] != b"G2PT" {
            return Err(bad("not a G2PT file"));
        }
        let hl = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let h: Header = serde_json::from_slice(&bytes[8..8 + hl]).map_err(|e| bad(&e.to_string()))?;
        let data = &bytes[8 + hl..];
        let f32s = |off: usize, n: usize| -> Vec<f32> {
            data[off..off + n * 4].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect()
        };
        let mut t: Tensors = HashMap::new();
        let metas: HashMap<&str, &TensorMeta> = h.tensors.iter().map(|m| (m.name.as_str(), m)).collect();
        for m in &h.tensors {
            let n: usize = m.shape.iter().product();
            if m.dtype.as_deref() == Some("i8") {
                let sm = metas.get(format!("{}.scale", m.name).as_str()).ok_or_else(|| bad("missing scale"))?;
                let scale = f32s(sm.offset, m.shape[0]);
                let cols = n / m.shape[0];
                let raw = &data[m.offset..m.offset + n];
                let v: Vec<f32> = raw.iter().enumerate().map(|(i, &b)| (b as i8) as f32 * scale[i / cols]).collect();
                t.insert(m.name.clone(), (m.shape.clone(), v));
            } else if !m.name.ends_with(".scale") {
                t.insert(m.name.clone(), (m.shape.clone(), f32s(m.offset, n)));
            }
        }
        let mut take = |name: &str| -> io::Result<(Vec<usize>, Vec<f32>)> {
            t.remove(name).ok_or_else(|| bad(&format!("missing tensor {name}")))
        };
        let lin = |take: &mut dyn FnMut(&str) -> io::Result<(Vec<usize>, Vec<f32>)>, p: &str| -> io::Result<Lin> {
            let (s, w) = take(&format!("{p}.weight"))?;
            let (_, b) = take(&format!("{p}.bias"))?;
            Ok(Lin { w, b, o: s[0], i: s[1] })
        };
        let ln = |take: &mut dyn FnMut(&str) -> io::Result<(Vec<usize>, Vec<f32>)>, p: &str| -> io::Result<Ln> {
            Ok(Ln { g: take(&format!("{p}.weight"))?.1, b: take(&format!("{p}.bias"))?.1 })
        };
        let cfg = h.config.clone();
        let mut blocks = Vec::new();
        for l in 0..cfg.layers {
            let p = format!("enc.{l}");
            blocks.push(Block {
                ln1: ln(&mut take, &format!("{p}.ln1"))?,
                q: lin(&mut take, &format!("{p}.sa.q"))?,
                k: lin(&mut take, &format!("{p}.sa.k"))?,
                v: lin(&mut take, &format!("{p}.sa.v"))?,
                o: lin(&mut take, &format!("{p}.sa.o"))?,
                ln3: ln(&mut take, &format!("{p}.ln3"))?,
                ff1: lin(&mut take, &format!("{p}.ff1"))?,
                ff2: lin(&mut take, &format!("{p}.ff2"))?,
            });
        }
        let vocab = h.vocab.into_iter().filter_map(|(k, v)| k.chars().next().map(|c| (c, v))).collect();
        let letter_cons = h.letter_cons.into_iter().filter_map(|(k, v)| k.chars().next().map(|c| (c, v))).collect();
        Ok(Tagger {
            emb: take("emb.weight")?.1,
            pos: take("pos.weight")?.1,
            norm: ln(&mut take, "norm")?,
            h_cons: lin(&mut take, "cons")?,
            h_vow: lin(&mut take, "vow")?,
            h_stress: lin(&mut take, "stress")?,
            blocks,
            cfg,
            vocab,
            cons_vocab: h.cons_vocab,
            vow_vocab: h.vow_vocab,
            letter_cons,
        })
    }

    /// Character ids for `text` (unknown characters -> 1), truncated to the window.
    pub fn encode(&self, text: &str) -> Vec<usize> {
        text.chars().take(self.cfg.max_len).map(|c| *self.vocab.get(&c).unwrap_or(&1)).collect()
    }

    /// Run the encoder over one window of character ids.
    pub fn forward(&self, ids: &[usize]) -> Logits {
        let d = self.cfg.d;
        let l = ids.len();
        let heads = self.cfg.heads;
        let hd = d / heads;
        let scale = 1.0 / (hd as f32).sqrt();
        let mut x = vec![0f32; l * d];
        for (p, &id) in ids.iter().enumerate() {
            let (e, ps) = (&self.emb[id * d..(id + 1) * d], &self.pos[p * d..(p + 1) * d]);
            for j in 0..d {
                x[p * d + j] = e[j] + ps[j];
            }
        }
        let mut h = vec![0f32; l * d];
        let mut q = vec![0f32; l * d];
        let mut k = vec![0f32; l * d];
        let mut v = vec![0f32; l * d];
        let mut att = vec![0f32; l * d];
        let mut o = vec![0f32; l * d];
        let mut f1 = vec![0f32; l * self.cfg.ff];
        let mut w = vec![0f32; l];
        for b in &self.blocks {
            b.ln1.apply(&x, l, d, &mut h);
            b.q.apply(&h, l, &mut q);
            b.k.apply(&h, l, &mut k);
            b.v.apply(&h, l, &mut v);
            for hh in 0..heads {
                let hs = hh * hd;
                for p in 0..l {
                    let qs = &q[p * d + hs..p * d + hs + hd];
                    let mut mx = f32::MIN;
                    for t in 0..l {
                        let s = dot(qs, &k[t * d + hs..t * d + hs + hd]) * scale;
                        w[t] = s;
                        mx = mx.max(s);
                    }
                    let mut sum = 0.0;
                    for s in w.iter_mut() {
                        *s = (*s - mx).exp();
                        sum += *s;
                    }
                    let inv = 1.0 / sum;
                    let os = &mut att[p * d + hs..p * d + hs + hd];
                    os.iter_mut().for_each(|z| *z = 0.0);
                    for t in 0..l {
                        let a = w[t] * inv;
                        let vs = &v[t * d + hs..t * d + hs + hd];
                        for (oz, &vv) in os.iter_mut().zip(vs) {
                            *oz += a * vv;
                        }
                    }
                }
            }
            b.o.apply(&att, l, &mut o);
            for (xi, oi) in x.iter_mut().zip(&o) {
                *xi += oi;
            }
            b.ln3.apply(&x, l, d, &mut h);
            b.ff1.apply(&h, l, &mut f1);
            for z in f1.iter_mut() {
                *z = z.max(0.0);
            }
            b.ff2.apply(&f1, l, &mut o);
            for (xi, oi) in x.iter_mut().zip(&o) {
                *xi += oi;
            }
        }
        self.norm.apply(&x, l, d, &mut h);
        let mut cons = vec![0f32; l * self.cfg.n_cons];
        let mut vow = vec![0f32; l * self.cfg.n_vow];
        let mut stress = vec![0f32; l * 2];
        self.h_cons.apply(&h, l, &mut cons);
        self.h_vow.apply(&h, l, &mut vow);
        self.h_stress.apply(&h, l, &mut stress);
        Logits { cons, vow, stress, len: l }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiled_matmul_matches_naive() {
        let (i, o, l) = (19, 7, 11);
        let w: Vec<f32> = (0..i * o).map(|k| ((k * 7919) % 97) as f32 / 50.0 - 1.0).collect();
        let b: Vec<f32> = (0..o).map(|k| k as f32 / 10.0).collect();
        let x: Vec<f32> = (0..i * l).map(|k| ((k * 31) % 53) as f32 / 20.0 - 1.0).collect();
        let lin = Lin { w: w.clone(), b: b.clone(), i, o };
        let mut y = vec![0f32; l * o];
        lin.apply(&x, l, &mut y);
        for r in 0..l {
            for c in 0..o {
                let want: f32 = (0..i).map(|k| x[r * i + k] * w[c * i + k]).sum::<f32>() + b[c];
                assert!((y[r * o + c] - want).abs() < 1e-4, "{r},{c}: {} vs {want}", y[r * o + c]);
            }
        }
    }
}
