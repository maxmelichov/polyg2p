//! English accents as a layer over the General American engine.
//!
//! The base engine (`en.rs`) does everything it always does: lexicon, homographs,
//! weak forms, numbers, word joining. This module rewrites each finished word
//! into the accent's phonemes, so all of that is inherited unchanged. Plain `en`
//! never enters this module.
//!
//! Two families:
//!
//! * non-rhotic, built on Southern British (`en-gb`): r dropped unless a vowel
//!   follows (linking r across words), LOT /ɒ/, GOAT /əʊ/, broad BATH, NEAR /ɪə/,
//!   SQUARE /eə/, CURE /ʊə/, no flapping, yod kept after t d n. A word list
//!   derived from Britfone (MIT; `data/en_gb_lex.tsv`) holds the words the rules
//!   cannot get from the American form (BATH words, tomato, laboratory, ...);
//!   a short hand-written one (`data/en_brit_words.tsv`) adds schedule, vase, ...
//!   `en-au`, `en-nz`, `en-in` are rule sets applied to the `en-gb`
//!   form; no open pronouncing dictionary with a licence that allows
//!   redistribution exists for them, so they are rule-derived.
//! * rhotic (`en-sc`, `en-ie`, `en-ca`): rules on the American form; Scottish
//!   and Irish first take the British word forms of `data/en_brit_words.tsv`
//!   (herb with h, tomato, schedule, ...) and keep hurry / marry / squirrel
//!   apart from furry / merry / stirring.
//!
//! The variants write phonemic differences (which phoneme a word has), not
//! fine vowel quality: Australian FACE is still written `eɪ`. A TTS model learns
//! the quality from the accent's recordings.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::en::English;
use crate::Out;

const GB_LEX_TSV: &str = include_str!("../data/en_gb_lex.tsv");
const GB_FUNC_TSV: &str = include_str!("../data/en_gb_func.tsv");
const BRIT_WORDS_TSV: &str = include_str!("../data/en_brit_words.tsv");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accent {
    /// Southern British (RP / SSBE)
    Gb,
    /// Australian
    Au,
    /// New Zealand
    Nz,
    /// Indian
    In,
    /// Scottish
    Sc,
    /// Irish
    Ie,
    /// Canadian
    Ca,
}

impl Accent {
    fn rhotic(self) -> bool {
        matches!(self, Accent::Sc | Accent::Ie | Accent::Ca)
    }
}

fn table(s: &'static str) -> HashMap<&'static str, &'static str> {
    s.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_once('\t')).collect()
}

fn gb_lex() -> &'static HashMap<&'static str, &'static str> {
    static T: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    T.get_or_init(|| table(GB_LEX_TSV))
}

/// American function-word form -> (weak British form, strong British form).
fn gb_func() -> &'static HashMap<&'static str, (&'static str, &'static str)> {
    static T: OnceLock<HashMap<&'static str, (&'static str, &'static str)>> = OnceLock::new();
    T.get_or_init(|| {
        GB_FUNC_TSV
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| {
                let mut f = l.split('\t');
                let (ga, weak) = (f.next()?, f.next()?);
                Some((ga, (weak, f.next().filter(|x| !x.is_empty()).unwrap_or(weak))))
            })
            .collect()
    })
}

/// British word forms shared by the rhotic accents of the British Isles, in
/// the American notation (r kept) with `ɒ` marking LOT.
fn brit_words() -> &'static HashMap<&'static str, &'static str> {
    static T: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    T.get_or_init(|| table(BRIT_WORDS_TSV))
}

/// One phoneme with the stress mark that precedes it.
#[derive(Debug, Clone, PartialEq)]
struct Tok {
    stress: Option<char>,
    p: String,
}

// affricates are single letters in the engine's output (ʧ ʤ ʦ ʣ)
const UNITS: &[&str] = &["aɪ", "aʊ", "eɪ", "oʊ", "ɔɪ", "əʊ", "ɪə", "eə", "ʊə"];
const VOWELS: &str = "aeiouæɑɔəɛɜɪʊʌɚᵻɐɒ";

fn is_vowel(p: &str) -> bool {
    p.chars().next().is_some_and(|c| VOWELS.contains(c))
}

fn tokenize(ph: &str) -> Vec<Tok> {
    let cs: Vec<char> = ph.chars().collect();
    let mut out = Vec::with_capacity(cs.len());
    let mut i = 0;
    let mut stress = None;
    while i < cs.len() {
        let c = cs[i];
        if c == 'ˈ' || c == 'ˌ' {
            stress = Some(c);
            i += 1;
            continue;
        }
        let mut p = String::new();
        let two: String = cs[i..(i + 2).min(cs.len())].iter().collect();
        // "ɪəʊ" is ɪ + əʊ (video), not ɪə + ʊ
        let centring_then_goat = cs.get(i + 2) == Some(&'ʊ') && cs.get(i + 1) == Some(&'ə');
        if UNITS.contains(&two.as_str()) && !centring_then_goat {
            p.push_str(&two);
            i += 2;
        } else {
            p.push(c);
            i += 1;
        }
        // length mark and combining marks stay with their phoneme
        while i < cs.len() && (cs[i] == 'ː' || ('\u{0300}'..='\u{036F}').contains(&cs[i])) {
            p.push(cs[i]);
            i += 1;
        }
        // a stress mark belongs to the next vowel; consonants in between keep none
        let s = if is_vowel(&p) { stress.take() } else { None };
        out.push(Tok { stress: s, p });
    }
    out
}

fn render(toks: &[Tok]) -> String {
    let mut s = String::new();
    for t in toks {
        if let Some(c) = t.stress {
            s.push(c);
        }
        s.push_str(&t.p);
    }
    s
}

/// A vowel-letter group of the spelling with its neighbours.
#[derive(Debug, Clone, Copy)]
struct Grp<'a> {
    letters: &'a str,
    before: &'a str,
    after: &'a str,
}

fn vowel_groups(src: &str) -> Vec<(usize, usize)> {
    let b = src.as_bytes();
    let isv = |i: usize| match b[i] {
        b'a' | b'e' | b'i' | b'o' | b'u' => true,
        b'y' => !(i == 0 && b.len() > 1 && matches!(b[1], b'a' | b'e' | b'i' | b'o' | b'u')),
        // aw, ew, ow: the w is part of the vowel unless a vowel follows (lower)
        b'w' => i > 0 && matches!(b[i - 1], b'a' | b'e' | b'o') && !b.get(i + 1).is_some_and(|c| matches!(c, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')),
        _ => false,
    };
    let mut g: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if isv(i) {
            let s = i;
            while i < b.len() && isv(i) {
                i += 1;
            }
            // "qu": the u belongs to the consonant
            let s = if s > 0 && b[s - 1] == b'q' && b[s] == b'u' && i - s > 1 { s + 1 } else { s };
            g.push((s, i));
        } else {
            i += 1;
        }
    }
    g
}

fn mk_grp(src: &str, s: usize, e: usize) -> Grp<'_> {
    Grp { letters: &src[s..e], before: &src[s.saturating_sub(2)..s], after: &src[e..(e + 2).min(src.len())] }
}

