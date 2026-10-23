//! Spanish (Castilian) grapheme-to-phoneme rules, espeak-style notation.
//!
//! Spanish spelling is close to phonemic, so this is rules only: syllabification,
//! orthographic stress, allophones (β ð ɣ), and cross-word effects that depend on
//! the previous word (b/d/g after a pause or nasal, n -> ŋ before velars).

use crate::Out;

/// Unstressed function words (espeak reads these without a stress mark).
const UNSTRESSED: &[&str] = &[
    "el", "la", "los", "las", "lo", "le", "les", "de", "del", "en", "con", "que", "me", "te", "se",
    "nos", "os", "y", "e", "o", "u", "a", "al", "por", "sin", "su", "sus", "mi", "mis", "tu", "tus",
    "ni", "si", "tan", "san", "don", "doña",
];
/// Function words with a secondary stress.
const SECONDARY: &[&str] = &[
    "pero", "para", "porque", "como", "cuando", "donde", "mientras", "aunque", "cuanto", "hasta",
    "desde", "entre", "sobre", "hacia", "según", "contra", "tras", "pues", "mas",
];

#[derive(Clone, Debug)]
struct Seg {
    ph: &'static str,
    /// off-glide of a falling diphthong (aɪ, eʊ, ...)
    glide: &'static str,
    nucleus: bool,
    /// written accent: this nucleus takes the primary stress
    accent: bool,
}

fn s(ph: &'static str) -> Seg {
    Seg { ph, glide: "", nucleus: false, accent: false }
}

fn base(c: char) -> char {
    match c {
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' | 'ü' => 'u',
        _ => c,
    }
}
fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü')
}
fn accented(c: char) -> bool {
    matches!(c, 'á' | 'é' | 'í' | 'ó' | 'ú')
}

/// A word's phones with the information the sentence pass needs.
pub struct EsWord {
    pub segs: Vec<String>,
}

/// Phonemize one lower-case word (no context). Initial b/d/g are written as stops;
/// the sentence pass softens them after vowels.
pub fn word(w: &str) -> String {
    let lw: String = w.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect();
    if lw.is_empty() {
        return String::new();
    }
    // x read as /x/ in a few names (México, Texas, Oaxaca)
    for (pre, rep) in [("méxic", "méjic"), ("mexic", "mejic"), ("oaxac", "oajac"), ("texa", "teja"), ("xavier", "javier")] {
        if lw.starts_with(pre) {
            let fixed = format!("{rep}{}", &lw[pre.len()..]);
            return word(&fixed);
        }
    }
    if let Some(stem) = lw.strip_suffix("mente") {
        // adverbs in -mente keep the adjective's stress and add their own
        if stem.chars().filter(|&c| is_vowel_letter(c)).count() >= 2 {
            return format!("{}{}", word_inner(stem, Stress::Primary), word_inner("mente", Stress::Primary));
        }
    }
    let st = if UNSTRESSED.contains(&lw.as_str()) {
        Stress::None
    } else if SECONDARY.contains(&lw.as_str()) {
        Stress::Secondary
    } else {
        Stress::Primary
    };
    word_inner(&lw, st)
}

#[derive(Clone, Copy, PartialEq)]
enum Stress {
    Primary,
    Secondary,
    None,
}

