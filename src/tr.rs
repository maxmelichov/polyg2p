//! Turkish grapheme-to-phoneme, espeak-style notation.
//!
//! Turkish spelling is close to phonemic, so this is rules only. The symbol
//! conventions follow espeak-ng's `tr` voice (its allophones are written out):
//!
//! * `i u o e ü` are written lax (`ɪ ʊ ɔ ɛ ø`) in a closed syllable and in the
//!   last syllable of the word; `e` becomes `æ` before a coda nasal or `l`;
//!   `ö` is always `œ`, `ı` is `ɯ`, `a` is `a`.
//! * `l` is dark `ɫ` next to back vowels, `r` is a tap `ɾ` between vowels,
//!   `g` is palatal `ɟ` next to front vowels (espeak writes `k` plain).
//! * `ğ` lengthens a preceding back vowel (`dağ` → `dˈaː`, `ağa` → `aː`),
//!   is `j` after `e`/`i`, and `ɯɯ` after `ı` (espeak prints a long `ɯ` that way).
//! * Stress is final by default, marked right before the vowel
//!   (`kitˈap`). Pre-stressing suffixes (`-(i)yor`, negative `-me/-ma`, `-ken`,
//!   `-(y)le`, copular `-dir`, `-ce`), unstressed clitics (`de`, `mi`, `ve`, …)
//!   and a short list of non-finally stressed words (place names, adverbs,
//!   loans) are handled.
//!
//! Limitation: the token arrives lower-cased by Rust's `to_lowercase`, which
//! maps `I` → `i` instead of Turkish `ı`. All-caps input (`ISPARTA`) is therefore
//! read with dotted `i`; mixed-case text is unaffected.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "virgül";
/// Letter names a..z (phonemes).
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "bɛ", "ʤɛ", "dɛ", "ɛ", "fɛ", "ɟɛ", "hɛ", "i", "ʒɛ", "kɛ", "lɛ", "mɛ", "nɛ", "o", "pɛ", "kø", "ɾɛ", "sɛ",
    "tɛ", "u", "vɛ", "ʧiftvɛ", "iks", "jɛ", "zɛ",
]);

/// Clitics and conjunctions espeak reads without a stress mark.
const UNSTRESSED: &[&str] = &["ve", "ki", "mı", "mi", "mu", "mü", "da", "de", "ta", "te"];
/// Words with a secondary stress only (espeak: `ˌilɛ`).
const SECONDARY: &[&str] = &["ile", "veya", "yoksa"];