/// Can this vowel phoneme be spelled with these letters? (alignment cost only)
fn spells(v: &str, l: &str) -> bool {
    let any = |xs: &[&str]| xs.contains(&l);
    match v {
        "ə" | "ɐ" | "ᵻ" | "ɚ" => true,
        "ɑː" => any(&["a", "o", "au", "aw", "ou", "ea", "oa", "ah", "aa"]),
        "ɔː" | "ɔ" => any(&["o", "a", "au", "aw", "ou", "oa", "oo", "awe"]),
        "æ" => any(&["a", "ai", "au"]),
        "ɛ" => any(&["e", "ea", "a", "ai", "ie", "ei", "u", "ay", "ae"]),
        "ɪ" => any(&["i", "y", "e", "u", "a", "ui", "ie", "ee", "o", "ei", "ey", "ai", "ia"]),
        "iː" | "i" => any(&["e", "ee", "ea", "ie", "ei", "i", "y", "ey", "eo", "ae", "oe", "ay"]),
        "ʌ" => any(&["u", "o", "ou", "oo", "oe"]),
        "ʊ" => any(&["u", "oo", "ou", "o", "eu"]),
        "uː" | "u" => any(&["u", "oo", "ou", "ew", "ue", "o", "ui", "eu", "oe", "eau", "ieu", "iew"]),
        "ɜː" => any(&["e", "i", "u", "o", "ea", "ou", "y", "eu"]),
        "eɪ" => any(&["a", "ai", "ay", "ei", "ey", "ea", "e", "ee", "au", "ae"]),
        "aɪ" => any(&["i", "y", "ie", "ei", "uy", "ye", "ai", "ui", "ey", "eye"]),
        "ɔɪ" => any(&["oi", "oy", "uoy"]),
        "aʊ" => any(&["ou", "ow", "au"]),
        "oʊ" => any(&["o", "oa", "ow", "ou", "oe", "ew", "eau", "au", "oo", "eo"]),
        _ => false,
    }
}

/// Vowel-letter groups of a word, one per vowel phoneme. When the counts agree
/// the groups are taken in order (silent final e and the e of -es / -ed are
/// tried as silent). Otherwise the cheapest line-up is searched: a group may be
/// silent (afor-e-thought), a two-letter group may hold two vowels (vid-e-o),
/// a vowel may have no letter (rhythm). A vowel without a letter gets an empty
/// group. `None` when nothing plausible lines up.
fn align<'a>(src: &'a str, vowels: &[&str]) -> Option<Vec<Grp<'a>>> {
    let n_vowels = vowels.len();
    if !src.is_ascii() || n_vowels == 0 || src.is_empty() {
        return None;
    }
    let b = src.as_bytes();
    let g = vowel_groups(src);
    let mk = |g: &[(usize, usize)]| -> Vec<Grp<'a>> { g.iter().map(|&(s, e)| mk_grp(src, s, e)).collect() };
    let fits = |gs: &[(usize, usize)]| gs.iter().zip(vowels).all(|(&(s, e), v)| spells(v, &src[s..e]));
    let exact = (g.len() == n_vowels).then(|| mk(&g));
    if exact.is_some() && fits(&g) {
        return exact;
    }
    if g.len() == n_vowels + 1 {
        let last = g.len() - 1;
        let (s, e) = g[last];
        // silent final e (made), or the e of -es / -ed (makes, liked)
        let silent_e = &src[s..e] == "e" && s > 0 && (e == b.len() || (e + 1 == b.len() && matches!(b[e], b's' | b'd')));
        if silent_e && (exact.is_none() || fits(&g[..last])) {
            return Some(mk(&g[..last]));
        }
    }
    // cheapest line-up (dynamic programme over groups x vowels)
    let (m, n) = (g.len(), n_vowels);
    if m == 0 || m > 24 || n > 24 {
        return exact;
    }
    const INF: u32 = u32::MAX / 4;
    let fit = |s: usize, e: usize, v: &str| -> u32 { if spells(v, &src[s..e]) { 0 } else { 4 } };
    // step: 0 match, 1 silent group, 2 group holds two vowels, 3 vowel without a letter
    let mut cost = vec![vec![INF; n + 1]; m + 1];
    let mut step = vec![vec![0u8; n + 1]; m + 1];
    cost[0][0] = 0;
    for i in 0..=m {
        for j in 0..=n {
            let c = cost[i][j];
            if c >= INF {
                continue;
            }
            let relax = |ii: usize, jj: usize, cc: u32, st: u8, cost: &mut Vec<Vec<u32>>, step: &mut Vec<Vec<u8>>| {
                if cc < cost[ii][jj] {
                    cost[ii][jj] = cc;
                    step[ii][jj] = st;
                }
            };
            if i < m && j < n {
                relax(i + 1, j + 1, c + fit(g[i].0, g[i].1, vowels[j]), 0, &mut cost, &mut step);
            }
            if i < m {
                let silent = if &src[g[i].0..g[i].1] == "e" { 1 } else { 5 };
                relax(i + 1, j, c + silent, 1, &mut cost, &mut step);
            }
            if i < m && j + 1 < n && g[i].1 - g[i].0 >= 2 {
                let mid = g[i].0 + 1;
                relax(i + 1, j + 2, c + 1 + fit(g[i].0, mid, vowels[j]) + fit(mid, g[i].1, vowels[j + 1]), 2, &mut cost, &mut step);
            }
            if j < n {
                let weak = matches!(vowels[j], "ə" | "ɐ" | "ᵻ" | "ɚ");
                relax(i, j + 1, c + if weak { 2 } else { 6 }, 3, &mut cost, &mut step);
            }
        }
    }
    // the counts agree but a pair looked wrong (espionage: the a is the ɑː, the final e is silent):
    // take the searched line-up only when it is clean, else the plain one
    if cost[m][n] > 2 * n as u32 + 2 || (exact.is_some() && cost[m][n] > 3) {
        return exact;
    }
    let mut out: Vec<Grp<'a>> = Vec::with_capacity(n);
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        match step[i][j] {
            0 => {
                out.push(mk_grp(src, g[i - 1].0, g[i - 1].1));
                i -= 1;
                j -= 1;
            }
            1 => i -= 1,
            2 => {
                let mid = g[i - 1].0 + 1;
                out.push(mk_grp(src, mid, g[i - 1].1));
                out.push(mk_grp(src, g[i - 1].0, mid));
                i -= 1;
                j -= 2;
            }
            _ => {
                let at = if i > 0 { g[i - 1].1 } else { 0 };
                out.push(mk_grp(src, at, at));
                j -= 1;
            }
        }
    }
    out.reverse();
    Some(out)
}

/// Is a GA /ɑː/ the LOT vowel (RP /ɒ/), or THOUGHT, or the PALM vowel?
#[derive(PartialEq)]
enum Open {
    Lot,
    Thought,
    Palm,
}

fn open_set(g: Option<Grp>, src: &str) -> Open {
    match g {
        Some(g) => {
            let l = g.letters;
            if l.is_empty() {
                Open::Palm
            } else if l == "ou" || l == "au" || l == "aw" || l == "oa" || (l == "a" && (g.after == "lk" || g.after == "ll" || g.after == "lt" || g.after == "ld")) {
                // bought, cause, law, broad; chalk, tall, salt, bald
                Open::Thought
            } else if l.contains('o') || l == "e" || l == "i" {
                Open::Lot
            } else if l == "a" && (g.before.ends_with('w') || g.before.ends_with("qu") || g.before.ends_with("wh")) {
                Open::Lot
            } else {
                Open::Palm
            }
        }
        // no line-up at all (a spelled-out acronym, a symbol): only an o-word without a is taken as LOT
        None => {
            if src.contains('o') && !src.contains('a') {
                Open::Lot
            } else {
                Open::Palm
            }
        }
    }
}

