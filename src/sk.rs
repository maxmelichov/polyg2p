//! Slovak grapheme-to-phoneme rules, notation shared with `cs.rs`.
//!
//! Slovak spelling is close to phonemic. Rules (Wikipedia, "Slovak phonology", and the
//! Wiktionary Slovak pronunciation appendix, both CC BY-SA, read 2026-11-28):
//! * stress is fixed on the first syllable ("Stress is fixed on the first syllable of a word");
//! * `ď ť ň ľ` are palatals `ɟ c ɲ ʎ`; plain `d t n l` are palatalised before `i í e` (and so before the
//!   `i` of `ia ie iu`) in native words; before long `é` and in loan stems they stay hard (table `HARD`,
//!   mined from the train split of the gold list, never from the held-out words);
//! * `ia ie iu` are rising diphthongs (`ɪ̯` + vowel) in native words, `ô` is `u̯o`, `ä` is `æ`;
//!   `au eu ou` are the diphthongs `aʊ eʊ oʊ`;
//! * `r l` between consonants (or consonant and word end) are syllabic, `ŕ ĺ` their long forms;
//! * `ch` is `x`, `h` is written `h` as `cs.rs` writes it, `dz dž` are affricates;
//! * voicing assimilation (`vták` -> `ftaːk`) and final devoicing; sonorants do not trigger voicing,
//!   `v` takes the voicing of what follows but does not give it;
//! * `v` is written `v` except word-initially before a voiceless obstruent, where it is `f` (`vták`).
//! A monosyllabic preposition and the following word form one stress unit (`join`).

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "čiarka";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "aː", "beː", "tseː", "deː", "eː", "ef", "ɡeː", "haː", "iː", "jeː", "kaː", "el", "em", "en", "oː", "peː", "kveː",
    "er", "es", "teː", "uː", "veː", "dvojiteːveː", "iks", "ipsilon", "zet",
]);

/// Monosyllabic prepositions that take the stress of the following word.
const PREPS: &[&str] = &[
    "na", "do", "po", "od", "pre", "pri", "za", "nad", "pod", "bez", "cez", "u", "o", "v", "k", "s", "z", "so", "zo", "ku",
    "vo", "pred", "medzi", "okolo", "popri", "podľa", "kvôli",
];

/// Clitics and function words read without a stress mark (the published sample passage writes
/// `sa`, `a`, `je`, `z` unstressed).
const UNSTRESSED: &[&str] = &[
    "a", "i", "sa", "si", "je", "som", "sme", "ste", "sú", "by", "bol", "bola", "bolo", "boli", "mi", "ti", "ho", "mu",
    "ma", "ťa", "nás", "vás", "im", "ich", "že", "aj", "ani", "ale", "no", "či", "ak", "keď", "lebo", "ako", "som",
];

/// Stems in which `d t n l` before `i e` stay hard (loans; native words are soft). Suffix and stem
/// shapes of loan vocabulary (-izmus, -atív, -ália, ...), a stem must cover the consonant.
const HARD: &[&str] = &["teo", "teó", "niz", "lizm", "atí", "tiz", "^oli", "lizá", "lium", "nium", "lista", "diu", "tika", "líd", "óni", "neo", "ília", "mini", "^ter", "net$", "lív", "tif", "tei", "ália", "énia", "tívu", "elek", "idit", "risti", "ticiz", "otera", "tivizá", "ualita", "nista$", "éli", "adel", "den$", "^ali", "stín", "níci", "ánia", "olik", "rtik", "dive", "teor", "tit$", "^din", "ten$", "oten", "andi", "olet", "listo", "dent$", "def"];

#[derive(Clone, Copy, PartialEq)]
enum K {
    V,
    Son,
    Obs,
}

#[derive(Clone)]
struct Seg {
    ph: &'static str,
    k: K,
}

fn s(ph: &'static str, k: K) -> Seg {
    Seg { ph, k }
}

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

fn is_voiced(p: &str) -> bool {
    !voiceless_of(p).is_empty() && p != "v"
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'á' | 'ä' | 'e' | 'é' | 'i' | 'í' | 'o' | 'ó' | 'ô' | 'u' | 'ú' | 'y' | 'ý')
}

