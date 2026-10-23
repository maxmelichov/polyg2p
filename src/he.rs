//! Hebrew: context-based niqqud/stress prediction + rules to IPA.
//!
//! Hebrew is written without vowels, so the engine is a sentence-level model: a
//! small character transformer encoder ([`crate::tagger`], trained by
//! `tools/train_he.py` on labels distilled from RenikudPlus) reads the whole
//! window and predicts, per Hebrew letter, the consonant realization (25
//! classes: dagesh / shin-sin / begadkefat / silent …), the vowel (a e i o u ∅)
//! and the word stress. The labels are then rendered to IPA with exactly the
//! rules RenikudPlus uses (`_phonemize_window`), so the output notation is the
//! one BlueTTS's Hebrew models were trained on.
//!
//! Pipeline for `phonemize(text)`:
//! 1. number front end (`he::num`, a port of `hebrew-num2words`),
//! 2. grapheme normalization (geresh/gershayim variants -> `'` / `"`),
//! 3. NFD + strip niqqud / cantillation,
//! 4. split into windows of at most [`WINDOW`] characters on sentence, then
//!    clause, then word boundaries (lossless concatenation),
//! 5. per window: tag -> constrained decode (letter-legal consonants, one
//!    stress per word on a vowel-bearing letter) -> render.
//!
//! Non-Hebrew characters pass through unchanged (digits are expanded first);
//! Latin-script words can be routed to a callback with [`Hebrew::with_latin`].
//! The model file is `he.g2p` in the data directory (see docs/hebrew.md).

use std::path::Path;
use std::sync::OnceLock;

use unicode_normalization::UnicodeNormalization;

use crate::tagger::{Logits, Tagger};

pub mod num;

pub const IMPLEMENTED: bool = true;

/// Decode window in characters (the student is trained on sentences of at most
/// 300 characters; longer text is split like RenikudPlus's `split_for_decode`).
pub const WINDOW: usize = 300;

const B36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

pub fn is_hebrew(c: char) -> bool {
    ('א'..='ת').contains(&c)
}

/// Niqqud, dagesh, shin/sin dots and cantillation (not maqaf / paseq / sof pasuq).
fn is_niqqud(c: char) -> bool {
    matches!(c as u32, 0x0591..=0x05BD | 0x05BF | 0x05C1 | 0x05C2 | 0x05C4 | 0x05C5 | 0x05C7)
}

/// `normalize_graphemes` + NFD + `strip_niqqud` of RenikudPlus.
pub fn normalize(text: &str) -> String {
    let t: String = text.chars().map(|c| match c {
        '׳' | '`' | '´' => '\'',
        '״' => '"',
        c => c,
    }).collect();
    t.nfd().filter(|&c| !is_niqqud(c)).collect()
}

// ---------------------------------------------------------------- windowing (port of split_for_decode)
const SENTENCE_ENDS: &[char] = &['.', '!', '?', ';', '\n', '\r', '…', '׃'];
const CLAUSE_ENDS: &[char] = &[',', ':', '–', '—'];

fn split_after(text: &[char], terms: &[char]) -> Vec<Vec<char>> {
    let mut pieces = Vec::new();
    let (mut start, mut i) = (0, 0);
    let n = text.len();
    while i < n {
        if terms.contains(&text[i]) {
            let mut j = i + 1;
            while j < n && terms.contains(&text[j]) {
                j += 1;
            }
            while j < n && text[j].is_whitespace() {
                j += 1;
            }
            pieces.push(text[start..j].to_vec());
            start = j;
            i = j;
        } else {
            i += 1;
        }
    }
    if start < n {
        pieces.push(text[start..].to_vec());
    }
    pieces
}

fn split_words(text: &[char]) -> Vec<Vec<char>> {
    let mut pieces = Vec::new();
    let (mut start, mut i) = (0, 0);
    let n = text.len();
    while i < n {
        if text[i].is_whitespace() {
            let mut j = i;
            while j < n && text[j].is_whitespace() {
                j += 1;
            }
            pieces.push(text[start..j].to_vec());
            start = j;
            i = j;
        } else {
            i += 1;
        }
    }
    if start < n {
        pieces.push(text[start..].to_vec());
    }
    pieces
}

