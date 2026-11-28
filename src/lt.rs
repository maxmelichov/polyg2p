//! Lithuanian grapheme-to-phoneme: rules, notation shared with `pl.rs` / `ru.rs` (palatalisation `ʲ`).
//!
//! Rules (Wikipedia, "Lithuanian phonology", and the Wiktionary Lithuanian pronunciation appendix,
//! both CC BY-SA, read 2026-11-28):
//! * `ą ę ė į y ū ų` are long (`aː ɛː eː iː iː uː uː`), `o` is long in native words, `ie uo ai au ei eu ui` are
//!   diphthongs; `ch` = `x`, `h` = `ɣ`, `dž dz` = `dʒ dz`, `č š ž` = `tʃ ʃ ʒ`;
//! * a consonant is palatalised (`ʲ`) before a front vowel (`e ę ė i į y ie`); `i` written between a consonant and a
//!   back vowel (`a o u ą ų ū`) is only the softening mark and has no sound;
//! * `n` is `ŋ` before `k g`; obstruent clusters assimilate in voicing (regressive);
//! * stress is free and mobile. With the optional data pack, `lt.lex` (word -> stressed syllable index) and
//!   `lt.sfx` (ending and syllable count -> stressed syllable from the end) give it; without the pack it is the penultimate
//!   syllable. The pitch accent (acute / circumflex) and the lengthening of stressed short vowels are not written.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "kablelis";
pub const LETTERS: Option<&[&str; 26]> = None;

const UNSTRESSED: &[&str] = &[
    "ir", "o", "bet", "ar", "kad", "jei", "nes", "su", "be", "iš", "per", "prie", "už", "nuo", "į", "apie", "po", "pas",
    "ant", "virš", "tai", "ne", "ni", "gi", "jau", "tik", "kaip", "kur", "kas", "man", "tau", "jam", "jai", "mes",
];

struct Res {
    lex: Option<Lexicon>,
    sfx: HashMap<String, usize>,
}
static RES: OnceLock<Res> = OnceLock::new();

fn res() -> &'static Res {
    RES.get_or_init(|| {
        let dir = crate::data_dir();
        let lex = dir.as_ref().map(|d| d.join("lt.lex")).filter(|p| p.exists()).and_then(|p| Lexicon::open(&p).ok());
        let mut sfx = HashMap::new();
        if let Some(p) = dir.map(|d| d.join("lt.sfx")).filter(|p| p.exists()) {
            if let Ok(t) = std::fs::read_to_string(p) {
                for l in t.lines() {
                    let mut f = l.split('\t');
                    if let (Some(s), Some(n)) = (f.next(), f.next()) {
                        if let Ok(n) = n.parse() {
                            sfx.insert(s.to_string(), n);
                        }
                    }
                }
            }
        }
        Res { lex, sfx }
    })
}

#[derive(Clone, Copy, PartialEq)]
enum K {
    V,
    Son,
    Obs,
}

struct Seg {
    ph: String,
    k: K,
    pal: bool,
}

fn voiced_of(p: &str) -> &'static str {
    match p {
        "p" => "b", "t" => "d", "k" => "ɡ", "f" => "v", "s" => "z", "ʃ" => "ʒ", "ts" => "dz", "tʃ" => "dʒ", "x" => "ɣ",
        _ => "",
    }
}

fn voiceless_of(p: &str) -> &'static str {
    match p {
        "b" => "p", "d" => "t", "ɡ" => "k", "v" => "f", "z" => "s", "ʒ" => "ʃ", "dz" => "ts", "dʒ" => "tʃ", "ɣ" => "x",
        _ => "",
    }
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'ą' | 'e' | 'ę' | 'ė' | 'i' | 'į' | 'y' | 'o' | 'u' | 'ų' | 'ū')
}

fn front(c: char) -> bool {
    matches!(c, 'e' | 'ę' | 'ė' | 'i' | 'į' | 'y')
}

/// `i` between a consonant and a back vowel only softens the consonant.
fn soft_mark(c: &[char], i: usize) -> bool {
    c[i] == 'i' && i > 0 && !is_v(c[i - 1]) && matches!(c.get(i + 1), Some('a' | 'o' | 'u' | 'ą' | 'ų' | 'ū'))
}

