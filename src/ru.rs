//! Russian G2P: Cyrillic -> narrow IPA in the 256-token TTS vocab notation.
//!
//! Exact port of Russian-G2P's `data/russian_g2p_rules.py` (one '+'-stress-marked
//! word -> IPA, Moscow-norm rules aimed at Wiktionary's ru-pron) and of the
//! sentence pipeline of `data/russian_phonemizer.py` (text normalization, numbers,
//! stress tiers, `data/text_vocab.normalize_text`). Output is byte-identical to the
//! Python reference; `src/bin/ru_check.rs` + `work/ru_parity.py` check that.
//!
//! RUAccent's two ONNX context models (homograph classifier and ё-tagger) are not
//! ported as such; their decisions are learned by a small native student
//! (`ru-ctx.g2p`, a character-level encoder run by `tagger.rs`, distilled with
//! `tools/ru_distill.py` + `tools/train_ru.py`) which [`Russian::load`] picks up
//! from the data directory (an int8 copy is built into the crate; `POLYG2P_RU_CTX=0`
//! turns it off) and consults at exactly the points where Python calls
//! the ONNX models ([`TaggerCtx`]). Other decisions can be injected through
//! [`ContextModels`] ([`Russian::phonemize_with`]). Without any model the
//! pipeline behaves as Python does when the model directories are missing
//! (corpus prior for homographs, the все/всё rule).
//! The legacy RUAccent+RUPhon engine for out-of-dictionary words is also absent
//! (as in Python when `ruaccent`/`ruphon` are not importable): the suffix-analogy
//! guess is used. Latin-script words go to an optional caller-supplied callback
//! ([`Russian::with_latin`]) instead of eSpeak; without one they are dropped.
//!
//! Lexicon files (all optional, in the directory given to [`Russian::load`]; build
//! them from `ru_lexicon/*.tsv.gz` with `work/ru_build_lex.py`):
//!
//! | file                    | Python tier (`ru_lexicon/`) | value                         |
//! |-------------------------|-----------------------------|-------------------------------|
//! | `ru_accents.lex`        | `accents.tsv.gz`            | stressed-vowel index          |
//! | `ru_yo.lex`             | `yo.tsv.gz`                 | ё-spelling                    |
//! | `ru_yo_homographs.lex`  | `yo_homographs.tsv.gz`      | ё-spelling (context decides)  |
//! | `ru_homographs.lex`     | `homographs.tsv.gz`         | `i1,i2,...` most likely first |
//! | `ru_homograph_ctx.lex`  | `homograph_ctx.tsv.gz`      | stress index for `p|prev|w` / `n|w|next` |
//! | `ru_suffix.lex`         | `suffix_stress.tsv.gz`      | stress position from the end  |
//! | `ru_train.lex`          | `train_words.tsv.gz`        | stressed-vowel index          |

use std::collections::HashMap;
use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use super::lexicon::Lexicon;
use super::tagger::{Logits, Tagger};
use std::cell::RefCell;

/// The context student shipped with the crate (int8, 1.7 MB; see `tools/train_ru.py`).
/// A `ru-ctx.g2p` in the data directory overrides it; `POLYG2P_RU_CTX=0` disables both.
const RU_CTX_BUILTIN: &[u8] = include_bytes!("../data/ru-ctx.g2p");

// ===========================================================================
// tables (generated from the Python modules; dict order kept where it matters)
// ===========================================================================

const MONO_STRESSED: &[&str] = &["а", "ко", "ли", "на", "но", "о", "по", "при", "то", "хоть"];
const CLITICS: &[(&str, &str)] = &[("же", "ʐɨ"), ("во", "vɐ"), ("нибудь", "nʲɪbʊtʲ")];
const EXCEPTIONS: &[(&str, &str)] = &[
    ("что", "ʂto"), ("чтобы", "ʂtˈobɨ"), ("чтоб", "ʂtop"), ("ничто", "nʲɪʂtˈo"),
    ("нечто", "nʲˈeʂtə"), ("конечно", "kɐnʲˈeʂnə"), ("скучно", "skˈuʂnə"), ("нарочно", "nɐrˈoʂnə"),
    ("какие", "kɐkʲˈijɪ"), ("скучный", "skˈuʂnɨj"), ("скучная", "skˈuʂnəjə"), ("скучное", "skˈuʂnəjə"),
    ("яичница", "jɪˈiʂnʲɪʦə"), ("яичницу", "jɪˈiʂnʲɪʦʊ"), ("яичницы", "jɪˈiʂnʲɪʦɨ"), ("скворечник", "skvɐrʲˈeʂnʲɪk"),
    ("разносчик", "rɐznˈoɕʧɪk"), ("сегодня", "sʲɪvˈodʲnʲə"), ("сегодняшний", "sʲɪvˈodʲnʲɪʂnʲɪj"), ("сегодняшнего", "sʲɪvˈodʲnʲɪʂnʲɪvə"),
    ("сегодняшняя", "sʲɪvˈodʲnʲɪʂnʲɪjə"), ("сегодняшнее", "sʲɪvˈodʲnʲɪʂnʲɪjə"), ("сегодняшнем", "sʲɪvˈodʲnʲɪʂnʲɪm"), ("итого", "ɪtɐvˈo"),
    ("бог", "box"), ("господь", "ɡɐspˈotʲ"), ("господи", "ɡˈospədʲɪ"), ("пожалуйста", "pɐʐˈaɫstə"),
    ("лестница", "lʲˈesʲnʲɪʦə"), ("лестницы", "lʲˈesʲnʲɪʦɨ"), ("лестнице", "lʲˈesʲnʲɪʦɨ"), ("лестницу", "lʲˈesʲnʲɪʦʊ"),
    ("кафе", "kɐfˈɛ"), ("шоссе", "ʂɐsːˈɛ"), ("пюре", "pʲʊrˈɛ"), ("радио", "rˈadʲɪo"),
    ("шестьсот", "ʂɨsːˈot"), ("счёл", "ɕʧˈɵɫ"), ("кашне", "kɐʂnˈɛ"), ("резюме", "rʲɪzʲʉmˈɛ"),
];
const HARD_STEMS: &[(&str, &str)] = &[
    ("тест", "Tест"), ("отел", "оTел"), ("модел", "моDел"), ("компьютер", "компьюTер"),
    ("интернет", "инTерNет"), ("теннис", "Tенис"), ("темп", "Tемп"), ("проект", "проЭкт"),
    ("энерги", "ƐNерги"), ("фонетик", "фоNетик"), ("генетик", "геNетик"), ("свитер", "свиTер"),
    ("бутерброд", "буTерброд"), ("детектив", "деTекTив"), ("шедевр", "шеDевр"), ("тенденци", "TенDенци"),
    ("фортепиано", "форTепиано"), ("стенд", "сTенд"), ("шрапнел", "шрапNел"), ("модерн", "моDерн"),
    ("бизнес", "бизNес"), ("денди", "Dенди"), ("тенор", "Tенор"), ("эстет", "эсTет"),
    ("синтез", "синTез"), ("ателье", "аTелье"), ("протез", "проTез"),
];
const OGO_KEEP_G: &[&str] = &[
    "благо", "дорого", "итого", "много", "недорого", "немного", "нестрого", "ого",
    "отлого", "пирого", "полого", "премного", "разлого", "строго", "убого",
];
const OE_FULL: &[&str] = &["двое", "кое", "пятеро", "трое"];
// sorted(_GEM_PREFIXES, key=len, reverse=True): stable, so equal lengths keep tuple order
const GEM_PREFIXES: &[&str] = &[
    "пред", "под", "над", "без", "бес", "раз", "рас", "воз", "вос", "от", "из", "ис", "вз", "вс", "об", "в", "с",
];
const CLUSTERS: &[(&str, &str)] = &[("стн", "сн"), ("здн", "зн"), ("стл", "Ŝл"), ("лнц", "нц"), ("рдц", "рц"), ("рдч", "рч")];
const AFFRICATE_SUBS: &[(&str, &str)] = &[
    ("сч", "Щ"), ("зч", "Щ"), ("жч", "Щ"), ("тч", "Ч"), ("дч", "Ч"), ("тц", "Ц"), ("дц", "Ц"), ("дж", "Д"),
    ("тш", "чш"), ("дш", "чш"), ("тщ", "чщ"), ("дщ", "чщ"), ("сж", "Ж"), ("зж", "Ж"), ("жж", "Ж"),
];
const SOFT_ENDINGS: &[&str] = &["", "м", "х", "ми", "т", "ця", "Ця"];

const STRESS: &str = "\u{2c8}"; // ˈ
// phone vowels
const A: &str = "a";
const E: &str = "e";
const I: &str = "i";
const O: &str = "o";
const U: &str = "u";
const Y: &str = "\u{268}"; // ɨ
const II: &str = "\u{26a}"; // ɪ
const SCHWA: &str = "\u{259}"; // ə
const AA: &str = "\u{250}"; // ɐ
const UU: &str = "\u{28a}"; // ʊ
const AE: &str = "\u{e6}"; // æ
const OE: &str = "\u{275}"; // ɵ
const UE: &str = "\u{289}"; // ʉ
const EH: &str = "\u{25b}"; // ɛ

