//! The Hebrew IPA renderer must reproduce RenikudPlus's output exactly from
//! RenikudPlus's own per-letter labels (data/he_fixture.tsv, 300 sentences).
use polyg2p::he;

#[test]
fn renderer_reproduces_teacher_ipa() {
    let fx = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/he_fixture.tsv")).unwrap();
    let mut n = 0;
    for line in fx.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 5, "bad fixture line");
        let got = he::render_codes(f[0], f[1], f[2], f[3]);
        assert_eq!(got, f[4], "text: {}", f[0]);
        n += 1;
    }
    assert!(n >= 200, "fixture too small: {n}");
}

#[test]
fn numbers_and_normalization() {
    assert_eq!(he::num::normalize("המחיר 1250 שקלים"), "המחיר אלף מאתיים וחמישים שקלים");
    assert_eq!(he::num::normalize("ב-3 במאי 2021"), "בשלושה במאי אלפיים עשרים ואחת");
    assert_eq!(he::num::normalize("25% מהם"), "עשרים וחמישה אחוז מהם");
    assert_eq!(he::num::normalize("בשעה 14:30"), "בשעה שתיים וחצי");
    assert_eq!(he::normalize("שָׁלוֹם"), "שלום");
}

#[test]
fn engine_without_model_is_empty() {
    let h = he::Hebrew::load(None);
    if !h.has_model() {
        assert_eq!(h.phonemize("שלום"), "");
    }
}
