//! Sentence-level heteronym model: which reading of "lead", "read", "wind", ... a
//! word has in its sentence. Language-neutral; only the reading table and the unit
//! / feature extractors are per language (English: `data/en_homograph.tsv`, word units).
//!
//! **Input.** The sentence as the engine sees it after tokenization and number
//! expansion: one unit per output word (lower-case source word, `^` in front when the
//! source word was capitalised) and one unit per punctuation mark that follows it.
//! `units` builds them from the `Out`s of a sentence; `tools/train_homograph.py` gets
//! the very same units from `en_homograph --units`, so training and inference cannot
//! drift. At most `max_len` units are used, centred on the target when longer.
//!
//! **Embedding** (fastText style, no vocabulary file): the sum of hashed feature
//! vectors per unit (`feature_ids`), plus a learned position. Features are
//! `w=<word>`, `p2= p3=` prefixes and `s1= s2= s3= s4=` suffixes (only when the word
//! is longer than the affix, counted in characters), `cap` for a capitalised word and
//! `T` on the target unit only. Hash: FNV-1a 64 over the UTF-8 bytes of the feature
//! string, modulo `n_buckets`; the Python trainer uses the same function.
//!
//! **Network.** The trunk of `tagger::Encoder` (the Hebrew tagger's block) and one
//! linear head read at the target position, one logit per row of the reading table,
//! masked to the rows of the target word.
//!
//! **File** (`<lang>_homograph.g2p`, `G2PT` container, see `tagger.rs`): header
//! `{"magic":"G2PT","kind":"homograph","lang":"en","features":"v1","config":{d,heads,
//! layers,ff,max_len,n_buckets,n_out},"readings":["word\treading",...],"tensors":[...]}`;
//! tensors `emb.weight [n_buckets,d]`, `pos.weight`, `enc.{l}.*`, `norm.*`,
//! `out.{weight,bias}`. The file names the table rows; the pronunciations come from the
//! committed table, never from the file. A word whose readings the table does not
//! know all of is not covered and falls back to the rules.
//!
//! The file is optional and never shipped with the crate: without it every language
//! is exactly what it was.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use memmap2::Mmap;

use crate::Out;
use crate::tagger::{Encoder, Lin, load_g2pt};

/// A word of the table: its readings as (name, pronunciation), in file order.
pub struct Table {
    words: HashMap<String, Vec<(String, String)>>,
}

impl Table {
    /// Parse `word<TAB>reading<TAB>class<TAB>ipa` lines (`#` starts a comment).
    pub fn parse(text: &str) -> Table {
        let mut words: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for l in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let mut f = l.split('\t');
            if let (Some(w), Some(r), Some(_class), Some(ipa)) = (f.next(), f.next(), f.next(), f.next()) {
                words.entry(w.to_string()).or_default().push((r.to_string(), ipa.to_string()));
            }
        }
        Table { words }
    }

    pub fn ipa(&self, word: &str, reading: &str) -> Option<&str> {
        self.words.get(word)?.iter().find(|(r, _)| r == reading).map(|(_, p)| p.as_str())
    }
}

/// One unit of the model's input.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    /// lower-case word, or one punctuation character
    pub text: String,
    pub cap: bool,
    /// a word (not punctuation)
    pub word: bool,
}

impl Unit {
    /// The unit as `en_homograph --units` prints it: `^` in front of a capitalised word.
    pub fn display(&self) -> String {
        if self.cap { format!("^{}", self.text) } else { self.text.clone() }
    }
}

/// The parts of a hyphenated source word (`drug-addict`), empty parts dropped; a word without a hyphen is one part.
pub fn hyphen_parts(src: &str) -> impl Iterator<Item = &str> {
    src.split('-').filter(|p| !p.is_empty())
}

/// The units of a sentence and, for each `Out`, the index of its first word unit (`usize::MAX` for an `Out` with no source word).
/// A hyphenated word becomes one word unit per part with a `-` unit between the parts, so the part at position `i` has unit
/// index `at + 2 * i`; this lets the model read a table word inside a compound ("drug-addict").
pub fn units(outs: &[Out]) -> (Vec<Unit>, Vec<usize>) {
    let mut u = Vec::with_capacity(outs.len() * 2);
    let mut at = vec![usize::MAX; outs.len()];
    for (k, o) in outs.iter().enumerate() {
        if o.src.is_empty() {
            continue;
        }
        at[k] = u.len();
        for (i, part) in hyphen_parts(&o.src).enumerate() {
            if i > 0 {
                u.push(Unit { text: "-".to_string(), cap: false, word: false });
            }
            u.push(Unit { text: part.to_string(), cap: o.cap && i == 0, word: true });
        }
        for c in o.punct.chars() {
            u.push(Unit { text: c.to_string(), cap: false, word: false });
        }
    }
    (u, at)
}

pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Hash bucket ids of the features of one unit (`features: "v1"`).
pub fn feature_ids(unit: &Unit, target: bool, n_buckets: usize) -> Vec<usize> {
    let ch: Vec<char> = unit.text.chars().collect();
    let n = ch.len();
    let sub = |a: usize, b: usize| ch[a..b].iter().collect::<String>();
    let mut f = vec![format!("w={}", unit.text)];
    for k in [2, 3] {
        if n > k {
            f.push(format!("p{k}={}", sub(0, k)));
        }
    }
    for k in [1, 2, 3, 4] {
        if n > k {
            f.push(format!("s{k}={}", sub(n - k, n)));
        }
    }
    if unit.cap {
        f.push("cap".into());
    }
    if target {
        f.push("T".into());
    }
    f.iter().map(|s| (fnv1a(s.as_bytes()) % n_buckets as u64) as usize).collect()
}

#[derive(serde::Deserialize, Clone)]
struct Cfg {
    d: usize,
    heads: usize,
    layers: usize,
    ff: usize,
    max_len: usize,
    n_buckets: usize,
}

/// Features `v2` (the fast "bag" model): hashed feature strings around the target unit, summed embeddings, one hidden layer.
/// The strings and their order are those of `bag_strings` in tools/train_homograph2.py (documented there).
const BAG_NF: usize = 112;

fn bag_features(units: &[Unit], t: usize, mut f: impl FnMut(&str)) {
    use std::fmt::Write;
    let n = units.len() as isize;
    let ti = t as isize;
    let low = |j: isize| -> &str { if j < 0 { "<s>" } else if j >= n { "</s>" } else { units[j as usize].text.as_str() } };
    let isw = |j: isize| j >= 0 && j < n && units[j as usize].text.chars().next().is_some_and(|c| c.is_alphabetic());
    let cap = |j: isize| j >= 0 && j < n && units[j as usize].cap;
    let mut buf = String::with_capacity(96);
    let mut count = 0usize;
    macro_rules! emit {
        ($($arg:tt)*) => {{
            if count < BAG_NF {
                buf.clear();
                let _ = write!(buf, $($arg)*);
                f(&buf);
                count += 1;
            }
        }};
    }
    for o in -4isize..=4 {
        let j = ti + o;
        let x = low(j);
        emit!("{o}|w={x}");
        if (1..=2).contains(&o.abs()) && j >= 0 && j < n {
            let nch = x.chars().count();
            for k in [2usize, 3] {
                if nch > k {
                    let suf: String = x.chars().skip(nch - k).collect();
                    emit!("{o}|s{k}={suf}");
                }
            }
        }
        if cap(j) {
            emit!("{o}|cap");
        }
    }
    emit!("b1|{}|{}", low(ti - 2), low(ti - 1));
    emit!("b2|{}|{}", low(ti - 1), low(ti + 1));
    emit!("b3|{}|{}", low(ti + 1), low(ti + 2));
    let first = t == 0;
    let last = !(ti + 1..n).any(isw);
    if first {
        emit!("first");
    }
    if last {
        emit!("last");
    }
    match units.last().map(|u| u.text.as_str()) {
        Some("?") => emit!("endq"),
        Some("!") => emit!("endx"),
        _ => {}
    }
    let tw = low(ti);
    let lo = (ti - 12).max(0);
    let hi = (ti + 13).min(n);
    for j in lo..ti {
        if isw(j) {
            emit!("L|{}", low(j));
        }
    }
    for j in ti + 1..hi {
        if isw(j) {
            emit!("R|{}", low(j));
        }
    }
    for o in [-2isize, -1, 1, 2] {
        emit!("t={tw}|{o}|{}", low(ti + o));
    }
    if first {
        emit!("t={tw}|first");
    }
    if last {
        emit!("t={tw}|last");
    }
    for j in lo..ti {
        if isw(j) {
            emit!("t={tw}|L|{}", low(j));
        }
    }
    for j in ti + 1..hi {
        if isw(j) {
            emit!("t={tw}|R|{}", low(j));
        }
    }
}

#[derive(serde::Deserialize)]
struct BagCfg {
    d: usize,
    h: usize,
    n_buckets: usize,
}

#[derive(serde::Deserialize)]
struct Meta {
    name: String,
    offset: usize,
}

/// The bag network. The embedding table stays in the memory-mapped file (int8 rows, one f32 scale per row) and is read row by
/// row; the two small dense layers are loaded as f32.
struct BagNet {
    map: Mmap,
    data0: usize,
    emb: usize,
    scale: usize,
    cfg: BagCfg,
    l1: Vec<f32>,
    b1: Vec<f32>,
    out: Vec<f32>,
    bo: Vec<f32>,
}