/// Is a GA /ɔː/ (not before r) the CLOTH vowel, which RP has as /ɒ/?
fn cloth(g: Option<Grp>, src: &str) -> bool {
    match g {
        Some(g) => {
            g.letters == "o"
                || (g.letters == "a"
                    && (g.before.ends_with('w') || g.before.ends_with("qu") || g.before.ends_with("wh"))
                    && !(g.after.starts_with('l') || g.after.starts_with('r') || g.after.starts_with("te")))
        }
        None => src.contains('o') && !["au", "aw", "al", "ou", "oa", "oo"].iter().any(|x| src.contains(x)),
    }
}

fn flap_letter(src: &str, toks: &[Tok], at: usize) -> &'static str {
    let (ht, hd) = (src.contains('t'), src.contains('d'));
    if ht && !hd {
        return "t";
    }
    if hd && !ht {
        return "d";
    }
    if !ht && !hd {
        return "t";
    }
    // both: line the t/d letters up with the t/d/flap phonemes when the counts agree
    let b = src.as_bytes();
    let mut letters: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if (c == b't' || c == b'd') && !(c == b't' && b.get(i + 1) == Some(&b'h')) {
            letters.push(c);
            while i + 1 < b.len() && b[i + 1] == c {
                i += 1;
            }
        }
        i += 1;
    }
    let idx: Vec<usize> = toks.iter().enumerate().filter(|(_, t)| matches!(t.p.as_str(), "t" | "d" | "ɾ" | "ʧ" | "ʤ" | "ʦ" | "ʣ" | "ʔ")).map(|(k, _)| k).collect();
    if letters.len() == idx.len() {
        if let Some(k) = idx.iter().position(|&k| k == at) {
            return if letters[k] == b'd' { "d" } else { "t" };
        }
    }
    "t"
}

/// Endings whose American and British forms differ as a class.
fn suffix_fix(ph: &str, src: &str) -> Option<String> {
    let swap = |from: &[&str], to: &str| -> Option<String> {
        for f in from {
            if let Some(stem) = ph.strip_suffix(f) {
                return Some(format!("{stem}{to}"));
            }
        }
        None
    };
    if src.len() >= 6 && (src.ends_with("ile") || src.ends_with("iles")) && !ph.contains("aɪl") {
        // fertile, hostile, missile: -aɪl
        return if src.ends_with('s') { swap(&["əlz", "ɪlz", "ᵻlz"], "aɪlz") } else { swap(&["əl", "ɪl", "ᵻl"], "aɪl") };
    }
    if src.len() >= 7 && (src.ends_with("ary") || src.ends_with("ery") || src.ends_with("ory")) {
        // secretary, territory: the American secondary stress is a schwa
        return swap(&["ˌɛɹi", "ˌɔːɹi"], "əɹi");
    }
    if src.len() >= 8 && (src.ends_with("aries") || src.ends_with("eries") || src.ends_with("ories")) {
        return swap(&["ˌɛɹiz", "ˌɔːɹiz"], "əɹiz");
    }
    if src.len() >= 7 && src.ends_with("ony") {
        return swap(&["ˌoʊni"], "əni");
    }
    if src.contains("ization") || src.contains("isation") {
        for f in ["ᵻzˈeɪʃən", "ɪzˈeɪʃən", "əzˈeɪʃən"] {
            if ph.contains(f) {
                return Some(ph.replace(f, "aɪzˈeɪʃən"));
            }
        }
    }
    None
}

/// What a rule set does with the American form.
#[derive(Clone, Copy)]
struct Rules {
    /// keep r everywhere (American r-coloured vowels stay as written)
    rhotic: bool,
    /// LOT: Some(symbol) rewrites it (ɒ, ɔ); None keeps ɑː
    lot: Option<&'static str>,
    /// THOUGHT when not before r: Some(symbol) rewrites it
    thought: Option<&'static str>,
    /// CLOTH (dog, off) joins LOT instead of THOUGHT
    cloth_is_lot: bool,
    /// PALM / START symbol when rewritten (Scottish a)
    palm: Option<&'static str>,
    goat: &'static str,
    face: &'static str,
    /// American flap back to t / d
    unflap: bool,
    /// j after t d n before uː (new, tune, duke)
    yod: bool,
    /// reduced-vowel notation: ɐ → ə, ᵻ → ɪ
    plain_reduced: bool,
    /// British endings (-ile, -ary, -ization)
    suffixes: bool,
    /// rhotic accents of the British Isles: hurry / marry / squirrel keep their short vowels before r
    brit_r: bool,
}

const GB: Rules = Rules { rhotic: false, lot: Some("ɒ"), thought: None, cloth_is_lot: true, palm: None, goat: "əʊ", face: "eɪ", unflap: true, yod: true, plain_reduced: true, suffixes: true, brit_r: false };
const SC: Rules = Rules { rhotic: true, lot: Some("ɔ"), thought: Some("ɔ"), cloth_is_lot: true, palm: Some("a"), goat: "oː", face: "eː", unflap: true, yod: true, plain_reduced: false, suffixes: true, brit_r: true };
const IE: Rules = Rules { rhotic: true, lot: None, thought: None, cloth_is_lot: false, palm: None, goat: "oː", face: "eː", unflap: true, yod: true, plain_reduced: false, suffixes: true, brit_r: true };
const CA: Rules = Rules { rhotic: true, lot: None, thought: Some("ɑː"), cloth_is_lot: false, palm: None, goat: "oʊ", face: "eɪ", unflap: false, yod: false, plain_reduced: false, suffixes: false, brit_r: false };

/// Rewrite one American word form. Returns the new form and whether the word
/// ends in an r that is only heard before a vowel (linking r).
/// STRUT before r in hurry, worry, courage; NURSE in furry, occurring, blurry
/// (a word in -ur plus an ending keeps the vowel of its base).
fn strut_before_r(g: Option<Grp>, src: &str) -> bool {
    let strut = match g {
        Some(g) => (matches!(g.letters, "u" | "o") && g.after.starts_with("rr")) || g.letters == "ou",
        None => ["urr", "orr", "our"].iter().any(|x| src.contains(x)),
    };
    if !strut {
        return false;
    }
    const NURSE_BASES: &[&str] = &["occur", "recur", "concur", "incur", "blur", "fur", "pur", "spur", "slur", "bur", "demur", "whir"];
    if let Some(at) = src.find("rr") {
        let ending = &src[at + 2..];
        if matches!(ending, "ing" | "ings" | "ed" | "y" | "er" | "ers" | "ier" | "iest" | "s") && NURSE_BASES.contains(&&src[..at + 1]) {
            return false;
        }
    }
    true
}

/// "or" + vowel with a short vowel in British speech (moral, historic, forest,
/// coroner, majority) as against story, glorious, memorial, boring.
fn short_or(g: Option<Grp>, src: &str) -> bool {
    let Some(g) = g else { return false };
    if g.letters == "a" {
        // quarrel, warrant, warrior
        return (g.before.ends_with('w') || g.before.ends_with("qu")) && g.after == "rr";
    }
    if g.letters != "o" || !g.after.starts_with('r') {
        return false;
    }
    if g.after == "rr" {
        return true;
    }
    let at = g.letters.as_ptr() as usize - src.as_ptr() as usize + 2;
    let rest = &src[at.min(src.len())..];
    // long: story, storey, glorious, memorial, chorus, boring, adorable, glorify, thorax, moron; oral, floral, choral
    let long = rest.is_empty()
        || ["y", "ey", "ia", "iu", "io", "ie", "eo", "us", "um", "ing", "ous", "able", "ably", "ify", "ifi", "ax"].iter().any(|x| rest.starts_with(x))
        || matches!(rest, "a" | "e" | "es" | "ed" | "er" | "ers" | "as" | "age" | "on")
        || (rest.starts_with("al") && !src.contains("moral") && !src.contains("coral"))
        || !rest.starts_with(['a', 'e', 'i', 'o', 'u', 'y']);
    !long
}

