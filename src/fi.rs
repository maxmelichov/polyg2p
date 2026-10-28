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

use crate::Out;

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

/// Words that geminate the initial consonant of the next word (boundary
/// gemination) and are not caught by the suffix rules: imperatives, first
/// infinitives, connegatives, and a few adverbs / nouns.
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
    "syö",
    "tee",
    "käy",
    "juo",
    "tuo",
    "vie",
    "lue",
    "istu",
    "avaa",
    "muista",
    "lähde",
    "jätä",
    "pidä",
    "nouse",
    "antaa",
    "ottaa",
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
    "kirjoittaa",
    "ymmärtää",
    "tietää",
    "ajatella",
    "pitää",
    "haluta",
    "syödä",
    "juoda",
    "käydä",
    "viedä",
    "tuoda",
    "ostaa",
    "myydä",
    "maksaa",
    "elää",
    "kuolla",
    "asua",
    "oppia",
    "opettaa",
    "auttaa",
    "etsiä",
    "löytää",
    "lähteä",
    "jäädä",
    "istua",
    "nukkua",
    "herätä",
    "pelata",
    "soittaa",
    "laulaa",
    "tanssia",
    "uida",
    "juosta",
    "kävellä",
    "ajaa",
    "lentää",
    "matkustaa",
    "rakastaa",
    "tuntea",
    "muistaa",
    "unohtaa",
    "yrittää",
    "aloittaa",
    "lopettaa",
    "jatkaa",
    "avata",
    "sulkea",
    "kysyä",
    "vastata",
    "pyytää",
    "tarvita",
    "osata",
    "ajeta",
    "auteta",
    "oteta",
    "pueta",
    "pukea",
    "purra",
    "sanota",
    "tulla",
    "mennä",
    "tehdä",
    "kiinni",
    "luo",
    "taa",
    "terve",
    "tuore",
    "kolme",
    "itse",
    "tänne",
    "sinne",
    "minne",
    "luokse",
    "taakse",
    "kotiin",
    "ulos",
    "sisään",
    "pois",
    "eteen",
    "alle",
    "ylle",
    "päälle",
    "kauas",
    "tänään",
    "huomenna",
];
/// Words in -lle / -nne that are not allative / adverb forms (names).
const NO_DOUBLE: &[&str] = &[
    "kalle", "ville", "pelle", "nalle", "hille", "galle", "polle", "promille", "anne", "hanne",
    "janne", "manne", "marianne", "susanne", "wille", "elle", "belle", "google", "apple", "mike",
    "nike", "one", "some", "line", "online", "home", "office", "service", "jatte", "kate", "me",
    "te", "he", "se", "ne", "ehkä",
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
                    // ng + vowel / r = long ŋ (kengän, kongressi); ng + consonant = ŋ (englanti)
                    let l = is_vowel(nx2) || nx2 == 'r';
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
                    out.push(Unit::C("tʃ", false));
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

/// Allative -lle before an enclitic (-kaan, -kin, -ko, -pa, ...) geminates the
/// enclitic's consonant (kenellekään -> kenellekːään): index of that consonant.
fn enclitic_geminate(c: &[char]) -> Option<usize> {
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
    let lw = w.to_lowercase();
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
    let letters: Vec<&str> = c
        .iter()
        .map(|&ch| letter_name(ch))
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

fn syllables(w: &str) -> usize {
    let c: Vec<char> = w.chars().collect();
    units(&c, None)
        .iter()
        .filter(|u| matches!(u, Unit::V(..)))
        .count()
}

/// Does this word geminate the initial consonant of the next one?
fn doubles(src: &str) -> bool {
    if NO_DOUBLE.contains(&src) {
        return false;
    }
    if DOUBLE.contains(&src) {
        return true;
    }
    let n = syllables(src);
    if n < 2 {
        return false;
    }
    if src.ends_with("lle") || src.ends_with("nne") {
        return true;
    }
    if src.ends_with("tse") && n >= 3 {
        return true;
    }
    if src.ends_with("sti")
        && n >= 3
        && !src.ends_with("viesti")
        && !src.ends_with("posti")
        && !src.ends_with("ksti")
    {
        return true;
    }
    // nominative -e nouns (perhe, kone, lause): not pronouns, not -mme/-tte verb forms
    src.ends_with('e') && !src.ends_with("mme") && !src.ends_with("tte") && !src.ends_with("ee")
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
        let (a, b) = outs.split_at_mut(k + 1);
        let prev = &mut a[k];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let Some((off, c)) = initial_consonant(&cur.ph) else {
            continue;
        };
        if c == "ts" || c == "tʃ" {
            continue;
        }
        let already_long =
            cur.ph[off + c.len()..].starts_with('ː') || cur.ph[off + c.len()..].starts_with(c);
        if doubles(&prev.src) && !already_long {
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
            g.phonemize("kolme miestä", crate::Lang::Fi),
            "kˈolmem mˈiestæ"
        );
        assert_eq!(
            g.phonemize("3,5", crate::Lang::Fi),
            "kˈolme pːˈilkːu vˈiːsɪ"
        );
    }
}