/// Words stressed on a non-final syllable: (word, 0-based index of the stressed vowel).
/// Place names (Sezer stress), adverbs, question words, kinship terms, loans.
const STRESS: &[(&str, u8)] = &[
    ("değil", 0), ("ama", 0), ("fakat", 0), ("ancak", 0), ("yalnız", 0), ("evet", 0), ("hayır", 0), ("şimdi", 0), ("sonra", 0),
    ("belki", 0), ("yani", 0), ("hani", 0), ("sanki", 0), ("keşke", 0), ("acaba", 0), ("asla", 0), ("böyle", 0),
    ("şöyle", 0), ("öyle", 0), ("nerede", 0), ("nereye", 0), ("nereden", 0), ("nasıl", 0), ("niçin", 0),
    ("neden", 0), ("nerde", 0), ("burada", 0), ("burda", 0), ("orada", 0), ("orda", 0), ("şurada", 0),
    ("buraya", 0), ("oraya", 0), ("buradan", 0), ("oradan", 0), ("hemen", 0), ("henüz", 0), ("zaten", 0),
    ("hatta", 0), ("çünkü", 0), ("yine", 0), ("gene", 0), ("daha", 0), ("artık", 0), ("ayrıca", 1), ("ise", 0), ("bile", 0), ("önce", 0), ("erken", 0), ("yoksa", 0),
    ("firma", 1), ("hurma", 1), ("forma", 1), ("lokma", 1), ("dolma", 1), ("sarma", 1), ("karma", 1), ("damla", 1),
    ("tabii", 0), ("bazen", 0), ("bazı", 0), ("hiç", 0), ("hangi", 0), ("kimi", 0), ("işte", 0), ("belli", 0),
    ("anne", 0), ("abla", 0), ("abi", 0), ("ağabey", 0), ("teyze", 0), ("amca", 0), ("dayı", 0), ("hala", 0),
    ("ankara", 0), ("izmir", 0), ("bursa", 0), ("konya", 0), ("samsun", 0), ("adana", 0), ("mersin", 0),
    ("antalya", 1), ("istanbul", 1), ("türkiye", 0), ("avrupa", 1), ("amerika", 1), ("almanya", 1),
    ("fransa", 0), ("italya", 1), ("ingiltere", 2), ("rusya", 0), ("suriye", 1), ("irak", 1), ("iran", 1),
    ("londra", 0), ("paris", 0), ("berlin", 0), ("moskova", 0), ("afrika", 1), ("asya", 0), ("kıbrıs", 0),
    ("trabzon", 0), ("erzurum", 0), ("diyarbakır", 2), ("eskişehir", 2), ("kayseri", 0), ("gaziantep", 2),
    ("sivas", 0), ("edirne", 0), ("van", 0), ("bodrum", 0), ("alanya", 1), ("marmara", 0), ("anadolu", 1),
    ("masa", 0), ("radyo", 0), ("banka", 0), ("posta", 0), ("gazete", 1), ("sinema", 1), ("fabrika", 1),
    ("lokanta", 1), ("tiyatro", 1), ("politika", 2), ("kamera", 0), ("opera", 0), ("limon", 0),
    ("portakal", 1), ("domates", 1), ("salata", 1), ("makarna", 1), ("pasta", 0), ("otobüs", 1),
    ("taksi", 0), ("metro", 0), ("bira", 0), ("kahve", 0), ("çay", 0), ("televizyon", 2), ("telefon", 1),
    ("program", 0), ("problem", 0), ("sistem", 0), ("müzik", 0), ("doktor", 0), ("profesör", 1),
    ("kilo", 0), ("litre", 0), ("saniye", 0), ("dakika", 1), ("hafta", 0), ("cumartesi", 2), ("pazartesi", 2),
    ("fasulye", 1), ("tarihçe", 1), ("tabanca", 1), ("sadece", 0), ("hep", 0), ("hepsi", 0), ("yahu", 0),
    ("hadi", 0), ("haydi", 0), ("lütfen", 0), ("peki", 0), ("tamam", 0), ("merhaba", 1), ("hoşça", 0),
    ("birlikte", 1), ("beraber", 1), ("mutlaka", 1), ("galiba", 1), ("herhalde", 1), ("maalesef", 2),
    ("nihayet", 1), ("elbette", 1), ("mesela", 1), ("örneğin", 0), ("aslında", 0), ("özellikle", 1),
    ("üstelik", 0), ("birden", 0), ("yeniden", 0), ("ayrı", 0), ("tekrar", 0), ("hâlâ", 0), ("hala", 0),
    ("dün", 0), ("bugün", 0), ("yarın", 0), ("demin", 0), ("şöyle", 0), ("nasılsın", 0), ("bence", 0),
    ("sence", 0), ("türkçe", 0), ("ingilizce", 2), ("almanca", 1), ("fransızca", 1), ("arapça", 0),
    ("rusça", 0), ("açıkça", 1), ("iyice", 1), ("gerçekten", 1), ("kesinlikle", 1), ("tamamen", 1),
];

/// Suffixes a lexicon stem may carry without losing its own stress (`ˈankaɾada`).
const CASE_SUFFIXES: &[&str] = &[
    "da", "de", "ta", "te", "dan", "den", "tan", "ten", "ya", "ye", "yı", "yi", "yu", "yü", "nın", "nin", "nun", "nün",
    "ın", "in", "un", "ün", "nda", "nde", "ndan", "nden", "na", "ne", "nı", "ni", "nu", "nü", "lar", "ler", "ları",
    "leri", "larda", "lerde", "lardan", "lerden", "lara", "lere", "ların", "lerin", "lı", "li", "lu", "lü", "lılar",
    "liler", "m", "n", "sı", "si", "su", "sü", "ım", "im", "um", "üm", "mız", "miz", "muz", "müz", "yla", "yle",
    "dır", "dir", "dur", "dür", "tır", "tir", "tur", "tür", "ca", "ce", "ki", "daki", "deki", "mı", "mi",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Vowel,
    Length,
    Cons,
}

#[derive(Clone, Copy)]
struct Ph {
    s: &'static str,
    kind: Kind,
    /// a back vowel (for l)
    back: bool,
    /// a long vowel that stands for two written vowels (ağa -> aː)
    merged: bool,
    /// a long vowel of a loan (â î û): never laxed
    tense: bool,
}

