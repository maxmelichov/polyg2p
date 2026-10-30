//! German (Standard, Duden-style), espeak-style notation.
//!
//! Order: unstressed function words, the lexicon (`de.lex`), compound splitting
//! against the lexicon, then the neural model (`de.g2p`) or built-in rules.

use std::collections::HashMap;
use std::path::Path;

use crate::lexicon::Lexicon;
use crate::nn::Model;

const FUNC_TSV: &str = include_str!("../data/de_func.tsv");

pub struct German {
    lex: Option<Lexicon>,
    model: Option<Model>,
    func: HashMap<&'static str, &'static str>,
}

/// Linking elements allowed between compound parts, with their pronunciation.
const LINKS: &[(&str, &str)] = &[("", ""), ("s", "s"), ("es", "əs"), ("n", "n"), ("en", "ən"), ("e", "ə"), ("er", "ɜ"), ("ens", "əns")];

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Demote every primary stress to secondary.
fn secondary(p: &str) -> String {
    p.replace('ˈ', "ˌ")
}

impl German {
    pub fn load(lex: Option<&Path>, model: Option<&Path>) -> Self {
        German {
            lex: lex.and_then(|p| Lexicon::open(p).ok()),
            model: model.and_then(|p| Model::open(p).ok()),
            func: FUNC_TSV.lines().filter_map(|l| l.split_once('\t')).collect(),
        }
    }

    fn lookup(&self, w: &str) -> Option<String> {
        let l = self.lex.as_ref()?;
        if let Some(p) = l.get(w) {
            return Some(p.to_string());
        }
        let lw = w.to_lowercase();
        if let Some(p) = l.get(&capitalize(&lw)) {
            return Some(p.to_string());
        }
        l.get(&lw).map(|p| p.to_string())
    }

    /// Pronunciation ignoring the unstressed function-word forms.
    fn content_word(&self, w: &str) -> String {
        if let Some(p) = self.lookup(w) {
            return p;
        }
        match &self.model {
            Some(m) => m.predict(&w.to_lowercase()),
            None => rules(&w.to_lowercase()),
        }
    }

    pub fn word(&self, w: &str) -> String {
        if w.contains('+') {
            // number words built by num.rs: ein+und+zwanzig
            let mut out = String::new();
            for (k, part) in w.split('+').enumerate() {
                let p = self.content_word(part);
                if out.ends_with('t') && p.starts_with("ts") {
                    out.pop();
                }
                if k == 0 {
                    out.push_str(&p);
                } else if part == "und" {
                    out.push_str(&p.replace(['ˈ', 'ˌ'], ""));
                } else {
                    out.push_str(&secondary(&p));
                }
            }
            return out;
        }
        let lw = w.to_lowercase();
        if let Some(f) = self.func.get(lw.as_str()) {
            return f.to_string();
        }
        if let Some(p) = self.pronominal_adverb(&lw) {
            return p;
        }
        if let Some(p) = self.lookup(w) {
            return p;
        }
        if w.contains('-') {
            let parts: Vec<String> = w.split('-').filter(|p| !p.is_empty()).map(|p| self.word(p)).collect();
            return parts.join(" ");
        }
        if self.lex.is_some() {
            if let Some(p) = self.compound(&lw) {
                return p;
            }
        }
        match &self.model {
            Some(m) => m.predict(&lw),
            None => rules(&lw),
        }
    }

    /// Pronominal adverbs (darüber, damit, worauf, hierher) stress the preposition,
    /// not the da-/wo-/hier- part; lexica and espeak both get many of these wrong.
    fn pronominal_adverb(&self, lw: &str) -> Option<String> {
        for (pre, ph) in [("dar", "dɑːr"), ("da", "dɑː"), ("wor", "voːr"), ("wo", "voː"), ("hier", "hiːr")] {
            let Some(rest) = lw.strip_prefix(pre) else { continue };
            if !PRON_ADV_PREP.contains(&rest) {
                continue;
            }
            // the linking -r- appears only before a vowel (darauf / damit, worin / womit)
            let vowel = rest.starts_with(is_v);
            if pre != "hier" && (pre.ends_with('r') != vowel) {
                continue;
            }
            let p = self.content_word(rest);
            let p = if p.contains('ˈ') { p } else { format!("ˈ{p}") };
            return Some(format!("{ph}{p}"));
        }
        None
    }

