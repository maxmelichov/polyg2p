//! Croatian (Latin script) grapheme-to-phoneme, notation shared with `cs.rs` / `pl.rs`.
//!
//! Rules (Wikipedia, "Serbo-Croatian phonology", and the Wiktionary Serbo-Croatian pronunciation
//! appendix, both CC BY-SA, read 2026-11-28):
//! * segments are almost one letter to one phone: `lj nj dž` digraphs, `č ć đ š ž` = `tʃ tɕ dʑ ʃ ʒ`,
//!   `c` = `ts`, `h` = `x`, `j` = `j`;
//! * `r` between consonants is syllabic (`r̩`);
//! * regressive voicing assimilation in obstruent clusters (final obstruents keep their voicing);
//! * stress and vowel length are lexical. With the optional data pack, `hr.lex` (word ->
//!   `stressed vowel|S/L per vowel`) gives both, and `hr.sfx` (ending -> stressed vowel from the end)
//!   gives a stress guess for other words, without length. Without the pack the stress is on the
//!   first syllable and no length is written. The pitch accent (rising / falling) is not written.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "zarez";
pub const LETTERS: Option<&[&str; 26]> = None;

/// Clitics read without a stress mark.
const UNSTRESSED: &[&str] = &[
    "i", "a", "u", "o", "na", "za", "od", "do", "po", "uz", "iz", "s", "sa", "k", "ka", "v", "je", "su", "se", "si",
    "sam", "smo", "ste", "ću", "ćeš", "će", "ćemo", "ćete", "bi", "li", "da", "ni", "ne", "mi", "ti", "ga", "ju", "ih",
    "mu", "joj", "nam", "vam", "im", "te", "ali", "pa", "što", "kao",
];

struct Res {
    lex: Option<Lexicon>,
    sfx: HashMap<String, usize>,
}
static RES: OnceLock<Res> = OnceLock::new();

fn res() -> &'static Res {
    RES.get_or_init(|| {
        let dir = crate::data_dir();
        let lex = dir.as_ref().map(|d| d.join("hr.lex")).filter(|p| p.exists()).and_then(|p| Lexicon::open(&p).ok());
        let mut sfx = HashMap::new();
        if let Some(p) = dir.map(|d| d.join("hr.sfx")).filter(|p| p.exists()) {
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
}

fn voiced_of(p: &str) -> &'static str {
    match p {
        "p" => "b", "t" => "d", "k" => "ɡ", "f" => "v", "s" => "z", "ʃ" => "ʒ", "ts" => "dz", "tʃ" => "dʒ", "tɕ" => "dʑ",
        "x" => "ɡ",
        _ => "",
    }
}

fn voiceless_of(p: &str) -> &'static str {
    match p {
        "b" => "p", "d" => "t", "ɡ" => "k", "v" => "f", "z" => "s", "ʒ" => "ʃ", "dz" => "ts", "dʒ" => "tʃ", "dʑ" => "tɕ",
        _ => "",
    }
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
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
    let mut stress: Option<usize> = None;
    let mut lens: Option<Vec<bool>> = None;
    if let Some(l) = &r.lex {
        if let Some(v) = l.get(&plain) {
            let v = v.split('\t').next().unwrap_or(v);
            if let Some((a, b)) = v.split_once('|') {
                stress = a.parse().ok();
                lens = Some(b.chars().map(|x| x == 'L').collect());
            }
        }
    }
    let clitic = UNSTRESSED.contains(&plain.as_str());
    render(&c, stress, lens.as_deref(), clitic)
}

fn syllabic_r(c: &[char], i: usize) -> bool {
    c[i] == 'r'
        && (i == 0 || (!is_v(c[i - 1]) && c[i - 1] != 'j'))
        && (i + 1 == c.len() || (!is_v(c[i + 1]) && c[i + 1] != 'j'))
}