const fn v(s: &'static str, back: bool) -> Ph {
    Ph { s, kind: Kind::Vowel, back, merged: false, tense: false }
}
const fn c(s: &'static str) -> Ph {
    Ph { s, kind: Kind::Cons, back: false, merged: false, tense: false }
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'a' | 'e' | 'ı' | 'i' | 'o' | 'ö' | 'u' | 'ü' | 'â' | 'î' | 'û' | 'ô')
}
fn is_back(ch: char) -> bool {
    matches!(ch, 'a' | 'ı' | 'o' | 'u' | 'â' | 'û' | 'ô')
}
fn is_front(ch: char) -> bool {
    matches!(ch, 'e' | 'i' | 'ö' | 'ü' | 'î' | 'â')
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    // hyphenated compounds: each part on its own
    if w.contains('-') && w.chars().any(|c| c.is_alphabetic()) {
        let parts: Vec<String> = w.split('-').map(word).filter(|p| !p.is_empty()).collect();
        return parts.join(" ");
    }
    // apostrophe: proper noun + suffix (Ankara'da) — the stem keeps its stress
    let (stem, suffix) = match w.split_once('\'') {
        Some((a, b)) if !a.is_empty() && !b.is_empty() => (a, b),
        _ => (w, ""),
    };
    let stem: String = stem.chars().filter(|c| c.is_alphabetic()).collect();
    let suffix: String = suffix.chars().filter(|c| c.is_alphabetic()).collect();
    let lw = format!("{stem}{suffix}");
    if lw.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = lw.chars().collect();
    let nv = chars.iter().filter(|&&c| is_vowel(c)).count();
    if nv == 0 {
        // consonant-only token: read as letter names
        return spell_consonants(&chars);
    }
    let stress = if UNSTRESSED.contains(&lw.as_str()) {
        Stress::None
    } else if SECONDARY.contains(&lw.as_str()) {
        Stress::Secondary(0)
    } else {
        let idx = match lexical_stress(&stem, nv) {
            Some(i) if !suffix.is_empty() => i,
            _ => stress_index(&lw, nv),
        };
        Stress::Primary(idx)
    };
    realize(&chars, stress)
}

#[derive(Clone, Copy)]
enum Stress {
    Primary(usize),
    Secondary(usize),
    None,
}

/// Stress of a lexicon word (also used as the stem before an apostrophe).
fn lexical_stress(stem: &str, nv: usize) -> Option<usize> {
    STRESS.iter().find(|&&(w, i)| w == stem && (i as usize) < nv).map(|&(_, i)| i as usize)
}

/// 0-based index (among vowels) of the stressed syllable.
fn stress_index(lw: &str, nv: usize) -> usize {
    if let Some(i) = lexical_stress(lw, nv) {
        return i;
    }
    // lexicon stem + case/possessive suffix (ankarada, annem)
    for &(w, i) in STRESS {
        if lw.len() > w.len() && lw.starts_with(w) && w.chars().count() >= 4 {
            let rest = &lw[w.len()..];
            if CASE_SUFFIXES.contains(&rest) || split_suffixes(rest) {
                return i as usize;
            }
        }
    }
    // pre-stressing suffixes: the stress falls on the syllable before them
    if let Some(cut) = prestress_cut(lw) {
        // number of vowels before byte position `cut`
        let before = lw[..cut].chars().filter(|&c| is_vowel(c)).count();
        if before >= 1 {
            return before - 1;
        }
    }
    nv - 1
}

/// Can `rest` be read as a chain of two common suffixes (`larda`, `sında`)?
fn split_suffixes(rest: &str) -> bool {
    for s in CASE_SUFFIXES {
        if let Some(r) = rest.strip_prefix(s) {
            if CASE_SUFFIXES.contains(&r) {
                return true;
            }
        }
    }
    false
}