    /// Split an unknown word into known parts (the last part is the head).
    fn compound(&self, lw: &str) -> Option<String> {
        let c: Vec<char> = lw.chars().collect();
        let n = c.len();
        if n < 6 {
            return None;
        }
        let s = |a: usize, b: usize| -> String { c[a..b].iter().collect() };
        // best[i] = (parts, pron) for c[i..]
        let mut best: Vec<Option<(usize, String)>> = vec![None; n + 1];
        best[n] = Some((0, String::new()));
        for i in (0..n).rev() {
            let mut cand: Option<(usize, String)> = None;
            for j in (i + 3)..=n {
                let Some((parts, rest)) = best[j].clone() else { continue };
                let piece = s(i, j);
                let last = j == n;
                let mut options: Vec<(String, &str)> = Vec::new();
                if last {
                    options.push((piece.clone(), ""));
                } else {
                    for (link, lp) in LINKS {
                        if let Some(stem) = piece.strip_suffix(link) {
                            if stem.chars().count() >= 3 {
                                options.push((stem.to_string(), lp));
                            }
                        }
                    }
                }
                for (stem, lp) in options {
                    let Some(p) = self.lookup(&stem) else { continue };
                    // a whole lower-case match of a function word is a poor compound part
                    if self.func.contains_key(stem.as_str()) && stem.chars().count() < 4 {
                        continue;
                    }
                    let pron = format!("{}{lp}{rest}", if i == 0 { p.clone() } else { secondary(&p) });
                    let score = parts + 1;
                    if cand.as_ref().map_or(true, |(k, _)| score < *k) {
                        cand = Some((score, pron));
                    }
                }
            }
            best[i] = cand;
        }
        best[0].clone().filter(|(k, _)| *k >= 2).map(|(_, p)| p)
    }
}

// ---------------------------------------------------------------- rules

/// Second parts of pronominal adverbs (da-/wo-/hier- + preposition).
const PRON_ADV_PREP: &[&str] = &[
    "über", "auf", "an", "in", "von", "mit", "zu", "nach", "um", "her", "hin", "aus", "bei", "für", "durch",
    "gegen", "hinter", "neben", "unter", "vor", "zwischen", "wider", "rum", "ran", "rauf", "raus", "rein",
    "runter", "rüber", "rin", "nieder", "zwischen", "entlang", "gegenüber",
];

const PREFIX_STRESSED: &[&str] = &[
    "zurück", "zusammen", "weiter", "wieder", "heraus", "herein", "hinaus", "hinein", "voran", "vorbei", "über",
    "unter", "durch", "nach", "ein", "aus", "auf", "mit", "vor", "weg", "her", "hin", "dar", "bei", "zu",
    "an", "ab", "um", "los", "fort", "fest", "frei", "hoch", "ur", "un",
];
/// Foreign suffixes that carry the main stress (on their first vowel).
const SUFFIX_STRESSED: &[&str] = &[
    "ieren", "ierst", "iert", "iere", "ierung", "tionen", "tion", "sion", "ität", "itäten", "ismus", "isten",
    "istin", "ist", "anten", "ant", "enten", "ent", "anz", "enz", "eur", "euse", "ös", "ell", "elle", "al",
    "ale", "alen", "är", "ie", "ien", "ei", "erei", "ur", "uren", "ik", "ikum", "abel", "ibel", "ade", "age",
    "ett", "ette", "on", "oge", "ogie", "ograf", "ograph", "ose", "iv", "ive",
];

/// Unstressed prefixes with their pronunciation.
const PREFIX_UNSTRESSED_PH: &[(&str, &str)] = &[
    ("miss", "mɪs"), ("emp", "ɛmp"), ("ent", "ɛnt"), ("zer", "tsɛɾ"), ("ver", "fɛɾ"), ("er", "ɛɾ"), ("be", "bə"), ("ge", "ɡə"),
];
/// Suffixes that start a new morpheme (final devoicing / vowel length before them).
const MORPH_SUFFIX: &[&str] = &["lich", "lein", "bar", "los", "heit", "keit", "schaft", "sam", "haft", "nis", "tum", "ling", "lung"];

/// Basic German letter-to-sound rules (used only without the lexicon/model).
pub fn rules(w: &str) -> String {
    let n = w.chars().count();
    for (p, ph) in PREFIX_UNSTRESSED_PH {
        if w.starts_with(p) && n >= p.len() + 4 && !w[p.len()..].starts_with('h') && !w[p.len()..].starts_with(|c: char| is_v(c) && *p == "ge") {
            return format!("{ph}{}", core(&w[p.len()..], true));
        }
    }
    for p in PREFIX_STRESSED {
        if w.starts_with(p) && n >= p.chars().count() + 4 {
            let rest = &w[p.len()..];
            return format!("{}{}", core(p, true), secondary(&core(rest, true)));
        }
    }
    core(w, true)
}

