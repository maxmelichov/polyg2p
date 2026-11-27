//! Amharic grapheme-to-phoneme: Ge'ez script (fidel) to IPA. Rules only.
//!
//! Fidel is a syllabary whose code points are laid out as consonant row x vowel
//! order, so the letter-to-sound part is a table lookup:
//!
//! * A fidel is `consonant + vowel`. The seven orders are `ə u i a e (none) o`;
//!   the eighth form of a row is the labialized `Cʷa` (`ሏ` → `lʷa`).
//! * The first order of the gutturals `ሀ ሐ ኀ አ ዐ` is `a`, not `ə`.
//! * `ሀ ሐ ኀ ኸ` are all `h`; `ሰ ሠ` are `s`; `ጸ ፀ` are `sʼ`. Ejectives are written
//!   with the modifier apostrophe: `pʼ tʼ kʼ sʼ tʃʼ`.
//! * `አ` and `ዐ` carry a vowel. No glottal stop is written at the start of a word;
//!   inside a word after a vowel it is `ʔ` (`ሰዓት` → `səʔat`).
//! * The sixth order is a bare consonant. The epenthetic vowel `ɨ` comes from a
//!   left-to-right syllabification, syllable = (C)V(C), with CVCC only at the end
//!   of the word: a bare consonant after a vowel is a coda; after a consonant or
//!   at the start of the word it opens a syllable and takes `ɨ`. The last
//!   consonant of the word joins the coda when sonority does not rise
//!   (`ልብስ` → `lɨbs`, `አንድ` → `and`); otherwise `ɨ` goes between the two
//!   (`ክፍል` → `kɨfɨl`, `እግር` → `ɨɡɨr`). `h` never closes a cluster (`እኅት` →
//!   `ɨhɨt`).
//! * A bare `t` or `d` never stands directly before `ʃ`, `ʒ` or `z`, nor `t`
//!   before a word-final `s`: it takes `ɨ` (`ቤትሽ` → `betɨʃ`). Written next to
//!   each other the two would be folded into the affricate (`fold.rs`), and
//!   `ቤትሽ` would come out the same as `ቤች`.
//!
//! Not predicted: gemination. Fidel does not write it, and it is lexical.
//! The `ɨ` rule works on the written word, so a one-letter prefix is syllabified
//! together with its stem.
//!
//! Text goes through the shared tokenizer after [`marks`] has mapped Ethiopic
//! punctuation (`። ፣ ፤ ፥ ፦ ፧ ፡`) to the kept marks and turned everything the
//! module does not read into a space. [`join`] then makes sure no mark is left
//! standing alone.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "ነጥብ";
pub const LETTERS: Option<&[&str; 26]> = None;

/// Consonant of each regular row (first code point of the row).
fn row(r: u32) -> Option<&'static str> {
    Some(match r {
        0x1200 | 0x1210 | 0x1280 | 0x12B8 => "h",
        0x1208 => "l",
        0x1218 => "m",
        0x1220 | 0x1230 => "s",
        0x1228 => "r",
        0x1238 => "ʃ",
        0x1240 | 0x1250 => "kʼ",
        0x1260 => "b",
        0x1268 => "v",
        0x1270 => "t",
        0x1278 => "tʃ",
        0x1290 => "n",
        0x1298 => "ɲ",
        0x12A0 | 0x12D0 => "ʔ",
        0x12A8 => "k",
        0x12C8 => "w",
        0x12D8 => "z",
        0x12E0 => "ʒ",
        0x12E8 => "j",
        0x12F0 | 0x12F8 => "d",
        0x1300 => "dʒ",
        0x1308 => "ɡ",
        0x1318 => "ŋ",
        0x1320 => "tʼ",
        0x1328 => "tʃʼ",
        0x1330 => "pʼ",
        0x1338 | 0x1340 => "sʼ",
        0x1348 => "f",
        0x1350 => "p",
        _ => return None,
    })
}

/// Labialized series `ቈ ኈ ኰ ጐ` (and the Tigrinya `ቘ ዀ`): `ʷə . ʷi ʷa ʷe ʷɨ`.
fn lab_row(r: u32) -> Option<&'static str> {
    Some(match r {
        0x1248 | 0x1258 => "kʼʷ",
        0x1288 | 0x12C0 => "hʷ",
        0x12B0 => "kʷ",
        0x1310 => "ɡʷ",
        _ => return None,
    })
}

