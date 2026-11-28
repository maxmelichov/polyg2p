//! English heteronym model on the author-written sentences (tests/en_homograph_cases.tsv).
//!
//! Needs `POLYG2P_DATA` to point at a folder with en.lex, en.g2p and the optional
//! `en_homograph.g2p`; without that model file the test prints why and passes, so a default
//! build stays green. With it, the four 'must' sentences have to be right and the accuracy over
//! the file has to reach `HOMOGRAPH_MIN_ACC` (default 0.85, a regression guard; the acceptance
//! number of the plan is 0.90).
use polyg2p::{G2p, Lang};

const CASES: &str = include_str!("en_homograph_cases.tsv");
const TABLE: &str = include_str!("../data/en_homograph.tsv");

fn ipa_of(word: &str, reading: &str) -> String {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() == 4 && f[0] == word && f[1] == reading).then(|| f[3].to_string())
        })
        .unwrap_or_else(|| panic!("no reading {word} {reading}"))
}

#[test]
fn adversarial_sentences() {
    let Some(dir) = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from) else {
        eprintln!("skipped: POLYG2P_DATA is not set");
        return;
    };
    if !dir.join("en_homograph.g2p").exists() || !dir.join("en.lex").exists() {
        eprintln!("skipped: {} has no en_homograph.g2p / en.lex", dir.display());
        return;
    }
    let g = G2p::new(Some(&dir));
    let (mut right, mut n, mut must_wrong) = (0, 0, Vec::new());
    for l in CASES.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let f: Vec<&str> = l.split('\t').collect();
        let want = polyg2p::fold_for_test(&ipa_of(f[1], f[2]));
        let got = g.phonemize_words(f[0], Lang::En).into_iter().find(|(src, _)| src == f[1]).map(|(_, p)| p);
        n += 1;
        if got.as_deref() == Some(want.as_str()) {
            right += 1;
        } else if f.get(3) == Some(&"must") {
            must_wrong.push(f[0]);
        }
    }
    let acc = right as f64 / n as f64;
    eprintln!("adversarial: {right}/{n} = {acc:.3}");
    assert!(must_wrong.is_empty(), "must-pass sentences wrong: {must_wrong:?}");
    let min = std::env::var("HOMOGRAPH_MIN_ACC").ok().and_then(|v| v.parse().ok()).unwrap_or(0.85);
    assert!(acc >= min, "accuracy {acc:.3} < {min}");
}
