//! Heteronym context models (src/hetero.rs, data/hetero_<lang>.tsv).
//!
//! `tables`, `no_model_unchanged` and the tagger-file tests need no model. `cases*`, `fr_never_leaks_into_phonemize`
//! and `long_line_passes_are_bounded` need the model files and skip (do not
//! fail) when a language's `<lang>-ctx.g2p` is not in POLYG2P_DATA, like the Hebrew tests:
//!   POLYG2P_DATA=<dir with pt-ctx.g2p, it-ctx.g2p ...> cargo test --test hetero -- --nocapture
//! `tests/hetero_cases.tsv`: hand-written sentences, the class the reading must have; a leading `!` marks a
//! case the shipped model misses (counted and printed, not asserted).
//! `tests/hetero_fixture.tsv`: the output of the main-branch build for the same sentences, with no model.
use polyg2p::{hetero, tagger::Tagger, G2p, Lang};

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

/// Hand-written cases through the model. Needs the model files (POLYG2P_DATA); a language without
/// its `<lang>-ctx.g2p` is skipped, not failed. Only the languages that ship a model are asserted here;
/// French (measured and dropped by the gate) is checked in `cases_fr_capture_only`.
fn run_cases(langs: &[&str]) {
    let Some(dir) = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from) else {
        eprintln!("skipped: POLYG2P_DATA not set (this test needs the <lang>-ctx.g2p files)");
        return;
    };
    let g = G2p::new(Some(&dir));
    let cases = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hetero_cases.tsv")).unwrap();
    for &l in langs {
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
            let got = d.iter().filter(|d| d.word == f[2]).nth(nth).and_then(|d| d.shipped_k());
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

#[test]
fn cases() {
    run_cases(&["pt", "it"]);
}

/// French is not shipped: `phonemize` never loads `fr-ctx.g2p`. The model is only reachable through
/// the explicit capture mode (evaluation), which is what this test uses.
#[test]
fn cases_fr_capture_only() {
    run_cases(&["fr"]);
}

/// With models present, ordinary phonemize of a non-shipped language equals the no-model output, also
/// right after a capture ran on the same thread (the capture must not leak).
#[test]
fn fr_never_leaks_into_phonemize() {
    let Some(dir) = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from) else {
        eprintln!("skipped: POLYG2P_DATA not set");
        return;
    };
    let g = G2p::new(Some(&dir));
    let empty = std::env::temp_dir().join("polyg2p_hetero_empty");
    let _ = std::fs::create_dir_all(&empty);
    let rules = G2p::new(Some(&empty));
    let s = "Je sens une odeur de brûlé. Tu sens la rose.";
    {
        let cap = hetero::capture(false);
        let _ = g.phonemize(s, Lang::Fr);
        assert!(!cap.take().decisions.is_empty());
    }
    assert_eq!(g.phonemize(s, Lang::Fr), rules.phonemize(s, Lang::Fr));
    // a capture that is dropped without take() does not leak either
    drop(hetero::capture(true));
    assert_eq!(g.phonemize(s, Lang::Fr), rules.phonemize(s, Lang::Fr));
}

/// One forward pass per window, not per hit: a long line full of listed words is bounded in passes.
/// Needs pt-ctx.g2p. Asserts the pass count (not wall time, which flakes under load); the time is printed.
#[test]
fn long_line_passes_are_bounded() {
    let Some(dir) = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from) else {
        eprintln!("skipped: POLYG2P_DATA not set (needs pt-ctx.g2p)");
        return;
    };
    if !dir.join("pt-ctx.g2p").exists() {
        eprintln!("skipped: no pt-ctx.g2p");
        return;
    }
    let g = G2p::new(Some(&dir));
    let line = "gosto ".repeat(3000);
    let cap = hetero::capture(false);
    let t = std::time::Instant::now();
    let _ = g.phonemize(&line, Lang::Pt);
    let el = t.elapsed();
    let c = cap.take();
    eprintln!("3000 hits: {} passes, {el:?}", c.passes);
    assert_eq!(c.decisions.len(), 3000);
    assert!(c.passes >= 1 && c.passes <= hetero::MAX_PASSES, "{} passes", c.passes);
    // hits beyond the cap keep the rules
    assert!(c.decisions.last().unwrap().model_k.is_none());
    assert!(c.decisions.first().unwrap().model_k.is_some());
    // a sentence that fits one window costs one pass, however many hits
    let _ = g.phonemize("Eu gosto de café, ele gosto de chá, tu gosto de pão.", Lang::Pt);
    assert_eq!(cap.take().passes, 1);
}

fn valid_model() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ru-ctx.g2p")).expect("data/ru-ctx.g2p ships with the repo")
}

/// A truncated or corrupt model file is an error (the engine then keeps the rules), never a panic.
#[test]
fn truncated_and_garbage_models_are_errors() {
    let good = valid_model();
    assert!(Tagger::from_bytes(&good).is_ok(), "the shipped file loads");
    let hl = u32::from_le_bytes(good[4..8].try_into().unwrap()) as usize;
    let cuts = [0, 3, 4, 7, 8, 9, 8 + hl / 2, 8 + hl - 1, 8 + hl, 8 + hl + 1, good.len() / 3, good.len() / 2, good.len() - 4, good.len() - 1];
    for &n in &cuts {
        assert!(Tagger::from_bytes(&good[..n]).is_err(), "cut at {n}");
    }
    // header length that points past the end / overflows
    for hl in [u32::MAX, good.len() as u32, 0, 1] {
        let mut b = good.clone();
        b[4..8].copy_from_slice(&hl.to_le_bytes());
        assert!(Tagger::from_bytes(&b).is_err(), "header length {hl}");
    }
    // garbage
    assert!(Tagger::from_bytes(b"").is_err());
    assert!(Tagger::from_bytes(b"G2PT").is_err());
    assert!(Tagger::from_bytes(&[0u8; 4096]).is_err());
    let mut junk = b"G2PT".to_vec();
    junk.extend_from_slice(&16u32.to_le_bytes());
    junk.extend_from_slice(&[0xffu8; 4096]);
    assert!(Tagger::from_bytes(&junk).is_err());
    // flipping bytes in the header region never panics (Ok or Err, both fine)
    for i in (8..8 + hl).step_by(97) {
        let mut b = good.clone();
        b[i] ^= 0xff;
        let _ = Tagger::from_bytes(&b);
    }
}

/// The engine falls back to the rules when its data directory holds a truncated model file.
#[test]
fn truncated_model_file_falls_back_to_rules() {
    let good = valid_model();
    let dir = std::env::temp_dir().join(format!("polyg2p_hetero_trunc_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("pt-ctx.g2p"), &good[..good.len() / 2]).unwrap();
    let empty = std::env::temp_dir().join("polyg2p_hetero_empty");
    let _ = std::fs::create_dir_all(&empty);
    let (g, rules) = (G2p::new(Some(&dir)), G2p::new(Some(&empty)));
    let s = "Eu gosto de café. Olho por olho.";
    assert_eq!(g.phonemize(s, Lang::Pt), rules.phonemize(s, Lang::Pt));
    let _ = std::fs::remove_file(dir.join("pt-ctx.g2p"));
    let _ = std::fs::remove_dir(&dir);
}

/// The valid shipped Russian tagger still loads and runs (same loader as Hebrew).
#[test]
fn valid_tagger_still_runs() {
    let t = Tagger::from_bytes(&valid_model()).unwrap();
    let lg = t.forward(&t.encode("замок на двери"));
    assert_eq!(lg.len, "замок на двери".chars().count());
    assert!(lg.vow.iter().all(|v| v.is_finite()));
}