const ORDERS: [&str; 7] = ["ə", "u", "i", "a", "e", "", "o"];

/// One fidel as (consonant, labialized, vowel). An empty vowel is the sixth
/// order (bare consonant); `labialized` is the eighth form `Cʷa`. `None` for
/// anything that is not an Amharic syllable letter.
pub fn fidel(ch: char) -> Option<(&'static str, bool, &'static str)> {
    match ch {
        'ኧ' => return Some(("ʔ", false, "ə")),
        'ፘ' => return Some(("rj", false, "a")),
        'ፙ' => return Some(("mj", false, "a")),
        'ፚ' => return Some(("fj", false, "a")),
        'ሇ' | 'ኇ' => return Some(("h", false, "o")),
        'ቇ' => return Some(("kʼ", false, "o")),
        'ኯ' => return Some(("k", false, "o")),
        'ዏ' => return Some(("w", false, "o")),
        'ዯ' => return Some(("j", false, "o")),
        'ጏ' => return Some(("ɡ", false, "o")),
        'ፇ' => return Some(("sʼ", false, "o")),
        _ => {}
    }
    let o = ch as u32;
    let (r, k) = (o & !7, (o & 7) as usize);
    if let Some(c) = lab_row(r) {
        let v = match k {
            0 => "ə",
            2 => "i",
            3 => "a",
            4 => "e",
            5 => "",
            _ => return None,
        };
        return Some((c, false, v));
    }
    // unassigned code points inside the rows below
    if matches!(o, 0x1257 | 0x12BF | 0x12D7) {
        return None;
    }
    let c = row(r)?;
    if k == 7 {
        return Some((c, true, "a"));
    }
    let guttural = matches!(r, 0x1200 | 0x1210 | 0x1280 | 0x12A0 | 0x12D0);
    Some((c, false, if guttural && k == 0 { "a" } else { ORDERS[k] }))
}

fn sonority(c: &str) -> u8 {
    match c.trim_end_matches('ʷ') {
        "w" | "j" => 4,
        "l" | "r" => 3,
        "m" | "n" | "ɲ" | "ŋ" => 2,
        _ => 1,
    }
}

/// Would `fold.rs` write `c1 c2` as one affricate letter wherever they meet?
fn folds(c1: &str, c2: &str) -> bool {
    matches!((c1, c2), ("t", "ʃ") | ("d", "ʒ") | ("d", "z"))
}

/// May the word end in the cluster `c1 c2` without `ɨ`? Sonority must not rise,
/// and two sonorants need a fall. `h` does not close a cluster, and neither does
/// a pair the affricate fold would merge (`ts` is folded at the end of a word).
fn coda_ok(c1: &str, c2: &str) -> bool {
    if c1 == c2 || c1 == "h" || folds(c1, c2) || (c1, c2) == ("t", "s") {
        return false;
    }
    let (s1, s2) = (sonority(c1), sonority(c2));
    if s2 > 1 { s1 > s2 } else { s1 >= s2 }
}

/// Letters of a written word: Ethiopic, and the Ethiopic Supplement, Extended
/// and Extended-A blocks. Only the first is read ([`fidel`]); a letter of the
/// other three is skipped without splitting the word.
fn in_run(c: char) -> bool {
    matches!(c as u32, 0x1200..=0x135A | 0x1380..=0x138F | 0x2D80..=0x2DDF | 0xAB01..=0xAB2E) && c.is_alphabetic()
}

