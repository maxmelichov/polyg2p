//! Danish grapheme-to-phoneme: lexicon first, notation shared with `sv.rs` / `de.rs`.
//!
//! Danish spelling hides most of the sound (stød, soft `d`, reduced vowels, silent letters), so the module is
//! built around a pronunciation lexicon. Lookup order for a word:
//! 1. `da.lex` from the optional data pack (word -> IPA with stress and the stød mark `ˀ`);
//! 2. the char-level transformer `da.g2p` of the data pack (`tools/train_g2p.py`), greedy decoding;
//! 3. without a pack: plain letter rules (stress on the first vowel, no stød, no soft `d`).
//!
//! Notation sources: Wikipedia "Danish phonology" (CC BY-SA, read 2026-11-28). Stød is `ˀ` placed after the vowel
//! (or sonorant) that carries it, length `ː` on vowels, the strong-stress mark `ˈ` right before the vowel.

use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::nn::Model;
use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "komma";
pub const LETTERS: Option<&[&str; 26]> = None;

struct Res {
    lex: Option<Lexicon>,
    model: Option<Model>,
}
static RES: OnceLock<Res> = OnceLock::new();

fn res() -> &'static Res {
    RES.get_or_init(|| {
        let dir = crate::data_dir();
        let lex = dir.as_ref().map(|d| d.join("da.lex")).filter(|p| p.exists()).and_then(|p| Lexicon::open(&p).ok());
        let model = dir.map(|d| d.join("da.g2p")).filter(|p| p.exists()).and_then(|p| Model::open(&p).ok());
        Res { lex, model }
    })
}

fn lookup(w: &str) -> Option<String> {
    let l = res().lex.as_ref()?;
    let v = l.get(w)?;
    Some(v.split('\t').next().unwrap_or(v).to_string())
}

/// Join head and tail of a compound: the head keeps the primary stress, the tail's primary becomes secondary.
#[allow(dead_code)]
fn compound(head: &str, link: &str, tail: &str) -> String {
    format!("{head}{link}{}", tail.replace('ˈ', "ˌ"))
}

#[allow(dead_code)]
fn split_compound(w: &str) -> Option<String> {
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 8 {
        return None;
    }
    // longest tail first (so the head is the shortest word that still leaves a known tail)
    for i in 4..n.saturating_sub(3) {
        let (a, rest): (String, String) = (c[..i].iter().collect(), c[i..].iter().collect());
        for (link, lp) in [("", ""), ("s", "s"), ("e", "ə")] {
            let b: String = if link.is_empty() { rest.clone() } else if rest.starts_with(link) { rest[link.len()..].to_string() } else { continue };
            if b.chars().count() < 4 {
                continue;
            }
            if let (Some(pa), Some(pb)) = (lookup(&a), lookup(&b).or_else(|| split_compound(&b))) {
                return Some(compound(&pa, lp, &pb));
            }
        }
    }
    None
}

fn letters(c: &[char]) -> String {
    let mut out = String::new();
    let mut stressed = false;
    for (i, &ch) in c.iter().enumerate() {
        let v = match ch {
            'a' => "a", 'e' => "e", 'i' => "i", 'o' => "o", 'u' => "u", 'y' => "y", 'æ' => "ɛ", 'ø' => "ø", 'å' => "ɔ",
            'b' => "b", 'c' => "k", 'd' => "d", 'f' => "f", 'g' => "ɡ", 'h' => "h", 'j' => "j", 'k' => "k", 'l' => "l",
            'm' => "m", 'n' => "n", 'p' => "p", 'q' => "k", 'r' => "ʁ", 's' => "s", 't' => "t", 'v' => "v", 'w' => "v",
            'x' => "ks", 'z' => "s", _ => "",
        };
        let vowel = matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'æ' | 'ø' | 'å');
        if vowel && !stressed {
            out.push('ˈ');
            stressed = true;
        }
        let _ = i;
        out.push_str(v);
    }
    out
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
    let plain: String = c.iter().collect();
    if let Some(p) = lookup(&plain) {
        return p;
    }
    let r = res();
    if let Some(m) = &r.model {
        let p = m.predict(&plain);
        if !p.is_empty() {
            return p;
        }
    }
    letters(&c)
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "nul", "en", "to", "tre", "fire", "fem", "seks", "syv", "otte", "ni", "ti", "elleve", "tolv", "tretten", "fjorten",
    "femten", "seksten", "sytten", "atten", "nitten",
];
const TENS: [&str; 10] = ["", "", "tyve", "tredive", "fyrre", "halvtreds", "tres", "halvfjerds", "firs", "halvfems"];

fn below_100(r: usize, out: &mut Vec<String>) {
    if r >= 20 {
        if r % 10 > 0 {
            out.push(format!("{}og{}", ONES[r % 10], TENS[r / 10]));
        } else {
            out.push(TENS[r / 10].to_string());
        }
    } else {
        out.push(ONES[r].to_string());
    }
}

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    if h > 0 {
        out.push(if h == 1 { "et hundrede".to_string() } else { format!("{} hundrede", ONES[h]) });
        if r > 0 {
            out.push("og".to_string());
        }
    }
    if r > 0 || h == 0 {
        below_100(r, out);
    }
}

pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| ONES[d.to_digit(10).unwrap() as usize].to_string()).collect();
    }
    let mut rest = n;
    for (div, one, many) in [(1_000_000_000u128, "en milliard", "milliarder"), (1_000_000, "en million", "millioner"), (1_000, "et tusind", "tusind")] {
        let g = rest / div;
        if g > 0 {
            if g == 1 {
                out.push(one.to_string());
            } else {
                below_1000(g, &mut out);
                out.push(many.to_string());
            }
            rest %= div;
            if rest > 0 && rest < 100 {
                out.push("og".to_string());
            }
        }
    }
    if rest > 0 || n == 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "procent",
        '$' => "dollar",
        '€' => "euro",
        '£' => "pund",
        '&' => "og",
        '+' => "plus",
        '@' => "snabel a",
        '°' => "grader",
        '=' => "lig med",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Da)
    }

    #[test]
    fn plain_rules_without_pack() {
        assert_eq!(p("kat"), "kˈat");
        assert_eq!(p("sol"), "sˈol");
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Da).join(" "), "to tusind og femogtyve");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
