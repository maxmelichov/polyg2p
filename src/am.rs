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
//!   (`ክፍል` → `kɨfɨl`, `እግር` → `ɨɡɨr`).
//!
//! Not predicted: gemination. Fidel does not write it, and it is lexical.
//! The `ɨ` rule works on the written word, so a one-letter prefix is syllabified
//! together with its stem.
//!
//! Amharic does not go through the shared tokenizer: `phonemize` reads the whole
//! text, because Ethiopic punctuation (`። ፣ ፤ ፥ ፦ ፧ ፡`) has to be mapped first
//! and apostrophes and hyphens are passed through as written.

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

/// May the word end in the cluster `c1 c2` without `ɨ`? Sonority must not rise,
/// and two sonorants need a fall.
fn coda_ok(c1: &str, c2: &str) -> bool {
    if c1 == c2 {
        return false;
    }
    let (s1, s2) = (sonority(c1), sonority(c2));
    if s2 > 1 { s1 > s2 } else { s1 >= s2 }
}

/// Characters that belong to a fidel run (Ethiopic, Ethiopic Supplement, Ethiopic Extended).
fn in_run(c: char) -> bool {
    matches!(c as u32, 0x1200..=0x135A | 0x1380..=0x138F | 0x2D80..=0x2DDF)
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
                || (seg[i + 1].0 == "ɲ" && !seg[i + 1].1 && !seg[i + 1].2.is_empty()))
        {
            // after a vowel, but it still opens a syllable: the final cluster comes
            // first (ስምንት → sɨmɨnt), and ɲ is long before a vowel (ማንኛው → manɨɲaw)
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

/// Ethiopic and typographic punctuation → the punctuation the output keeps.
/// `Some("")` deletes the character (quotes), so the letters around it join.
fn punct(c: char) -> Option<&'static str> {
    Some(match c {
        '።' | '፨' | '…' => ".",
        '፣' | '፤' | '፥' | '(' | ')' | '–' | '—' | ';' => ",",
        '፦' => ":",
        '፧' => "?",
        '፡' | '፠' => " ",
        '«' | '»' | '“' | '”' | '‘' | '’' | '"' => "",
        _ => return None,
    })
}

const KEPT: &[char] = &['.', ',', '?', '!', ':'];

/// Phonemize running Amharic text.
///
/// Fidel runs are converted, Ethiopic punctuation is mapped to `. , : ?`, and
/// spacing is normalized: no space before a mark, one after it, and `.`, `,` or
/// `:` right after another mark is dropped (`?.` → `?`, typed `::` → `:`).
/// Digits are read as Amharic numbers and `%` as its word. Apostrophes and
/// hyphens are passed through. Anything else (Latin letters, other scripts) is
/// treated as a space.
pub fn phonemize(text: &str) -> String {
    // 1. punctuation, digits, symbols
    let src: Vec<char> = text.chars().collect();
    let mut s = String::with_capacity(text.len() * 2);
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        if let Some(p) = punct(c) {
            s.push_str(p);
            i += 1;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < src.len() && src[i].is_ascii_digit() {
                i += 1;
            }
            // 1,000,000: groups of exactly three digits
            if i - start <= 3 {
                while i + 3 < src.len() && src[i] == ',' && src[i + 1..i + 4].iter().all(char::is_ascii_digit) && !src.get(i + 4).is_some_and(char::is_ascii_digit) {
                    i += 4;
                }
            }
            let int: String = src[start..i].iter().filter(|c| c.is_ascii_digit()).collect();
            s.push(' ');
            number(&int, &mut s);
            // 3.5: the fraction is read digit by digit
            if i + 1 < src.len() && src[i] == '.' && src[i + 1].is_ascii_digit() {
                s.push_str(DECIMAL);
                s.push(' ');
                i += 1;
                while i < src.len() && src[i].is_ascii_digit() {
                    s.push_str(UNITS[src[i] as usize - '0' as usize]);
                    s.push(' ');
                    i += 1;
                }
            }
        } else if c == '%' {
            s.push_str(" በመቶ ");
            i += 1;
        } else if in_run(c) || c.is_whitespace() || KEPT.contains(&c) || matches!(c, '\'' | '-') {
            s.push(c);
            i += 1;
        } else {
            s.push(' ');
            i += 1;
        }
    }
    // 2. fidel runs → phonemes; whitespace runs → one space
    let mut t: Vec<char> = Vec::with_capacity(s.len());
    let mut cur = String::new();
    let mut buf = String::new();
    for c in s.chars().chain(std::iter::once(' ')) {
        if in_run(c) {
            cur.push(c);
            continue;
        }
        if !cur.is_empty() {
            buf.clear();
            run(&cur, &mut buf);
            t.extend(buf.chars());
            cur.clear();
        }
        if c.is_whitespace() {
            if t.last() != Some(&' ') {
                t.push(' ');
            }
        } else {
            t.push(c);
        }
    }
    // 3. no space before a mark
    let mut u: Vec<char> = Vec::with_capacity(t.len());
    for &c in &t {
        if KEPT.contains(&c) {
            while u.last() == Some(&' ') {
                u.pop();
            }
        }
        u.push(c);
    }
    // 4. one space after a mark that is followed by text
    let mut v: Vec<char> = Vec::with_capacity(u.len() + 8);
    for (k, &c) in u.iter().enumerate() {
        v.push(c);
        if KEPT.contains(&c) && u.get(k + 1).is_some_and(|&d| d != ' ' && !KEPT.contains(&d)) {
            v.push(' ');
        }
    }
    // 5. `.`, `,`, `:` directly after a mark are dropped
    let mut out = String::with_capacity(v.len() * 2);
    let mut k = 0;
    while k < v.len() {
        let c = v[k];
        out.push(c);
        k += 1;
        if KEPT.contains(&c) {
            while k < v.len() && matches!(v[k], '.' | ',' | ':') {
                k += 1;
            }
        }
    }
    out.trim().to_string()
}