/// Phonemes of one run of fidel (one written word).
fn run(w: &str, out: &mut String) {
    let seg: Vec<(&str, bool, &str)> = w.chars().filter_map(fidel).collect();
    let n = seg.len();
    let bare = |i: usize| seg[i].0 != "ʔ" && seg[i].2.is_empty();
    let mut ph: Vec<&str> = Vec::with_capacity(3 * n);
    // consonants since the last vowel
    let mut coda = 0u8;
    for i in 0..n {
        let (c, lab, v) = seg[i];
        let last = i + 1 == n;
        if c == "ʔ" && !lab {
            // vowel carrier (sixth order እ = ɨ); ʔ only in hiatus inside the word
            if !ph.is_empty() && coda == 0 {
                ph.push("ʔ");
            }
            // ረቡዕ → rəbuʔ: no ɨ after a word-final carrier that follows a vowel
            if !(last && v.is_empty() && !ph.is_empty() && coda == 0) {
                ph.push(if v.is_empty() { "ɨ" } else { v });
            }
            coda = 0;
        } else if !v.is_empty() {
            ph.push(c);
            if lab {
                ph.push("ʷ");
            }
            ph.push(v);
            coda = 0;
        } else if !last && (ph.is_empty() || coda > 0) {
            // opens a syllable
            ph.push(c);
            ph.push("ɨ");
            coda = 0;
        } else if !last
            && ((i + 3 == n && bare(n - 2) && bare(n - 1) && coda_ok(seg[n - 2].0, seg[n - 1].0))
                || (seg[i + 1].0 == "ɲ" && !seg[i + 1].1 && !seg[i + 1].2.is_empty())
                || folds(c, seg[i + 1].0))
        {
            // after a vowel, but it still opens a syllable: the final cluster comes
            // first (ስምንት → sɨmɨnt), ɲ is long before a vowel (ማንኛው → manɨɲaw), and
            // t + ʃ must not read as the affricate (ቤትሻ → betɨʃa)
            ph.push(c);
            ph.push("ɨ");
            coda = 0;
        } else if !last || coda == 0 {
            // coda after a vowel (a word that is one bare consonant stays bare)
            ph.push(c);
            coda = 1;
        } else if coda == 1 && coda_ok(ph[ph.len() - 1], c) {
            ph.push(c);
            coda = 2;
        } else {
            ph.push("ɨ");
            ph.push(c);
            coda = 1;
        }
    }
    for p in ph {
        out.push_str(p);
    }
}

/// Context-free phonemes of one word: its fidel letters as one run (anything
/// else in the token is skipped).
pub fn word(lw: &str) -> String {
    let mut out = String::with_capacity(lw.len());
    run(lw, &mut out);
    out
}

/// Ethiopic and typographic punctuation → the mark the output keeps, followed
/// by a space so that it never reads as a digit separator (`3፣4` is two
/// numbers). `Some(" ")` is a word boundary; `Some("")` deletes the character,
/// so the letters around it join: quotation marks, apostrophes, and the
/// combining marks (the Ethiopic gemination and vowel-length marks
/// U+135D..U+135F, and U+0300..U+036F).
fn punct(c: char) -> Option<&'static str> {
    Some(match c {
        '።' | '፨' | '…' => ". ",
        '፣' | '፤' | '፥' | '(' | ')' | '–' | '—' | ';' => ", ",
        '፦' => ": ",
        '፧' => "? ",
        '፡' | '፠' => " ",
        '«' | '»' | '“' | '”' | '‘' | '’' | '"' | '\'' | '\u{135D}'..='\u{135F}' | '\u{0300}'..='\u{036F}' => "",
        _ => return None,
    })
}

const KEPT: &[char] = &['.', ',', '?', '!', ':'];
/// Symbols the shared tokenizer hands to [`symbol`].
const SPOKEN: &[char] = &['%', '$', '€', '£', '₪', '¥', '₽', '&', '+', '@', '°', '='];

/// The punctuation pass that runs before the shared tokenizer.
///
/// * Ethiopic punctuation becomes `. , : ?`; the word space `፡` becomes a space.
/// * `፡፡` and `::`, the usual ways to type `።`, are a full stop; `፡-` is `፦`.
/// * Quotation marks, apostrophes and combining marks are deleted.
/// * Fidel, digits, `. , ? ! :`, white space and the symbols that are read
///   (`%`, currency signs, `&`, `°`) pass through.
/// * Anything else is a space: hyphens and dashes, slashes, Latin letters, other
///   scripts, Ethiopic numerals.
pub fn marks(text: &str) -> String {
    let src: Vec<char> = text.chars().collect();
    let mut s = String::with_capacity(text.len() + 16);
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        i += 1;
        if matches!(c, '፡' | ':') && src.get(i) == Some(&c) {
            while src.get(i) == Some(&c) {
                i += 1;
            }
            s.push_str(". ");
        } else if c == '፡' && src.get(i) == Some(&'-') {
            i += 1;
            s.push_str(": ");
        } else if let Some(p) = punct(c) {
            s.push_str(p);
        } else if in_run(c) || c.is_ascii_digit() || c.is_whitespace() || KEPT.contains(&c) || SPOKEN.contains(&c) {
            s.push(c);
        } else {
            s.push(' ');
        }
    }
    s
}