impl BagNet {
    fn from_map(map: Mmap, hv: &serde_json::Value, data0: usize) -> io::Result<BagNet> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        let cfg: BagCfg = serde_json::from_value(hv["config"].clone()).map_err(|e| bad(&e.to_string()))?;
        let metas: Vec<Meta> = serde_json::from_value(hv["tensors"].clone()).map_err(|e| bad(&e.to_string()))?;
        let off = |name: &str| metas.iter().find(|m| m.name == name).map(|m| data0 + m.offset).ok_or_else(|| bad(&format!("missing tensor {name}")));
        let f32s = |name: &str, n: usize| -> io::Result<Vec<f32>> {
            let o = off(name)?;
            let b = map.get(o..o + n * 4).ok_or_else(|| bad("truncated tensor"))?;
            Ok(b.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect())
        };
        let n_out = hv["readings"].as_array().map_or(0, |a| a.len());
        let (emb, scale) = (off("emb.weight")?, off("emb.weight.scale")?);
        if map.len() < emb + cfg.n_buckets * cfg.d || map.len() < scale + cfg.n_buckets * 4 {
            return Err(bad("embedding does not fit the file"));
        }
        let l1 = f32s("l1.weight", cfg.h * cfg.d)?;
        let b1 = f32s("l1.bias", cfg.h)?;
        let out = f32s("out.weight", n_out * cfg.h)?;
        let bo = f32s("out.bias", n_out)?;
        Ok(BagNet { map, data0, emb, scale, cfg, l1, b1, out, bo })
    }

    /// Logits of the given output rows for the unit at `t`.
    fn logits(&self, units: &[Unit], t: usize, rows: &[usize]) -> Vec<f32> {
        let (d, h, nb) = (self.cfg.d, self.cfg.h, self.cfg.n_buckets as u64);
        let mut x = vec![0f32; d];
        let m = &self.map[..];
        bag_features(units, t, |s| {
            let id = (fnv1a(s.as_bytes()) % nb) as usize;
            let sc = f32::from_le_bytes(m[self.scale + id * 4..self.scale + id * 4 + 4].try_into().unwrap());
            let row = &m[self.emb + id * d..self.emb + (id + 1) * d];
            for j in 0..d {
                x[j] += (row[j] as i8) as f32 * sc;
            }
        });
        let _ = self.data0;
        let mut hid = vec![0f32; h];
        for (k, hk) in hid.iter_mut().enumerate() {
            let w = &self.l1[k * d..(k + 1) * d];
            let mut a = self.b1[k];
            for j in 0..d {
                a += w[j] * x[j];
            }
            *hk = a.max(0.0);
        }
        rows.iter()
            .map(|&r| {
                let w = &self.out[r * h..(r + 1) * h];
                self.bo[r] + w.iter().zip(&hid).map(|(a, b)| a * b).sum::<f32>()
            })
            .collect()
    }
}

enum Net {
    Trunk { cfg: Cfg, emb: Vec<f32>, enc: Encoder, out: Lin },
    Bag(BagNet),
}

/// A loaded heteronym model, bound to a reading table.
pub struct Model {
    net: Net,
    /// covered word -> (output row, pronunciation)
    rows: HashMap<String, Vec<(usize, String)>>,
}

impl Model {
    pub fn open(path: &Path, table: &Table) -> io::Result<Model> {
        let file = std::fs::File::open(path)?;
        // SAFETY: the file is only read; a concurrent truncation is outside this crate's contract (as for the lexicon).
        let map = unsafe { Mmap::map(&file)? };
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        if map.len() >= 8 && &map[..4] == b"G2PT" {
            let hl = u32::from_le_bytes(map[4..8].try_into().unwrap()) as usize;
            if map.len() >= 8 + hl {
                let hv: serde_json::Value = serde_json::from_slice(&map[8..8 + hl]).map_err(|e| bad(&e.to_string()))?;
                if hv.get("features").and_then(|k| k.as_str()) == Some("v2") && hv.get("kind").and_then(|k| k.as_str()) == Some("homograph") {
                    let readings: Vec<String> = serde_json::from_value(hv["readings"].clone()).map_err(|e| bad(&e.to_string()))?;
                    let rows = Self::group(&readings, table)?;
                    let net = BagNet::from_map(map, &hv, 8 + hl)?;
                    return Ok(Model { net: Net::Bag(net), rows });
                }
            }
        }
        Self::from_bytes(&map, table)
    }