fn core(w: &str, stem_initial: bool) -> String {
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n == 0 {
        return String::new();
    }
    let mut stress_at: Option<usize> = None;
    for suf in SUFFIX_STRESSED {
        if w.len() > suf.len() + 1 && w.ends_with(suf) {
            let start = n - suf.chars().count();
            stress_at = (start..n).find(|&i| is_v(c[i]));
            if suf.starts_with("tion") || suf.starts_with("sion") {
                stress_at = (start..n).find(|&i| c[i] == 'o');
            }
            break;
        }
    }
    let sp = stress_at.or_else(|| (0..n).find(|&i| is_v(c[i]))).unwrap_or(0);
    // morpheme boundary before a derivational suffix
    let mut bound = n;
    for suf in MORPH_SUFFIX {
        if w.ends_with(suf) && n > suf.chars().count() + 2 {
            bound = n - suf.chars().count();
            break;
        }
    }
    let _ = stem_initial;
    let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { c[i as usize] } };
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = at(i as isize + 1);
        let n2 = at(i as isize + 2);
        if is_v(ch) {
            let stressed = i == sp || (i + 1 == sp && is_diph(ch, nx));
            if stressed {
                out.push('ˈ');
            }
            // diphthongs and digraph vowels
            let two: String = [ch, nx].iter().collect();
            let (ph, used) = match two.as_str() {
                "ie" if !(i + 2 == n) || true => ("iː", 2),
                "ei" | "ai" | "ey" | "ay" => ("aɪ", 2),
                "eu" | "äu" => ("ɔø", 2),
                "au" => ("aʊ", 2),
                "aa" => ("ɑː", 2),
                "ee" => ("eː", 2),
                "oo" => ("oː", 2),
                _ => ("", 0),
            };
            if used > 0 {
                out.push_str(ph);
                i += used;
                if at(i as isize) == 'h' && !is_v(at(i as isize + 1)) {
                    i += 1;
                }
                continue;
            }
            let long = if nx == 'h' && !is_v(n2) {
                true
            } else if !stressed && matches!(ch, 'i' | 'u' | 'o' | 'e' | 'ü' | 'ö') {
                // espeak keeps unstressed tense vowels long in open syllables
                long_vowel(&c[..bound.max(i + 1)], i) && is_v(c.get(i + 2).copied().unwrap_or('#'))
            } else if !stressed {
                false
            } else {
                long_vowel(&c[..bound.max(i + 1)], i)
            };
            let unstressed_e = ch == 'e' && !stressed;
            let ph = match ch {
                'a' => if long { "ɑː" } else { "a" },
                'e' => {
                    if unstressed_e && i > sp {
                        if nx == 'r' && (n2 == '#' || !is_v(n2)) {
                            i += 2;
                            out.push('ɜ');
                            continue;
                        }
                        "ə"
                    } else if unstressed_e {
                        if long { "eː" } else { "ɛ" }
                    } else if long {
                        "eː"
                    } else {
                        "ɛ"
                    }
                }
                'i' => if long { "iː" } else { "ɪ" },
                'o' => if long || (stressed && nx == 'n' && at(i as isize - 1) == 'i') { "oː" } else { "ɔ" },
                'u' => if long { "uː" } else { "ʊ" },
                'ä' => if long { "ɛː" } else { "ɛ" },
                'ö' => if long { "øː" } else { "œ" },
                'ü' | 'y' => if long { "yː" } else { "ʏ" },
                _ => "",
            };
            out.push_str(ph);
            i += 1;
            if at(i as isize) == 'h' && !is_v(at(i as isize + 1)) {
                i += 1;
            }
            continue;
        }
        let last = i + 1 == n || (i + 2 == n && nx == 's') || i + 1 == bound;
        let prev_v = is_v(at(i as isize - 1));
        match ch {
            's' => {
                if nx == 'c' && n2 == 'h' {
                    out.push('ʃ');
                    i += 3;
                    continue;
                }
                if i == 0 && matches!(nx, 'p' | 't') {
                    out.push('ʃ');
                } else if nx == 's' {
                    out.push('s');
                    i += 2;
                    continue;
                } else if is_v(nx) && (i == 0 || prev_v || matches!(at(i as isize - 1), 'l' | 'n' | 'r' | 'm')) {
                    out.push('z');
                } else {
                    out.push('s');
                }
            }
            'ß' => out.push('s'),
            'c' => {
                if nx == 'h' {
                    if n2 == 's' {
                        out.push_str("ks");
                        i += 3;
                        continue;
                    }
                    let back = matches!(at(i as isize - 1), 'a' | 'o' | 'u') && !(at(i as isize - 2) == 'e' && at(i as isize - 1) == 'u');
                    let back = back || (at(i as isize - 1) == 'u' && at(i as isize - 2) == 'a');
                    out.push(if back { 'x' } else { 'ç' });
                    i += 2;
                    continue;
                }
                if nx == 'k' {
                    out.push('k');
                    i += 2;
                    continue;
                }
                out.push_str(if matches!(nx, 'e' | 'i' | 'ä' | 'y') { "ts" } else { "k" });
            }
            't' => {
                if nx == 's' && n2 == 'c' && at(i as isize + 3) == 'h' {
                    out.push('ʧ');
                    i += 4;
                    continue;
                }
                if nx == 'z' {
                    out.push_str("ts");
                    i += 2;
                    continue;
                }
                if nx == 'h' {
                    out.push('t');
                    i += 2;
                    continue;
                }
                if nx == 'i' && n2 == 'o' && at(i as isize + 3) == 'n' {
                    out.push_str("tsj");
                    i += 2;
                    continue;
                }
                out.push('t');
            }
            'z' => out.push_str("ts"),
            'q' => {
                out.push_str("kv");
                if nx == 'u' {
                    i += 1;
                }
            }
            'x' => out.push_str("ks"),
            'p' => {
                if nx == 'h' {
                    out.push('f');
                    i += 2;
                    continue;
                }
                if nx == 'f' {
                    out.push_str("pf");
                    i += 2;
                    continue;
                }
                out.push('p');
            }
            'v' => out.push('f'),
            'w' => out.push('v'),
            'j' => out.push('j'),
            'b' => out.push(if last || (!is_v(nx) && !matches!(nx, 'l' | 'r') && nx != 'b') { 'p' } else { 'b' }),
            'd' => out.push(if last || (!is_v(nx) && !matches!(nx, 'l' | 'r') && nx != 'd') { 't' } else { 'd' }),
            'g' => {
                if at(i as isize - 1) == 'i' && (last || !is_v(nx)) && !matches!(nx, 'l' | 'r') {
                    // -ig: ɪç
                    out.push('ç');
                } else {
                    out.push(if last || (!is_v(nx) && !matches!(nx, 'l' | 'r') && nx != 'g') { 'k' } else { 'ɡ' });
                }
            }
            'n' => {
                if nx == 'g' {
                    out.push('ŋ');
                    i += 2;
                    continue;
                }
                if nx == 'k' {
                    out.push('ŋ');
                } else {
                    out.push('n');
                }
            }
            'r' => {
                let after_v = prev_v;
                let plosive_before = matches!(at(i as isize - 1), 'p' | 'b' | 't' | 'd' | 'k' | 'g');
                if plosive_before || (after_v && !is_v(nx)) {
                    out.push('ɾ');
                } else {
                    out.push('r');
                }
            }
            'h' => {
                if is_v(nx) && (i == 0 || !prev_v) {
                    out.push('h');
                }
            }
            'k' => out.push('k'),
            'f' | 'l' | 'm' => out.push(ch),
            _ => {}
        }
        // doubled consonant letters are one sound
        if nx == ch && !is_v(ch) {
            i += 1;
        }
        i += 1;
    }
    out
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ä' | 'ö' | 'ü' | 'y')
}