fn pack(pieces: Vec<Vec<char>>, limit: usize) -> Vec<Vec<char>> {
    let mut out = Vec::new();
    let mut cur: Vec<char> = Vec::new();
    for p in pieces {
        if !cur.is_empty() && cur.len() + p.len() > limit {
            out.push(std::mem::take(&mut cur));
        }
        cur.extend(p);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Split into decode windows of at most `limit` characters; the pieces are
/// contiguous, so their concatenation is the input.
pub fn split_for_decode(text: &[char], limit: usize) -> Vec<Vec<char>> {
    if text.len() <= limit {
        return vec![text.to_vec()];
    }
    let mut out = Vec::new();
    for sent in pack(split_after(text, SENTENCE_ENDS), limit) {
        if sent.len() <= limit {
            out.push(sent);
            continue;
        }
        for clause in pack(split_after(&sent, CLAUSE_ENDS), limit) {
            if clause.len() <= limit {
                out.push(clause);
                continue;
            }
            for words in pack(split_words(&clause), limit) {
                if words.len() <= limit {
                    out.push(words);
                } else {
                    out.extend(words.chunks(limit).map(|c| c.to_vec()));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- labels and rendering

/// One Hebrew letter's prediction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Label {
    pub cons: usize,
    pub vow: usize,
    pub stress: bool,
}

/// Consonant / vowel class names in the teacher's order.
pub const CONS: [&str; 25] = [
    "∅", "b", "v", "d", "h", "z", "χ", "t", "j", "k", "l", "m", "n", "s", "f", "p", "ts", "tʃ", "w", "ʔ", "ɡ", "ʁ", "ʃ", "ʒ",
    "dʒ",
];
pub const VOW: [&str; 6] = ["∅", "a", "e", "i", "o", "u"];

/// Legal consonant classes per letter (first = most common).
pub fn letter_cons(c: char) -> &'static [usize] {
    match c {
        'א' | 'ע' => &[19, 0],
        'ב' => &[1, 2],
        'ג' => &[20, 24],
        'ד' => &[3],
        'ה' => &[4, 0],
        'ו' => &[2, 18, 0],
        'ז' => &[5, 23],
        'ח' => &[6],
        'ט' | 'ת' => &[7],
        'י' => &[8, 0],
        'כ' | 'ך' => &[9, 6],
        'ל' => &[10, 0],
        'מ' | 'ם' => &[11],
        'נ' | 'ן' => &[12],
        'ס' => &[13],
        'פ' | 'ף' => &[15, 14],
        'צ' | 'ץ' => &[16, 17],
        'ק' => &[9],
        'ר' => &[21],
        'ש' => &[22, 13, 0],
        _ => &[0],
    }
}

fn geresh(c: char) -> Option<&'static str> {
    match c {
        'ץ' | 'צ' => Some("tʃ"),
        'ג' => Some("dʒ"),
        'ז' => Some("ʒ"),
        _ => None,
    }
}

/// Render one window from per-character labels (`None` on non-Hebrew
/// positions), mirroring RenikudPlus `_phonemize_window`. `latin` rewrites
/// runs of non-Hebrew letters.
pub fn render(text: &[char], labels: &[Option<Label>], latin: Option<&LatinFn>) -> String {
    let n = text.len();
    let mut out = String::with_capacity(n * 2);
    let mut i = 0;
    while i < n {
        let ch = text[i];
        let Some(lab) = labels[i] else {
            if ch == '\'' || ch == '"' {
                i += 1;
                continue;
            }
            if let Some(f) = latin {
                if ch.is_alphabetic() {
                    let st = i;
                    while i < n && (text[i].is_alphabetic() && !is_hebrew(text[i]) || text[i] == '\'' && i + 1 < n && text[i + 1].is_alphabetic() && !is_hebrew(text[i + 1])) {
                        i += 1;
                    }
                    let w: String = text[st..i].iter().collect();
                    out.push_str(&f(&w));
                    continue;
                }
            }
            out.push(ch);
            i += 1;
            continue;
        };
        let mut cons = CONS[lab.cons];
        let vow = VOW[lab.vow];
        if i + 1 < n && text[i + 1] == '\'' {
            if let Some(g) = geresh(ch) {
                cons = g;
            }
        }
        let word_final = i + 1 >= n || !text[i + 1].is_alphabetic();
        if ch == 'ח' && word_final && vow == "a" {
            if lab.stress {
                out.push('ˈ');
            }
            out.push_str("aχ");
        } else {
            if lab.cons != 0 {
                out.push_str(cons);
            }
            if lab.stress && lab.vow != 0 {
                out.push('ˈ');
            }
            if lab.vow != 0 {
                out.push_str(vow);
            }
        }
        i += 1;
    }
    out
}

/// Render from the label strings of `tools/he_distill.py` (`c` base-36 consonant
/// ids, `v` vowel ids, `s` stress flags, `-` on non-Hebrew positions).
pub fn render_codes(text: &str, c: &str, v: &str, s: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let labels: Vec<Option<Label>> = c
        .chars()
        .zip(v.chars())
        .zip(s.chars())
        .map(|((c, v), s)| {
            if c == '-' {
                None
            } else {
                Some(Label { cons: c.to_digit(36).unwrap() as usize, vow: v.to_digit(10).unwrap() as usize, stress: s == '1' })
            }
        })
        .collect();
    render(&chars, &labels, None)
}

/// Constrained greedy decode of the tagger's logits: legal consonant per
/// letter, vowel argmax, one stress per whitespace word on the vowel-bearing
/// letter with the largest stress margin.
pub fn decode(text: &[char], lg: &Logits) -> Vec<Option<Label>> {
    let n = text.len().min(lg.len);
    let (nc, nv) = (lg.cons.len() / lg.len.max(1), lg.vow.len() / lg.len.max(1));
    let mut labels: Vec<Option<Label>> = vec![None; text.len()];
    for i in 0..n {
        if !is_hebrew(text[i]) {
            continue;
        }
        let row = &lg.cons[i * nc..(i + 1) * nc];
        let cons = *letter_cons(text[i]).iter().max_by(|&&a, &&b| row[a].partial_cmp(&row[b]).unwrap()).unwrap();
        let vrow = &lg.vow[i * nv..(i + 1) * nv];
        let vow = (0..nv).max_by(|&a, &b| vrow[a].partial_cmp(&vrow[b]).unwrap()).unwrap();
        labels[i] = Some(Label { cons, vow, stress: false });
    }
    let mut i = 0;
    while i < n {
        if text[i].is_whitespace() {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && !text[j].is_whitespace() {
            j += 1;
        }
        let mut best: Option<(usize, f32)> = None;
        for p in i..j {
            if let Some(l) = labels[p] {
                if l.vow != 0 {
                    let m = lg.stress[p * 2 + 1] - lg.stress[p * 2];
                    if best.is_none_or(|(_, bm)| m > bm) {
                        best = Some((p, m));
                    }
                }
            }
        }
        if let Some((p, _)) = best {
            labels[p].as_mut().unwrap().stress = true;
        }
        i = j;
    }
    labels
}

pub type LatinFn = dyn Fn(&str) -> String + Send + Sync;

pub struct Hebrew {
    model: Option<Tagger>,
    latin: Option<Box<LatinFn>>,
    window: usize,
}

impl Hebrew {
    /// Loads `he.g2p` from `data_dir` (or `$POLYG2P_HE_MODEL`). Without a model
    /// `phonemize` returns an empty string (and warns once).
    pub fn load(data_dir: Option<&Path>) -> Self {
        let path = std::env::var_os("POLYG2P_HE_MODEL")
            .map(std::path::PathBuf::from)
            .or_else(|| data_dir.map(|d| d.join("he.g2p")));
        let model = path.and_then(|p| match Tagger::open(&p) {
            Ok(m) => Some(m),
            Err(e) => {
                if p.exists() {
                    eprintln!("polyg2p: cannot load Hebrew model {}: {e}", p.display());
                }
                None
            }
        });
        let window = model.as_ref().map_or(WINDOW, |m| WINDOW.min(m.cfg.max_len));
        Hebrew { model, latin: None, window }
    }

    pub fn from_model(model: Tagger) -> Self {
        let window = WINDOW.min(model.cfg.max_len);
        Hebrew { model: Some(model), latin: None, window }
    }

    pub fn has_model(&self) -> bool {
        self.model.is_some()
    }

    /// Route Latin-script words to `f` (e.g. the English engine) instead of
    /// passing them through verbatim.
    pub fn with_latin<F: Fn(&str) -> String + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.latin = Some(Box::new(f));
        self
    }

    /// Hebrew text -> IPA in RenikudPlus notation.
    pub fn phonemize(&self, text: &str) -> String {
        let Some(model) = &self.model else {
            static WARNED: OnceLock<()> = OnceLock::new();
            WARNED.get_or_init(|| eprintln!("polyg2p: Hebrew model he.g2p not found; set POLYG2P_DATA"));
            return String::new();
        };
        let text = if num::contains_digit(text) { num::normalize(text) } else { text.to_string() };
        let norm: Vec<char> = normalize(&text).chars().collect();
        let mut out = String::with_capacity(norm.len() * 2);
        for w in split_for_decode(&norm, self.window) {
            if !w.iter().any(|&c| is_hebrew(c)) {
                out.push_str(&render(&w, &vec![None; w.len()], self.latin.as_deref()));
                continue;
            }
            let ids = model.encode(&w.iter().collect::<String>());
            let lg = model.forward(&ids);
            let labels = decode(&w, &lg);
            out.push_str(&render(&w, &labels, self.latin.as_deref()));
        }
        out
    }

    /// Per-character labels of the normalized text (for parity tests): the
    /// `(text, c, v, s)` strings in `tools/he_distill.py` encoding.
    pub fn label_codes(&self, text: &str) -> Option<(String, String, String, String)> {
        let model = self.model.as_ref()?;
        let text = if num::contains_digit(text) { num::normalize(text) } else { text.to_string() };
        let norm: Vec<char> = normalize(&text).chars().collect();
        let (mut c, mut v, mut s) = (String::new(), String::new(), String::new());
        for w in split_for_decode(&norm, self.window) {
            let labels = if w.iter().any(|&c| is_hebrew(c)) {
                let lg = model.forward(&model.encode(&w.iter().collect::<String>()));
                decode(&w, &lg)
            } else {
                vec![None; w.len()]
            };
            for l in labels {
                match l {
                    None => {
                        c.push('-');
                        v.push('-');
                        s.push('-');
                    }
                    Some(l) => {
                        c.push(B36[l.cons] as char);
                        v.push(B36[l.vow] as char);
                        s.push(if l.stress { '1' } else { '0' });
                    }
                }
            }
        }
        Some((norm.iter().collect(), c, v, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_matches_teacher_rules() {
        // שלום לכולם -> ʃalˈom lekulˈam
        assert_eq!(render_codes("שלום לכולם", "ma0b-a90ab", "1400-25010", "0100-00010"), "ʃalˈom lekulˈam");
        // geresh, furtive patah, orthographic markers dropped, passthrough
        assert_eq!(render_codes("ג'ורג' רוח, ok", "k-0lk--l06----", "4-000--501----", "0-000--100----"), "dʒoʁdʒ ʁˈuaχ, ok");
    }

    #[test]
    fn windowing_is_lossless() {
        let t: Vec<char> = "אבג דהו. זחט, יכל מנס עפצ קרש ת".chars().collect();
        for lim in [3, 5, 8, 12, 100] {
            let w = split_for_decode(&t, lim);
            let j: Vec<char> = w.iter().flatten().copied().collect();
            assert_eq!(j, t);
            assert!(w.iter().all(|p| p.len() <= lim));
        }
    }

    #[test]
    fn normalize_strips_points() {
        assert_eq!(normalize("שָׁלוֹם ג׳ ב״ח"), "שלום ג' ב\"ח");
    }
}