/// Is `c` Ethiopic punctuation? (`lib.rs` keeps it with an inline tag: `[laughs]።`.)
pub fn is_mark(c: char) -> bool {
    matches!(c, '።' | '፨' | '፣' | '፤' | '፥' | '፦' | '፧' | '፡' | '፠')
}

/// `.`, `,`, `:` right after another mark are dropped (`?.` → `?`).
fn dedupe(punct: &mut String) {
    let mut first = true;
    punct.retain(|c| {
        let keep = first || !matches!(c, '.' | ',' | ':');
        first = false;
        keep
    });
}

/// The kept marks of a run of punctuation that follows an inline tag.
pub fn tag_marks(run: &str) -> String {
    let mut p: String = marks(run).chars().filter(|c| KEPT.contains(c)).collect();
    dedupe(&mut p);
    p
}

/// Sentence pass: a mark never stands alone. A token with no phonemes (letters
/// the module does not read) is removed and its marks go to the word before it;
/// marks before the first word are dropped.
pub fn join(outs: &mut Vec<Out>) {
    let mut kept: Vec<Out> = Vec::with_capacity(outs.len());
    for o in outs.drain(..) {
        if !o.ph.is_empty() {
            kept.push(o);
        } else if let Some(prev) = kept.last_mut() {
            prev.pause_after |= !o.punct.is_empty();
            prev.punct.push_str(&o.punct);
        }
    }
    for o in &mut kept {
        dedupe(&mut o.punct);
    }
    *outs = kept;
}

/// Digits written against letters are one word. The letters before go in front
/// of the first number word (`በ5` → `በአምስት`). The letters after restate the end
/// of the last one, as in the written language:
///
/// * `ኛ` makes the ordinal: a final bare consonant takes its first order
///   (`5ኛ` → `አምስተኛ`, `1ኛ` → `አንደኛ`; `9ኛ` → `ዘጠነኛ`);
/// * a suffix that starts with the consonant the number word ends in replaces
///   that letter (`2ቱ` → `ሁለቱ`, `10ሩ` → `አሥሩ`);
/// * anything else is appended (`2ም` → `ሁለትም`).
pub fn glue(before: Option<&str>, words: &mut [String], after: Option<&str>) {
    if let (Some(b), Some(w)) = (before, words.first_mut()) {
        w.insert_str(0, b);
    }
    let (Some(a), Some(w)) = (after, words.last_mut()) else { return };
    let end = w.chars().last().and_then(|e| Some((e, fidel(e)?)));
    let start = a.chars().next().and_then(fidel);
    if let (Some((e, (ec, false, ""))), Some((sc, _, _))) = (end, start) {
        // the number word ends in a bare consonant
        let guttural = matches!(ec, "h" | "ʔ");
        if a.starts_with('ኛ') && !guttural {
            w.pop();
            w.push(if e == 'ኝ' { 'ነ' } else { char::from_u32(e as u32 & !7).unwrap_or(e) });
        } else if sc == ec && !guttural {
            w.pop();
        }
    }
    w.push_str(a);
}

const UNITS: [&str; 10] = ["ዜሮ", "አንድ", "ሁለት", "ሦስት", "አራት", "አምስት", "ስድስት", "ሰባት", "ስምንት", "ዘጠኝ"];
const TENS: [&str; 10] = ["", "አሥር", "ሃያ", "ሠላሳ", "አርባ", "ሃምሳ", "ስልሳ", "ሰባ", "ሰማንያ", "ዘጠና"];
const SCALES: [&str; 5] = ["", "ሺህ", "ሚሊዮን", "ቢሊዮን", "ትሪሊዮን"];

fn below_thousand(n: u128, out: &mut Vec<String>) {
    let (h, r) = (n / 100, n % 100);
    if h > 0 {
        out.push(UNITS[h as usize].to_string());
        out.push("መቶ".to_string());
    }
    let (t, u) = (r / 10, r % 10);
    match (t, u) {
        (0, 0) => {}
        (0, u) => out.push(UNITS[u as usize].to_string()),
        (1, 0) => out.push(TENS[1].to_string()),
        (1, u) => {
            // 11..19: አሥራ + unit
            out.push("አሥራ".to_string());
            out.push(UNITS[u as usize].to_string());
        }
        (t, 0) => out.push(TENS[t as usize].to_string()),
        (t, u) => {
            out.push(TENS[t as usize].to_string());
            out.push(UNITS[u as usize].to_string());
        }
    }
}

