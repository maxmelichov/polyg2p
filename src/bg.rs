//! Bulgarian grapheme-to-phoneme rules, notation shared with `ru.rs` / `uk.rs`.
//!
//! Rules (Wikipedia, "Bulgarian phonology", and the Wiktionary Bulgarian pronunciation appendix,
//! both CC BY-SA, read 2026-11-28):
//! * stress is free and lexical. With the optional data pack it comes from `bg.lex` (word ->
//!   index of the stressed vowel from the start) and, for words the lexicon lacks, from the ending
//!   statistics `bg.sfx` (longest ending -> stressed vowel from the end). Without the pack the
//!   stress is on the penultimate vowel;
//! * vowel reduction follows stress: unstressed `а ъ` are `ɐ`, unstressed `о` is `ɔ`, stressed
//!   `ъ` is `ɤ` (the unstressed raising of `е`, `о` is not written);
//! * `щ` is `ʃt`, `ц ч ш ж` are `ts tʃ ʃ ʒ`, `дж дз` are affricates, `й` is `j`;
//! * `ю я` are `ju ja`, after a consonant they palatalise it (`ʲ`) and give `u a`; `ьо` is `ʲo`;
//! * word-final devoicing and regressive voicing assimilation.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "запетая";
pub const LETTERS: Option<&[&str; 26]> = None;

/// Function words read without a stress mark.
const UNSTRESSED: &[&str] = &[
    "и", "в", "във", "на", "с", "със", "за", "от", "до", "по", "при", "че", "да", "не", "се", "си", "ще", "бе", "ни",
    "ви", "го", "ги", "му", "ми", "ти", "то", "тя", "те", "ме", "ли", "а", "о", "у", "под", "над", "към", "без",
];

struct Res {
    lex: Option<Lexicon>,
    sfx: HashMap<String, usize>,
}

static RES: OnceLock<Res> = OnceLock::new();

