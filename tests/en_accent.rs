//! English accent variants on whole sentences (tests/en_accent_cases.tsv). The
//! cases need the English lexicon: set POLYG2P_DATA to a folder with en.lex and
//! en.g2p; without it the test only checks the codes parse. The `en` rows pin
//! plain English, which the accent layer must never change.
use polyg2p::{G2p, Lang};

#[test]
fn codes_parse() {
    for c in ["en-gb", "en-au", "en-nz", "en-za", "en-in", "en-sc", "en-ie", "en-ca"] {
        let l = Lang::parse(c).unwrap();
        assert_eq!(l.code(), c);
        assert_eq!(l.base(), Lang::En);
        assert!(l.accent().is_some() && l.implemented());
    }
    assert!(Lang::En.accent().is_none());
    assert_eq!(Lang::parse("en-us"), Some(Lang::En));
}

#[test]
fn sentences() {
    let Some(dir) = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from).filter(|d| d.join("en.lex").exists()) else {
        eprintln!("POLYG2P_DATA has no en.lex: sentence cases skipped");
        return;
    };
    if std::env::var("POLYG2P_EN_MODEL").is_ok_and(|v| v == "large") {
        eprintln!("cases were written with the default model: skipped under POLYG2P_EN_MODEL=large");
        return;
    }
    let g = G2p::new(Some(&dir));
    let cases = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/en_accent_cases.tsv")).unwrap();
    let mut n = 0;
    for line in cases.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 3, "bad case line: {line}");
        assert_eq!(g.phonemize(f[1], Lang::parse(f[0]).unwrap()), f[2], "{}: {}", f[0], f[1]);
        n += 1;
    }
    assert!(n >= 50, "too few cases: {n}");
}
