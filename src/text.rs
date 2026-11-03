//! Tokenization shared by all languages.

/// A piece of input text.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// A run of letters (may contain internal apostrophes / hyphens).
    Word(String),
    /// A run of digits, possibly with separators (`1,000.5`, `3.14`, `1.000,5`).
    Num(String),
    /// Punctuation that survives into the phoneme string (`. , ! ? : ;`).
    Punct(char),
    /// A symbol that is spoken (`%`, `$`, `€`, `£`, `&`, `+`, `@`, `°`).
    Sym(char),
    /// Anything else that only separates words (spaces, quotes, brackets, dashes).
    Sep,
}

pub const KEPT_PUNCT: &[char] = &['.', ',', '!', '?', ':', ';'];

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || is_mark(c)
}

/// Combining marks (Hebrew points, stress accents typed as combining chars, …).
pub fn is_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x0591..=0x05C7 | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF)
}

/// Split text into tokens. Apostrophes and hyphens stay inside words when they
/// sit between letters (`don't`, `l'uomo`, `Baden-Württemberg`).
pub fn tokenize(text: &str) -> Vec<Tok> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // a clitic written with a leading apostrophe ('s morgens, 't, 'em) stays one token
        let clitic = matches!(c, '\'' | '’')
            && (i == 0 || !is_word_char(chars[i - 1]))
            && i + 1 < chars.len()
            && is_word_char(chars[i + 1])
            && !(i + 2 < chars.len() && is_word_char(chars[i + 2]) && i + 3 < chars.len() && is_word_char(chars[i + 3]));
        if is_word_char(c) || clitic {
            let start = i;
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                if is_word_char(d) {
                    i += 1;
                } else if matches!(d, '\'' | '’' | '-' | '‐' | '·') && i + 1 < chars.len() && is_word_char(chars[i + 1]) {
                    i += 1;
                } else {
                    break;
                }
            }
            let w: String = chars[start..i].iter().map(|&c| if c == '’' { '\'' } else { c }).collect();
            out.push(Tok::Word(w));
        } else if c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                if d.is_ascii_digit() {
                    i += 1;
                } else if matches!(d, '.' | ',' | '\u{202F}' | '\u{00A0}' | '\'')
                    && i + 1 < chars.len()
                    && chars[i + 1].is_ascii_digit()
                {
                    i += 1;
                } else if d == ':'
                    && i + 2 < chars.len()
                    && chars[i + 1].is_ascii_digit()
                    && chars[i + 2].is_ascii_digit()
                    && i - start <= 2
                    && !chars.get(i + 3).is_some_and(|c| c.is_ascii_digit())
                {
                    // clock time 3:45 / 14:05
                    i += 1;
                } else {
                    break;
                }
            }
            // a date with slashes: 12/03/2021, 3/10/21
            if chars.get(i) == Some(&'/') && i - start <= 2 {
                let mut j = i + 1;
                let mut ok = false;
                let m0 = j;
                while j < chars.len() && chars[j].is_ascii_digit() { j += 1; }
                if j - m0 >= 1 && j - m0 <= 2 && chars.get(j) == Some(&'/') {
                    let y0 = j + 1;
                    let mut k = y0;
                    while k < chars.len() && chars[k].is_ascii_digit() { k += 1; }
                    if (k - y0 == 4 || k - y0 == 2) && chars.get(k).map_or(true, |c| !c.is_alphanumeric() && *c != '/') {
                        ok = true;
                        i = k;
                    }
                }
                let _ = ok;
            }
            // a decade: 1990s
            if i - start == 4 && matches!(chars.get(i), Some('s') | Some('S')) && chars.get(i + 1).map_or(true, |c| !c.is_alphanumeric()) {
                i += 1;
            }
            // ordinal suffix glued to the digits: 5th, 21st, 2nd, 3rd
            if i + 1 < chars.len() && !matches!(chars[i - 1], 's' | 'S') {
                let suf: String = chars[i..(i + 2).min(chars.len())].iter().collect::<String>().to_ascii_lowercase();
                let after_ok = chars.get(i + 2).map_or(true, |c| !c.is_alphanumeric());
                if matches!(suf.as_str(), "st" | "nd" | "rd" | "th") && after_ok {
                    i += 2;
                }
            }
            out.push(Tok::Num(chars[start..i].iter().collect()));
        } else if KEPT_PUNCT.contains(&c) {
            out.push(Tok::Punct(c));
            i += 1;
        } else if c == '…' {
            out.push(Tok::Punct('.'));
            i += 1;
        } else if matches!(c, '%' | '$' | '€' | '£' | '₪' | '&' | '+' | '@' | '°' | '=' | '¥' | '₽') {
            out.push(Tok::Sym(c));
            i += 1;
        } else {
            out.push(Tok::Sep);
            i += 1;
        }
    }
    out
}
