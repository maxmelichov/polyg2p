//! Estonian grapheme-to-phoneme rules, notation shared with `fi.rs` / `cs.rs`.
//!
//! Estonian spelling is close to phonemic (Wikipedia, "Estonian phonology" and the Wiktionary
//! Estonian pronunciation appendix, both CC BY-SA, read 2026-11-28). Rules written here:
//! * stress on the first syllable (compounds' later stress is not marked);
//! * `õ ä ö ü` are `ɤ æ ø y`, `a` is `ɑ`; a doubled vowel is long (`ː`);
//! * a doubled consonant is long (`ː`), `pp tt kk` likewise: the third (overlong) degree is not
//!   written in the orthography and is not marked;
//! * `b d g` are the unaspirated lenis stops, written `b d ɡ` as spelled; `p t k` are written `p t k`;
//! * `ng` is `ŋk`, `nk` is `ŋk`, `š ž` are `ʃ ʒ`, `h` is `h`, `j` is `j`, `v` is `v`;
//! * loan letters `c q w x z f` are mapped to `k k v ks ts f`.
//! Palatalisation and quantity degree 3 are not in the spelling and are left unmarked.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "koma";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "ɑː", "beː", "tseː", "deː", "eː", "ef", "ɡeː", "hɑː", "iː", "jot", "kɑː", "el", "em", "en", "oː", "peː", "kuː",
    "er", "es", "teː", "uː", "veː", "kɑksisveː", "iks", "iɡrek", "tset",
]);

/// Function words read without a stress mark.
const UNSTRESSED: &[&str] = &[
    "ja", "ka", "on", "ei", "ta", "me", "te", "nad", "see", "mis", "kes", "et", "kui", "aga", "ning", "või", "ma", "sa",
    "oli", "olid", "ole", "vaid", "nii", "siis",
];

fn vowel(c: char) -> Option<(&'static str, &'static str)> {
    Some(match c {
        'a' => ("ɑ", "ɑː"),
        'e' => ("e", "eː"),
        'i' => ("i", "iː"),
        'o' => ("o", "oː"),
        'u' => ("u", "uː"),
        'õ' => ("ɤ", "ɤː"),
        'ä' => ("æ", "æː"),
        'ö' => ("ø", "øː"),
        'ü' => ("y", "yː"),
        'y' => ("y", "yː"),
        _ => return None,
    })
}

fn cons(c: char) -> Option<&'static str> {
    Some(match c {
        'b' => "b", 'd' => "d", 'f' => "f", 'g' => "ɡ", 'h' => "h", 'j' => "j", 'k' => "k", 'l' => "l", 'm' => "m",
        'n' => "n", 'p' => "p", 'r' => "r", 's' => "s", 't' => "t", 'v' => "v", 'w' => "v", 'z' => "ts", 'š' => "ʃ",
        'ž' => "ʒ", 'c' => "k", 'q' => "k", 'x' => "ks",
        _ => return None,
    })
}

pub fn word(w: &str) -> String {
    let lw = w.to_lowercase();
    if lw.contains('-') {
        return lw.split('-').filter(|p| !p.is_empty()).map(word).filter(|p| !p.is_empty()).collect::<Vec<_>>().join("");
    }
    let c: Vec<char> = lw.chars().filter(|c| c.is_alphabetic()).collect();
    if c.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut stressed = !UNSTRESSED.contains(&lw.as_str());
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if let Some((short, long)) = vowel(ch) {
            if stressed {
                out.push('ˈ');
                stressed = false;
            }
            if c.get(i + 1) == Some(&ch) {
                out.push_str(long);
                i += 2;
            } else {
                out.push_str(short);
                i += 1;
            }
            continue;
        }
        if let Some(ph) = cons(ch) {
            let dbl = c.get(i + 1) == Some(&ch) && !matches!(ch, 'h' | 'j' | 'v');
            if ch == 'n' && c.get(i + 1) == Some(&'g') {
                out.push_str("ŋk");
                i += 2;
                continue;
            }
            if ch == 'n' && c.get(i + 1) == Some(&'k') {
                out.push_str("ŋ");
                i += 1;
                continue;
            }
            out.push_str(ph);
            if dbl {
                out.push('ː');
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    out
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "null", "üks", "kaks", "kolm", "neli", "viis", "kuus", "seitse", "kaheksa", "üheksa", "kümme", "üksteist", "kaksteist",
    "kolmteist", "neliteist", "viisteist", "kuusteist", "seitseteist", "kaheksateist", "üheksateist",
];
const STEM: [&str; 10] = ["", "üks", "kaks", "kolm", "neli", "viis", "kuus", "seitse", "kaheksa", "üheksa"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    if h > 0 {
        out.push(if h == 1 { "sada".to_string() } else { format!("{}sada", STEM[h]) });
    }
    if r >= 20 {
        out.push(format!("{}kümmend", STEM[r / 10]));
        if r % 10 > 0 {
            out.push(ONES[r % 10].to_string());
        }
    } else if r > 0 || h == 0 {
        out.push(ONES[r].to_string());
    }
}

pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| ONES[d.to_digit(10).unwrap() as usize].to_string()).collect();
    }
    let mut rest = n;
    for (div, name) in [(1_000_000_000u128, "miljard"), (1_000_000, "miljon")] {
        let g = rest / div;
        if g > 0 {
            below_1000(g, &mut out);
            out.push(if g == 1 { name.to_string() } else { format!("{name}it") });
            rest %= div;
        }
    }
    let t = rest / 1000;
    if t > 0 {
        if t > 1 {
            below_1000(t, &mut out);
        }
        out.push("tuhat".to_string());
        rest %= 1000;
    }
    if rest > 0 || n == 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "protsenti",
        '$' => "dollarit",
        '€' => "eurot",
        '£' => "naela",
        '&' => "ja",
        '+' => "pluss",
        '@' => "ätt",
        '°' => "kraadi",
        '=' => "võrdub",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Et)
    }

    #[test]
    fn rules() {
        for (w, e) in [
            ("kool", "kˈoːl"), ("maa", "mˈɑː"), ("kapp", "kˈɑpː"), ("õde", "ˈɤde"), ("jäätis", "jˈæːtis"), ("tütar", "tˈytɑr"),
            ("linn", "lˈinː"), ("hing", "hˈiŋk"), ("tänav", "tˈænɑv"), ("ülikool", "ˈylikoːl"), ("šokolaad", "ʃˈokolɑːd"),
            ("ja", "jɑ"), ("tere", "tˈere"),
        ] {
            assert_eq!(p(w), e, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Et).join(" "), "kaks tuhat kakskümmend viis");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
