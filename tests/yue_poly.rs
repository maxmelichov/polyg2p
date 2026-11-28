//! The shipped Cantonese model (data/yue-poly.g2p): the Rust reader must reproduce, for hand-written sentences, the
//! readings the PyTorch int8 loader gave when tests/yue_poly_cases.tsv was recorded (a parity pin, not a gold standard).
use polyg2p::polyphone::Polyphone;

#[test]
fn rust_matches_recorded_python_int8() {
    let m = Polyphone::open(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/data/yue-poly.g2p"))).unwrap();
    let txt = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/yue_poly_cases.tsv")).unwrap();
    let mut n = 0;
    for l in txt.lines().filter(|l| !l.is_empty()) {
        let f: Vec<&str> = l.split('\t').collect();
        let i: usize = f[1].parse().unwrap();
        let r = m.readings(f[0]);
        let got = r.iter().find(|x| x.index == i).unwrap_or_else(|| panic!("no reading at {i} in {}", f[0]));
        assert_eq!(got.reading, f[2], "{} [{i}]", f[0]);
        n += 1;
    }
    assert!(n >= 30);
}
