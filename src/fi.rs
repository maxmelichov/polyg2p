//! Finnish grapheme-to-phoneme rules, espeak-style notation.
//!
//! Finnish spelling is phonemic: every letter is one phone, doubled letters
//! are long, stress is fixed on the first syllable. What the rules add:
//!   - syllabification into nuclei (long vowels, the 18 diphthongs, hiatus)
//!     and syllable weight, for espeak's weight-sensitive secondary stress
//!     (third syllable, moved to the fourth when the third is light and the
//!     fourth heavy, then every second syllable, never the last);
//!   - compound boundaries (hyphens, vowel-harmony violations, numeral
//!     morphemes, a list of frequent first elements), where the stress
//!     pattern restarts with a secondary stress;
//!   - `ng` -> `ŋŋ`, `nk` -> `ŋk`, loan letters (c q w x z š ž å, sh/ch);
//!   - sentence sandhi (`join`): boundary gemination (*loppukahdennus*) after
//!     -e nominals, -lle, -nne, -sti, -tse and the common imperatives /
//!     infinitives, and word-final n -> ŋ/m before k/p, both as espeak
//!     writes them (`ˌoleh hˈyvæ`, `tˈule tːˈænne`, `mˈennæːŋ kˈotɪːn`).
//!
//! Notation follows espeak's `fi` voice: `a æ ø y` (Wiktionary writes ɑ),
//! `v` (Wiktionary ʋ), unstressed short/long `i` as `ɪ`/`ɪː`, long vowels
//! and long stops with `ː`, long sonorants and fricatives doubled (`ss`,
//! `ll`, `ŋŋ`), diphthongs as plain vowel pairs, stress marks right before
//! the vowel. Function words carry no stress (`ja`, `on`, `se`) or only a
//! secondary one (`ˌole`, `mˌinæ`), as in espeak's dictionary.

use crate::fi_verbs::{INFINITIVE, INFINITIVE_CTX};
use crate::text::Tok;
use crate::Out;
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "pilkku";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "aː",
    "beː",
    "seː",
    "deː",
    "eː",
    "æf",
    "ɡeː",
    "hoː",
    "iː",
    "jiː",
    "koː",
    "æl",
    "æm",
    "æn",
    "oː",
    "peː",
    "kuː",
    "ær",
    "æs",
    "teː",
    "uː",
    "veː",
    "kaksoisveː",
    "æks",
    "yː",
    "tset",
]);

/// Monosyllabic function words espeak reads without any stress mark.
const UNSTRESSED: &[&str] = &[
    "ja", "on", "ei", "se", "hän", "tai", "en", "et", "ne", "he", "vai", "me", "te",
];
/// Pronouns and forms of olla/lienee that only get a secondary stress.
const SECONDARY: &[&str] = &[
    "minä",
    "sinä",
    "ovat",
    "ole",
    "olleet",
    "ollut",
    "olin",
    "olit",
    "oli",
    "olimme",
    "olitte",
    "olivat",
    "olisin",
    "olisit",
    "olisi",
    "olisimme",
    "olisitte",
    "olisivat",
    "olen",
    "olet",
    "olla",
    "olemme",
    "olette",
    "eivät",
    "emme",
    "ette",
    "entä",
    "liene",
    "lienen",
    "lienet",
    "lienee",
    "lienemme",
    "lienette",
    "lienevät",
    "lienneet",
];

/// Words that always geminate the initial consonant of the next word (boundary
/// gemination) and are not caught by the suffix rules: imperatives and
/// connegatives, first infinitives, a few adverbs / nouns. Only forms no other
/// word is spelled like; the homographs are in `IMPERATIVE_CTX` and
/// `fi_verbs::INFINITIVE_CTX`.
const DOUBLE: &[&str] = &[
    "anna",
    "ota",
    "sano",
    "katso",
    "tule",
    "mene",
    "kuule",
    "odota",
    "laita",
    "pane",
    "aja",
    "auta",
    "pue",
    "pure",
    "lue",
    "istu",
    "lähde",
    "jätä",
    "pidä",
    "nouse",
    "sanoa",
    "katsoa",
    "tulla",
    "mennä",
    "kuulla",
    "tehdä",
    "olla",
    "saada",
    "nähdä",
    "voida",
    "puhua",
    "lukea",
    "ajatella",
    "haluta",
    "syödä",
    "juoda",
    "käydä",
    "viedä",
    "tuoda",
    "myydä",
    "kuolla",
    "asua",
    "oppia",
    "etsiä",
    "lähteä",
    "jäädä",
    "istua",
    "nukkua",
    "herätä",
    "pelata",
    "tanssia",
    "uida",
    "juosta",
    "kävellä",
    "tuntea",
    "avata",
    "sulkea",
    "kysyä",
    "vastata",
    "tarvita",
    "osata",
    "ajeta",
    "auteta",
    "oteta",
    "pueta",
    "pukea",
    "purra",
    "sanota",
    "kiinni",
    "taa",
    "terve",
    "tuore",
    "tänne",
    "sinne",
    "minne",
    "luokse",
    "taakse",
    "alle",
    "ylle",
    "päälle",
    "kerro",
    "puhu",
    "sovi",
    "näy",
    "aio",
    "tiedä",
    "ymmärrä",
    "jaksa",
    "viitsi",
    "kysy",
    "etsi",
    "nuku",
    "soita",
    "osta",
    "kirjoita",
    "lopeta",
    "aloita",
    "unohda",
    "löydä",
    "vedä",
    "lähetä",
    "näytä",
    "käytä",
    "yritä",
    "mieti",
    "pyydä",
    "kiitä",
    "seiso",
    "välitä",
    "kestä",
    "estä",
    "pysty",
    "elä",
    "opi",
    "asti",
    "yleensä",
    "tahansa",
    "yhteensä",
    "sinänsä",
    "hyvänsä",
    "ympäriinsä",
    "paeta",
    "havaita",
    "ansaita",
    "kyetä",
    "niellä",
    "nuolla",
    "surra",
];
/// Words the suffix rules would take for triggers and that are not: names in
/// -lle / -nne / -e, loans and slang in -e without the historical final
/// consonant (VISK § 34: kolme, itse, nukke), nouns in -da / -ta.
const NO_DOUBLE: &[&str] = &[
    "kalle", "ville", "pelle", "nalle", "hille", "galle", "polle", "promille", "anne", "hanne",
    "janne", "manne", "marianne", "susanne", "wille", "elle", "belle", "google", "apple", "mike",
    "nike", "one", "some", "line", "online", "home", "office", "service", "jatte", "kate", "me",
    "te", "he", "se", "ne", "ehkä", "kolme", "itse", "nukke", "sote", "steve", "house", "anime",
    "ale", "eve", "mode", "move", "madame", "ope", "posse", "empire", "ultimate", "jade", "hype",
    "drone", "karate", "karaoke", "kake", "deadline", "adele", "lime", "tele", "jeppe", "slide",
    "akne", "sake", "skene", "lande", "saame", "kurre", "striptease", "poke", "veke", "machete",
    "nasse", "toope", "ukulele", "rope", "nuke", "nisse", "guacamole", "yorkshire", "fore",
    "mousse", "nysse", "mosse", "kore", "tempe", "mascarpone", "tilde", "andante", "mese", "pose",
    "nonsense", "bonde", "söde", "mokke", "spede", "jumaliste", "itsenne", "avenue", "fondue",
    "revue", "reggae", "sundae", "oboe", "aaloe", "niue", "joe", "jane", "pete", "lasse", "tove",
    "arne", "aune", "tyyne", "irene", "rene", "marlene", "louise", "denise", "eugene", "pierre",
    "dante", "goethe", "skype", "iphone", "store", "date", "like", "site", "game", "time", "live",
    "more", "love", "nice", "style", "type", "mobile", "profile", "blue", "true", "rule", "cute",
    "base", "single", "range", "lounge", "orange", "challenge", "change", "jungle", "iine", "sooda",
    "gouda", "lauda", "siida", "skeida", "kooda", "aida", "haida", "toyota", "dakota", "anita",
    "lolita", "nikita", "evita", "margarita", "akita", "emerita", "prostata", "päärata", "jäärata",
    "suursota", "madeleine", "genre", "freestyle", "franchise", "bisse", "spade", "grape", "flare",
    "joule", "penne", "sprite", "pointe", "skole", "peseta", "fajita", "smoothie",
];

/// Frequent foreign words and abbreviations, respelled in Finnish orthography
/// (several words allowed).
const RESPELL: &[(&str, &str)] = &[
    ("facebook", "feisbuk"),
    ("city", "siti"),
    ("show", "šou"),
    ("online", "onlain"),
    ("design", "disain"),
    ("washington", "vašinkton"),
    ("hockey", "hokkei"),
    ("college", "kollidš"),
    ("chicago", "šikago"),
    ("diesel", "diisel"),
    ("nobel", "noobel"),
    ("blues", "bluus"),
    ("cup", "kap"),
    ("house", "haus"),
    ("out", "aut"),
    ("about", "abaut"),
    ("go", "goo"),
    ("pro", "proo"),
    ("gay", "gei"),
    ("miami", "maiami"),
    ("baseball", "beisbool"),
    ("country", "kantri"),
    ("caesar", "keesar"),
    ("heavy", "hevi"),
    ("sydney", "sidni"),
    ("thaimaa", "taimaa"),
    ("review", "revjuu"),
    ("bluetooth", "bluutuut"),
    ("newton", "njuuton"),
    ("hudson", "hadson"),
    ("siemens", "siimens"),
    ("volkswagen", "folksvaagen"),
    ("rosberg", "ruusberi"),
    ("stora", "stuura"),
    ("rehn", "reen"),
    ("åbo", "oobu"),
    ("youtube", "juutuub"),
    ("google", "guugle"),
    ("iphone", "aifoun"),
    ("twitter", "tvitter"),
    ("whatsapp", "vatsap"),
    ("email", "iimeil"),
    ("web", "veb"),
    ("team", "tiim"),
    ("business", "bisnes"),
    ("jazz", "jats"),
    ("cowboy", "kauboi"),
    ("new", "njuu"),
    ("york", "jork"),
    ("game", "geim"),
    ("live", "laiv"),
    ("light", "lait"),
    ("night", "nait"),
    ("time", "taim"),
    ("free", "frii"),
    ("street", "striit"),
    ("all", "ool"),
    ("stars", "staars"),
    ("star", "staar"),
    ("open", "oupen"),
    ("power", "pauer"),
    ("black", "bläk"),
    ("white", "vait"),
    ("blue", "bluu"),
    ("green", "griin"),
    ("one", "van"),
    ("music", "mjuusik"),
    ("rock", "rok"),
    ("club", "klab"),
    ("shop", "šop"),
    ("beach", "biitš"),
    ("boy", "boi"),
    ("girl", "göörl"),
    ("baby", "beibi"),
    ("happy", "häppi"),
    ("michael", "maikel"),
    ("james", "džeims"),
    ("john", "džon"),
    ("george", "džoordž"),
    ("charles", "tšaarls"),
    ("shakespeare", "šeikspier"),
    ("manchester", "mäntšester"),
    ("liverpool", "liverpuul"),
    ("münchen", "mynhen"),
    ("zürich", "tsyyrih"),
    ("peugeot", "pöžoo"),
    ("renault", "renoo"),
    ("citroën", "sitroen"),
    ("bordeaux", "bordoo"),
    ("champagne", "šampanja"),
    ("tri", "tohtori"),
    ("prof", "professori"),
    ("hra", "herra"),
    ("rva", "rouva"),
    ("nro", "numero"),
    ("puh", "puhelin"),
    ("klo", "kello"),
    ("esim", "esimerkiksi"),
    ("jne", "ja niin edelleen"),
    ("ym", "ynnä muuta"),
    ("ns", "niin sanottu"),
    ("tms", "tai muuta sellaista"),
    ("yms", "ynnä muuta sellaista"),
    ("vrt", "vertaa"),
    ("kpl", "kappaletta"),
    ("ap", "aamupäivällä"),
    ("ip", "iltapäivällä"),
    ("hki", "helsinki"),
    ("os", "osoite"),
    ("ruoan", "ruuan"),
];
/// Lower-case initialisms read letter by letter.
const SPELLED: &[&str] = &[
    "oy", "oyj", "ry", "ab", "atk", "eu", "yk", "usa", "bbc", "cnn", "nhl", "nba", "hjk", "sdp",
    "ps", "pdf", "sms", "wc", "tv", "dvd", "cd", "pc", "it", "ok", "vip", "dj",
];

/// Numeral morphemes: a word made entirely of these is a numeral compound
/// (kaksikymmentäviisi) whose stress pattern restarts at each one.
const NUMERAL: &[&str] = &[
    "yksi",
    "kaksi",
    "kolme",
    "neljä",
    "viisi",
    "kuusi",
    "seitsemän",
    "kahdeksan",
    "yhdeksän",
    "kymmenen",
    "kymmentä",
    "toista",
    "sata",
    "sataa",
    "tuhat",
    "tuhatta",
    "miljoona",
    "miljoonaa",
];