fn base(c: char) -> Option<&'static str> {
    Some(match c {
        'б' => "b",
        'в' => "v",
        'г' => "ɡ",
        'д' => "d",
        'ж' => "ʐ",
        'з' => "z",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'ф' => "f",
        'х' => "x",
        'ц' => "ʦ",
        'ч' => "ʧ",
        'ш' => "ʂ",
        'щ' => "ɕː",
        'й' => "j",
        // placeholders written by rewrite()
        'Ч' => "ʧː",
        'Ц' => "ʦː",
        'Д' => "ʤʐ",
        'Щ' => "ɕː",
        'Ж' => "ʐː",
        'Ш' => "ʂː",
        'Ƭ' => "ʦ",
        'Ǳ' => "ʣ",
        'Ŝ' => "s",
        'Ç' => "ʦ",
        'Ś' => "ʂ",
        'ɕ' => "ɕ",
        // hard-consonant placeholders (HARD_STEMS)
        'T' => "t",
        'D' => "d",
        'N' => "n",
        'S' => "s",
        'Z' => "z",
        'R' => "r",
        'L' => "ɫ",
        'M' => "m",
        'F' => "f",
        'P' => "p",
        'B' => "b",
        'V' => "v",
        _ => return None,
    })
}
fn is_obstruent(p: &str) -> bool {
    matches!(p, "b" | "d" | "f" | "k" | "p" | "s" | "t" | "v" | "x" | "z" | "ɕː" | "ɡ" | "ʂ" | "ʂː" | "ʐ" | "ʐː" | "ʣ" | "ʤʐ" | "ʦ" | "ʦː" | "ʧ" | "ʧʂ" | "ʧː")
}
fn is_voicing_trigger(p: &str) -> bool {
    matches!(p, "b" | "d" | "z" | "ɡ" | "ʐ" | "ʐː" | "ʣ" | "ʤʐ")
}
fn no_pal_mark(p: &str) -> bool {
    matches!(p, "j" | "ɕ" | "ɕː" | "ʂ" | "ʂː" | "ʐ" | "ʐː" | "ʤʐ" | "ʦ" | "ʦː" | "ʧ" | "ʧʂ" | "ʧː")
}
fn devoice(p: &'static str) -> &'static str {
    match p {
        "b" => "p",
        "d" => "t",
        "ɡ" => "k",
        "z" => "s",
        "ʐ" => "ʂ",
        "v" => "f",
        "ʤʐ" => "ʧʂ",
        "ʐː" => "ʂː",
        "ʣ" => "ʦ",
        _ => p,
    }
}
fn voice(p: &'static str) -> &'static str {
    match p {
        "p" => "b",
        "t" => "d",
        "k" => "ɡ",
        "s" => "z",
        "ʂ" => "ʐ",
        "f" => "v",
        "ʧ" => "ʤ",
        "ʦ" => "ʣ",
        _ => p,
    }
}
fn is_vowel(c: char) -> bool {
    matches!(c, 'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' | 'Э' | 'Ɛ')
}
fn is_iotated(c: char) -> bool {
    matches!(c, 'е' | 'ё' | 'ю' | 'я')
}
fn is_soft_sign(c: char) -> bool {
    matches!(c, 'е' | 'ё' | 'и' | 'ю' | 'я' | 'ь')
}
fn is_hard_only(c: char) -> bool {
    matches!(c, 'B' | 'D' | 'F' | 'L' | 'M' | 'N' | 'P' | 'R' | 'S' | 'T' | 'V' | 'Z' | 'Ç' | 'Ś' | 'Ŝ' | 'Д' | 'Ж' | 'Ц' | 'Ш' | 'ж' | 'ц' | 'ш')
}
fn is_palatal(c: char) -> bool {
    matches!(c, 'ɕ' | 'Ч' | 'Щ' | 'й' | 'ч' | 'щ')
}
fn is_hush(c: char) -> bool {
    matches!(c, 'Ç' | 'Ś' | 'Д' | 'Ж' | 'Ц' | 'Ш' | 'ж' | 'ц' | 'ш')
}
fn is_phone_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ɨ' | 'ɪ' | 'ə' | 'ɐ' | 'ʊ' | 'æ' | 'ɵ' | 'ʉ' | 'ɛ')
}
/// CYRILLIC_LETTERS: а..я and ё
fn is_cyr_letter(c: char) -> bool {
    matches!(c, 'а'..='я' | 'ё')
}

fn lookup(table: &[(&str, &'static str)], k: &str) -> Option<&'static str> {
    table.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
}

// ===========================================================================
// char-slice helpers (Python str semantics on code points)
// ===========================================================================

fn match_at(s: &[char], i: usize, pat: &str) -> bool {
    for (k, pc) in (i..).zip(pat.chars()) {
        if k >= s.len() || s[k] != pc {
            return false;
        }
    }
    true
}
fn find(s: &[char], pat: &str, from: usize) -> Option<usize> {
    let first = pat.chars().next()?;
    (from..s.len()).find(|&i| s[i] == first && match_at(s, i, pat))
}
fn contains(s: &[char], pat: &str) -> bool {
    find(s, pat, 0).is_some()
}
fn ends_with(s: &[char], pat: &str) -> bool {
    let n = pat.chars().count();
    n <= s.len() && match_at(s, s.len() - n, pat)
}
fn eq_str(s: &[char], pat: &str) -> bool {
    s.len() == pat.chars().count() && match_at(s, 0, pat)
}
fn tail_eq(s: &[char], from: usize, pat: &str) -> bool {
    from <= s.len() && eq_str(&s[from..], pat)
}

// ===========================================================================
// word -> IPA
// ===========================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    C,
    V,
    J,
}

#[derive(Clone, Copy)]
struct Seg {
    kind: Kind,
    p: &'static str,
    soft: bool,
    stressed: bool,
    jglide: bool,
    letter: char,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Left {
    Initial,
    Vowel,
    Hush,
    Hard,
    Soft,
    J,
}

#[derive(Clone, Copy)]
struct Tok {
    p: &'static str,
    pal: bool,
}

impl Tok {
    fn is(&self, s: &str) -> bool {
        !self.pal && self.p == s
    }
    fn push_to(&self, out: &mut String) {
        out.push_str(self.p);
        if self.pal {
            out.push('ʲ');
        }
    }
}

/// Index (among vowels) of the vowel carrying ˈ in an IPA word, or None.
fn stress_vowel_number(ipa: &str) -> Option<usize> {
    let mut v = 0;
    let mut marked = false;
    for ch in ipa.chars() {
        if ch == 'ˈ' {
            marked = true;
            continue;
        }
        if is_phone_vowel(ch) {
            if marked {
                return Some(v);
            }
            v += 1;
        }
    }
    None
}

/// One Russian word -> IPA (port of `word_to_ipa`).
///
/// `word` is Cyrillic with '+' before the stressed vowel (RUAccent convention); ё
/// implies its own stress. Other characters are dropped. `mark_mono` forces
/// (Some(true)) or suppresses (Some(false)) ˈ on a monosyllable; None applies the
/// label convention.
pub fn word_to_ipa(word: &str, mark_mono: Option<bool>) -> String {
    // word.lower().replace("́", "") + split_stress
    let mut low: Vec<char> = Vec::with_capacity(word.len());
    let mut plus: Option<usize> = None;
    for lc in word.chars().flat_map(char::to_lowercase) {
        if lc == '\u{301}' {
            continue;
        }
        if lc == '+' {
            plus = Some(low.len());
            continue;
        }
        low.push(lc);
    }
    let idx = plus.and_then(|p| (p..low.len()).find(|&j| is_vowel(low[j])));
    let mut w: Vec<char> = Vec::with_capacity(low.len());
    let mut stress: Option<usize> = None;
    for (i, &c) in low.iter().enumerate() {
        if is_cyr_letter(c) {
            if idx == Some(i) {
                stress = Some(w.len());
            }
            w.push(c);
        }
    }
    if w.is_empty() {
        return String::new();
    }
    let vpos: Vec<usize> = (0..w.len()).filter(|&i| is_vowel(w[i])).collect();
    if stress.is_none_or(|st| !vpos.contains(&st)) {
        stress = match w.iter().rposition(|&c| c == 'ё') {
            Some(p) => Some(p),
            None if vpos.len() == 1 => Some(vpos[0]),
            None => None,
        };
    }
    let nvow = vpos.len();
    let wstr: String = w.iter().collect();
    let mono_mark = mark_mono.unwrap_or_else(|| MONO_STRESSED.contains(&wstr.as_str()) || wstr == "всё");

    if nvow == 0 {
        if wstr == "ж" {
            return "ʂ".to_string();
        }
        return w.iter().filter_map(|&c| base(c)).collect();
    }
    if let Some(c) = lookup(CLITICS, &wstr)
        && !(nvow == 1 && mono_mark)
    {
        return c.to_string();
    }
    if let Some(exc) = lookup(EXCEPTIONS, &wstr) {
        let k = stress.and_then(|st| vpos.iter().position(|&v| v == st));
        if nvow <= 1 || k.is_none() || stress_vowel_number(exc) == k {
            return if nvow > 1 || mono_mark { exc.to_string() } else { exc.replace('ˈ', "") };
        }
    }

    let (s, stress, prefix_len) = rewrite(&w, &wstr, stress);
    let soft = soft_flags(&s);
    let mut segs = segments(&s, stress, &soft);
    voicing(&mut segs);
    let ipa = render(&segs, prefix_len);
    if nvow <= 1 && !mono_mark {
        return ipa.replace('ˈ', "");
    }
    ipa
}

/// Replace src->dst everywhere (or only at the start), keeping `stress` on the same vowel.
fn sub_tracking(s: &mut Vec<char>, stress: &mut Option<usize>, src: &str, dst: &str, anchored: bool) {
    let Some(mut pos) = find(s, src, 0) else { return };
    let src_c: Vec<char> = src.chars().collect();
    let dst_c: Vec<char> = dst.chars().collect();
    loop {
        if anchored && pos != 0 {
            break;
        }
        s.splice(pos..pos + src_c.len(), dst_c.iter().copied());
        if let Some(st) = *stress {
            if st >= pos + src_c.len() {
                *stress = Some(st + dst_c.len() - src_c.len());
            } else if st >= pos {
                let nb = src_c[..st - pos].iter().filter(|&&c| is_vowel(c)).count();
                let vi: Vec<usize> = (0..dst_c.len()).filter(|&i| is_vowel(dst_c[i])).collect();
                *stress = Some(pos + if nb < vi.len() { vi[nb] } else { st - pos });
            }
        }
        match find(s, src, pos + dst_c.len()) {
            Some(p) => pos = p,
            None => break,
        }
    }
}

/// Orthographic rewrites that the phone rules then treat as ordinary letters.
fn rewrite(w: &[char], wstr: &str, mut stress: Option<usize>) -> (Vec<char>, Option<usize>, usize) {
    let mut s: Vec<char> = w.to_vec();
    for (stem, rep) in HARD_STEMS {
        if match_at(&s, 0, stem) {
            sub_tracking(&mut s, &mut stress, stem, rep, true);
            break;
        }
    }
    if contains(&s, "чувств") || match_at(&s, 0, "здравств") {
        sub_tracking(&mut s, &mut stress, "вств", "ств", false);
    }
    for (src, dst) in CLUSTERS {
        sub_tracking(&mut s, &mut stress, src, dst, false);
    }
    // -ого / -его (and participles in -ого/-егося): г -> в
    let n = s.len();
    if n > 3 && (ends_with(&s, "ого") || ends_with(&s, "его")) && !OGO_KEEP_G.contains(&wstr) {
        s[n - 2] = 'в';
    } else if n > 6 && (ends_with(&s, "огося") || ends_with(&s, "егося")) {
        s[n - 4] = 'в';
    } else if ["его", "кого", "того", "чего", "сего"].iter().any(|x| eq_str(&s, x)) {
        s[n - 2] = 'в';
    }
    // -ться / -тся: ʦː right after the stressed vowel, ʦ elsewhere (stress index is not adjusted, as in Python)
    for (end, el) in [("ться", 4usize), ("тся", 3usize)] {
        let n = s.len();
        if ends_with(&s, end) && n > el {
            let after_stress = stress == Some(n - el - 1);
            s.truncate(n - el);
            s.push(if after_stress { 'Ц' } else if el == 4 { 'Ç' } else { 'ц' });
            s.push('я');
            break;
        }
    }
    for (src, dst) in AFFRICATE_SUBS {
        sub_tracking(&mut s, &mut stress, src, dst, false);
    }
    for src in ["сш", "зш"] {
        sub_tracking(&mut s, &mut stress, src, "Śш", false);
    }
    // m = re.search(r"(?<!н)[тд]с(?=к)", s); replace every occurrence of m.group(0)
    loop {
        let m = (0..s.len()).find(|&i| {
            matches!(s[i], 'т' | 'д') && s.get(i + 1) == Some(&'с') && s.get(i + 2) == Some(&'к') && (i == 0 || s[i - 1] != 'н')
        });
        let Some(i) = m else { break };
        let src: String = [s[i], 'с'].iter().collect();
        sub_tracking(&mut s, &mut stress, &src, "Ƭ", false);
    }
    sub_tracking(&mut s, &mut stress, "тьс", "Ƭ", false);
    if !match_at(&s, 0, "отст") {
        for src in ["тст", "дст"] {
            sub_tracking(&mut s, &mut stress, src, "Ƭт", false);
        }
    }
    // re.sub(r"(?<!н)[тд](?=с)|(?<=н)[тд](?=с[^к])", "Ƭ", s): 1:1, lookarounds never see a replaced char
    for i in 0..s.len() {
        if matches!(s[i], 'т' | 'д') && s.get(i + 1) == Some(&'с') {
            let after_n = i > 0 && s[i - 1] == 'н';
            if !after_n || s.get(i + 2).is_some_and(|&c| c != 'к') {
                s[i] = 'Ƭ';
            }
        }
    }
    // re.sub(r"[тд](?=з)", "Ǳ", s)
    for i in 0..s.len() {
        if matches!(s[i], 'т' | 'д') && s.get(i + 1) == Some(&'з') {
            s[i] = 'Ǳ';
        }
    }
    let mut prefix_len = 0;
    for p in GEM_PREFIXES {
        let pl = p.chars().count();
        if match_at(w, 0, p) && w.len() > pl + 1 && !is_vowel(w[pl]) && w[pl] != 'ь' && w[pl] != 'ъ' {
            prefix_len = pl;
            break;
        }
    }
    (s, stress, prefix_len)
}

/// `_is_soft` for every position (it only looks to the right, so one right-to-left pass).
fn soft_flags(s: &[char]) -> Vec<bool> {
    let n = s.len();
    let mut soft = vec![false; n];
    for j in (0..n).rev() {
        let c = s[j];
        if is_hard_only(c) || is_palatal(c) {
            continue;
        }
        let Some(b) = base(c) else { continue };
        let res = match s.get(j + 1).copied() {
            Some(x) if is_soft_sign(x) => true,
            Some(x) if base(x).is_some() && !is_palatal(x) && !is_hard_only(x) => {
                let nb = base(x).unwrap_or("");
                soft[j + 1]
                    && ((matches!(b, "s" | "z") && matches!(nb, "t" | "d" | "s" | "z" | "l"))
                        || (b == "z" && nb == "n")
                        || (matches!(b, "t" | "d" | "n" | "ʦ") && matches!(nb, "t" | "d" | "s" | "z" | "n"))
                        || (matches!(b, "v" | "f" | "m" | "p" | "b") && matches!(nb, "p" | "b"))
                        || (matches!(b, "ɡ" | "k" | "x") && matches!(nb, "k" | "ɡ")))
            }
            Some('ч' | 'щ' | 'Ч' | 'Щ') => b == "n",
            _ => false,
        };
        soft[j] = res;
    }
    soft
}

fn right_soft(s: &[char], i: usize, soft: &[bool]) -> bool {
    let j = i + 1;
    if j >= s.len() {
        return false;
    }
    let c = s[j];
    if is_vowel(c) {
        return is_iotated(c);
    }
    if is_palatal(c) {
        return true;
    }
    if base(c).is_some() {
        return soft[j];
    }
    false
}

fn segments(s: &[char], stress: Option<usize>, soft: &[bool]) -> Vec<Seg> {
    let nv = s.iter().filter(|&&c| is_vowel(c)).count();
    let sk = stress.and_then(|st| if st < s.len() && is_vowel(s[st]) { Some(s[..st].iter().filter(|&&c| is_vowel(c)).count()) } else { None });
    debug_assert!(sk.is_none_or(|k| k < nv));
    let mut segs: Vec<Seg> = Vec::with_capacity(s.len() + 4);
    let mut k = 0usize;
    for (i, &c) in s.iter().enumerate() {
        let prv = if i > 0 { Some(s[i - 1]) } else { None };
        if let Some(b) = base(c) {
            if c == 'й' {
                segs.push(Seg { kind: Kind::C, p: "j", soft: false, stressed: false, jglide: false, letter: c });
                continue;
            }
            let sf = soft[i];
            let p = if b == "l" && !sf { "ɫ" } else { b };
            segs.push(Seg { kind: Kind::C, p, soft: sf, stressed: false, jglide: false, letter: c });
            continue;
        }
        if !is_vowel(c) {
            continue; // ь ъ
        }
        let kk = k;
        k += 1;
        let stressed = sk == Some(kk);
        let rel = sk.map(|sk| kk as isize - sk as isize);
        let left = match prv {
            None => Left::Initial,
            Some(p) if is_vowel(p) || p == 'ь' || p == 'ъ' => Left::Vowel,
            Some(p) if is_hush(p) => Left::Hush,
            Some(p) if is_hard_only(p) => Left::Hard,
            Some(p) if is_palatal(p) && p != 'й' => Left::Soft,
            Some('й') => Left::J,
            Some(_) => {
                if soft[i - 1] {
                    Left::Soft
                } else {
                    Left::Hard
                }
            }
        };
        let mut jglide = false;
        let mut cq = c;
        if (is_iotated(c) && matches!(left, Left::Initial | Left::Vowel)) || ((c == 'и' || c == 'о') && prv == Some('ь')) {
            segs.push(Seg { kind: Kind::J, p: "", soft: false, stressed: false, jglide: false, letter: ' ' });
            jglide = true;
            if c == 'о' {
                cq = 'ё'; // ьо = /jo/ -> jɵ (синьор, бульон)
            }
        }
        let p = vowel(cq, stressed, rel, if jglide { Left::J } else { left }, left, s, i, soft);
        segs.push(Seg { kind: Kind::V, p, soft: false, stressed, jglide: jglide || prv == Some('й'), letter: c });
    }
    segs
}

#[allow(clippy::too_many_arguments)]
fn vowel(c0: char, stressed: bool, rel: Option<isize>, left: Left, left_raw: Left, s: &[char], i: usize, soft: &[bool]) -> &'static str {
    if c0 == 'Ɛ' {
        return EH; // энергия ɛnˈɛrɡʲɪjə
    }
    let soft_left = matches!(left, Left::Soft | Left::J);
    let nsoft = right_soft(s, i, soft);
    let fin = i == s.len() - 1;
    let c = if c0 == 'Э' { 'э' } else { c0 };
    if stressed {
        return match c {
            'а' | 'я' => {
                if soft_left && nsoft {
                    AE
                } else {
                    A
                }
            }
            'о' | 'ё' => {
                if matches!(left, Left::Hush | Left::Hard) {
                    O
                } else if c == 'ё' || left == Left::Soft {
                    OE
                } else {
                    O
                }
            }
            'у' | 'ю' => {
                if soft_left && nsoft {
                    UE
                } else {
                    U
                }
            }
            'е' => {
                if matches!(left, Left::Hush | Left::Hard) {
                    EH
                } else {
                    E
                }
            }
            'э' => EH,
            'и' => {
                if left == Left::Hush {
                    Y
                } else {
                    I
                }
            }
            _ => Y, // ы
        };
    }
    let pre = rel.is_some_and(|r| r < 0);
    let pretonic1 = rel == Some(-1);
    match c {
        'ы' => return Y,
        'и' => return if left == Left::Hush { Y } else { II },
        'у' | 'ю' => return if soft_left && nsoft { UE } else { UU },
        'э' => {
            return if matches!(left_raw, Left::Initial | Left::Vowel) {
                II
            } else if pre {
                Y
            } else {
                SCHWA
            };
        }
        _ => {}
    }
    if c == 'а' || c == 'о' {
        if left == Left::Soft {
            // after ч щ
            let soft_end = SOFT_ENDINGS.iter().any(|e| tail_eq(s, i + 1, e));
            return if pre || !soft_end { II } else { SCHWA };
        }
        if left == Left::Hush
            && c == 'а'
            && nsoft
            && ((pretonic1 && (contains(s, "жал") || contains(s, "лошад"))) || (matches!(s[i - 1], 'ц' | 'Ц') && match_at(s, i + 1, "ть")))
        {
            return Y;
        }
        if rel.is_none() {
            return if left_raw == Left::Initial { AA } else { SCHWA };
        }
        if pre && (pretonic1 || left_raw == Left::Initial) {
            return AA;
        }
        if pre && (matches!(s.get(i + 1), Some('а' | 'о')) || (i > 0 && matches!(s[i - 1], 'а' | 'о'))) {
            return AA;
        }
        return SCHWA;
    }
    if matches!(c, 'я' | 'е' | 'ё') {
        if left == Left::Hush {
            if c == 'е' && fin && matches!(s[i - 1], 'ц' | 'Ц') && !ends_with(s, "ице") {
                return SCHWA;
            }
            return if c == 'е' || c == 'ё' {
                Y
            } else if pre {
                AA
            } else {
                SCHWA
            };
        }
        if left == Left::Hard {
            return if c == 'е' || pre { Y } else { SCHWA };
        }
        if pre {
            return II;
        }
        if c == 'е' && fin {
            if i > 0 && s[i - 1] == 'о' && !OE_FULL.iter().any(|x| eq_str(s, x)) {
                return SCHWA; // adjectival -ое (такое tɐkˈojə)
            }
            return E; // -ие/-ые/-ье/-ее
        }
        if c == 'я' && SOFT_ENDINGS.iter().any(|e| tail_eq(s, i + 1, e)) && !ends_with(s, "десят") {
            return SCHWA;
        }
        return II;
    }
    SCHWA
}

/// Right-to-left: word-final devoicing and regressive voicing assimilation.
fn voicing(segs: &mut [Seg]) {
    let mut following: Option<(Kind, &'static str)> = None;
    for sg in segs.iter_mut().rev() {
        if sg.kind == Kind::C && is_obstruent(sg.p) {
            match following {
                None => sg.p = devoice(sg.p),
                Some((Kind::C, fp)) if is_obstruent(fp) => {
                    if is_voicing_trigger(fp) {
                        if !matches!(sg.p, "ʦ" | "ʧ" | "x") {
                            sg.p = voice(sg.p);
                        }
                    } else if fp != "v" {
                        sg.p = devoice(sg.p);
                        if sg.letter == 'г' && matches!(fp, "k" | "ʧ" | "ʧː") {
                            sg.p = "x";
                        }
                    }
                }
                _ => {}
            }
        }
        following = Some((sg.kind, sg.p));
    }
}

fn render(segs: &[Seg], prefix_len: usize) -> String {
    let mut toks: Vec<Tok> = Vec::with_capacity(segs.len() + 2);
    for sg in segs {
        match sg.kind {
            Kind::J => toks.push(Tok { p: "j", pal: false }),
            Kind::V => {
                if sg.stressed {
                    if sg.jglide && toks.last().is_some_and(|t| t.is("j")) {
                        let n = toks.len();
                        toks.insert(n - 1, Tok { p: STRESS, pal: false });
                    } else {
                        toks.push(Tok { p: STRESS, pal: false });
                    }
                }
                toks.push(Tok { p: sg.p, pal: false });
            }
            Kind::C => {
                let pal = sg.soft && (!no_pal_mark(sg.p) || sg.letter == 'Ƭ');
                toks.push(Tok { p: sg.p, pal });
            }
        }
    }
    geminate(&toks, segs, prefix_len)
}

/// `_cons_key` is non-empty: the token is not ˈ and does not start with a vowel.
fn has_cons_key(t: &Tok) -> bool {
    t.p != STRESS && t.p.chars().next().is_some_and(|c| !is_phone_vowel(c))
}

/// `_cons_key(a) == _cons_key(b)` for two tokens with a non-empty key: the consonant
/// without ʲ ː, with ɫ read as l.
fn cons_key_eq(a: &Tok, b: &Tok) -> bool {
    let key = |t: &Tok| t.p.chars().filter(|&c| c != 'ʲ' && c != 'ː').map(|c| if c == 'ɫ' { 'l' } else { c });
    key(a).eq(key(b))
}

/// Collapse two identical consonants into one, long or short.
fn geminate(toks: &[Tok], segs: &[Seg], prefix_len: usize) -> String {
    let letters: Vec<usize> = (0..segs.len()).filter(|&k| segs[k].kind == Kind::C).collect();
    // number of non-J segments in segs[..=k]
    let mut nonj = Vec::with_capacity(segs.len());
    let mut acc = 0usize;
    for sg in segs {
        if sg.kind != Kind::J {
            acc += 1;
        }
        nonj.push(acc);
    }
    let mut res = String::with_capacity(toks.len() * 3);
    let mut res_empty = true;
    let mut ci: isize = -1;
    let mut stressed_seen = false;
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i];
        if t.p == STRESS {
            stressed_seen = true;
            res.push_str(STRESS);
            res_empty = false;
            i += 1;
            continue;
        }
        let is_cons = has_cons_key(&t) && !t.is("j");
        if is_cons {
            ci += 1;
        }
        if is_cons && i + 1 < toks.len() {
            let nx = toks[i + 1];
            let same = has_cons_key(&nx) && !nx.is("j") && cons_key_eq(&nx, &t);
            if same {
                let cu = ci as usize;
                let a = &segs[letters[cu]];
                let b = &segs[letters[cu + 1]];
                let after = toks.get(i + 2);
                let before_cons = after.is_some_and(|x| x.p != STRESS && !x.p.chars().next().is_some_and(is_phone_vowel));
                let a_idx = letters[cu];
                let at_prefix = prefix_len != 0 && nonj[a_idx] == prefix_len;
                let long = if after.is_none() {
                    false
                } else if at_prefix {
                    true
                } else if before_cons {
                    false
                } else if a.letter != b.letter {
                    stressed_seen || cu == 0 || matches!(a.letter, 'Ж' | 'Ч' | 'Ц' | 'Щ')
                } else if a.letter == 'н' || (a.letter == 'м' && stressed_seen) {
                    true
                } else {
                    cu == 0 && res_empty // word-initial сс-, жж-
                };
                nx.push_to(&mut res);
                if long && !nx.p.contains('ː') {
                    res.push('ː');
                }
                res_empty = false;
                ci += 1;
                i += 2;
                continue;
            }
        }
        t.push_to(&mut res);
        res_empty = false;
        i += 1;
    }
    res
}

// ===========================================================================
// Python `re`/`str` character classes
// ===========================================================================

/// `str.isspace()` / `re` `\s`.
fn py_isspace(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{d}' | '\u{1c}'..='\u{20}' | '\u{85}' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}')
}
fn py_strip(s: &str) -> &str {
    s.trim_matches(py_isspace)
}
/// `re` `\d` (Unicode Nd) -> digit value.
fn py_digit(c: char) -> Option<u8> {
    let cp = c as u32;
    if c.is_ascii_digit() {
        return Some((cp - 0x30) as u8);
    }
    if cp < 0x660 {
        return None;
    }
    let k = PY_DIGIT_ZEROS.partition_point(|&z| z <= cp);
    if k == 0 {
        return None;
    }
    let z = PY_DIGIT_ZEROS[k - 1];
    if cp < z + 10 { Some((cp - z) as u8) } else { None }
}
/// `re` `\w`.
fn py_word(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphanumeric() || c == '_';
    }
    let cp = c as u32;
    let k = PY_WORD.partition_point(|&(a, _)| a <= cp);
    k > 0 && cp <= PY_WORD[k - 1].1
}

