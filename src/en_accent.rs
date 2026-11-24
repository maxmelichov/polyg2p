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
//!   cannot get from the American form (BATH words, schedule, tomato, ...).
//!   `en-au`, `en-nz`, `en-za`, `en-in` are rule sets applied to the `en-gb`
//!   form; no open pronouncing dictionary with a licence that allows
//!   redistribution exists for them, so they are rule-derived.
//! * rhotic (`en-sc`, `en-ie`, `en-ca`): rules on the American form.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accent {
    /// Southern British (RP / SSBE)
    Gb,
    /// Australian
    Au,
    /// New Zealand
    Nz,
    /// South African
    Za,
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

fn gb_func() -> &'static HashMap<&'static str, &'static str> {
    static T: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    T.get_or_init(|| table(GB_FUNC_TSV))
}

/// One phoneme with the stress mark that precedes it.
#[derive(Debug, Clone, PartialEq)]
struct Tok {
    stress: Option<char>,
    p: String,
}

const UNITS: &[&str] = &["aɪ", "aʊ", "eɪ", "oʊ", "ɔɪ", "əʊ", "ɪə", "eə", "ʊə", "tʃ", "dʒ"];
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
        if UNITS.contains(&two.as_str()) {
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

/// Vowel-letter groups of a word, one per vowel phoneme when the counts agree
/// (silent final e and the e of -es/-ed are tried as silent). `None` when the
/// spelling cannot be lined up with the vowels.
fn align<'a>(src: &'a str, n_vowels: usize) -> Option<Vec<Grp<'a>>> {
    if !src.is_ascii() || n_vowels == 0 {
        return None;
    }
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
            if &src[s..i] == "u" && s > 0 && b[s - 1] == b'q' && i < b.len() && isv(i) {
                continue;
            }
            let s = if s > 0 && b[s - 1] == b'q' && b[s] == b'u' && i - s > 1 { s + 1 } else { s };
            g.push((s, i));
        } else {
            i += 1;
        }
    }
    let mk = |g: &[(usize, usize)]| -> Vec<Grp<'a>> {
        g.iter().map(|&(s, e)| Grp { letters: &src[s..e], before: &src[s.saturating_sub(2)..s], after: &src[e..(e + 2).min(src.len())] }).collect()
    };
    if g.len() == n_vowels {
        return Some(mk(&g));
    }
    if g.len() == n_vowels + 1 {
        let last = g.len() - 1;
        let (s, e) = g[last];
        // silent final e (made), or the e of -es / -ed (makes, liked)
        let silent_e = &src[s..e] == "e" && s > 0 && (e == b.len() || (e + 1 == b.len() && matches!(b[e], b's' | b'd')));
        if silent_e {
            return Some(mk(&g[..last]));
        }
    }
    None
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
            if l == "ou" || l == "au" || l == "aw" || l == "oa" || (l == "a" && (g.after == "lk" || g.after == "ll" || g.after == "lt" || g.after == "ld")) {
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
        None => {
            if src.contains('o') {
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
    let idx: Vec<usize> = toks.iter().enumerate().filter(|(_, t)| matches!(t.p.as_str(), "t" | "d" | "ɾ" | "tʃ" | "dʒ")).map(|(k, _)| k).collect();
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
}

const GB: Rules = Rules { rhotic: false, lot: Some("ɒ"), thought: None, cloth_is_lot: true, palm: None, goat: "əʊ", face: "eɪ", unflap: true, yod: true, plain_reduced: true, suffixes: true };
const SC: Rules = Rules { rhotic: true, lot: Some("ɔ"), thought: Some("ɔ"), cloth_is_lot: true, palm: Some("a"), goat: "oː", face: "eː", unflap: true, yod: true, plain_reduced: false, suffixes: false };
const IE: Rules = Rules { rhotic: true, lot: None, thought: None, cloth_is_lot: false, palm: None, goat: "oː", face: "eː", unflap: true, yod: true, plain_reduced: false, suffixes: false };
const CA: Rules = Rules { rhotic: true, lot: None, thought: Some("ɑː"), cloth_is_lot: false, palm: None, goat: "oʊ", face: "eɪ", unflap: false, yod: false, plain_reduced: false, suffixes: false };

/// Rewrite one American word form. Returns the new form and whether the word
/// ends in an r that is only heard before a vowel (linking r).
fn rules(ph: &str, src: &str, r: Rules) -> (String, bool) {
    let fixed = if r.suffixes { suffix_fix(ph, src) } else { None };
    let ph = fixed.as_deref().unwrap_or(ph);
    let toks = tokenize(ph);
    let n = toks.len();
    let vowel_idx: Vec<usize> = (0..n).filter(|&i| is_vowel(&toks[i].p)).collect();
    let groups = align(src, vowel_idx.len());
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
                    // hurry, worry, courage: STRUT; furry, stirring: NURSE
                    let strut = match g {
                        Some(g) => (matches!(g.letters, "u" | "o") && g.after.starts_with("rr")) || g.letters == "ou",
                        None => ["urr", "orr", "our"].iter().any(|x| src.contains(x)),
                    };
                    push(&mut out, t.stress, if strut { "ʌ" } else { "ɜː" });
                } else {
                    push(&mut out, t.stress, "ɜː");
                    if vowel_at(i + 1) {
                        push(&mut out, None, "ɹ");
                    }
                }
                final_r |= last;
            }
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
                        Open::Thought => push(&mut out, t.stress, r.thought.unwrap_or(if r.lot.is_some() { "ɔː" } else { "ɑː" })),
                        Open::Palm => push(&mut out, t.stress, r.palm.unwrap_or("ɑː")),
                    }
                }
            }
            "ɔː" => {
                if before_r {
                    // forest, orange keep a short vowel in British speech (word list); "orr" by rule
                    let short = !r.rhotic && r_kept && g.is_some_and(|g| g.letters == "o" && g.after.starts_with("rr"));
                    push(&mut out, t.stress, if short { "ɒ" } else { "ɔː" });
                } else if r.cloth_is_lot && cloth(g, src) {
                    push(&mut out, t.stress, r.lot.unwrap_or("ɔː"));
                } else {
                    push(&mut out, t.stress, r.thought.unwrap_or("ɔː"));
                }
            }
            "ɔ" => push(&mut out, t.stress, r.lot.unwrap_or("ɔ")),
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
            "ʊ" if before_r && !r.rhotic => push(&mut out, t.stress, "ʊə"),
            "aɪ" | "aʊ" | "ɔɪ" if before_r && !r_kept && !r.rhotic => {
                push(&mut out, t.stress, &t.p);
                push(&mut out, None, "ə");
            }
            "oʊ" => push(&mut out, t.stress, r.goat),
            "eɪ" => push(&mut out, t.stress, r.face),
            "ɾ" if r.unflap => push(&mut out, None, flap_letter(src, &toks, i)),
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

