//! Amharic (tests/am_cases.tsv, tests/am_fidel.tsv). Rules only: no data directory needed.
//!
//! `am_cases.tsv` has one labelled case per line: the rule it pins, the input and
//! the expected output of `phonemize`. `am_fidel.tsv` lists the reading of every
//! letter of the Ethiopic block that the module reads.
use polyg2p::{G2p, Lang, am};

fn rows(file: &str) -> Vec<Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

#[test]
fn codes() {
    for c in ["am", "amh", "AM", "am-ET", "am_et"] {
        assert_eq!(Lang::parse(c), Some(Lang::Am), "{c}");
    }
    assert_eq!(Lang::Am.code(), "am");
    assert!(Lang::Am.implemented() && Lang::ALL.contains(&Lang::Am));
}

#[test]
fn cases() {
    let g = G2p::new(None);
    let cases = rows("am_cases.tsv");
    assert!(cases.len() >= 100, "am_cases.tsv is short: {}", cases.len());
    let mut bad = Vec::new();
    for c in &cases {
        let got = g.phonemize(&c[1], Lang::Am);
        if got != c[2] {
            bad.push(format!("[{}] {} -> {got} (expected {})", c[0], c[1], c[2]));
        }
    }
    assert!(bad.is_empty(), "{} of {} cases differ:\n{}", bad.len(), cases.len(), bad.join("\n"));
}

#[test]
fn letters() {
    let table = rows("am_fidel.tsv");
    let mut listed = std::collections::HashSet::new();
    for r in &table {
        let ch = char::from_u32(u32::from_str_radix(&r[0], 16).unwrap()).unwrap();
        assert_eq!(ch.to_string(), r[1], "U+{}", r[0]);
        let (c, lab, v) = am::fidel(ch).unwrap_or_else(|| panic!("U+{} {ch} is not read", r[0]));
        let cons = if lab { format!("{c}ʷ") } else { c.to_string() };
        assert_eq!((cons.as_str(), v), (r[2].as_str(), r.get(3).map_or("", String::as_str)), "U+{} {ch}", r[0]);
        listed.insert(ch);
    }
    // nothing else in the block is read as a letter
    for o in 0x1200u32..=0x139F {
        let ch = char::from_u32(o).unwrap();
        assert_eq!(am::fidel(ch).is_some(), listed.contains(&ch), "U+{o:04X}");
    }
}

#[test]
fn output_symbols() {
    // every symbol the module can write: the 7 vowels, the consonants, ʼ ʷ, kept punctuation
    const OK: &str = "əuiaeɨo hlmsrʃkbvtnɲʔwzʒjdɡŋpfʧʤʦʣʼʷ.,?!:'-";
    let g = G2p::new(None);
    for c in rows("am_cases.tsv") {
        let got = g.phonemize(&c[1], Lang::Am);
        assert!(got.chars().all(|x| OK.contains(x)), "{} -> {got}", c[1]);
    }
    let all: String = (0x1200u32..=0x135A).filter_map(char::from_u32).filter(|&c| am::fidel(c).is_some()).collect();
    for w in all.chars().collect::<Vec<_>>().chunks(5) {
        let got = g.phonemize(&w.iter().collect::<String>(), Lang::Am);
        assert!(!got.is_empty() && got.chars().all(|x| OK.contains(x)), "{got}");
    }
}

#[test]
fn tags_and_batch() {
    let g = G2p::new(None);
    assert_eq!(g.phonemize("ሰላም [laughs] ዓለም።", Lang::Am), "səlam [laughs] aləm.");
    let texts: Vec<String> = (0..200).map(|i| if i % 2 == 0 { "ሰላም፣ ዓለም።".to_string() } else { "ቤቶች".to_string() }).collect();
    let out = g.phonemize_batch(&texts, Lang::Am);
    assert_eq!(out.len(), 200);
    assert!(out.iter().step_by(2).all(|o| o == "səlam, aləm.") && out.iter().skip(1).step_by(2).all(|o| o == "betoʧ"));
}