/// Byte offset where a pre-stressing suffix chain begins, if any.
fn prestress_cut(lw: &str) -> Option<usize> {
    // -(i)yor: stress on the vowel before "yor"
    if let Some(p) = lw.find("yor") {
        if p > 0 && lw[..p].ends_with(|c: char| matches!(c, 'ı' | 'i' | 'u' | 'ü')) {
            // negative -mıyor: stress before the negation
            let pre = &lw[..p];
            let neg = ["mı", "mi", "mu", "mü"].iter().find_map(|m| pre.strip_suffix(m).map(|s| s.len()));
            if let Some(q) = neg {
                if lw[..q].chars().any(is_vowel) {
                    return Some(q);
                }
            }
            // the vowel before yor is the stressed one: cut after it
            return Some(p);
        }
    }
    // negation -ma/-me followed by verbal suffixes, or bare at the end
    const NEG_TAILS: &[&str] = &[
        "", "dı", "di", "du", "dü", "dık", "dik", "duk", "dük", "dım", "dim", "dum", "düm", "dın", "din", "dun", "dün",
        "dılar", "diler", "dular", "düler", "dınız", "diniz", "dunuz", "dünüz", "dan", "den", "mak", "mek",
        "mış", "miş", "muş", "müş", "mışlar", "mişler", "muşlar", "müşler", "yan",
        "yen", "yacak", "yecek", "yacağım", "yeceğim", "yacağız", "yeceğiz", "yacaksın", "yeceksin", "yacaklar",
        "yecekler", "z", "zsın", "zsin", "zsınız", "zsiniz", "zlar", "zler", "yız", "yiz", "yınız",
        "yiniz", "sın", "sin", "sınlar", "sinler", "m", "yınca", "yince", "yıp", "yip", "dıkça",
        "dikçe", "sa", "se", "sam", "sem", "san", "sen", "sak", "sek", "ksızın", "ksizin",
        "dığı", "diği", "dığını", "diğini", "dığında", "diğinde", "yarak", "yerek", "yasın", "yesin",
        "yayım", "yeyim", "yalım", "yelim", "yın", "yin",
    ];
    for m in ["ma", "me"] {
        for (p, _) in lw.rmatch_indices(m) {
            let stem = &lw[..p];
            let tail = &lw[p + 2..];
            // a verb stem: at least one vowel, ends in a consonant
            if !stem.chars().any(is_vowel) || stem.chars().count() < 2 || stem.ends_with(is_vowel) {
                continue;
            }
            if tail.is_empty() {
                if stem.chars().count() >= 3 {
                    return Some(p);
                }
            } else if NEG_TAILS.contains(&tail) {
                return Some(p);
            }
        }
    }
    // copular / aorist personal endings after a vowel: iyiyim, hastayız, gelirsiniz, güzelsin
    for (suf, min) in [
        ("yim", 2), ("yım", 2), ("yum", 2), ("yüm", 2), ("yiz", 2), ("yız", 2), ("yuz", 2), ("yüz", 2),
        ("siniz", 2), ("sınız", 2), ("sunuz", 2), ("sünüz", 2), ("sin", 2), ("sın", 2), ("sun", 2), ("sün", 2),
    ] {
        if let Some(s) = lw.strip_suffix(suf) {
            if s.chars().filter(|&c| is_vowel(c)).count() >= min {
                return Some(s.len());
            }
        }
    }
    // -ken (while): gelirken, yaparken, çocukken
    if let Some(s) = lw.strip_suffix("ken") {
        if s.chars().filter(|&c| is_vowel(c)).count() >= 2 {
            return Some(s.len());
        }
    }
    // -(y)le / -(y)la instrumental (arabayla, onunla, seninle, benimle)
    for (suf, min) in [("yla", 1), ("yle", 1), ("nla", 2), ("nle", 2), ("mla", 2), ("mle", 2)] {
        if let Some(s) = lw.strip_suffix(suf) {
            if s.chars().filter(|&c| is_vowel(c)).count() >= min {
                return Some(s.len() + 1);
            }
        }
    }
    // copula -dir/-tir after a stem of two or more syllables (güzeldir, yoktur)
    for suf in ["dır", "dir", "dur", "dür", "tır", "tir", "tur", "tür"] {
        if let Some(s) = lw.strip_suffix(suf) {
            if s.chars().filter(|&c| is_vowel(c)).count() >= 2 || s == "var" || s == "yok" {
                return Some(s.len());
            }
        }
    }
    None
}