/// The shared sentence pass has nothing to do for Amharic.
pub fn join(_outs: &mut [Out]) {}

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

/// A digit string as number words followed by a space each. A leading zero
/// (phone numbers, codes) is read digit by digit.
fn number(digits: &str, s: &mut String) {
    let words: Vec<String> = match digits.parse::<u128>() {
        Ok(n) if !(digits.len() > 1 && digits.starts_with('0')) => cardinal(n),
        _ => digits.bytes().map(|b| UNITS[(b - b'0') as usize].to_string()).collect(),
    };
    for w in words {
        s.push_str(&w);
        s.push(' ');
    }
}

pub fn symbol(c: char) -> Option<&'static str> {
    match c {
        '%' => Some("በመቶ"),
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
    }

    #[test]
    fn text() {
        assert_eq!(phonemize("ሰላም፣ እንዴት ነህ?"), "səlam, ɨndet nəh?");
        assert_eq!(phonemize("ቡና እፈልጋለሁ።"), "buna ɨfəlɡaləhu.");
        assert_eq!(phonemize("ለምሳሌ፦ፖሊስ፣ሚሊሻ"), "ləmsale: polis, miliʃa");
        assert_eq!(phonemize("ነው ::."), "nəw:");
        assert_eq!(phonemize("ሰላም፡ዓለም"), "səlam aləm");
        assert_eq!(phonemize("  «ሰላም»  "), "səlam");
        assert_eq!(phonemize("ይካሄዳ?."), "jɨkaheda?");
        assert_eq!(phonemize(""), "");
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), ["ዜሮ"]);
        assert_eq!(cardinal(11), ["አሥራ", "አንድ"]);
        assert_eq!(cardinal(21), ["ሃያ", "አንድ"]);
        assert_eq!(cardinal(100), ["አንድ", "መቶ"]);
        assert_eq!(cardinal(1234), ["አንድ", "ሺህ", "ሁለት", "መቶ", "ሠላሳ", "አራት"]);
        assert_eq!(cardinal(2_000_000), ["ሁለት", "ሚሊዮን"]);
        assert_eq!(phonemize("3 ሰዎች"), "sost səwotʃ");
        assert_eq!(phonemize("1,000 ብር"), "and ʃih bɨr");
        assert_eq!(phonemize("50%"), "hamsa bəməto");
        assert_eq!(phonemize("2.5"), "hulət nətʼb amɨst");
    }
}