/// THOUGHT symbol of a rule set. Scottish shortens it to ɔ, but before ɪ that would
/// spell the CHOICE diphthong ɔɪ (drawing, sawing): the long vowel stays there.
fn thought_sym(r: Rules, next: &str, default: &'static str) -> &'static str {
    match r.thought {
        Some("ɔ") if next == "ɪ" => "ɔː",
        Some(t) => t,
        None => default,
    }
}

fn rules(ph: &str, src: &str, r: Rules) -> (String, bool) {
    let fixed = if r.suffixes { suffix_fix(ph, src) } else { None };
    let ph = fixed.as_deref().unwrap_or(ph);
    let toks = tokenize(ph);
    let n = toks.len();
    let vowel_idx: Vec<usize> = (0..n).filter(|&i| is_vowel(&toks[i].p)).collect();
    let vowels: Vec<&str> = vowel_idx.iter().map(|&i| toks[i].p.as_str()).collect();
    let groups = align(src, &vowels);
    let grp = |i: usize| -> Option<Grp> { groups.as_ref().and_then(|g| vowel_idx.iter().position(|&k| k == i).map(|k| g[k])) };
    let p = |i: usize| -> &str { toks.get(i).map_or("", |t| t.p.as_str()) };
    let vowel_at = |i: usize| -> bool { toks.get(i).is_some_and(|t| is_vowel(&t.p)) };
    let mut out: Vec<Tok> = Vec::with_capacity(n + 2);
    let mut final_r = false;
    let push = |out: &mut Vec<Tok>, stress: Option<char>, s: &str| out.push(Tok { stress, p: s.to_string() });
    for i in 0..n {
        let t = &toks[i];
        let last = i + 1 == n;
        let g = grp(i);
        let before_r = p(i + 1) == "ɹ";
        // r is heard when a vowel follows inside the word
        let r_kept = before_r && vowel_at(i + 2);
        match t.p.as_str() {
            "ɚ" if !r.rhotic => {
                push(&mut out, t.stress, "ə");
                if vowel_at(i + 1) {
                    push(&mut out, None, "ɹ");
                }
                final_r |= last;
            }
            "ɜː" if !r.rhotic => {
                if r_kept {
                    // hurry, worry, courage: STRUT; squirrel, stirrup: KIT; furry, stirring: NURSE
                    let kit = g.is_some_and(|g| (matches!(g.letters, "i" | "y") && g.after == "rr") || (g.letters == "y" && g.after.starts_with('r'))) && !src.ends_with("ing") && !src.ends_with("ed");
                    push(&mut out, t.stress, if kit { "ɪ" } else if strut_before_r(g, src) { "ʌ" } else { "ɜː" });
                } else {
                    push(&mut out, t.stress, "ɜː");
                    if vowel_at(i + 1) {
                        push(&mut out, None, "ɹ");
                    }
                }
                final_r |= last;
            }
            "ɜː" if r.brit_r && r_kept => {
                let kit = g.is_some_and(|g| (matches!(g.letters, "i" | "y") && g.after == "rr") || (g.letters == "y" && g.after.starts_with('r'))) && !src.ends_with("ing") && !src.ends_with("ed");
                push(&mut out, t.stress, if kit { "ɪ" } else if strut_before_r(g, src) { "ʌ" } else { "ɜː" });
            }
            // marry, carry, narrow: TRAP; Mary, vary, parent, square: the FACE vowel; merry, very keep DRESS
            "ɛ" if r.brit_r && r_kept && g.is_some_and(|g| g.letters == "a" && g.after == "rr") => push(&mut out, t.stress, "æ"),
            "ɛ" if r.brit_r && !r.rhotic && before_r && g.is_some_and(|g| matches!(g.letters, "a" | "ai" | "ae" | "ei")) => push(&mut out, t.stress, r.face),
            "ɹ" if !r.rhotic => {
                if vowel_at(i + 1) {
                    push(&mut out, None, "ɹ");
                }
                final_r |= last;
            }
            "ɑː" => {
                let set = open_set(g, src);
                if before_r {
                    // START; sorry / tomorrow / borrow have LOT
                    if r_kept && set == Open::Lot && r.lot.is_some() {
                        push(&mut out, t.stress, r.lot.unwrap());
                    } else {
                        push(&mut out, t.stress, r.palm.unwrap_or("ɑː"));
                    }
                } else {
                    match set {
                        Open::Lot => push(&mut out, t.stress, r.lot.unwrap_or("ɑː")),
                        Open::Thought => push(&mut out, t.stress, thought_sym(r, p(i + 1), if r.lot.is_some() { "ɔː" } else { "ɑː" })),
                        Open::Palm => push(&mut out, t.stress, r.palm.unwrap_or("ɑː")),
                    }
                }
            }
            "ɔː" => {
                if before_r {
                    // forest, orange keep a short vowel in British speech (word list); "orr" by rule
                    let short = !r.rhotic && r_kept && short_or(g, src);
                    push(&mut out, t.stress, if short { "ɒ" } else { "ɔː" });
                } else if r.cloth_is_lot && cloth(g, src) {
                    push(&mut out, t.stress, r.lot.unwrap_or("ɔː"));
                } else {
                    push(&mut out, t.stress, thought_sym(r, p(i + 1), "ɔː"));
                }
            }
            // the short ɔ of weak "on": LOT where LOT is rewritten, else the merged vowel (Canadian)
            "ɔ" => push(&mut out, t.stress, r.lot.or(r.thought).unwrap_or("ɔ")),
            // LOT written out in data/en_brit_words.tsv
            "ɒ" => push(&mut out, t.stress, r.lot.unwrap_or("ɑː")),
            "ɪ" | "iː" if before_r && !r.rhotic => {
                let short = r_kept
                    && t.p == "ɪ"
                    && match g {
                        Some(g) => matches!(g.letters, "i" | "y"),
                        None => src.contains("irr") || src.contains("yr"),
                    };
                push(&mut out, t.stress, if short { "ɪ" } else { "ɪə" });
            }
            "ɛ" if before_r && !r.rhotic => {
                let v = if !r_kept {
                    "eə"
                } else {
                    match g {
                        Some(g) if g.letters == "a" && g.after.starts_with("rr") => "æ",
                        Some(g) if matches!(g.letters, "a" | "ai" | "ea" | "ei" | "ae" | "aye") => "eə",
                        Some(_) => "ɛ",
                        None => {
                            if src.contains("arr") {
                                "æ"
                            } else if src.contains("er") {
                                "ɛ"
                            } else {
                                "eə"
                            }
                        }
                    }
                };
                push(&mut out, t.stress, v);
            }
            "ʊ" if before_r && (!r.rhotic || r.yod) => {
                // during, neural, endure: yod after t d n
                let yod = r.yod
                    && i > 0
                    && matches!(p(i - 1), "t" | "d" | "n")
                    && match g {
                        Some(g) => matches!(g.letters, "u" | "eu" | "ew"),
                        None => src.contains("ur") || src.contains("eur"),
                    };
                if yod {
                    push(&mut out, None, "j");
                }
                push(&mut out, t.stress, if r.rhotic { "ʊ" } else { "ʊə" });
            }
            "aɪ" | "aʊ" | "ɔɪ" if before_r && !r_kept && !r.rhotic => {
                push(&mut out, t.stress, &t.p);
                push(&mut out, None, "ə");
            }
            "oʊ" => push(&mut out, t.stress, r.goat),
            "eɪ" => push(&mut out, t.stress, r.face),
            "ɾ" if r.unflap => push(&mut out, None, flap_letter(src, &toks, i)),
            // kitten, certain, written: the American glottal stop + syllabic n is t + schwa + n
            "ʔ" if r.unflap => push(&mut out, None, "t"),
            "n̩" if r.unflap => {
                // Latin, satin, Martin: ɪ
                let i_spelt = src.ends_with("tin") || src.ends_with("tins");
                push(&mut out, None, if i_spelt { "ɪ" } else { "ə" });
                push(&mut out, None, "n");
            }
            "l̩" if r.unflap => {
                push(&mut out, None, "ə");
                push(&mut out, None, "l");
            }
            "uː" if r.yod && i > 0 && matches!(p(i - 1), "t" | "d" | "n") => {
                let yod = match g {
                    Some(g) => matches!(g.letters, "u" | "ue" | "ew" | "eu") || (g.letters == "e" && g.after.starts_with('w')) || (g.letters == "ui" && p(i - 1) == "n"),
                    None => src.contains("ew") || src.contains("eu") || (src.contains('u') && !src.contains("oo") && !src.contains("ou")),
                };
                if yod {
                    push(&mut out, None, "j");
                }
                push(&mut out, t.stress, "uː");
            }
            "ɐ" if r.plain_reduced => push(&mut out, t.stress, "ə"),
            "ᵻ" if r.plain_reduced => push(&mut out, t.stress, "ɪ"),
            _ => out.push(t.clone()),
        }
    }
    (render(&out), final_r)
}

fn ends_in_r(ga: &str) -> bool {
    ga.ends_with('ɹ') || ga.ends_with('ɚ') || ga.ends_with("ɜː")
}

/// Abbreviations the sentence pass reads as a full word ("Dr. Smith"): the spelling rules need that word.
const ABBR: &[(&str, &str)] = &[
    ("dr", "doctor"), ("mr", "mister"), ("mrs", "missus"), ("jr", "junior"), ("sr", "senior"), ("prof", "professor"), ("mt", "mount"),
    ("rev", "reverend"), ("capt", "captain"), ("lt", "lieutenant"), ("sgt", "sergeant"), ("gov", "governor"), ("sen", "senator"),
    ("vs", "versus"), ("etc", "etcetera"), ("inc", "incorporated"), ("ltd", "limited"), ("st", "saint"), ("st", "street"),
];

fn expand_abbr<'a>(en: &English, ph: &str, src: &'a str) -> &'a str {
    ABBR.iter().find(|(a, full)| *a == src && en.lexicon_form(full) == Some(ph)).map_or(src, |(_, full)| *full)
}