/// Frequent first elements of compounds (audited against the 20k most
/// frequent words). The remainder must be at least four letters, contain a
/// vowel, start like a Finnish word and not look like a derivational suffix.
const STEMS: &[&str] = &[
    "maailman",
    "maailma",
    "kansain",
    "yhteis",
    "euroopan",
    "suomen",
    "kaupungin",
    "valtio",
    "pohjois",
    "etelä",
    "länsi",
    "keski",
    "lento",
    "tieto",
    "koti",
    "epä",
    "yli",
    "jälki",
    "ennakko",
    "ihmis",
    "kansallis",
    "kunnallis",
    "yksityis",
    "sosiaali",
    "talous",
    "oikeus",
    "terveys",
    "turvallisuus",
    "ympäristö",
    "luonnon",
    "ilmasto",
    "puolustus",
    "liikenne",
    "verkko",
    "nuoriso",
    "urheilu",
    "sähkö",
    "video",
    "viikon",
    "vuosi",
    "päivä",
    "joulu",
    "mies",
    "nais",
    "ruoka",
    "kahvi",
    "palkka",
    "vero",
    "hinta",
    "osake",
    "työ",
    "puhelin",
    "lehti",
    "ulko",
    "sisä",
    "itä",
    "ilta",
    "aamu",
    "kesä",
    "talvi",
    "kevät",
    "syys",
    "olut",
    "viini",
    "raha",
    "elin",
    "elämän",
    "markkina",
    "liike",
    "kauppa",
    "ammatti",
    "tele",
    "radio",
    "musiikki",
    "taide",
    "kieli",
    "lapsi",
    "koira",
    "kissa",
    "jalka",
    "silmä",
    "sydän",
    "vesi",
    "meri",
    "järvi",
    "metsä",
    "pelto",
    "kivi",
    "rauta",
    "kulta",
    "hopea",
    "tuli",
    "lumi",
    "sade",
    "tuuli",
    "aurinko",
    "tähti",
    "taivas",
    "huone",
    "ovi",
    "ikkuna",
    "katu",
    "silta",
    "linna",
    "kirkko",
    "sota",
    "rauha",
    "laki",
    "kunta",
    "kunnan",
    "kaupunki",
    "pankki",
    "posti",
    "poliisi",
    "lääke",
    "sairaala",
    "perhe",
    "isä",
    "äiti",
    "veli",
    "sisar",
    "poika",
    "tyttö",
    "kuningas",
    "sana",
    "pikku",
    "puoli",
    "pohja",
    "ala",
    "ylä",
    "etu",
    "taka",
    "väli",
    "lisä",
    "vasta",
    "sivu",
    "ala",
    "pää",
    "maa",
    "jää",
    "puu",
    "kuu",
    "tie",
    "yö",
    "suu",
    "luu",
    "kaksi",
    "kolme",
    "neljä",
    "viisi",
    "kuusi",
    "seitsemän",
    "kahdeksan",
    "yhdeksän",
    "kymmen",
    "yksin",
    "kaksin",
    "kolmin",
    "sata",
    "tuhat",
];
/// Short stems (one syllable) only split before a remainder that is itself a
/// listed stem or a frequent second element, since `tie`+`tysti`, `kuu`+`lostaa`
/// would otherwise be split.
const SHORT_STEMS: &[&str] = &["pää", "maa", "jää", "puu", "kuu", "tie", "yö", "suu", "luu"];
const SECOND: &[&str] = &[
    "ministeri",
    "kaupunki",
    "asia",
    "määrä",
    "johtaja",
    "toimittaja",
    "valmentaja",
    "sihteeri",
    "kirjoitus",
    "henkilö",
    "konttori",
    "rakennus",
    "paino",
    "maja",
    "mies",
    "tavoite",
    "ilma",
    "talous",
    "kunta",
    "joukkue",
    "pallo",
    "seutu",
    "perä",
    "tila",
    "kaasu",
    "kiekko",
    "kaappi",
    "halli",
    "tarha",
    "seppä",
    "tavara",
    "villa",
    "kausi",
    "lauta",
    "ura",
    "paikka",
    "voima",
    "nousu",
    "laki",
    "lasi",
    "osuus",
    "aika",
    "elämä",
    "raha",
];

#[derive(Clone)]
enum Unit {
    /// nucleus: phones, long or diphthong, plain i (eligible for ɪ)
    V(String, bool, bool),
    /// consonant: phone, long
    C(&'static str, bool),
}

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | 'ä'
            | 'ö'
            | 'å'
            | 'é'
            | 'ü'
            | 'ø'
            | 'æ'
            | 'ï'
            | 'ë'
            | 'á'
            | 'ó'
            | 'à'
            | 'è'
            | 'ì'
            | 'ò'
            | 'ù'
            | 'â'
            | 'ê'
            | 'î'
            | 'ô'
            | 'û'
            | 'í'
            | 'ú'
    )
}

fn vowel_ph(c: char) -> &'static str {
    match c {
        'a' | 'á' | 'à' | 'â' => "a",
        'e' | 'é' | 'ë' | 'è' | 'ê' => "e",
        'i' | 'ï' | 'í' | 'ì' | 'î' => "i",
        'o' | 'ó' | 'å' | 'ò' | 'ô' => "o",
        'u' | 'ú' | 'ù' | 'û' => "u",
        'y' | 'ü' => "y",
        'ä' | 'æ' => "æ",
        'ö' | 'ø' => "ø",
        _ => "",
    }
}

/// The Finnish diphthongs (plus ie/uo/yö in every position, as espeak does).
fn diphthong(a: char, b: char) -> bool {
    matches!(
        (a, b),
        ('a', 'i')
            | ('e', 'i')
            | ('o', 'i')
            | ('u', 'i')
            | ('y', 'i')
            | ('ä', 'i')
            | ('ö', 'i')
            | ('a', 'u')
            | ('e', 'u')
            | ('i', 'u')
            | ('o', 'u')
            | ('e', 'y')
            | ('i', 'y')
            | ('ä', 'y')
            | ('ö', 'y')
            | ('i', 'e')
            | ('u', 'o')
            | ('y', 'ö')
    )
}

fn is_stop(ph: &str) -> bool {
    matches!(ph, "p" | "t" | "k" | "b" | "d" | "ɡ")
}

/// Letters of one compound part -> phone units.
fn units(c: &[char], gem: Option<usize>) -> Vec<Unit> {
    let n = c.len();
    let at = |i: usize| c.get(i).copied().unwrap_or('#');
    let mut out: Vec<Unit> = Vec::with_capacity(n + 2);
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = at(i + 1);
        let nx2 = at(i + 2);
        if ch == '\'' {
            i += 1;
            continue;
        }
        if gem == Some(i) {
            out.push(Unit::C(
                match ch {
                    'k' => "k",
                    'p' => "p",
                    _ => "h",
                },
                true,
            ));
            i += 1;
            continue;
        }
        if is_vowel(ch) {
            let v = vowel_ph(ch);
            if nx == ch {
                out.push(Unit::V(format!("{v}ː"), true, ch == 'i'));
                i += 2;
            } else if is_vowel(nx) && diphthong(ch, nx) && nx2 != nx {
                out.push(Unit::V(format!("{v}{}", vowel_ph(nx)), true, false));
                i += 2;
            } else {
                out.push(Unit::V(v.to_string(), false, ch == 'i'));
                i += 1;
            }
            continue;
        }
        // doubled consonant letters are long
        let long = nx == ch && !matches!(ch, 'n' | 'c' | 's' | 'z' | 'x');
        let ph: &'static str = match ch {
            'n' => {
                if nx == 'g' {
                    // ng + vowel = long ŋ (kengän); ng + consonant = ŋ (englanti, kongressi:
                    // Wiktionary /ˈkoŋresːi/, /ˈsɑŋriɑ/, /ˈmɑŋroʋe/)
                    let l = is_vowel(nx2);
                    out.push(Unit::C("ŋ", l));
                    i += 2;
                    continue;
                }
                if nx == 'k' {
                    "ŋ"
                } else if nx == 'n' {
                    out.push(Unit::C("n", true));
                    i += 2;
                    continue;
                } else {
                    "n"
                }
            }
            'c' => {
                if nx == 'h' && i == 0 {
                    // chr-, chl- are k (Chris, Chloe, kromi); otherwise tʃ (chatti)
                    out.push(Unit::C(if matches!(nx2, 'r' | 'l') { "k" } else { "tʃ" }, false));
                    i += 2;
                    continue;
                }
                if nx == 'k' || nx == 'c' {
                    out.push(Unit::C("k", nx == 'c'));
                    i += 2;
                    continue;
                }
                if matches!(nx, 'e' | 'i' | 'y' | 'é') {
                    "s"
                } else {
                    "k"
                }
            }
            's' => {
                if nx == 'h' && i == 0 {
                    out.push(Unit::C("ʃ", false));
                    i += 2;
                    continue;
                }
                if nx == 'c' && nx2 == 'h' {
                    out.push(Unit::C("ʃ", false));
                    i += 3;
                    continue;
                }
                if nx == 's' {
                    out.push(Unit::C("s", true));
                    i += 2;
                    continue;
                }
                "s"
            }
            'z' => {
                out.push(Unit::C("ts", false));
                i += if nx == 'z' { 2 } else { 1 };
                continue;
            }
            'x' => {
                out.push(Unit::C("k", false));
                out.push(Unit::C("s", false));
                i += if nx == 'x' { 2 } else { 1 };
                continue;
            }
            'q' => {
                out.push(Unit::C("k", false));
                if nx == 'u' && is_vowel(nx2) {
                    out.push(Unit::C("v", false));
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            // ph is f in every loan (Philip, Sophia); Finnish has no p + h of its own
            'p' if nx == 'h' => {
                out.push(Unit::C("f", false));
                i += 2;
                continue;
            }
            // th-, gh- at the start and -th at the end are loans (thai, ghetto, Smith);
            // inside a word t + h is a clitic or compound seam (olethan, lyhythihainen)
            't' | 'g' if nx == 'h' && (i == 0 && (is_vowel(nx2) || ch == 't' && nx2 == 'r') || ch == 't' && i > 0 && i + 2 == n) => {
                out.push(Unit::C(if ch == 't' { "t" } else { "ɡ" }, false));
                i += 2;
                continue;
            }
            'p' => "p",
            't' => "t",
            'k' => "k",
            'b' => "b",
            'd' => "d",
            'g' => "ɡ",
            'f' => "f",
            'h' => "h",
            'j' => "j",
            'l' => "l",
            'm' => "m",
            'r' => "r",
            'v' | 'w' => "v",
            'š' => "ʃ",
            'ž' => "ʒ",
            'ç' => "s",
            'ß' => "s",
            _ => {
                i += 1;
                continue;
            }
        };
        out.push(Unit::C(ph, long));
        i += if long { 2 } else { 1 };
    }
    out
}

/// Syllable weights of a part: heavy = long vowel/diphthong, closed syllable.
fn weights(u: &[Unit]) -> Vec<bool> {
    let nuclei: Vec<usize> = u
        .iter()
        .enumerate()
        .filter(|(_, x)| matches!(x, Unit::V(..)))
        .map(|(i, _)| i)
        .collect();
    let mut w = Vec::with_capacity(nuclei.len());
    for (k, &i) in nuclei.iter().enumerate() {
        let long = matches!(u[i], Unit::V(_, true, _));
        let end = nuclei.get(k + 1).copied().unwrap_or(u.len());
        let cons: usize = u[i + 1..end]
            .iter()
            .map(|x| {
                if let Unit::C(_, l) = x {
                    if *l {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                }
            })
            .sum();
        let last = k + 1 == nuclei.len();
        w.push(long || cons >= 2 || (last && cons >= 1));
    }
    w
}

/// Stress marks per nucleus (0 none, 1 secondary, 2 primary): `first` on the
/// first syllable, then espeak's alternating secondary stress that skips from a
/// light syllable to a following heavy one and never lands on the last.
fn stress_pattern(w: &[bool], first: u8) -> Vec<u8> {
    let n = w.len();
    let mut s = vec![0u8; n];
    if n == 0 {
        return s;
    }
    s[0] = first;
    let mut pos = 2;
    while pos + 1 < n {
        if !w[pos] {
            if w[pos + 1] {
                // light before heavy: the heavy syllable takes it (lost if final)
                pos += 1;
                if pos + 1 >= n {
                    break;
                }
            } else if pos + 3 < n && w[pos + 2] {
                // two lights before a non-final heavy: the heavy one takes it
                pos += 2;
            }
        }
        s[pos] = 1;
        pos += 2;
    }
    s
}

fn render(u: &[Unit], marks: &[u8], out: &mut String) {
    let mut k = 0;
    for x in u {
        match x {
            Unit::V(ph, _, plain_i) => {
                let m = marks.get(k).copied().unwrap_or(0);
                k += 1;
                out.push_str(match m {
                    2 => "ˈ",
                    1 => "ˌ",
                    _ => "",
                });
                if *plain_i && m == 0 {
                    out.push_str(&ph.replace('i', "ɪ"));
                } else {
                    out.push_str(ph);
                }
            }
            Unit::C(ph, long) => {
                out.push_str(ph);
                if *long {
                    if is_stop(ph) {
                        out.push('ː');
                    } else {
                        out.push_str(ph);
                    }
                }
            }
        }
    }
}

fn vowel_class(c: char) -> i8 {
    match c {
        'a' | 'o' | 'u' | 'å' => 1,
        'ä' | 'ö' | 'y' | 'ü' | 'ø' | 'æ' => -1,
        _ => 0,
    }
}

/// Can a Finnish word start with this letter sequence (vowel, C + vowel, or a loan cluster)?
fn legal_onset(c: &[char]) -> bool {
    if c.is_empty() {
        return false;
    }
    if is_vowel(c[0]) {
        return true;
    }
    if c.len() < 2 {
        return false;
    }
    if is_vowel(c[1]) {
        return true;
    }
    c.len() >= 3
        && is_vowel(c[2])
        && matches!(
            (c[0], c[1]),
            ('s', 't')
                | ('s', 'k')
                | ('s', 'p')
                | ('t', 'r')
                | ('p', 'r')
                | ('k', 'r')
                | ('k', 'l')
                | ('p', 'l')
                | ('f', 'l')
                | ('f', 'r')
                | ('b', 'r')
                | ('d', 'r')
                | ('g', 'r')
                | ('g', 'l')
                | ('b', 'l')
        )
}

/// Parse `c` as a sequence of numeral morphemes; `out` gets the boundaries.
fn numeral_parts(c: &[char], out: &mut Vec<usize>) -> bool {
    if c.is_empty() {
        return true;
    }
    for m in NUMERAL {
        let mc: Vec<char> = m.chars().collect();
        if c.len() >= mc.len() && c[..mc.len()] == mc[..] {
            let before = out.len();
            out.push(mc.len());
            if numeral_parts(&c[mc.len()..], out) {
                return true;
            }
            out.truncate(before);
        }
    }
    false
}

/// Vowel nuclei of a letter sequence as (start, end) ranges: a doubled vowel
/// letter or a diphthong is one nucleus, anything else is hiatus.
fn nuclei(c: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if !is_vowel(c[i]) {
            i += 1;
            continue;
        }
        let nx = c.get(i + 1).copied().unwrap_or('#');
        let nx2 = c.get(i + 2).copied().unwrap_or('#');
        if nx == c[i] || (is_vowel(nx) && diphthong(c[i], nx) && nx2 != nx) {
            out.push((i, i + 2));
            i += 2;
        } else {
            out.push((i, i + 1));
            i += 1;
        }
    }
    out
}

