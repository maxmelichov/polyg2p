//! No input may panic the front end: random and mangled lines (mixed scripts, odd punctuation,
//! combining marks, word starts and ends that the rules index around) through every language.
use polyg2p::{G2p, Lang};
use std::panic::{catch_unwind, AssertUnwindSafe};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len())]
    }
}

/// Fragments that sit next to the rules' lookbehind/lookahead: word edges, suffixes, digraphs, silent tails.
const FRAGS: &[&str] = &[
    "ti", "tien", "tient", "tiente", "tientes", "tiens", "tie", "tion", "tia", "tium", "s", "ss", "sc", "sh", "as", "es", "is",
    "a", "e", "é", "è", "ê", "i", "o", "u", "y", "œ", "ç", "qu", "gu", "ge", "gn", "ill", "il", "ent", "ant", "en", "an", "on",
    "ain", "ein", "oin", "eau", "au", "ou", "oi", "ai", "ei", "x", "z", "t", "d", "p", "c", "g", "h", "k", "l", "m", "n", "r", "v",
    "w", "b", "f", "j", "re", "pré", "trans", "tran", "ing", "ez", "er", "ier", "ment", "ie", "ée", "ées", "ué", "ïe", "ë",
    "sch", "ch", "ph", "th", "ij", "ae", "oe", "ue", "ia", "io", "ão", "ñ", "ł", "ż", "ș", "ț", "ğ", "ı", "ő", "ű", "ö", "ü",
];
const ODD: &[&str] = &[
    " ", " ", " ", " ", ",", ".", "!", "?", ";", ":", "-", "—", "–", "'", "’", "\"", "«", "»", "(", ")", "[", "]", "[x]", "[laugh]",
    "…", "/", "\\", "&", "%", "$", "€", "#", "@", "*", "+", "=", "_", "~", "|", "<", ">", "{", "}", "\u{0301}", "\u{0308}", "\u{200d}",
    "\u{200b}", "\u{00a0}", "\u{feff}", "\t", "0", "1", "2", "7", "12", "2024", "1er", "3e", "1,5", "0.5", "-3", "ⅷ", "²", "½",
    "😀", "👍🏽", "аб", "Привет", "שלום", "שָׁלוֹם", "مرحبا", "γειά", "你好", "日本語", "한국어", "ሰላም", "ñandú", "ǅ", "ß", "İ", "ǰ", "ŉ",
];

fn line(r: &mut Rng) -> String {
    let mut s = String::new();
    for _ in 0..1 + r.below(8) {
        match r.below(10) {
            0 => s.push_str(r.pick(ODD)),
            1 => {
                // mangled: random Unicode scalar from a few blocks
                let base = *r.pick(&[0x20u32, 0xc0, 0x100, 0x370, 0x400, 0x590, 0x600, 0x1200, 0x2000, 0x3040, 0x4e00, 0x1f600]);
                if let Some(c) = char::from_u32(base + r.below(0x80) as u32) {
                    s.push(c);
                }
            }
            _ => {
                for _ in 0..1 + r.below(4) {
                    s.push_str(r.pick(FRAGS));
                }
                if r.below(4) == 0 {
                    let w = s.clone();
                    s.push_str(&w.to_uppercase());
                }
            }
        }
        if r.below(3) != 0 {
            s.push(' ');
        }
        if r.below(12) == 0 {
            s.push_str(r.pick(ODD));
        }
    }
    s
}

fn fuzz(l: Lang, n: usize, seed: u64) -> Vec<String> {
    let g = G2p::new(None);
    let mut r = Rng(seed);
    std::panic::set_hook(Box::new(|_| {}));
    let mut bad = Vec::new();
    for _ in 0..n {
        let t = line(&mut r);
        if catch_unwind(AssertUnwindSafe(|| g.phonemize(&t, l))).is_err() {
            bad.push(t);
        }
    }
    bad
}

#[test]
fn fr_never_panics() {
    let bad = fuzz(Lang::Fr, 10_000, 0x9e3779b97f4a7c15);
    assert!(bad.is_empty(), "{} panicking lines, first: {:?}", bad.len(), &bad[..bad.len().min(5)]);
}

/// Word-initial `tient…`: `ti_is_s` looked at the letter before the word start.
#[test]
fn fr_word_initial_tient() {
    let g = G2p::new(None);
    for w in ["tiente", "tientes", "tientr", "Tiente", "TIENTE", "x tiente", "(tientes)"] {
        g.phonemize(w, Lang::Fr);
    }
}

#[test]
fn every_language_never_panics() {
    let mut failed = Vec::new();
    for l in ["en", "es", "es-la", "it", "de", "ru", "he", "fr", "pt", "pl", "uk", "tr", "nl", "cs", "el", "ro", "hu", "fi", "sv", "id", "ca", "ar", "am", "bg", "da", "et", "hr", "lt", "lv", "sk", "sl", "en-gb", "en-au", "en-in"] {
        let Some(lang) = Lang::parse(l) else { continue };
        if !lang.implemented() || (l == "he" && polyg2p::data_dir().is_none()) {
            continue;
        }
        let bad = fuzz(lang, 10_000, 0x2545f4914f6cdd1d);
        if !bad.is_empty() {
            failed.push(format!("{l}: {} panicking lines, first {:?}", bad.len(), &bad[..bad.len().min(3)]));
        }
    }
    assert!(failed.is_empty(), "{failed:#?}");
}
