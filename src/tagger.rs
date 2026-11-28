//! Pure-Rust inference for the character-level transformer **encoder tagger**
//! used by the Hebrew engine (trained by `tools/train_he.py`). Mirrors it
//! exactly: learned positions, pre-LayerNorm blocks, ReLU FFN, final LayerNorm,
//! three linear heads per position (consonant / vowel / stress).
//!
//! Weight file (`G2PT`): b"G2PT" | u32 header_len | JSON header | tensors.
//! Tensors are little-endian f32, or int8 with a per-output-row f32 scale
//! (`"dtype": "i8"`, scale tensor `<name>.scale`). On aarch64 with the dot-product
//! extension the int8 weights stay int8 and the linear layers run an int8 `sdot`
//! kernel over per-token dynamically quantized activations (4× the MACs per
//! instruction of f32 NEON); elsewhere they are dequantized at load.
//!
//! All other matrix products go through one cache-blocked `A·Bᵀ` kernel (4×4
//! register tile, NEON on aarch64, portable fallback elsewhere), including
//! attention, so one sentence costs about a millisecond single-threaded.

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
}

// ---------------------------------------------------------------- kernels

#[inline(always)]
fn dot_tail(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[inline(always)]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() - a.len() % 4;
    let mut acc = [0f32; 4];
    let mut k = 0;
    while k < n {
        for j in 0..4 {
            acc[j] += a[k + j] * b[k + j];
        }
        k += 4;
    }
    acc.iter().sum::<f32>() + dot_tail(&a[n..], &b[n..])
}

/// 4 rows of A × 4 rows of B over `k` elements (k % 4 == 0): `out[r * 4 + c] = a[r] · b[c]`.
#[inline(always)]
fn tile4x4(a: [&[f32]; 4], b: [&[f32]; 4], k: usize) -> [f32; 16] {
    #[cfg(all(target_arch = "aarch64", not(polyg2p_portable)))]
    // SAFETY: NEON is baseline on aarch64; every slice holds at least `k`
    // elements (k is a multiple of 4) and is only read.
    unsafe {
        tile4x4_neon(a, b, k)
    }
    #[cfg(any(not(target_arch = "aarch64"), polyg2p_portable))]
    tile4x4_generic(a, b, k)
}

#[cfg(all(target_arch = "aarch64", not(polyg2p_portable)))]
#[inline(always)]
unsafe fn tile4x4_neon(x: [&[f32]; 4], w: [&[f32]; 4], k: usize) -> [f32; 16] {
    use std::arch::aarch64::*;
    debug_assert!(k % 4 == 0 && x.iter().all(|s| s.len() >= k) && w.iter().all(|s| s.len() >= k));
    unsafe {
        let (mut a0, mut a1, mut a2, mut a3) = (vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0));
        let (mut a4, mut a5, mut a6, mut a7) = (vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0));
        let (mut a8, mut a9, mut a10, mut a11) = (vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0));
        let (mut a12, mut a13, mut a14, mut a15) = (vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0), vdupq_n_f32(0.0));
        let (x0, x1, x2, x3) = (x[0].as_ptr(), x[1].as_ptr(), x[2].as_ptr(), x[3].as_ptr());
        let (w0, w1, w2, w3) = (w[0].as_ptr(), w[1].as_ptr(), w[2].as_ptr(), w[3].as_ptr());
        let mut i = 0;
        while i < k {
            let p0 = vld1q_f32(x0.add(i));
            let p1 = vld1q_f32(x1.add(i));
            let p2 = vld1q_f32(x2.add(i));
            let p3 = vld1q_f32(x3.add(i));
            let q0 = vld1q_f32(w0.add(i));
            let q1 = vld1q_f32(w1.add(i));
            let q2 = vld1q_f32(w2.add(i));
            let q3 = vld1q_f32(w3.add(i));
            a0 = vfmaq_f32(a0, p0, q0);
            a1 = vfmaq_f32(a1, p0, q1);
            a2 = vfmaq_f32(a2, p0, q2);
            a3 = vfmaq_f32(a3, p0, q3);
            a4 = vfmaq_f32(a4, p1, q0);
            a5 = vfmaq_f32(a5, p1, q1);
            a6 = vfmaq_f32(a6, p1, q2);
            a7 = vfmaq_f32(a7, p1, q3);
            a8 = vfmaq_f32(a8, p2, q0);
            a9 = vfmaq_f32(a9, p2, q1);
            a10 = vfmaq_f32(a10, p2, q2);
            a11 = vfmaq_f32(a11, p2, q3);
            a12 = vfmaq_f32(a12, p3, q0);
            a13 = vfmaq_f32(a13, p3, q1);
            a14 = vfmaq_f32(a14, p3, q2);
            a15 = vfmaq_f32(a15, p3, q3);
            i += 4;
        }
        [
            vaddvq_f32(a0), vaddvq_f32(a1), vaddvq_f32(a2), vaddvq_f32(a3),
            vaddvq_f32(a4), vaddvq_f32(a5), vaddvq_f32(a6), vaddvq_f32(a7),
            vaddvq_f32(a8), vaddvq_f32(a9), vaddvq_f32(a10), vaddvq_f32(a11),
            vaddvq_f32(a12), vaddvq_f32(a13), vaddvq_f32(a14), vaddvq_f32(a15),
        ]
    }
}