fn is_diph(a: char, b: char) -> bool {
    matches!((a, b), ('e', 'i') | ('a', 'i') | ('e', 'u') | ('ä', 'u') | ('a', 'u') | ('i', 'e'))
}

/// A vowel is long before at most one consonant followed by a vowel (or end),
/// short before two or more consonants.
fn long_vowel(c: &[char], i: usize) -> bool {
    let mut k = i + 1;
    let mut cons = 0;
    while k < c.len() && !is_v(c[k]) {
        // digraphs count as one consonant
        if c[k] == 'c' && c.get(k + 1) == Some(&'h') {
            cons += 2; // ch shortens the vowel in most words (Bach, Buch is the exception)
            k += 2;
            continue;
        }
        if c[k] == 's' && c.get(k + 1) == Some(&'c') && c.get(k + 2) == Some(&'h') {
            cons += 2;
            k += 3;
            continue;
        }
        if c[k] == 'ß' {
            return true;
        }
        cons += 1;
        k += 1;
    }
    if k >= c.len() {
        // word-final: monosyllables with one consonant are long (Tag, gut), else short
        return cons <= 1 && c.iter().filter(|&&x| is_v(x)).count() == 1;
    }
    cons <= 1
}

#[cfg(test)]
mod tests {
    use super::rules;
    #[test]
    fn show() {
        for w in ["bundesregierung", "haus", "schule", "lesen", "entscheiden", "kindergarten", "information", "regierung", "spielen", "wasser", "sprechen", "machen", "zeitung", "freundlich", "verstehen"] {
            eprintln!("{w} {}", rules(w));
        }
    }
    #[test]
    fn basic_rules() {
        assert_eq!(rules("haus"), "hˈaʊs");
        assert_eq!(rules("schule"), "ʃˈuːlə");
        assert_eq!(rules("verstehen"), "fɛɾʃtˈeːən");
        assert_eq!(rules("information"), "ɪnfɔɾmatsjˈoːn");
        assert_eq!(rules("lesen"), "lˈeːzən");
    }
}
