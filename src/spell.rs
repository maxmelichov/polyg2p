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
        Lang::Es => Some(&ES),
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
    n == 2 || !has_vowel || (!vowel(&letters[0]) && !vowel(&letters[1]))
}

/// Spell `w` letter by letter: secondary stress on each letter, primary on the last.
pub fn spell(w: &str, lang: Lang) -> Option<String> {
    let table = names(lang)?;
    let letters: Vec<usize> = w.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| (c.to_ascii_lowercase() as u8 - b'a') as usize).collect();
    let mut out = String::new();
    for (k, &l) in letters.iter().enumerate() {
        let name = table[l];
        let mark = if k + 1 == letters.len() { "ˈ" } else { "ˌ" };
        let pos = name.find(|c: char| V.contains(c)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    Some(out)
}
