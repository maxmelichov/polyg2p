//! European label fixes: whole-sentence cases for the rule fixes of this branch (no data directory needed).
use polyg2p::{G2p, Lang};

fn p(l: &str, t: &str) -> String {
    G2p::new(None).phonemize(t, Lang::parse(l).unwrap())
}

/// A sentence-initial capital one-letter preposition is the word (Wiktionary pl `w`, `z`; cs `s`, `z`, `k`, `v`):
/// the text reads exactly as its lower-case spelling; a lone capital elsewhere stays a letter name.
#[test]
fn capital_prepositions() {
    for (l, a, b) in [
        ("pl", "W tej chwili.", "w tej chwili."), ("pl", "Z tego.", "z tego."),
        ("cs", "S tebou.", "s tebou."), ("cs", "Z toho.", "z toho."), ("cs", "K domu.", "k domu."), ("cs", "V domě.", "v domě."),
        ("cs", "Byl tam. S cizími věcmi.", "Byl tam. s cizími věcmi."),
    ] {
        assert_eq!(p(l, a), p(l, b), "{a}");
    }
    assert!(p("cs", "Je to vitamín K.").ends_with("kˈaː."));
    assert!(p("pl", "To jest punkt Z.").ends_with("zˈɛt."));
}

/// Pre-1934 Dutch spelling: sch after a consonant (end / -e / -en) and ssch read s; real sch words are untouched.
#[test]
fn dutch_old_spelling() {
    for (a, b) in [
        ("menschen", "mensen"), ("mensch", "mens"), ("Duitsche", "Duitse"), ("Haagsche", "Haagse"), ("versch", "vers"),
        ("tusschen", "tussen"), ("wasschen", "wassen"), ("frissche", "frisse"), ("Vlaamsche", "Vlaamse"), ("Fransche", "Franse"),
    ] {
        assert_eq!(p("nl", a), p("nl", b), "{a}");
    }
    for w in ["school", "handschoen", "misschien", "geschenk", "schenen", "schip"] {
        assert!(p("nl", w).contains("sx"), "{w}: {}", p("nl", w));
    }
    // archaic articles are unstressed; 't / 's without the apostrophe; Den Haag keeps its full vowel
    assert_eq!(p("nl", "eene"), "ənə");
    assert!(p("nl", "der man").starts_with("dɛr "));
    assert_eq!(p("nl", "t is dat"), p("nl", "'t is dat"));
    assert_eq!(p("nl", "s avonds"), p("nl", "'s avonds"));
    assert!(p("nl", "Den Haag").starts_with("dɛn "));
}

/// Abbreviations are expanded and their own period is dropped mid-sentence (sv, cs); sv `Tom` the name.
#[test]
fn abbreviations_sv_cs() {
    assert_eq!(p("sv", "T.ex. kan man det."), p("sv", "Till exempel kan man det."));
    assert_eq!(p("sv", "Vi kom, d.v.s. vi gick."), p("sv", "Vi kom, det vill säga vi gick."));
    assert_eq!(p("sv", "Det finns bl.a. katter."), p("sv", "Det finns bland annat katter."));
    assert_eq!(p("cs", "Tzv. nové věci."), p("cs", "Takzvaný nové věci."));
    assert_eq!(p("cs", "Koupil chleba, mléko atd. Pak šel."), p("cs", "Koupil chleba, mléko a tak dále. Pak šel."));
    assert!(p("sv", "Tom är här.").starts_with("tˈɔm "));
    assert!(p("sv", "en tom flaska").contains("tˈuːm"));
}

/// Turkish -ca/-ce takes no stress, -ınca/-ince stresses its own first vowel; stems that merely end in them keep final stress.
#[test]
fn turkish_ce_stress() {
    assert_eq!(p("tr", "güzelce"), "ɟyzˈælʤɛ");
    assert_eq!(p("tr", "gelince"), "ɟelˈɪnʤɛ");
    assert!(p("tr", "İngilizce").contains("lˈɪzʤɛ"));
    for (w, ph) in [("gece", "ɟeʤˈɛ"), ("hoca", "hoʤˈa"), ("bahçe", "bahʧˈɛ"), ("karınca", "kaɾɯnʤˈa"), ("Türkçe", "tˈʏrkʧɛ"), ("bence", "bˈænʤɛ")] {
        assert_eq!(p("tr", w), ph, "{w}");
    }
}

/// French compass points and the verb ending -tions after nous (fr.wiktionary nord-est, portions).
#[test]
fn french_compass_and_tions() {
    assert!(p("fr", "le nord-est").ends_with("nɔʁˈɛst"));
    assert!(p("fr", "le sud-ouest.").contains("sydwˈɛst"));
    assert!(p("fr", "à l'est").ends_with("lˈɛst"));
    assert_eq!(p("fr", "nous portions des sacs"), "nu pɔʁtjˈɔ̃ de sˈak");
    assert!(p("fr", "nous ne portions pas").contains("pɔʁtjˈɔ̃"));
    assert!(p("fr", "les portions sont petites").contains("pɔʁsjˈɔ̃"));
}

/// Ordinals written with a digit: cs/hu `N.` before a lower-case word, el digits + ending.
#[test]
fn digit_ordinals() {
    assert_eq!(p("cs", "5. května"), p("cs", "pátého května"));
    assert_eq!(p("cs", "21. ledna"), p("cs", "dvacátého prvního ledna"));
    assert_eq!(p("cs", "3. patro"), p("cs", "třetí patro"));
    assert_eq!(p("hu", "5. napon"), p("hu", "ötödik napon"));
    assert_eq!(p("hu", "21. század"), p("hu", "huszonegyedik század"));
    assert_eq!(p("hu", "32. nap"), p("hu", "harminckettedik nap"));
    // a sentence-ending number keeps its period and stays a cardinal
    assert_eq!(p("cs", "Je to 5."), p("cs", "Je to pět."));
    assert_eq!(p("el", "21ου αιώνα"), p("el", "εικοστού πρώτου αιώνα"));
    assert_eq!(p("el", "1η Μαΐου"), p("el", "πρώτη Μαΐου"));
    assert_eq!(p("el", "4ο"), p("el", "τέταρτο"));
    assert_eq!(p("el", "4ου"), p("el", "τετάρτου"));
}

/// Romanian final -i after a consonant: one palatal offglide.
#[test]
fn romanian_single_palatal() {
    assert_eq!(p("ro", "buni"), "bˈunʲ");
    for w in ["pomi", "oameni", "faceți", "nu-mi"] {
        assert!(!p("ro", w).contains("ʲʲ"), "{w}");
    }
    assert_eq!(p("ro", "aici"), "aˈiʧʲ");
    assert_eq!(p("ro", "lupii"), "lˈupiɪ");
}

/// Greek: a secondary stress after the primary survives only before an enclitic.
#[test]
fn greek_final_secondary() {
    assert_eq!(p("el", "άνθρωπος"), "ˈanθropos");
    assert_eq!(p("el", "αυτοκίνητο"), "ˌaftocˈinito");
    assert!(p("el", "άνθρωπος μου").contains("ˈanθropˌos") || p("el", "άνθρωπος μου").contains("ˈanθropˌoz"));
}
