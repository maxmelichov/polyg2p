//! Amharic verb rules (optional pack am-verbs.tsv): a one-root, three-template pack written inline.
use polyg2p::{G2p, Lang, am};

// root s-b-r (ሰበረ); perfect: 2nd radical long; passive imperfect ይሰበራል: 1st radical long; jussive ይሰብር: nothing
const PACK: &str = "R\ts b r\nR\tm r tʼ\n\
K\t0\tR0:ə:0 R1:ə:0 R2:ə:0\t1\n\
K\t0\tXተ R0:ə:0 R1:ə:0 R2:ə:0\t2\n\
K\t3\tXይ R0:ə:0 R1:ə:0 R2:a:0 Xል\t1\n\
K\t4\tXይ R0:ə:0 R1::0 R2::0\t\n";

fn with(pack: Option<&str>, table: Option<&str>) -> G2p {
    G2p::new(None).with_amharic(am::Gem::from_packs(table, pack))
}

fn long_flags(g: &am::Gem, w: &str) -> Vec<bool> {
    g.flags(&[w]).remove(0)
}

#[test]
fn no_pack_is_the_plain_reading() {
    let plain = G2p::new(None);
    let none = with(None, None);
    for s in ["ሰበረ ተሰበረ ይሰበራል ሰበረች።", "ምሽት አምስት ሀብታምነት።"] {
        assert_eq!(plain.phonemize(s, Lang::Am), none.phonemize(s, Lang::Am));
    }
}

#[test]
fn perfect_forms_of_a_known_root_get_the_long_second_radical() {
    let g = am::Gem::from_packs(None, Some(PACK));
    let want = vec![false, true, false]; // ሰ በ ረ: long በ
    assert_eq!(long_flags(&g, "ሰበረ"), want);
    // person endings of the last radical, a prefix chain and a clitic keep the mark
    for w in ["ሰበረች", "ሰበሩ", "ሰበርኩ", "በሰበረ"] {
        let f = long_flags(&g, w);
        assert_eq!(f.iter().filter(|&&x| x).count(), 1, "{w}");
    }
    assert_eq!(long_flags(&g, "ሰበረና")[1], true);
    // passive perfect marks the second radical of the stem (ተ ሰ በ ረ: letter 2)
    assert_eq!(long_flags(&g, "ተሰበረ"), vec![false, false, true, false]);
    // the reading changes the phonemes
    let marked = with(Some(PACK), None).phonemize("ሰበረ", Lang::Am);
    assert!(marked.contains('ː'), "{marked}");
    assert_eq!(marked, with(None, Some("ሰበረ\tሰበ\u{135F}ረ\n")).phonemize("ሰበረ", Lang::Am));
}

#[test]
fn unknown_roots_nouns_and_ambiguous_readings_stay_plain() {
    let g = am::Gem::from_packs(None, Some(PACK));
    for w in ["ቀጠረ", "ሰበራ", "ሰላም", "ምሽት"] {
        assert!(long_flags(&g, w).iter().all(|&x| !x), "{w}");
    }
    // the jussive template has no long letter: ይሰብር is a veto-free plain reading
    assert!(long_flags(&g, "ይሰብር").iter().all(|&x| !x));
}

#[test]
fn the_word_table_wins_over_the_rules() {
    // the table lists ሰበረ without a long letter: the verb rules never see it
    let g = am::Gem::from_packs(Some("ሰበረ\tሰበረ\n"), Some(PACK));
    assert!(long_flags(&g, "ሰበረ").iter().all(|&x| !x));
    // a word the table does not list falls through to the rules
    assert!(long_flags(&g, "ሰበረች").iter().any(|&x| x));
}