/// Syllable nuclei as char spans: vowels, diphthongs, and the mixed diphthongs `a e i u` + `l m n r`.
fn nuclei(c: &[char]) -> Vec<(usize, usize)> {
    let n = c.len();
    let mut r: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < n {
        if is_v(c[i]) {
            if soft_mark(c, i) {
                i += 1;
                continue;
            }
            if i + 1 < n && matches!((c[i], c[i + 1]), ('a', 'i') | ('a', 'u') | ('e', 'i') | ('e', 'u') | ('u', 'i') | ('i', 'e') | ('u', 'o')) {
                r.push((i, i + 2));
                i += 2;
                continue;
            }
            r.push((i, i + 1));
        }
        i += 1;
    }
    r.iter()
        .map(|&(a, b)| {
            if b - a == 1 && matches!(c[a], 'a' | 'e' | 'i' | 'u') && b < n && matches!(c[b], 'l' | 'm' | 'n' | 'r') && (b + 1 == n || !is_v(c[b + 1])) {
                (a, b + 1)
            } else {
                (a, b)
            }
        })
        .collect()
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
    let mut stress: Option<usize> = None;
    if let Some(l) = &res().lex {
        if let Some(v) = l.get(&plain) {
            stress = v.split('\t').next().and_then(|x| x.parse().ok());
        }
    }
    render(&c, stress, UNSTRESSED.contains(&plain.as_str()))
}

fn vowel(c: &[char]) -> &'static str {
    match c.iter().collect::<String>().as_str() {
        "a" => "a", "ą" => "aː", "e" => "ɛ", "ę" => "ɛː", "ė" => "eː", "i" => "ɪ", "į" | "y" => "iː", "o" => "oː", "u" => "ʊ",
        "ų" | "ū" => "uː", "ie" => "ie", "uo" => "uo", "ai" => "ai", "au" => "au", "ei" => "ei", "eu" => "ɛu", "ui" => "ui",
        _ => "",
    }
}

fn render(c: &[char], stress: Option<usize>, clitic: bool) -> String {
    let n = c.len();
    let nu = nuclei(c);
    let nv = nu.len();
    let r = res();
    let st = if clitic || nv == 0 {
        None
    } else if let Some(s) = stress.filter(|&s| s < nv) {
        Some(s)
    } else if nv == 1 {
        Some(0)
    } else {
        let mut g = nv - 2;
        for k in (1..=8.min(n)).rev() {
            let suf: String = c[n - k..].iter().collect();
            if let Some(&fe) = r.sfx.get(&format!("{suf}|{}", nv.min(4))) {
                if fe < nv {
                    g = nv - 1 - fe;
                    break;
                }
            }
        }
        Some(g)
    };
    let mut segs: Vec<Seg> = Vec::new();
    let mut stressed_seg = None;
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = c.get(i + 1).copied();
        if let Some(k) = nu.iter().position(|&(a, _)| a == i) {
            let (a, b) = nu[k];
            let vlen = if b - a == 2 && !(matches!(c[a + 1], 'i' | 'u' | 'e' | 'o')) { 1 } else { b - a };
            let _ = vlen;
            let (sa, sb) = if b - a == 2 && !matches!(c[a..b].iter().collect::<String>().as_str(), "ai" | "au" | "ei" | "eu" | "ui" | "ie" | "uo") { (a, a + 1) } else { (a, b) };
            if st == Some(k) {
                stressed_seg = Some(segs.len());
            }
            let mut v = vowel(&c[sa..sb]).to_string();
            // `a` after `j` or after a softening `i` is fronted: ja -> jɛ, ia -> ʲɛ
            if i > 0 && (c[i - 1] == 'j' || soft_mark(c, i - 1)) && v.starts_with('a') {
                v = v.replacen('a', "ɛ", 1);
            }
            segs.push(Seg { ph: v, k: K::V, pal: false });
            i = sb;
            continue;
        }
        if is_v(ch) {
            // softening `i` or a vowel outside the nucleus list (second vowel of a hiatus is its own nucleus above)
            i += 1;
            continue;
        }
        // palatalised before a front vowel, or before the softening `i`
        let pal = match nx {
            Some(x) if front(x) => true,
            _ => false,
        };
        let (ph, k, adv): (&str, K, usize) = match ch {
            'b' => ("b", K::Obs, 1),
            'c' => ("ts", K::Obs, 1),
            'č' => ("tʃ", K::Obs, 1),
            'd' => match nx {
                Some('ž') => ("dʒ", K::Obs, 2),
                Some('z') => ("dz", K::Obs, 2),
                _ => ("d", K::Obs, 1),
            },
            'f' => ("f", K::Obs, 1),
            'g' => ("ɡ", K::Obs, 1),
            'h' => ("ɣ", K::Obs, 1),
            'j' => ("j", K::Son, 1),
            'k' => ("k", K::Obs, 1),
            'l' => ("l", K::Son, 1),
            'm' => ("m", K::Son, 1),
            'n' => if matches!(nx, Some('k') | Some('g')) { ("ŋ", K::Son, 1) } else { ("n", K::Son, 1) },
            'p' => ("p", K::Obs, 1),
            'r' => ("r", K::Son, 1),
            's' => if nx == Some('h') { ("ʃ", K::Obs, 2) } else { ("s", K::Obs, 1) },
            'š' => ("ʃ", K::Obs, 1),
            't' => ("t", K::Obs, 1),
            'v' => ("ʋ", K::Son, 1),
            'z' => ("z", K::Obs, 1),
            'ž' => ("ʒ", K::Obs, 1),
            'w' => ("ʋ", K::Son, 1),
            'x' => ("ks", K::Obs, 1),
            'q' => ("k", K::Obs, 1),
            _ => ("", K::Son, 1),
        };
        // `ch` is one phone
        let (ph, k, adv) = if ch == 'c' && nx == Some('h') { ("x", K::Obs, 2) } else { (ph, k, adv) };
        // a softening `i` after this consonant (and before a back vowel) also palatalises it
        let soft = c.get(i + adv).copied() == Some('i') && soft_mark(c, i + adv);
        let pal2 = match c.get(i + adv) {
            Some(&x) if front(x) && !soft_mark(c, i + adv) => true,
            _ => soft,
        };
        let _ = pal;
        if !ph.is_empty() {
            segs.push(Seg { ph: ph.to_string(), k, pal: pal2 && ph != "j" });
        }
        i += adv;
    }
    // regressive voicing assimilation (no final devoicing)
    let m = segs.len();
    for j in (0..m).rev() {
        if segs[j].k != K::Obs {
            continue;
        }
        if let Some(nx) = segs.get(j + 1) {
            if nx.k == K::Obs {
                let rep = if !voiceless_of(&nx.ph).is_empty() { voiced_of(&segs[j].ph) } else { voiceless_of(&segs[j].ph) };
                if !rep.is_empty() {
                    segs[j].ph = rep.to_string();
                }
            }
        }
    }
    // palatalisation spreads left over the consonants of a cluster
    for j in (0..m.saturating_sub(1)).rev() {
        if segs[j].k != K::V && !matches!(segs[j].ph.as_str(), "j" | "k" | "ɡ" | "ŋ" | "x" | "ɣ") && segs[j + 1].k != K::V && segs[j + 1].pal {
            segs[j].pal = true;
        }
    }
    let mut out = String::new();
    for (j, sg) in segs.iter().enumerate() {
        if stressed_seg == Some(j) {
            out.push('ˈ');
        }
        out.push_str(&sg.ph);
        if sg.pal {
            out.push('ʲ');
        }
    }
    out
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "nulis", "vienas", "du", "trys", "keturi", "penki", "šeši", "septyni", "aštuoni", "devyni", "dešimt", "vienuolika",
    "dvylika", "trylika", "keturiolika", "penkiolika", "šešiolika", "septyniolika", "aštuoniolika", "devyniolika",
];
const TENS: [&str; 10] = ["", "", "dvidešimt", "trisdešimt", "keturiasdešimt", "penkiasdešimt", "šešiasdešimt", "septyniasdešimt", "aštuoniasdešimt", "devyniasdešimt"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    if h > 0 {
        if h > 1 {
            out.push(ONES[h].to_string());
            out.push("šimtai".to_string());
        } else {
            out.push("šimtas".to_string());
        }
    }
    if r >= 20 {
        out.push(TENS[r / 10].to_string());
        if r % 10 > 0 {
            out.push(ONES[r % 10].to_string());
        }
    } else if r > 0 || h == 0 {
        out.push(ONES[r].to_string());
    }
}