#[cfg(any(not(target_arch = "aarch64"), polyg2p_portable))]
#[inline(always)]
fn tile4x4_generic(x: [&[f32]; 4], w: [&[f32]; 4], k: usize) -> [f32; 16] {
    let mut a = [[0f32; 4]; 16];
    let mut i = 0;
    while i < k {
        for j in 0..4 {
            for r in 0..4 {
                for c in 0..4 {
                    a[r * 4 + c][j] += x[r][i + j] * w[c][i + j];
                }
            }
        }
        i += 4;
    }
    let mut out = [0f32; 16];
    for j in 0..16 {
        out[j] = a[j].iter().sum();
    }
    out
}

/// Rows of B kept hot in L1 per pass over A (16 rows × k ≤ 1024 × 4 B = 64 KB).
const NB: usize = 16;

/// `out[i, j] = a[i] · b[j] + bias[j]` for `m` rows of A and `n` rows of B of
/// length `k`, with arbitrary row strides (`a[i]` starts at `i * sa`, etc.).
#[allow(clippy::too_many_arguments)]
fn matmul_abt(a: &[f32], sa: usize, b: &[f32], sb: usize, m: usize, n: usize, k: usize, bias: Option<&[f32]>, out: &mut [f32], so: usize) {
    let kmain = k - k % 4;
    let row_a = |i: usize| &a[i * sa..i * sa + k];
    let row_b = |j: usize| &b[j * sb..j * sb + k];
    let bias_at = |j: usize| bias.map_or(0.0, |bb| bb[j]);
    let mut c0 = 0;
    while c0 < n {
        let c1 = (c0 + NB).min(n);
        let mut r = 0;
        while r + 4 <= m {
            let xs = [row_a(r), row_a(r + 1), row_a(r + 2), row_a(r + 3)];
            let mut c = c0;
            while c + 4 <= c1 {
                let ws = [row_b(c), row_b(c + 1), row_b(c + 2), row_b(c + 3)];
                let s = tile4x4(xs, ws, kmain);
                for rr in 0..4 {
                    for cc in 0..4 {
                        out[(r + rr) * so + c + cc] =
                            s[rr * 4 + cc] + dot_tail(&xs[rr][kmain..], &ws[cc][kmain..]) + bias_at(c + cc);
                    }
                }
                c += 4;
            }
            while c < c1 {
                let w = row_b(c);
                for rr in 0..4 {
                    out[(r + rr) * so + c] = dot(xs[rr], w) + bias_at(c);
                }
                c += 1;
            }
            r += 4;
        }
        while r < m {
            let x = row_a(r);
            for c in c0..c1 {
                out[r * so + c] = dot(x, row_b(c)) + bias_at(c);
            }
            r += 1;
        }
        c0 = c1;
    }
}