// ===========================================================================
// numbers (russian_phonemizer._expand_numbers)
// ===========================================================================

const UNITS: &[&str] = &["ноль", "один", "два", "три", "четыре", "пять", "шесть", "семь", "восемь", "девять"];
const TEENS: &[&str] = &[
    "десять", "одиннадцать", "двенадцать", "тринадцать", "четырнадцать", "пятнадцать", "шестнадцать", "семнадцать",
    "восемнадцать", "девятнадцать",
];
const TENS: &[&str] = &["", "", "двадцать", "тридцать", "сорок", "пятьдесят", "шестьдесят", "семьдесят", "восемьдесят", "девяносто"];
const HUNDREDS: &[&str] = &["", "сто", "двести", "триста", "четыреста", "пятьсот", "шестьсот", "семьсот", "восемьсот", "девятьсот"];
const SCALES: &[(&str, &str, &str, bool)] =
    &[("тысяча", "тысячи", "тысяч", true), ("миллион", "миллиона", "миллионов", false), ("миллиард", "миллиарда", "миллиардов", false)];
const ORD: &[(&str, &str)] = &[
    ("ноль", "нулев"), ("один", "перв"), ("два", "втор"), ("три", "трет"),
    ("четыре", "четвёрт"), ("пять", "пят"), ("шесть", "шест"), ("семь", "седьм"),
    ("восемь", "восьм"), ("девять", "девят"), ("десять", "десят"), ("одиннадцать", "одиннадцат"),
    ("двенадцать", "двенадцат"), ("тринадцать", "тринадцат"), ("четырнадцать", "четырнадцат"), ("пятнадцать", "пятнадцат"),
    ("шестнадцать", "шестнадцат"), ("семнадцать", "семнадцат"), ("восемнадцать", "восемнадцат"), ("девятнадцать", "девятнадцат"),
    ("двадцать", "двадцат"), ("тридцать", "тридцат"), ("сорок", "сороков"), ("пятьдесят", "пятидесят"),
    ("шестьдесят", "шестидесят"), ("семьдесят", "семидесят"), ("восемьдесят", "восьмидесят"), ("девяносто", "девяност"),
    ("сто", "сот"), ("двести", "двухсот"), ("триста", "трёхсот"), ("четыреста", "четырёхсот"),
    ("пятьсот", "пятисот"), ("шестьсот", "шестисот"), ("семьсот", "семисот"), ("восемьсот", "восьмисот"),
    ("девятьсот", "девятисот"), ("тысяча", "тысячн"), ("миллион", "миллионн"),
];
const GOD_CASE: &[(&str, &str)] = &[
    ("год", "nom"), ("года", "gen"), ("году", "prep"), ("годом", "ins"),
    ("годов", "gen"), ("годах", "prep"), ("годам", "dat"), ("годами", "ins"),
];
const MONTHS_GEN: &[&str] =
    &["августа", "апреля", "декабря", "июля", "июня", "марта", "мая", "ноября", "октября", "сентября", "февраля", "января"];
const SIGNS: &[(char, &str)] = &[
    ('$', "долларов"), ('€', "евро"), ('₽', "рублей"), ('£', "фунтов"), ('₪', "шекелей"),
    ('№', "номер"), ('&', "и"), ('+', "плюс"), ('=', "равно"), ('°', "градусов"),
];