fn starts(c: &[char], s: &str) -> bool {
    let mut it = c.iter();
    s.chars().all(|a| it.next() == Some(&a))
}

/// Does the remainder after a stem look like a second compound element
/// (rather than a derivational suffix such as -inen, -isesti, -lainen, -stää)?
fn element_like(rest: &[char], stem_last: char) -> bool {
    if rest.len() < 4
        || rest[0] == stem_last
        || !legal_onset(rest)
        || !rest.iter().any(|&x| is_vowel(x))
    {
        return false;
    }
    let c1 = rest[1];
    if matches!(rest[0], 'i' | 'ö') && !is_vowel(c1) {
        return [
            "ilma", "ilta", "isä", "ihmis", "ikkuna", "ikä", "ilmoit", "ihme", "öljy",
        ]
        .iter()
        .any(|s| starts(rest, s));
    }
    if rest[0] == 's' && !is_vowel(c1) {
        return false;
    }
    ![
        "lainen", "läinen", "lais", "läis", "mainen", "maisuu", "maisesti", "esti", "sto",
    ]
    .iter()
    .any(|s| starts(rest, s))
}

/// Length of a listed first element at the start of `rest`, if the remainder is plausible.
fn stem_cut(rest: &[char]) -> Option<usize> {
    let mut cut: Option<usize> = None;
    for st in STEMS {
        let m = st.chars().count();
        if cut.is_some_and(|x| x >= m) || rest.len() < m + 4 || !starts(rest, st) {
            continue;
        }
        let tail = &rest[m..];
        if !element_like(tail, rest[m - 1]) {
            continue;
        }
        if SHORT_STEMS.contains(st) && !SECOND.iter().chain(STEMS.iter()).any(|s| starts(tail, s)) {
            continue;
        }
        cut = Some(m);
    }
    cut
}

/// Vowel harmony: a front vowel after a back one (or vice versa) proves a
/// compound boundary somewhere between them. Candidate boundaries are the
/// syllable onsets in that stretch; the best-scoring one wins (both parts of
/// two or more syllables, a legal onset, a plausible final letter, hiatus
/// before a full vowel), ties going to the leftmost.
fn harmony_cut(rest: &[char]) -> Option<usize> {
    let nuc = nuclei(rest);
    let mut class = 0i8;
    let mut last = 0usize;
    for (k, &(s, e)) in nuc.iter().enumerate() {
        let cl = rest[s..e]
            .iter()
            .map(|&x| vowel_class(x))
            .sum::<i8>()
            .signum();
        if cl != 0 && class != 0 && cl != class {
            let mut best: Option<(i32, usize)> = None;
            for p in last + 1..=k {
                let (ps, _) = nuc[p];
                let cons = ps - nuc[p - 1].1;
                let cut = if cons == 0 { ps } else { ps - 1 };
                if cut == 0 || rest[cut - 1] == '\'' {
                    continue;
                }
                let mut score = 0;
                let lc = rest[cut - 1];
                if is_vowel(lc) || matches!(lc, 'n' | 's' | 't' | 'l' | 'r') {
                    score += 2;
                }
                if legal_onset(&rest[cut..]) {
                    score += 2;
                }
                if p >= 2 && nuc.len() - p >= 2 {
                    score += 4;
                }
                if cons == 0 && vowel_class(rest[ps]) != 0 {
                    score += 2;
                }
                if cons >= 1 {
                    score += 1;
                }
                if score >= 6 && best.is_none_or(|(bs, _)| score > bs) {
                    best = Some((score, cut));
                }
            }
            return best.map(|(_, cut)| cut);
        }
        if cl != 0 {
            class = cl;
            last = k;
        }
    }
    None
}

/// Split a hyphen-free word into compound parts (letter ranges).
fn split_compound(c: &[char]) -> Vec<(usize, usize)> {
    let n = c.len();
    // numeral compounds
    let mut nb = Vec::new();
    if numeral_parts(c, &mut nb) && nb.len() >= 2 {
        let mut parts = Vec::new();
        let mut s = 0;
        for l in nb {
            parts.push((s, s + l));
            s += l;
        }
        return parts;
    }
    let mut parts = Vec::new();
    let mut start = 0;
    while start < n {
        let rest = &c[start..];
        match stem_cut(rest).or_else(|| harmony_cut(rest)) {
            Some(cut) if cut > 0 && cut < rest.len() => {
                parts.push((start, start + cut));
                start += cut;
            }
            _ => {
                parts.push((start, n));
                break;
            }
        }
    }
    parts
}
/// Phonemize one compound-free, hyphen-free word with `first` as the stress of
/// its first syllable.
fn word_part(c: &[char], first: u8, out: &mut String) {
    let parts = split_compound(c);
    let last = parts.len() - 1;
    for (k, (s, e)) in parts.into_iter().enumerate() {
        let p = &c[s..e];
        let u = units(
            p,
            if k == last {
                enclitic_geminate(p)
            } else {
                None
            },
        );
        let w = weights(&u);
        let mut marks = stress_pattern(&w, if k == 0 { first } else { 1.min(first) });
        // espeak marks the vowel before word-final -lla (a/e/i + lla, e + llä)
        // as unstressed, so that syllable never takes the secondary stress
        let n = marks.len();
        let short = p.len() >= 5 && p[p.len() - 5] != p[p.len() - 4];
        if n >= 3
            && short
            && (ends(p, "alla") || ends(p, "ella") || ends(p, "illa") || ends(p, "ellä"))
            && marks[n - 2] == 1
        {
            marks[n - 2] = 0;
        }
        render(&u, &marks, out);
    }
}

fn ends(c: &[char], s: &str) -> bool {
    let m = s.chars().count();
    c.len() >= m && starts(&c[c.len() - m..], s)
}

/// Imperatives that take an enclitic often enough to list (not käy, syö, juo:
/// käypä, syöpä and juopa are other words).
const CLITIC_IMPERATIVE: &[&str] = &[
    "ole", "anna", "tule", "mene", "ota", "sano", "katso", "kuule", "odota", "laita", "pane", "lue", "istu", "lähde",
    "nouse", "kerro", "tee", "vie", "pidä", "jätä", "auta", "aja", "mieti", "kokeile", "näytä", "soita",
];

/// Allative -lle before an enclitic (-kaan, -kin, -ko, -pa, ...) geminates the
/// enclitic's consonant (kenellekään -> kenellekːään): index of that consonant.
fn enclitic_geminate(c: &[char]) -> Option<usize> {
    // imperative + -pa/-pä/-han/-hän: otapa -> otapːa, tulehan -> tulehhan
    for enc in ["pa", "pä", "han", "hän"] {
        let m = enc.chars().count();
        if c.len() > m + 1 && ends(c, enc) {
            let base: String = c[..c.len() - m].iter().collect();
            if CLITIC_IMPERATIVE.contains(&base.as_str()) {
                return Some(c.len() - m);
            }
        }
    }
    for enc in ["kaan", "kään", "kin", "ko", "kö", "pa", "pä", "han", "hän"] {
        let m = enc.chars().count();
        if c.len() > m + 3 && ends(c, enc) && ends(&c[..c.len() - m], "lle") {
            let stem = &c[..c.len() - m - 3];
            let ok = nuclei(stem).len() >= 2
                || [
                    "si", "mi", "jo", "ke", "mei", "tei", "hei", "noi", "näi", "nii",
                ]
                .iter()
                .any(|s| starts(stem, s) && stem.len() == s.chars().count());
            let base: String = c[..c.len() - m].iter().collect();
            return if ok && !NO_DOUBLE.contains(&base.as_str()) {
                Some(c.len() - m)
            } else {
                None
            };
        }
    }
    None
}

/// Phonemize one lower-case word (no sentence context).
pub fn word(w: &str) -> String {
    let lw = fold_letters(&w.to_lowercase());
    if lw.contains('-') {
        let mut out = String::new();
        for (k, p) in lw
            .split('-')
            .filter(|p| p.chars().any(|c| c.is_alphabetic()))
            .enumerate()
        {
            let s = word(p);
            out.push_str(&if k == 0 { s } else { s.replace('ˈ', "ˌ") });
        }
        return out;
    }
    let c: Vec<char> = lw
        .chars()
        .filter(|c| c.is_alphabetic() || *c == '\'')
        .collect();
    if !c.iter().any(|&x| x.is_alphabetic()) {
        return String::new();
    }
    if let Some((_, r)) = RESPELL.iter().find(|(k, _)| *k == lw) {
        return r.split(' ').map(word).collect::<Vec<_>>().join(" ");
    }
    if let Some(r) = respell_inflected(&lw) {
        return word(&r);
    }
    if !c.iter().any(|&x| is_vowel(x)) || c.len() == 1 || SPELLED.contains(&lw.as_str()) {
        return spell(&c);
    }
    let first = if UNSTRESSED.contains(&lw.as_str()) {
        0
    } else if SECONDARY.contains(&lw.as_str()) {
        1
    } else {
        2
    };
    let mut out = String::with_capacity(c.len() * 2);
    word_part(&c, first, &mut out);
    assimilate_nasals(&out)
}

/// Case endings a respelled loan can carry (Chicagossa, Facebookin, Youtubeen).
const LOAN_ENDINGS: &[&str] = &[
    "n", "a", "ä", "ssa", "ssä", "sta", "stä", "lla", "llä", "lta", "ltä", "lle", "ksi", "na", "nä", "in", "ia", "iä",
    "issa", "issä", "ista", "istä", "illa", "illä", "ilta", "iltä", "ille", "iksi", "ina", "inä", "iin", "oon", "aan",
    "een", "hun", "hon", "hin", "t", "it",
];

/// An inflected form of a respelled loan of five letters or more:
/// chicagossa -> šikagossa, facebookissa -> feisbukissa, youtubessa -> juutuubissa.
fn respell_inflected(lw: &str) -> Option<String> {
    for (k, r) in RESPELL {
        if k.chars().count() < 5 || r.contains(' ') {
            continue;
        }
        let Some(rest) = lw.strip_prefix(k) else {
            continue;
        };
        if !LOAN_ENDINGS.contains(&rest) {
            continue;
        }
        let stem_c = r.chars().last().is_some_and(|c| !is_vowel(c));
        let rest_cc = rest.chars().count() >= 2 && rest.chars().take(2).all(|c| !is_vowel(c));
        // a consonant stem takes its ending through -i- (juutuub + ssa -> juutuubissa)
        let link = if stem_c && (rest_cc || rest == "n" || rest == "t") { "i" } else { "" };
        return Some(format!("{r}{link}{rest}"));
    }
    None
}

/// Letters outside the Finnish alphabet and its usual loan letters: composed
/// form first (a + combining diaeresis is ä), a macron is a long vowel
/// (Tōkyō -> tookyoo), ñ is nj, other diacritics fall back to the base letter
/// (ř -> r, ğ -> g, ã -> a). They used to be dropped.
fn fold_letters(lw: &str) -> String {
    let known = |c: char| c.is_ascii() || is_vowel(c) || matches!(c, 'š' | 'ž' | 'ç' | 'ß');
    if lw.chars().all(known) {
        return lw.to_string();
    }
    let mut out = String::with_capacity(lw.len() + 4);
    for c in lw.nfc() {
        if known(c) {
            out.push(c);
            continue;
        }
        match c {
            'ñ' => out.push_str("nj"),
            'ő' => out.push('ö'),
            'ű' => out.push('y'),
            'þ' => out.push('t'),
            'ð' | 'đ' => out.push('d'),
            'ł' => out.push('l'),
            'ı' => out.push('i'),
            'č' | 'ć' => out.push_str("tš"),
            _ => {
                let mut d = c.nfd();
                let base = d.next().unwrap_or(c);
                let macron = d.any(|m| m == '\u{0304}');
                if base == c {
                    out.push(c);
                } else {
                    out.push(base);
                    if macron && is_vowel(base) {
                        out.push(base);
                    }
                }
            }
        }
    }
    out
}