/// The British form (American notation) of a word of data/en_brit_words.tsv, or of
/// its possessive, when the engine gave the word its plain reading.
fn brit_form(en: &English, ph: &str, src: &str) -> Option<String> {
    if src.is_empty() || en.homograph_word(src) || !en.is_default_form(src, ph) {
        return None;
    }
    if let Some(b) = brit_words().get(src) {
        return Some(b.to_string());
    }
    let b = brit_words().get(src.strip_suffix("'s")?)?;
    let last = b.chars().last()?;
    Some(match last {
        't' => format!("{}ʦ", &b[..b.len() - 1]),
        'd' => format!("{}ʣ", &b[..b.len() - 1]),
        's' | 'z' | 'ʃ' | 'ʒ' | 'ʧ' | 'ʤ' => format!("{b}ᵻz"),
        'p' | 'k' | 'f' | 'θ' => format!("{b}s"),
        _ => format!("{b}z"),
    })
}

/// How far two forms are apart: edits over phonemes, a stress mark counting as
/// part of its vowel.
fn distance(a: &str, b: &str) -> usize {
    let (a, b) = (tokenize(a), tokenize(b));
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, y) in b.iter().enumerate() {
            cur.push((prev[j] + usize::from(x != y)).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The listed British reading of a context-dependent word that belongs to the
/// reading the engine chose: the one nearest to that reading run through the
/// rules. `None` (keep the rule form) when two listed readings are equally near
/// or the nearest is far off, so the engine's choice is never overridden.
fn pick_reading<'a>(vars: &'a str, by_rule: &str) -> Option<&'a str> {
    // same number of vowels and the same stresses on them: separate (verb, -ˌeɪt) is not separate (adjective)
    let stresses = |p: &str| -> Vec<Option<char>> { tokenize(p).iter().filter(|t| is_vowel(&t.p)).map(|t| t.stress).collect() };
    let want = stresses(by_rule);
    let mut best: Option<(usize, &str)> = None;
    let mut tie = false;
    let cands: Vec<&str> = vars.split('|').filter(|v| stresses(v) == want).collect();
    // the only listed reading with the engine's stress pattern is the reading, however far its vowels are (produce noun)
    if let [only] = cands[..] {
        return Some(only);
    }
    for v in cands {
        let d = distance(v, by_rule);
        match best {
            Some((bd, _)) if d == bd => tie = true,
            Some((bd, _)) if d > bd => {}
            _ => {
                best = Some((d, v));
                tie = false;
            }
        }
    }
    best.filter(|&(d, _)| !tie && d <= 2).map(|(_, v)| v)
}

/// Southern British form of one word of the American output. `strong`: the
/// word stands before a pause, where of / from / was / been are not reduced.
fn to_gb(en: &English, ph: &str, src: &str, use_lex: bool, strong: bool) -> (String, bool) {
    // bin is not been: the table's bare bɪn rows are the weak forms of "been" and keep only that spelling.
    // (merged pairs such as hɐvbɪn come with the first word as source and are "have been" by construction)
    let is_bin = ph.replace(['ˈ', 'ˌ'], "") == "bɪn";
    if let Some(&(weak, full)) = gb_func().get(ph).filter(|_| !is_bin || src.to_lowercase().ends_with("been")) {
        return ((if strong { full } else { weak }).to_string(), ends_in_r(ph));
    }
    let src = expand_abbr(en, ph, src);
    if use_lex && !src.is_empty() {
        if en.homograph_word(src) {
            // the list holds every British reading; take the one that matches the reading the engine chose
            if let Some(vars) = gb_lex().get(src) {
                let (by_rule, fr) = rules(ph, src, GB);
                return (pick_reading(vars, &by_rule).map_or(by_rule.clone(), |v| v.to_string()), fr);
            }
        } else if en.is_default_form(src, ph) {
            if let Some(g) = gb_lex().get(src) {
                return (g.split('|').next().unwrap_or(g).to_string(), ends_in_r(ph));
            }
            // an inflected form of a listed word keeps the listed stem (passes, classes, dancing, aunts)
            for suf in ["'s", "s", "es", "ed", "ing", "er", "ers", "ly", "est", "ier", "iest", "ily"] {
                let Some(stem) = src.strip_suffix(suf) else { continue };
                if stem.len() < 3 {
                    continue;
                }
                let mut cands = vec![stem.to_string()];
                if matches!(suf, "ed" | "ing" | "er" | "ers" | "es" | "est") {
                    cands.push(format!("{stem}e"));
                }
                if matches!(suf, "ier" | "iest" | "ily") {
                    cands = vec![format!("{stem}y")];
                }
                for c in cands {
                    if en.homograph_word(&c) {
                        continue;
                    }
                    let (Some(g), Some(ga_stem)) = (gb_lex().get(c.as_str()), en.lexicon_form(&c)) else { continue };
                    // t + s and d + z are one letter in the engine's output (aunt, aunts: ˈænt, ˈænʦ)
                    if matches!(suf, "s" | "'s") {
                        // path, paths: θ, ðz
                        let fused = [("t", "ʦ"), ("d", "ʣ"), ("θ", "ðz")].iter().find_map(|(c1, lig)| {
                            let (ga0, g0) = (ga_stem.strip_suffix(c1)?, g.strip_suffix(c1)?);
                            (ph.strip_prefix(ga0)? == *lig).then(|| format!("{g0}{lig}"))
                        });
                        if let Some(f) = fused {
                            return (f, false);
                        }
                    }
                    // nasty, nastier: the stem's final i belongs to the ending
                    let ga_stem = if matches!(suf, "ier" | "iest" | "ily") { ga_stem.strip_suffix('i').unwrap_or(ga_stem) } else { ga_stem };
                    let g: &str = if matches!(suf, "ier" | "iest" | "ily") { g.strip_suffix('i').unwrap_or(g) } else { g };
                    let Some(rest) = ph.strip_prefix(ga_stem) else { continue };
                    if rest.is_empty() || rest.chars().any(|x| x == 'ˈ') {
                        continue;
                    }
                    let (tail, fr) = rules(rest, suf, GB);
                    let link = if ends_in_r(ga_stem) && !g.ends_with('ɹ') && tail.chars().next().is_some_and(|x| VOWELS.contains(x)) { "ɹ" } else { "" };
                    return (format!("{g}{link}{tail}"), fr);
                }
            }
        }
    }
    // British word forms the list does not hold (vase, schedule, yoghurt)
    if use_lex {
        if let Some(b) = brit_form(en, ph, src) {
            return rules(&b, src, GB);
        }
    }
    rules(ph, src, GB)
}