    fn group(readings: &[String], table: &Table) -> io::Result<HashMap<String, Vec<(usize, String)>>> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        // group the rows by word; a word is covered only when the table knows every one of its readings
        let mut by_word: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        let mut unknown: std::collections::HashSet<String> = Default::default();
        for (i, r) in readings.iter().enumerate() {
            let (w, name) = r.split_once('\t').ok_or_else(|| bad("bad reading name"))?;
            match table.ipa(w, name) {
                Some(p) => by_word.entry(w.to_string()).or_default().push((i, p.to_string())),
                None => {
                    unknown.insert(w.to_string());
                }
            }
        }
        by_word.retain(|w, rows| !unknown.contains(w) && rows.len() >= 2);
        Ok(by_word)
    }

    pub fn from_bytes(bytes: &[u8], table: &Table) -> io::Result<Model> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        let (hv, mut bag) = load_g2pt(bytes)?;
        if hv.get("kind").and_then(|k| k.as_str()) != Some("homograph") || hv.get("features").and_then(|k| k.as_str()) != Some("v1") {
            return Err(bad("not a homograph v1 model"));
        }
        let cfg: Cfg = serde_json::from_value(hv["config"].clone()).map_err(|e| bad(&e.to_string()))?;
        let readings: Vec<String> = serde_json::from_value(hv["readings"].clone()).map_err(|e| bad(&e.to_string()))?;
        let by_word = Self::group(&readings, table)?;
        let emb = bag.tensor("emb.weight")?;
        if emb.len() != cfg.n_buckets * cfg.d {
            return Err(bad("embedding size does not match the config"));
        }
        let enc = Encoder::from_bag(&mut bag, cfg.d, cfg.heads, cfg.layers, cfg.ff)?;
        let out = bag.lin("out")?;
        Ok(Model { net: Net::Trunk { cfg, emb, enc, out }, rows: by_word })
    }

    pub fn covers(&self, word: &str) -> bool {
        self.rows.contains_key(word)
    }

    /// Masked logits of the target word's readings: (pronunciation, logit) in table order.
    pub fn scores(&self, units: &[Unit], target: usize) -> Option<Vec<(&str, f32)>> {
        let rows = self.rows.get(units.get(target)?.text.as_str())?;
        match &self.net {
            Net::Bag(b) => {
                let idx: Vec<usize> = rows.iter().map(|(i, _)| *i).collect();
                let lg = b.logits(units, target, &idx);
                Some(rows.iter().zip(lg).map(|((_, p), l)| (p.as_str(), l)).collect())
            }
            Net::Trunk { cfg, emb, enc, out } => {
                let (d, max_len) = (cfg.d, cfg.max_len);
                let start = if units.len() > max_len { target.saturating_sub(max_len / 2).min(units.len() - max_len) } else { 0 };
                let win = &units[start..units.len().min(start + max_len)];
                let l = win.len();
                let mut x = vec![0f32; l * d];
                for (p, u) in win.iter().enumerate() {
                    for id in feature_ids(u, start + p == target, cfg.n_buckets) {
                        let e = &emb[id * d..(id + 1) * d];
                        for j in 0..d {
                            x[p * d + j] += e[j];
                        }
                    }
                }
                enc.add_pos(&mut x, l);
                let h = enc.run(x, l);
                let mut logits = vec![0f32; out.out_dim()];
                out.apply(&h[(target - start) * d..(target - start + 1) * d], 1, &mut logits);
                Some(rows.iter().map(|(i, p)| (p.as_str(), logits[*i])).collect())
            }
        }
    }

    /// The pronunciation the model picks for the word at unit `target`, when it covers the word.
    pub fn pick(&self, units: &[Unit], target: usize) -> Option<&str> {
        self.scores(units, target)?.into_iter().fold(None, |best: Option<(&str, f32)>, s| match best {
            Some(b) if b.1 >= s.1 => Some(b),
            _ => Some(s),
        }).map(|(p, _)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_known_values() {
        // reference values of 64-bit FNV-1a
        assert_eq!(fnv1a(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn features_follow_the_spec() {
        let w = |t: &str, cap| Unit { text: t.into(), cap, word: true };
        // "lead": w, p2, p3, s1..s3 (s4 needs more than four letters), plus T
        assert_eq!(feature_ids(&w("lead", false), true, 1 << 20).len(), 1 + 2 + 3 + 1);
        assert_eq!(feature_ids(&w("a", false), false, 1 << 20).len(), 1);
        assert_eq!(feature_ids(&w("Lead", true), false, 1 << 20).len(), 1 + 2 + 3 + 1);
        // characters, not bytes
        assert_eq!(feature_ids(&w("été", false), false, 1 << 20).len(), 1 + 1 + 2);
        assert!(feature_ids(&w("lead", false), false, 1000).iter().all(|&i| i < 1000));
    }

    #[test]
    fn table_parses() {
        let t = Table::parse("# c\nlead\tverb\tV\tlˈiːd\nlead\tmetal\tN\tlˈɛd\n");
        assert_eq!(t.ipa("lead", "metal"), Some("lˈɛd"));
        assert_eq!(t.ipa("lead", "x"), None);
    }
}