fn ord_end(case: &str, kind: u8) -> &'static str {
    // kind 0: _ORD_END_HARD, 1: _ORD_END_STRESSED (nom=ой), 2: _ORD_END_SOFT
    match (kind, case) {
        (2, "nom") => "ий",
        (2, "gen") => "ьего",
        (2, "dat") => "ьему",
        (2, "prep") => "ьем",
        (2, "ins") => "ьим",
        (2, _) => "ье",
        (1, "nom") => "ой",
        (_, "nom") => "ый",
        (_, "gen") => "ого",
        (_, "dat") => "ому",
        (_, "prep") => "ом",
        (_, "ins") => "ым",
        _ => "ое",
    }
}

/// An integer as Python's `int()` of a digit string.
struct Num {
    digits: Vec<u8>,
    /// the value when below 10^12 (the range `_number_to_words` handles)
    small: Option<u64>,
    m100: u32,
}

impl Num {
    fn new(digits: Vec<u8>) -> Num {
        let sig: &[u8] = {
            let k = digits.iter().position(|&d| d != 0).unwrap_or(digits.len());
            &digits[k..]
        };
        let small = if sig.len() <= 12 { Some(sig.iter().fold(0u64, |a, &d| a * 10 + d as u64)) } else { None };
        let n = digits.len();
        let m100 = digits[n.saturating_sub(2)..].iter().fold(0u32, |a, &d| a * 10 + d as u32);
        Num { digits, small, m100 }
    }
    fn in_range(&self, lo: u64, hi: u64) -> bool {
        self.small.is_some_and(|n| lo <= n && n <= hi)
    }
}

fn plural<'a>(m100: u32, forms: (&'a str, &'a str, &'a str)) -> &'a str {
    let n10 = m100 % 10;
    if n10 == 1 && m100 != 11 {
        forms.0
    } else if (2..=4).contains(&n10) && !(12..=14).contains(&m100) {
        forms.1
    } else {
        forms.2
    }
}

fn triple(n: u64, feminine: bool, out: &mut Vec<&'static str>) {
    let (h, r) = ((n / 100) as usize, (n % 100) as usize);
    if h != 0 {
        out.push(HUNDREDS[h]);
    }
    if (10..20).contains(&r) {
        out.push(TEENS[r - 10]);
    } else {
        let (t, u) = (r / 10, r % 10);
        if t != 0 {
            out.push(TENS[t]);
        }
        if u != 0 {
            out.push(match (feminine, u) {
                (true, 1) => "одна",
                (true, 2) => "две",
                _ => UNITS[u],
            });
        }
    }
}

/// `_number_to_words`. Python raises IndexError at >= 10^12 (no scale word); here
/// such numbers are read digit by digit instead.
fn number_to_words(num: &Num, feminine: bool) -> Vec<&'static str> {
    let Some(mut n) = num.small else {
        let k = num.digits.iter().position(|&d| d != 0).unwrap_or(0);
        return num.digits[k..].iter().map(|&d| UNITS[d as usize]).collect();
    };
    if n == 0 {
        return vec!["ноль"];
    }
    let mut parts = Vec::with_capacity(4);
    while n != 0 {
        parts.push(n % 1000);
        n /= 1000;
    }
    let mut words = Vec::new();
    for scale in (0..parts.len()).rev() {
        let r = parts[scale];
        if r == 0 {
            continue;
        }
        if scale == 0 {
            triple(r, feminine, &mut words);
        } else {
            let (one, few, many, fem) = SCALES[scale - 1];
            if !(scale == 1 && r == 1) {
                triple(r, fem, &mut words);
            }
            words.push(plural((r % 100) as u32, (one, few, many)));
        }
    }
    words
}

fn ordinal(words: Vec<&'static str>, case: &str) -> String {
    if words.len() == 2 && words[0] == "две" && words[1] == "тысячи" {
        return format!("двухтысячн{}", ord_end(case, 0));
    }
    let last = *words.last().unwrap_or(&"");
    let Some(stem) = lookup(ORD, last) else { return words.join(" ") };
    let kind = if stem == "трет" {
        2
    } else if matches!(stem, "втор" | "шест" | "седьм" | "восьм" | "сороков") {
        1
    } else {
        0
    };
    let mut out: Vec<String> = words[..words.len() - 1].iter().map(|s| s.to_string()).collect();
    out.push(format!("{stem}{}", ord_end(case, kind)));
    out.join(" ")
}

fn is_ru_letter(c: char) -> bool {
    matches!(c, 'а'..='я' | 'ё' | 'А'..='Я' | 'Ё')
}

fn number_repl(num: Num, frac: Option<Num>, frac_len: usize, nxt_w: &str) -> String {
    if let Some(fr) = frac {
        let mut a = number_to_words(&num, true);
        let b = number_to_words(&fr, true);
        let den = match frac_len {
            1 => Some(("десятая", "десятых", "десятых")),
            2 => Some(("сотая", "сотых", "сотых")),
            3 => Some(("тысячная", "тысячных", "тысячных")),
            _ => None,
        };
        match den {
            None => {
                a.push("запятая");
                a.extend(b);
            }
            Some(d) => {
                a.push(plural(num.m100, ("целая", "целых", "целых")));
                a.extend(b);
                a.push(plural(fr.m100, d));
            }
        }
        return a.join(" ");
    }
    if let Some(case) = lookup(GOD_CASE, nxt_w)
        && num.in_range(1000, 2100)
    {
        return ordinal(number_to_words(&num, false), case);
    }
    if MONTHS_GEN.contains(&nxt_w) && num.in_range(1, 31) {
        return ordinal(number_to_words(&num, false), "neut");
    }
    let feminine = (nxt_w.ends_with('а') || nxt_w.ends_with('я') || nxt_w.ends_with('ь'))
        && !matches!(nxt_w, "человека" | "года" | "раза" | "дня" | "рубля");
    let fem = feminine && matches!(num.m100 % 10, 1 | 2) && !matches!(num.m100, 11 | 12);
    number_to_words(&num, fem).join(" ")
}

fn expand_numbers(text: &str) -> String {
    let t: Vec<char> = text.chars().collect();
    // (\d+)\s*%  ->  digits + " " + процент(а/ов)
    let mut a: Vec<char> = Vec::with_capacity(t.len() + 16);
    let mut i = 0;
    while i < t.len() {
        if py_digit(t[i]).is_some() {
            let st = i;
            while i < t.len() && py_digit(t[i]).is_some() {
                i += 1;
            }
            let mut j = i;
            while j < t.len() && py_isspace(t[j]) {
                j += 1;
            }
            a.extend_from_slice(&t[st..i]);
            if j < t.len() && t[j] == '%' {
                let num = Num::new(t[st..i].iter().filter_map(|&c| py_digit(c)).collect());
                a.push(' ');
                a.extend(plural(num.m100, ("процент", "процента", "процентов")).chars());
                i = j + 1;
            }
            continue;
        }
        a.push(t[i]);
        i += 1;
    }
    // (\d[\d \xa0]*\d|\d)(?:[.,](\d+))?
    let t = a;
    let mut out = String::with_capacity(t.len() * 2);
    let mut i = 0;
    while i < t.len() {
        if py_digit(t[i]).is_none() {
            out.push(t[i]);
            i += 1;
            continue;
        }
        let st = i;
        let mut last = i;
        let mut j = i + 1;
        while j < t.len() && (py_digit(t[j]).is_some() || t[j] == ' ' || t[j] == '\u{a0}') {
            if py_digit(t[j]).is_some() {
                last = j;
            }
            j += 1;
        }
        let g1_end = last + 1;
        let num = Num::new(t[st..g1_end].iter().filter_map(|&c| py_digit(c)).collect());
        let mut end = g1_end;
        let mut frac = None;
        let mut frac_len = 0;
        if g1_end + 1 < t.len() && (t[g1_end] == '.' || t[g1_end] == ',') && py_digit(t[g1_end + 1]).is_some() {
            let mut k = g1_end + 1;
            while k < t.len() && py_digit(t[k]).is_some() {
                k += 1;
            }
            frac_len = k - (g1_end + 1);
            frac = Some(Num::new(t[g1_end + 1..k].iter().filter_map(|&c| py_digit(c)).collect()));
            end = k;
        }
        // nxt = re.match(r"\s*([а-яёА-ЯЁ]+)", text[m.end():])
        let mut k = end;
        while k < t.len() && py_isspace(t[k]) {
            k += 1;
        }
        let ws = k;
        while k < t.len() && is_ru_letter(t[k]) {
            k += 1;
        }
        let nxt_w: String = t[ws..k].iter().flat_map(|c| c.to_lowercase()).collect();
        out.push_str(&number_repl(num, frac, frac_len, &nxt_w));
        i = end;
    }
    out
}

// ===========================================================================
// tokenization and text_vocab.normalize_text
// ===========================================================================

const KEEP_PUNCT: &str = ".,!?:;";

fn is_keep_punct(c: char) -> bool {
    KEEP_PUNCT.contains(c)
}
/// `[а-яё]` under re.IGNORECASE
fn is_tok_cyr(c: char) -> bool {
    matches!(c, '\u{401}' | '\u{410}'..='\u{44f}' | '\u{451}' | '\u{1c80}'..='\u{1c86}')
}
/// `[a-z]` under re.IGNORECASE
fn is_tok_lat(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, '\u{130}' | '\u{131}' | '\u{17f}' | '\u{212a}')
}

/// `_TOKEN.findall`: `[а-яё]+|[a-z]+(?:'[a-z]+)?|[.,!?:;…]+` (IGNORECASE)
fn tokenize(text: &str) -> Vec<&str> {
    let ci: Vec<(usize, char)> = text.char_indices().collect();
    let byte = |k: usize| if k < ci.len() { ci[k].0 } else { text.len() };
    let mut toks = Vec::new();
    let mut i = 0;
    while i < ci.len() {
        let c = ci[i].1;
        let st = i;
        if is_tok_cyr(c) {
            while i < ci.len() && is_tok_cyr(ci[i].1) {
                i += 1;
            }
        } else if is_tok_lat(c) {
            while i < ci.len() && is_tok_lat(ci[i].1) {
                i += 1;
            }
            if i + 1 < ci.len() && ci[i].1 == '\'' && is_tok_lat(ci[i + 1].1) {
                i += 1;
                while i < ci.len() && is_tok_lat(ci[i].1) {
                    i += 1;
                }
            }
        } else if is_keep_punct(c) || c == '…' {
            while i < ci.len() && (is_keep_punct(ci[i].1) || ci[i].1 == '…') {
                i += 1;
            }
        } else {
            i += 1;
            continue;
        }
        toks.push(&text[byte(st)..byte(i)]);
    }
    toks
}

fn is_emoji(c: char) -> bool {
    matches!(c as u32,
        0x1f600..=0x1f64f | 0x1f300..=0x1f5ff | 0x1f680..=0x1f6ff | 0x1f700..=0x1f77f | 0x1f780..=0x1f7ff
        | 0x1f800..=0x1f8ff | 0x1f900..=0x1f9ff | 0x1fa00..=0x1fa6f | 0x1fa70..=0x1faff | 0x2600..=0x26ff
        | 0x2700..=0x27bf | 0x1f1e6..=0x1f1ff)
}

/// One `_AFFRICATE_LIGATURES` pass: `X͡Y|X͡Y|XY` (re.I) -> lig; `need_boundary` adds
/// the `(?=[^\w]|$)` lookahead of the 2-char form (the ts -> ʦ pattern).
fn ligature(text: &str, x: fn(char) -> bool, y: fn(char) -> bool, need_boundary: bool, lig: char) -> String {
    let t: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < t.len() {
        if x(t[i]) {
            if i + 2 < t.len() && t[i + 1] == '\u{361}' && y(t[i + 2]) {
                out.push(lig);
                i += 3;
                continue;
            }
            if i + 1 < t.len() && y(t[i + 1]) && (!need_boundary || i + 2 >= t.len() || !py_word(t[i + 2])) {
                out.push(lig);
                i += 2;
                continue;
            }
        }
        out.push(t[i]);
        i += 1;
    }
    out
}