fn word_inner(lw: &str, stress: Stress) -> String {
    let c: Vec<char> = lw.chars().collect();
    let n = c.len();
    let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { c[i as usize] } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 2);
    // letter index -> whether it was consumed as a glide/silent etc. is handled inline
    let mut i = 0usize;
    // Is a 'y' at i a vowel?  (alone, word-final after a vowel, or between consonants)
    let y_vowel = |i: usize| -> bool {
        let prev = at(i as isize - 1);
        let next = at(i as isize + 1);
        next == '#' || (!is_vowel_letter(next) && !is_vowel_letter(prev) && prev != '#') || n == 1
    };
    while i < n {
        let ch = c[i];
        let nx = at(i as isize + 1);
        let pv = at(i as isize - 1);
        if is_vowel_letter(ch) || (ch == 'y' && y_vowel(i)) {
            // collect a vowel cluster (h between vowels is silent and splits nothing)
            let start = i;
            let mut j = i;
            let mut letters: Vec<char> = Vec::new();
            while j < n {
                let d = c[j];
                if is_vowel_letter(d) || (d == 'y' && (j == start || y_vowel(j))) {
                    letters.push(d);
                    j += 1;
                } else {
                    break;
                }
            }
            i = j;
            // a full vowel instead of a glide after a consonant + r/l cluster (tɾio, flua)
            // or a word-initial r (rioxano)
            let cluster = start > 0
                && matches!(c[start - 1], 'r' | 'l')
                && (start == 1 || !is_vowel_letter(c[start - 2]) && c[start - 2] != 'l');
            push_vowel_cluster(&letters, &mut segs, i >= n, cluster);
            continue;
        }
        let next_front = matches!(nx, 'e' | 'i' | 'é' | 'í' | 'y');
        let next_cons = nx != '#' && !is_vowel_letter(nx) && nx != 'y' && nx != 'h';
        match ch {
            'b' | 'v' => {
                let stop = i == 0 || matches!(pv, 'm' | 'n') || (next_cons && !matches!(nx, 'l' | 'r' | 's'));
                segs.push(s(if stop { "b" } else { "β" }))
            }
            'd' => {
                let stop = i == 0 || pv == 'n' || i + 1 == n || matches!(nx, 'l' | 'k' | 'q' | 'c' | 'm' | 'n');
                segs.push(s(if stop { "d" } else { "ð" }))
            }
            'g' => {
                if next_front {
                    segs.push(s("x"));
                } else if nx == 'u' && matches!(at(i as isize + 2), 'e' | 'i' | 'é' | 'í') {
                    segs.push(s(if i == 0 || pv == 'n' { "ɡ" } else { "ɣ" }));
                    i += 1; // silent u
                } else {
                    let stop = i == 0 || pv == 'n' || matches!(nx, 'n' | 'm');
                    segs.push(s(if stop { "ɡ" } else { "ɣ" }));
                }
            }
            'c' => {
                if nx == 'h' {
                    segs.push(s("ʧ"));
                    i += 1;
                } else if next_front {
                    segs.push(s("θ"));
                } else if nx == 'k' {
                    // "ck": one k
                } else {
                    segs.push(s("k"));
                }
            }
            'q' => {
                segs.push(s("k"));
                if nx == 'u' {
                    if matches!(at(i as isize + 2), 'a' | 'o' | 'á' | 'ó') {
                        segs.push(s("w"));
                    }
                    i += 1;
                }
            }
            'k' => segs.push(s("k")),
            'z' => {
                if nx == 'z' {
                    segs.push(s("ts"));
                    i += 1;
                } else {
                    segs.push(s("θ"));
                }
            }
            's' => {
                if nx == 'h' {
                    segs.push(s(if i == 0 { "ʃ" } else { "s" }));
                    i += 1;
                } else {
                    segs.push(s("s"));
                }
            }
            'j' => segs.push(s("x")),
            'h' => {
                // silent; "hie-" / "hue-" start with a glide
                if nx == 'i' && matches!(at(i as isize + 2), 'e' | 'é') && i == 0 {
                    segs.push(s("j"));
                    i += 1;
                } else if nx == 'u' && matches!(at(i as isize + 2), 'e' | 'é' | 'i' | 'í') {
                    segs.push(s("w"));
                    i += 1;
                }
            }
            'l' => {
                if nx == 'l' {
                    segs.push(s("ʎ"));
                    i += 1;
                } else {
                    segs.push(s("l"));
                }
            }
            'r' => {
                if nx == 'r' {
                    segs.push(s("r"));
                    i += 1;
                } else if i == 0 || matches!(pv, 'n' | 'l' | 's') {
                    segs.push(s("r"));
                } else {
                    segs.push(s("ɾ"));
                }
            }
            'ñ' => segs.push(s("ɲ")),
            'n' => {
                if matches!(nx, 'g' | 'j') {
                    segs.push(s("ŋ"));
                } else if matches!(nx, 'b' | 'v' | 'p') || (nx == 'f' && matches!(at(i as isize + 2), 'r' | 'l')) {
                    segs.push(s("m"));
                } else if nx == 'y' && is_vowel_letter(at(i as isize + 2)) {
                    segs.push(s("ɲ"));
                    i += 1;
                } else {
                    segs.push(s("n"));
                }
            }
            'm' => segs.push(s("m")),
            'p' => {
                if i == 0 && nx == 's' {
                    // initial ps- (psicología): p is silent
                } else if nx == 't' {
                    segs.push(s("pː"));
                } else {
                    segs.push(s("p"));
                }
            }
            't' => segs.push(s("t")),
            'f' => segs.push(s("f")),
            'x' => {
                if i == 0 {
                    segs.push(s("s"));
                } else {
                    segs.push(s("k"));
                    segs.push(s("s"));
                }
            }
            'y' => segs.push(s("ʝ")),
            'w' => segs.push(s("w")),
            'ç' => segs.push(s("θ")),
            _ => {}
        }
        i += 1;
    }
    // stress
    let nuclei: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| s.nucleus).map(|(i, _)| i).collect();
    if nuclei.is_empty() {
        return segs.iter().map(|s| s.ph).collect();
    }
    let primary = nuclei.iter().position(|&ix| segs[ix].accent);
    let primary = primary.unwrap_or_else(|| {
        let last = c[n - 1];
        if nuclei.len() == 1 {
            0
        } else if is_vowel_letter(last) || last == 'n' || last == 's' {
            nuclei.len() - 2
        } else {
            nuclei.len() - 1
        }
    });
    // espeak opens a stressed e before n + consonant, and next to /x/
    {
        let ix = nuclei[primary];
        if segs[ix].ph == "e" && !segs[ix].accent && segs[ix].glide.is_empty() && lw != "mente" {
            let n1 = segs.get(ix + 1).map(|s| s.ph);
            let n2 = segs.get(ix + 2);
            let before_nc = matches!(n1, Some("n") | Some("ŋ")) && n2.map_or(false, |s| !s.nucleus && !matches!(s.ph, "j" | "w"));
            let by_x = false;
            if before_nc || by_x {
                segs[ix].ph = "ɛ";
            }
        }
    }
    let mut out = String::with_capacity(lw.len() * 2);
    let mut nk = 0usize;
    for seg in &segs {
        if seg.nucleus {
            let mark = match stress {
                Stress::None => "",
                Stress::Secondary => if nk == primary { "ˌ" } else { "" },
                Stress::Primary => {
                    if nk == primary {
                        "ˈ"
                    } else if nk + 1 < primary && nk % 2 == 0 {
                        "ˌ"
                    } else {
                        ""
                    }
                }
            };
            out.push_str(mark);
            nk += 1;
        }
        out.push_str(seg.ph);
        out.push_str(seg.glide);
    }
    out
}

