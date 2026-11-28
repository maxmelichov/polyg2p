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

/// The units of a sentence and, for each `Out`, the index of its word unit (`usize::MAX` for an `Out` with no source word).
pub fn units(outs: &[Out]) -> (Vec<Unit>, Vec<usize>) {
    let mut u = Vec::with_capacity(outs.len() * 2);
    let mut at = vec![usize::MAX; outs.len()];
    for (k, o) in outs.iter().enumerate() {
        if o.src.is_empty() {
            continue;
        }
        at[k] = u.len();
        u.push(Unit { text: o.src.clone(), cap: o.cap, word: true });
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

/// A loaded heteronym model, bound to a reading table.
pub struct Model {
    cfg: Cfg,
    emb: Vec<f32>,
    enc: Encoder,
    out: Lin,
    /// covered word -> (output row, pronunciation)
    rows: HashMap<String, Vec<(usize, String)>>,
}

impl Model {
    pub fn open(path: &Path, table: &Table) -> io::Result<Model> {
        Self::from_bytes(&std::fs::read(path)?, table)
    }

    pub fn from_bytes(bytes: &[u8], table: &Table) -> io::Result<Model> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        let (hv, mut bag) = load_g2pt(bytes)?;
        if hv.get("kind").and_then(|k| k.as_str()) != Some("homograph") || hv.get("features").and_then(|k| k.as_str()) != Some("v1") {
            return Err(bad("not a homograph v1 model"));
        }
        let cfg: Cfg = serde_json::from_value(hv["config"].clone()).map_err(|e| bad(&e.to_string()))?;
        let readings: Vec<String> = serde_json::from_value(hv["readings"].clone()).map_err(|e| bad(&e.to_string()))?;
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
        let emb = bag.tensor("emb.weight")?;
        if emb.len() != cfg.n_buckets * cfg.d {
            return Err(bad("embedding size does not match the config"));
        }
        let enc = Encoder::from_bag(&mut bag, cfg.d, cfg.heads, cfg.layers, cfg.ff)?;
        let out = bag.lin("out")?;
        Ok(Model { cfg, emb, enc, out, rows: by_word })
    }

    pub fn covers(&self, word: &str) -> bool {
        self.rows.contains_key(word)
    }

    /// Masked logits of the target word's readings: (pronunciation, logit) in table order.
    pub fn scores(&self, units: &[Unit], target: usize) -> Option<Vec<(&str, f32)>> {
        let rows = self.rows.get(units.get(target)?.text.as_str())?;
        let (d, max_len) = (self.cfg.d, self.cfg.max_len);
        let start = if units.len() > max_len { target.saturating_sub(max_len / 2).min(units.len() - max_len) } else { 0 };
        let win = &units[start..units.len().min(start + max_len)];
        let l = win.len();
        let mut x = vec![0f32; l * d];
        for (p, u) in win.iter().enumerate() {
            for id in feature_ids(u, start + p == target, self.cfg.n_buckets) {
                let e = &self.emb[id * d..(id + 1) * d];
                for j in 0..d {
                    x[p * d + j] += e[j];
                }
            }
        }
        self.enc.add_pos(&mut x, l);
        let h = self.enc.run(x, l);
        let mut logits = vec![0f32; self.out.out_dim()];
        self.out.apply(&h[(target - start) * d..(target - start + 1) * d], 1, &mut logits);
        Some(rows.iter().map(|(i, p)| (p.as_str(), logits[*i])).collect())
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
