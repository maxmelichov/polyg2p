//! Letter-name spelling for acronyms (es, it, de; English has its own in en.rs).

use crate::Lang;
use crate::{fr, pt, pl, uk, tr, nl, cs, el, ro, hu, fi, sv, id, ca, ar};

fn names(lang: Lang) -> Option<&'static [&'static str; 26]> {
    const ES: [&str; 26] = [
        "a", "be", "θe", "de", "e", "efe", "xe", "aʧe", "i", "xota", "ka", "ele", "eme", "ene", "o", "pe", "ku",
        "ere", "ese", "te", "u", "uβe", "uβeðoβle", "ekis", "je", "θeta",
    ];
    const IT: [&str; 26] = [
        "a", "bi", "ʧi", "di", "e", "ɛffe", "ʤi", "akka", "i", "illuŋɡa", "kappa", "ɛlle", "ɛmme", "ɛnne", "ɔ", "pi",
        "ku", "ɛrre", "ɛsse", "ti", "u", "vu", "doppjavu", "iks", "ipsilon", "ʣɛta",
    ];
    const DE: [&str; 26] = [
        "aː", "beː", "tseː", "deː", "eː", "ɛf", "ɡeː", "haː", "iː", "jɔt", "kaː", "ɛl", "ɛm", "ɛn", "oː", "peː",
        "kuː", "ɛɾ", "ɛs", "teː", "uː", "faʊ", "veː", "ɪks", "ʏpsilɔn", "tsɛt",
    ];
    match lang {
        Lang::Es | Lang::EsLa => Some(&ES),
        Lang::It => Some(&IT),
        Lang::De => Some(&DE),
        Lang::En | Lang::Ru | Lang::He => None,
        Lang::Fr => fr::LETTERS,
        Lang::Pt => pt::LETTERS,
        Lang::Pl => pl::LETTERS,
        Lang::Uk => uk::LETTERS,
        Lang::Tr => tr::LETTERS,
        Lang::Nl => nl::LETTERS,
        Lang::Cs => cs::LETTERS,
        Lang::El => el::LETTERS,
        Lang::Ro => ro::LETTERS,
        Lang::Hu => hu::LETTERS,
        Lang::Fi => fi::LETTERS,
        Lang::Sv => sv::LETTERS,
        Lang::Id => id::LETTERS,
        Lang::Ca => ca::LETTERS,
        Lang::Ar => ar::LETTERS,
    }
}

const V: &str = "aeiouɛɔɪʊʏyø";

/// Is `w` an acronym to spell out? (all caps, 2–5 letters, not pronounceable as a word)
pub fn is_acronym(w: &str) -> bool {
    let letters: Vec<char> = w.chars().filter(|c| c.is_alphabetic()).collect();
    let n = letters.len();
    if !(2..=5).contains(&n) || !letters.iter().all(|c| c.is_ascii_uppercase()) {
        return false;
    }
    let vowel = |c: &char| "AEIOUY".contains(*c);
    let has_vowel = letters.iter().any(vowel);
    // spell unless it reads as a word (ONU, RAI, NATO): two letters, no vowel,
    // or an initial consonant cluster (FBI, SPD) mean letter names
    if n == 2 || !has_vowel || (!vowel(&letters[0]) && !vowel(&letters[1])) {
        return true;
    }
    // a run of three consonants (ADHD, OTSC) or a final cluster no word ends in
    // (ABN, AVG, APBJ) cannot be read as a word either
    let mut run = 0;
    for c in &letters {
        run = if vowel(c) { 0 } else { run + 1 };
        if run >= 3 {
            return true;
        }
    }
    if n >= 3 && !vowel(&letters[n - 1]) && !vowel(&letters[n - 2]) {
        let coda: String = letters[n - 2..].iter().collect::<String>().to_ascii_lowercase();
        return !CODAS.contains(&coda.as_str());
    }
    false
}

/// Two-consonant word endings that read as a syllable coda (FARC, USP, AIDS).
const CODAS: &[&str] = &[
    "st", "nt", "nd", "lt", "rt", "ns", "rs", "ls", "ks", "ts", "ps", "rk", "lk", "nk", "sk", "rd", "ld", "rn", "ln", "rm",
    "lm", "mp", "rl", "ng", "ch", "sh", "th", "ck", "ll", "ss", "tt", "nn", "rr", "ff", "pp", "mm", "ph", "rc", "sp", "nc",
    "ds", "lf", "rf", "rb", "rp", "ct", "pt", "ft", "mb", "lp", "lb", "nf", "ms", "cs", "ns", "gs", "bs",
];

/// Spell `w` letter by letter: secondary stress on each letter, primary on the last.
pub fn spell(w: &str, lang: Lang) -> Option<String> {
    let table = names(lang)?;
    let letters: Vec<&str> = w
        .chars()
        .filter(|c| c.is_alphabetic())
        .filter_map(|c| match c.to_lowercase().next().unwrap_or(c) {
            'ñ' => Some("eɲe"),
            'ç' => Some(match lang { Lang::Fr => "sesedij", _ => "sedilja" }),
            'ß' => Some("ɛstsɛt"),
            c if c.is_ascii_alphabetic() => Some(table[(c as u8 - b'a') as usize]),
            _ => None,
        })
        .collect();
    let mut out = String::new();
    for (k, &name) in letters.iter().enumerate() {
        let mark = if k + 1 == letters.len() { "ˈ" } else { "ˌ" };
        let pos = name.find(|c: char| V.contains(c)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    Some(out)
}