fn covered(c: &[char], i: usize, stems: &[&str]) -> bool {
    // `^` and `$` in a stem anchor it to the start / end of the word
    let w: String = std::iter::once('^').chain(c.iter().copied()).chain(std::iter::once('$')).collect();
    let i = i + 1;
    for st in stems {
        let n = st.chars().count();
        let mut from = 0;
        while let Some(p) = w[from..].find(st) {
            let start = w[..from + p].chars().count();
            if start <= i && i < start + n {
                return true;
            }
            from += p + st.len().max(1);
            if from >= w.len() {
                break;
            }
        }
    }
    false
}

/// d t n l before a front vowel: soft in native words.
fn soft_here(c: &[char], i: usize) -> bool {
    matches!(c.get(i + 1), Some('i' | 'í' | 'e' | 'ä')) && !covered(c, i, HARD)
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
    render(&segments(&c), !UNSTRESSED.contains(&lw.as_str()))
}

/// Letters of two words glued with a `|` boundary (blocks palatalisation, syllabic and glide rules).
fn glued(a: &str, b: &str) -> String {
    let c: Vec<char> = format!("{a}|{b}").chars().filter(|c| c.is_alphabetic() || *c == '|').collect();
    render(&segments(&c), true)
}

fn is_cons_letter(c: char) -> bool {
    c.is_alphabetic() && !is_vowel_letter(c)
}