/// Turn a vowel-letter cluster into glides + nuclei.
fn push_vowel_cluster(letters: &[char], segs: &mut Vec<Seg>, word_final: bool, after_rl: bool) {
    let strong = |c: char| matches!(c, 'a' | 'e' | 'o' | 'á' | 'é' | 'ó' | 'í' | 'ú' | 'ü');
    let m = letters.len();
    let mut k = 0;
    while k < m {
        let ch = letters[k];
        let next = letters.get(k + 1).copied();
        if strong(ch) {
            segs.push(nucleus(ch));
            // falling diphthong: strong + unaccented i/u/y
            if let Some(nx) = next {
                if matches!(nx, 'i' | 'u' | 'y') {
                    let last = segs.last_mut().unwrap();
                    let b = base(ch);
                    let glide = match (b, nx) {
                        ('a', 'i' | 'y') => Some("ɪ"),
                        ('e', 'i' | 'y') => Some("ɪ"),
                        ('o', 'i' | 'y') => Some("ɪ"),
                        ('a', 'u') => Some("ʊ"),
                        ('e', 'u') => Some("ʊ"),
                        ('o', 'u') => Some("ʊ"),
                        _ => None,
                    };
                    if let Some(g) = glide {
                        // after a written accent espeak writes the plain vowel (-áis, -éu)
                        last.glide = if accented(ch) { if nx == 'u' { "u" } else { "i" } } else { g };
                        k += 2;
                        continue;
                    }
                }
            }
            k += 1;
        } else {
            // weak i/u/y
            let first_of_cluster = k == 0;
            match next {
                Some(nx) if strong(nx) || (ch == 'i' && nx == 'u') => {
                    if after_rl && first_of_cluster {
                        // tɾio, flua: espeak keeps the full vowel (not a glide)
                        segs.push(s(if base(ch) == 'u' { "u" } else { "i" }));
                    } else {
                        segs.push(s(if base(ch) == 'u' { "w" } else { "j" }));
                    }
                }
                Some('i') if ch == 'u' && !(after_rl && first_of_cluster) => {
                    segs.push(s("w"));
                }
                Some('y') if word_final && k + 2 == m => {
                    // "muy" -> uj
                    segs.push(nucleus(ch));
                    segs.push(s("j"));
                    k += 2;
                    continue;
                }
                _ => segs.push(nucleus(if ch == 'y' { 'i' } else { ch })),
            }
            k += 1;
        }
    }
}

