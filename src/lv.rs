//! Latvian grapheme-to-phoneme rules, notation shared with `pl.rs` / `ru.rs` (palatals as one phone).
//!
//! Rules (Wikipedia, "Latvian phonology", and the Wiktionary Latvian pronunciation appendix,
//! both CC BY-SA, read 2026-11-28):
//! * stress is on the first syllable of a word (the pitch accent is not marked: tier A);
//! * `ā ē ī ū` are long (`ː`), `ie` is a diphthong, native `o` is `uo` (loans keep `o`);
//! * `e ē` are open `æ` before a back vowel in the next syllable, otherwise closed `ɛ`; the
//!   quality is lexical in the language, this rule is a stand-in without a lexicon;
//! * `ķ ģ ļ ņ` are the palatals `c ɟ ʎ ɲ`; `c č dz dž š ž` are `ts tʃ dz dʒ ʃ ʒ`; `h` is `x`;
//! * a doubled consonant is long; `n` before `k g` is `ŋ`;
//! * `v` before a consonant or at the end is the glide `w`; word-final `-šs -žs` is one sibilant;
//! * obstruent clusters assimilate in voicing (regressive).

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "komats";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "aː", "beː", "tseː", "deː", "eː", "ef", "ɡeː", "xaː", "iː", "jɔt", "kaː", "el", "em", "en", "oː", "peː", "kuː",
    "er", "es", "teː", "uː", "veː", "dubultveː", "iks", "iɡrek", "zeː",
]);

/// Function words read without a stress mark.
const UNSTRESSED: &[&str] = &[
    "un", "ir", "es", "tu", "uz", "no", "ar", "pa", "pie", "par", "pēc", "bet", "vai", "ka", "kā", "ne", "arī", "līdz",
    "jo", "kad", "tas", "tā", "to", "ko", "kas", "man", "mans", "viņš", "viņa",
];

const BACK: &str = "aāoōuū";
const FRONT: &str = "eēiī";

fn voiced_of(p: &str) -> &'static str {
    match p {
        "p" => "b", "t" => "d", "c" => "ɟ", "k" => "ɡ", "f" => "v", "s" => "z", "ʃ" => "ʒ", "x" => "h", "ts" => "dz",
        "tʃ" => "dʒ",
        _ => "",
    }
}

fn voiceless_of(p: &str) -> &'static str {
    match p {
        "b" => "p", "d" => "t", "ɟ" => "c", "ɡ" => "k", "v" => "f", "z" => "s", "ʒ" => "ʃ", "h" => "x", "dz" => "ts",
        "dʒ" => "tʃ",
        _ => "",
    }
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

fn seg(ph: &str, k: K) -> Seg {
    Seg { ph: ph.to_string(), k }
}