/// Southern British form of one word of the American output.
fn to_gb(en: &English, ph: &str, src: &str, use_lex: bool) -> (String, bool) {
    if let Some(g) = gb_func().get(ph) {
        return (g.to_string(), ends_in_r(ph));
    }
    let src = expand_abbr(en, ph, src);
    if use_lex && !src.is_empty() && !crate::en::is_homograph(src) && en.is_default_form(src, ph) {
        if let Some(g) = gb_lex().get(src) {
            return (g.to_string(), ends_in_r(ph));
        }
        // an inflected form of a listed word keeps the listed stem (passes, classes, dancing)
        for suf in ["'s", "s", "es", "ed", "ing", "er", "ers", "ly"] {
            let Some(stem) = src.strip_suffix(suf) else { continue };
            if stem.len() < 3 {
                continue;
            }
            let mut cands = vec![stem.to_string()];
            if matches!(suf, "ed" | "ing" | "er" | "ers" | "es") {
                cands.push(format!("{stem}e"));
            }
            for c in cands {
                let (Some(g), Some(ga_stem)) = (gb_lex().get(c.as_str()), en.lexicon_form(&c)) else { continue };
                let Some(rest) = ph.strip_prefix(ga_stem) else { continue };
                if rest.is_empty() || rest.chars().any(|x| x == 'ˈ') {
                    continue;
                }
                let (tail, fr) = rules(rest, suf, GB);
                let link = if ends_in_r(ga_stem) && tail.chars().next().is_some_and(|x| VOWELS.contains(x)) { "ɹ" } else { "" };
                return (format!("{g}{link}{tail}"), fr || (rest.is_empty() && ends_in_r(ga_stem)));
            }
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
                && matches!(next2.as_str(), "s" | "t" | "d" | "p" | "tʃ")
                && ["anc", "ant", "and", "amp", "ans", "anch"].iter().any(|x| src.contains(x))
                && !src.contains("ar")
                && !matches!(src, "can't" | "cant" | "shan't" | "aunt" | "aunts") =>
            {
                toks[i].p = "æ".into();
            }
            // sure, poor, tour: THOUGHT; pure, cure keep the diphthong
            "ʊə" if i == 0 || toks[i - 1].p != "j" => toks[i].p = "ɔː".into(),
            // New Zealand: NEAR and SQUARE are one vowel
            "eə" if a == Accent::Nz => toks[i].p = "ɪə".into(),
            _ => {}
        }
    }
    // tune, dune: tj → tʃ, dj → dʒ
    render(&toks).replace("tj", "tʃ").replace("dj", "dʒ")
}

