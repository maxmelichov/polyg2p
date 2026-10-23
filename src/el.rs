//! Modern Greek grapheme-to-phoneme rules, espeak-style notation.
//!
//! Greek spelling is phonemic once the written accent (tonos) is known, so this
//! is rules only: vowel digraphs (αι ει οι υι ου, αυ/ευ with a following v/f),
//! the voiced stops μπ/ντ/γκ, palatalization of κ/χ/γ before front vowels,
//! σ-voicing, synizesis (unstressed ι + vowel read as a glide: παιδιά, μια,
//! ήλιος) and the stress pattern espeak uses (secondary stress on every second
//! syllable away from the primary). The sentence pass handles the sandhi of the
//! clitics τον/την/δεν (τον πατέρα → tom batˈera) and of word-final /s/.
//!
//! Notation (espeak `el`): vowels a e i o u; ɪ for an unstressed ι/η/υ after a
//! vowel (Μάιος mˈaɪos); c ç j for palatal κ χ γ (j is used for both γ before a
//! front vowel and the glide); ʎ ɲ for λ/ν + glide; ts dz; b d ɡ (mb nd ŋɡ
//! word-internally). ˈ/ˌ go right before the stressed vowel.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "κόμμα";
/// Latin letter names as Greek speakers say them (for Latin acronyms in Greek text).
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "ei", "bi", "si", "di", "i", "ef", "dzi", "eits", "ai", "dzei", "kei", "el", "em", "en", "ou", "pi", "kju", "ar",
    "es", "ti", "ju", "vi", "dablju", "eks", "ɣuai", "zet",
]);

/// Clitics and particles espeak reads without any stress mark.
const UNSTRESSED: &[&str] = &[
    "ο", "η", "το", "οι", "τα", "καί", "του", "της", "τη", "την", "τον", "των", "τους", "τις", "να", "θα", "και", "κι", "με",
    "σε", "μου", "σου", "μας", "σας", "που", "αν", "ως", "εκ", "ας", "στο", "στη", "στην", "στον", "στα", "στου",
    "στης", "στις", "στους", "στων", "σ", "μ", "τ", "ν", "δ", "κ", "θ", "στ", "απ", "κατ", "μετ", "αφ", "υπ", "παρ",
    "εφ", "αντ", "γι",
];
/// Auxiliaries and a few function words espeak marks with secondary stress only.
const WEAK: &[&str] = &[
    "είναι", "είμαι", "είσαι", "είμαστε", "είστε", "ήμουν", "ήσουν", "ήταν", "ήμασταν", "ήσασταν", "έχω", "έχεις",
    "έχει", "έχουμε", "έχετε", "έχουν", "είχα", "είχες", "είχε", "είχαμε", "είχατε", "είχαν", "από", "αλλά", "μέσα", "μαζί", "επί", "παρά", "εάν", "διά",
];

/// Common words whose initial δια- keeps the glide (learned δια- words have a hiatus).
const GLIDE_DIA: &[&str] = &["διαβα", "διαβο", "διαφορ", "διαμαντ", "διαλεγ", "διανομ"];

/// A letter after normalization: base letter plus accent / diaeresis flags.
#[derive(Clone, Copy, Debug)]
struct L {
    b: char,
    acc: bool,
    dia: bool,
}

/// Fold the Greek Extended (polytonic) block to base letter + accent + diaeresis.
fn polytonic(u: u32) -> Option<(char, bool, bool)> {
    let acc_low = |u: u32| (u & 7) >= 2; // ἂ ἃ ἄ ἅ ἆ ἇ carry an accent, ἀ ἁ do not
    Some(match u {
        0x1F00..=0x1F0F => ('α', acc_low(u), false),
        0x1F10..=0x1F1D => ('ε', acc_low(u), false),
        0x1F20..=0x1F2F => ('η', acc_low(u), false),
        0x1F30..=0x1F3F => ('ι', acc_low(u), false),
        0x1F40..=0x1F4D => ('ο', acc_low(u), false),
        0x1F50..=0x1F5F => ('υ', acc_low(u), false),
        0x1F60..=0x1F6F => ('ω', acc_low(u), false),
        0x1F70 | 0x1F71 => ('α', true, false),
        0x1F72 | 0x1F73 => ('ε', true, false),
        0x1F74 | 0x1F75 => ('η', true, false),
        0x1F76 | 0x1F77 => ('ι', true, false),
        0x1F78 | 0x1F79 => ('ο', true, false),
        0x1F7A | 0x1F7B => ('υ', true, false),
        0x1F7C | 0x1F7D => ('ω', true, false),
        0x1F80..=0x1F8F => ('α', acc_low(u), false),
        0x1F90..=0x1F9F => ('η', acc_low(u), false),
        0x1FA0..=0x1FAF => ('ω', acc_low(u), false),
        0x1FB0..=0x1FBC => ('α', matches!(u, 0x1FB2 | 0x1FB4 | 0x1FB6 | 0x1FB7 | 0x1FBA | 0x1FBB), false),
        0x1FC2..=0x1FC7 => ('η', matches!(u, 0x1FC2 | 0x1FC4 | 0x1FC6 | 0x1FC7), false),
        0x1FC8 | 0x1FC9 => ('ε', true, false),
        0x1FCA | 0x1FCB => ('η', true, false),
        0x1FCC => ('η', false, false),
        0x1FD0..=0x1FDB => ('ι', matches!(u, 0x1FD2 | 0x1FD3 | 0x1FD6 | 0x1FD7 | 0x1FDA | 0x1FDB), matches!(u, 0x1FD2 | 0x1FD3 | 0x1FD7)),
        0x1FE4 | 0x1FE5 | 0x1FEC => ('ρ', false, false),
        0x1FE0..=0x1FEB => ('υ', matches!(u, 0x1FE2 | 0x1FE3 | 0x1FE6 | 0x1FE7 | 0x1FEA | 0x1FEB), matches!(u, 0x1FE2 | 0x1FE3 | 0x1FE7)),
        0x1FF2..=0x1FF7 => ('ω', matches!(u, 0x1FF2 | 0x1FF4 | 0x1FF6 | 0x1FF7), false),
        0x1FF8 | 0x1FF9 => ('ο', true, false),
        0x1FFA | 0x1FFB => ('ω', true, false),
        0x1FFC => ('ω', false, false),
        _ => return None,
    })
}

