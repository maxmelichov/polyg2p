//! English accent variants on whole sentences (tests/en_accent_cases.tsv).
//!
//! `codes` needs nothing. `sentences` and `words` need the English lexicon and are
//! marked `#[ignore]`: run them with
//!   POLYG2P_DATA=<dir with en.lex and en.g2p> cargo test --test en_accent -- --include-ignored
//! They fail (they do not skip) when that folder has no en.lex. The cases were
//! written with the default English model; unset POLYG2P_EN_MODEL for them.
//! The `en` rows pin plain English, which the accent layer must never change.
use polyg2p::{G2p, Lang};

const CODES: [&str; 7] = ["en-gb", "en-au", "en-nz", "en-in", "en-sc", "en-ie", "en-ca"];

fn engine() -> G2p {
    let dir = std::env::var_os("POLYG2P_DATA").map(std::path::PathBuf::from).expect("set POLYG2P_DATA to a folder with en.lex and en.g2p");
    assert!(dir.join("en.lex").exists(), "POLYG2P_DATA has no en.lex");
    assert!(std::env::var("POLYG2P_EN_MODEL").map_or(true, |v| v != "large"), "the cases were written with the default model: unset POLYG2P_EN_MODEL");
    // the cases pin the context rules; the optional heteronym model reads some of their sentences differently
    assert!(
        !dir.join("en_homograph.g2p").exists() || std::env::var("POLYG2P_EN_HOMOGRAPH").is_ok_and(|v| v == "off"),
        "the cases were written with the context rules: set POLYG2P_EN_HOMOGRAPH=off when the folder has en_homograph.g2p"
    );
    G2p::new(Some(&dir))
}

#[test]
fn codes() {
    for c in CODES {
        let l = Lang::parse(c).unwrap();
        assert_eq!(l.code(), c);
        assert_eq!(l.base(), Lang::En);
        assert!(l.accent().is_some() && l.implemented());
    }
    assert!(Lang::En.accent().is_none());
    assert_eq!(Lang::parse("en-us"), Some(Lang::En));
    // aliases, in any case and with either separator
    for (alias, code) in [("en_GB", "en-gb"), ("EN-UK", "en-gb"), ("en_uk", "en-gb"), ("en_au", "en-au"), ("en_sc", "en-sc"), ("en-gb-scotland", "en-sc"), ("en_IE", "en-ie")] {
        assert_eq!(Lang::parse(alias).map(|l| l.code()), Some(code), "{alias}");
    }
    // unknown codes are refused, not read as plain English
    for bad in ["en-xx", "en-za", "en-", "gb", "en-gbx", "english"] {
        assert_eq!(Lang::parse(bad), None, "{bad}");
    }
}

#[test]
#[ignore = "needs POLYG2P_DATA (en.lex, en.g2p)"]
fn sentences() {
    let g = engine();
    let cases = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/en_accent_cases.tsv")).unwrap();
    let mut n = 0;
    for line in cases.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 3, "bad case line: {line}");
        assert_eq!(g.phonemize(f[1], Lang::parse(f[0]).unwrap()), f[2], "{}: {}", f[0], f[1]);
        n += 1;
    }
    assert!(n >= 70, "too few cases: {n}");
}

#[test]
#[ignore = "needs POLYG2P_DATA (en.lex, en.g2p)"]
fn words() {
    let g = engine();
    let say = |code: &str, text: &str| g.phonemize(text, Lang::parse(code).unwrap());
    // linking r: kept before a vowel, dropped before a pause and at the end
    assert_eq!(say("en-gb", "far away"), "fˈɑːɹ əwˈeɪ");
    assert_eq!(say("en-gb", "far, away"), "fˈɑː, əwˈeɪ");
    assert_eq!(say("en-gb", "far."), "fˈɑː.");
    // hyphenated compounds: each part through the word list, linking r inside
    assert_eq!(say("en-gb", "half-past"), "hˈɑːf pˈɑːst");
    assert_eq!(say("en-gb", "far-off"), "fˈɑːɹ ˈɒf");
    // an inflected form of a listed word keeps the listed stem, also with t + s in one letter
    assert_eq!(say("en-gb", "aunts"), "ˈɑːnʦ");
    assert_eq!(say("en-gb", "commands"), "kəmˈɑːnʣ");
    assert_eq!(say("en-gb", "passes"), "pˈɑːsɪz");
    assert_eq!(say("en-gb", "dancing"), "dˈɑːnsɪŋ");
    // of / from are strong before a pause, weak elsewhere
    assert_eq!(say("en-gb", "What is it made of?"), "wɒt ɪz ɪt mˈeɪd ɒv?");
    assert_eq!(say("en-gb", "made of it"), "mˈeɪd əv ɪt");
    assert_eq!(say("en-gb", "Where is he from, then?"), "wˌeəɹ ɪz hiː fɹɒm, ðˈɛn?");
    // the reading the engine chose picks the British reading
    assert_eq!(say("en-gb", "a record"), "ə ɹˈɛkɔːd");
    assert_eq!(say("en-gb", "to record"), "tə ɹɪkˈɔːd");
    // words the checked list leaves to the rules
    assert_eq!(say("en-gb", "geese"), "ɡˈiːs");
    assert_eq!(say("en-gb", "glory"), "ɡlˈɔːɹi");
    // bin is not been: only the spelling "been" takes the weak form
    for code in ["en-gb", "en-au", "en-nz", "en-in"] {
        for t in ["The bin is full.", "Put it in the bin.", "bin Laden", "rubbish bin"] {
            assert!(!say(code, t).contains("biːn"), "{code}: {t}");
            assert!(say(code, t).contains("bˈɪn"), "{code}: {t}");
        }
        assert!(say(code, "Where have you been?").contains("bˈiːn"), "{code}");
        assert!(say(code, "I have been there.").contains("həvbiːn"), "{code}");
    }
    // no doubled linking r when the stem already ends in one
    for code in ["en-gb", "en-au", "en-nz", "en-in"] {
        for w in ["contrarily", "wirier", "wiriest"] {
            assert!(!say(code, w).contains("ɹɹ"), "{code}: {w}");
        }
    }
    assert_eq!(say("en-gb", "contrarily"), "kˈɒntɹəɹili");
    // Scottish THOUGHT stays long before ɪ so drawing / sawing do not collide with CHOICE (boy)
    assert_eq!(say("en-sc", "drawing"), "dɹˈɔːɪŋ");
    assert_eq!(say("en-sc", "sawing"), "sˈɔːɪŋ");
    assert_eq!(say("en-sc", "boy"), "bˈɔɪ");
    // every code leaves plain English alone
    let plain = say("en", "The kitten had written it during the far-off record of the tomato schedule.");
    assert_eq!(plain, "ðə kˈɪʔn̩ hæd ɹˈɪʔn̩ ɪt dˈʊɹɪŋ ðə fˈɑːɹ ˈɔːf ɹˈɛkɚd ʌvðə təmˈeɪtˌoʊ skˈɛʤʊl.");
}