// ------------------------------------------------------- int8 kernel (aarch64)

/// `sdot acc, a, b`: four i32 lanes each accumulating a 4-byte dot product.
/// Inline asm because the `vdotq_s32` intrinsic is not stable yet.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn sdot(acc: std::arch::aarch64::int32x4_t, a: std::arch::aarch64::int8x16_t, b: std::arch::aarch64::int8x16_t) -> std::arch::aarch64::int32x4_t {
    let mut r = acc;
    unsafe {
        std::arch::asm!(".arch_extension dotprod", "sdot {0:v}.4s, {1:v}.16b, {2:v}.16b",
            inout(vreg) r, in(vreg) a, in(vreg) b, options(pure, nomem, nostack));
    }
    r
}

/// 4 rows of A × 4 rows of B over `k` int8 elements (k % 16 == 0).
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn tile4x4_q8(x: [&[i8]; 4], w: [&[i8]; 4], k: usize) -> [i32; 16] {
    use std::arch::aarch64::*;
    debug_assert!(k % 16 == 0 && x.iter().all(|s| s.len() >= k) && w.iter().all(|s| s.len() >= k));
    unsafe {
        let z = vdupq_n_s32(0);
        let mut a = [z; 16];
        let (x0, x1, x2, x3) = (x[0].as_ptr(), x[1].as_ptr(), x[2].as_ptr(), x[3].as_ptr());
        let (w0, w1, w2, w3) = (w[0].as_ptr(), w[1].as_ptr(), w[2].as_ptr(), w[3].as_ptr());
        let mut i = 0;
        while i < k {
            let p = [vld1q_s8(x0.add(i)), vld1q_s8(x1.add(i)), vld1q_s8(x2.add(i)), vld1q_s8(x3.add(i))];
            let q = [vld1q_s8(w0.add(i)), vld1q_s8(w1.add(i)), vld1q_s8(w2.add(i)), vld1q_s8(w3.add(i))];
            for r in 0..4 {
                for c in 0..4 {
                    a[r * 4 + c] = sdot(a[r * 4 + c], p[r], q[c]);
                }
            }
            i += 16;
        }
        let mut out = [0i32; 16];
        for j in 0..16 {
            out[j] = vaddvq_s32(a[j]);
        }
        out
    }
}

#[cfg(target_arch = "aarch64")]
fn dot_q8(a: &[i8], b: &[i8]) -> i32 {
    a.iter().zip(b).map(|(&x, &y)| x as i32 * y as i32).sum()
}

/// `out[r, c] = sx[r]·sw[c]·(xq[r] · wq[c]) + bias[c]` for `m` rows of `k` int8
/// activations against `n` int8 weight rows (`k % 16 == 0`).
#[cfg(target_arch = "aarch64")]
fn matmul_q8(xq: &[i8], sx: &[f32], wq: &[i8], sw: &[f32], m: usize, n: usize, k: usize, bias: &[f32], out: &mut [f32]) {
    let row_x = |i: usize| &xq[i * k..(i + 1) * k];
    let row_w = |j: usize| &wq[j * k..(j + 1) * k];
    let mut c0 = 0;
    while c0 < n {
        let c1 = (c0 + NB).min(n);
        let mut r = 0;
        while r + 4 <= m {
            let xs = [row_x(r), row_x(r + 1), row_x(r + 2), row_x(r + 3)];
            let mut c = c0;
            while c + 4 <= c1 {
                let ws = [row_w(c), row_w(c + 1), row_w(c + 2), row_w(c + 3)];
                let s = unsafe { tile4x4_q8(xs, ws, k) };
                for rr in 0..4 {
                    for cc in 0..4 {
                        out[(r + rr) * n + c + cc] = s[rr * 4 + cc] as f32 * sx[r + rr] * sw[c + cc] + bias[c + cc];
                    }
                }
                c += 4;
            }
            while c < c1 {
                for rr in 0..4 {
                    out[(r + rr) * n + c] = dot_q8(xs[rr], row_w(c)) as f32 * sx[r + rr] * sw[c] + bias[c];
                }
                c += 1;
            }
            r += 4;
        }
        while r < m {
            for c in c0..c1 {
                out[r * n + c] = dot_q8(row_x(r), row_w(c)) as f32 * sx[r] * sw[c] + bias[c];
            }
            r += 1;
        }
        c0 = c1;
    }
}

