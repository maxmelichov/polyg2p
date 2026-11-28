//! Heteronym context models (src/hetero.rs, data/hetero_<lang>.tsv).
//!
//! `tables` and `no_model_unchanged` need nothing. `cases` needs the model files and skips (does not
//! fail) when a language's `<lang>-ctx.g2p` is not in POLYG2P_DATA, like the Hebrew tests:
//!   POLYG2P_DATA=<dir with pt-ctx.g2p, it-ctx.g2p ...> cargo test --test hetero -- --nocapture
//! `tests/hetero_cases.tsv`: hand-written sentences, the class the reading must have; a leading `!` marks a
//! case the shipped model misses (counted and printed, not asserted).
//! `tests/hetero_fixture.tsv`: the output of the main-branch build for the same sentences, with no model.
use polyg2p::{G2p, Lang};

const LANGS: [&str; 7] = ["fr", "de", "pt", "uk", "it", "nl", "ro"];

#[test]
fn tables() {
    let g = G2p::new(None);
    for l in LANGS {
        let path = format!("{}/data/hetero_{l}.tsv", env!("CARGO_MANIFEST_DIR"));
        let t = std::fs::read_to_string(&path).unwrap();
        let mut rows = 0;
        for line in t.lines().filter(|x| !x.is_empty() && !x.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            assert!(f.len() >= 5, "{l}: short row {line}");
            let k: u8 = f[1].parse().unwrap_or_else(|_| panic!("{l}: bad class in {line}"));
            assert!(k < 4, "{l}: class out of range in {line}");
            let r = f[2];
            if let Some(s) = r.strip_prefix("respell:") {
                assert!(!g.phonemize(s, Lang::parse(l).unwrap()).is_empty(), "{l}: respelling gives nothing: {line}");
            } else if let Some(p) = r.strip_prefix("ph:") {
                assert!(!p.is_empty() && !p.contains(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit() || c.is_whitespace() || c == '+'), "{l}: odd phonemes in {line}");
            } else if let Some(s) = r.strip_prefix("sub:") {
                assert!(s.contains('>'), "{l}: sub needs from>to: {line}");
            } else {
                assert_eq!(r, "rule", "{l}: unknown reading kind in {line}");
            }
            rows += 1;
        }
        assert!(rows >= 2, "{l}: empty table");
    }
}

#[test]
fn no_model_unchanged() {
    // a data directory without model files: the module must leave the rules' output alone
    let empty = std::env::temp_dir().join("polyg2p_hetero_empty");
    let _ = std::fs::create_dir_all(&empty);
    let g = G2p::new(Some(&empty));
    let fx = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hetero_fixture.tsv")).unwrap();
    let mut n = 0;
    for line in fx.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 3, "bad fixture line: {line}");
        assert_eq!(g.phonemize(f[1], Lang::parse(f[0]).unwrap()), f[2], "{}: {}", f[0], f[1]);
        n += 1;
    }
    assert!(n >= 60, "fixture too small: {n}");
}

#[test]
fn cases() {
    let Some(dir) = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from) else {
        eprintln!("skipped: POLYG2P_DATA not set");
        return;
    };
    let g = G2p::new(Some(&dir));
    let cases = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hetero_cases.tsv")).unwrap();
    for l in LANGS {
        if !dir.join(format!("{l}-ctx.g2p")).exists() {
            eprintln!("{l}: skipped (no {l}-ctx.g2p in POLYG2P_DATA)");
            continue;
        }
        let (mut ok, mut miss, mut known) = (0, 0, 0);
        for line in cases.lines().filter(|x| !x.is_empty() && !x.starts_with('#')) {
            let (known_miss, line) = match line.strip_prefix('!') {
                Some(r) => (true, r),
                None => (false, line),
            };
            let f: Vec<&str> = line.split('\t').collect();
            if f[0] != l {
                continue;
            }
            let nth: usize = f[3].parse().unwrap();
            let want: u8 = f[4].parse().unwrap();
            let d = g.hetero_decisions(f[1], Lang::parse(l).unwrap());
            let got = d.iter().filter(|d| d.word == f[2]).nth(nth).and_then(|d| if d.mode_rule { d.rule_k } else { d.model_k });
            if got == Some(want) {
                ok += 1;
            } else if known_miss {
                known += 1;
            } else {
                miss += 1;
                eprintln!("{l}: wanted class {want}, got {got:?}: {}", f[1]);
            }
        }
        eprintln!("{l}: {ok} right, {known} known misses, {miss} new misses");
        assert_eq!(miss, 0, "{l}: new misses");
    }
}