/// Letter names for a vowel-less token (consonant abbreviations).
fn spell_consonants(chars: &[char]) -> String {
    let mut out = String::new();
    let n = chars.len();
    for (k, &ch) in chars.iter().enumerate() {
        let name = match ch {
            'b' => "bɛ", 'c' => "ʤɛ", 'ç' => "ʧɛ", 'd' => "dɛ", 'f' => "fɛ", 'g' => "ɟɛ", 'ğ' => "ɟɛ", 'h' => "hɛ",
            'j' => "ʒɛ", 'k' => "kɛ", 'l' => "lɛ", 'm' => "mɛ", 'n' => "nɛ", 'p' => "pɛ", 'r' => "ɾɛ", 's' => "sɛ",
            'ş' => "ʃɛ", 't' => "tɛ", 'v' => "vɛ", 'y' => "jɛ", 'z' => "zɛ", 'x' => "iks", 'q' => "kø", 'w' => "ʧiftvɛ",
            _ => continue,
        };
        let mark = if k + 1 == n { "ˈ" } else { "ˌ" };
        let pos = name.find(|c: char| "aeiouɛøɔɪʊ".contains(c)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    out
}

/// Letters -> phones with espeak's allophones and stress marks.
fn realize(chars: &[char], stress: Stress) -> String {
    let n = chars.len();
    let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { chars[i as usize] } };
    let mut ph: Vec<Ph> = Vec::with_capacity(n + 2);
    let mut i = 0usize;
    while i < n {
        let ch = chars[i];
        let nx = at(i as isize + 1);
        let pv = at(i as isize - 1);
        // doubled stop letters: one phone + length (etti -> etːɪ); espeak keeps ll/nn/ss double
        let gem = nx == ch && matches!(ch, 'b' | 'c' | 'ç' | 'd' | 'g' | 'k' | 'p' | 't');
        let before = ph.len();
        match ch {
            'a' => ph.push(v("a", true)),
            'â' => {
                // circumflex: long after k/g (kâğıt, rüzgâr) and in Arabic/Persian loans
                ph.push(Ph { tense: true, ..v("a", true) });
                ph.push(Ph { s: "ː", kind: Kind::Length, back: true, merged: false, tense: false });
            }
            'e' => ph.push(v("e", false)),
            'ı' => ph.push(v("ɯ", true)),
            'i' => ph.push(v("i", false)),
            'î' => {
                ph.push(Ph { tense: true, ..v("i", false) });
                ph.push(Ph { s: "ː", kind: Kind::Length, back: false, merged: false, tense: false });
            }
            'o' | 'ô' => ph.push(v("o", true)),
            'ö' => ph.push(v("œ", false)),
            'u' => ph.push(v("u", true)),
            'û' => {
                ph.push(Ph { tense: true, ..v("u", true) });
                ph.push(Ph { s: "ː", kind: Kind::Length, back: true, merged: false, tense: false });
            }
            'ü' => ph.push(v("y", false)),
            'ğ' => {
                if ph.last().map_or(false, |p| p.kind == Kind::Length) {
                    // already long (kâğıt): nothing to add
                } else if is_vowel(pv) {
                    if matches!(pv, 'e' | 'i' | 'î') {
                        ph.push(c("j"));
                    } else if pv == 'ı' {
                        ph.push(Ph { s: "ɯ", kind: Kind::Length, back: true, merged: false, tense: false });
                    } else {
                        let merged = nx == pv;
                        ph.push(Ph { s: "ː", kind: Kind::Length, back: is_back(pv), merged, tense: false });
                        // ağa, oğo, uğu, öğö, üğü: one long vowel
                        if merged {
                            i += 1;
                        }
                    }
                } else if i == 0 || !is_vowel(nx) {
                    // never word-initial in Turkish; foreign spelling
                    ph.push(c("ɡ"));
                }
            }
            'b' => ph.push(c("b")),
            'c' => ph.push(c("ʤ")),
            'ç' => ph.push(c("ʧ")),
            'd' => ph.push(c("d")),
            'f' => ph.push(c("f")),
            'g' => {
                // palatal before a front vowel, or after one in a coda (espeak: J)
                let front_next = is_front(nx);
                let front_prev = is_front(pv) && !is_vowel(nx);
                ph.push(c(if front_next || front_prev { "ɟ" } else { "ɡ" }));
            }
            'h' => ph.push(c("h")),
            'j' => ph.push(c("ʒ")),
            'k' => ph.push(c("k")),
            'q' => ph.push(c("k")),
            'l' => ph.push(c("l")), // dark/clear decided below
            'm' => ph.push(c("m")),
            'n' => ph.push(c("n")),
            'p' => ph.push(c("p")),
            'r' => ph.push(c("r")), // tap decided below
            's' => ph.push(c("s")),
            'ş' => ph.push(c("ʃ")),
            't' => ph.push(c("t")),
            'v' => ph.push(c("v")),
            'w' => ph.push(c("v")),
            'x' => {
                ph.push(c("k"));
                ph.push(c("s"));
            }
            'y' => ph.push(c("j")),
            'z' => ph.push(c("z")),
            _ => {}
        }
        if gem && ph.len() > before {
            ph.push(Ph { s: "ː", kind: Kind::Length, back: false, merged: false, tense: false });
            i += 1;
        }
        i += 1;
    }
    let m = ph.len();
    // vowel-ish for r/l: vowels and length
    let vowelish = |k: usize| k < m && ph[k].kind != Kind::Cons;
    let is_v = |k: usize| k < m && ph[k].kind == Kind::Vowel;
    // first index at or after k that is not a length mark
    let skip_len = |mut k: usize| {
        while k < m && ph[k].kind == Kind::Length {
            k += 1;
        }
        k
    };
    // index of the last vowel
    let last_v = ph.iter().rposition(|p| p.kind == Kind::Vowel).unwrap_or(0);
    let mut out: Vec<&'static str> = Vec::with_capacity(m);
    let mut nk = 0usize; // vowel counter
    for k in 0..m {
        let p = ph[k];
        let s: &'static str = match p.kind {
            Kind::Length => p.s,
            Kind::Vowel => {
                // closed syllable: next phone and the one after are both non-vowels (end counts)
                // (length is part of the vowel, not a following phone: doːɾʊ)
                let j = skip_len(k + 1);
                let j2 = skip_len(j + 1);
                let closed = !is_v(j) && !is_v(j2);
                let final_v = k == last_v;
                let next = ph.get(j).map(|q| q.s).unwrap_or("#");
                let coda_sonorant = matches!(next, "m" | "n" | "l") && !is_v(j2);
                match p.s {
                    _ if p.tense => p.s,
                    "e" if coda_sonorant => "æ",
                    "e" if closed || final_v => "ɛ",
                    "i" if closed || final_v => "ɪ",
                    "u" if closed || final_v => "ʊ",
                    "o" if closed || final_v => "ɔ",
                    "y" if closed || final_v => "ø",
                    other => other,
                }
            }
            Kind::Cons => match p.s {
                "l" => {
                    // dark before a back vowel, or after one when no vowel follows
                    let next_back = k + 1 < m && ph[k + 1].kind == Kind::Vowel && ph[k + 1].back;
                    let prev_back = k > 0 && vowelish(k - 1) && ph[k - 1].back && !is_v(k + 1);
                    if next_back || prev_back { "ɫ" } else { "l" }
                }
                "r" => {
                    if k > 0 && vowelish(k - 1) && is_v(k + 1) { "ɾ" } else { "r" }
                }
                other => other,
            },
        };
        if p.kind == Kind::Vowel {
            // a merged long vowel stands for two written vowels
            let span = if ph.get(k + 1).map_or(false, |q| q.merged) { 2 } else { 1 };
            let hit = |i: usize| i >= nk && i < nk + span;
            let mark = match stress {
                Stress::None => "",
                Stress::Secondary(i) if hit(i) => "ˌ",
                Stress::Primary(i) if hit(i) => "ˈ",
                _ => "",
            };
            // espeak writes the mark right before the vowel (kitˈap, ˈankaɾa)
            out.push(mark);
            nk += span;
        }
        out.push(s);
    }
    out.concat()
}

