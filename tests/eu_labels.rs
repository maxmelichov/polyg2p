//! European label fixes: whole-sentence cases for the rule fixes of this branch (no data directory needed).
use polyg2p::{G2p, Lang};

fn p(l: &str, t: &str) -> String {
    G2p::new(None).phonemize(t, Lang::parse(l).unwrap())
}

/// A sentence-initial capital one-letter preposition is the word (Wiktionary pl `w`, `z`; cs `s`, `z`, `k`, `v`):
/// the text reads exactly as its lower-case spelling; a lone capital elsewhere stays a letter name.
#[test]
fn capital_prepositions() {
    for (l, a, b) in [
        ("pl", "W tej chwili.", "w tej chwili."), ("pl", "Z tego.", "z tego."),
        ("cs", "S tebou.", "s tebou."), ("cs", "Z toho.", "z toho."), ("cs", "K domu.", "k domu."), ("cs", "V domě.", "v domě."),
        ("cs", "Byl tam. S cizími věcmi.", "Byl tam. s cizími věcmi."),
    ] {
        assert_eq!(p(l, a), p(l, b), "{a}");
    }
    assert!(p("cs", "Je to vitamín K.").ends_with("kˈaː."));
    assert!(p("pl", "To jest punkt Z.").ends_with("zˈɛt."));
}