fn unstressed_i_to_schwa(toks: &mut [Tok]) {
    // weak vowel merger (rabbit = abbot): unstressed ɪ before t d s z, not in the first syllable
    let has_primary = toks.iter().any(|t| t.stress == Some('ˈ'));
    if !has_primary {
        return;
    }
    let first_v = toks.iter().position(|t| is_vowel(&t.p));
    for i in 0..toks.len() {
        if toks[i].p == "ɪ" && toks[i].stress.is_none() && Some(i) != first_v && toks.get(i + 1).is_some_and(|t| matches!(t.p.as_str(), "t" | "d" | "s" | "z")) {
            toks[i].p = "ə".into();
        }
    }
}

/// Australian and New Zealand rules on the Southern British form.
fn antipodean(gb: &str, src: &str, a: Accent) -> String {
    let mut toks = tokenize(gb);
    unstressed_i_to_schwa(&mut toks);
    let n = toks.len();
    for i in 0..n {
        let next = toks.get(i + 1).map(|t| t.p.clone()).unwrap_or_default();
        let next2 = toks.get(i + 2).map(|t| t.p.clone()).unwrap_or_default();
        match toks[i].p.as_str() {
            // dance, chance, plant, example: TRAP in Australia (BATH stays broad in grass, path, can't, aunt)
            "ɑː" if a == Accent::Au
                && matches!(next.as_str(), "n" | "m")
                && matches!(next2.as_str(), "s" | "t" | "d" | "p" | "ʧ" | "ʦ" | "ʣ")
                && ["anc", "ant", "and", "amp", "ans", "anch"].iter().any(|x| src.contains(x))
                && !src.contains("ar")
                && !matches!(src, "can't" | "cant" | "shan't" | "aunt" | "aunts") =>
            {
                toks[i].p = "æ".into();
            }
            // sure, poor, tour: THOUGHT; pure, cure keep the diphthong
            // (after ʧ / ʤ too: mature, endure keep it like during, whose ʤ comes from dj)
            "ʊə" if i == 0 || !matches!(toks[i - 1].p.as_str(), "j" | "ʧ" | "ʤ") => toks[i].p = "ɔː".into(),
            // New Zealand: NEAR and SQUARE are one vowel
            "eə" if a == Accent::Nz => toks[i].p = "ɪə".into(),
            _ => {}
        }
    }
    // tune, dune, during: tj → ʧ, dj → ʤ before the u vowels only (not court-yard, Good-year)
    let mut out: Vec<Tok> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let u_next = toks.get(i + 2).is_some_and(|t| matches!(t.p.as_str(), "uː" | "u" | "ʊ" | "ʊə" | "ə"));
        if matches!(toks[i].p.as_str(), "t" | "d") && toks.get(i + 1).is_some_and(|t| t.p == "j") && u_next {
            out.push(Tok { stress: None, p: if toks[i].p == "t" { "ʧ".into() } else { "ʤ".into() } });
            i += 2;
            continue;
        }
        out.push(toks[i].clone());
        i += 1;
    }
    render(&out)
}

fn wh(ph: &str, src: &str) -> Option<String> {
    // which / witch are different words in Scotland and Ireland; also inside somewhere, meanwhile, overwhelm
    let at = src.find("wh")?;
    if at == 0 {
        // who, whole start with h; whopping, whoa with the wh sound
        return ph.strip_prefix('w').map(|rest| format!("ʍ{rest}"));
    }
    if src[at..].starts_with("who") {
        return None;
    }
    (ph.matches('w').count() == 1).then(|| ph.replace('w', "ʍ"))
}

fn scottish(ph: &str) -> String {
    // TRAP = PALM, FOOT = GOOSE
    let mut toks = tokenize(ph);
    for t in toks.iter_mut() {
        match t.p.as_str() {
            "æ" => t.p = "a".into(),
            "ʊ" | "uː" => t.p = "u".into(),
            _ => {}
        }
    }
    render(&toks)
}

fn starts_with_vowel(ph: &str) -> bool {
    ph.chars().find(|&c| c != 'ˈ' && c != 'ˌ').is_some_and(|c| VOWELS.contains(c))
}

/// One word of the American output in the given accent (no linking r across
/// words). Returns the form and whether it ends in an r that is only heard
/// before a vowel.
pub fn word(en: &English, ph: &str, src: &str, a: Accent) -> (String, bool) {
    word_at(en, ph, src, a, false, true)
}