/// Symmetric per-row int8 quantization of `l` rows of `k` activations.
#[cfg(target_arch = "aarch64")]
fn quantize_rows(x: &[f32], l: usize, k: usize) -> (Vec<i8>, Vec<f32>) {
    let mut q = vec![0i8; l * k];
    let mut s = vec![0f32; l];
    for r in 0..l {
        let row = &x[r * k..(r + 1) * k];
        let amax = row.iter().fold(0f32, |m, v| m.max(v.abs()));
        let scale = if amax > 0.0 { amax / 127.0 } else { 1.0 };
        s[r] = scale;
        let inv = 1.0 / scale;
        for (dst, v) in q[r * k..(r + 1) * k].iter_mut().zip(row) {
            *dst = (v * inv).round().clamp(-127.0, 127.0) as i8;
        }
    }
    (q, s)
}

/// Int8 weight rows with a scale per output row.
struct Q8 {
    w: Vec<i8>,
    scale: Vec<f32>,
}

/// Linear layer, weight in PyTorch layout `[out, in]`; `q` holds the int8
/// weights when the layer runs the int8 kernel (then `w` is empty).
pub(crate) struct Lin {
    w: Vec<f32>,
    q: Option<Q8>,
    b: Vec<f32>,
    i: usize,
    o: usize,
}

impl Lin {
    pub(crate) fn out_dim(&self) -> usize {
        self.o
    }

    /// `y[l, o] = x[l, i] · Wᵀ + b` for `l` rows.
    pub(crate) fn apply(&self, x: &[f32], l: usize, y: &mut [f32]) {
        debug_assert_eq!(x.len(), l * self.i);
        debug_assert_eq!(y.len(), l * self.o);
        #[cfg(target_arch = "aarch64")]
        if let Some(q) = &self.q {
            let (xq, sx) = quantize_rows(x, l, self.i);
            matmul_q8(&xq, &sx, &q.w, &q.scale, l, self.o, self.i, &self.b, y);
            return;
        }
        matmul_abt(x, self.i, &self.w, self.i, l, self.o, self.i, Some(&self.b), y, self.o);
    }
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
    enc: Encoder,
    h_cons: Lin,
    h_vow: Lin,
    h_stress: Lin,
}

type Tensors = HashMap<String, (Vec<usize>, Vec<f32>)>;

/// Does this machine run the int8 kernel? (aarch64 with the dot-product extension)
fn int8_kernel() -> bool {
    #[cfg(all(target_arch = "aarch64", not(polyg2p_portable)))]
    {
        !matches!(std::env::var("POLYG2P_INT8").as_deref(), Ok("0" | "off" | "false"))
            && std::arch::is_aarch64_feature_detected!("dotprod")
    }
    #[cfg(any(not(target_arch = "aarch64"), polyg2p_portable))]
    {
        false
    }
}

/// A parsed G2PT file: its tensors by name, ready to be taken into layers.
pub(crate) struct Bag {
    t: Tensors,
    q8: HashMap<String, Q8>,
}

fn bad(m: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, m.to_string())
}