/// `data.text_vocab.normalize_text(text, apply_hebrew_fixes=False)`.
fn normalize_text(text: &str) -> String {
    let mut t = String::with_capacity(text.len() + 4);
    for c in py_strip(text).nfd() {
        if is_emoji(c) || matches!(c, '\u{2665}' | '\u{2606}' | '\u{2661}' | '\u{a9}') {
            continue;
        }
        t.push(match c {
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2018}' | '\u{2019}' | '\u{b4}' | '`' => '\'',
            '\u{2013}' | '\u{2011}' | '\u{2014}' => '-',
            '_' | '[' | ']' | '|' | '^' | '$' => ' ',
            _ => c,
        });
    }
    for p in [",", ".", "!", "?", ";", ":", "'"] {
        let sp = format!(" {p}");
        if t.contains(&sp) {
            t = t.replace(&sp, p);
        }
    }
    for q in ["\"\"", "''", "``"] {
        while t.contains(q) {
            t = t.replace(q, &q[..1]);
        }
    }
    let t = ligature(&t, |c| matches!(c, 't' | 'T'), |c| matches!(c, 's' | 'S' | 'ſ'), true, 'ʦ');
    let t = ligature(&t, |c| matches!(c, 't' | 'T'), |c| matches!(c, 'ʃ' | 'Ʃ'), false, 'ʧ');
    let t = ligature(&t, |c| matches!(c, 'd' | 'D'), |c| matches!(c, 'ʒ' | 'Ʒ'), false, 'ʤ');
    let t = ligature(&t, |c| matches!(c, 'd' | 'D'), |c| matches!(c, 'z' | 'Z'), false, 'ʣ');
    let mut out = String::with_capacity(t.len() + 1);
    for part in t.split(py_isspace).filter(|p| !p.is_empty()) {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(part);
    }
    let ends_ok = out.chars().next_back().is_some_and(|c| {
        matches!(c, '.' | '!' | '?' | ';' | ':' | ',' | '\'' | '"' | ')' | ']' | '}' | '…' | '。' | '\\' | '』' | '】' | '〉' | '》' | '›' | '»')
    });
    if !ends_ok {
        out.push('.');
    }
    out
}

fn py_split(s: &str) -> Vec<String> {
    s.split(py_isspace).filter(|p| !p.is_empty()).map(str::to_string).collect()
}

// ===========================================================================
// the sentence pipeline
// ===========================================================================

const VSE_PLURAL_NEXT: &[&str] = &[
    "были", "ваши", "вместе", "вы", "двое", "другие", "люди", "мои", "мы", "наши", "они", "остальные", "свои", "стали",
    "такие", "те", "трое", "эти",
];
const VSE_SINGULAR_NEXT: &[&str] = &[
    "ближе", "более", "будет", "было", "в", "время", "дело", "еще", "ещё", "же", "знаю", "и", "как", "кончено", "кругом",
    "ли", "на", "не", "одно", "остальное", "от", "по", "понятно", "равно", "с", "так", "таки", "то", "у", "хорошо",
    "что", "это",
];
const YO_LITERARY_E: &[&str] = &["далеко", "недалеко"];

/// все (all, plural) vs всё (everything): the orthography rarely writes ё.
fn vse(next_word: Option<&str>) -> &'static str {
    let Some(n) = next_word else { return "всё" };
    if VSE_SINGULAR_NEXT.contains(&n) {
        return "всё";
    }
    if VSE_PLURAL_NEXT.contains(&n) || ["ые", "ие", "ия", "ы", "ют", "ят", "ли", "ны", "и"].iter().any(|e| n.ends_with(e)) {
        return "все";
    }
    "всё"
}

fn is_ru_vowel(c: char) -> bool {
    matches!(c, 'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я')
}

/// byte offsets of the vowels of `w`
fn vowel_offsets(w: &str) -> Vec<usize> {
    w.char_indices().filter(|(_, c)| is_ru_vowel(*c)).map(|(b, _)| b).collect()
}

fn insert_plus(w: &str, at: usize) -> String {
    let mut s = String::with_capacity(w.len() + 1);
    s.push_str(&w[..at]);
    s.push('+');
    s.push_str(&w[at..]);
    s
}

/// Python list indexing `vs[k]` after `k = min(k, len(vs) - 1)`.
fn py_pick(vs: &[usize], k: i64) -> usize {
    let n = vs.len() as i64;
    let mut k = k.min(n - 1);
    if k < 0 {
        k += n; // negative index from the end (never produced by build_ru_lexicon.py)
    }
    vs[k.max(0) as usize]
}

/// RUAccent's context models (homograph stress classifier, ё-homograph tagger).
///
/// The Rust pipeline calls these at exactly the points where
/// `russian_phonemizer.py` calls `_ContextModels`, with the same inputs, so an
/// ONNX implementation (or a bridge to the Python one) plugs in unchanged.
pub trait ContextModels {
    /// `yo_model/model.onnx` present (Python `yo_ok`).
    fn has_yo_model(&self) -> bool;
    /// `omograph/model.onnx` present (Python `om_ok`).
    fn has_homograph_model(&self) -> bool;
    /// ё-tagger over `sentence` (lower case, words space-separated, punctuation
    /// glued to the preceding word): word-start char offset -> YO (true) / NO_YO.
    /// `words`/`offsets` are the tokens and their offsets in `sentence`.
    /// Called once per text, only if it has >= 3 words and one of them is a
    /// ё-homograph.
    fn yo_labels(&self, sentence: &str, words: &[String], offsets: &[usize]) -> HashMap<usize, bool>;
    /// Pick one of `hypotheses` ('+'-forms) for token `word_index`; `marked` is the
    /// sentence with that word wrapped in `<w></w>`.
    fn choose_stress(&self, word_index: usize, marked: &str, hypotheses: &[String]) -> String;
}

/// Decisions made elsewhere (e.g. by the Python ONNX models), keyed by token index
/// in [`Russian::tokens`]. They apply only where the models would have been asked.
#[derive(Default, Clone, Debug)]
pub struct Overrides {
    /// token index -> ё (true) / no ё (false), for ё-homographs
    pub yo: HashMap<usize, bool>,
    /// token index -> chosen '+'-form, for stress homographs
    pub stress: HashMap<usize, String>,
}

impl ContextModels for Overrides {
    fn has_yo_model(&self) -> bool {
        !self.yo.is_empty()
    }
    fn has_homograph_model(&self) -> bool {
        !self.stress.is_empty()
    }
    fn yo_labels(&self, _sentence: &str, _words: &[String], offsets: &[usize]) -> HashMap<usize, bool> {
        self.yo.iter().filter_map(|(&i, &v)| offsets.get(i).map(|&o| (o, v))).collect()
    }
    fn choose_stress(&self, word_index: usize, _marked: &str, hypotheses: &[String]) -> String {
        match self.stress.get(&word_index) {
            Some(s) if hypotheses.contains(s) => s.clone(),
            _ => hypotheses[0].clone(),
        }
    }
}

/// The native context student (`ru-ctx.g2p`, trained by `tools/train_ru.py`): a
/// character-level encoder distilled from RUAccent's homograph classifier and
/// ё-tagger. One pass per sentence serves both decisions; the pass is cached so
/// every homograph in the sentence reuses it.
pub struct TaggerCtx<'a> {
    model: &'a Tagger,
    cache: RefCell<HashMap<String, Logits>>,
}

impl<'a> TaggerCtx<'a> {
    pub fn new(model: &'a Tagger) -> Self {
        TaggerCtx { model, cache: RefCell::new(HashMap::new()) }
    }

    /// Window of at most `max_len` characters around `focus`, snapped to a word
    /// start: (start offset, window text).
    fn window(&self, sc: &[char], focus: usize) -> (usize, String) {
        let max = self.model.cfg.max_len;
        if sc.len() <= max {
            return (0, sc.iter().collect());
        }
        let mut start = focus.saturating_sub(max / 2).min(sc.len() - max);
        while start > 0 && sc[start - 1] != ' ' && focus - start < max - 1 {
            start -= 1;
        }
        (start, sc[start..start + max].iter().collect())
    }

    /// Stress margin (stressed − unstressed logit) and ё probability per character
    /// of the window that covers `focus`; `start` is the window's offset.
    fn with_logits<R>(&self, sc: &[char], focus: usize, f: impl FnOnce(usize, &Logits) -> R) -> R {
        let (start, text) = self.window(sc, focus);
        let mut cache = self.cache.borrow_mut();
        if !cache.contains_key(&text) {
            let ids = self.model.encode(&text);
            let lg = self.model.forward(&ids);
            cache.insert(text.clone(), lg);
        }
        f(start, &cache[&text])
    }
}

impl ContextModels for TaggerCtx<'_> {
    fn has_yo_model(&self) -> bool {
        true
    }
    fn has_homograph_model(&self) -> bool {
        true
    }
    fn yo_labels(&self, sentence: &str, words: &[String], offsets: &[usize]) -> HashMap<usize, bool> {
        let sc: Vec<char> = sentence.chars().collect();
        let mut out = HashMap::new();
        for (w, &off) in words.iter().zip(offsets) {
            if !w.contains('е') || w.contains('ё') {
                continue;
            }
            let yo = self.with_logits(&sc, off, |start, lg| {
                w.chars().enumerate().any(|(k, c)| {
                    let p = off + k;
                    c == 'е' && p >= start && p - start < lg.len && lg.vow[(p - start) * 2 + 1] > lg.vow[(p - start) * 2]
                })
            });
            out.insert(off, yo);
        }
        out
    }
    fn choose_stress(&self, _word_index: usize, marked: &str, hypotheses: &[String]) -> String {
        let Some(a) = marked.find("<w>") else { return hypotheses[0].clone() };
        let a = marked[..a].chars().count();
        let sentence = marked.replacen("<w>", "", 1).replacen("</w>", "", 1);
        let sc: Vec<char> = sentence.chars().collect();
        self.with_logits(&sc, a, |start, lg| {
            let mut best = (f32::MIN, 0usize);
            for (i, h) in hypotheses.iter().enumerate() {
                let Some(k) = h.chars().position(|c| c == '+') else { continue };
                let p = a + k;
                if p < start || p - start >= lg.len {
                    continue;
                }
                let m = lg.stress[(p - start) * 2 + 1] - lg.stress[(p - start) * 2];
                if m > best.0 {
                    best = (m, i);
                }
            }
            hypotheses[best.1].clone()
        })
    }
}

/// An optional mmap'd lexicon tier.
struct Tier(Option<Lexicon>);

impl Tier {
    fn open(dir: Option<&Path>, name: &str) -> Tier {
        Tier(dir.map(|d| d.join(name)).filter(|p| p.is_file()).and_then(|p| Lexicon::open(&p).ok()))
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.0.as_ref()?.get_all(k)
    }
    fn int(&self, k: &str) -> Option<i64> {
        self.get(k)?.trim().parse().ok()
    }
    fn has(&self, k: &str) -> bool {
        self.get(k).is_some()
    }
}

type LatinFn = Box<dyn Fn(&str) -> String + Send + Sync>;
/// (word, tier, ipa) for every word, as `last_report()`.
pub type Report = Vec<(String, &'static str, String)>;

/// The Russian sentence pipeline (port of `phonemize_russian`).
pub struct Russian {
    acc: Tier,
    yo: Tier,
    hom: Tier,
    suf: Tier,
    train: Tier,
    ctx: Tier,
    yo_hom: Tier,
    latin: Option<LatinFn>,
    /// `ru-ctx.g2p`: the native homograph / ё context student, if present
    ctx_model: Option<Tagger>,
}

impl Russian {
    /// Open the optional lexicon tiers in `dir` (see the module docs for file
    /// names). Missing files are empty tiers: with none at all every polysyllable
    /// gets the out-of-dictionary suffix guess (which then falls back to the
    /// penultimate vowel).
    pub fn load(dir: Option<&Path>) -> Russian {
        Russian {
            acc: Tier::open(dir, "ru_accents.lex"),
            yo: Tier::open(dir, "ru_yo.lex"),
            hom: Tier::open(dir, "ru_homographs.lex"),
            suf: Tier::open(dir, "ru_suffix.lex"),
            train: Tier::open(dir, "ru_train.lex"),
            ctx: Tier::open(dir, "ru_homograph_ctx.lex"),
            yo_hom: Tier::open(dir, "ru_yo_homographs.lex"),
            latin: None,
            ctx_model: Self::load_ctx(dir),
        }
    }

    fn load_ctx(dir: Option<&Path>) -> Option<Tagger> {
        if matches!(std::env::var("POLYG2P_RU_CTX").as_deref(), Ok("0") | Ok("off") | Ok("false")) {
            return None;
        }
        dir.and_then(|d| Tagger::open(&d.join("ru-ctx.g2p")).ok()).or_else(|| Tagger::from_bytes(RU_CTX_BUILTIN).ok())
    }

    /// Whether the native context student (`ru-ctx.g2p`) is loaded.
    pub fn has_context_model(&self) -> bool {
        self.ctx_model.is_some()
    }

    /// Route Latin-script runs to `f` (e.g. an English G2P). It receives the run of
    /// consecutive Latin words joined by spaces and returns their IPA; if the result
    /// has as many whitespace-separated words as the run, they are kept one per
    /// word, else as one token (as Python does with eSpeak's output).
    pub fn with_latin<F: Fn(&str) -> String + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.latin = Some(Box::new(f));
        self
    }

    /// Which lexicon tiers are loaded: (accents, yo, homographs, suffix, train, ctx, yo_homographs).
    pub fn tiers_loaded(&self) -> [bool; 7] {
        [&self.acc, &self.yo, &self.hom, &self.suf, &self.train, &self.ctx, &self.yo_hom].map(|t| t.0.is_some())
    }

    /// Russian text -> normalized IPA in the training-label conventions.
    pub fn phonemize(&self, text: &str) -> String {
        match &self.ctx_model {
            Some(m) => self.run(text, Some(&TaggerCtx::new(m)), None),
            None => self.run(text, None, None),
        }
    }

    /// As [`phonemize`](Self::phonemize), with context-model decisions.
    pub fn phonemize_with(&self, text: &str, models: Option<&dyn ContextModels>) -> String {
        self.phonemize_report(text, models).0
    }

    /// (word, tier, ipa) for every word of `text` (Python `last_report()`).
    pub fn stress_report(&self, text: &str) -> Report {
        let mut r = Vec::new();
        match &self.ctx_model {
            Some(m) => self.run(text, Some(&TaggerCtx::new(m)), Some(&mut r)),
            None => self.run(text, None, Some(&mut r)),
        };
        r
    }

    /// `phonemize_with` plus the report.
    pub fn phonemize_report(&self, text: &str, models: Option<&dyn ContextModels>) -> (String, Report) {
        let mut r = Vec::new();
        // explicit decisions win; otherwise the native student, if loaded
        let s = match (models, &self.ctx_model) {
            (Some(m), _) => self.run(text, Some(m), Some(&mut r)),
            (None, Some(t)) => self.run(text, Some(&TaggerCtx::new(t)), Some(&mut r)),
            (None, None) => self.run(text, None, Some(&mut r)),
        };
        (s, r)
    }

    /// The normalized lower-case tokens (words and punctuation runs) the pipeline
    /// works on; [`Overrides`] indices refer to positions in this list.
    pub fn tokens(&self, text: &str) -> Vec<String> {
        let t = Self::normalize_input(text);
        tokenize(&t).into_iter().map(str::to_lowercase).collect()
    }

    fn normalize_input(text: &str) -> String {
        let mut t: String = text.nfc().collect();
        for (sign, word) in SIGNS {
            if t.contains(*sign) {
                t = t.replace(*sign, &format!(" {word} "));
            }
        }
        let t = expand_numbers(&t);
        t.replace('…', ".").replace('\u{301}', "")
    }

    fn suffix_stress(&self, w: &str) -> Option<usize> {
        let nv = w.chars().filter(|&c| is_ru_vowel(c)).count();
        if nv < 2 {
            return if nv > 0 { Some(0) } else { None };
        }
        let cb: Vec<usize> = w.char_indices().map(|(b, _)| b).collect();
        let n = cb.len();
        for l in [6usize, 5, 4, 3] {
            if n > l
                && let Some(v) = self.suf.int(&w[cb[n - l]..])
            {
                let k = nv as i64 - 1 - v;
                if 0 <= k && k < nv as i64 {
                    return Some(k as usize);
                }
            }
        }
        Some(nv.saturating_sub(2)) // penultimate
    }

    /// `_stress_word` -> ('+'-accented word, tier). `marked` builds the `<w>` sentence.
    #[allow(clippy::too_many_arguments)]
    fn stress_word(
        &self,
        tok: &str,
        idx: usize,
        nxt: Option<&str>,
        prv: Option<&str>,
        yo_label: Option<bool>,
        models: Option<&dyn ContextModels>,
        marked: Option<&dyn Fn() -> String>,
    ) -> (String, &'static str) {
        let mut w: String = tok.to_string();
        let no_yo = !w.contains('ё');
        if no_yo && self.yo_hom.has(&w) && !YO_LITERARY_E.contains(&w.as_str()) {
            if let Some(lbl) = yo_label {
                if lbl && let Some(v) = self.yo_hom.get(&w) {
                    w = v.to_string();
                }
            } else if w == "все" && (nxt.is_some() || prv.is_some()) {
                let v = vse(nxt);
                if v == "все" {
                    return ("вс+е".to_string(), "context");
                }
                w = v.to_string();
            }
        } else if no_yo && let Some(v) = self.yo.get(&w) {
            w = v.to_string();
        }
        let vs = vowel_offsets(&w);
        if vs.is_empty() {
            return (w, "novowel");
        }
        if let Some(i) = w.rfind('ё') {
            return (insert_plus(&w, i), "yo");
        }
        if vs.len() == 1 {
            return (insert_plus(&w, vs[0]), "mono");
        }
        let (k, tier);
        if let Some(hv) = self.hom.get(&w) {
            let cands: Vec<i64> = hv.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            if let (Some(m), Some(marked)) = (models, marked) {
                // [c for c in hom[w] if c < len(vs)], then vs[c] (Python indexing)
                let valid: Vec<i64> = cands.iter().copied().filter(|&c| c < vs.len() as i64 && c >= -(vs.len() as i64)).collect();
                if m.has_homograph_model() && valid.len() > 1 {
                    let hyps: Vec<String> = valid.iter().map(|&c| insert_plus(&w, py_pick(&vs, c))).collect();
                    return (m.choose_stress(idx, &marked(), &hyps), "homograph-model");
                }
            }
            let mut kk = cands.first().copied().unwrap_or(0);
            let mut tt = "homograph-prior";
            let keys = [format!("p|{}|{}", prv.unwrap_or("<s>"), w), format!("n|{}|{}", w, nxt.unwrap_or("</s>"))];
            for key in &keys {
                if let Some(v) = self.ctx.int(key) {
                    kk = v;
                    tt = "homograph-prior-ctx";
                    break;
                }
            }
            k = kk;
            tier = tt;
        } else if let Some(v) = self.acc.int(&w) {
            k = v;
            tier = "dict";
        } else if let Some(v) = self.train.int(&w) {
            k = v;
            tier = "train";
        } else {
            return (w, "ood");
        }
        (insert_plus(&w, py_pick(&vs, k)), tier)
    }

    fn flush_latin(&self, latin_run: &mut Vec<&str>, out: &mut Vec<String>, report: &mut Option<&mut Report>) {
        if latin_run.is_empty() {
            return;
        }
        if let Some(f) = &self.latin {
            let o = f(&latin_run.join(" "));
            let parts = py_split(&o);
            let ipas = if parts.len() == latin_run.len() { parts } else { vec![py_strip(&o).to_string()] };
            if let Some(r) = report.as_deref_mut() {
                for (w, p) in latin_run.iter().zip(&ipas) {
                    r.push((w.to_string(), "latin", p.clone()));
                }
            }
            out.extend(ipas);
        }
        latin_run.clear();
    }

    fn run(&self, text: &str, models: Option<&dyn ContextModels>, mut report: Option<&mut Report>) -> String {
        let norm = Self::normalize_input(text);
        let words: Vec<String> = tokenize(&norm).into_iter().map(str::to_lowercase).collect();
        let n = words.len();
        let is_p: Vec<bool> = words.iter().map(|w| w.chars().next().is_some_and(is_keep_punct)).collect();
        let mut prev_w: Vec<Option<usize>> = vec![None; n];
        let mut next_w: Vec<Option<usize>> = vec![None; n];
        let mut last = None;
        for i in 0..n {
            prev_w[i] = last;
            if !is_p[i] {
                last = Some(i);
            }
        }
        last = None;
        for i in (0..n).rev() {
            next_w[i] = last;
            if !is_p[i] {
                last = Some(i);
            }
        }

        // the sentence as the context models saw it in RUAccent
        let mut sentence = String::new();
        let mut offsets: Vec<usize> = Vec::new();
        let mut yo: HashMap<usize, bool> = HashMap::new();
        if let Some(m) = models {
            let mut pos = 0usize;
            for (k, t) in words.iter().enumerate() {
                let glue = k > 0 && t.chars().next().is_some_and(|c| is_keep_punct(c) || c == '…');
                if k > 0 && !glue {
                    pos += 1;
                    sentence.push(' ');
                }
                offsets.push(pos);
                sentence.push_str(t);
                pos += t.chars().count();
            }
            let n_words = is_p.iter().filter(|&&p| !p).count();
            if m.has_yo_model() && n_words >= 3 && words.iter().any(|t| self.yo_hom.has(t)) {
                let lab = m.yo_labels(&sentence, &words, &offsets);
                for (k, t) in words.iter().enumerate() {
                    if self.yo_hom.has(t)
                        && let Some(&l) = lab.get(&offsets[k])
                    {
                        yo.insert(k, l);
                    }
                }
            }
        }
        let marked_for = |k: usize| -> String {
            let sc: Vec<char> = sentence.chars().collect();
            let a = offsets[k];
            let wl = words[k].chars().count();
            let mut s: String = sc[..a].iter().collect();
            s.push_str("<w>");
            s.push_str(&words[k]);
            s.push_str("</w>");
            s.extend(sc[(a + wl).min(sc.len())..].iter());
            s
        };

        let mut out: Vec<String> = Vec::with_capacity(n);
        let mut latin_run: Vec<&str> = Vec::new();
        for (i, tok) in words.iter().enumerate() {
            if is_p[i] {
                self.flush_latin(&mut latin_run, &mut out, &mut report);
                let p: String = tok.chars().filter(|&c| is_keep_punct(c)).collect();
                if !p.is_empty()
                    && let Some(last) = out.last_mut()
                {
                    // collapse runs like '?!' '...' onto the labels' single-mark style
                    let first = p.chars().next().unwrap_or('.');
                    let distinct = p.chars().any(|c| c != first);
                    let tail = if distinct && (first == '?' || first == '!') { p.clone() } else { first.to_string() };
                    let keep = last.trim_end_matches(is_keep_punct).len();
                    last.truncate(keep);
                    last.push_str(&tail);
                }
                continue;
            }
            if tok.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                latin_run.push(tok);
                continue;
            }
            self.flush_latin(&mut latin_run, &mut out, &mut report);
            let nxt = next_w[i].map(|k| words[k].as_str());
            let prv = prev_w[i].map(|k| words[k].as_str());
            let marked = || marked_for(i);
            // Python builds the <w> sentence only for tokens that are (ё-)homographs themselves
            let marked_opt: Option<&dyn Fn() -> String> = if self.hom.has(tok) || self.yo_hom.has(tok) { Some(&marked) } else { None };
            let (mut acc, mut tier) = self.stress_word(tok, i, nxt, prv, yo.get(&i).copied(), models, marked_opt);
            if tier == "ood" {
                let vs = vowel_offsets(tok);
                acc = match self.suffix_stress(tok) {
                    Some(k) if !vs.is_empty() => insert_plus(tok, vs[k]),
                    _ => tok.clone(),
                };
                tier = "ood-suffix";
            }
            let ipa = word_to_ipa(&acc, None);
            if let Some(r) = report.as_deref_mut() {
                r.push((tok.clone(), tier, ipa.clone()));
            }
            out.push(ipa);
        }
        self.flush_latin(&mut latin_run, &mut out, &mut report);
        let mut s = String::new();
        for x in out.iter().filter(|x| !x.is_empty()) {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(x);
        }
        if s.is_empty() { s } else { normalize_text(&s) }
    }
}

// ===========================================================================
// generated Unicode tables (Python 3.13, Unicode 15.1)
// ===========================================================================

/// Python `\d` (Unicode Nd): the zero of each run of ten consecutive digits.
const PY_DIGIT_ZEROS: &[u32] = &[
    0x30, 0x660, 0x6F0, 0x7C0, 0x966, 0x9E6, 0xA66, 0xAE6, 0xB66, 0xBE6, 0xC66, 0xCE6,
    0xD66, 0xDE6, 0xE50, 0xED0, 0xF20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10, 0x104A0, 0x10D30, 0x11066, 0x110F0, 0x11136, 0x111D0, 0x112F0, 0x11450, 0x114D0, 0x11650, 0x116C0,
    0x11730, 0x118E0, 0x11950, 0x11C50, 0x11D50, 0x11DA0, 0x11F50, 0x16A60, 0x16AC0, 0x16B50, 0x1D7CE, 0x1D7D8,
    0x1D7E2, 0x1D7EC, 0x1D7F6, 0x1E140, 0x1E2F0, 0x1E4F0, 0x1E950, 0x1FBF0,
];

/// Python `\w` (str.isalnum() or "_"): inclusive code point ranges.
const PY_WORD: &[(u32, u32)] = &[
    (0x30, 0x39), (0x41, 0x5A), (0x5F, 0x5F), (0x61, 0x7A), (0xAA, 0xAA), (0xB2, 0xB3), (0xB5, 0xB5), (0xB9, 0xBA),
    (0xBC, 0xBE), (0xC0, 0xD6), (0xD8, 0xF6), (0xF8, 0x2C1), (0x2C6, 0x2D1), (0x2E0, 0x2E4), (0x2EC, 0x2EC), (0x2EE, 0x2EE),
    (0x370, 0x374), (0x376, 0x377), (0x37A, 0x37D), (0x37F, 0x37F), (0x386, 0x386), (0x388, 0x38A), (0x38C, 0x38C), (0x38E, 0x3A1),
    (0x3A3, 0x3F5), (0x3F7, 0x481), (0x48A, 0x52F), (0x531, 0x556), (0x559, 0x559), (0x560, 0x588), (0x5D0, 0x5EA), (0x5EF, 0x5F2),
    (0x620, 0x64A), (0x660, 0x669), (0x66E, 0x66F), (0x671, 0x6D3), (0x6D5, 0x6D5), (0x6E5, 0x6E6), (0x6EE, 0x6FC), (0x6FF, 0x6FF),
    (0x710, 0x710), (0x712, 0x72F), (0x74D, 0x7A5), (0x7B1, 0x7B1), (0x7C0, 0x7EA), (0x7F4, 0x7F5), (0x7FA, 0x7FA), (0x800, 0x815),
    (0x81A, 0x81A), (0x824, 0x824), (0x828, 0x828), (0x840, 0x858), (0x860, 0x86A), (0x870, 0x887), (0x889, 0x88E), (0x8A0, 0x8C9),
    (0x904, 0x939), (0x93D, 0x93D), (0x950, 0x950), (0x958, 0x961), (0x966, 0x96F), (0x971, 0x980), (0x985, 0x98C), (0x98F, 0x990),
    (0x993, 0x9A8), (0x9AA, 0x9B0), (0x9B2, 0x9B2), (0x9B6, 0x9B9), (0x9BD, 0x9BD), (0x9CE, 0x9CE), (0x9DC, 0x9DD), (0x9DF, 0x9E1),
    (0x9E6, 0x9F1), (0x9F4, 0x9F9), (0x9FC, 0x9FC), (0xA05, 0xA0A), (0xA0F, 0xA10), (0xA13, 0xA28), (0xA2A, 0xA30), (0xA32, 0xA33),
    (0xA35, 0xA36), (0xA38, 0xA39), (0xA59, 0xA5C), (0xA5E, 0xA5E), (0xA66, 0xA6F), (0xA72, 0xA74), (0xA85, 0xA8D), (0xA8F, 0xA91),
    (0xA93, 0xAA8), (0xAAA, 0xAB0), (0xAB2, 0xAB3), (0xAB5, 0xAB9), (0xABD, 0xABD), (0xAD0, 0xAD0), (0xAE0, 0xAE1), (0xAE6, 0xAEF),
    (0xAF9, 0xAF9), (0xB05, 0xB0C), (0xB0F, 0xB10), (0xB13, 0xB28), (0xB2A, 0xB30), (0xB32, 0xB33), (0xB35, 0xB39), (0xB3D, 0xB3D),
    (0xB5C, 0xB5D), (0xB5F, 0xB61), (0xB66, 0xB6F), (0xB71, 0xB77), (0xB83, 0xB83), (0xB85, 0xB8A), (0xB8E, 0xB90), (0xB92, 0xB95),
    (0xB99, 0xB9A), (0xB9C, 0xB9C), (0xB9E, 0xB9F), (0xBA3, 0xBA4), (0xBA8, 0xBAA), (0xBAE, 0xBB9), (0xBD0, 0xBD0), (0xBE6, 0xBF2),
    (0xC05, 0xC0C), (0xC0E, 0xC10), (0xC12, 0xC28), (0xC2A, 0xC39), (0xC3D, 0xC3D), (0xC58, 0xC5A), (0xC5D, 0xC5D), (0xC60, 0xC61),
    (0xC66, 0xC6F), (0xC78, 0xC7E), (0xC80, 0xC80), (0xC85, 0xC8C), (0xC8E, 0xC90), (0xC92, 0xCA8), (0xCAA, 0xCB3), (0xCB5, 0xCB9),
    (0xCBD, 0xCBD), (0xCDD, 0xCDE), (0xCE0, 0xCE1), (0xCE6, 0xCEF), (0xCF1, 0xCF2), (0xD04, 0xD0C), (0xD0E, 0xD10), (0xD12, 0xD3A),
    (0xD3D, 0xD3D), (0xD4E, 0xD4E), (0xD54, 0xD56), (0xD58, 0xD61), (0xD66, 0xD78), (0xD7A, 0xD7F), (0xD85, 0xD96), (0xD9A, 0xDB1),
    (0xDB3, 0xDBB), (0xDBD, 0xDBD), (0xDC0, 0xDC6), (0xDE6, 0xDEF), (0xE01, 0xE30), (0xE32, 0xE33), (0xE40, 0xE46), (0xE50, 0xE59),
    (0xE81, 0xE82), (0xE84, 0xE84), (0xE86, 0xE8A), (0xE8C, 0xEA3), (0xEA5, 0xEA5), (0xEA7, 0xEB0), (0xEB2, 0xEB3), (0xEBD, 0xEBD),
    (0xEC0, 0xEC4), (0xEC6, 0xEC6), (0xED0, 0xED9), (0xEDC, 0xEDF), (0xF00, 0xF00), (0xF20, 0xF33), (0xF40, 0xF47), (0xF49, 0xF6C),
    (0xF88, 0xF8C), (0x1000, 0x102A), (0x103F, 0x1049), (0x1050, 0x1055), (0x105A, 0x105D), (0x1061, 0x1061), (0x1065, 0x1066), (0x106E, 0x1070),
    (0x1075, 0x1081), (0x108E, 0x108E), (0x1090, 0x1099), (0x10A0, 0x10C5), (0x10C7, 0x10C7), (0x10CD, 0x10CD), (0x10D0, 0x10FA), (0x10FC, 0x1248),
    (0x124A, 0x124D), (0x1250, 0x1256), (0x1258, 0x1258), (0x125A, 0x125D), (0x1260, 0x1288), (0x128A, 0x128D), (0x1290, 0x12B0), (0x12B2, 0x12B5),
    (0x12B8, 0x12BE), (0x12C0, 0x12C0), (0x12C2, 0x12C5), (0x12C8, 0x12D6), (0x12D8, 0x1310), (0x1312, 0x1315), (0x1318, 0x135A), (0x1369, 0x137C),
    (0x1380, 0x138F), (0x13A0, 0x13F5), (0x13F8, 0x13FD), (0x1401, 0x166C), (0x166F, 0x167F), (0x1681, 0x169A), (0x16A0, 0x16EA), (0x16EE, 0x16F8),
    (0x1700, 0x1711), (0x171F, 0x1731), (0x1740, 0x1751), (0x1760, 0x176C), (0x176E, 0x1770), (0x1780, 0x17B3), (0x17D7, 0x17D7), (0x17DC, 0x17DC),
    (0x17E0, 0x17E9), (0x17F0, 0x17F9), (0x1810, 0x1819), (0x1820, 0x1878), (0x1880, 0x1884), (0x1887, 0x18A8), (0x18AA, 0x18AA), (0x18B0, 0x18F5),
    (0x1900, 0x191E), (0x1946, 0x196D), (0x1970, 0x1974), (0x1980, 0x19AB), (0x19B0, 0x19C9), (0x19D0, 0x19DA), (0x1A00, 0x1A16), (0x1A20, 0x1A54),
    (0x1A80, 0x1A89), (0x1A90, 0x1A99), (0x1AA7, 0x1AA7), (0x1B05, 0x1B33), (0x1B45, 0x1B4C), (0x1B50, 0x1B59), (0x1B83, 0x1BA0), (0x1BAE, 0x1BE5),
    (0x1C00, 0x1C23), (0x1C40, 0x1C49), (0x1C4D, 0x1C7D), (0x1C80, 0x1C88), (0x1C90, 0x1CBA), (0x1CBD, 0x1CBF), (0x1CE9, 0x1CEC), (0x1CEE, 0x1CF3),
    (0x1CF5, 0x1CF6), (0x1CFA, 0x1CFA), (0x1D00, 0x1DBF), (0x1E00, 0x1F15), (0x1F18, 0x1F1D), (0x1F20, 0x1F45), (0x1F48, 0x1F4D), (0x1F50, 0x1F57),
    (0x1F59, 0x1F59), (0x1F5B, 0x1F5B), (0x1F5D, 0x1F5D), (0x1F5F, 0x1F7D), (0x1F80, 0x1FB4), (0x1FB6, 0x1FBC), (0x1FBE, 0x1FBE), (0x1FC2, 0x1FC4),
    (0x1FC6, 0x1FCC), (0x1FD0, 0x1FD3), (0x1FD6, 0x1FDB), (0x1FE0, 0x1FEC), (0x1FF2, 0x1FF4), (0x1FF6, 0x1FFC), (0x2070, 0x2071), (0x2074, 0x2079),
    (0x207F, 0x2089), (0x2090, 0x209C), (0x2102, 0x2102), (0x2107, 0x2107), (0x210A, 0x2113), (0x2115, 0x2115), (0x2119, 0x211D), (0x2124, 0x2124),
    (0x2126, 0x2126), (0x2128, 0x2128), (0x212A, 0x212D), (0x212F, 0x2139), (0x213C, 0x213F), (0x2145, 0x2149), (0x214E, 0x214E), (0x2150, 0x2189),
    (0x2460, 0x249B), (0x24EA, 0x24FF), (0x2776, 0x2793), (0x2C00, 0x2CE4), (0x2CEB, 0x2CEE), (0x2CF2, 0x2CF3), (0x2CFD, 0x2CFD), (0x2D00, 0x2D25),
    (0x2D27, 0x2D27), (0x2D2D, 0x2D2D), (0x2D30, 0x2D67), (0x2D6F, 0x2D6F), (0x2D80, 0x2D96), (0x2DA0, 0x2DA6), (0x2DA8, 0x2DAE), (0x2DB0, 0x2DB6),
    (0x2DB8, 0x2DBE), (0x2DC0, 0x2DC6), (0x2DC8, 0x2DCE), (0x2DD0, 0x2DD6), (0x2DD8, 0x2DDE), (0x2E2F, 0x2E2F), (0x3005, 0x3007), (0x3021, 0x3029),
    (0x3031, 0x3035), (0x3038, 0x303C), (0x3041, 0x3096), (0x309D, 0x309F), (0x30A1, 0x30FA), (0x30FC, 0x30FF), (0x3105, 0x312F), (0x3131, 0x318E),
    (0x3192, 0x3195), (0x31A0, 0x31BF), (0x31F0, 0x31FF), (0x3220, 0x3229), (0x3248, 0x324F), (0x3251, 0x325F), (0x3280, 0x3289), (0x32B1, 0x32BF),
    (0x3400, 0x4DBF), (0x4E00, 0xA48C), (0xA4D0, 0xA4FD), (0xA500, 0xA60C), (0xA610, 0xA62B), (0xA640, 0xA66E), (0xA67F, 0xA69D), (0xA6A0, 0xA6EF),
    (0xA717, 0xA71F), (0xA722, 0xA788), (0xA78B, 0xA7CA), (0xA7D0, 0xA7D1), (0xA7D3, 0xA7D3), (0xA7D5, 0xA7D9), (0xA7F2, 0xA801), (0xA803, 0xA805),
    (0xA807, 0xA80A), (0xA80C, 0xA822), (0xA830, 0xA835), (0xA840, 0xA873), (0xA882, 0xA8B3), (0xA8D0, 0xA8D9), (0xA8F2, 0xA8F7), (0xA8FB, 0xA8FB),
    (0xA8FD, 0xA8FE), (0xA900, 0xA925), (0xA930, 0xA946), (0xA960, 0xA97C), (0xA984, 0xA9B2), (0xA9CF, 0xA9D9), (0xA9E0, 0xA9E4), (0xA9E6, 0xA9FE),
    (0xAA00, 0xAA28), (0xAA40, 0xAA42), (0xAA44, 0xAA4B), (0xAA50, 0xAA59), (0xAA60, 0xAA76), (0xAA7A, 0xAA7A), (0xAA7E, 0xAAAF), (0xAAB1, 0xAAB1),
    (0xAAB5, 0xAAB6), (0xAAB9, 0xAABD), (0xAAC0, 0xAAC0), (0xAAC2, 0xAAC2), (0xAADB, 0xAADD), (0xAAE0, 0xAAEA), (0xAAF2, 0xAAF4), (0xAB01, 0xAB06),
    (0xAB09, 0xAB0E), (0xAB11, 0xAB16), (0xAB20, 0xAB26), (0xAB28, 0xAB2E), (0xAB30, 0xAB5A), (0xAB5C, 0xAB69), (0xAB70, 0xABE2), (0xABF0, 0xABF9),
    (0xAC00, 0xD7A3), (0xD7B0, 0xD7C6), (0xD7CB, 0xD7FB), (0xF900, 0xFA6D), (0xFA70, 0xFAD9), (0xFB00, 0xFB06), (0xFB13, 0xFB17), (0xFB1D, 0xFB1D),
    (0xFB1F, 0xFB28), (0xFB2A, 0xFB36), (0xFB38, 0xFB3C), (0xFB3E, 0xFB3E), (0xFB40, 0xFB41), (0xFB43, 0xFB44), (0xFB46, 0xFBB1), (0xFBD3, 0xFD3D),
    (0xFD50, 0xFD8F), (0xFD92, 0xFDC7), (0xFDF0, 0xFDFB), (0xFE70, 0xFE74), (0xFE76, 0xFEFC), (0xFF10, 0xFF19), (0xFF21, 0xFF3A), (0xFF41, 0xFF5A),
    (0xFF66, 0xFFBE), (0xFFC2, 0xFFC7), (0xFFCA, 0xFFCF), (0xFFD2, 0xFFD7), (0xFFDA, 0xFFDC), (0x10000, 0x1000B), (0x1000D, 0x10026), (0x10028, 0x1003A),
    (0x1003C, 0x1003D), (0x1003F, 0x1004D), (0x10050, 0x1005D), (0x10080, 0x100FA), (0x10107, 0x10133), (0x10140, 0x10178), (0x1018A, 0x1018B), (0x10280, 0x1029C),
    (0x102A0, 0x102D0), (0x102E1, 0x102FB), (0x10300, 0x10323), (0x1032D, 0x1034A), (0x10350, 0x10375), (0x10380, 0x1039D), (0x103A0, 0x103C3), (0x103C8, 0x103CF),
    (0x103D1, 0x103D5), (0x10400, 0x1049D), (0x104A0, 0x104A9), (0x104B0, 0x104D3), (0x104D8, 0x104FB), (0x10500, 0x10527), (0x10530, 0x10563), (0x10570, 0x1057A),
    (0x1057C, 0x1058A), (0x1058C, 0x10592), (0x10594, 0x10595), (0x10597, 0x105A1), (0x105A3, 0x105B1), (0x105B3, 0x105B9), (0x105BB, 0x105BC), (0x10600, 0x10736),
    (0x10740, 0x10755), (0x10760, 0x10767), (0x10780, 0x10785), (0x10787, 0x107B0), (0x107B2, 0x107BA), (0x10800, 0x10805), (0x10808, 0x10808), (0x1080A, 0x10835),
    (0x10837, 0x10838), (0x1083C, 0x1083C), (0x1083F, 0x10855), (0x10858, 0x10876), (0x10879, 0x1089E), (0x108A7, 0x108AF), (0x108E0, 0x108F2), (0x108F4, 0x108F5),
    (0x108FB, 0x1091B), (0x10920, 0x10939), (0x10980, 0x109B7), (0x109BC, 0x109CF), (0x109D2, 0x10A00), (0x10A10, 0x10A13), (0x10A15, 0x10A17), (0x10A19, 0x10A35),
    (0x10A40, 0x10A48), (0x10A60, 0x10A7E), (0x10A80, 0x10A9F), (0x10AC0, 0x10AC7), (0x10AC9, 0x10AE4), (0x10AEB, 0x10AEF), (0x10B00, 0x10B35), (0x10B40, 0x10B55),
    (0x10B58, 0x10B72), (0x10B78, 0x10B91), (0x10BA9, 0x10BAF), (0x10C00, 0x10C48), (0x10C80, 0x10CB2), (0x10CC0, 0x10CF2), (0x10CFA, 0x10D23), (0x10D30, 0x10D39),
    (0x10E60, 0x10E7E), (0x10E80, 0x10EA9), (0x10EB0, 0x10EB1), (0x10F00, 0x10F27), (0x10F30, 0x10F45), (0x10F51, 0x10F54), (0x10F70, 0x10F81), (0x10FB0, 0x10FCB),
    (0x10FE0, 0x10FF6), (0x11003, 0x11037), (0x11052, 0x1106F), (0x11071, 0x11072), (0x11075, 0x11075), (0x11083, 0x110AF), (0x110D0, 0x110E8), (0x110F0, 0x110F9),
    (0x11103, 0x11126), (0x11136, 0x1113F), (0x11144, 0x11144), (0x11147, 0x11147), (0x11150, 0x11172), (0x11176, 0x11176), (0x11183, 0x111B2), (0x111C1, 0x111C4),
    (0x111D0, 0x111DA), (0x111DC, 0x111DC), (0x111E1, 0x111F4), (0x11200, 0x11211), (0x11213, 0x1122B), (0x1123F, 0x11240), (0x11280, 0x11286), (0x11288, 0x11288),
    (0x1128A, 0x1128D), (0x1128F, 0x1129D), (0x1129F, 0x112A8), (0x112B0, 0x112DE), (0x112F0, 0x112F9), (0x11305, 0x1130C), (0x1130F, 0x11310), (0x11313, 0x11328),
    (0x1132A, 0x11330), (0x11332, 0x11333), (0x11335, 0x11339), (0x1133D, 0x1133D), (0x11350, 0x11350), (0x1135D, 0x11361), (0x11400, 0x11434), (0x11447, 0x1144A),
    (0x11450, 0x11459), (0x1145F, 0x11461), (0x11480, 0x114AF), (0x114C4, 0x114C5), (0x114C7, 0x114C7), (0x114D0, 0x114D9), (0x11580, 0x115AE), (0x115D8, 0x115DB),
    (0x11600, 0x1162F), (0x11644, 0x11644), (0x11650, 0x11659), (0x11680, 0x116AA), (0x116B8, 0x116B8), (0x116C0, 0x116C9), (0x11700, 0x1171A), (0x11730, 0x1173B),
    (0x11740, 0x11746), (0x11800, 0x1182B), (0x118A0, 0x118F2), (0x118FF, 0x11906), (0x11909, 0x11909), (0x1190C, 0x11913), (0x11915, 0x11916), (0x11918, 0x1192F),
    (0x1193F, 0x1193F), (0x11941, 0x11941), (0x11950, 0x11959), (0x119A0, 0x119A7), (0x119AA, 0x119D0), (0x119E1, 0x119E1), (0x119E3, 0x119E3), (0x11A00, 0x11A00),
    (0x11A0B, 0x11A32), (0x11A3A, 0x11A3A), (0x11A50, 0x11A50), (0x11A5C, 0x11A89), (0x11A9D, 0x11A9D), (0x11AB0, 0x11AF8), (0x11C00, 0x11C08), (0x11C0A, 0x11C2E),
    (0x11C40, 0x11C40), (0x11C50, 0x11C6C), (0x11C72, 0x11C8F), (0x11D00, 0x11D06), (0x11D08, 0x11D09), (0x11D0B, 0x11D30), (0x11D46, 0x11D46), (0x11D50, 0x11D59),
    (0x11D60, 0x11D65), (0x11D67, 0x11D68), (0x11D6A, 0x11D89), (0x11D98, 0x11D98), (0x11DA0, 0x11DA9), (0x11EE0, 0x11EF2), (0x11F02, 0x11F02), (0x11F04, 0x11F10),
    (0x11F12, 0x11F33), (0x11F50, 0x11F59), (0x11FB0, 0x11FB0), (0x11FC0, 0x11FD4), (0x12000, 0x12399), (0x12400, 0x1246E), (0x12480, 0x12543), (0x12F90, 0x12FF0),
    (0x13000, 0x1342F), (0x13441, 0x13446), (0x14400, 0x14646), (0x16800, 0x16A38), (0x16A40, 0x16A5E), (0x16A60, 0x16A69), (0x16A70, 0x16ABE), (0x16AC0, 0x16AC9),
    (0x16AD0, 0x16AED), (0x16B00, 0x16B2F), (0x16B40, 0x16B43), (0x16B50, 0x16B59), (0x16B5B, 0x16B61), (0x16B63, 0x16B77), (0x16B7D, 0x16B8F), (0x16E40, 0x16E96),
    (0x16F00, 0x16F4A), (0x16F50, 0x16F50), (0x16F93, 0x16F9F), (0x16FE0, 0x16FE1), (0x16FE3, 0x16FE3), (0x17000, 0x187F7), (0x18800, 0x18CD5), (0x18D00, 0x18D08),
    (0x1AFF0, 0x1AFF3), (0x1AFF5, 0x1AFFB), (0x1AFFD, 0x1AFFE), (0x1B000, 0x1B122), (0x1B132, 0x1B132), (0x1B150, 0x1B152), (0x1B155, 0x1B155), (0x1B164, 0x1B167),
    (0x1B170, 0x1B2FB), (0x1BC00, 0x1BC6A), (0x1BC70, 0x1BC7C), (0x1BC80, 0x1BC88), (0x1BC90, 0x1BC99), (0x1D2C0, 0x1D2D3), (0x1D2E0, 0x1D2F3), (0x1D360, 0x1D378),
    (0x1D400, 0x1D454), (0x1D456, 0x1D49C), (0x1D49E, 0x1D49F), (0x1D4A2, 0x1D4A2), (0x1D4A5, 0x1D4A6), (0x1D4A9, 0x1D4AC), (0x1D4AE, 0x1D4B9), (0x1D4BB, 0x1D4BB),
    (0x1D4BD, 0x1D4C3), (0x1D4C5, 0x1D505), (0x1D507, 0x1D50A), (0x1D50D, 0x1D514), (0x1D516, 0x1D51C), (0x1D51E, 0x1D539), (0x1D53B, 0x1D53E), (0x1D540, 0x1D544),
    (0x1D546, 0x1D546), (0x1D54A, 0x1D550), (0x1D552, 0x1D6A5), (0x1D6A8, 0x1D6C0), (0x1D6C2, 0x1D6DA), (0x1D6DC, 0x1D6FA), (0x1D6FC, 0x1D714), (0x1D716, 0x1D734),
    (0x1D736, 0x1D74E), (0x1D750, 0x1D76E), (0x1D770, 0x1D788), (0x1D78A, 0x1D7A8), (0x1D7AA, 0x1D7C2), (0x1D7C4, 0x1D7CB), (0x1D7CE, 0x1D7FF), (0x1DF00, 0x1DF1E),
    (0x1DF25, 0x1DF2A), (0x1E030, 0x1E06D), (0x1E100, 0x1E12C), (0x1E137, 0x1E13D), (0x1E140, 0x1E149), (0x1E14E, 0x1E14E), (0x1E290, 0x1E2AD), (0x1E2C0, 0x1E2EB),
    (0x1E2F0, 0x1E2F9), (0x1E4D0, 0x1E4EB), (0x1E4F0, 0x1E4F9), (0x1E7E0, 0x1E7E6), (0x1E7E8, 0x1E7EB), (0x1E7ED, 0x1E7EE), (0x1E7F0, 0x1E7FE), (0x1E800, 0x1E8C4),
    (0x1E8C7, 0x1E8CF), (0x1E900, 0x1E943), (0x1E94B, 0x1E94B), (0x1E950, 0x1E959), (0x1EC71, 0x1ECAB), (0x1ECAD, 0x1ECAF), (0x1ECB1, 0x1ECB4), (0x1ED01, 0x1ED2D),
    (0x1ED2F, 0x1ED3D), (0x1EE00, 0x1EE03), (0x1EE05, 0x1EE1F), (0x1EE21, 0x1EE22), (0x1EE24, 0x1EE24), (0x1EE27, 0x1EE27), (0x1EE29, 0x1EE32), (0x1EE34, 0x1EE37),
    (0x1EE39, 0x1EE39), (0x1EE3B, 0x1EE3B), (0x1EE42, 0x1EE42), (0x1EE47, 0x1EE47), (0x1EE49, 0x1EE49), (0x1EE4B, 0x1EE4B), (0x1EE4D, 0x1EE4F), (0x1EE51, 0x1EE52),
    (0x1EE54, 0x1EE54), (0x1EE57, 0x1EE57), (0x1EE59, 0x1EE59), (0x1EE5B, 0x1EE5B), (0x1EE5D, 0x1EE5D), (0x1EE5F, 0x1EE5F), (0x1EE61, 0x1EE62), (0x1EE64, 0x1EE64),
    (0x1EE67, 0x1EE6A), (0x1EE6C, 0x1EE72), (0x1EE74, 0x1EE77), (0x1EE79, 0x1EE7C), (0x1EE7E, 0x1EE7E), (0x1EE80, 0x1EE89), (0x1EE8B, 0x1EE9B), (0x1EEA1, 0x1EEA3),
    (0x1EEA5, 0x1EEA9), (0x1EEAB, 0x1EEBB), (0x1F100, 0x1F10C), (0x1FBF0, 0x1FBF9), (0x20000, 0x2A6DF), (0x2A700, 0x2B739), (0x2B740, 0x2B81D), (0x2B820, 0x2CEA1),
    (0x2CEB0, 0x2EBE0), (0x2EBF0, 0x2EE5D), (0x2F800, 0x2FA1D), (0x30000, 0x3134A), (0x31350, 0x323AF),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gold_words() {
        // (input, Python word_to_ipa output == Wiktionary gold)
        for (w, ipa) in [
            ("з+амок", "zˈamək"),
            ("зам+ок", "zɐmˈok"),
            ("кон+ечно", "kɐnʲˈeʂnə"),
            ("с+олнце", "sˈonʦə"),
            ("пр+аздник", "prˈazʲnʲɪk"),
            ("интеллиг+ентский", "ɪnʲtʲɪlʲɪɡʲˈentskʲɪj"),
            ("отс+утствие", "ɐʦsˈuʦtvʲɪje"),
            ("сег+одня", "sʲɪvˈodʲnʲə"),
            ("вс+ё", "fsʲˈɵ"),
            ("н+а", "nˈa"),
            ("+если", "ˈjesʲlʲɪ"),
            ("жал+еть", "ʐɨlʲˈetʲ"),
            ("дв+адцать", "dvˈaʦːɨtʲ"),
            ("над+еяться", "nɐdʲˈejɪʦə"),
            ("стан+овятся", "stɐnˈovʲəʦə"),
            ("счастл+ивый", "ɕːɪslʲˈivɨj"),
            ("т+ест", "tɛst"),
            ("эн+ергия", "ɛnˈɛrɡʲɪjə"),
            ("+именно", "ˈimʲɪnːə"),
            ("отд+ать", "ɐdːˈatʲ"),
            ("кр+асного", "krˈasnəvə"),
            ("бесср+очный", "bʲɪsːrˈoʧnɨj"),
            ("в", "v"),
            ("ж", "ʂ"),
            ("же", "ʐɨ"),
            ("подж+ечь", "pɐʤʐˈɛʧ"),
            ("с+ердце", "sʲˈerʦə"),
            ("сумасш+едший", "sʊmɐʂˈɛʧʂɨj"),
            ("р+ассказ", "rˈasːkəs"),
            ("п+ятьсот", "pʲˈaʦət"),
            ("Вс+еволод", "fsʲˈevəɫət"),
            ("сдел+ать", "zʲdʲɪɫˈatʲ"),
            ("высший", "vɨʂɨj"),
        ] {
            assert_eq!(word_to_ipa(w, None), ipa, "{w}");
        }
    }

    #[test]
    fn mono_marking() {
        assert_eq!(word_to_ipa("д+ом", None), "dom");
        assert_eq!(word_to_ipa("д+ом", Some(true)), "dˈom");
        assert_eq!(word_to_ipa("н+а", Some(false)), "na");
        assert_eq!(word_to_ipa("", None), "");
        assert_eq!(word_to_ipa("+", None), "");
    }

    #[test]
    fn sentence_without_lexicon() {
        let ru = Russian::load(None);
        assert_eq!(ru.phonemize("В 1945 году было 25 человек, 3,5 %."), ru.phonemize("В тысяча девятьсот сорок пятом году было двадцать пять человек, три целых пять десятых процентов."));
        // dashes, quotes, brackets dropped; … -> '.'; Latin dropped without a callback
        assert_eq!(ru.phonemize("Привет, как дела? — «Хорошо»… (да) hello"), "prʲˈivʲɪt, kak dʲˈeɫə? xɐrˈoʂə. da.");
        assert_eq!(ru.phonemize(""), "");
        let en = Russian::load(None).with_latin(|s| s.to_uppercase());
        assert!(en.phonemize("это test").ends_with("TEST."));
        let rep = ru.stress_report("Он повесил замок");
        assert_eq!(rep.len(), 3);
        assert_eq!(rep[0].1, "mono");
        assert_eq!(rep[1].1, "ood-suffix");
    }

    #[test]
    fn numbers() {
        assert_eq!(expand_numbers("9 мая"), "девятое мая");
        assert_eq!(expand_numbers("2000 году"), "двухтысячном году");
        assert_eq!(expand_numbers("21 книга"), "двадцать одна книга");
        assert_eq!(expand_numbers("1 000 000"), "один миллион");
        assert_eq!(expand_numbers("5%"), "пять процентов");
        assert_eq!(expand_numbers("1,25"), "одна целая двадцать пять сотых");
    }

    #[test]
    fn python_classes() {
        assert_eq!(py_digit('٣'), Some(3));
        assert_eq!(py_digit('a'), None);
        assert!(py_word('ж') && py_word('_') && !py_word(' ') && !py_word('-'));
        assert_eq!(normalize_text("tsa ts"), "tsa ʦ.");
    }
}