fn scale(g: u128, one: &str, few: &str, many: &str, out: &mut Vec<String>) {
    if g == 1 {
        out.push(one.to_string());
    } else {
        below_1000(g, out);
        out.push(if (11..=19).contains(&(g % 100)) || g % 10 == 0 { many } else { few }.to_string());
    }
}

pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| ONES[d.to_digit(10).unwrap() as usize].to_string()).collect();
    }
    let mut rest = n;
    for (div, one, few, many) in [
        (1_000_000_000u128, "milijardas", "milijardai", "milijardų"),
        (1_000_000, "milijonas", "milijonai", "milijonų"),
        (1_000, "tūkstantis", "tūkstančiai", "tūkstančių"),
    ] {
        let g = rest / div;
        if g > 0 {
            scale(g, one, few, many, &mut out);
            rest %= div;
        }
    }
    if rest > 0 || n == 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "procentų",
        '$' => "dolerių",
        '€' => "eurų",
        '£' => "svarų",
        '&' => "ir",
        '+' => "plius",
        '@' => "eta",
        '°' => "laipsnių",
        '=' => "lygu",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Lt)
    }

    #[test]
    fn rules_without_pack() {
        for (w, e) in [("žvakė", "ʒʋˈakʲeː"), ("šuo", "ʃˈuo"), ("ir", "ɪr")] {
            assert_eq!(p(w).replace('ʧ', "tʃ"), e, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Lt).join(" "), "du tūkstančiai dvidešimt penki");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