/// Parse the `G2PT` container shared by every model in this crate (Hebrew tagger,
/// heteronym models): returns the JSON header and the tensors. Weights marked
/// `i8` stay int8 where the int8 kernel runs, and are dequantized otherwise.
pub(crate) fn load_g2pt(bytes: &[u8]) -> io::Result<(serde_json::Value, Bag)> {
    if bytes.len() < 8 || &bytes[..4] != b"G2PT" {
        return Err(bad("not a G2PT file"));
    }
    let hl = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if bytes.len() < 8 + hl {
        return Err(bad("truncated header"));
    }
    let hend = 8usize.checked_add(hl).filter(|&e| e <= bytes.len()).ok_or_else(|| bad("truncated header"))?;
    let hv: serde_json::Value = serde_json::from_slice(&bytes[8..hend]).map_err(|e| bad(&e.to_string()))?;
    let tensors: Vec<TensorMeta> =
        serde_json::from_value(hv.get("tensors").cloned().unwrap_or_default()).map_err(|e| bad(&e.to_string()))?;
    let data = &bytes[hend..];
    // every read of the weight blob is bounds-checked: a truncated or corrupt file is an error, not a panic
    let span = |off: usize, len: usize| -> io::Result<&[u8]> {
        off.checked_add(len).and_then(|e| data.get(off..e)).ok_or_else(|| bad("tensor outside the file (truncated?)"))
    };
    let f32s = |off: usize, n: usize| -> io::Result<Vec<f32>> {
        let len = n.checked_mul(4).ok_or_else(|| bad("tensor too large"))?;
        Ok(span(off, len)?.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect())
    };
    let use_q8 = int8_kernel();
    let mut t: Tensors = HashMap::new();
    let mut q8: HashMap<String, Q8> = HashMap::new();
    let metas: HashMap<&str, &TensorMeta> = tensors.iter().map(|m| (m.name.as_str(), m)).collect();
    for m in &tensors {
        let n = m.shape.iter().try_fold(1usize, |a, &d| a.checked_mul(d)).ok_or_else(|| bad("tensor too large"))?;
        if m.dtype.as_deref() == Some("i8") {
            let rows = *m.shape.first().filter(|&&r| r > 0).ok_or_else(|| bad("empty int8 tensor"))?;
            let sm = metas.get(format!("{}.scale", m.name).as_str()).ok_or_else(|| bad("missing scale"))?;
            let scale = f32s(sm.offset, rows)?;
            let cols = n / rows;
            let raw = span(m.offset, n)?;
            if use_q8 && m.shape.len() == 2 && cols % 16 == 0 {
                q8.insert(m.name.clone(), Q8 { w: raw.iter().map(|&b| b as i8).collect(), scale });
                t.insert(m.name.clone(), (m.shape.clone(), Vec::new()));
            } else {
                let v: Vec<f32> = raw.iter().enumerate().map(|(i, &b)| (b as i8) as f32 * scale[i / cols]).collect();
                t.insert(m.name.clone(), (m.shape.clone(), v));
            }
        } else if !m.name.ends_with(".scale") {
            t.insert(m.name.clone(), (m.shape.clone(), f32s(m.offset, n)?));
        }
    }
    Ok((hv, Bag { t, q8 }))
}

impl Bag {
    fn take(&mut self, name: &str) -> io::Result<(Vec<usize>, Vec<f32>, Option<Q8>)> {
        let (s, v) = self.t.remove(name).ok_or_else(|| bad(&format!("missing tensor {name}")))?;
        Ok((s, v, self.q8.remove(name)))
    }

    /// An f32 tensor (embedding table, positions, ...), as stored.
    pub(crate) fn tensor(&mut self, name: &str) -> io::Result<Vec<f32>> {
        Ok(self.take(name)?.1)
    }

    pub(crate) fn lin(&mut self, p: &str) -> io::Result<Lin> {
        let (s, w, q) = self.take(&format!("{p}.weight"))?;
        let (_, b, _) = self.take(&format!("{p}.bias"))?;
        if s.len() != 2 || b.len() != s[0] {
            return Err(bad(&format!("bad shape for {p}")));
        }
        Ok(Lin { w, q, b, o: s[0], i: s[1] })
    }

    fn ln(&mut self, p: &str) -> io::Result<Ln> {
        Ok(Ln { g: self.take(&format!("{p}.weight"))?.1, b: self.take(&format!("{p}.bias"))?.1 })
    }
}