/// As `word`, with the word list switched off (`use_lex` false: rules only, for
/// tools/en_gb_build.py) or the word standing before a pause (`strong`).
pub fn word_at(en: &English, ph: &str, src: &str, a: Accent, strong: bool, use_lex: bool) -> (String, bool) {
    // a hyphenated compound comes as its parts: each part is a word of its own (half-past, far-off)
    if src.contains('-') && ph.contains(' ') {
        let (pp, sp): (Vec<&str>, Vec<&str>) = (ph.split(' ').collect(), src.split('-').filter(|x| !x.is_empty()).collect());
        if pp.len() == sp.len() {
            let mut parts: Vec<(String, bool)> = pp.iter().zip(&sp).enumerate().map(|(k, (p, s))| word_at(en, p, s, a, strong && k + 1 == pp.len(), use_lex)).collect();
            for k in 0..parts.len() - 1 {
                if parts[k].1 && !a.rhotic() && starts_with_vowel(&parts[k + 1].0) && !parts[k].0.ends_with('ɹ') {
                    parts[k].0.push('ɹ');
                }
            }
            let fr = parts.last().is_some_and(|x| x.1);
            return (parts.into_iter().map(|x| x.0).collect::<Vec<_>>().join(" "), fr);
        }
    }
    // a hyphenated word the lexicon holds as one form (half-hour): the first part, then the rest
    if let Some((first, rest_src)) = src.split_once('-') {
        if !ph.contains(' ') && !first.is_empty() && !rest_src.is_empty() {
            if let Some(rest) = en.lexicon_form(first).and_then(|ga| ph.strip_prefix(ga)).filter(|r| !r.is_empty()) {
                let (mut head, fr_head) = word_at(en, &ph[..ph.len() - rest.len()], first, a, false, use_lex);
                let (tail, fr) = word_at(en, rest, rest_src, a, strong, use_lex);
                if fr_head && !a.rhotic() && starts_with_vowel(&tail) && !head.ends_with('ɹ') {
                    head.push('ɹ');
                }
                return (format!("{head}{tail}"), fr);
            }
        }
    }
    // a spelled-out acronym (ROI, CRO): letter names, the spelling says nothing about their vowels
    let spelled = !src.is_empty() && en.is_spelled(src, ph);
    let rule_src = if spelled { "" } else { src };
    match a {
        Accent::Gb => to_gb(en, ph, rule_src, use_lex && !spelled, strong),
        Accent::Au | Accent::Nz => {
            let (g, fr) = to_gb(en, ph, rule_src, use_lex && !spelled, strong);
            (antipodean(&g, src, a), fr)
        }
        Accent::In => {
            let (g, fr) = to_gb(en, ph, rule_src, use_lex && !spelled, strong);
            let mut toks = tokenize(&g);
            for t in toks.iter_mut() {
                match t.p.as_str() {
                    "eɪ" => t.p = "eː".into(),
                    "əʊ" => t.p = "oː".into(),
                    _ => {}
                }
            }
            (render(&toks), fr)
        }
        Accent::Sc | Accent::Ie => {
            let src = expand_abbr(en, ph, rule_src);
            // British word forms (herb, tomato, schedule) when the engine gave the word its plain reading
            let brit = brit_form(en, ph, src);
            let (s, _) = rules(brit.as_deref().unwrap_or(ph), src, if a == Accent::Sc { SC } else { IE });
            let s = if a == Accent::Sc { scottish(&s) } else { s };
            (wh(&s, src).unwrap_or(s), false)
        }
        Accent::Ca => (rules(ph, expand_abbr(en, ph, rule_src), CA).0, false),
    }
}