fn wh(ph: &str, src: &str) -> Option<String> {
    // which / witch are different words in Scotland and Ireland
    if src.starts_with("wh") && !src.starts_with("who") {
        if let Some(rest) = ph.strip_prefix('w') {
            return Some(format!("ʍ{rest}"));
        }
    }
    None
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

/// One word of the American output in the given accent (no linking r).
pub fn word(en: &English, ph: &str, src: &str, a: Accent) -> (String, bool) {
    match a {
        Accent::Gb | Accent::Za => to_gb(en, ph, src, true),
        Accent::Au | Accent::Nz => {
            let (g, fr) = to_gb(en, ph, src, true);
            (antipodean(&g, src, a), fr)
        }
        Accent::In => {
            let (g, fr) = to_gb(en, ph, src, true);
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
        Accent::Sc => {
            let src = expand_abbr(en, ph, src);
            let (s, _) = rules(ph, src, SC);
            let s = scottish(&s);
            (wh(&s, src).unwrap_or(s), false)
        }
        Accent::Ie => {
            let src = expand_abbr(en, ph, src);
            let (s, _) = rules(ph, src, IE);
            (wh(&s, src).unwrap_or(s), false)
        }
        Accent::Ca => (rules(ph, expand_abbr(en, ph, src), CA).0, false),
    }
}

/// Rewrite the finished American words of one sentence into the accent.
pub fn apply(en: &English, outs: &mut [Out], a: Accent) {
    let n = outs.len();
    let mut link = vec![false; n];
    for k in 0..n {
        if outs[k].ph.is_empty() {
            continue;
        }
        let glued = outs[k].ph.starts_with('\u{0}');
        let ph = outs[k].ph.trim_start_matches('\u{0}').to_string();
        let (new, fr) = word(en, &ph, &outs[k].src, a);
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
        for s in ["sˈɛkɹətˌɛɹi", "ɐbˌaʊt", "fˈaɪɚ", "tʃˈɜːtʃ", "nˈuː"] {
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
        assert_eq!(gb("wˈɑːtʃ", "watch"), "wˈɒtʃ");
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
    fn other_accents() {
        assert_eq!(antipodean("dˈɑːns", "dance", Accent::Au), "dˈæns");
        assert_eq!(antipodean("dˈɑːns", "dance", Accent::Nz), "dˈɑːns");
        assert_eq!(antipodean("ɡɹˈɑːs", "grass", Accent::Au), "ɡɹˈɑːs");
        assert_eq!(antipodean("ɹˈæbɪt", "rabbit", Accent::Au), "ɹˈæbət");
        assert_eq!(antipodean("tjˈuːn", "tune", Accent::Au), "tʃˈuːn");
        assert_eq!(antipodean("skwˈeə", "square", Accent::Nz), "skwˈɪə");
        assert_eq!(antipodean("ʃˈʊə", "sure", Accent::Au), "ʃˈɔː");
        assert_eq!(antipodean("pjˈʊə", "pure", Accent::Au), "pjˈʊə");
        assert_eq!(scottish(&rules("fˈʊt", "foot", SC).0), "fˈut");
        assert_eq!(scottish(&rules("hˈoʊm", "home", SC).0), "hˈoːm");
        assert_eq!(scottish(&rules("kˈɑːt", "cot", SC).0), "kˈɔt");
        assert_eq!(scottish(&rules("kˈɔːt", "caught", SC).0), "kˈɔt");
        assert_eq!(scottish(&rules("kˈɑːɹ", "car", SC).0), "kˈaɹ");
        assert_eq!(wh("wˌɪtʃ", "which").as_deref(), Some("ʍˌɪtʃ"));
        assert_eq!(wh("hˌuː", "who"), None);
        assert_eq!(rules("kˈɔːt", "caught", CA).0, "kˈɑːt");
        assert_eq!(rules("fˈɔːɹ", "four", CA).0, "fˈɔːɹ");
        assert_eq!(rules("dˈeɪ", "day", IE).0, "dˈeː");
    }
}