fn res() -> &'static Res {
    RES.get_or_init(|| {
        let dir = crate::data_dir();
        let lex = dir.as_ref().map(|d| d.join("bg.lex")).filter(|p| p.exists()).and_then(|p| Lexicon::open(&p).ok());
        let mut sfx = HashMap::new();
        if let Some(p) = dir.map(|d| d.join("bg.sfx")).filter(|p| p.exists()) {
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

fn is_vowel(c: char) -> bool {
    matches!(c, 'а' | 'ъ' | 'о' | 'у' | 'е' | 'и' | 'ю' | 'я')
}

/// Index (from the start, among vowel letters) of the stressed vowel.
fn stress_index(lw: &str, nv: usize) -> usize {
    if nv <= 1 {
        return 0;
    }
    let r = res();
    if let Some(l) = &r.lex {
        if let Some(v) = l.get(lw) {
            if let Ok(i) = v.split('\t').next().unwrap_or(v).parse::<usize>() {
                if i < nv {
                    return i;
                }
            }
        }
    }
    let chars: Vec<char> = lw.chars().collect();
    for k in (1..=6.min(chars.len())).rev() {
        let s: String = chars[chars.len() - k..].iter().collect();
        if let Some(&fe) = r.sfx.get(&s) {
            if fe < nv {
                return nv - 1 - fe;
            }
        }
    }
    nv - 2
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

#[derive(Clone, PartialEq)]
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

pub fn word(w: &str) -> String {
    let lw = w.to_lowercase().replace('ѝ', "и");
    if lw.contains('-') {
        return lw.split('-').filter(|p| !p.is_empty()).map(word).filter(|p| !p.is_empty()).collect::<Vec<_>>().join("");
    }
    let c: Vec<char> = lw.chars().filter(|c| c.is_alphabetic()).collect();
    if c.is_empty() {
        return String::new();
    }
    let nv = c.iter().filter(|&&x| is_vowel(x)).count();
    let plain: String = c.iter().collect();
    let st = if UNSTRESSED.contains(&plain.as_str()) || nv == 0 { usize::MAX } else { stress_index(&plain, nv) };
    render(&c, st)
}

/// Rules with the stressed vowel (index among vowel letters) given; also used by the evaluation.
pub fn render(c: &[char], st: usize) -> String {
    let n = c.len();
    let mut segs: Vec<Seg> = Vec::new();
    let mut vi = 0usize; // vowel index
    let mut i = 0;
    let mut vowel_of_stress = vec![];
    let after_cons = |segs: &Vec<Seg>| segs.last().is_some_and(|s| s.k != K::V);
    while i < n {
        let ch = c[i];
        let nx = c.get(i + 1).copied();
        if is_vowel(ch) {
            let stressed = vi == st;
            vi += 1;
            let pal_cons = matches!(ch, 'ю' | 'я') && after_cons(&segs);
            if pal_cons {
                if let Some(last) = segs.last_mut() {
                    // only consonants that have a palatal series are marked
                    if !matches!(last.ph.as_str(), "ʒ" | "ʃ" | "tʃ" | "ts" | "dʒ" | "j") {
                        last.pal = true;
                    }
                }
            }
            let ph: &str = match ch {
                'а' => if stressed { "a" } else { "ɐ" },
                'ъ' => if stressed { "ɤ" } else { "ɐ" },
                'о' => if stressed { "o" } else { "ɔ" },
                'у' => "u",
                'е' => "ɛ",
                'и' => "i",
                'ю' => {
                    if !pal_cons {
                        segs.push(Seg { ph: "j".into(), k: K::Son, pal: false });
                    }
                    "u"
                }
                'я' => {
                    if !pal_cons {
                        segs.push(Seg { ph: "j".into(), k: K::Son, pal: false });
                    }
                    if stressed { "a" } else { "ɐ" }
                }
                _ => "",
            };
            segs.push(Seg { ph: ph.into(), k: K::V, pal: false });
            if stressed {
                vowel_of_stress.push(segs.len() - 1);
            }
            i += 1;
            continue;
        }
        let (ph, k): (&str, K) = match ch {
            'б' => ("b", K::Obs),
            'в' => ("v", K::Obs),
            'г' => ("ɡ", K::Obs),
            'д' => {
                if nx == Some('ж') {
                    i += 1;
                    ("dʒ", K::Obs)
                } else if nx == Some('з') && c.get(i + 2).is_some_and(|x| is_vowel(*x)) && false {
                    i += 1;
                    ("dz", K::Obs)
                } else {
                    ("d", K::Obs)
                }
            }
            'ж' => ("ʒ", K::Obs),
            'з' => ("z", K::Obs),
            'й' => ("j", K::Son),
            'к' => ("k", K::Obs),
            'л' => ("l", K::Son),
            'м' => ("m", K::Son),
            'н' => ("n", K::Son),
            'п' => ("p", K::Obs),
            'р' => ("r", K::Son),
            'с' => ("s", K::Obs),
            'т' => ("t", K::Obs),
            'ф' => ("f", K::Obs),
            'х' => ("x", K::Obs),
            'ц' => ("ts", K::Obs),
            'ч' => ("tʃ", K::Obs),
            'ш' => ("ʃ", K::Obs),
            'щ' => ("ʃt", K::Obs),
            'ь' => {
                // ьо: palatalises the previous consonant
                if let Some(last) = segs.last_mut() {
                    if last.k != K::V {
                        last.pal = true;
                    }
                }
                i += 1;
                continue;
            }
            _ => {
                i += 1;
                continue;
            }
        };
        if ph == "ʃt" {
            segs.push(Seg { ph: "ʃ".into(), k: K::Obs, pal: false });
            segs.push(Seg { ph: "t".into(), k: K::Obs, pal: false });
        } else {
            segs.push(Seg { ph: ph.into(), k, pal: false });
        }
        i += 1;
    }
    // regressive voicing, final devoicing
    let m = segs.len();
    for j in (0..m).rev() {
        if segs[j].k != K::Obs {
            continue;
        }
        let nxt = segs.get(j + 1);
        match nxt {
            None => {
                let v = voiceless_of(&segs[j].ph);
                if !v.is_empty() {
                    segs[j].ph = v.to_string();
                }
            }
            Some(nx) if nx.k == K::Obs => {
                let voiced_next = !voiceless_of(&nx.ph).is_empty() && nx.ph != "v";
                let rep = if voiced_next { voiced_of(&segs[j].ph) } else { voiceless_of(&segs[j].ph) };
                if !rep.is_empty() && nx.ph != "v" {
                    segs[j].ph = rep.to_string();
                }
            }
            _ => {}
        }
    }
    let mut out = String::new();
    for (j, sg) in segs.iter().enumerate() {
        if vowel_of_stress.contains(&j) {
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
    "нула", "едно", "две", "три", "четири", "пет", "шест", "седем", "осем", "девет", "десет", "единадесет", "дванадесет",
    "тринадесет", "четиринадесет", "петнадесет", "шестнадесет", "седемнадесет", "осемнадесет", "деветнадесет",
];
const TENS: [&str; 10] = ["", "", "двадесет", "тридесет", "четиридесет", "петдесет", "шестдесет", "седемдесет", "осемдесет", "деветдесет"];
const HUND: [&str; 10] = ["", "сто", "двеста", "триста", "четиристотин", "петстотин", "шестстотин", "седемстотин", "осемстотин", "деветстотин"];

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
    for (div, one, many) in [(1_000_000_000u128, "милиард", "милиарда"), (1_000_000, "милион", "милиона")] {
        let g = rest / div;
        if g > 0 {
            if g == 1 {
                out.push(format!("един {one}"));
            } else {
                below_1000(g, &mut out);
                out.push(many.to_string());
            }
            rest %= div;
        }
    }
    let t = rest / 1000;
    if t > 0 {
        if t == 1 {
            out.push("хиляда".to_string());
        } else {
            below_1000(t, &mut out);
            out.push("хиляди".to_string());
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
        '%' => "процента",
        '$' => "долара",
        '€' => "евро",
        '£' => "лири",
        '&' => "и",
        '+' => "плюс",
        '@' => "кльомба",
        '°' => "градуса",
        '=' => "равно",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Bg)
    }

    #[test]
    fn rules_given_stress() {
        // without a data pack the stress is penultimate: книга -> ˈi, писмо -> ˈi
        assert_eq!(p("книга"), "knˈiɡɐ");
        assert_eq!(p("щастие"), "ʃtɐstˈiɛ");
        assert_eq!(p("и"), "i");
        assert_eq!(p("път"), "pˈɤt");
        assert_eq!(p("юли"), "jˈuli");
        assert_eq!(p("земя"), "zˈɛmʲɐ");
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Bg).join(" "), "две хиляди двадесет пет");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