/// n before k/ɡ is ŋ and before p/b is m, also across compound parts and
/// stress marks (suomeŋkielinen, eteːmpæin), as espeak writes them.
fn assimilate_nasals(s: &str) -> String {
    let ch: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in ch.iter().enumerate() {
        if c == 'n' && (i == 0 || ch[i - 1] != 'n') {
            let next = ch[i + 1..].iter().find(|&&x| x != 'ˈ' && x != 'ˌ').copied();
            match next {
                Some('k') | Some('ɡ') => {
                    out.push('ŋ');
                    continue;
                }
                Some('p') | Some('b') => {
                    out.push('m');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

fn letter_name(c: char) -> &'static str {
    match c {
        'a'..='z' => LETTERS.unwrap()[(c as u8 - b'a') as usize],
        'ä' => "æː",
        'ö' => "øː",
        'å' => "ruotsalainen oː",
        _ => "",
    }
}

/// Spell letter by letter: secondary stress on each letter, primary on the last.
fn spell(c: &[char]) -> String {
    let mut out = String::new();
    // W in an abbreviation is read vee (WC "veesee", BMW, www: Kielitoimiston
    // sanakirja); only the letter on its own is kaksoisvee
    let abbreviation = c.len() > 1;
    let letters: Vec<&str> = c
        .iter()
        .map(|&ch| {
            if ch == 'w' && abbreviation {
                "veː"
            } else {
                letter_name(ch)
            }
        })
        .filter(|s| !s.is_empty())
        .collect();
    for (k, name) in letters.iter().enumerate() {
        let mark = if k + 1 == letters.len() { "ˈ" } else { "ˌ" };
        let pos = name.find(|x: char| "aeiouyæø".contains(x)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    out
}

/// Spell an upper-case abbreviation (WC, EU, BMW).
pub fn spell_acronym(w: &str) -> String {
    let c: Vec<char> = w.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect();
    spell(&c)
}

fn syllables(w: &str) -> usize {
    let c: Vec<char> = w.chars().collect();
    units(&c, None)
        .iter()
        .filter(|u| matches!(u, Unit::V(..)))
        .count()
}

/// Imperative / connegative forms spelled like another form: the 3rd person
/// singular (hän käy, juo, syö, vie, luo, avaa, saa), a noun (tee, jää, voi) or a
/// pronoun (muista). They geminate only as an imperative or connegative: after
/// a negative verb, or (second field) at the start of a clause.
const IMPERATIVE_CTX: &[(&str, bool)] = &[
    ("käy", true),
    ("juo", true),
    ("syö", true),
    ("vie", true),
    ("luo", true),
    ("tee", true),
    ("muista", true),
    ("avaa", true),
    ("pelaa", true),
    ("tapaa", true),
    ("lupaa", true),
    ("herää", true),
    ("palaa", true),
    ("saa", false),
    ("jää", false),
    ("voi", false),
    ("ui", false),
    ("myy", false),
    ("lyö", false),
    ("osaa", false),
];

/// Imperatives that open a clause another imperative can be coordinated with
/// (ole hyvä ja avaa ovi; tule ja syö).
const IMPERATIVE_HEAD: &[&str] = &[
    "ole", "anna", "tule", "mene", "ota", "sano", "katso", "kuule", "odota", "laita", "pane", "lue", "istu", "lähde",
    "nouse", "älä",
];

/// Words that start a new clause without a comma: the search for a governing
/// verb or a negation stops there.
const CLAUSE_BREAK: &[&str] = &[
    "että", "kun", "jos", "koska", "vaikka", "kunnes", "jotta", "mutta", "vaan", "sillä", "joka", "jota", "jonka",
    "jotka", "mikä", "minkä", "kuin",
];

/// Verb forms that take a first infinitive (VISK § 469-470: voida, saada,
/// täytyä, pitää, haluta, osata, aikoa, yrittää, tarvita, kannattaa, alkaa,
/// antaa, tahtoa, taitaa, saattaa, ehtiä, jaksaa, uskaltaa, viitsiä, muistaa,
/// unohtaa, luvata, päättää, …) and the nouns and adjectives of `on pakko /
/// aika / helppo / mahdollista tehdä`.
const GOVERNOR: &[&str] = &[
    "voi", "voin", "voit", "voimme", "voitte", "voivat", "voida", "voidaan", "voitiin", "voinut", "voineet", "voitu",
    "voinee", "saa", "saan", "saat", "saamme", "saatte", "saavat", "sai", "sain", "sait", "saimme", "saitte", "saivat",
    "saada", "saadaan", "saatiin", "saanut", "saaneet", "saatu", "pitää", "piti", "pidä", "pitänyt", "pitäneet", "osaa",
    "osaan", "osaat", "osaamme", "osaatte", "osaavat", "osasi", "osasin", "osasit", "osasimme", "osasivat", "osata",
    "osannut", "osanneet", "aikoo", "aikoi", "aikovat", "aikoivat", "aikonut", "aikoneet", "aikoa", "tarvita",
    "kannata", "kannatti", "alkaa", "alkoi", "alkavat", "alkoivat", "alkanut", "alkaneet", "alan", "alat", "alamme",
    "alatte", "aloin", "aloit", "aloimme", "anna", "antaa", "antoi", "annan", "annat", "annamme", "annatte", "antavat",
    "antoivat", "annoin", "annoit", "antanut", "taitaa", "taitavat", "taisi", "taisin", "taisit", "päätti", "päätin",
    "päätit", "päätimme", "päättivät", "päättää", "päätän", "päättänyt", "muista", "muistan", "muistat", "muisti",
    "muistin", "muistaa", "muistanut", "mahtaa", "mahdan", "mahdat", "mahtoi", "sopii", "sopisi", "kuuluu", "kuului",
    "kuuluisi", "helppo", "helppoa", "vaikea", "vaikeaa", "vaikeata", "hauska", "hauskaa", "mukava", "mukavaa", "kiva",
    "kivaa", "ihana", "ihanaa", "hyvä", "parempi", "parasta", "paras", "pakko", "aika", "aikaa", "syytä", "tapana",
    "tarkoitus", "varaa", "lupa", "oikeus", "tilaisuus", "mahdollisuus", "tärkeää", "tärkeä", "turha", "turhaa",
    "vaarallista", "kiellettyä", "sallittua", "tarpeen", "kyky", "vaikeuksia", "ongelmia", "hankala", "hankalaa",
    "järkevää", "viisasta", "pakollista", "mahdollista", "mahdotonta", "mahdoton", "tapa", "keino", "tarve",
];
const GOVERNOR_PREFIX: &[&str] = &[
    "vois", "voitais", "sais", "saatais", "täyty", "pitäis", "halu", "osais", "aio", "aikois", "yrit", "tarvits",
    "tarvin", "kannatta", "alkais", "antais", "tahdo", "tahto", "taida", "saata", "saatta", "saatto", "ehdi", "ehti",
    "jaksa", "uskal", "viitsi", "kehta", "kehda", "unoh", "lupa", "luva", "käske", "käski", "salli", "tykkä", "tykä",
    "rakasta",
];

/// The word without one enclitic particle (-ko, -han, -pa, -kin, -kaan, -kä).
fn strip_clitic(s: &str) -> &str {
    for enc in ["kaan", "kään", "kin", "han", "hän", "pa", "pä", "ko", "kö", "kä"] {
        if let Some(r) = s.strip_suffix(enc) {
            if r.chars().count() >= 2 {
                return r;
            }
        }
    }
    s
}

/// Does this word form take a first infinitive as its complement?
fn governs_infinitive(src: &str) -> bool {
    let gov = |s: &str| {
        GOVERNOR.contains(&s)
            || GOVERNOR_PREFIX.iter().any(|p| s.starts_with(p)) && !s.starts_with("yrity")
    };
    gov(src) || {
        let s = strip_clitic(src);
        s != src && gov(s)
    }
}

/// The negative verb: en, et, ei, emme, ette, eivät, the imperative älä, with
/// an enclitic (eikä, enhän, eikö) or fused to a conjunction (ettei, ellen,
/// miksei, muttei, vaikkei, jottei).
fn is_negative(src: &str) -> bool {
    const NEG: [&str; 7] = ["en", "et", "ei", "emme", "ette", "eivät", "älä"];
    let s = strip_clitic(src);
    NEG.contains(&src)
        || NEG.contains(&s)
        || ["ett", "ell", "joll", "miks", "mutt", "vaikk", "jott"]
            .iter()
            .any(|p| s.strip_prefix(p).is_some_and(|r| NEG[..6].contains(&r)))
}

/// Is there a word satisfying `hit` earlier in the same clause, at most `window`
/// words back, with no pause or clause-opening conjunction in between?
fn earlier_in_clause(outs: &[Out], k: usize, window: usize, hit: impl Fn(&str) -> bool) -> bool {
    let mut j = k;
    while j > 0 && k - j < window {
        j -= 1;
        if outs[j].pause_after {
            return false;
        }
        let s = outs[j].src.as_str();
        if hit(s) {
            return true;
        }
        if CLAUSE_BREAK.contains(&s) {
            return false;
        }
    }
    false
}

/// An adverb in -sti (nopeasti, yleisesti, normaalisti, varmasti), as opposed to
/// a noun (turisti, eversti, protesti) or a past tense (varasti, pelasti, järjesti).
fn sti_adverb(src: &str) -> bool {
    const SHORT: &[&str] = &[
        "varmasti", "helposti", "tietysti", "kovasti", "huonosti", "pahasti", "hienosti", "kahdesti", "monesti",
        "kolmesti", "reilusti", "aidosti", "avoimesti", "laajasti", "suuresti", "taatusti", "lyhyesti", "rajusti",
        "tiukasti", "nätisti", "kiltisti", "siististi", "hauskasti", "ihanasti", "lujasti", "hyvästi", "pitkästi",
        "hullusti", "kamalasti", "ankarasti", "ahkerasti", "heikosti", "niukasti", "kevyesti", "kummasti", "oudosti",
        "julmasti", "hellästi", "hurjasti", "syvästi", "tyhmästi", "rumasti", "kylmästi", "kuumasti", "laiskasti",
        "nöyrästi", "köyhästi", "villisti", "äkisti", "leikisti", "rennosti", "paksusti", "herkästi", "jyrkästi",
        "rutkasti", "näppärästi", "fiksusti", "isosti", "huimasti", "rankasti", "pirusti", "roimasti", "hassusti",
        "visusti", "täpärästi", "tylysti", "typerästi", "löyhästi", "kehnosti", "jännästi", "ovelasti", "hämärästi",
        "hankalasti", "tyynesti", "nolosti", "pienesti", "väljästi", "kömpelösti", "tuplasti", "vitusti",
        "helvetisti", "hemmetisti", "saatanasti", "hitosti", "kolmasti", "neljästi", "viidesti", "kuudesti",
    ];
    if SHORT.contains(&src) {
        return true;
    }
    if ["siisti", "laasti", "puusti", "rovasti", "insesti", "viesti", "posti", "ksti"]
        .iter()
        .any(|x| src.ends_with(x))
    {
        return false;
    }
    let c: Vec<char> = src.chars().collect();
    let m = c.len() - 3; // the stem before -sti
    if m < 3 {
        return false;
    }
    let (a, b) = (c[m - 2], c[m - 1]);
    // -sesti (yleisesti), a long vowel (hitaasti, kauniisti), -easti / -eästi
    // (nopeasti), -vasti / -västi (jatkuvasti, selvästi)
    if (a == 's' && b == 'e') || (a == b && is_vowel(a)) || (a == 'e' && matches!(b, 'a' | 'ä')) {
        return true;
    }
    if a == 'v' && matches!(b, 'a' | 'ä') {
        return true;
    }
    // -io / -iö adjectives (mainiosti) and passive participles in -tu / -ty after a
    // vowel or l, n, r, t (tunnetusti, perustellusti); vastusti, puolusti are verbs
    if a == 'i' && matches!(b, 'o' | 'ö') {
        return true;
    }
    if a == 't' && matches!(b, 'u' | 'y') && m >= 4 && syllables(src) >= 4 {
        let before = c[m - 3];
        if is_vowel(before) || matches!(before, 'l' | 'n' | 'r' | 't') {
            return true;
        }
    }
    // -ttomasti / -ttömästi and other long stems in -ma (ehdottomasti, välittömästi)
    if a == 'm' && matches!(b, 'a' | 'ä') && syllables(src) >= 4 {
        return true;
    }
    // adjective in long vowel + consonant + i: normaalisti, spontaanisti, anonyymisti
    b == 'i' && m >= 4 && !is_vowel(a) && is_vowel(c[m - 3]) && c[m - 4] == c[m - 3]
}

/// A nominative in -e with the historical final consonant (perhe, kone, lause,
/// joukkue, koe) or an imperative / connegative in -e (tule, sulje, kuuntele):
/// not -mme / -tte verb forms, not the diphthong -ie (tie and its compounds),
/// not a loan with a foreign letter near the end (college, face, bridge).
fn e_trigger(src: &str) -> bool {
    if !src.ends_with('e') || src.ends_with("mme") || src.ends_with("tte") || src.ends_with("ee") {
        return false;
    }
    // compounds of nukke and saame keep their lack of a final consonant
    if src.ends_with("nukke") || src.ends_with("saame") {
        return false;
    }
    if src.ends_with("ie") {
        return src.ends_with("aie") || src.ends_with("säie") || src == "hie";
    }
    let c: Vec<char> = src.chars().collect();
    !c[c.len().saturating_sub(4)..]
        .iter()
        .any(|x| matches!(x, 'b' | 'c' | 'f' | 'g' | 'q' | 'w' | 'x' | 'z' | 'š' | 'ž'))
}

/// A first infinitive, imperative or connegative in -da / -dä after a long
/// vowel, diphthong or h (saada, tuoda, tehdä, arvioida, huuda, vaihda), or in
/// consonant + short vowel + -ta / -tä with three or more syllables (haluta,
/// pelata, valita, odota, lopeta). The partitive -ta never follows a short
/// vowel after a consonant; nouns that end this way are compounds of sota,
/// rata, pata … (maailmansota, junarata), kept out by their length.
fn ta_trigger(src: &str, n: usize) -> bool {
    let c: Vec<char> = src.chars().collect();
    let m = c.len();
    if m < 4 || !matches!(c[m - 1], 'a' | 'ä') {
        return false;
    }
    if c[m - 2] == 'd' {
        return (c[m - 3] == 'h' || is_vowel(c[m - 3]) && is_vowel(c[m - 4])) && !src.ends_with("sooda");
    }
    if c[m - 2] != 't' || n < 3 || !is_vowel(c[m - 3]) || is_vowel(c[m - 4]) {
        return false;
    }
    let tail: String = c[m - 4..].iter().collect();
    let noun = matches!(
        tail.as_str(),
        "sota" | "setä" | "hätä" | "rata" | "sata" | "pata" | "kota" | "mätä" | "data" | "kita" | "nata" | "jata" | "muta" | "vita" | "kuta"
    );
    // -eerata verbs are long too (riskeerata, parkkeerata)
    !noun || n == 3 || src.ends_with("eerata")
}

/// The 2nd person plural possessive -nne on a case form (talossanne,
/// ystävällenne), which unlike sinne, tänne, tunne, liikenne does not geminate.
fn possessive_nne(src: &str) -> bool {
    [
        "ssanne", "ssänne", "stanne", "stänne", "llanne", "llänne", "ltanne", "ltänne", "llenne", "ksenne", "nanne",
        "nänne",
    ]
    .iter()
    .any(|x| src.ends_with(x))
}

/// How a word can trigger boundary gemination.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Trigger {
    No,
    Always,
    /// only as a first infinitive (antaa, oppia, ilmaista)
    Infinitive,
    /// only as an imperative / connegative; `true`: also clause-initially
    Imperative(bool),
}

fn trigger_class(src: &str) -> Trigger {
    // only a vowel-final word can lengthen the next consonant (tänään, kotiin, pois never do)
    if NO_DOUBLE.contains(&src) || !src.chars().last().is_some_and(is_vowel) {
        return Trigger::No;
    }
    if let Some((_, initial)) = IMPERATIVE_CTX.iter().find(|(w, _)| *w == src) {
        return Trigger::Imperative(*initial);
    }
    if DOUBLE.contains(&src) || INFINITIVE.binary_search(&src).is_ok() {
        return Trigger::Always;
    }
    if INFINITIVE_CTX.binary_search(&src).is_ok() {
        return Trigger::Infinitive;
    }
    let n = syllables(src);
    if n < 2 {
        return Trigger::No;
    }
    let hit = if src.ends_with("nne") {
        !possessive_nne(src)
    } else {
        src.ends_with("lle")
            || src.ends_with("tse") && n >= 3
            || src.ends_with("sti") && n >= 3 && sti_adverb(src)
            || ta_trigger(src, n)
            || e_trigger(src)
    };
    if hit {
        Trigger::Always
    } else {
        Trigger::No
    }
}

/// Does word `k` geminate the initial consonant of the next one?
fn doubles(outs: &[Out], k: usize) -> bool {
    match trigger_class(&outs[k].src) {
        Trigger::No => false,
        Trigger::Always => true,
        Trigger::Infinitive => earlier_in_clause(outs, k, 6, governs_infinitive),
        Trigger::Imperative(initial) => {
            if earlier_in_clause(outs, k, 4, is_negative) {
                return true;
            }
            if !initial {
                return false;
            }
            if k == 0 || outs[k - 1].pause_after {
                return true;
            }
            // ole hyvä ja avaa ovi: coordinated with the imperative that opens the clause
            if matches!(outs[k - 1].src.as_str(), "ja" | "tai" | "sekä") {
                let mut h = k - 1;
                while h > 0 && !outs[h - 1].pause_after {
                    h -= 1;
                }
                return IMPERATIVE_HEAD.contains(&outs[h].src.as_str());
            }
            false
        }
    }
}

/// First consonant phone of a word's phonemes, if it starts with one.
fn initial_consonant(ph: &str) -> Option<(usize, &str)> {
    let t = ph.trim_start_matches(['ˈ', 'ˌ']);
    let off = ph.len() - t.len();
    let c = t.chars().next()?;
    if "aeiouyæøɪ".contains(c) {
        return None;
    }
    let mut end = c.len_utf8();
    if c == 't' && t[end..].starts_with('ʃ') {
        end += 'ʃ'.len_utf8();
    }
    Some((off, &t[..end]))
}

/// Sentence pass: boundary gemination and word-final n before k/p, the way
/// espeak writes them.
pub fn join(outs: &mut [Out]) {
    let n = outs.len();
    for k in 0..n.saturating_sub(1) {
        if outs[k].pause_after || outs[k].ph.is_empty() || outs[k + 1].ph.is_empty() {
            continue;
        }
        let geminate = doubles(outs, k);
        let (a, b) = outs.split_at_mut(k + 1);
        let prev = &mut a[k];
        let cur = &mut b[0];
        let Some((off, c)) = initial_consonant(&cur.ph) else {
            continue;
        };
        if c == "ts" || c == "tʃ" {
            continue;
        }
        let already_long =
            cur.ph[off + c.len()..].starts_with('ː') || cur.ph[off + c.len()..].starts_with(c);
        if geminate && !already_long {
            if is_stop(c) {
                cur.ph.insert(off + c.len(), 'ː');
            } else {
                prev.ph.push_str(c);
            }
            continue;
        }
        if prev.ph.ends_with('n') && !prev.ph.ends_with("nn") {
            let rep = match c {
                "k" | "ɡ" => "ŋ",
                "p" | "b" => "m",
                _ => continue,
            };
            prev.ph.pop();
            prev.ph.push_str(rep);
        }
    }
}

const ONES: [&str; 10] = [
    "nolla",
    "yksi",
    "kaksi",
    "kolme",
    "neljä",
    "viisi",
    "kuusi",
    "seitsemän",
    "kahdeksan",
    "yhdeksän",
];

fn below_100(n: u128, out: &mut String) {
    if n == 0 {
        return;
    }
    if n < 10 {
        out.push_str(ONES[n as usize]);
    } else if n == 10 {
        out.push_str("kymmenen");
    } else if n < 20 {
        out.push_str(ONES[(n - 10) as usize]);
        out.push_str("toista");
    } else {
        out.push_str(ONES[(n / 10) as usize]);
        out.push_str("kymmentä");
        if n % 10 != 0 {
            out.push_str(ONES[(n % 10) as usize]);
        }
    }
}

fn below_1000(n: u128, out: &mut String) {
    match n / 100 {
        0 => {}
        1 => out.push_str("sata"),
        h => {
            out.push_str(ONES[h as usize]);
            out.push_str("sataa");
        }
    }
    below_100(n % 100, out);
}

/// Numbers below a million are one orthographic word (kaksituhattakolmesataa).
fn below_million(n: u128) -> String {
    let mut out = String::new();
    match n / 1000 {
        0 => {}
        1 => out.push_str("tuhat"),
        t => {
            below_1000(t, &mut out);
            out.push_str("tuhatta");
        }
    }
    below_1000(n % 1000, &mut out);
    out
}

/// Finnish cardinal number words.
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["nolla".into()];
    }
    let mut out = Vec::new();
    let scales: [(u128, &str, &str); 4] = [
        (1_000_000_000_000_000_000, "triljoona", "triljoonaa"),
        (1_000_000_000_000, "biljoona", "biljoonaa"),
        (1_000_000_000, "miljardi", "miljardia"),
        (1_000_000, "miljoona", "miljoonaa"),
    ];
    let mut rest = n;
    for (val, one, many) in scales {
        let q = rest / val;
        if q == 0 {
            continue;
        }
        rest %= val;
        if q == 1 {
            out.push(one.into());
        } else {
            if q < 1_000_000 {
                out.push(below_million(q));
            } else {
                out.extend(cardinal(q));
            }
            out.push(many.into());
        }
    }
    if rest > 0 {
        out.push(below_million(rest));
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "prosenttia",
        '$' => "dollaria",
        '€' => "euroa",
        '£' => "puntaa",
        '&' => "ja",
        '+' => "plus",
        '@' => "ät",
        '°' => "astetta",
        '=' => "on yhtä kuin",
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Text normalization: inflected numerals, ordinals, dates, clock times, units
// and abbreviations (Kielitoimiston ohjepankki: "Numeroiden ja tunnusten
// taivutus", "Päiväys", "Kellonajat", "Lyhenteet").

/// Grammatical case of a numeral.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Nom,
    Gen,
    Part,
    Ine,
    Ela,
    Ill,
    Ade,
    Abl,
    All,
    Ess,
    Tra,
}

/// A numeral word: nominative, weak stem (genitive and the local cases), strong
/// stem (essive; ordinals also illative), partitive, illative.
struct NumWord(&'static str, &'static str, &'static str, &'static str, &'static str);

const CARD: [NumWord; 10] = [
    NumWord("nolla", "nolla", "nolla", "nollaa", "nollaan"),
    NumWord("yksi", "yhde", "yhte", "yhtä", "yhteen"),
    NumWord("kaksi", "kahde", "kahte", "kahta", "kahteen"),
    NumWord("kolme", "kolme", "kolme", "kolmea", "kolmeen"),
    NumWord("neljä", "neljä", "neljä", "neljää", "neljään"),
    NumWord("viisi", "viide", "viite", "viittä", "viiteen"),
    NumWord("kuusi", "kuude", "kuute", "kuutta", "kuuteen"),
    NumWord("seitsemän", "seitsemä", "seitsemä", "seitsemää", "seitsemään"),
    NumWord("kahdeksan", "kahdeksa", "kahdeksa", "kahdeksaa", "kahdeksaan"),
    NumWord("yhdeksän", "yhdeksä", "yhdeksä", "yhdeksää", "yhdeksään"),
];
const TEN: NumWord = NumWord("kymmenen", "kymmene", "kymmene", "kymmentä", "kymmeneen");
const HUNDRED: NumWord = NumWord("sata", "sada", "sata", "sataa", "sataan");
const THOUSAND: NumWord = NumWord("tuhat", "tuhanne", "tuhante", "tuhatta", "tuhanteen");
const MILLION: NumWord = NumWord("miljoona", "miljoona", "miljoona", "miljoonaa", "miljoonaan");

const ORD: [NumWord; 10] = [
    NumWord("nollas", "nollanne", "nollante", "nollatta", "nollanteen"),
    NumWord("ensimmäinen", "ensimmäise", "ensimmäise", "ensimmäistä", "ensimmäiseen"),
    NumWord("toinen", "toise", "toise", "toista", "toiseen"),
    NumWord("kolmas", "kolmanne", "kolmante", "kolmatta", "kolmanteen"),
    NumWord("neljäs", "neljänne", "neljänte", "neljättä", "neljänteen"),
    NumWord("viides", "viidenne", "viidente", "viidettä", "viidenteen"),
    NumWord("kuudes", "kuudenne", "kuudente", "kuudetta", "kuudenteen"),
    NumWord("seitsemäs", "seitsemänne", "seitsemänte", "seitsemättä", "seitsemänteen"),
    NumWord("kahdeksas", "kahdeksanne", "kahdeksante", "kahdeksatta", "kahdeksanteen"),
    NumWord("yhdeksäs", "yhdeksänne", "yhdeksänte", "yhdeksättä", "yhdeksänteen"),
];
/// 1 and 2 inside a compound ordinal: yhdestoista, kahdeskymmenes.
const ORD_1: NumWord = NumWord("yhdes", "yhdenne", "yhdente", "yhdettä", "yhdenteen");
const ORD_2: NumWord = NumWord("kahdes", "kahdenne", "kahdente", "kahdetta", "kahdenteen");
const ORD_TEN: NumWord = NumWord("kymmenes", "kymmenenne", "kymmenente", "kymmenettä", "kymmenenteen");
const ORD_HUNDRED: NumWord = NumWord("sadas", "sadanne", "sadante", "sadatta", "sadanteen");

/// One numeral word in a case. `counted`: the tens / hundreds / thousands word
/// after a multiplier, partitive in the nominative (kaksikymmentä, kolmesataa).
fn inflect(w: &NumWord, case: Case, counted: bool) -> String {
    let a = if w.0.chars().any(|c| matches!(c, 'a' | 'o' | 'u')) { 'a' } else { 'ä' };
    match case {
        Case::Nom => (if counted { w.3 } else { w.0 }).to_string(),
        Case::Part => w.3.to_string(),
        Case::Ill => w.4.to_string(),
        Case::Gen => format!("{}n", w.1),
        Case::Ine => format!("{}ss{a}", w.1),
        Case::Ela => format!("{}st{a}", w.1),
        Case::Ade => format!("{}ll{a}", w.1),
        Case::Abl => format!("{}lt{a}", w.1),
        Case::All => format!("{}lle", w.1),
        Case::Tra => format!("{}ksi", w.1),
        Case::Ess => format!("{}n{a}", w.2),
    }
}

fn case_below_100(n: u128, case: Case, out: &mut Vec<String>) {
    match n {
        0 => {}
        1..=9 => out.push(inflect(&CARD[n as usize], case, false)),
        10 => out.push(inflect(&TEN, case, false)),
        11..=19 => {
            out.push(inflect(&CARD[(n - 10) as usize], case, false));
            out.push("toista".into());
        }
        _ => {
            out.push(inflect(&CARD[(n / 10) as usize], case, false));
            out.push(inflect(&TEN, case, true));
            if n % 10 != 0 {
                out.push(inflect(&CARD[(n % 10) as usize], case, false));
            }
        }
    }
}

fn case_below_1000(n: u128, case: Case, out: &mut Vec<String>) {
    match n / 100 {
        0 => {}
        1 => out.push(inflect(&HUNDRED, case, false)),
        h => {
            out.push(inflect(&CARD[h as usize], case, false));
            out.push(inflect(&HUNDRED, case, true));
        }
    }
    case_below_100(n % 100, case, out);
}

fn case_below_million(n: u128, case: Case) -> Vec<String> {
    let mut out = Vec::new();
    match n / 1000 {
        0 => {}
        1 => out.push(inflect(&THOUSAND, case, false)),
        t => {
            case_below_1000(t, case, &mut out);
            out.push(inflect(&THOUSAND, case, true));
        }
    }
    case_below_1000(n % 1000, case, &mut out);
    out
}

/// A cardinal below a milliard in a case, as words made of morphemes; every
/// part inflects.
fn cardinal_morphemes(n: u128, case: Case) -> Option<Vec<Vec<String>>> {
    if n >= 1_000_000_000 {
        return None;
    }
    if n == 0 {
        return Some(vec![vec![inflect(&CARD[0], case, false)]]);
    }
    let mut out = Vec::new();
    match n / 1_000_000 {
        0 => {}
        1 => out.push(vec![inflect(&MILLION, case, false)]),
        q => {
            out.push(case_below_million(q, case));
            out.push(vec![inflect(&MILLION, case, true)]);
        }
    }
    if n % 1_000_000 > 0 {
        out.push(case_below_million(n % 1_000_000, case));
    }
    Some(out)
}

/// A cardinal below a milliard in a case (5:ssä -> viidessä, 25:n ->
/// kahdenkymmenenviiden, 300:aa -> kolmeasataa).
pub fn cardinal_case(n: u128, case: Case) -> Option<Vec<String>> {
    Some(cardinal_morphemes(n, case)?.iter().map(|w| w.concat()).collect())
}

/// An ordinal below a thousand in a case, as the morphemes of one word.
fn ordinal_morphemes(n: u128, case: Case) -> Option<Vec<String>> {
    if n == 0 || n >= 1000 {
        return None;
    }
    let inner = |d: u128| match d {
        1 => &ORD_1,
        2 => &ORD_2,
        d => &ORD[d as usize],
    };
    let mut out = Vec::new();
    match n / 100 {
        0 => {}
        1 => out.push(inflect(&ORD_HUNDRED, case, false)),
        h => {
            out.push(inflect(inner(h), case, false));
            out.push(inflect(&ORD_HUNDRED, case, false));
        }
    }
    let r = n % 100;
    match r {
        0 => {}
        1..=9 => out.push(inflect(&ORD[r as usize], case, false)),
        10 => out.push(inflect(&ORD_TEN, case, false)),
        11..=19 => {
            out.push(inflect(inner(r - 10), case, false));
            out.push("toista".into());
        }
        _ => {
            out.push(inflect(inner(r / 10), case, false));
            out.push(inflect(&ORD_TEN, case, false));
            if r % 10 != 0 {
                out.push(inflect(&ORD[(r % 10) as usize], case, false));
            }
        }
    }
    Some(out)
}

/// An ordinal below a thousand in a case (12. -> kahdestoista, 21. ->
/// kahdeskymmenesensimmäinen, 3:nnessa -> kolmannessa).
pub fn ordinal(n: u128, case: Case) -> Option<String> {
    Some(ordinal_morphemes(n, case)?.concat())
}

/// Phonemes of a numeral word given as morphemes: the stress pattern restarts
/// at each one, as in the nominative (kˈaksɪkˌymmentævˌiːsɪ).
fn numeral_piece(morphemes: &[String]) -> Piece {
    let mut ph = String::new();
    for (k, m) in morphemes.iter().enumerate() {
        let p = word(m);
        ph.push_str(&if k == 0 { p } else { p.replace('ˈ', "ˌ") });
    }
    Piece::Ph(assimilate_nasals(&ph), morphemes.concat())
}

/// The case (and cardinal / ordinal reading) a written ending after a colon
/// stands for: 5:n, 5:ssä, 5:een, 5:ttä; 5:s, 5:nnen, 5:nnessä.
fn colon_case(e: &str) -> Option<(Case, bool)> {
    Some(match e {
        "n" => (Case::Gen, false),
        "ssa" | "ssä" => (Case::Ine, false),
        "sta" | "stä" => (Case::Ela, false),
        "lla" | "llä" => (Case::Ade, false),
        "lta" | "ltä" => (Case::Abl, false),
        "lle" => (Case::All, false),
        "ksi" => (Case::Tra, false),
        "na" | "nä" => (Case::Ess, false),
        "a" | "ä" | "aa" | "ää" | "ta" | "tä" | "tta" | "ttä" => (Case::Part, false),
        "een" | "en" | "aan" | "ään" | "teen" | "hen" | "iin" | "seen" => (Case::Ill, false),
        "s" => (Case::Nom, true),
        "nnen" => (Case::Gen, true),
        "nnessa" | "nnessä" => (Case::Ine, true),
        "nnesta" | "nnestä" => (Case::Ela, true),
        "nnella" | "nnellä" => (Case::Ade, true),
        "nnelta" | "nneltä" => (Case::Abl, true),
        "nnelle" => (Case::All, true),
        "nneksi" => (Case::Tra, true),
        "ntena" | "ntenä" => (Case::Ess, true),
        "nteen" => (Case::Ill, true),
        _ => return None,
    })
}

const MONTHS: [&str; 12] = [
    "tammikuu", "helmikuu", "maaliskuu", "huhtikuu", "toukokuu", "kesäkuu", "heinäkuu", "elokuu", "syyskuu", "lokakuu",
    "marraskuu", "joulukuu",
];

/// Units after a number: partitive, or nominative after 1 (5 kg -> viisi
/// kilogrammaa, 1 km -> yksi kilometri). Symbols are case-sensitive.
fn unit(w: &str, one: bool) -> Option<&'static str> {
    let (sg, pl) = match w {
        "kg" => ("kilogramma", "kilogrammaa"),
        "g" => ("gramma", "grammaa"),
        "mg" => ("milligramma", "milligrammaa"),
        "km" => ("kilometri", "kilometriä"),
        "m" => ("metri", "metriä"),
        "cm" => ("senttimetri", "senttimetriä"),
        "mm" => ("millimetri", "millimetriä"),
        "l" => ("litra", "litraa"),
        "dl" => ("desilitra", "desilitraa"),
        "cl" => ("senttilitra", "senttilitraa"),
        "ml" => ("millilitra", "millilitraa"),
        "h" => ("tunti", "tuntia"),
        "min" => ("minuutti", "minuuttia"),
        "s" => ("sekunti", "sekuntia"),
        "kk" => ("kuukausi", "kuukautta"),
        "vk" => ("viikko", "viikkoa"),
        "v" => ("vuosi", "vuotta"),
        "t" => ("tonni", "tonnia"),
        "ha" => ("hehtaari", "hehtaaria"),
        "kpl" => ("kappale", "kappaletta"),
        "snt" => ("sentti", "senttiä"),
        "kWh" => ("kilowattitunti", "kilowattituntia"),
        "kW" => ("kilowatti", "kilowattia"),
        "MW" => ("megawatti", "megawattia"),
        "W" => ("watti", "wattia"),
        "GB" => ("gigatavu", "gigatavua"),
        "MB" => ("megatavu", "megatavua"),
        "mrd" => ("miljardi", "miljardia"),
        "milj" => ("miljoona", "miljoonaa"),
        _ => return None,
    };
    Some(if one { sg } else { pl })
}

/// Abbreviations written with a period. `n.`, `v.` and `s.` only before a number
/// (n. 5 km, v. 1995, s. 12); elsewhere they are initials.
fn abbrev(lw: &str, before_number: bool) -> Option<&'static str> {
    Some(match lw {
        "esim" => "esimerkiksi",
        "jne" => "ja niin edelleen",
        "ym" => "ynnä muuta",
        "yms" => "ynnä muuta sellaista",
        "tms" => "tai muuta sellaista",
        "ns" => "niin sanottu",
        "vrt" => "vertaa",
        "ks" | "kts" => "katso",
        "mm" => "muun muassa",
        "nro" => "numero",
        "puh" => "puhelin",
        "ko" => "kyseinen",
        "em" => "edellä mainittu",
        "ao" => "asianomainen",
        "ts" => "toisin sanoen",
        "so" => "se on",
        "huom" => "huomaa",
        "prof" => "professori",
        "tri" => "tohtori",
        "hra" => "herra",
        "rva" => "rouva",
        "nti" => "neiti",
        "kpl" => "kappaletta",
        "mrd" => "miljardia",
        "milj" => "miljoonaa",
        "klo" => "kello",
        "pvm" => "päivämäärä",
        "yht" => "yhteensä",
        "alk" => "alkaen",
        "synt" => "syntynyt",
        "ca" => "noin",
        "ekr" => "ennen Kristusta",
        "jkr" => "jälkeen Kristuksen",
        "n" if before_number => "noin",
        "v" if before_number => "vuonna",
        "s" if before_number => "sivu",
        _ => return None,
    })
}

/// What a token group expands to: words for the ordinary word path, or
/// finished phonemes with their source word.
pub enum Piece {
    Word(String),
    Ph(String, String),
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn lower_initial(w: &str) -> bool {
    w.chars().next().is_some_and(|c| c.is_lowercase())
}

/// `H.MM` / `H:MM` as a clock time: kello neljätoista kolmekymmentä; minutes
/// below ten keep their zero (14.05 -> neljätoista nolla viisi).
fn clock(n: &str) -> Option<Vec<String>> {
    let (h, m) = n.split_once(['.', ':'])?;
    if !digits(h) || h.len() > 2 || !digits(m) || m.len() != 2 {
        return None;
    }
    let (h, m): (u128, u128) = (h.parse().ok()?, m.parse().ok()?);
    if h > 24 || m > 59 {
        return None;
    }
    let mut out = cardinal(h);
    if m > 0 {
        if m < 10 {
            out.push("nolla".into());
        }
        out.extend(cardinal(m));
    }
    Some(out)
}

/// `24.12.2023` and `24.12` as a date: the day as an ordinal, the month in the
/// partitive, the year as a cardinal (kahdeskymmenesneljäs joulukuuta …).
fn date(n: &str) -> Option<Vec<Piece>> {
    let parts: Vec<&str> = n.split('.').collect();
    if !(2..=3).contains(&parts.len()) || !parts.iter().all(|p| digits(p)) || parts[0].len() > 2 || parts[1].len() > 2 {
        return None;
    }
    let (d, m): (u128, usize) = (parts[0].parse().ok()?, parts[1].parse().ok()?);
    if !(1..=31).contains(&d) || !(1..=12).contains(&m) {
        return None;
    }
    let mut out = vec![numeral_piece(&ordinal_morphemes(d, Case::Nom)?), Piece::Word(format!("{}ta", MONTHS[m - 1]))];
    if let Some(y) = parts.get(2) {
        if y.len() != 4 {
            return None;
        }
        out.extend(cardinal(y.parse().ok()?).into_iter().map(Piece::Word));
    }
    Some(out)
}

/// The case an ordinal before this noun agrees in, read off the noun's ending
/// where the ending is unmistakable (5. kerroksessa -> viidennessä); the
/// nominative otherwise (12. joulukuuta, 5. luokka).
fn noun_case(w: &str) -> Case {
    if matches!(w, "päivänä" | "kertana" | "vuonna" | "kerroksena") {
        return Case::Ess;
    }
    for (suf, case) in [
        ("ssa", Case::Ine),
        ("ssä", Case::Ine),
        ("sta", Case::Ela),
        ("stä", Case::Ela),
        ("lla", Case::Ade),
        ("llä", Case::Ade),
        ("lta", Case::Abl),
        ("ltä", Case::Abl),
        ("lle", Case::All),
        ("ksi", Case::Tra),
    ] {
        if w.ends_with(suf) && w.chars().count() >= 6 {
            return case;
        }
    }
    Case::Nom
}

/// Finnish readings of a token group starting at `toks[i]`: inflected numerals
/// and abbreviations (5:ssä, EU:n), ordinals (5. luokka), dates, clock times
/// after klo, units after a number, abbreviations with a period. Returns the
/// pieces and the number of tokens they replace.
pub fn expand_token(toks: &[Tok], i: usize, prev: &str) -> Option<(Vec<Piece>, usize)> {
    let words = |v: Vec<String>| v.into_iter().map(Piece::Word).collect::<Vec<_>>();
    // the next token that is not a separator, and its index
    let after = |from: usize| toks.iter().enumerate().skip(from).find(|(_, t)| !matches!(t, Tok::Sep));
    let colon_ending = || match (toks.get(i + 1), toks.get(i + 2)) {
        (Some(Tok::Punct(':')), Some(Tok::Word(e))) if e.chars().all(|c| c.is_lowercase()) => Some(e.as_str()),
        _ => None,
    };
    match &toks[i] {
        Tok::Num(n) => {
            if digits(n) && n.len() <= 9 {
                if let Some((case, ord)) = colon_ending().and_then(colon_case) {
                    let v: u128 = n.parse().ok()?;
                    let w = if ord {
                        vec![numeral_piece(&ordinal_morphemes(v, case)?)]
                    } else {
                        cardinal_morphemes(v, case)?.iter().map(|m| numeral_piece(m)).collect()
                    };
                    return Some((w, 3));
                }
            }
            if n.contains(':') || matches!(prev, "klo" | "kello") {
                if let Some(w) = clock(n) {
                    return Some((words(w), 1));
                }
            }
            let dot = toks.get(i + 1) == Some(&Tok::Punct('.'));
            let next = if dot { after(i + 2) } else { None };
            let next_lower = matches!(next, Some((_, Tok::Word(w))) if lower_initial(w) && w.chars().count() > 1);
            match n.matches('.').count() {
                2 => {
                    if let Some(w) = date(n) {
                        return Some((w, 1));
                    }
                }
                // 24.12. needs its closing period and a sentence that goes on
                1 if dot && next_lower => {
                    if let Some(w) = date(n) {
                        return Some((w, 2));
                    }
                }
                _ => {}
            }
            // 5. luokka, 12. joulukuuta: a number, a period and a lower-case word
            if digits(n) && n.len() <= 3 && dot && next_lower {
                let Some((_, Tok::Word(w))) = next else { return None };
                let lw = w.to_lowercase();
                let month = MONTHS.iter().any(|m| lw.starts_with(m));
                let case = if month { Case::Nom } else { noun_case(&lw) };
                return Some((vec![numeral_piece(&ordinal_morphemes(n.parse().ok()?, case)?)], 2));
            }
            None
        }
        Tok::Word(w) => {
            // EU:n, USA:ssa, A:n: the letter names, then the ending
            if let Some(e) = colon_ending() {
                let upper = w.chars().count() <= 6 && w.chars().all(|c| c.is_uppercase());
                if upper && (colon_case(e).is_some() || matches!(e, "t" | "hun" | "hin" | "hon" | "hyn")) {
                    let lw = w.to_lowercase();
                    let ph = if w.chars().count() > 1 && !crate::spell::is_acronym(w) && !SPELLED.contains(&lw.as_str()) {
                        // read as a word: NATO:n -> naton
                        word(&format!("{lw}{e}"))
                    } else {
                        acronym_case(&lw, e)
                    };
                    return Some((vec![Piece::Ph(ph, format!("{lw}{e}"))], 3));
                }
            }
            // a unit right after a number
            let before = toks[..i].iter().rev().find(|t| !matches!(t, Tok::Sep));
            if let Some(Tok::Num(n)) = before {
                if toks.get(i + 1) != Some(&Tok::Punct(':')) {
                    if let Some(u) = unit(w, n == "1") {
                        return Some((vec![Piece::Word(u.to_string())], 1));
                    }
                }
            }
            // an abbreviation with its period
            if toks.get(i + 1) == Some(&Tok::Punct('.')) {
                let next = after(i + 2);
                let full = abbrev(&w.to_lowercase(), matches!(next, Some((_, Tok::Num(_)))))?;
                // the period ends the sentence only when nothing follows, or when a
                // capitalized word follows an abbreviation a sentence can end in
                let closes = matches!(w.to_lowercase().as_str(), "jne" | "ym" | "yms" | "tms");
                let keep = match next {
                    None => true,
                    Some((_, Tok::Word(nw))) => closes && !lower_initial(nw),
                    _ => false,
                };
                return Some((words(full.split(' ').map(String::from).collect()), if keep { 1 } else { 2 }));
            }
            None
        }
        _ => None,
    }
}

/// A spelled abbreviation with a case ending: EU:n -> ˌeːˈuːn; after a letter
/// name that ends in a consonant the ending comes through a doubled consonant
/// and ä, as it is spoken (PS:n -> pee-ässän, MM:ssä -> äm-ämmässä).
fn acronym_case(lw: &str, ending: &str) -> String {
    let c: Vec<char> = lw.chars().collect();
    let mut out = spell(&c);
    let mut tail = String::new();
    match c.last() {
        Some('f') | Some('l') | Some('m') | Some('n') | Some('r') | Some('s') => {
            let d = match c[c.len() - 1] {
                'f' => 'f',
                'l' => 'l',
                'm' => 'm',
                'n' => 'n',
                'r' => 'r',
                _ => 's',
            };
            tail.push(d);
            tail.push('ä');
        }
        Some('x') => tail.push('ä'),
        Some('z') => tail.push('a'),
        _ => {}
    }
    tail.push_str(ending);
    let t: Vec<char> = tail.chars().collect();
    let u = units(&t, None);
    render(&u, &[], &mut out);
    assimilate_nasals(&out)
}

/// Rewrites done on the raw text before tokenizing: a space as the thousands
/// separator (2 300 000, the Finnish standard) becomes a no-break space so the
/// number stays one token; a minus sign before a number is read; km/h and °C.
pub fn prenormalize(text: &str) -> Cow<'_, str> {
    if !text.chars().any(|c| c.is_ascii_digit()) {
        return Cow::Borrowed(text);
    }
    let c: Vec<char> = text.chars().collect();
    let n = c.len();
    let digit = |i: usize| c.get(i).is_some_and(|x| x.is_ascii_digit());
    let mut out = String::with_capacity(text.len() + 16);
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let at_start = i == 0 || matches!(c[i - 1], ' ' | '(' | '\u{00A0}');
        if matches!(ch, '-' | '−') && digit(i + 1) && at_start {
            out.push_str("miinus ");
            i += 1;
            continue;
        }
        if ch == '°' && c.get(i + 1) == Some(&'C') && !c.get(i + 2).is_some_and(|x| x.is_alphanumeric()) {
            out.push_str("celsiusastetta");
            i += 2;
            continue;
        }
        if i > 0 && (digit(i - 1) || c[i - 1] == ' ' && i > 1 && digit(i - 2)) {
            let rate = [("km/h", "kilometriä tunnissa"), ("m/s", "metriä sekunnissa")].into_iter().find(|(sym, _)| {
                let m = sym.chars().count();
                c[i..].iter().take(m).copied().eq(sym.chars()) && !c.get(i + m).is_some_and(|x| x.is_alphanumeric())
            });
            if let Some((sym, full)) = rate {
                out.push_str(full);
                i += sym.chars().count();
                continue;
            }
        }
        // a digit run that opens a number: not glued to an earlier digit group
        let opens = ch.is_ascii_digit()
            && ch != '0'
            && (i == 0 || !(digit(i - 1) || matches!(c[i - 1], '.' | ',' | ':' | '+')))
            && !(i >= 2 && c[i - 1] == ' ' && (digit(i - 2) || matches!(c[i - 2], '.' | ',' | ':')));
        if opens {
            let mut j = i;
            while digit(j) {
                j += 1;
            }
            if j - i <= 3 {
                let mut k = j;
                let mut groups = 0;
                while c.get(k) == Some(&' ') && digit(k + 1) && digit(k + 2) && digit(k + 3) && !digit(k + 4) {
                    k += 4;
                    groups += 1;
                }
                // not when more digits follow (a list, a phone number)
                let more = c.get(k) == Some(&' ') && digit(k + 1);
                if groups > 0 && !more {
                    for &x in &c[i..k] {
                        out.push(if x == ' ' { '\u{00A0}' } else { x });
                    }
                    i = k;
                    continue;
                }
            }
            for &x in &c[i..j] {
                out.push(x);
            }
            i = j;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn basics() {
        for (w, p) in [
            ("talo", "tˈalo"),
            ("kissa", "kˈissa"),
            ("kukka", "kˈukːa"),
            ("aakkoset", "ˈaːkːoset"),
            ("kengät", "kˈeŋŋæt"),
            ("lanka", "lˈaŋka"),
            ("englanti", "ˈeŋlantɪ"),
            ("yksi", "ˈyksɪ"),
            ("viisi", "vˈiːsɪ"),
            ("äiti", "ˈæitɪ"),
            ("työ", "tˈyø"),
            ("pöytä", "pˈøytæ"),
            ("koira", "kˈoira"),
            ("maailma", "mˈaːˌilma"),
            ("näkemiin", "nˈækemɪːn"),
            ("vaa'an", "vˈaːan"),
            ("asia", "ˈasɪa"),
            ("hiukan", "hˈiukan"),
            // weight-sensitive secondary stress
            ("suomalainen", "sˈuomalˌainen"),
            ("ravintola", "rˈavɪntˌola"),
            ("vuosituhat", "vˈuosɪtˌuhat"),
            ("todennäköisesti", "tˈodennˌækøisˌestɪ"),
            ("eurooppalainen", "ˈeuroːpːalˌainen"),
            ("amerikkalainen", "ˈamerˌikːalˌainen"),
            ("hallitukselle", "hˈallɪtˌukselle"),
            // compounds: harmony, stems, numerals, hyphens
            ("yhteiskunta", "ˈyhteiskˌunta"),
            ("yliopisto", "ˈylɪˌopɪsto"),
            ("epäonnistuminen", "ˈepæˌonnɪstˌumɪnen"),
            ("lentokone", "lˈentokˌone"),
            ("kaksikymmentäviisi", "kˈaksɪkˌymmentævˌiːsɪ"),
            ("itä-suomi", "ˈitæsˌuomɪ"),
            // loan letters
            ("pizza", "pˈitsa"),
            ("show", "ʃˈou"),
            ("celsius", "sˈelsius"),
            ("taxi", "tˈaksɪ"),
            ("åbo", "ˈoːbu"),
            // function words
            ("ja", "ja"),
            ("ole", "ˌole"),
            ("minä", "mˌinæ"),
            ("tv", "tˌeːvˈeː"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0).join(" "), "nolla");
        assert_eq!(cardinal(11).join(" "), "yksitoista");
        assert_eq!(cardinal(25).join(" "), "kaksikymmentäviisi");
        assert_eq!(cardinal(100).join(" "), "sata");
        assert_eq!(cardinal(235).join(" "), "kaksisataakolmekymmentäviisi");
        assert_eq!(cardinal(1000).join(" "), "tuhat");
        assert_eq!(
            cardinal(1995).join(" "),
            "tuhatyhdeksänsataayhdeksänkymmentäviisi"
        );
        assert_eq!(cardinal(2300).join(" "), "kaksituhattakolmesataa");
        assert_eq!(cardinal(1_000_000).join(" "), "miljoona");
        assert_eq!(
            cardinal(2_300_000).join(" "),
            "kaksi miljoonaa kolmesataatuhatta"
        );
        assert_eq!(cardinal(3_000_000_000).join(" "), "kolme miljardia");
    }

    #[test]
    fn sentences() {
        let g = crate::G2p::new(None);
        assert_eq!(g.phonemize("ole hyvä", crate::Lang::Fi), "ˌoleh hˈyvæ");
        assert_eq!(g.phonemize("tule tänne", crate::Lang::Fi), "tˈule tːˈænne");
        assert_eq!(
            g.phonemize("mennään kotiin", crate::Lang::Fi),
            "mˈennæːŋ kˈotɪːn"
        );
        assert_eq!(
            g.phonemize("kone toimii", crate::Lang::Fi),
            "kˈone tːˈoimɪː"
        );
        assert_eq!(
            g.phonemize("3,5", crate::Lang::Fi),
            "kˈolme pˈilkːu vˈiːsɪ"
        );
    }

    /// Boundary gemination needs a vowel-final trigger word (Wikipedia, "Finnish
    /// phonology", Sandhi); kolme, itse and nukke are the -e words without it
    /// (VISK § 34), and the essive adverb huomenna and the pronoun tuo have none.
    #[test]
    fn no_false_boundary_gemination() {
        let g = crate::G2p::new(None);
        for (s, p) in [
            ("tänään kotona", "tˈænæːŋ kˈotona"),
            ("pois kotoa", "pˈois kˈotoa"),
            ("ulos talosta", "ˈulos tˈalosta"),
            ("huomenna tulen", "hˈuomenna tˈulen"),
            ("kolme koiraa", "kˈolme kˈoiraː"),
            ("kolme miestä", "kˈolme mˈiestæ"),
            ("itse tehty", "ˈitse tˈehty"),
            ("tuo poika", "tˈuo pˈoika"),
            // real triggers still geminate: imperative, -e noun
            ("tule tänne", "tˈule tːˈænne"),
            ("kone toimii", "kˈone tːˈoimɪː"),
        ] {
            assert_eq!(g.phonemize(s, crate::Lang::Fi), p, "{s}");
        }
    }

    fn check(rows: &[(&str, &str)]) {
        let g = crate::G2p::new(None);
        for (s, p) in rows {
            assert_eq!(g.phonemize(s, crate::Lang::Fi), *p, "{s}");
        }
    }

    /// Boundary gemination by context: an infinitive geminates only after a
    /// governing verb, an imperative only after a negative verb or clause-initially
    /// (Wiktionary, Appendix:Finnish pronunciation, "final gemination").
    #[test]
    fn gemination_context() {
        check(&[
            ("Hän antaa meille vaatteet.", "hæn ˈantaː mˈeillev vˈaːtːeːt."),
            ("Voisitko antaa hänelle tämän?", "vˈoisɪtko ˈantaːh hˈænelle tːˈæmæn?"),
            ("Tomi käy kirkossa.", "tˈomɪ kˈæy kˈirkossa."),
            ("Se ei käy päinsä.", "se ei kˈæy pːˈæinsæ."),
            ("Tule tänne.", "tˈule tːˈænne."),
            ("Gangsterijoukko varasti rahaa.", "ɡˈaŋsterɪjˌoukːo vˈarastɪ rˈahaː."),
            ("Tom pitää siitä.", "tˈom pˈitæː sˈiːtæ."),
            ("Minun pitää lähteä.", "mˈinum pˈitæː lˈæhteæ."),
            ("Hän haluaa tehdä sen.", "hæn hˈaluaː tˈehdæs sˈen."),
            ("Voin antaa sen sinulle.", "vˈoin ˈantaːs sˈen sˈinulle."),
            ("En voi tehdä sitä.", "en vˈoi tːˈehdæs sˈitæ."),
            ("Ei saa tulla sisään.", "ei sˈaː tːˈullas sˈisæːn."),
            ("Tom käy koulua.", "tˈom kˈæy kˈoulua."),
            ("Älä käy kimppuun.", "ˈælæ kˈæy kːˈimpːuːn."),
            ("Mene ja avaa kaikki ovet.", "mˈenej ja ˈavaː kːˈaikːɪ ˈovet."),
            ("He puhuvat helposti kaikesta.", "he pˈuhuvat hˈelpostɪ kːˈaikesta."),
            ("Turisti kävi kaupassa.", "tˈurɪstɪ kˈævɪ kˈaupassa."),
            ("Hän pelasti kissan.", "hæm pˈelastɪ kˈissan."),
            ("Tie kulkee kylään.", "tˈie kˈulkeː kˈylæːn."),
            ("Steve kysyi tietä.", "stˈeve kˈysyi tˈietæ."),
            ("Koe kesti kauan.", "kˈoe kːˈestɪ kˈauan."),
            ("Talossanne kukaan ei asu.", "tˈalossˌanne kˈukaːn ei ˈasu."),
            ("otapa kakku", "ˈotapːa kˈakːu"),
            ("tulehan tänne", "tˈulehhan tˈænne"),
            ("Voimme mennä kotiin ja syödä kakkua.", "vˈoimme mˈennæ kːˈotɪːn ja sˈyødæ kːˈakːua."),
            ("Hän tuo kukkia.", "hæn tˈuo kˈukːɪa."),
            ("kuusi koiraa", "kˈuːsɪ kˈoiraː"),
            ("Tämä on hyvä asia.", "tˈæmæ on hˈyvæ ˈasɪa."),
        ]);
    }

    /// -sti: adverbs geminate, nouns and past tenses do not.
    #[test]
    fn sti_adverb_rule() {
        use super::sti_adverb;
        for w in ["nopeasti", "yleisesti", "varmasti", "helposti", "normaalisti", "hitaasti", "jatkuvasti", "ehdottomasti"] {
            assert!(sti_adverb(w), "{w}");
        }
        for w in ["turisti", "artisti", "eversti", "protesti", "varasti", "pelasti", "asti", "posti", "viesti"] {
            assert!(!sti_adverb(w), "{w}");
        }
    }

    /// -e nominatives geminate, the diphthong -ie and loans do not; -da/-ta.
    #[test]
    fn e_and_ta_rules() {
        use super::{e_trigger, syllables, ta_trigger};
        for w in ["perhe", "kone", "lause", "joukkue", "koe", "tule", "aie"] {
            assert!(e_trigger(w), "{w}");
        }
        // the diphthong -ie, a foreign letter near the end, nukke / saame compounds
        for w in ["tie", "college", "face", "bridge", "nukke", "saame", "lapsinukke"] {
            assert!(!e_trigger(w), "{w}");
        }
        for w in ["saada", "tuoda", "tehdä", "arvioida", "haluta", "pelata", "valita", "odota", "lopeta"] {
            assert!(ta_trigger(w, syllables(w)), "{w}");
        }
        for w in ["sota", "rata", "maailmansota", "junarata", "kaupunkia"] {
            assert!(!ta_trigger(w, syllables(w)), "{w}");
        }
    }

    #[test]
    fn trigger_tables_sorted_and_disjoint() {
        use super::{Trigger, trigger_class};
        use crate::fi_verbs::{INFINITIVE, INFINITIVE_CTX};
        assert!(INFINITIVE.windows(2).all(|w| w[0] < w[1]));
        assert!(INFINITIVE_CTX.windows(2).all(|w| w[0] < w[1]));
        for w in INFINITIVE_CTX {
            assert!(INFINITIVE.binary_search(w).is_err(), "{w} in both lists");
        }
        for w in super::NO_DOUBLE.iter() {
            assert_eq!(trigger_class(w), Trigger::No, "{w}");
        }
        assert_eq!(trigger_class("antaa"), Trigger::Infinitive);
        assert_eq!(trigger_class("tehdä"), Trigger::Always);
    }

    /// Letters: macrons, diacritics, ng+r, ph/th/gh, respelled loans, W in abbreviations.
    #[test]
    fn letters() {
        check(&[
            ("Tōkyō", "tˈoːkyoː"),
            ("kongressi", "kˈoŋressɪ"),
            ("sangria", "sˈaŋrɪa"),
            ("Smith", "smˈit"),
            ("Philip", "fˈilɪp"),
            ("Chris", "krˈis"),
            ("ghetto", "ɡˈetːo"),
            ("thriller", "trˈiller"),
            ("olethan", "ˈolethan"),
            ("Chicagossa", "ʃˈikaɡˌossa"),
            ("Facebookissa", "fˈeisbukˌissa"),
            ("Youtubessa", "jˈuːtuːbˌissa"),
            ("Thaimaassa", "tˈaimaːssa"),
            ("WC", "vˌeːsˈeː"),
            ("BMW", "bˌeːˌæmvˈeː"),
            ("www", "vˌeːvˌeːvˈeː"),
            ("äiti", "ˈæitɪ"),
        ]);
        // a + combining diaeresis is ä
        let g = crate::G2p::new(None);
        assert_eq!(g.phonemize("a\u{308}iti", crate::Lang::Fi), g.phonemize("äiti", crate::Lang::Fi));
    }

    /// Inflected numerals, ordinals, dates, clock times, units, abbreviations.
    #[test]
    fn numbers_and_abbreviations() {
        check(&[
            ("5:ssä", "vˈiːdessæ"),
            ("5:n", "vˈiːden"),
            ("3:lla", "kˈolmella"),
            ("25:n", "kˈahdeŋkˌymmenenvˌiːden"),
            ("300:aa", "kˈolmeasˌataː"),
            ("3:nnessa kerroksessa", "kˈolmannˌessa kˈerroksˌessa"),
            ("5:s", "vˈiːdes"),
            ("2:ksi", "kˈahdeksɪ"),
            ("7:ään", "sˈeitsemæːn"),
            ("12. joulukuuta", "kˈahdestˌoista jˈoulukˌuːta"),
            ("5. luokka", "vˈiːdes lˈuokːa"),
            ("5. kerroksessa", "vˈiːdennˌessæ kˈerroksˌessa"),
            ("3. päivänä", "kˈolmantˌena pˈæivænæ"),
            ("21. kerros", "kˈahdeskˌymmenesˌensɪmmˌæineŋ kˈerros"),
            ("24.12.2023", "kˈahdeskˌymmenesnˌeljæs jˈoulukˌuːta kˈaksɪtˌuhatːakˌaksɪkˌymmentækˌolme"),
            ("Tulen 24.12. kotiin", "tˈuleŋ kˈahdeskˌymmenesnˌeljæs jˈoulukˌuːta kˈotɪːn"),
            ("klo 14.30", "kˈello nˈeljætˌoista kˈolmekˌymmentæ"),
            ("kello 14.05", "kˈello nˈeljætˌoista nˈolla vˈiːsɪ"),
            ("2 300 000", "kˈaksɪ mˈiljoːnaː kˈolmesˌataːtˌuhatːa"),
            ("1 000", "tˈuhat"),
            ("100 000 euroa", "sˈatatˌuhatːa ˈeuroa"),
            ("1,5 kg", "ˈyksɪ pˈilkːu vˈiːsɪ kˈiloɡrˌammaː"),
            ("1 km", "ˈyksɪ kˈilomˌetrɪ"),
            ("50 km/h", "vˈiːsɪkˌymmentæ kˈilomˌetrɪæ tˈunnɪssa"),
            ("25 °C", "kˈaksɪkˌymmentævˌiːsɪ sˈelsiusˌastetːa"),
            ("-5 astetta", "mˈiːnus vˈiːsɪ ˈastetːa"),
            ("2 h 30 min", "kˈaksɪ tˈuntɪa kˈolmekˌymmentæ mˈinuːtːˌia"),
            ("n. 5 km", "nˈoin vˈiːsɪ kˈilomˌetrɪæ"),
            ("mm. tämä", "mˈuːn mˈuassa tˈæmæ"),
            ("esim. tämä", "ˈesɪmˌerkɪksɪ tˈæmæ"),
            ("EU:n", "ˌeːˈuːn"),
            ("EU:ssa", "ˌeːˈuːssa"),
            ("USA:n", "ˌuːˌæsˈaːn"),
            ("PS:n", "pˌeːˈæssæn"),
            ("MM:ssä", "ˌæmˈæmmæssæ"),
            ("NATO:n", "nˈaton"),
            ("A:n", "ˈaːn"),
            ("Tämä on arvosana 3.5. Se on hyvä.", "tˈæmæ on ˈarvosˌana kˈolme pˈilkːu vˈiːsɪ. se on hˈyvæ."),
        ]);
    }

    /// Inputs that must keep their old reading: decimals, plain years, cardinals
    /// with a final period, phone numbers.
    #[test]
    fn numbers_stay_as_before() {
        check(&[
            ("vuosina 2010 2011", "vˈuosɪna kˈaksɪtˌuhatːakˌymmeneŋ kˈaksɪtˌuhatːaˌyksɪtˌoista"),
            ("sivut 1 000 ja 2 000", "sˈivut tˈuhat ja kˈaksɪtˌuhatːa"),
            ("Olen 25. Hän on 30.", "ˌoleŋ kˈaksɪkˌymmentævˌiːsɪ. hæn oŋ kˈolmekˌymmentæ."),
            ("8.30", "kˈahdeksam pˈilkːu kˈolmekˌymmentæ"),
            ("Tämä on arvosana 3.5. Se on hyvä.", "tˈæmæ on ˈarvosˌana kˈolme pˈilkːu vˈiːsɪ. se on hˈyvæ."),
        ]);
        let g = crate::G2p::new(None);
        // a leading zero blocks merging into one number
        let p = g.phonemize("040 123 4567", crate::Lang::Fi);
        assert!(p.starts_with("nˈolla nˈeljæ nˈolla"), "{p}");
    }

    #[test]
    fn case_numerals() {
        use super::{cardinal_case, ordinal, Case};
        for n in (0..=100_000u128).step_by(7).chain([1_000_000, 2_300_000, 123_456_789]) {
            assert_eq!(cardinal_case(n, Case::Nom), Some(cardinal(n)), "{n}");
        }
        let want = [
            "ensimmäinen", "toinen", "kolmas", "neljäs", "viides", "kuudes", "seitsemäs", "kahdeksas", "yhdeksäs",
            "kymmenes", "yhdestoista", "kahdestoista", "kolmastoista", "neljästoista", "viidestoista", "kuudestoista",
            "seitsemästoista", "kahdeksastoista", "yhdeksästoista", "kahdeskymmenes", "kahdeskymmenesensimmäinen",
            "kahdeskymmenestoinen", "kahdeskymmeneskolmas", "kahdeskymmenesneljäs", "kahdeskymmenesviides",
            "kahdeskymmeneskuudes", "kahdeskymmenesseitsemäs", "kahdeskymmeneskahdeksas",
            "kahdeskymmenesyhdeksäs", "kolmaskymmenes", "kolmaskymmenesensimmäinen",
        ];
        for (i, w) in want.iter().enumerate() {
            assert_eq!(ordinal(i as u128 + 1, Case::Nom).as_deref(), Some(*w), "{}", i + 1);
        }
    }
}