/// The transformer encoder trunk shared by the Hebrew tagger and the heteronym
/// models: learned positions, `layers` pre-LayerNorm blocks (self-attention,
/// ReLU FFN), final LayerNorm. Tensor names `pos.weight`, `enc.{l}.*`, `norm.*`.
/// Callers embed their input themselves, add the positions with `add_pos`, run
/// the trunk, and put their own heads on the result.
pub(crate) struct Encoder {
    pos: Vec<f32>,
    blocks: Vec<Block>,
    norm: Ln,
    d: usize,
    heads: usize,
    ff: usize,
}

impl Encoder {
    pub(crate) fn from_bag(bag: &mut Bag, d: usize, heads: usize, layers: usize, ff: usize) -> io::Result<Encoder> {
        let mut blocks = Vec::new();
        for l in 0..layers {
            let p = format!("enc.{l}");
            blocks.push(Block {
                ln1: bag.ln(&format!("{p}.ln1"))?,
                q: bag.lin(&format!("{p}.sa.q"))?,
                k: bag.lin(&format!("{p}.sa.k"))?,
                v: bag.lin(&format!("{p}.sa.v"))?,
                o: bag.lin(&format!("{p}.sa.o"))?,
                ln3: bag.ln(&format!("{p}.ln3"))?,
                ff1: bag.lin(&format!("{p}.ff1"))?,
                ff2: bag.lin(&format!("{p}.ff2"))?,
            });
        }
        Ok(Encoder { pos: bag.tensor("pos.weight")?, norm: bag.ln("norm")?, blocks, d, heads, ff })
    }

    /// The pos / norm / block tensors are as large as the config says (a corrupt file must not index out of range).
    fn sizes_ok(&self, cfg: &Cfg) -> bool {
        let d = self.d;
        self.pos.len() >= cfg.max_len * d
            && self.norm.g.len() == d
            && self.norm.b.len() == d
            && self.blocks.iter().all(|b| {
                [&b.q, &b.k, &b.v, &b.o].iter().all(|l| l.i == d && l.o == d)
                    && b.ff1.i == d
                    && b.ff1.o == cfg.ff
                    && b.ff2.i == cfg.ff
                    && b.ff2.o == d
                    && b.ln1.g.len() == d
                    && b.ln1.b.len() == d
                    && b.ln3.g.len() == d
                    && b.ln3.b.len() == d
            })
    }

    /// `x[p] += pos[p]` for the `l` rows of `x` (row length `d`).
    pub(crate) fn add_pos(&self, x: &mut [f32], l: usize) {
        let d = self.d;
        for p in 0..l {
            for j in 0..d {
                x[p * d + j] += self.pos[p * d + j];
            }
        }
    }