/// Normalize a word to base Greek letters with accent/diaeresis flags. Latin
/// letters are transliterated so that foreign words at least read aloud.
fn letters(w: &str) -> Vec<L> {
    let mut out: Vec<L> = Vec::with_capacity(w.len());
    macro_rules! push {
        ($b:expr, $acc:expr, $dia:expr) => {
            out.push(L { b: $b, acc: $acc, dia: $dia })
        };
    }
    let mut pending_latin: Vec<char> = Vec::new();
    for c in w.chars() {
        let u = c as u32;
        match c {
            'α'..='ω' => {
                if c == 'ς' {
                    push!('σ', false, false)
                } else {
                    push!(c, false, false)
                }
            }
            'ά' => push!('α', true, false),
            'έ' => push!('ε', true, false),
            'ή' => push!('η', true, false),
            'ί' => push!('ι', true, false),
            'ό' => push!('ο', true, false),
            'ύ' => push!('υ', true, false),
            'ώ' => push!('ω', true, false),
            'ϊ' => push!('ι', false, true),
            'ϋ' => push!('υ', false, true),
            'ΐ' => push!('ι', true, true),
            'ΰ' => push!('υ', true, true),
            'Α'..='Ω' => {
                let l = c.to_lowercase().next().unwrap_or(c);
                push!(if l == 'ς' { 'σ' } else { l }, false, false)
            }
            'Ά' => push!('α', true, false),
            'Έ' => push!('ε', true, false),
            'Ή' => push!('η', true, false),
            'Ί' => push!('ι', true, false),
            'Ό' => push!('ο', true, false),
            'Ύ' => push!('υ', true, false),
            'Ώ' => push!('ω', true, false),
            'Ϊ' => push!('ι', false, true),
            'Ϋ' => push!('υ', false, true),
            'ϐ' => push!('β', false, false),
            'ϑ' => push!('θ', false, false),
            'ϰ' => push!('κ', false, false),
            'ϱ' => push!('ρ', false, false),
            'ϕ' => push!('φ', false, false),
            'ϖ' => push!('π', false, false),
            'ϲ' | 'Ϲ' => push!('σ', false, false),
            // combining marks (NFD input): attach to the previous letter
            '\u{0301}' | '\u{0300}' | '\u{0342}' | '\u{0341}' | '\u{0340}' => {
                if let Some(l) = out.last_mut() {
                    l.acc = true;
                }
            }
            '\u{0308}' => {
                if let Some(l) = out.last_mut() {
                    l.dia = true;
                }
            }
            '\u{0344}' => {
                if let Some(l) = out.last_mut() {
                    l.acc = true;
                    l.dia = true;
                }
            }
            _ if (0x1F00..=0x1FFF).contains(&u) => {
                if let Some((b, acc, dia)) = polytonic(u) {
                    push!(b, acc, dia);
                }
            }
            'a'..='z' | 'A'..='Z' => pending_latin.push(c.to_ascii_lowercase()),
            _ => {}
        }
    }
    if !pending_latin.is_empty() {
        // crude transliteration of Latin-script words: read them with Greek letters
        let n = pending_latin.len();
        for (i, &c) in pending_latin.iter().enumerate() {
            let nx = pending_latin.get(i + 1).copied().unwrap_or(' ');
            let vowel_next = matches!(nx, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
            match c {
                'a' => push!('α', false, false),
                'b' => push!('b', false, false),
                'c' => push!(if matches!(nx, 'e' | 'i' | 'y') { 'σ' } else { 'κ' }, false, false),
                'd' => push!('d', false, false),
                'e' => {
                    if i > 0 && pending_latin[i - 1] == 'e' {
                        // "ee": the first one became ι
                    } else if nx == 'e' {
                        push!('ι', false, false)
                    } else if !(i + 1 == n && n > 2) {
                        push!('ε', false, false)
                    }
                }
                'f' => push!('φ', false, false),
                'g' => push!('g', false, false),
                'h' => {
                    if i == 0 || !matches!(pending_latin[i - 1], 'c' | 's' | 't' | 'p') {
                        push!('χ', false, false)
                    }
                }
                'i' | 'y' => push!('ι', false, false),
                'j' => {
                    push!('τ', false, false);
                    push!('ζ', false, false)
                }
                'k' | 'q' => push!('κ', false, false),
                'l' => push!('λ', false, false),
                'm' => push!('μ', false, false),
                'n' => push!('ν', false, false),
                'o' => {
                    if i > 0 && pending_latin[i - 1] == 'o' {
                        // "oo": the first one became ου
                    } else if nx == 'o' {
                        push!('ο', false, false);
                        push!('υ', false, false)
                    } else {
                        push!('ο', false, false)
                    }
                }
                'p' => push!('π', false, false),
                'r' => push!('ρ', false, false),
                's' => push!('σ', false, false),
                't' => push!('τ', false, false),
                'u' => {
                    push!('ο', false, false);
                    push!('υ', false, false)
                }
                'v' | 'w' => {
                    if c == 'w' && vowel_next {
                        push!('ο', false, false);
                        push!('υ', false, false)
                    } else {
                        push!('β', false, false)
                    }
                }
                'x' => push!('ξ', false, false),
                'z' => push!('ζ', false, false),
                _ => {}
            }
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Iota {
    No,
    /// ι υ η (and οι, which patterns with ι in synizesis)
    I,
    /// ει
    Ei,
}

#[derive(Clone, Debug)]
struct Seg {
    ph: &'static str,
    v: bool,
    acc: bool,
    iota: Iota,
    /// the original vowel letter of an iota group (ι υ η ε ο)
    ltr: char,
    /// υ of αυ/ευ/ηυ: a v/f resolved after the whole word is known
    au: bool,
    /// a vowel written as a digraph (αι ει οι ου υι)
    dig: bool,
    /// written with a diaeresis (ϊ ϋ): always its own syllable
    dia: bool,
    /// index of the first letter this segment came from
    pos: usize,
}

fn cons(ph: &'static str) -> Seg {
    Seg { ph, v: false, acc: false, iota: Iota::No, ltr: ' ', au: false, dig: false, dia: false, pos: 0 }
}
fn vow(ph: &'static str, acc: bool, iota: Iota, ltr: char, dig: bool) -> Seg {
    Seg { ph, v: true, acc, iota, ltr, au: false, dig, dia: false, pos: 0 }
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'α' | 'ε' | 'η' | 'ι' | 'ο' | 'υ' | 'ω')
}

fn voiced_start(ph: &str) -> bool {
    matches!(ph.chars().next(), Some('v' | 'ð' | 'ɣ' | 'z' | 'b' | 'd' | 'ɡ' | 'm' | 'n' | 'l' | 'r' | 'j' | 'ʎ' | 'ɲ' | 'ŋ'))
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    let ls = letters(w);
    if ls.is_empty() {
        return String::new();
    }
    // lookup key: base letters with a monotonic tonos (so polytonic καὶ matches και)
    let lw: String = ls
        .iter()
        .map(|l| match (l.b, l.acc) {
            ('α', true) => 'ά',
            ('ε', true) => 'έ',
            ('η', true) => 'ή',
            ('ι', true) => 'ί',
            ('ο', true) => 'ό',
            ('υ', true) => 'ύ',
            ('ω', true) => 'ώ',
            (b, _) => b,
        })
        .collect();
    let mode = if UNSTRESSED.contains(&lw.as_str()) {
        Stress::None
    } else if WEAK.contains(&lw.as_str()) {
        Stress::Weak
    } else {
        Stress::Primary
    };
    let mut segs = parse(&ls);
    resolve(&mut segs, &ls);
    render(&segs, mode)
}

#[derive(Clone, Copy, PartialEq)]
enum Stress {
    Primary,
    Weak,
    None,
}

/// Letters -> segments: digraphs, consonant clusters, double consonants.
fn parse(ls: &[L]) -> Vec<Seg> {
    let n = ls.len();
    let at = |i: usize| -> Option<L> { ls.get(i).copied() };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 2);
    let mut i = 0;
    while i < n {
        let start = i;
        let before = segs.len();
        let l = ls[i];
        let nx = at(i + 1);
        let nb = nx.map(|l| l.b).unwrap_or(' ');
        if is_vowel_letter(l.b) {
            // digraphs: second letter without diaeresis, first without accent
            let pair = nx.filter(|x| !x.dia && !l.acc);
            let acc2 = pair.map(|x| x.acc).unwrap_or(false);
            let d = match (l.b, pair.map(|x| x.b)) {
                ('α', Some('ι')) => Some(vow("e", acc2, Iota::No, 'α', true)),
                ('ε', Some('ι')) => Some(vow("i", acc2, Iota::Ei, 'ε', true)),
                ('ο', Some('ι')) => Some(vow("i", acc2, Iota::I, 'ο', true)),
                ('υ', Some('ι')) => Some(vow("i", acc2, Iota::No, 'υ', true)),
                ('ο', Some('υ')) => Some(vow("u", acc2, Iota::No, 'ο', true)),
                _ => None,
            };
            if let Some(s) = d {
                segs.push(s);
                i += 2;
                tag(&mut segs, before, start);
                continue;
            }
            if matches!(l.b, 'α' | 'ε' | 'η') && pair.map(|x| x.b) == Some('υ') {
                let ph = match l.b {
                    'α' => "a",
                    'ε' => "e",
                    _ => "i",
                };
                segs.push(vow(ph, acc2, Iota::No, l.b, true));
                let mut s = cons("v");
                s.au = true;
                segs.push(s);
                i += 2;
                tag(&mut segs, before, start);
                continue;
            }
            let (ph, iota) = match l.b {
                'α' => ("a", Iota::No),
                'ε' => ("e", Iota::No),
                'η' => ("i", Iota::I),
                'ι' => ("i", Iota::I),
                'ο' => ("o", Iota::No),
                'υ' => ("i", Iota::I),
                _ => ("o", Iota::No),
            };
            let mut sg = vow(ph, l.acc, iota, l.b, false);
            sg.dia = l.dia;
            segs.push(sg);
            i += 1;
            tag(&mut segs, before, start);
                continue;
        }
        // consonants
        let initial = segs.is_empty();
        match (l.b, nb) {
            ('μ', 'π') => {
                let after = at(i + 2).map(|l| l.b).unwrap_or(' ');
                let before_voiceless = matches!(after, 'τ' | 'σ' | 'κ' | 'φ' | 'θ' | 'χ' | 'ξ' | 'ψ');
                segs.push(cons(if before_voiceless { "mp" } else if initial { "b" } else { "mb" }));
                i += 2;
            }
            ('ν', 'γ') if at(i + 2).map(|l| l.b) == Some('κ') => {
                // Κονγκό: the ν is the nasal of ŋɡ
                i += 1;
            }
            ('ν', 'κ') | ('ν', 'χ') | ('ν', 'ξ') | ('ν', 'γ') => {
                segs.push(cons("ŋ"));
                i += 1;
            }
            ('ν', 'τ') => {
                segs.push(cons(if initial { "d" } else { "nd" }));
                i += 2;
            }
            ('γ', 'κ') => {
                segs.push(cons(if initial { "ɡ" } else { "ŋɡ" }));
                i += 2;
            }
            ('γ', 'γ') => {
                // γγ: ŋɡ before a vowel (άγγελος), ŋɣ before a consonant (συγγραφέας, έγγραφο)
                let after = at(i + 2).map(|l| l.b).unwrap_or(' ');
                segs.push(cons(if is_vowel_letter(after) { "ŋɡ" } else { "ŋɣ" }));
                i += 2;
            }
            ('γ', 'χ') | ('γ', 'ξ') => {
                segs.push(cons("ŋ"));
                i += 1;
            }
            ('τ', 'σ') => {
                segs.push(cons("ts"));
                i += 2;
            }
            ('τ', 'ζ') => {
                segs.push(cons("dz"));
                i += 2;
            }
            (a, b) if a == b => {
                // double consonant: single sound
                i += 1;
            }
            _ => {
                let ph = match l.b {
                    'b' => "b",
                    'd' => "d",
                    'g' => "ɡ",
                    'β' => "v",
                    'γ' => "ɣ",
                    'δ' => "ð",
                    'ζ' => "z",
                    'θ' => "θ",
                    'κ' => "k",
                    'λ' => "l",
                    'μ' => "m",
                    'ν' => "n",
                    'ξ' => "ks",
                    'π' => "p",
                    'ρ' => "r",
                    'σ' => "s",
                    'τ' => "t",
                    'φ' => "f",
                    'χ' => "x",
                    'ψ' => "ps",
                    _ => "",
                };
                if !ph.is_empty() {
                    segs.push(cons(ph));
                }
                i += 1;
            }
        }
        tag(&mut segs, before, start);
    }
    segs
}

/// Record the letter index each new segment came from.
fn tag(segs: &mut [Seg], from: usize, pos: usize) {
    for s in &mut segs[from..] {
        s.pos = pos;
    }
}

/// Context rules: αυ/ευ voicing, σ voicing, palatalization, synizesis.
fn resolve(segs: &mut Vec<Seg>, ls: &[L]) {
    let n = segs.len();
    // σ before a voiced consonant
    for k in 0..n.saturating_sub(1) {
        if segs[k].ph == "s" && !segs[k + 1].v && voiced_start(segs[k + 1].ph) {
            segs[k].ph = "z";
        }
    }
    // αυ/ευ: v before a vowel or voiced consonant, f otherwise (and word-finally)
    for k in 0..n {
        if segs[k].au {
            let voiced = match segs.get(k + 1) {
                None => false,
                Some(s) if s.v => true,
                Some(s) => voiced_start(s.ph),
            };
            segs[k].ph = if voiced { "v" } else { "f" };
        }
    }
    // palatal κ χ γ before e / i
    for k in 0..n.saturating_sub(1) {
        let front = segs[k + 1].v && matches!(segs[k + 1].ph, "e" | "i");
        if front {
            segs[k].ph = match segs[k].ph {
                "k" => "c",
                "x" => "ç",
                "ɣ" => "j",
                other => other,
            };
        }
    }
    // synizesis: an unaccented ι-group between a consonant and a vowel
    let any_acc = segs.iter().any(|s| s.v && s.acc);
    let nvow = segs.iter().filter(|s| s.v).count();
    let lw: String = ls.iter().map(|l| l.b).collect();
    let mut k = 1;
    while k + 1 < segs.len() {
        let s = &segs[k];
        if !(s.v && s.iota != Iota::No && !s.acc && segs[k + 1].v && !segs[k - 1].v && !segs[k - 1].au) {
            k += 1;
            continue;
        }
        let prev = segs[k - 1].ph;
        let last = prev.chars().last().unwrap_or(' ');
        let cluster = k >= 2 && !segs[k - 2].v;
        let rcluster = last == 'r' && cluster;
        let next_ph = segs[k + 1].ph;
        let stressed_next = segs[k + 1].acc || (!any_acc && nvow == 2);
        let rest = &segs[k + 2..];
        let fin = rest.is_empty() || (rest.len() == 1 && !rest[0].v);
        let ei = s.iota == Iota::Ei;
        let oi = s.ltr == 'ο';
        // letters after the ι-group (for suffix checks)
        let tail: String = ls.iter().map(|l| l.b).collect::<String>();
        let after_iota: String = tail.chars().skip(s.pos + if s.dig { 2 } else { 1 }).collect();
        let gen_pl = after_iota.starts_with("ων") && after_iota.chars().count() == 2 && segs[k + 1].acc; // -ιών
        let ending_e = next_ph == "e" && segs[k + 1].acc; // ιέ
        let glide = if lw.starts_with("δημιουργ") || after_iota.starts_with("οτητ") {
            false // δημιουργία, ποιότητα, αρμοδιότητα
        } else if next_ph == "i" && (!fin || segs[k + 1].acc) {
            false // περιήγηση, υγιεινή, χρησιμοποιεί
        } else if last == 'j' || last == 'ɣ' {
            // γι + vowel is the spelling of /ʝ/ + vowel, except the learned -γεια / -γιο / -γιών
            // (ενέργεια, κολέγιο, τεχνολογιών; but γεια, γιος, άγιο, γιορτή)
            !((ei || gen_pl || matches!(next_ph, "o" | "u")) && nvow > 3 && k > 1 && !lw.starts_with("κουραγ"))
        } else if next_ph == "i" {
            // -ιοι plural: όποιοι, ντόπιοι glide; αρμόδιοι, κύριοι hiatus
            last == 'p' && oi
        } else if stressed_next {
            if rcluster {
                false // τριάντα, χρειάζομαι, μακριά
            } else if last == 'ð' && k == 1 && nvow > 2 {
                // learned δια-/διε- prefix: διάφορος, διέταξε; common words keep the glide
                GLIDE_DIA.iter().any(|p| lw.starts_with(p))
            } else if last == 'ð' && k == 2 && segs[0].ph == "i" && nvow > 2 {
                false // ιδιότητα
            } else if lw.starts_with("περι") && k == 3 {
                false // περίοδος, περιέχω
            } else if ending_e {
                matches!(last, 'l' | 'n' | 'r' | 't') // μπανιέρα, καριέρα, αναρωτιέμαι; but πιέζω, αφιέρωμα, Βιέννη
            } else if gen_pl && (ei || cluster) {
                false // ενεργειών, εταιρειών
            } else if ei && !fin {
                false // σημειώνω, μειώνω
            } else if last == 's' && !fin {
                false // παρουσιάζω, πλησιάζω
            } else if last == 'p' && oi && nvow > 2 {
                false // ποιότητα, χρησιμοποιώ
            } else if last == 'v' && next_ph != "a" {
                false // επιβιώσει, βιολογία
            } else {
                true
            }
        } else if !fin {
            match last {
                'ç' | 'x' => k == 1,              // χιόνι, χιούμορ
                'l' => k == 1,                    // λιοντάρι
                'm' => s.ltr == 'υ' && next_ph == "a" && !cluster, // μυαλό (not μυωπία, βαθμιαία)
                'ð' => (k > 1 && next_ph == "a" && !(k == 2 && segs[0].ph == "i")) || (k == 1 && GLIDE_DIA.iter().any(|p| lw.starts_with(p))), // ενδιαφέρον, διαβάζω; not διαθέτω, ιδιαίτερα
                'v' => next_ph == "a",            // βιαστικά
                'r' => lw.starts_with("κυριακ"),  // Κυριακή
                't' => cluster && lw.contains("φτια"), // έφτιαξα
                _ => false,
            }
        } else if rcluster {
            false // Σεπτέμβριος, Μάρτιος
        } else if ei {
            matches!(last, 'ð' | 'θ') // άδεια, αλήθεια; but τέλεια, συνέπεια, οικογένεια
        } else if oi {
            matches!(last, 'p' | 't' | 'm') // κάποια, τέτοια, όποιο, όμοια; but έννοια, Εύβοια, παράνοια
        } else if matches!(next_ph, "o" | "u") {
            // γαλάζιο, ήλιος, γέλιο, μπάνιο; but κύριος, σχέδιο, στάδιο, Ιούνιος
            matches!(last, 'z') || ["ηλι", "γελι", "μπανι", "δικι"].iter().any(|p| lw.starts_with(p) && lw.chars().count() <= p.chars().count() + 3)
        } else if last == 'r' {
            // ψάρια, ζευγάρια, αγόρια glide; learned εκατομμύρια, κύρια, εργαστήρια keep the hiatus
            k >= 2 && matches!(segs[k - 2].ph, "a" | "o" | "e") && segs[k - 2].acc
        } else if last == 'v' {
            false // Εύβοια, ακρίβεια, σωσίβια
        } else {
            // -ια / -ιες: μάτια, κάποια, χρόνια, άδεια
            !matches!(prev, "ks" | "x" | "ç")
        };
        if glide {
            let rep: &'static str = match prev {
                "l" => "ʎ",
                "n" => "ɲ",
                "m" => "mɲ",
                "c" | "k" => "c",
                "ç" | "x" => "ç",
                "j" | "ɣ" => "j",
                "p" => "pç",
                "t" => "tç",
                "f" => "fç",
                "θ" => "θç",
                "s" => "sç",
                "ts" => "tsç",
                "ks" => "ksç",
                "ps" => "psç",
                "v" => "vj",
                "ð" => "ðj",
                "z" => "zj",
                "r" => "rj",
                "b" => "bj",
                "d" => "dj",
                "ɡ" => "ɡj",
                "mb" => "mbj",
                "nd" => "ndj",
                "ŋɡ" => "ŋɡj",
                "dz" => "dzj",
                other => other,
            };
            segs[k - 1].ph = rep;
            segs.remove(k);
            // the consonant is now followed by a back vowel; undo a palatal from the glide's i
            continue;
        }
        k += 1;
    }
}

/// Stress assignment and output.
fn render(segs: &[Seg], mode: Stress) -> String {
    // espeak writes an unstressed ι/η/υ right after another vowel as a non-syllabic ɪ (Μάιος, ρολόι)
    // (ϊ too, as espeak does: προϊόντα proɪˈonda; ϋ stays a syllable: προϋπόθεση)
    let offglide = |i: usize| i > 0 && segs[i - 1].v && segs[i].ph == "i" && !segs[i].dig && !(segs[i].dia && segs[i].ltr == 'υ') && !segs[i].acc && segs[i].iota == Iota::I;
    let nuclei: Vec<usize> = segs.iter().enumerate().filter(|&(i, s)| s.v && !offglide(i)).map(|(i, _)| i).collect();
    let mut out = String::with_capacity(segs.len() * 2 + 4);
    if nuclei.is_empty() {
        for s in segs {
            out.push_str(s.ph);
        }
        return out;
    }
    let primary = nuclei.iter().position(|&ix| segs[ix].acc).unwrap_or(if nuclei.len() >= 2 { nuclei.len() - 2 } else { 0 });
    let mut nk = 0usize;
    for (i, seg) in segs.iter().enumerate() {
        if seg.v && offglide(i) && !nuclei.is_empty() {
            out.push('ɪ');
        } else if seg.v {
            let mark = match mode {
                Stress::None => "",
                Stress::Weak | Stress::Primary => {
                    if nk == primary || segs[i].acc {
                        // every written accent is a primary stress (enclitic groups: όνομά μου)
                        if mode == Stress::Weak {
                            "ˌ"
                        } else {
                            "ˈ"
                        }
                    } else if (nk + 1 < primary && nk % 2 == 0) || (nk > primary && (nk - primary) % 2 == 0) {
                        "ˌ"
                    } else {
                        ""
                    }
                }
            };
            out.push_str(mark);
            out.push_str(seg.ph);
            nk += 1;
        } else {
            out.push_str(seg.ph);
        }
    }
    out
}

fn first_phone(s: &str) -> Option<char> {
    s.chars().find(|&c| c != 'ˈ' && c != 'ˌ' && c != '\u{0}')
}

/// Sentence pass: final-ν sandhi of the clitics (τον πατέρα → tom batˈera),
/// voicing of a word-final s before a voiced consonant (τους γονείς → tuz ɣonˈis),
/// and elided clitics (σ' αυτό → saftˈo).
pub fn join(words: &mut [Out]) {
    for k in 1..words.len() {
        let (a, b) = words.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        // elided clitic written with an apostrophe (σ', μ', τ', γι', απ', κατ')
        let prev_has_vowel = prev.ph.contains(|c: char| "aeiouɪ".contains(c));
        if prev.src == "γι" && first_phone(&cur.ph).is_some_and(|c| "aeiou".contains(c)) {
            prev.ph = "j".into(); // γι' αυτό
        }
        if (!prev_has_vowel || prev.src == "γι") && !prev.src.is_empty() {
            if prev.ph.chars().all(|c| c.is_alphabetic() || c == 'ŋ') {
                let mut glued = String::with_capacity(cur.ph.len() + 1);
                glued.push('\u{0}');
                glued.push_str(&cur.ph);
                cur.ph = glued;
                continue;
            }
        }
        let first = first_phone(&cur.ph);
        let last = prev.ph.chars().last().unwrap();
        if last == 'n' && matches!(prev.src.as_str(), "τον" | "την" | "δεν" | "μην" | "σαν" | "έναν" | "αυτόν" | "στον" | "στην") {
            // clitic-final ν + voiceless stop: voiced stop, nasal assimilated
            let rest = cur.ph.trim_start_matches(|c| c == 'ˈ' || c == 'ˌ');
            let marks = &cur.ph[..cur.ph.len() - rest.len()];
            let (nasal, rep): (&str, Option<String>) = match first {
                Some('p') => ("m", Some(format!("b{}", &rest[1..]))),
                Some('t') if rest.starts_with("ts") => ("n", Some(format!("dz{}", &rest[2..]))),
                Some('t') => ("n", Some(format!("d{}", &rest[1..]))),
                Some('k') if rest.starts_with("ks") => ("ŋ", Some(format!("ɡz{}", &rest[2..]))),
                Some('k') => ("ŋ", Some(format!("ɡ{}", &rest[1..]))),
                Some('c') => ("ŋ", Some(format!("ɟ{}", &rest[1..]))),
                Some('b') => ("m", None),
                Some('d') => ("n", None),
                Some('ɡ') => ("ŋ", None),
                _ => ("n", None),
            };
            if let Some(r) = rep {
                let mut p: String = prev.ph.chars().collect();
                p.pop();
                p.push_str(nasal);
                prev.ph = p;
                let mut nph = String::with_capacity(cur.ph.len() + 2);
                nph.push_str(marks);
                // espeak has no ɟ: write ɡ before a front vowel too
                nph.push_str(&r.replace('ɟ', "ɡ"));
                cur.ph = nph;
            } else if nasal != "n" {
                let mut p: String = prev.ph.chars().collect();
                p.pop();
                p.push_str(nasal);
                prev.ph = p;
            }
        } else if last == 's' && first.is_some_and(|c| matches!(c, 'v' | 'ð' | 'ɣ' | 'z' | 'b' | 'd' | 'ɡ' | 'm' | 'n' | 'l' | 'r' | 'j' | 'ʎ' | 'ɲ')) {
            let mut p: String = prev.ph.chars().collect();
            p.pop();
            p.push('z');
            prev.ph = p;
        }
    }
}

// ---------------------------------------------------------------- numbers

const ONES: [&str; 20] = [
    "μηδέν", "ένα", "δύο", "τρία", "τέσσερα", "πέντε", "έξι", "επτά", "οκτώ", "εννέα", "δέκα", "έντεκα", "δώδεκα",
    "δεκατρία", "δεκατέσσερα", "δεκαπέντε", "δεκαέξι", "δεκαεπτά", "δεκαοκτώ", "δεκαεννέα",
];
const TENS: [&str; 10] = ["", "", "είκοσι", "τριάντα", "σαράντα", "πενήντα", "εξήντα", "εβδομήντα", "ογδόντα", "ενενήντα"];
const HUNDREDS: [&str; 10] = [
    "", "εκατό", "διακόσια", "τριακόσια", "τετρακόσια", "πεντακόσια", "εξακόσια", "επτακόσια", "οκτακόσια", "εννιακόσια",
];

/// Number words (neuter forms; thousands take the feminine as Greek requires).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["μηδέν".into()];
    }
    let mut groups = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push((m % 1000) as u32);
        m /= 1000;
    }
    if groups.len() > 5 {
        return n.to_string().chars().map(|d| ONES[d as usize - '0' as usize].to_string()).collect();
    }
    let mut out = Vec::new();
    for (k, &g) in groups.iter().enumerate().rev() {
        if g == 0 {
            continue;
        }
        match k {
            0 => below_1000(g, false, &mut out),
            1 => {
                if g == 1 {
                    out.push("χίλια".into());
                } else {
                    below_1000(g, true, &mut out);
                    out.push("χιλιάδες".into());
                }
            }
            _ => {
                let (one, many) = match k {
                    2 => ("εκατομμύριο", "εκατομμύρια"),
                    3 => ("δισεκατομμύριο", "δισεκατομμύρια"),
                    _ => ("τρισεκατομμύριο", "τρισεκατομμύρια"),
                };
                below_1000(g, false, &mut out);
                out.push(if g == 1 { one } else { many }.into());
            }
        }
    }
    out
}

