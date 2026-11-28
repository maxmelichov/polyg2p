//! Context readings for polyphonic characters (Mandarin, Cantonese, Japanese heteronyms).
//!
//! One shared character-level transformer encoder ([`crate::tagger`]'s encoder body) and one
//! classification head over reading classes. For every character that has more than one
//! reading, the head's logits are masked to that character's own candidate readings
//! (`char_classes` in the file header) and the arg-max reading is returned with its probability.
//! Trained by `tools/train_polyphone.py`; weight file = G2PT container, header `"kind": "polyphone"`.
//!
//! The model only *chooses among a character's listed readings*; mapping a reading to phonemes
//! (pinyin -> IPA, tone sandhi, ...) stays in the caller.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use crate::tagger::{Cfg, Encoder, Lin, read_g2pt};

#[derive(serde::Deserialize)]
struct Header {
    config: Cfg,
    vocab: HashMap<String, usize>,
    classes: Vec<String>,
    char_classes: HashMap<String, Vec<String>>,
}

/// Languages with a polyphone model. Weight files live in the data directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyLang {
    /// Mandarin; readings are pinyin with tone digit (`zhong4`, neutral tone `5`, ü as `v`)
    Zh,
    /// Cantonese; readings are Jyutping with tone digit
    Yue,
    /// Japanese heteronyms; readings are katakana
    Ja,
}

impl PolyLang {
    pub const ALL: [PolyLang; 3] = [PolyLang::Zh, PolyLang::Yue, PolyLang::Ja];

    pub fn parse(s: &str) -> Option<PolyLang> {
        match s.to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh-tw" | "zho" | "cmn" => Some(PolyLang::Zh),
            "yue" | "zh-yue" | "zh-hk" | "cantonese" => Some(PolyLang::Yue),
            "ja" | "jpn" | "ja-jp" => Some(PolyLang::Ja),
            _ => None,
        }
    }

    /// File name of the model in the data directory.
    pub fn file(self) -> &'static str {
        match self {
            PolyLang::Zh => "zh-poly.g2p",
            PolyLang::Yue => "yue-poly.g2p",
            PolyLang::Ja => "ja-poly.g2p",
        }
    }

    pub(crate) fn idx(self) -> usize {
        self as usize
    }
}

/// One decided reading: `index` is the character index (not the byte offset) in the text.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub index: usize,
    pub reading: String,
    pub prob: f32,
}

pub struct Polyphone {
    enc: Encoder,
    head: Lin,
    classes: Vec<String>,
    /// legal class ids per character
    cands: HashMap<char, Vec<usize>>,
}

impl Polyphone {
    pub fn open(path: &Path) -> io::Result<Polyphone> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Polyphone> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
        let (hv, mut w) = read_g2pt(bytes)?;
        if hv.get("kind").and_then(|k| k.as_str()) != Some("polyphone") {
            return Err(bad("not a polyphone model (header kind)"));
        }
        let h: Header = serde_json::from_value(hv).map_err(|e| bad(&e.to_string()))?;
        let id: HashMap<&str, usize> = h.classes.iter().enumerate().map(|(i, c)| (c.as_str(), i)).collect();
        let mut cands = HashMap::new();
        for (c, rs) in &h.char_classes {
            if let Some(ch) = c.chars().next() {
                let ids: Vec<usize> = rs.iter().filter_map(|r| id.get(r.as_str()).copied()).collect();
                if !ids.is_empty() {
                    cands.insert(ch, ids);
                }
            }
        }
        let enc = Encoder::load(&h.config, h.vocab, &mut w)?;
        let head = w.lin("head")?;
        Ok(Polyphone { enc, head, classes: h.classes, cands })
    }

    /// Does `c` have candidate readings in this model?
    pub fn knows(&self, c: char) -> bool {
        self.cands.contains_key(&c)
    }

    /// Readings of every known character of `text`, in text order. Texts longer than the
    /// model window are read in windows centred on each target.
    pub fn readings(&self, text: &str) -> Vec<Reading> {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let targets: Vec<usize> = (0..n).filter(|&i| self.cands.contains_key(&chars[i])).collect();
        if targets.is_empty() {
            return Vec::new();
        }
        let max_len = self.enc.cfg.max_len;
        let mut out = Vec::with_capacity(targets.len());
        if n <= max_len {
            let s: String = chars.iter().collect();
            self.run(&s, 0, &chars, &targets, &mut out);
        } else {
            for &i in &targets {
                let st = i.saturating_sub(max_len / 2).min(n - max_len);
                let s: String = chars[st..st + max_len].iter().collect();
                self.run(&s, st, &chars, &[i], &mut out);
            }
        }
        out
    }

    fn run(&self, window: &str, start: usize, chars: &[char], targets: &[usize], out: &mut Vec<Reading>) {
        let ids = self.enc.encode(window);
        let l = ids.len();
        let h = self.enc.hidden(&ids);
        let nc = self.classes.len();
        // the head is only applied to the target rows
        let d = self.enc.cfg.d;
        let mut rows = vec![0f32; targets.len() * d];
        for (k, &t) in targets.iter().enumerate() {
            let p = t - start;
            debug_assert!(p < l);
            rows[k * d..(k + 1) * d].copy_from_slice(&h[p * d..(p + 1) * d]);
        }
        let mut lg = vec![0f32; targets.len() * nc];
        self.head.apply(&rows, targets.len(), &mut lg);
        for (k, &t) in targets.iter().enumerate() {
            let cand = &self.cands[&chars[t]];
            let row = &lg[k * nc..(k + 1) * nc];
            let mx = cand.iter().map(|&c| row[c]).fold(f32::MIN, f32::max);
            let z: f32 = cand.iter().map(|&c| (row[c] - mx).exp()).sum();
            let (best, bl) = cand.iter().map(|&c| (c, row[c])).fold((cand[0], f32::MIN), |a, b| if b.1 > a.1 { b } else { a });
            out.push(Reading { index: t, reading: self.classes[best].clone(), prob: (bl - mx).exp() / z });
        }
    }
}