    /// The trunk over `l` rows (`x`: `[l, d]`, positions already added); returns the normalized `[l, d]`.
    pub(crate) fn run(&self, mut x: Vec<f32>, l: usize) -> Vec<f32> {
        let (d, heads, ff) = (self.d, self.heads, self.ff);
        let hd = d / heads;
        let scale = 1.0 / (hd as f32).sqrt();
        let mut h = vec![0f32; l * d];
        let mut q = vec![0f32; l * d];
        let mut k = vec![0f32; l * d];
        let mut v = vec![0f32; l * d];
        let mut vt = vec![0f32; hd * l];
        let mut sc = vec![0f32; l * l];
        let mut att = vec![0f32; l * d];
        let mut o = vec![0f32; l * d];
        let mut f1 = vec![0f32; l * ff];
        for b in &self.blocks {
            b.ln1.apply(&x, l, d, &mut h);
            b.q.apply(&h, l, &mut q);
            b.k.apply(&h, l, &mut k);
            b.v.apply(&h, l, &mut v);
            for hh in 0..heads {
                let hs = hh * hd;
                // scores = Q_h · K_hᵀ, softmax per row
                matmul_abt(&q[hs..], d, &k[hs..], d, l, l, hd, None, &mut sc, l);
                for p in 0..l {
                    let row = &mut sc[p * l..(p + 1) * l];
                    let mut mx = f32::MIN;
                    for s in row.iter_mut() {
                        *s *= scale;
                        mx = mx.max(*s);
                    }
                    let mut sum = 0.0;
                    for s in row.iter_mut() {
                        *s = (*s - mx).exp();
                        sum += *s;
                    }
                    let inv = 1.0 / sum;
                    row.iter_mut().for_each(|s| *s *= inv);
                }
                // att_h = P · V_h  (V_h transposed to [hd, l] so it is a second A·Bᵀ)
                for t in 0..l {
                    for j in 0..hd {
                        vt[j * l + t] = v[t * d + hs + j];
                    }
                }
                matmul_abt(&sc, l, &vt, l, l, hd, l, None, &mut att[hs..], d);
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
        h
    }
}

impl Tagger {
    pub fn open(path: &Path) -> io::Result<Tagger> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Tagger> {
        let (hv, mut bag) = load_g2pt(bytes)?;
        let h: Header = serde_json::from_value(hv).map_err(|e| bad(&e.to_string()))?;
        let cfg = h.config.clone();
        if cfg.d == 0 || cfg.heads == 0 || cfg.d % cfg.heads != 0 || cfg.max_len == 0 || cfg.n_vow == 0 || cfg.n_cons == 0 {
            return Err(bad("bad config"));
        }
        let enc = Encoder::from_bag(&mut bag, cfg.d, cfg.heads, cfg.layers, cfg.ff)?;
        let vocab: HashMap<char, usize> = h.vocab.into_iter().filter_map(|(k, v)| k.chars().next().map(|c| (c, v))).collect();
        let letter_cons = h.letter_cons.into_iter().filter_map(|(k, v)| k.chars().next().map(|c| (c, v))).collect();
        let tg = Tagger {
            emb: bag.tensor("emb.weight")?,
            enc,
            h_cons: bag.lin("cons")?,
            h_vow: bag.lin("vow")?,
            h_stress: bag.lin("stress")?,
            cfg,
            vocab,
            cons_vocab: h.cons_vocab,
            vow_vocab: h.vow_vocab,
            letter_cons,
        };
        // the tensors the forward pass indexes must be as large as the config says
        let d = tg.cfg.d;
        let ok = tg.emb.len() % d == 0
            && tg.vocab.values().all(|&id| (id + 1) * d <= tg.emb.len())
            && 1 < tg.emb.len() / d
            && tg.enc.sizes_ok(&tg.cfg)
            && [&tg.h_cons, &tg.h_vow, &tg.h_stress].iter().all(|l| l.i == d)
            && tg.h_cons.o >= tg.cfg.n_cons
            && tg.h_vow.o >= tg.cfg.n_vow;
        if !ok {
            return Err(bad("tensor sizes do not match the config"));
        }
        Ok(tg)
    }

    /// Character ids for `text` (unknown characters -> 1), truncated to the window.
    pub fn encode(&self, text: &str) -> Vec<usize> {
        text.chars().take(self.cfg.max_len).map(|c| *self.vocab.get(&c).unwrap_or(&1)).collect()
    }

    /// Run the encoder over one window of character ids.
    pub fn forward(&self, ids: &[usize]) -> Logits {
        let d = self.cfg.d;
        let l = ids.len();
        let mut x = vec![0f32; l * d];
        for (p, &id) in ids.iter().enumerate() {
            x[p * d..(p + 1) * d].copy_from_slice(&self.emb[id * d..(id + 1) * d]);
        }
        self.enc.add_pos(&mut x, l);
        let h = self.enc.run(x, l);
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
    fn blocked_matmul_matches_naive() {
        for &(i, o, l) in &[(19usize, 7usize, 11usize), (64, 37, 9), (256, 40, 4), (5, 3, 1)] {
            let w: Vec<f32> = (0..i * o).map(|k| ((k * 7919) % 97) as f32 / 50.0 - 1.0).collect();
            let b: Vec<f32> = (0..o).map(|k| k as f32 / 10.0).collect();
            let x: Vec<f32> = (0..i * l).map(|k| ((k * 31) % 53) as f32 / 20.0 - 1.0).collect();
            let lin = Lin { w: w.clone(), q: None, b: b.clone(), i, o };
            let mut y = vec![0f32; l * o];
            lin.apply(&x, l, &mut y);
            for r in 0..l {
                for c in 0..o {
                    let want: f32 = (0..i).map(|k| x[r * i + k] * w[c * i + k]).sum::<f32>() + b[c];
                    assert!((y[r * o + c] - want).abs() < 1e-3, "{i},{o},{l} @ {r},{c}: {} vs {want}", y[r * o + c]);
                }
            }
        }
        // strided (attention-style) use: rows of length 8 inside stride-16 buffers, output offset
        let a: Vec<f32> = (0..5 * 16).map(|k| (k % 7) as f32 - 3.0).collect();
        let bm: Vec<f32> = (0..6 * 16).map(|k| (k % 5) as f32 - 2.0).collect();
        let mut out = vec![0f32; 5 * 10];
        matmul_abt(&a[4..], 16, &bm[2..], 16, 5, 6, 8, None, &mut out[3..], 10);
        for r in 0..5 {
            for c in 0..6 {
                let want: f32 = (0..8).map(|k| a[r * 16 + 4 + k] * bm[c * 16 + 2 + k]).sum();
                assert!((out[r * 10 + 3 + c] - want).abs() < 1e-4);
            }
        }
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    #[cfg(target_arch = "aarch64")]
    #[test]
    fn int8_matmul_matches_f32() {
        let (m, n, k) = (7, 13, 192);
        let x: Vec<f32> = (0..m * k).map(|i| ((i * 37 % 101) as f32 / 50.0 - 1.0) * 3.0).collect();
        let w: Vec<f32> = (0..n * k).map(|i| ((i * 53 % 97) as f32 / 48.0 - 1.0) * 0.1).collect();
        let b: Vec<f32> = (0..n).map(|i| i as f32 * 0.01).collect();
        // per-row weight quantization, as the exporter does
        let mut wq = vec![0i8; n * k];
        let mut sw = vec![0f32; n];
        for j in 0..n {
            let row = &w[j * k..(j + 1) * k];
            let amax = row.iter().fold(0f32, |a, v| a.max(v.abs()));
            sw[j] = amax / 127.0;
            for (d, v) in wq[j * k..(j + 1) * k].iter_mut().zip(row) {
                *d = (v / sw[j]).round() as i8;
            }
        }
        let mut want = vec![0f32; m * n];
        matmul_abt(&x, k, &w, k, m, n, k, Some(&b), &mut want, n);
        let (xq, sx) = quantize_rows(&x, m, k);
        let mut got = vec![0f32; m * n];
        matmul_q8(&xq, &sx, &wq, &sw, m, n, k, &b, &mut got);
        let scale = want.iter().fold(0f32, |a, v| a.max(v.abs()));
        for (g, w) in got.iter().zip(&want) {
            assert!((g - w).abs() < 0.02 * scale, "{g} vs {w}");
        }
    }

    #[test]
    #[ignore]
    fn kernel_gflops() {
        for &(l, i, o) in &[(135usize, 256usize, 256usize), (135, 256, 1024), (135, 1024, 256), (135, 64, 135)] {
            let w: Vec<f32> = (0..i * o).map(|k| ((k * 7919) % 97) as f32 / 50.0 - 1.0).collect();
            let b = vec![0f32; o];
            let x: Vec<f32> = (0..i * l).map(|k| ((k * 31) % 53) as f32 / 20.0 - 1.0).collect();
            let lin = Lin { w, q: None, b, i, o };
            let mut y = vec![0f32; l * o];
            let reps = 50;
            let t0 = std::time::Instant::now();
            for _ in 0..reps {
                lin.apply(&x, l, &mut y);
            }
            let dt = t0.elapsed().as_secs_f64() / reps as f64;
            eprintln!("{l}x{i}x{o}: {:.3} ms  {:.1} GFLOP/s", dt * 1e3, 2.0 * (l * i * o) as f64 / dt / 1e9);
        }
    }
}