/// A loan-shaped word keeps `o` (letters foreign to native vocabulary or loan endings).
fn loan_shape(c: &[char]) -> bool {
    let w: String = c.iter().collect();
    c.iter().any(|x| matches!(x, 'f' | 'h' | 'x' | 'w' | 'y' | 'q'))
        || ["ors", "ors", "ijs", "ija", "cija", "ēts", "ents", "ānts", "ists", "isms", "ika", "ons", "fons"].iter().any(|s| w.ends_with(s))
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
    let loan = loan_shape(&c);
    let mut segs: Vec<Seg> = Vec::new();
    let n = c.len();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = c.get(i + 1).copied();
        match ch {
            'a' => segs.push(seg("a", K::V)),
            'ā' => segs.push(seg("aː", K::V)),
            'e' | 'ē' => {
                // first vowel of the following syllable
                let fv = c[i + 1..].iter().copied().find(|x| BACK.contains(*x) || FRONT.contains(*x));
                let open = fv.is_some_and(|x| BACK.contains(x)) && !loan;
                let (s, l) = if open { ("æ", "æː") } else { ("ɛ", "ɛː") };
                segs.push(seg(if ch == 'ē' { l } else { s }, K::V));
            }
            'i' | 'ī' => {
                if ch == 'i' && nx == Some('e') {
                    segs.push(seg("ie", K::V));
                    i += 1;
                } else {
                    segs.push(seg(if ch == 'ī' { "iː" } else { "i" }, K::V));
                }
            }
            'o' => segs.push(seg(if loan { "o" } else { "uo" }, K::V)),
            'ō' => segs.push(seg("oː", K::V)),
            'u' | 'ū' => segs.push(seg(if ch == 'ū' { "uː" } else { "u" }, K::V)),
            'b' => segs.push(seg("b", K::Obs)),
            'c' => segs.push(seg("ts", K::Obs)),
            'č' => segs.push(seg("tʃ", K::Obs)),
            'd' => {
                if nx == Some('z') {
                    segs.push(seg("dz", K::Obs));
                    i += 1;
                } else if nx == Some('ž') {
                    segs.push(seg("dʒ", K::Obs));
                    i += 1;
                } else {
                    segs.push(seg("d", K::Obs));
                }
            }
            'f' => segs.push(seg("f", K::Obs)),
            'g' => segs.push(seg("ɡ", K::Obs)),
            'ģ' => segs.push(seg("ɟ", K::Obs)),
            'h' => segs.push(seg("x", K::Obs)),
            'j' => segs.push(seg("j", K::Son)),
            'k' => segs.push(seg("k", K::Obs)),
            'ķ' => segs.push(seg("c", K::Obs)),
            'l' => segs.push(seg("l", K::Son)),
            'ļ' => segs.push(seg("ʎ", K::Son)),
            'm' => segs.push(seg("m", K::Son)),
            'n' => segs.push(seg(if matches!(nx, Some('k' | 'g')) { "ŋ" } else { "n" }, K::Son)),
            'ņ' => segs.push(seg("ɲ", K::Son)),
            'p' => segs.push(seg("p", K::Obs)),
            'r' => segs.push(seg("r", K::Son)),
            's' => segs.push(seg("s", K::Obs)),
            'š' => segs.push(seg("ʃ", K::Obs)),
            't' => segs.push(seg("t", K::Obs)),
            'v' | 'w' => {
                // coda v (before a consonant or at the end) is a glide
                let coda = !matches!(nx, Some(x) if BACK.contains(x) || FRONT.contains(x));
                segs.push(seg(if coda { "w" } else { "v" }, K::Son));
            }
            'x' => {
                segs.push(seg("k", K::Obs));
                segs.push(seg("s", K::Obs));
            }
            'y' => segs.push(seg("i", K::V)),
            'z' => segs.push(seg("z", K::Obs)),
            'ž' => segs.push(seg("ʒ", K::Obs)),
            'q' => segs.push(seg("k", K::Obs)),
            _ => {}
        }
        // doubled consonant: long
        if let Some(last) = segs.last() {
            if last.k != K::V && c.get(i + 1) == Some(&ch) && !matches!(ch, 'd' | 'j') {
                let k = last.k;
                let ph = format!("{}ː", last.ph);
                segs.pop();
                segs.push(Seg { ph, k });
                i += 1;
            }
        }
        i += 1;
    }
    // -šs / -žs at the end of a word: one sibilant (ašs, ciešs)
    if segs.len() >= 2 && segs[segs.len() - 1].ph == "s" && matches!(segs[segs.len() - 2].ph.as_str(), "ʃ" | "ʒ") {
        segs.pop();
    }
    // regressive voicing assimilation in obstruent clusters
    for j in (0..segs.len().saturating_sub(1)).rev() {
        if segs[j].k == K::Obs && segs[j + 1].k == K::Obs {
            let nx = segs[j + 1].ph.clone();
            let nxb = nx.trim_end_matches('ː');
            let want_voiced = !voiceless_of(nxb).is_empty();
            let cur = segs[j].ph.clone();
            let curb = cur.trim_end_matches('ː');
            let rep = if want_voiced { voiced_of(curb) } else { voiceless_of(curb) };
            if !rep.is_empty() {
                let suffix = &cur[curb.len()..];
                segs[j].ph = format!("{rep}{suffix}");
            }
        }
    }
    let mut out = String::new();
    let mut stressed = !UNSTRESSED.contains(&lw.as_str());
    for sg in &segs {
        if sg.k == K::V && stressed {
            out.push('ˈ');
            stressed = false;
        }
        out.push_str(&sg.ph);
    }
    out
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "nulle", "viens", "divi", "trīs", "četri", "pieci", "seši", "septiņi", "astoņi", "deviņi", "desmit", "vienpadsmit",
    "divpadsmit", "trīspadsmit", "četrpadsmit", "piecpadsmit", "sešpadsmit", "septiņpadsmit", "astoņpadsmit",
    "deviņpadsmit",
];
const STEM: [&str; 10] = ["", "vien", "div", "trīs", "četr", "piec", "seš", "septiņ", "astoņ", "deviņ"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    if h > 0 {
        out.push(if h == 1 { "simts".to_string() } else { format!("{}simt", STEM[h]) });
    }
    if r >= 20 {
        out.push(format!("{}desmit", STEM[r / 10]));
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
    for (div, one, many) in [(1_000_000_000u128, "miljards", "miljardi"), (1_000_000, "miljons", "miljoni")] {
        let g = rest / div;
        if g > 0 {
            below_1000(g, &mut out);
            out.push(if g == 1 { one.to_string() } else { many.to_string() });
            rest %= div;
        }
    }
    let t = rest / 1000;
    if t > 0 {
        if t == 1 {
            out.push("tūkstotis".to_string());
        } else {
            below_1000(t, &mut out);
            out.push("tūkstoši".to_string());
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
        '%' => "procenti",
        '$' => "dolāri",
        '€' => "eiro",
        '£' => "sterliņu mārciņas",
        '&' => "un",
        '+' => "plus",
        '@' => "at",
        '°' => "grādi",
        '=' => "vienāds",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Lv)
    }

    #[test]
    fn rules() {
        for (w, e) in [
            ("māja", "mˈaːja"), ("ļoti", "ʎˈuoti"), ("ziema", "zˈiema"), ("jūra", "jˈuːra"),
            ("ģimene", "ɟˈimɛnɛ"), ("pulkstenis", "pˈulkstɛnis"), ("ceļš", "tsˈɛʎʃ"), ("cilvēks", "tsˈilvɛːks"),
            ("un", "un"), ("dzeltens", "ʣˈɛltɛns"),
        ] {
            assert_eq!(p(w), e, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(crate::num::cardinal(2025, Lang::Lv).join(" "), "divi tūkstoši divdesmit pieci");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