fn nucleus(c: char) -> Seg {
    let p = match base(c) {
        'a' => "a",
        'e' => "e",
        'i' => "i",
        'o' => "o",
        _ => "u",
    };
    Seg { ph: p, glide: "", nucleus: true, accent: accented(c) }
}

/// Sentence pass: soften word-initial b/d/g after a vowel or continuant in the
/// previous word, and assimilate a word-final n before a velar.
pub fn join(words: &mut [Out]) {
    for k in 1..words.len() {
        let (a, b) = words.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let first = first_phone(&cur.ph);
        let last = prev.ph.chars().last().unwrap();
        let soft = match first {
            Some('b') => !matches!(last, 'm' | 'n' | 'ŋ'),
            Some('d') => !matches!(last, 'm' | 'n' | 'ŋ' | 'l'),
            Some('ɡ') => !matches!(last, 'm' | 'n' | 'ŋ'),
            _ => false,
        };
        if soft {
            let rep = match first.unwrap() {
                'b' => 'β',
                'd' => 'ð',
                _ => 'ɣ',
            };
            cur.ph = replace_first_phone(&cur.ph, rep);
        }
        if last == 'n' && matches!(first, Some('ɡ') | Some('x') | Some('ɣ')) {
            let mut p: String = prev.ph.chars().collect();
            p.pop();
            p.push('ŋ');
            prev.ph = p;
        }
    }
}

fn first_phone(s: &str) -> Option<char> {
    s.chars().find(|&c| c != 'ˈ' && c != 'ˌ')
}

fn replace_first_phone(s: &str, rep: char) -> String {
    let mut out = String::with_capacity(s.len());
    let mut done = false;
    for c in s.chars() {
        if !done && c != 'ˈ' && c != 'ˌ' {
            out.push(rep);
            done = true;
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::word;
    #[test]
    fn basics() {
        for (w, p) in [
            ("casa", "kˈasa"),
            ("perro", "pˈero"),
            ("pero", "pˌeɾo"),
            ("cielo", "θjˈelo"),
            ("guitarra", "ɡitˈara"),
            ("llave", "ʎˈaβe"),
            ("ciudad", "θjuðˈad"),
            ("abogado", "ˌaβoɣˈaðo"),
            ("aire", "ˈaɪɾe"),
            ("muy", "mˈuj"),
            ("rey", "rˈeɪ"),
            ("universidad", "ˌuniβˌeɾsiðˈad"),
            ("inglés", "iŋɡlˈes"),
            ("huevo", "wˈeβo"),
            ("Paraguay", "pˌaɾaɣwˈaɪ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }
}
