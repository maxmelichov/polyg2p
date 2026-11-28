//! Amharic gemination layer: the table lookup and its guards. No data directory is needed.
use polyg2p::{G2p, Lang, am};

#[test]
fn no_table_is_the_plain_reading() {
    let plain = G2p::new(None);
    let empty = G2p::new(None).with_amharic(am::Gem::from_table(None));
    for s in ["ምሽት አምስት ሀብታምነት።", "ሰላም ነው! እንዴት ነህ?"] {
        assert_eq!(plain.phonemize(s, Lang::Am), empty.phonemize(s, Lang::Am));
        assert!(!plain.phonemize(s, Lang::Am).contains('ː'));
    }
}

#[test]
fn table_marks_its_words_only() {
    // ምሽት has no long letter, አምስት has ም long (U+135F after the letter)
    let table = "ምሽት\tምሽት\nአምስት\tአም\u{135F}ስት\n";
    let g = G2p::new(None).with_amharic(am::Gem::from_table(Some(table)));
    let plain = G2p::new(None);
    assert_eq!(g.phonemize("አምስት", Lang::Am), "amːɨst");
    assert_eq!(g.phonemize("ምሽት", Lang::Am), plain.phonemize("ምሽት", Lang::Am));
    // a word the table does not list keeps the plain reading
    assert_eq!(g.phonemize("ሰላም ነው", Lang::Am), plain.phonemize("ሰላም ነው", Lang::Am));
    // a homophone spelling finds the same row (ሥ folds to ስ)
    let t2 = "አምስት\tአም\u{135F}ስት\n";
    let g2 = G2p::new(None).with_amharic(am::Gem::from_table(Some(t2)));
    assert_eq!(g2.phonemize("አምሥት", Lang::Am), "amːɨst");
}

#[test]
fn guards_override_the_table() {
    // a mark on a first letter or a final bare letter is dropped
    let table = "ሰላም\tሰ\u{135F}ላም\u{135F}\n";
    let g = G2p::new(None).with_amharic(am::Gem::from_table(Some(table)));
    assert_eq!(g.phonemize("ሰላም", Lang::Am), G2p::new(None).phonemize("ሰላም", Lang::Am));
}
