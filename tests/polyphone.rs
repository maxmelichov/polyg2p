//! Polyphone reader: a tiny fixture model (tests/data/zh_poly_fixture.g2p, trained by tools/train_polyphone.py on
//! the hand-written sentences of tests/zh_poly_cases.tsv, not on any corpus) must give the labelled reading for every
//! case, through the same code path real models take (G2PT container, int8 weights, masked head, windows).
use polyg2p::polyphone::{PolyLang, Polyphone};

fn model() -> Polyphone {
    Polyphone::open(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/zh_poly_fixture.g2p"))).unwrap()
}

fn cases() -> Vec<(String, usize, String)> {
    let txt = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/zh_poly_cases.tsv")).unwrap();
    txt.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 3, "bad case line: {l}");
            (f[0].to_string(), f[1].parse().unwrap(), f[2].to_string())
        })
        .collect()
}

#[test]
fn every_case_reads_as_labelled() {
    let m = model();
    let cs = cases();
    assert!(cs.len() >= 50);
    for (s, i, want) in &cs {
        let r = m.readings(s);
        let got = r.iter().find(|x| x.index == *i).unwrap_or_else(|| panic!("no reading at {i} in {s}"));
        assert_eq!(&got.reading, want, "{s} [{i}]");
        assert!(got.prob > 0.0 && got.prob <= 1.0);
    }
}

#[test]
fn readings_are_character_indices_and_cover_known_chars_only() {
    let m = model();
    let r = m.readings("银行行长abc，了");
    let idx: Vec<usize> = r.iter().map(|x| x.index).collect();
    assert_eq!(idx, vec![1, 2, 3, 8]);
    assert!(m.readings("你好吗").iter().all(|x| m.knows('好') && x.index == 1));
    assert!(m.readings("abc 123").is_empty());
}

#[test]
fn long_text_is_read_in_windows() {
    let m = model();
    // longer than the model window (128): a target is read in the 128-character window centred on it,
    // i.e. exactly as if that slice had been given alone
    let pad = "我们今天去公园玩".repeat(12);
    let s = format!("{pad}他的行为很长{pad}");
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    assert!(n > 128);
    let i = chars.iter().position(|&c| c == '行').unwrap();
    let st = i.saturating_sub(64).min(n - 128);
    let slice: String = chars[st..st + 128].iter().collect();
    let want = m.readings(&slice).into_iter().find(|r| r.index == i - st).unwrap();
    let got = m.readings(&s).into_iter().find(|r| r.index == i).unwrap();
    assert_eq!(got.reading, want.reading);
    assert_eq!(got.prob, want.prob);
    // a known character in the last 64 positions is still read (window clamped to the end)
    let s2 = format!("{pad}{pad}他的行为很长");
    let n2 = s2.chars().count();
    assert_eq!(m.readings(&s2).last().unwrap().index, n2 - 1);
}

#[test]
fn lang_codes_and_missing_model() {
    assert_eq!(PolyLang::parse("zh-TW"), Some(PolyLang::Zh));
    assert_eq!(PolyLang::parse("YUE"), Some(PolyLang::Yue));
    assert_eq!(PolyLang::parse("ja"), Some(PolyLang::Ja));
    assert_eq!(PolyLang::parse("en"), None);
    // no data directory: the engine answers with nothing rather than failing
    let g = polyg2p::G2p::new(None);
    assert!(g.readings("银行", "zh").is_empty());
    assert!(g.readings("银行", "xx").is_empty());
}

#[test]
fn engine_loads_model_from_data_dir() {
    let d = std::env::temp_dir().join(format!("polyg2p_poly_{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/zh_poly_fixture.g2p"), d.join("zh-poly.g2p")).unwrap();
    let g = polyg2p::G2p::new(Some(&d));
    let r = g.readings("我去银行办事", "zh");
    assert_eq!(r, vec![(3, "hang2".to_string())]);
    let _ = std::fs::remove_file(d.join("zh-poly.g2p"));
    let _ = std::fs::remove_dir(&d);
}

#[test]
fn a_hebrew_tagger_file_is_not_a_polyphone_model() {
    let he = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ru-ctx.g2p")).unwrap();
    assert!(Polyphone::from_bytes(&he).is_err());
}