/// Rewrite the finished American words of one sentence into the accent.
pub fn apply(en: &English, outs: &mut [Out], a: Accent) {
    let n = outs.len();
    let mut link = vec![false; n];
    let last = (0..n).rev().find(|&k| !outs[k].ph.is_empty());
    for k in 0..n {
        if outs[k].ph.is_empty() {
            continue;
        }
        let glued = outs[k].ph.starts_with('\u{0}');
        let ph = outs[k].ph.trim_start_matches('\u{0}').to_string();
        let strong = outs[k].pause_after || Some(k) == last;
        let (new, fr) = word_at(en, &ph, &outs[k].src, a, strong, true);
        outs[k].ph = if glued { format!("\u{0}{new}") } else { new };
        link[k] = fr;
    }
    if a.rhotic() {
        return;
    }
    // linking r: "far away", "for a" keep the r that "far." drops
    for k in 0..n {
        if !link[k] || outs[k].pause_after {
            continue;
        }
        if let Some(next) = outs[k + 1..].iter().find(|o| !o.ph.is_empty()) {
            if starts_with_vowel(next.ph.trim_start_matches('\u{0}')) && !outs[k].ph.ends_with('ɹ') {
                outs[k].ph.push('ɹ');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gb(ph: &str, src: &str) -> String {
        rules(ph, src, GB).0
    }

    #[test]
    fn tokens_round_trip() {
        for s in ["sˈɛkɹətˌɛɹi", "ɐbˌaʊt", "fˈaɪɚ", "ʧˈɜːʧ", "nˈuː"] {
            assert_eq!(render(&tokenize(s)), s);
        }
    }

    #[test]
    fn non_rhotic() {
        assert_eq!(gb("kˈɑːɹ", "car"), "kˈɑː");
        assert_eq!(gb("nˈɪɹ", "near"), "nˈɪə");
        assert_eq!(gb("skwˈɛɹ", "square"), "skwˈeə");
        assert_eq!(gb("lˈɛɾɚ", "letter"), "lˈɛtə");
        assert_eq!(gb("fˈaɪɚ", "fire"), "fˈaɪə");
        assert_eq!(gb("bˈɜːd", "bird"), "bˈɜːd");
        assert_eq!(gb("stˈɔːɹi", "story"), "stˈɔːɹi");
        assert_eq!(gb("vˈɛɹi", "very"), "vˈɛɹi");
        assert_eq!(gb("hˈɜːɹi", "hurry"), "hˈʌɹi");
        assert_eq!(gb("sˈɑːɹi", "sorry"), "sˈɒɹi");
        assert_eq!(gb("hˈɪɹoʊ", "hero"), "hˈɪəɹəʊ");
        assert_eq!(gb("spˈɪɹɪt", "spirit"), "spˈɪɹɪt");
        assert!(rules("fˈɑːɹ", "far", GB).1);
        assert!(!rules("fˈɑːðɚz", "fathers", GB).1);
    }

    #[test]
    fn lot_goat_flap_yod() {
        assert_eq!(gb("hˈɑːt", "hot"), "hˈɒt");
        assert_eq!(gb("fˈɑːðɚ", "father"), "fˈɑːðə");
        assert_eq!(gb("wˈɑːʧ", "watch"), "wˈɒʧ");
        assert_eq!(gb("dˈɔːɡ", "dog"), "dˈɒɡ");
        assert_eq!(gb("lˈɔː", "law"), "lˈɔː");
        assert_eq!(gb("hˈoʊm", "home"), "hˈəʊm");
        assert_eq!(gb("sˈɪɾi", "city"), "sˈɪti");
        assert_eq!(gb("lˈæɾɚ", "ladder"), "lˈædə");
        assert_eq!(gb("nˈuː", "new"), "njˈuː");
        assert_eq!(gb("tˈuːn", "tune"), "tjˈuːn");
        assert_eq!(gb("nˈuːn", "noon"), "nˈuːn");
        assert_eq!(gb("fˈɜːɾɪl", "fertile"), "fˈɜːtaɪl");
        assert_eq!(gb("sˈɛkɹətˌɛɹi", "secretary"), "sˈɛkɹətəɹi");
    }

    #[test]
    fn review_cases() {
        // ɪ + əʊ is not ɪə + ʊ
        assert_eq!(tokenize("vˈɪdɪəʊ").iter().map(|t| t.p.as_str()).collect::<Vec<_>>(), ["v", "ɪ", "d", "ɪ", "əʊ"]);
        // affricate letters after the nasal
        assert_eq!(antipodean("plˈɑːnʦ", "plants", Accent::Au), "plˈænʦ");
        assert_eq!(antipodean("kəmˈɑːnʣ", "commands", Accent::Au), "kəmˈænʣ");
        assert_eq!(antipodean("bɹˈɑːnʧ", "branch", Accent::Au), "bɹˈænʧ");
        assert_eq!(antipodean("bɹˈɑːnʧ", "branch", Accent::Nz), "bɹˈɑːnʧ");
        // yod coalescence only before the u vowels
        assert_eq!(antipodean("kˈɔːtjˌɑːd", "courtyard", Accent::Au), "kˈɔːtjˌɑːd");
        assert_eq!(antipodean("ɡˈʊdjɪə", "goodyear", Accent::Au), "ɡˈʊdjɪə");
        assert_eq!(antipodean("djˈʊəɹɪŋ", "during", Accent::Au), "ʤˈʊəɹɪŋ");
        assert_eq!(antipodean("məʧˈʊə", "mature", Accent::Au), "məʧˈʊə");
        // NURSE in a word in -ur plus an ending, STRUT in hurry
        assert_eq!(gb("əkˈɜːɹɪŋ", "occurring"), "əkˈɜːɹɪŋ");
        assert_eq!(gb("blˈɜːɹi", "blurry"), "blˈɜːɹi");
        assert_eq!(gb("kˈɜːɹi", "curry"), "kˈʌɹi");
        assert_eq!(gb("skwˈɜːɹəl", "squirrel"), "skwˈɪɹəl");
        // LOT before r
        assert_eq!(gb("kwˈɔːɹəl", "quarrel"), "kwˈɒɹəl");
        assert_eq!(gb("ɪmˈɔːɹəl", "immoral"), "ɪmˈɒɹəl");
        assert_eq!(gb("kˈɔːɹənɚ", "coroner"), "kˈɒɹənə");
        assert_eq!(gb("ɡlˈɔːɹi", "glory"), "ɡlˈɔːɹi");
        assert_eq!(gb("məmˈɔːɹiəl", "memorial"), "məmˈɔːɹiəl");
        // spelling line-up when the counts disagree
        assert_eq!(gb("əfˈɔːɹθˌɑːt", "aforethought"), "əfˈɔːθˌɔːt");
        assert_eq!(gb("kˈoʊlslˌɑː", "coleslaw"), "kˈəʊlslˌɔː");
        assert_eq!(gb("ˌɪmpɹɪsˈɑːɹiˌoʊ", "impresario"), "ˌɪmpɹɪsˈɑːɹiˌəʊ");
        // letter names: no spelling to go by
        assert_eq!(gb("ˌɑːɹˌoʊˈaɪ", ""), "ˌɑːɹˌəʊˈaɪ");
        // glottal stop + syllabic n
        assert_eq!(gb("kˈɪʔn̩", "kitten"), "kˈɪtən");
        assert_eq!(rules("ɹˈɪʔn̩", "written", IE).0, "ɹˈɪtən");
        assert_eq!(rules("kˈɪʔn̩", "kitten", CA).0, "kˈɪʔn̩");
        // rhotic accents of the British Isles
        assert_eq!(rules("hˈɜːɹi", "hurry", IE).0, "hˈʌɹi");
        assert_eq!(rules("mˈɛɹi", "marry", IE).0, "mˈæɹi");
        assert_eq!(rules("mˈɛɹi", "merry", IE).0, "mˈɛɹi");
        assert_eq!(rules("dˈʊɹɪŋ", "during", IE).0, "djˈʊɹɪŋ");
        assert_eq!(rules("jˈɒt", "yacht", IE).0, "jˈɑːt");
        assert_eq!(scottish(&rules("jˈɒt", "yacht", SC).0), "jˈɔt");
        assert_eq!(rules("ˌɔn", "on", CA).0, "ˌɑːn");
        assert_eq!(wh("sˈʌmwˌɛɹ", "somewhere").as_deref(), Some("sˈʌmʍˌɛɹ"));
        assert_eq!(wh("hˈoʊl", "whole"), None);
        // readings of a context-dependent word: the nearest listed one, never against the engine's choice
        assert_eq!(pick_reading("ɹˈɛkɔːd|ɹɪkˈɔːd", "ɹˈɛkəd"), Some("ɹˈɛkɔːd"));
        assert_eq!(pick_reading("ɹˈɛkɔːd|ɹɪkˈɔːd", "ɹɪkˈɔːd"), Some("ɹɪkˈɔːd"));
        assert_eq!(pick_reading("ɹˈiːd|ɹˈɛd", "ɹˈɛd"), Some("ɹˈɛd"));
        assert_eq!(pick_reading("jˈuːsɪz|jˈuːzɪz", "jˈuːzɪz"), Some("jˈuːzɪz"));
        assert_eq!(pick_reading("mˈɒdəɹˌeɪt|mˈɒdəɹɪt", "mˈɒdəɹət"), Some("mˈɒdəɹɪt"));
        assert_eq!(pick_reading("lˈɪv|lˈaɪv", "lˈaɪv"), Some("lˈaɪv"));
        assert_eq!(pick_reading("sˈɛpəɹɪt", "sˈɛpəɹˌeɪt"), None);
        // equally near, or nothing near: the rule form stays
        assert_eq!(pick_reading("wˈɪnd|wˈaɪnd", "wˈʌnd"), None);
        assert_eq!(pick_reading("kəntɹˈɑːst", "kˈɒntɹæst"), None);
        // more of the review's words
        assert_eq!(gb("ˈɔːɹəl", "oral"), "ˈɔːɹəl");
        assert_eq!(gb("stˈɔːɹi", "storey"), "stˈɔːɹi");
        assert_eq!(gb("ɐdˈɔːɹəbəl", "adorable"), "ədˈɔːɹəbəl");
        assert_eq!(gb("mˈɔːɹəl", "moral"), "mˈɒɹəl");
        assert_eq!(gb("lˈæʔn̩", "latin"), "lˈætɪn");
        assert_eq!(gb("ˈɛspiənɑːʤ", "espionage"), "ˈɛspiənɑːʤ");
        assert_eq!(rules("mˈɛɹi", "mary", GB).0, "mˈeəɹi");
        assert_eq!(rules("mˈɛɹi", "mary", IE).0, "mˈɛɹi");
        assert_eq!(rules("sˈɜːɹəp", "syrup", IE).0, "sˈɪɹəp");
        assert_eq!(wh("wˈɑːpɪŋ", "whopping").as_deref(), Some("ʍˈɑːpɪŋ"));
    }

    #[test]
    fn other_accents() {
        assert_eq!(antipodean("dˈɑːns", "dance", Accent::Au), "dˈæns");
        assert_eq!(antipodean("dˈɑːns", "dance", Accent::Nz), "dˈɑːns");
        assert_eq!(antipodean("ɡɹˈɑːs", "grass", Accent::Au), "ɡɹˈɑːs");
        assert_eq!(antipodean("ɹˈæbɪt", "rabbit", Accent::Au), "ɹˈæbət");
        assert_eq!(antipodean("tjˈuːn", "tune", Accent::Au), "ʧˈuːn");
        assert_eq!(antipodean("skwˈeə", "square", Accent::Nz), "skwˈɪə");
        assert_eq!(antipodean("ʃˈʊə", "sure", Accent::Au), "ʃˈɔː");
        assert_eq!(antipodean("pjˈʊə", "pure", Accent::Au), "pjˈʊə");
        assert_eq!(scottish(&rules("fˈʊt", "foot", SC).0), "fˈut");
        assert_eq!(scottish(&rules("hˈoʊm", "home", SC).0), "hˈoːm");
        assert_eq!(scottish(&rules("kˈɑːt", "cot", SC).0), "kˈɔt");
        assert_eq!(scottish(&rules("kˈɔːt", "caught", SC).0), "kˈɔt");
        assert_eq!(scottish(&rules("kˈɑːɹ", "car", SC).0), "kˈaɹ");
        assert_eq!(wh("wˌɪʧ", "which").as_deref(), Some("ʍˌɪʧ"));
        assert_eq!(wh("hˌuː", "who"), None);
        assert_eq!(rules("kˈɔːt", "caught", CA).0, "kˈɑːt");
        assert_eq!(rules("fˈɔːɹ", "four", CA).0, "fˈɔːɹ");
        assert_eq!(rules("dˈeɪ", "day", IE).0, "dˈeː");
    }
}
