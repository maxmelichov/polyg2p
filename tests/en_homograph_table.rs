//! data/en_homograph.tsv: the reading table behind the English heteronym model.
//! The one-to-one mapping of its rows to the benchmark corpus labels is checked by
//! `python tools/en_homograph_bench.py <corpus>/data --check-table` (the corpus is not in this repository).
use std::collections::{HashMap, HashSet};

const TABLE: &str = include_str!("../data/en_homograph.tsv");
const CASES: &str = include_str!("en_homograph_cases.tsv");
/// Every symbol English phoneme strings may contain (the set en.lex uses).
const SYMBOLS: &str = "abdefhijklmnopstuvwzæðŋɐɑɔəɚɛɜɡɪɹɾʃʊʌʒʔʣʤʦʧˈˌː\u{329}θᵻ";
const CLASSES: [&str; 6] = ["N", "A", "V", "PAST", "NAME", "-"];

fn rows() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 4, "need word, reading, class, ipa: {l:?}");
            (f[0], f[1], f[2], f[3])
        })
        .collect()
}

#[test]
fn table_is_well_formed() {
    let rows = rows();
    assert!(rows.len() >= 300);
    let mut by_word: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    for (w, r, c, ipa) in &rows {
        assert!(w.chars().all(|c| c.is_ascii_lowercase()), "{w}");
        assert!(c.split(',').all(|x| CLASSES.contains(&x)), "{w} {r}: class {c}");
        let bad: Vec<char> = ipa.chars().filter(|ch| !SYMBOLS.contains(*ch)).collect();
        assert!(bad.is_empty(), "{w} {r}: symbols outside the vocabulary {bad:?} in {ipa}");
        assert!(ipa.contains('ˈ'), "{w} {r}: no primary stress in {ipa}");
        by_word.entry(w).or_default().push((r, ipa));
    }
    assert!(by_word.len() >= 160);
    for (w, rs) in &by_word {
        assert!(rs.len() >= 2, "{w} has one reading");
        let names: HashSet<_> = rs.iter().map(|x| x.0).collect();
        let ipas: HashSet<_> = rs.iter().map(|x| x.1).collect();
        assert_eq!(names.len(), rs.len(), "{w}: duplicate reading name");
        assert_eq!(ipas.len(), rs.len(), "{w}: two readings share one pronunciation");
    }
}

#[test]
fn cases_name_table_readings() {
    let mut known: HashSet<(&str, &str)> = HashSet::new();
    for (w, r, _, _) in rows() {
        known.insert((w, r));
    }
    let (mut n, mut must, mut words) = (0, 0, HashSet::new());
    for l in CASES.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let f: Vec<&str> = l.split('\t').collect();
        assert!(f.len() >= 3, "{l:?}");
        assert!(known.contains(&(f[1], f[2])), "no reading {} {} in the table", f[1], f[2]);
        assert!(f[0].to_lowercase().contains(f[1]), "{:?} lacks the word {}", f[0], f[1]);
        n += 1;
        must += (f.get(3) == Some(&"must")) as usize;
        words.insert(f[1]);
    }
    assert!(n >= 150 && words.len() >= 30, "{n} cases over {} words", words.len());
    assert_eq!(must, 4, "the four 'lead' sentences are the must-pass cases");
}
