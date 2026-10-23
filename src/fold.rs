//! Final output conventions, mirroring what BlueTTS's `normalize_text` does to
//! espeak output before it reaches the model, so training and inference agree:
//! NFD decomposition (ç -> c + U+0327) and affricate ligatures.

use unicode_normalization::UnicodeNormalization;

/// Fold one finished phoneme string.
pub fn fold(s: &str) -> String {
    let s: String = s.nfd().collect();
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let n = chars.get(i + 1).copied();
        let tie = n == Some('\u{0361}');
        let (second, skip) = if tie { (chars.get(i + 2).copied(), 3) } else { (n, 2) };
        match (c, second) {
            ('t', Some('ʃ')) => {
                out.push('ʧ');
                i += skip;
                continue;
            }
            ('d', Some('ʒ')) => {
                out.push('ʤ');
                i += skip;
                continue;
            }
            ('d', Some('z')) => {
                out.push('ʣ');
                i += skip;
                continue;
            }
            ('t', Some('s')) => {
                // normalize_text folds "ts" only at the end of a word (or with a tie bar)
                let after = chars.get(i + skip).copied();
                if tie || after.map_or(true, |a| !a.is_alphanumeric() && !crate::text::is_mark(a)) {
                    out.push('ʦ');
                    i += skip;
                    continue;
                }
            }
            _ => {}
        }
        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::fold;
    #[test]
    fn ligatures() {
        assert_eq!(fold("tʃitːˈa"), "ʧitːˈa");
        assert_eq!(fold("mənts"), "mənʦ");
        assert_eq!(fold("fabritsˌiːrən"), "fabritsˌiːrən");
        assert_eq!(fold("ç"), "c\u{0327}");
    }
}