/// Number words, one written word per item (`መቶ` hundred, `ሺህ` thousand).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec![UNITS[0].to_string()];
    }
    if n >= 1_000_000_000_000_000 {
        // beyond the named scales: digit by digit
        return n.to_string().bytes().map(|b| UNITS[(b - b'0') as usize].to_string()).collect();
    }
    let mut groups = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push(m % 1000);
        m /= 1000;
    }
    let mut out = Vec::new();
    for (k, &g) in groups.iter().enumerate().rev() {
        if g == 0 {
            continue;
        }
        below_thousand(g, &mut out);
        if k > 0 {
            out.push(SCALES[k].to_string());
        }
    }
    out
}

/// Words for a spoken symbol. Symbols without an entry are skipped.
pub fn symbol(c: char) -> Option<&'static str> {
    match c {
        '%' => Some("በመቶ"),
        '$' => Some("ዶላር"),
        '€' => Some("ዩሮ"),
        '£' => Some("ፓውንድ"),
        '&' => Some("እና"),
        '°' => Some("ዲግሪ"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters() {
        for (w, p) in [
            ("ሰላም", "səlam"),
            ("ሀሎ", "halo"),
            ("ሁለት", "hulət"),
            ("ውሃ", "wɨha"),
            ("ቋንቋ", "kʼʷankʼʷa"),
            ("ጐረቤት", "ɡʷərəbet"),
            ("ጸሐይ", "sʼəhaj"),
            ("ፀሐይ", "sʼəhaj"),
            ("ጨው", "tʃʼəw"),
            ("ጳጳስ", "pʼapʼas"),
            ("ጀመረ", "dʒəmərə"),
            ("ዠ", "ʒə"),
            ("ኧረ", "ərə"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn carriers() {
        for (w, p) in [("አንድ", "and"), ("ኢትዮጵያ", "itjopʼja"), ("እግር", "ɨɡɨr"), ("ሰዓት", "səʔat"), ("ረቡዕ", "rəbuʔ"), ("ዓለም", "aləm")] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn epenthesis() {
        for (w, p) in [
            ("ስም", "sɨm"),
            ("ብር", "bɨr"),
            ("ልብስ", "lɨbs"),
            ("ስልክ", "sɨlk"),
            ("ክፍል", "kɨfɨl"),
            ("ክብር", "kɨbɨr"),
            ("ጥቅም", "tʼɨkʼɨm"),
            ("ድንግል", "dɨnɡɨl"),
            ("ትምህርት", "tɨmhɨrt"),
            ("መንግሥት", "mənɡɨst"),
            ("ስምንት", "sɨmɨnt"),
            ("አንድነት", "andɨnət"),
            ("ምክንያት", "mɨknɨjat"),
            ("ማንኛው", "manɨɲaw"),
            ("ሆስፒታል", "hospital"),
            ("ች", "tʃ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn clusters() {
        assert!(coda_ok("b", "s") && coda_ok("n", "d") && coda_ok("r", "t") && coda_ok("j", "kʼ") && coda_ok("r", "n"));
        assert!(!coda_ok("f", "l") && !coda_ok("ɡ", "r") && !coda_ok("kʼ", "m") && !coda_ok("m", "n") && !coda_ok("t", "t"));
        // h does not close a cluster
        assert!(!coda_ok("h", "t") && coda_ok("r", "h"));
        assert_eq!(word("እኅት"), "ɨhɨt");
    }

    #[test]
    fn no_false_affricate() {
        // pairs fold.rs would write as ʧ ʦ ʤ ʣ are never left next to each other
        assert!(!coda_ok("t", "ʃ") && !coda_ok("t", "s") && !coda_ok("d", "ʒ") && !coda_ok("d", "z"));
        assert!(coda_ok("tʼ", "s") && coda_ok("d", "s") && coda_ok("t", "sʼ"));
        for (w, p) in [
            ("ቤትሽ", "betɨʃ"),
            ("ኮትሽ", "kotɨʃ"),
            ("ቤትሽን", "betɨʃɨn"),
            ("ቤትስ", "betɨs"),
            ("ቤትሻ", "betɨʃa"),
            ("ከብትሽ", "kəbtɨʃ"),
            ("ውድዥ", "wɨdɨʒ"),
            ("ውድዝ", "wɨdɨz"),
            ("ውድዛ", "wɨdɨza"),
            ("ትሿ", "tɨʃʷa"),
            // the real affricates and other clusters are untouched
            ("ቤች", "betʃ"),
            ("ቤትሳ", "betsa"),
            ("መንግሥት", "mənɡɨst"),
        ] {
            assert_eq!(word(w), p, "{w}");
            let folded = crate::fold::fold(&word(w));
            assert_eq!(folded.contains(['ʧ', 'ʦ', 'ʤ', 'ʣ']), w == "ቤች", "{w} -> {folded}");
        }
    }

    #[test]
    fn punctuation_pass() {
        assert_eq!(marks("ሰላም፣ ዓለም።"), "ሰላም,  ዓለም. ");
        // ፡፡ and :: are a full stop, ፡- is ፦, one ፡ is a space
        assert_eq!(marks("ሀገር፡፡"), "ሀገር. ");
        assert_eq!(marks("ሰላም::"), "ሰላም. ");
        assert_eq!(marks("ለምሳሌ፡- ቡና"), "ለምሳሌ:  ቡና");
        assert_eq!(marks("ሰላም፡ዓለም"), "ሰላም ዓለም");
        // an Ethiopic comma between digits is not a digit separator
        assert_eq!(marks("3፣4"), "3, 4");
        // deleted: quotation marks, apostrophes, combining marks
        assert_eq!(marks("«የ'ሰ\u{135F}ላ\u{0301}ም'»"), "የሰላም");
        // a space: hyphens, the minus sign, slashes, Latin letters, Ethiopic numerals
        assert_eq!(marks("ሥነ-ሥርዓት −5 ዶ/ር COVID-19 ፩"), "ሥነ ሥርዓት  5 ዶ ር       19  ");
        assert_eq!(marks("$5 50% 3:30 1,000.5"), "$5 50% 3:30 1,000.5");
        assert_eq!(tag_marks("።"), ".");
        assert_eq!(tag_marks("?።"), "?");
        assert_eq!(tag_marks("፡፡"), ".");
        assert_eq!(tag_marks(";"), ",");
        assert_eq!(tag_marks("፡"), "");
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), ["ዜሮ"]);
        assert_eq!(cardinal(11), ["አሥራ", "አንድ"]);
        assert_eq!(cardinal(21), ["ሃያ", "አንድ"]);
        assert_eq!(cardinal(100), ["አንድ", "መቶ"]);
        assert_eq!(cardinal(1234), ["አንድ", "ሺህ", "ሁለት", "መቶ", "ሠላሳ", "አራት"]);
        assert_eq!(cardinal(2_000_000), ["ሁለት", "ሚሊዮን"]);
        assert_eq!(symbol('%'), Some("በመቶ"));
        assert_eq!(symbol('$'), Some("ዶላር"));
        assert_eq!(symbol('+'), None);
    }

    #[test]
    fn digits_against_letters() {
        let glued = |before: Option<&str>, n: u128, after: Option<&str>| {
            let mut w = cardinal(n);
            glue(before, &mut w, after);
            w.join(" ")
        };
        for (n, after, want) in [
            (1, "ኛ", "አንደኛ"),
            (2, "ኛ", "ሁለተኛ"),
            (5, "ኛ", "አምስተኛ"),
            (9, "ኛ", "ዘጠነኛ"),
            (10, "ኛ", "አሥረኛ"),
            (20, "ኛ", "ሃያኛ"),
            (100, "ኛው", "አንድ መቶኛው"),
            (1000, "ኛ", "አንድ ሺህኛ"),
            (2, "ቱ", "ሁለቱ"),
            (5, "ቱ", "አምስቱ"),
            (9, "ኙ", "ዘጠኙ"),
            (10, "ሩ", "አሥሩ"),
            (2, "ም", "ሁለትም"),
            (20, "ዎቹ", "ሃያዎቹ"),
        ] {
            assert_eq!(glued(None, n, Some(after)), want, "{n}{after}");
        }
        assert_eq!(glued(Some("በ"), 5, None), "በአምስት");
        assert_eq!(glued(Some("ከ"), 1000, Some("ኛ")), "ከአንድ ሺህኛ");
        assert_eq!(glued(None, 5, None), "አምስት");
    }
}
