//! Slovenian grapheme-to-phoneme: rules, notation shared with `cs.rs` / `pl.rs`.
//!
//! Rules (Wikipedia, "Slovene phonology", and the Wiktionary Slovene pronunciation appendix,
//! both CC BY-SA, read 2026-11-28):
//! * `č š ž` = `tʃ ʃ ʒ`, `c` = `ts`, `h` = `x`, `dž` = `dʒ`, `v` is `ʋ` before a vowel and the glide
//!   `u̯` before a consonant or at the end of a word, `n` is `ŋ` before `k g`;
//! * `r` between consonants is syllabic and written `ər`;
//! * obstruents devoice at the end of a word and assimilate in voicing to the next obstruent;
//! * stress, vowel length (`ː`, only under stress) and the vowel quality of `e o` (`ɛ e ə`, `ɔ o`) are
//!   lexical. With the optional data pack, `sl.lex` (word -> finished IPA), `sl.sfx` (ending -> stressed
//!   vowel counted from the end) and `sl.vq` (context -> vowel symbol) supply them; the lexicon wins,
//!   `sfx` and `vq` guess for words outside it. Without the pack the stress is on the penultimate vowel,
//!   every `e o` is open and nothing is long. The pitch accent is not written.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "vejica";
pub const LETTERS: Option<&[&str; 26]> = None;

/// Clitics and one-letter prepositions read without a stress mark.
const UNSTRESSED: &[&str] = &[
    "in", "a", "k", "s", "z", "v", "o", "u", "na", "za", "od", "do", "po", "pri", "ob", "iz", "ali", "da", "je", "so",
    "se", "si", "sem", "sva", "smo", "ste", "bi", "bo", "ne", "ni", "pa", "naj", "le", "mi", "ti", "ga", "jo", "ju",
    "mu", "ji", "nam", "vam", "jim", "me", "te", "nas", "vas", "jih",
];

struct Res {
    lex: Option<Lexicon>,
    sfx: HashMap<String, usize>,
    vq: HashMap<String, String>,
}
static RES: OnceLock<Res> = OnceLock::new();