fn below_1000(g: u32, fem: bool, out: &mut Vec<String>) {
    let h = g / 100;
    let r = g % 100;
    if h > 0 {
        let w = if h == 1 {
            if r > 0 { "εκατόν" } else { "εκατό" }
        } else {
            HUNDREDS[h as usize]
        };
        let w = if fem && h > 1 { w.replace("όσια", "όσιες") } else { w.to_string() };
        out.push(w);
    }
    if r == 0 {
        return;
    }
    if r < 20 {
        let w = match (r, fem) {
            (1, true) => "μία",
            (3, true) => "τρεις",
            (4, true) => "τέσσερις",
            _ => ONES[r as usize],
        };
        out.push(w.into());
    } else {
        out.push(TENS[(r / 10) as usize].into());
        let u = r % 10;
        if u > 0 {
            let w = match (u, fem) {
                (1, true) => "μία",
                (3, true) => "τρεις",
                (4, true) => "τέσσερις",
                _ => ONES[u as usize],
            };
            out.push(w.into());
        }
    }
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "τοις εκατό",
        '$' => "δολάρια",
        '€' => "ευρώ",
        '£' => "λίρες",
        '&' => "και",
        '+' => "συν",
        '@' => "παπάκι",
        '°' => "βαθμοί",
        '=' => "ίσον",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn basics() {
        for (w, p) in [
            ("καλημέρα", "kˌalimˈera"),
            ("άνθρωπος", "ˈanθropˌos"),
            ("αυτός", "aftˈos"),
            ("αυγό", "avɣˈo"),
            ("ευρώ", "evrˈo"),
            ("ευχαριστώ", "ˌefxaristˈo"),
            ("ταυ", "tˈaf"),
            ("μπορώ", "borˈo"),
            ("κάμπος", "kˈambos"),
            ("ντομάτα", "domˈata"),
            ("πέντε", "pˈende"),
            ("γκολ", "ɡˈol"),
            ("άγγελος", "ˈaŋɡelˌos"),
            ("άγχος", "ˈaŋxos"),
            ("γιατί", "jatˈi"),
            ("γυναίκα", "jinˈeka"),
            ("χέρι", "çˈeri"),
            ("κήπος", "cˈipos"),
            ("κόσμος", "kˈozmos"),
            ("σβήνω", "zvˈino"),
            ("τσάι", "tsˈaɪ"),
            ("τζάμι", "dzˈami"),
            ("Ελλάδα", "elˈaða"),
            ("θάλασσα", "θˈalasˌa"),
            ("παιδιά", "peðjˈa"),
            ("ποιος", "pçˈos"),
            ("μια", "mɲˈa"),
            ("δύο", "ðˈio"),
            ("ήλιος", "ˈiʎos"),
            ("χρόνια", "xrˈoɲa"),
            ("κύριος", "cˈiriˌos"),
            ("σχέδιο", "sçˈeðiˌo"),
            ("τριάντα", "triˈanda"),
            ("δουλειά", "ðuʎˈa"),
            ("οικογένεια", "ˌikojˈeniˌa"),
            ("Μάιος", "mˈaɪos"),
            ("προϋπόθεση", "prˌoipˈoθesˌi"),
            ("ξέρω", "ksˈero"),
            ("ψάρι", "psˈari"),
            ("καΐκι", "kaˈici"),
            ("Αθήνα", "aθˈina"),
            ("ανδρας", "ˈanðras"),
            ("και", "ce"),
            ("είναι", "ˌine"),
            ("ένα", "ˈena"),
            ("όνομά", "ˈonomˈa"),
            ("προϊόντα", "proɪˈonda"),
            ("χρησιμοποιεί", "xrˌisimˌopiˈi"),
            ("ευρωπαϊκή", "ˌevropaɪcˈi"),
            ("διαβάζω", "ðjavˈazo"),
            ("διάλογος", "ðiˈaloɣˌos"),
            ("καὶ", "ce"),
            ("δισεκατομμύρια", "ðˌisekˌatomˈiriˌa"),
            ("ψάρια", "psˈarja"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(21).join(" "), "είκοσι ένα");
        assert_eq!(cardinal(101).join(" "), "εκατόν ένα");
        assert_eq!(cardinal(200).join(" "), "διακόσια");
        assert_eq!(cardinal(1000).join(" "), "χίλια");
        assert_eq!(cardinal(3000).join(" "), "τρεις χιλιάδες");
        assert_eq!(cardinal(201000).join(" "), "διακόσιες μία χιλιάδες");
        assert_eq!(cardinal(2_000_000).join(" "), "δύο εκατομμύρια");
        assert_eq!(cardinal(1_000_000_000).join(" "), "ένα δισεκατομμύριο");
    }
}