/// `i` + vowel read as two syllables (hiatus) rather than a rising diphthong: loan endings
/// -cia -fia -gia -zia -mia, -ium/-ius, and -ria -lia -nia -nie after a stressed long vowel.
fn hiatus(c: &[char], i: usize) -> bool {
    let n = c.len();
    let end = i + 2 == n; // `i` + vowel closes the word
    let pre = c[i - 1];
    let v = c[i + 1];
    if v == 'u' {
        return i + 3 == n && matches!(c.get(i + 2), Some('m' | 's'));
    }
    if v == 'a' && end && matches!(pre, 'c' | 'f' | 'g' | 'z' | 'm') {
        return true;
    }
    if end && matches!(pre, 'l' | 'r' | 'n' | 'd' | 's') && (v == 'a' || (v == 'e' && pre == 'n')) {
        return i >= 2 && matches!(c[i - 2], 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ý');
    }
    false
}

fn segments(c: &[char]) -> Vec<Seg> {
    let mut out: Vec<Seg> = Vec::new();
    let n = c.len();
    let mut i = 0;
    let prev_vowel = |out: &Vec<Seg>| out.last().is_some_and(|x| x.k == K::V);
    while i < n {
        let ch = c[i];
        let nx = c.get(i + 1).copied();
        let long = |short: &'static str, lng: &'static str, is_long: bool| if is_long { lng } else { short };
        match ch {
            '|' => {}
            'a' | 'á' => out.push(s(long("a", "aː", ch == 'á'), K::V)),
            'ä' => out.push(s("æ", K::V)),
            'e' | 'é' => out.push(s(long("e", "eː", ch == 'é'), K::V)),
            'i' | 'í' | 'y' | 'ý' => {
                let is_long = matches!(ch, 'í' | 'ý');
                // ia ie iu after a consonant: rising diphthong
                let after_cons = i > 0 && is_cons_letter(c[i - 1]);
                if ch == 'i' && after_cons && matches!(nx, Some('a' | 'e' | 'u')) && !hiatus(c, i) {
                    out.push(s("ɪ̯", K::Son));
                } else {
                    out.push(s(long("i", "iː", is_long), K::V));
                }
            }
            'o' | 'ó' => {
                if ch == 'o' && nx == Some('u') {
                    out.push(s("oʊ", K::V));
                    i += 1;
                } else {
                    out.push(s(long("o", "oː", ch == 'ó'), K::V));
                }
            }
            'ô' => {
                out.push(s("u̯", K::Son));
                out.push(s("o", K::V));
            }
            'u' | 'ú' => out.push(s(long("u", "uː", ch == 'ú'), K::V)),
            'b' => out.push(s("b", K::Obs)),
            'c' => out.push(s("ts", K::Obs)),
            'č' => out.push(s("tʃ", K::Obs)),
            'd' => {
                if nx == Some('z') {
                    out.push(s("dz", K::Obs));
                    i += 1;
                } else if nx == Some('ž') {
                    out.push(s("dʒ", K::Obs));
                    i += 1;
                } else if soft_here(c, i) {
                    out.push(s("ɟ", K::Obs));
                } else {
                    out.push(s("d", K::Obs));
                }
            }
            'ď' => out.push(s("ɟ", K::Obs)),
            'f' => out.push(s("f", K::Obs)),
            'g' => out.push(s("ɡ", K::Obs)),
            'h' => out.push(s("h", K::Obs)),
            'j' => out.push(s("j", K::Son)),
            'k' => out.push(s("k", K::Obs)),
            'l' | 'ĺ' | 'r' | 'ŕ' => {
                let liquid = if matches!(ch, 'l' | 'ĺ') { "l" } else { "r" };
                let syl = {
                    let prev_c = i == 0 || (is_cons_letter(c[i - 1]) && c[i - 1] != '|') || (c[i - 1] == '|' && false);
                    let next_ok = match nx {
                        None => true,
                        Some(x) => !is_vowel_letter(x) && x != '|' && !matches!(x, 'ĺ' | 'ŕ'),
                    };
                    matches!(ch, 'ĺ' | 'ŕ') || (prev_c && next_ok && !(i == 0 && nx.is_none()))
                };
                if matches!(ch, 'ĺ' | 'ŕ') {
                    out.push(s(if liquid == "l" { "l̩ː" } else { "r̩ː" }, K::V));
                } else if syl && !prev_vowel(&out) {
                    out.push(s(if liquid == "l" { "l̩" } else { "r̩" }, K::V));
                } else if liquid == "l" && soft_here(c, i) {
                    out.push(s("ʎ", K::Son));
                } else {
                    out.push(s(liquid, K::Son));
                }
            }
            'ľ' => out.push(s("ʎ", K::Son)),
            'm' => out.push(s("m", K::Son)),
            'n' => {
                if soft_here(c, i) {
                    out.push(s("ɲ", K::Son));
                } else if matches!(nx, Some('k' | 'g')) {
                    out.push(s("ŋ", K::Son));
                } else {
                    out.push(s("n", K::Son));
                }
            }
            'ň' => out.push(s("ɲ", K::Son)),
            'p' => out.push(s("p", K::Obs)),
            'q' => {
                out.push(s("k", K::Obs));
                out.push(s("v", K::Obs));
            }
            's' => out.push(s("s", K::Obs)),
            'š' => out.push(s("ʃ", K::Obs)),
            't' => {
                if soft_here(c, i) {
                    out.push(s("c", K::Obs));
                } else {
                    out.push(s("t", K::Obs));
                }
            }
            'ť' => out.push(s("c", K::Obs)),
            'v' | 'w' => out.push(s("v", K::Obs)),
            'x' => {
                out.push(s("k", K::Obs));
                out.push(s("s", K::Obs));
            }
            'z' => out.push(s("z", K::Obs)),
            'ž' => out.push(s("ʒ", K::Obs)),
            _ => {}
        }
        // ch digraph handled when we see 'c' followed by 'h'
        if ch == 'c' && nx == Some('h') {
            out.pop();
            out.push(s("x", K::Obs));
            i += 1;
        }
        i += 1;
    }
    // au eu: u after a vowel is the glide of a diphthong
    let mut j = 0;
    while j + 1 < out.len() {
        if out[j].k == K::V && out[j].ph.len() == 1 && matches!(out[j].ph, "a" | "e") && out[j + 1].ph == "u" {
            // only when spelled au/eu with u short: keep the vowel and add the glide
            out[j].ph = if out[j].ph == "a" { "aʊ" } else { "eʊ" };
            out.remove(j + 1);
        }
        j += 1;
    }
    assimilate(&mut out);
    out
}

fn assimilate(segs: &mut Vec<Seg>) {
    let n = segs.len();
    // regressive voicing, right to left; word-final obstruents devoice
    for i in (0..n).rev() {
        if segs[i].k != K::Obs {
            continue;
        }
        let ph = segs[i].ph;
        let next_obs = segs.get(i + 1).filter(|x| x.k == K::Obs).map(|x| x.ph);
        match next_obs {
            Some(nx) if nx != "v" || true => {
                if nx == "v" && ph != "v" {
                    // v does not trigger voicing, but a voiceless/voiced pair before it stays as spelled
                    continue;
                }
                if is_voiced(nx) {
                    let v = voiced_of(ph);
                    if !v.is_empty() {
                        segs[i].ph = v;
                    }
                } else if nx != "v" {
                    let v = voiceless_of(ph);
                    if !v.is_empty() {
                        segs[i].ph = v;
                    }
                } else {
                    // before v
                }
            }
            _ => {
                if i + 1 == n {
                    let v = voiceless_of(ph);
                    if !v.is_empty() && ph != "v" {
                        segs[i].ph = v;
                    }
                } else if ph == "v" {
                    // v before a voiceless obstruent is f handled below; before sonorant/vowel it stays
                }
            }
        }
    }
    // a stop before the affricate of the same place merges into it (dcéra, srdce: one ts)
    let mut j = 0;
    while j + 1 < segs.len() {
        if matches!(segs[j].ph, "t" | "d") && matches!(segs[j + 1].ph, "ts" | "tʃ" | "dz" | "dʒ") {
            segs[j].ph = "";
        }
        j += 1;
    }
    // word-initial v before a voiceless obstruent -> f (vták, vkus); elsewhere v stays v
    if n >= 2 && segs[0].ph == "v" && segs[1].k == K::Obs && !is_voiced(segs[1].ph) && segs[1].ph != "v" {
        segs[0].ph = "f";
    }
}

fn render(segs: &[Seg], stressed: bool) -> String {
    let mut out = String::with_capacity(segs.len() * 2);
    let mut done = !stressed;
    for (i, sg) in segs.iter().enumerate() {
        // stress goes before the first nucleus; a rising glide belongs to its vowel
        let is_glide = matches!(sg.ph, "ɪ̯" | "u̯") && segs.get(i + 1).is_some_and(|x| x.k == K::V);
        if !done && (sg.k == K::V || is_glide) {
            out.push('ˈ');
            done = true;
        }
        out.push_str(sg.ph);
    }
    out
}

fn first_phone(p: &str) -> &str {
    let t = p.trim_start_matches(['ˈ', 'ˌ']);
    for two in ["ts", "tʃ", "dz", "dʒ"] {
        if t.starts_with(two) {
            return two;
        }
    }
    t.chars().next().map_or("", |c| &t[..c.len_utf8()])
}

/// Voice the word-final obstruent cluster of `ph` (a devoiced final before a voiced onset).
fn revoice_final(ph: &str) -> String {
    let mut out = ph.to_string();
    // operate on the last phone only (clusters were devoiced as a unit; restore via pairs)
    let pairs = [("ts", "dz"), ("tʃ", "dʒ"), ("p", "b"), ("t", "d"), ("c", "ɟ"), ("k", "ɡ"), ("s", "z"), ("ʃ", "ʒ"), ("x", "h"), ("f", "v")];
    loop {
        let mut changed = false;
        for (a, b) in pairs {
            if out.ends_with(a) && !(a == "t" && (out.ends_with("ts") || out.ends_with("tʃ"))) && !(a == "s" && out.ends_with("ts")) {
                let l = out.len() - a.len();
                out.truncate(l);
                out.push_str(b);
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
        // continue only through a cluster of obstruents
        let mut chars = out.chars().rev();
        chars.next();
        match chars.next() {
            Some(c) if "ptckfsʃxbdɟɡzʒhvs".contains(c) => {}
            _ => break,
        }
        // stop after one more only if the previous char is an obstruent: handled by loop condition
        break;
    }
    out
}

/// Sentence pass: a monosyllabic preposition and its word form one phonological word; a
/// word-final obstruent is voiced before a voiced obstruent onset.
pub fn join(outs: &mut [Out]) {
    let n = outs.len();
    for k in 0..n.saturating_sub(1) {
        let (a, b) = outs.split_at_mut(k + 1);
        let prev = &mut a[k];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let src = prev.src.as_str();
        if !PREPS.contains(&src) || (PREPS.contains(&cur.src.as_str()) && cur.src.chars().count() <= 2) {
            continue;
        }
        if cur.src.chars().any(|c| !c.is_alphabetic()) {
            continue;
        }
        // keep the next word's own stress when the preposition has no vowel; otherwise one unit
        let merged = glued(src, &cur.src);
        cur.ph = merged;
        prev.ph.clear();
    }
    for k in 0..n.saturating_sub(1) {
        let (a, b) = outs.split_at_mut(k + 1);
        let prev = &mut a[k];
        let cur = &b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let f = first_phone(&cur.ph);
        if is_voiced(f) {
            prev.ph = revoice_final(&prev.ph);
        }
    }
}

const ONES: [&str; 20] = [
    "nula", "jeden", "dva", "tri", "štyri", "päť", "šesť", "sedem", "osem", "deväť", "desať", "jedenásť", "dvanásť",
    "trinásť", "štrnásť", "pätnásť", "šestnásť", "sedemnásť", "osemnásť", "devätnásť",
];
const TENS: [&str; 10] = [
    "", "", "dvadsať", "tridsať", "štyridsať", "päťdesiat", "šesťdesiat", "sedemdesiat", "osemdesiat", "deväťdesiat",
];
const HUND: [&str; 10] = [
    "", "sto", "dvesto", "tristo", "štyristo", "päťsto", "šesťsto", "sedemsto", "osemsto", "deväťsto",
];

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

fn scale(n: u128, one: &str, few: &str, many: &str, out: &mut Vec<String>) {
    let r = n % 100;
    if n == 1 {
        out.push(format!("jeden {one}"));
        return;
    }
    below_1000(n, out);
    let form = if (2..=4).contains(&(n % 10)) && !(12..=14).contains(&r) { few } else { many };
    out.push(form.to_string());
}

pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n < 1000 {
        below_1000(n, &mut out);
        return out;
    }
    let mut rest = n;
    // groups: miliardy (10^9), milióny (10^6), tisíc (10^3); anything above 10^12 is read as digits
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| ONES[d.to_digit(10).unwrap() as usize].to_string()).collect();
    }
    let g = rest / 1_000_000_000;
    if g > 0 {
        scale(g, "miliarda", "miliardy", "miliárd", &mut out);
        rest %= 1_000_000_000;
    }
    let m = rest / 1_000_000;
    if m > 0 {
        scale(m, "milión", "milióny", "miliónov", &mut out);
        rest %= 1_000_000;
    }
    let t = rest / 1000;
    if t > 0 {
        if t == 1 {
            out.push("tisíc".to_string());
        } else if t < 10 && t != 1 {
            let h = ["", "", "dvetisíc", "tritisíc", "štyritisíc", "päťtisíc", "šesťtisíc", "sedemtisíc", "osemtisíc", "deväťtisíc"];
            out.push(h[t as usize].to_string());
        } else {
            below_1000(t, &mut out);
            out.push("tisíc".to_string());
        }
        rest %= 1000;
    }
    if rest > 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "percent",
        '$' => "dolárov",
        '€' => "eur",
        '£' => "libier",
        '&' => "a",
        '+' => "plus",
        '@' => "zavináč",
        '°' => "stupňov",
        '=' => "rovná sa",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{G2p, Lang};

    fn p(t: &str) -> String {
        G2p::new(None).phonemize(t, Lang::Sk)
    }

    #[test]
    fn rules() {
        for (w, e) in [
            ("chlieb", "xʎˈɪ̯ep"), ("ľudia", "ʎˈuɟɪ̯a"), ("ôsmy", "ˈu̯osmi"), ("mäso", "mˈæso"), ("vták", "ftˈaːk"),
            ("krv", "kˈr̩v"), ("vlk", "vˈl̩k"), ("auto", "ˈaʊto"), ("obchod", "ˈopxot"), ("kniha", "kɲˈiha"),
            ("slnko", "sˈl̩ŋko"), ("dedina", "ɟˈeɟina"), ("mesto", "mˈesto"), ("zvuk", "zvˈuk"), ("shoda", "zhˈoda"),
            ("teória", "tˈeoːria"), ("dcéra", "tsˈeːra"), ("ďakujem", "ɟˈakujem"), ("každý", "kˈaʒdiː"),
            ("izmus", "ˈizmus"), ("dobrý deň", "dˈobriː ɟˈeɲ"),
        ] {
            assert_eq!(p(w).replace('ʦ', "ts").replace('ʧ', "tʃ"), e.replace('ʦ', "ts"), "{w}");
        }
    }

    #[test]
    fn clitics_and_prepositions() {
        assert_eq!(p("v lese"), "vʎˈese");
        assert_eq!(p("a sa"), "a sa");
    }

    #[test]
    fn numbers_do_not_panic() {
        assert_eq!(crate::num::cardinal(2025, Lang::Sk).join(" "), "dvetisíc dvadsať päť");
        assert!(!p("1000000000000000000000").is_empty());
    }
}