/// Sentence pass: nothing cross-word in Turkish beyond the clitics handled in `word`.
pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 10] = ["sıfır", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz"];
const TENS: [&str; 10] = ["", "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen", "doksan"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = n / 100;
    let r = n % 100;
    if h > 0 {
        if h > 1 {
            out.push(ONES[h as usize].into());
        }
        out.push("yüz".into());
    }
    if r >= 10 {
        out.push(TENS[(r / 10) as usize].into());
    }
    if r % 10 > 0 || (n == 0) {
        out.push(ONES[(r % 10) as usize].into());
    }
}

/// Cardinal number words.
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["sıfır".into()];
    }
    let mut out = Vec::new();
    const SCALES: [(u128, &str); 6] = [
        (1_000_000_000_000_000_000, "kentilyon"),
        (1_000_000_000_000_000, "katrilyon"),
        (1_000_000_000_000, "trilyon"),
        (1_000_000_000, "milyar"),
        (1_000_000, "milyon"),
        (1_000, "bin"),
    ];
    let mut rest = n;
    for (val, name) in SCALES {
        let q = rest / val;
        if q > 0 {
            // "bin" never takes "bir"; the others do
            if !(q == 1 && val == 1_000) {
                below_1000(q, &mut out);
            }
            out.push(name.into());
            rest %= val;
        }
    }
    if rest > 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "yüzde",
        '$' => "dolar",
        '€' => "avro",
        '£' => "sterlin",
        '&' => "ve",
        '+' => "artı",
        '@' => "et",
        '°' => "derece",
        '=' => "eşittir",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn basics() {
        for (w, p) in [
            ("kitap", "kitˈap"),
            ("kadar", "kadˈar"),
            ("çok", "ʧˈɔk"),
            ("için", "iʧˈɪn"),
            ("bir", "bˈɪr"),
            ("gibi", "ɟibˈɪ"),
            ("olarak", "oɫaɾˈak"),
            ("gelmek", "ɟælmˈɛk"),
            ("ben", "bˈæn"),
            ("her", "hˈɛr"),
            ("evler", "ɛvlˈɛr"),
            ("büyük", "byjˈøk"),
            ("göre", "ɟœɾˈɛ"),
            ("üniversite", "ynivɛrsitˈɛ"),
            ("kalp", "kˈaɫp"),
            ("ilk", "ˈɪlk"),
            ("lale", "ɫalˈɛ"),
            ("sonra", "sˈɔnra"),
            ("yıl", "jˈɯɫ"),
            ("şey", "ʃˈɛj"),
            ("cumhuriyet", "ʤʊmhuɾijˈɛt"),
            ("öğretmen", "œːɾɛtmˈæn"),
            ("saat", "saˈat"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn yumusak_g() {
        for (w, p) in [
            ("dağ", "dˈaː"),
            ("dağlar", "daːɫˈar"),
            ("ağaç", "ˈaːʧ"),
            ("soğuk", "soːˈʊk"),
            ("değil", "dˈejɪl"),
            ("eğer", "ejˈɛr"),
            ("çığ", "ʧˈɯɯ"),
            ("doğru", "doːɾˈʊ"),
            ("yağı", "jaːˈɯ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn stress() {
        for (w, p) in [
            ("geliyor", "ɟelˈijɔr"),
            ("gelmiyor", "ɟˈælmijɔr"),
            ("gelmedi", "ɟˈælmedɪ"),
            ("yapma", "jˈapma"),
            ("elma", "ælmˈa"),
            ("gelirken", "ɟelˈɪrkæn"),
            ("güzeldir", "ɟyzˈældɪr"),
            ("ankara", "ˈankaɾa"),
            ("ankara'da", "ˈankaɾada"),
            ("istanbul", "ɪstˈanbʊɫ"),
            ("arabayla", "aɾabˈajɫa"),
            ("ama", "ˈama"),
            ("ve", "vɛ"),
            ("mi", "mɪ"),
            ("ile", "ˌilɛ"),
            ("kâğıt", "kaːˈɯt"),
            ("iyiyim", "ijˈijɪm"),
            ("gelirsiniz", "ɟelˈɪrsinɪz"),
            ("olmadan", "ˈɔɫmadan"),
            ("yapmamak", "jˈapmamak"),
            ("ise", "ˈisɛ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn length_and_geminates() {
        for (w, p) in [
            ("etti", "etːˈɪ"),
            ("hakkında", "hakːɯndˈa"),
            ("milli", "mɪllˈɪ"),
            ("millî", "mɪllˈiː"),
            ("hikâye", "hikaːjˈɛ"),
            ("tv", "tˌɛvˈɛ"),
            ("türk-islam", "tˈørk ɪsɫˈam"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), vec!["sıfır"]);
        assert_eq!(cardinal(1000), vec!["bin"]);
        assert_eq!(cardinal(2024), vec!["iki", "bin", "yirmi", "dört"]);
        assert_eq!(cardinal(100), vec!["yüz"]);
        assert_eq!(cardinal(1_000_000), vec!["bir", "milyon"]);
        assert_eq!(cardinal(315), vec!["üç", "yüz", "on", "beş"]);
    }
}