fn res() -> &'static Res {
    RES.get_or_init(|| {
        let dir = crate::data_dir();
        let lex = dir.as_ref().map(|d| d.join("sl.lex")).filter(|p| p.exists()).and_then(|p| Lexicon::open(&p).ok());
        let mut sfx = HashMap::new();
        let mut vq = HashMap::new();
        if let Some(p) = dir.as_ref().map(|d| d.join("sl.sfx")).filter(|p| p.exists()) {
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
        if let Some(p) = dir.map(|d| d.join("sl.vq")).filter(|p| p.exists()) {
            if let Ok(t) = std::fs::read_to_string(p) {
                for l in t.lines() {
                    if let Some((k, v)) = l.split_once('\t') {
                        vq.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }
        Res { lex, sfx, vq }
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
}

fn voiced_of(p: &str) -> &'static str {
    match p {
        "p" => "b", "t" => "d", "k" => "ɡ", "f" => "v", "s" => "z", "ʃ" => "ʒ", "ts" => "dz", "tʃ" => "dʒ", "x" => "ɡ",
        _ => "",
    }
}

fn voiceless_of(p: &str) -> &'static str {
    match p {
        "b" => "p", "d" => "t", "ɡ" => "k", "v" => "f", "z" => "s", "ʒ" => "ʃ", "dz" => "ts", "dʒ" => "tʃ",
        _ => "",
    }
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// `r` between consonants (or at a word edge next to one) is a syllable nucleus.
fn syllabic_r(c: &[char], i: usize) -> bool {
    c[i] == 'r' && (i == 0 || !is_v(c[i - 1])) && (i + 1 == c.len() || !is_v(c[i + 1]))
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
    let r = res();
    let clitic = UNSTRESSED.contains(&plain.as_str());
    if !clitic {
        if let Some(l) = &r.lex {
            if let Some(v) = l.get(&plain) {
                return v.split('\t').next().unwrap_or(v).to_string();
            }
        }
    }
    render(&c, clitic)
}

fn vowel_sym(c: &[char], i: usize, stressed: bool, fin: bool) -> String {
    let ch = c[i];
    let r = res();
    let nx: String = c[i + 1..].iter().take(2).collect();
    let nx1: String = nx.chars().take(1).collect();
    let (s, f) = (stressed as u8, fin as u8);
    for key in [format!("{ch}|{s}|{f}|{nx}"), format!("{ch}|{s}|{f}|{nx1}"), format!("{ch}|{s}|{f}"), format!("{ch}|{s}")] {
        if let Some(v) = r.vq.get(&key) {
            return v.clone();
        }
    }
    match ch {
        'e' => "ɛ".into(),
        'o' => "ɔ".into(),
        x => x.to_string(),
    }
}

fn render(c: &[char], clitic: bool) -> String {
    let n = c.len();
    let nuc: Vec<usize> = (0..n).filter(|&i| is_v(c[i]) || syllabic_r(c, i)).collect();
    let nv = nuc.len();
    let r = res();
    let st = if clitic || nv == 0 {
        None
    } else if nv == 1 {
        Some(0)
    } else {
        let mut g = nv - 2;
        for k in (1..=8.min(n)).rev() {
            let suf: String = c[n - k..].iter().collect();
            if let Some(&fe) = r.sfx.get(&suf) {
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
    let mut vi = 0usize;
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = c.get(i + 1).copied();
        if is_v(ch) || syllabic_r(c, i) {
            let stressed = st == Some(vi);
            if stressed {
                stressed_seg = Some(segs.len());
            }
            if ch == 'r' {
                segs.push(Seg { ph: "ə".into(), k: K::V });
                segs.push(Seg { ph: "r".into(), k: K::Son });
            } else {
                segs.push(Seg { ph: vowel_sym(c, i, stressed, vi + 1 == nv), k: K::V });
            }
            vi += 1;
            i += 1;
            continue;
        }
        let cons_next = nx.is_some_and(|x| !is_v(x));
        let (ph, k, adv): (&str, K, usize) = match ch {
            'b' => ("b", K::Obs, 1),
            'c' => ("ts", K::Obs, 1),
            'č' => ("tʃ", K::Obs, 1),
            'ć' => ("tʃ", K::Obs, 1),
            'd' => if nx == Some('ž') { ("dʒ", K::Obs, 2) } else { ("d", K::Obs, 1) },
            'đ' => ("dʒ", K::Obs, 1),
            'f' => ("f", K::Obs, 1),
            'g' => ("ɡ", K::Obs, 1),
            'h' => ("x", K::Obs, 1),
            // lj / nj before a consonant or at the end are plain l / n (kralj, Kranj, Kostanjšek)
            'j' if i > 0 && matches!(c[i - 1], 'l' | 'n') && (nx.is_none() || cons_next) => ("", K::Son, 1),
            'j' => ("j", K::Son, 1),
            'k' => ("k", K::Obs, 1),
            'l' => ("l", K::Son, 1),
            'm' => ("m", K::Son, 1),
            'n' => if matches!(nx, Some('k') | Some('g')) { ("ŋ", K::Son, 1) } else { ("n", K::Son, 1) },
            'p' => ("p", K::Obs, 1),
            'r' => ("r", K::Son, 1),
            's' => ("s", K::Obs, 1),
            'š' => ("ʃ", K::Obs, 1),
            't' => ("t", K::Obs, 1),
            'v' => if nx.is_none() || cons_next { ("u̯", K::Son, 1) } else { ("ʋ", K::Son, 1) },
            'z' => ("z", K::Obs, 1),
            'ž' => ("ʒ", K::Obs, 1),
            'q' => ("k", K::Obs, 1),
            'w' => ("ʋ", K::Son, 1),
            'x' => ("ks", K::Obs, 1),
            'y' => ("i", K::V, 1),
            _ => ("", K::Son, 1),
        };
        if !ph.is_empty() {
            segs.push(Seg { ph: ph.to_string(), k });
        }
        i += adv;
    }
    // voicing: final devoicing and regressive assimilation
    let m = segs.len();
    for j in (0..m).rev() {
        if segs[j].k != K::Obs {
            continue;
        }
        let rep = match segs.get(j + 1) {
            None => voiceless_of(&segs[j].ph),
            Some(nx) if nx.k == K::Obs => {
                if !voiceless_of(&nx.ph).is_empty() {
                    voiced_of(&segs[j].ph)
                } else {
                    voiceless_of(&segs[j].ph)
                }
            }
            _ => "",
        };
        if !rep.is_empty() {
            segs[j].ph = rep.to_string();
        }
    }
    let mut out = String::new();
    for (j, sg) in segs.iter().enumerate() {
        if stressed_seg == Some(j) {
            out.push('ˈ');
        }
        out.push_str(&sg.ph);
    }
    out
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "nič", "ena", "dve", "tri", "štiri", "pet", "šest", "sedem", "osem", "devet", "deset", "enajst", "dvanajst",
    "trinajst", "štirinajst", "petnajst", "šestnajst", "sedemnajst", "osemnajst", "devetnajst",
];
const TENS: [&str; 10] = ["", "", "dvajset", "trideset", "štirideset", "petdeset", "šestdeset", "sedemdeset", "osemdeset", "devetdeset"];
const HUND: [&str; 10] = ["", "sto", "dvesto", "tristo", "štiristo", "petsto", "šeststo", "sedemsto", "osemsto", "devetsto"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    if h > 0 {
        out.push(HUND[h].to_string());
    }
    if r >= 20 {
        if r % 10 > 0 {
            // enaindvajset, triindvajset
            out.push(format!("{}in{}", ONES[r % 10], TENS[r / 10]));
        } else {
            out.push(TENS[r / 10].to_string());
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
    for (div, one, two, few, many) in [
        (1_000_000_000u128, "milijarda", "milijardi", "milijarde", "milijard"),
        (1_000_000, "milijon", "milijona", "milijone", "milijonov"),
    ] {
        let g = rest / div;
        if g > 0 {
            if g == 1 {
                out.push(format!("ena {one}").replace("ena milijon", "en milijon"));
            } else {
                below_1000(g, &mut out);
                out.push(if g % 100 == 2 { two } else if (3..=4).contains(&(g % 100)) { few } else { many }.to_string());
            }
            rest %= div;
        }
    }
    let t = rest / 1000;
    if t > 0 {
        if t == 1 {
            out.push("tisoč".to_string());
        } else {
            below_1000(t, &mut out);
            out.push("tisoč".to_string());
        }
        rest %= 1000;
    }
    if rest > 0 || n == 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "odstotkov",
        '$' => "dolarjev",
        '€' => "evrov",
        '£' => "funtov",
        '&' => "in",
        '+' => "plus",
        '@' => "afna",
        '°' => "stopinj",
        '=' => "je enako",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Sl)
    }

    #[test]
    fn rules_without_pack() {
        for (w, e) in [("čaj", "tʃˈaj"), ("hiša", "xˈiʃa"), ("v", "u̯"), ("vlak", "u̯lˈak"), ("trg", "tˈərk")] {
            assert_eq!(p(w).replace('ʧ', "tʃ"), e, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Sl).join(" "), "dve tisoč petindvajset");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