fn render(c: &[char], stress: Option<usize>, lens: Option<&[bool]>, clitic: bool) -> String {
    let n = c.len();
    let nv = (0..n).filter(|&i| is_v(c[i]) || syllabic_r(c, i)).count();
    let r = res();
    let st = if clitic || nv == 0 {
        None
    } else if let Some(s) = stress.filter(|&s| s < nv) {
        Some(s)
    } else {
        let plain: String = c.iter().collect();
        let mut g = 0usize;
        'o: for k in (1..=6.min(n)).rev() {
            let suf: String = c[n - k..].iter().collect();
            if let Some(&fe) = r.sfx.get(&suf) {
                if fe < nv {
                    g = nv - 1 - fe;
                    break 'o;
                }
            }
        }
        let _ = plain;
        Some(g)
    };
    let mut segs: Vec<Seg> = Vec::new();
    let mut vi = 0usize;
    let mut i = 0;
    let mut stressed_seg = None;
    while i < n {
        let ch = c[i];
        let nx = c.get(i + 1).copied();
        if is_v(ch) || syllabic_r(c, i) {
            let long = lens.is_some_and(|l| l.get(vi).copied().unwrap_or(false));
            let base: &str = match ch {
                'a' => "a",
                'e' => "e",
                'i' => "i",
                'o' => "o",
                'u' => "u",
                _ => "r̩",
            };
            if st == Some(vi) {
                stressed_seg = Some(segs.len());
            }
            vi += 1;
            segs.push(Seg { ph: if long { format!("{base}ː") } else { base.to_string() }, k: K::V });
            i += 1;
            continue;
        }
        let (ph, k, adv): (&str, K, usize) = match ch {
            'b' => ("b", K::Obs, 1),
            'c' => ("ts", K::Obs, 1),
            'č' => ("tʃ", K::Obs, 1),
            'ć' => ("tɕ", K::Obs, 1),
            'd' => if nx == Some('ž') { ("dʒ", K::Obs, 2) } else { ("d", K::Obs, 1) },
            'đ' => ("dʑ", K::Obs, 1),
            'f' => ("f", K::Obs, 1),
            'g' => ("ɡ", K::Obs, 1),
            'h' => ("x", K::Obs, 1),
            'j' => ("j", K::Son, 1),
            'k' => ("k", K::Obs, 1),
            'l' => if nx == Some('j') { ("ʎ", K::Son, 2) } else { ("l", K::Son, 1) },
            'm' => ("m", K::Son, 1),
            'n' => if nx == Some('j') { ("ɲ", K::Son, 2) } else { ("n", K::Son, 1) },
            'p' => ("p", K::Obs, 1),
            'r' => ("r", K::Son, 1),
            's' => ("s", K::Obs, 1),
            'š' => ("ʃ", K::Obs, 1),
            't' => ("t", K::Obs, 1),
            'v' => ("v", K::Son, 1),
            'z' => ("z", K::Obs, 1),
            'ž' => ("ʒ", K::Obs, 1),
            'q' => ("k", K::Obs, 1),
            'w' => ("v", K::Son, 1),
            'x' => ("ks", K::Obs, 1),
            'y' => ("i", K::V, 1),
            _ => ("", K::Son, 1),
        };
        if !ph.is_empty() {
            segs.push(Seg { ph: ph.to_string(), k });
        }
        i += adv;
    }
    // regressive voicing assimilation
    let m = segs.len();
    for j in (0..m).rev() {
        if segs[j].k != K::Obs {
            continue;
        }
        match segs.get(j + 1) {
            // no final devoicing: the gold keeps word-final voiced stops voiced (bez, dug)
            None => {}
            Some(nx) if nx.k == K::Obs => {
                let voiced_next = !voiceless_of(&nx.ph).is_empty();
                let rep = if voiced_next { voiced_of(&segs[j].ph) } else { voiceless_of(&segs[j].ph) };
                if !rep.is_empty() {
                    segs[j].ph = rep.to_string();
                }
            }
            _ => {}
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
    "nula", "jedan", "dva", "tri", "četiri", "pet", "šest", "sedam", "osam", "devet", "deset", "jedanaest", "dvanaest",
    "trinaest", "četrnaest", "petnaest", "šesnaest", "sedamnaest", "osamnaest", "devetnaest",
];
const TENS: [&str; 10] = ["", "", "dvadeset", "trideset", "četrdeset", "pedeset", "šezdeset", "sedamdeset", "osamdeset", "devedeset"];
const HUND: [&str; 10] = ["", "sto", "dvjesto", "tristo", "četiristo", "petsto", "šesto", "sedamsto", "osamsto", "devetsto"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    if h > 0 {
        out.push(HUND[h].to_string());
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

pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| ONES[d.to_digit(10).unwrap() as usize].to_string()).collect();
    }
    let mut rest = n;
    for (div, one, few, many) in [(1_000_000_000u128, "milijarda", "milijarde", "milijardi"), (1_000_000, "milijun", "milijuna", "milijuna")] {
        let g = rest / div;
        if g > 0 {
            if g == 1 {
                out.push(format!("jedan {one}"));
            } else {
                below_1000(g, &mut out);
                out.push(if (2..=4).contains(&(g % 10)) && !(12..=14).contains(&(g % 100)) { few } else { many }.to_string());
            }
            rest %= div;
        }
    }
    let t = rest / 1000;
    if t > 0 {
        if t == 1 {
            out.push("tisuću".to_string());
        } else {
            below_1000(t, &mut out);
            // tisuća is feminine: ... jedna / dvije tisuće
            if let Some(last) = out.last_mut() {
                if last == "jedan" {
                    *last = "jedna".to_string();
                } else if last == "dva" {
                    *last = "dvije".to_string();
                }
            }
            out.push(if (2..=4).contains(&(t % 10)) && !(12..=14).contains(&(t % 100)) { "tisuće" } else { "tisuća" }.to_string());
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
        '%' => "posto",
        '$' => "dolara",
        '€' => "eura",
        '£' => "funti",
        '&' => "i",
        '+' => "plus",
        '@' => "et",
        '°' => "stupnjeva",
        '=' => "jednako",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Hr)
    }

    #[test]
    fn rules_without_pack() {
        for (w, e) in [
            ("ljubav", "ʎˈubav"), ("njiva", "ɲˈiva"), ("čovjek", "tʃˈovjek"), ("đak", "dʑˈak"), ("hrvatski", "xˈr̩vatski"),
            ("grad", "ɡrˈad"), ("i", "i"), ("džep", "dʒˈep"), ("svijet", "svˈijet"),
        ] {
            assert_eq!(p(w).replace('ʧ', "tʃ").replace('ʤ', "dʒ"), e.replace('ʧ', "tʃ").replace('ʤ', "dʒ"), "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Hr).join(" "), "dvije tisuće dvadeset pet");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
