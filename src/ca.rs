//! Catalan (Central) grapheme-to-phoneme rules, espeak-style notation.
//!
//! Rules only: syllabification, orthographic stress (written accents, else
//! penultimate for words ending in a vowel, vowel + s, -en, -in; final
//! otherwise), Central vowel reduction (unstressed a/e -> ə, o -> u), open vs
//! closed stressed e/o from written accents or, when unaccented, from
//! ending statistics (`data/ca_open.tsv`, derived from Wiktionary), the b/v
//! merger with spirant allophones β ð ɣ, final devoicing, regressive voicing
//! assimilation, final-cluster simplification (sant, molt, camp, amb, cinc),
//! silent final -r (infinitives, -ar/-er/-or words), x/ix/ex, ig, tx/tj/tg,
//! ll/l·l/ny, and sentence-level sandhi (els amics -> əlz əmˈiks).
//!
//! Notation follows espeak-ng's `ca` voice: `ə` for the reduced vowel,
//! `ɛ ɔ` open mid vowels, `ɾ` tap between vowels, `r` elsewhere, `β ð ɣ`
//! spirants, `ʎ ɲ ŋ ɱ`, `tʃ dʒ dz` affricates (folded to ligatures by
//! `fold.rs`), `j w` glides, `ˈ` before the stressed vowel after its onset
//! consonants (`kˈazə`, `dəspɾˈes`).

use crate::Out;
use std::collections::HashMap;
use std::sync::OnceLock;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "coma";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "be", "se", "de", "e", "efə", "ʒe", "ak", "i", "ʒɔtə", "ka", "elə", "emə", "enə", "o", "pe", "ku",
    "erə", "esə", "te", "u", "be", "bedobblə", "iks", "iɣɾeɣə", "zɛtə",
]);

const OPEN_TSV: &str = include_str!("../data/ca_open.tsv");

/// Clitics and function words espeak reads without a stress mark.
const UNSTRESSED: &[&str] = &[
    "de", "la", "a", "i", "que", "el", "en", "per", "del", "dels", "un", "les", "els", "amb", "al", "es", "hi", "si",
    "als", "ho", "ens", "se", "pel", "li", "em", "us", "pels", "et", "uns", "lo", "nos", "los", "me", "te",
    "vos", "ne", "ses", "tes", "sos", "tos", "mos", "n", "l", "d", "m", "t",
    "s", "ls", "ns",
    
];
/// Enclitic pronouns after a hyphen (unstressed, reduced).
const ENCLITIC: &[&str] = &[
    "me", "te", "se", "nos", "vos", "lo", "la", "los", "les", "li", "ho", "hi", "ne", "en", "el", "els", "em",
    "et", "es", "ens", "us", "n", "m", "t", "s", "l", "ls", "ns",
];

/// Monosyllables whose final -r is silent in Central Catalan.
const MONO_SILENT_R: &[&str] = &["clar", "dur", "flor", "por", "ser", "fer", "dir", "plor",
    
];
/// Polysyllables in -ur with a silent r.
const UR_SILENT: &[&str] = &["segur", "madur", "insegur", "immadur"];
/// Words in -rs where the r belongs to the stem (not a plural of a silent-r word).
const RS_STEMS: &[&str] = &[
    "univers", "divers", "envers", "revers", "pervers", "dispers", "convers", "invers", "immers", "submers",
    "concurs", "discurs", "recurs", "transcurs", "decurs", "excurs", "advers", "anvers", "obvers", "tergivers",
];
/// Compounds and prefixes whose first vowel is not reduced: (prefix, letter index, phone).
const UNREDUCED: &[(&str, usize, &str)] = &[
    ("sobre", 1, "o"), ("benvingu", 1, "e"), ("benestar", 1, "e"), ("gairebé", 1, "a"), ("qualsevol", 2, "a"),
    ("portaveu", 1, "ɔ"), ("passaport", 1, "a"), ("malaurada", 1, "a"), ("contrasenya", 1, "ɔ"),
];
/// Polysyllables in -or/-ar/-er/-ir whose final r is pronounced.
const R_PRONOUNCED: &[&str] = &[
    "favor", "motor", "valor", "vapor", "vigor", "rigor", "tumor", "sabor", "sonor", "decor", "licor", "labor",
    "furor", "fervor", "pudor", "tricolor", "incolor", "locomotor", "ciclomotor", "fluor", "meteor", "estor",
    "icor", "astor", "clor", "amor", "honor", "humor", "rumor", "horror", "error", "terror", "fòsfor", "condor",
    "popular", "singular", "regular", "irregular", "particular", "similar", "familiar", "militar", "circular",
    "nuclear", "solar", "polar", "lunar", "escalar", "exemplar", "vulgar", "cel·lular", "molecular", "muscular",
    "titular", "secular", "angular", "triangular", "rectangular", "auxiliar", "peculiar", "articular",
    "espectacular", "capitular", "consular", "insular", "peninsular", "radar", "cigar", "jaguar", "avatar",
    "autocar", "caviar", "tsar", "calamar", "atzar", "basar", "billar", "mular", "ultramar", "plenamar",
    "baixamar", "aerofar", "hangar", "bazar", "pilar", "altar", "nectar", "nèctar", "estàndard", "lumbar",
    "linear", "ocular", "dinar", "sonar", "velar", "tubular", "vascular", "cardiovascular", "mandibular",
    "sincer", "enter", "sever", "auster", "iber", "berber", "khmer", "moher", "acer", "afer", "bòer",
    "emir", "safir", "visir", "vampir", "tapir", "papir", "albir", "respir", "sospir", "retir", "algutzir",
    "caixmir", "autogir", "faquir", "baixkir", "cuir", "amateur", "zèfir",
];

/// Paroxytones in -er that are nouns (the r is pronounced); other -er
/// paroxytones are infinitives (córrer, néixer) with a silent r.
const ER_NOUNS: &[&str] = &[
    "càncer", "cràter", "caràcter", "pòster", "màster", "hàmster", "líder", "súper", "búnquer", "revòlver",
    "júpiter", "èter", "úter", "kàiser", "làser", "màser", "cúter", "escúter", "escàner", "córner", "clúster",
    "bíter", "càrter", "catèter", "helicòpter", "ornitòpter", "quadrilàter", "equilàter", "suèter", "wàter",
    "vàter", "zòster", "cadàver", "jòquer", "pòquer", "quàquer", "mànager", "gàngster", "espòiler", "tràiler",
    "còrner", "sènior", "júnior", "lúcifer", "húmer", "fèmur", "àlber", "sàber", "clíper", "rèquiem",
];

/// Adjective-stem endings (before the -a of -ament) that mark an adverb.
const ADV_A_ENDINGS: &[&str] = &[
    "ad", "id", "ud", "at", "ct", "pt", "st", "nt", "rt", "os", "iv", "ic", "ur", "er", "ar", "ns", "pl",
    "x", "rm", "tr", "rg", "ix", "mpl",
    
];
/// Nouns in -ament the endings above would misread as adverbs.
const NOUN_MENT: &[&str] = &[
    "ajuntament", "enfrontament", "assentament", "trasplantament", "atemptament", "comportament", "apartament",
    "departament", "alliberament", "pensament", "compensament", "armament", "desarmament", "allargament",
    "ajustament", "tractament", "aprofitament", "esgotament", "alletament", "reclutament", "contractament",
    "acatament", "dilatament", "rebaixament", "abaixament", "segment", "fragment", "pigment", "ferment",
    "torment", "augment", "complement", "suplement", "implement", "aixecament", "esdeveniment", "coneixement",
    "creixement", "naixement", "element", "parlament", "testament", "firmament", "medicament", "sagrament",
    "ornament", "fonament", "reglament", "campament", "subministrament", "lliurament", "escalfament",
    "plantejament", "enregistrament", "atansament", "acostament",
];
/// Adjective stems (feminine, ending in -a) with other endings that form adverbs in -ament.
const ADV_A_STEMS: &[&str] = &[
    "sola", "mala", "plena", "bona", "humana", "sana", "fina", "llunyana", "quotidiana", "clandestina", "moderna",
    "externa", "interna", "eterna", "serena", "comuna", "oportuna", "digna", "falsa", "escassa", "espessa",
    "grossa", "russa", "confusa", "difusa", "precisa", "concisa", "expressa", "vaga", "cega", "llarga", "amarga",
    "completa", "concreta", "discreta", "secreta", "neta", "quieta", "estreta", "inquieta", "nova", "freda",
    "calenta", "lenta", "blanca", "franca", "poca", "seca", "rica", "nua", "mútua", "antiga", "senzilla",
    "dolça", "sincera", "lleugera", "primera", "darrera", "mera", "vera", "severa", "entera", "sencera",
    "segura", "dura", "pura", "madura", "obscura", "summa", "altra", "justa", "injusta", "honesta", "modesta",
    "llesta", "trista", "vasta", "robusta", "ampla", "simple", "doble", "profunda", "rotunda", "fecunda",
    "immensa", "intensa", "extensa", "densa", "baixa", "fixa", "complexa", "exacta", "directa", "perfecta",
    "correcta", "estricta", "indirecta", "incorrecta", "absoluta", "immediata", "ferma", "forta", "curta",
    "certa", "oberta", "alta", "sobtada", "privada",
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Vowel,
    Glide,
    Nasal,
    Liquid,
    /// obstruent, voiced?
    Obs(bool),
}

fn kind(ph: &str) -> Kind {
    match ph {
        "a" | "e" | "ɛ" | "i" | "o" | "ɔ" | "u" | "ə" => Kind::Vowel,
        "j" | "w" => Kind::Glide,
        "m" | "n" | "ɲ" | "ŋ" | "ɱ" => Kind::Nasal,
        "l" | "ʎ" | "r" | "ɾ" => Kind::Liquid,
        "b" | "d" | "ɡ" | "β" | "ð" | "ɣ" | "v" | "z" | "ʒ" | "dʒ" | "dz" => Kind::Obs(true),
        _ => Kind::Obs(false),
    }
}

fn voiced_of(ph: &str) -> &'static str {
    match ph {
        "p" => "b",
        "t" => "d",
        "k" => "ɡ",
        "f" => "v",
        "s" => "z",
        "ʃ" => "ʒ",
        "tʃ" => "dʒ",
        "ts" => "dz",
        "b" => "b",
        "d" => "d",
        "ɡ" => "ɡ",
        "β" => "β",
        "ð" => "ð",
        "ɣ" => "ɣ",
        "v" => "v",
        "z" => "z",
        "ʒ" => "ʒ",
        "dʒ" => "dʒ",
        "dz" => "dz",
        _ => "",
    }
}
fn voiceless_of(ph: &str) -> &'static str {
    match ph {
        "b" | "β" => "p",
        "d" | "ð" => "t",
        "ɡ" | "ɣ" => "k",
        "v" => "f",
        "z" => "s",
        "ʒ" => "ʃ",
        "dʒ" => "tʃ",
        "dz" => "ts",
        "p" => "p",
        "t" => "t",
        "k" => "k",
        "f" => "f",
        "s" => "s",
        "ʃ" => "ʃ",
        "tʃ" => "tʃ",
        "ts" => "ts",
        _ => "",
    }
}

#[derive(Clone, Debug)]
struct Seg {
    ph: &'static str,
    nucleus: bool,
    /// orthographic vowel (base letter) of a nucleus
    letter: char,
    /// written accent: Some(open?) for è/ò (true), é/ó (false), à/í/ú (false)
    accent: Option<bool>,
    /// index of the letter in the word
    idx: usize,
}

fn seg(ph: &'static str) -> Seg {
    Seg { ph, nucleus: false, letter: ' ', accent: None, idx: 0 }
}

fn base(c: char) -> char {
    match c {
        'à' => 'a',
        'è' | 'é' => 'e',
        'í' | 'ï' | 'y' => 'i',
        'ò' | 'ó' => 'o',
        'ú' | 'ü' => 'u',
        _ => c,
    }
}
fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'à' | 'è' | 'é' | 'í' | 'ò' | 'ó' | 'ú' | 'ï' | 'ü' | 'y')
}
fn accent_of(c: char) -> Option<bool> {
    match c {
        'è' | 'ò' => Some(true),
        'à' | 'é' | 'í' | 'ó' | 'ú' => Some(false),
        _ => None,
    }
}

fn open_table() -> &'static HashMap<&'static str, (f32, u32)> {
    static T: OnceLock<HashMap<&'static str, (f32, u32)>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m = HashMap::new();
        for line in OPEN_TSV.lines() {
            if line.starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() >= 3 {
                if let (Ok(p), Ok(n)) = (f[1].parse::<f32>(), f[2].parse::<u32>()) {
                    m.insert(f[0], (p, n));
                }
            }
        }
        m
    })
}

/// Frequent words whose stressed e/o goes against the ending statistics (hand-written).
const OPEN_WORDS: &[&str] = &[
    "cosa", "coses", "hora", "hores", "home", "homes", "dona", "dones", "bona", "bones", "bo", "bons", "nova",
    "noves", "nou", "nous", "pot", "poden", "lloc", "llocs", "com", "sort", "fort", "forts", "forta", "fortes",
    "mort", "morts", "morta", "cor", "cors", "flor", "flors", "escola", "escoles", "obra", "obres", "porta",
    "portes", "roba", "prova", "proves", "poble", "pobles", "nord", "acord", "acords", "doncs", "font", "fonts",
    "pont", "ponts", "foc", "focs", "joc", "jocs", "cop", "cops", "roca", "hola",
    "jo", "roig", "boig", "goig", "cos", "cossos", "os", "ordre", "ordres", "bosc", "boscos", "costa",
    "costes", "força", "forces", "noia", "noies", "roda", "moda", "boda", "colla", "olla", "oli", "or", "fora",
    "prou", "bou", "sou", "oca", "tort", "port", "ports", "hort", "record", "tropa", "poca", "poc", "pocs", "poques", "roc", "groc", "groga", "rosa", "roses", "nosa", "prosa", "posa", "posen",
    "poso", "sol", "sols", "soc", "esport", "esports", "corda", "cordes", "cova", "coves", "ou", "ous",
    "goma", "proa", "gota", "gotes", "nota", "notes", "coll", "colls", "moll", "soca", "toca", "toquen", "toco",
    "troba", "troben", "trobo", "trobes", "col", "cols", "vol", "volen", "vols", "volem", "voleu",
    "porto", "porten", "portem", "porteu", "plora", "ploren", "ploro", "mora", "gros", "grossos", "grossa", "grosses", "cost", "costos", "post", "posts", "tros", "trossos",
    "terra", "terres", "ferro", "serra", "guerra", "guerres", "cert", "certa", "certs", "certes", "obert",
    "oberta", "oberts", "obertes", "verd", "verda", "verds", "verdes", "nen", "nena", "nens", "nenes", "tres",
    "deu", "deus", "peu", "peus", "veu", "veus", "creu", "creus", "preu", "preus", "fred", "freda", "freds",
    "cel", "cels", "mel", "aquell",
    "aquella", "aquells", "aquelles", "sec", "seca", "secs", "seques",
    "hivern", "govern", "governs", "escena", "escenes", "verb", "verbs", "perd", "perdre",
    "perden", "perdo", "servei", "serveis", "veure", "creure", "treure", "beure", "seure", "mesos", "res", "feina", "feines", "feia", "deia", "veia", "feien", "deien",
    "esquerra", "gel", "arrel", "arrels", "lleu", "europeu", "europeus",
    "museu", "museus", "hebreu", "ferm", "ferma", "expert", "experta", "experts", "concert", "concerts",
    "desert", "deserts", "cobert", "coberta", "descobert", "ofert", "oferta", "ofertes", "trec", "trenc", "venc",
    "vens", "dent", "dents", "serp", "serps", "setze", "quinze",
    "disset", "divuit", "dinou", "vuit", "mestre", "mestres", "mestra", "metre", "metres", "problema", "problemes", "haver", "poder", "voler", "saber", "valer",
    "caber", "deure",
    "aquest", "aquesta", "aquests", "aquestes", "contra", "nom", "noms", "bon", "sembla", "semblen", "consell", "consells", "mena", "dreta", "cadena", "cadenes", "plena", "plenes", "parella", "parelles", "parell", "parells", "vermell", "vermella", "vermells", "vermelles", "trenta", "arreu", "reus", "imperi", "francesos", "esposa", "quelcom", "con", "vora", "propi", "propis", "pròpia", "noi", "nois", "alhora", "sola", "soles", "to", "plaer", "museu", "museus", "mesos", "sens", "concert", "ensems", "dimecres", "tenebra", "secreta", "escena", "escola", "terra", "serp", "cel", "mel", "gel",
    "porc", "porcs", "sona", "sonen", "pobla", "poema", "poemes", "zero", "sencer", "sencera", "cabell", "cabells", "aparell", "aparells", "ven", "penes", "estreta", "estret", "perfecta", "perfecte", "verge", "fenomen", "rellotge", "rellotges", "aleshores", "setze", "univers", "pep", "texas",
    "colles", "sons", "pons", "flora", "eco", "arena", "orella", "orelles", "meravella", "empenta", "cel·la", "afores", "ter", "coll", "malson", "col·legi", "col·legis",
];
const CLOSED_WORDS: &[&str] = &[
    "molt", "molts", "molta", "moltes", "tot", "tots", "tota", "totes", "dos", "no", "fou",
    "pou", "pous", "fons", "forma", "formes", "cotxe", "cotxes", "pols", "ona", "ones", "persona", "persones",
    "corona", "gola", "conte", "contes", "compte", "comptes", "ell", "ella", "ells", "elles", "gent",
    "temps", "sempre", "pena", "arena", "mes", "dret", "drets", "crec", "neu",
    "paper", "papers", "carrer", "carrers", "primer", "primera", "metge", "fetge", "llet", "net", "neta", "fet",
    "fets", "feta", "fetes", "ple", "plens", "entre", "sense", "mentre", "mateix", "mateixa", "mateixos", "mateixes", "conec", "coneix", "coneixen",
    "sembla", "semblen", "pes", "pesos", "mesa", "meses", "estrena", "peça", "peces",
    "castell", "castells", "ocell", "ocells", "anell", "anells",
    "clavell", "cervell", "orella", "orelles", "abella", "abelles", "ampolla", "ampolles", "fulla", "fulles", "francesos", "anglesos", "pagesos", "sota", "nosaltres", "vosaltres", "llarg", "llarga", "llargs", "llargues", "sord", "sorda", "sords", "surt",
    "surten", "trenta", "cinquanta", "seixanta", "setanta", "vuitanta", "noranta", "cent", "vint", "era", "eres",
    "ser", "fer", "veritat",
    "festa", "festes", "terme", "termes", "pedra", "pedres", "pell", "pells", "vell", "vella", "vells", "velles", "meu", "teu", "seu", "meus", "teus", "seus", "greu", "greus", "breu", "breus", "sistema", "sistemes", "tema", "temes", "esquema", "cinema", "poema", "poemes", "onze", "boca", "copa", "vent", "vents", "primavera", "tenen", "sobre", "sobretot", "foto", "fotos", "joves", "jove", "doble", "donen", "gos", "gossos", "membres", "membre", "reina", "secret", "crema", "diners", "darrers", "darrer", "ebre", "dret", "drets", "sota", "dimecres", "sens", "fera", "esfera", "set", "sec", "seca",
    "foto", "fotos", "moto", "sopa", "bossa", "bosses", "tomba", "bord", "cort", "pop", "llop", "fosc", "dono", "formen", "escolta", "escolten", "fossin", "dobles", "dotze", "extrem", "extrema", "extrems", "extremes", "concret", "concreta", "concrets", "concretes", "adreça", "pressa", "expressa", "bell", "bella", "bells", "belles", "alegre", "nega", "directa", "directes", "directe", "olot", "neu", "correu", "fosca",
    "andreu", "mateu", "relleu", "bartomeu", "correu", "correus", "rei", "reis", "llei", "lleis", "motos", "febre", "cec", "coco", "catorze", "tretze", "encendre", "travessa", "ossos", "paquet", "paquets",
];

/// Open (true) or closed stressed e/o without a written accent: hand-written
/// exceptions first, then the ending statistics.
fn is_open(letters: &[char], si: usize) -> bool {
    let w: String = letters.iter().collect::<String>().replace('ł', "l·l");
    if OPEN_WORDS.contains(&w.as_str()) {
        return true;
    }
    if CLOSED_WORDS.contains(&w.as_str()) {
        return false;
    }
    let t = open_table();
    let sv = letters[si];
    let rest: String = letters[si + 1..].iter().collect();
    let prev = if si > 0 { letters[si - 1] } else { '#' };
    let mut best: Option<f32> = None;
    let rest_chars: Vec<char> = rest.chars().collect();
    for k in 0..=rest_chars.len() {
        let mut key = String::new();
        key.push(sv);
        key.extend(rest_chars[..k].iter());
        if k == rest_chars.len() {
            key.push('#');
        }
        if let Some(&(p, n)) = t.get(key.as_str()) {
            if n >= 3 {
                best = Some(p);
            }
        }
        let key2 = format!("{prev}{key}");
        if let Some(&(p, n)) = t.get(key2.as_str()) {
            if n >= 3 {
                best = Some(p);
            }
        }
    }
    best.map_or(false, |p| p > 0.5)
}

#[derive(Clone, Copy, PartialEq)]
enum Stress {
    Primary,
    None,
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    let lw: String = w
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphabetic() || matches!(c, '\'' | '-' | '·'))
        .collect();
    let lw = lw.replace("l·l", "ł").replace("ŀl", "ł").replace('·', "");
    if lw.chars().all(|c| !c.is_alphabetic()) {
        return String::new();
    }
    // hyphenated compounds: each part keeps its stress; enclitics are unstressed
    if lw.contains('-') {
        let parts: Vec<&str> = lw.split('-').filter(|p| !p.is_empty()).collect();
        let mut out = String::new();
        for (k, p) in parts.iter().enumerate() {
            let last = k + 1 == parts.len();
            let is_clitic = |x: &str| x.split('\'').all(|q| q.is_empty() || ENCLITIC.contains(&q));
            let next_enclitic = parts.get(k + 1).map_or(false, |n| is_clitic(n));
            let piece = if k > 0 && is_clitic(p) {
                // enclitic chain: -me'l, -se'n, -los-hi
                p.split('\'').filter(|q| !q.is_empty()).map(clitic).collect::<Vec<_>>().concat()
            } else if *p == "i" {
                "i".to_string()
            } else {
                // a verb keeps its final r/t before an enclitic (portar-se, tenint-ho)
                word_plain(p, Stress::Primary, last && !next_enclitic)
            };
            glue(&mut out, &piece);
        }
        return out;
    }
    if lw.contains('\'') {
        let parts: Vec<&str> = lw.split('\'').filter(|p| !p.is_empty()).collect();
        let mut out = String::new();
        for (k, p) in parts.iter().enumerate() {
            let last = k + 1 == parts.len();
            let piece = if k == 0 && parts.len() > 1 && matches!(*p, "l" | "d" | "m" | "t" | "s" | "n") {
                p.to_string()
            } else if k > 0 && ENCLITIC.contains(p) {
                clitic(p)
            } else {
                let st = if UNSTRESSED.contains(p) { Stress::None } else { Stress::Primary };
                word_plain(p, st, last)
            };
            glue(&mut out, &piece);
        }
        return out;
    }
    word_full(&lw)
}

/// An unstressed enclitic pronoun.
fn clitic(p: &str) -> String {
    match p {
        "ho" => "u".into(),
        "hi" => "i".into(),
        "n" | "m" | "t" | "s" | "l" | "ls" | "ns" => p.to_string(),
        _ => word_plain(p, Stress::None, true),
    }
}

/// Append `piece` to `out` applying sandhi at the junction (as between words).
fn glue(out: &mut String, piece: &str) {
    if out.is_empty() {
        out.push_str(piece);
        return;
    }
    let mut cur = piece.to_string();
    sandhi(out, &mut cur, true);
    // r between vowels inside the compound is a tap (fer-ho, anar-hi)
    if out.ends_with('r') && cur.chars().next().map_or(false, |c| kind(&c.to_string()) == Kind::Vowel) {
        out.pop();
        out.push('ɾ');
    }
    out.push_str(&cur);
}

fn word_full(lw: &str) -> String {
    // adverbs in -ment: the adjective keeps its stress, -ment takes another
    if let Some(stem) = lw.strip_suffix("ment") {
        if is_adverb_stem(stem, lw) {
            let mut st = word_plain(stem, Stress::Primary, true);
            if stem.ends_with('r') && !st.ends_with('r') {
                st.push('r');
            }
            if st.ends_with('n') {
                st.pop();
                st.push('m');
            }
            return format!("{st}mˈen");
        }
    }
    if lw.chars().count() == 1 {
        let ch = lw.chars().next().unwrap();
        if ch.is_ascii_lowercase() && !matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u') {
            let name = LETTERS.unwrap()[(ch as u8 - b'a') as usize];
            let pos = name.find(|c: char| "aeiouɛɔ".contains(c)).unwrap_or(0);
            return format!("{}ˈ{}", &name[..pos], &name[pos..]);
        }
    }
    let st = if UNSTRESSED.contains(&lw) { Stress::None } else { Stress::Primary };
    word_plain(lw, st, true)
}

fn is_adverb_stem(stem: &str, lw: &str) -> bool {
    let n = stem.chars().count();
    if n < 3 || NOUN_MENT.contains(&lw) {
        return false;
    }
    let nuclei = stem.chars().filter(|&c| is_vowel_letter(c)).count();
    if nuclei == 0 {
        return false;
    }
    if stem.chars().any(|c| matches!(c, 'à' | 'è' | 'é' | 'í' | 'ò' | 'ó' | 'ú')) {
        return true;
    }
    let last = stem.chars().last().unwrap();
    if !is_vowel_letter(last) {
        // consonant stem: final+ment, feliç+ment, fàcil+ment
        return n >= 4;
    }
    if let Some(adj) = stem.strip_suffix('a') {
        if ADV_A_STEMS.contains(&stem) || ADV_A_ENDINGS.iter().any(|e| adj.ends_with(e)) {
            return true;
        }
        return false;
    }
    if stem.ends_with('e') {
        return stem.ends_with("ble") || stem.ends_with("ple") || stem.ends_with("bre") || stem.ends_with("tre")
            || stem.ends_with("gre") || stem.ends_with("rme") || stem.ends_with("ure") || stem == "lliure";
    }
    if stem.ends_with('u') {
        return matches!(stem, "breu" | "greu" | "suau" | "lleu");
    }
    false
}

/// Does a word-final -r (or -rs) stay silent?
fn final_r_silent(letters: &[char], nuclei: &[usize], stressed: usize) -> bool {
    let w: String = letters.iter().collect();
    let w = w.strip_suffix('s').filter(|_| w.ends_with("rs")).unwrap_or(&w).to_string();
    if !w.ends_with('r') {
        return false;
    }
    // stress not on the last syllable: pronounced (càncer, dòlar), except the
    // -er infinitives (córrer, conèixer, néixer)
    if stressed + 1 != nuclei.len() {
        return w.ends_with("er") && !ER_NOUNS.contains(&w.as_str());
    }
    if nuclei.len() == 1 {
        return MONO_SILENT_R.contains(&w.as_str());
    }
    if R_PRONOUNCED.contains(&w.as_str()) || RS_STEMS.iter().any(|st| w.as_str() == &st[..st.len() - 1]) {
        return false;
    }
    if w.ends_with("ur") || w.ends_with("ür") {
        return UR_SILENT.contains(&w.as_str());
    }
    true
}

/// Core rules for one plain word. `final_` = the word is not followed by an
/// enclitic / compound part (final cluster rules and devoicing apply).
fn word_plain(lw: &str, stress: Stress, final_: bool) -> String {
    let c: Vec<char> = lw.chars().filter(|c| c.is_alphabetic()).collect();
    let n = c.len();
    if n == 0 {
        return String::new();
    }
    let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { c[i as usize] } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 2);
    let mut i = 0usize;
    while i < n {
        let ch = c[i];
        let pv = at(i as isize - 1);
        let nx = at(i as isize + 1);
        if is_vowel_letter(ch) {
            // vowel cluster
            let start = i;
            let mut j = i;
            while j < n && is_vowel_letter(c[j]) {
                j += 1;
            }
            let after = at(j as isize);
            let pc = at(start as isize - 1);
            let m = j - start;
            let mut k = 0;
            // does the previous letter of the cluster stand as a nucleus?
            let mut prev_nuc = false;
            while k < m {
                let l = c[start + k];
                let next = if k + 1 < m { Some(c[start + k + 1]) } else { None };
                let first = k == 0;
                let b = base(l);
                let mut nuc = true;
                match l {
                    'i' | 'u' | 'y' if accent_of(l).is_none() => {
                        // infinitives/gerunds in vowel + ir/int are hiatus (construir, obeint)
                        let tail: String = c[(j + 1).min(n)..].iter().collect();
                        let inf_hiatus = l == 'i' && prev_nuc && next.is_none()
                            && ((after == 'r' && matches!(tail.as_str(), "" | "é" | "à" | "ia" | "ies" | "ien" | "em" | "eu" | "an" | "às" | "íem" | "íeu"))
                                || (after == 'n' && at(j as isize + 1) == 't'));
                        if let Some(nv) = next {
                            if l == 'u' && first && matches!(pc, 'g' | 'q') {
                                if matches!(base(nv), 'e' | 'i') {
                                    // gue/gui/que/qui: silent
                                } else {
                                    segs.push(seg("w"));
                                }
                                nuc = false;
                            } else if (prev_nuc || (first && pc == '#')) && !(l == 'u' && start + k == 2 && lw.starts_with("reun")) {
                                // rising glide after a vowel or at word start (noia, diuen, iogurt)
                                segs.push(seg(if b == 'u' { "w" } else { "j" }));
                                nuc = false;
                            } else {
                                // u before i/u (cuina) and i before u (ciutat) are nuclei;
                                // otherwise hiatus after a consonant (ciència, família)
                                segs.push(Seg { ph: if b == 'u' { "u" } else { "i" }, nucleus: true, letter: b, accent: None, idx: start + k });
                            }
                        } else if prev_nuc && !inf_hiatus {
                            // falling diphthong off-glide; i before x / final g is silent (caixa, baix, vaig)
                            let final_ig = after == 'g' && (j + 1 == n || (j + 2 == n && c[n - 1] == 's'));
                            if l != 'u' && (after == 'x' || final_ig) {
                                // silent
                            } else {
                                segs.push(seg(if b == 'u' { "w" } else { "j" }));
                            }
                            nuc = false;
                        } else {
                            segs.push(Seg { ph: if b == 'u' { "u" } else { "i" }, nucleus: true, letter: b, accent: None, idx: start + k });
                        }
                    }
                    'ü' if matches!(pc, 'g' | 'q') && first && next.is_some() => {
                        segs.push(seg("w"));
                        nuc = false;
                    }
                    _ => {
                        segs.push(Seg { ph: "", nucleus: true, letter: b, accent: accent_of(l), idx: start + k });
                    }
                }
                prev_nuc = nuc;
                k += 1;
            }
            i = j;
            continue;
        }
        let next_front = matches!(nx, 'e' | 'i' | 'è' | 'é' | 'í' | 'ï' | 'y');
        let next_vowel = is_vowel_letter(nx);
        let prev_vowel = is_vowel_letter(pv);
        let initial = i == 0;
        let last = i + 1 == n;
        match ch {
            'b' | 'v' => {
                if nx == 'b' || nx == 'v' || (pv == 'm' && !next_vowel && !matches!(nx, 'l' | 'r')) {
                    // bb: one stop; mb + consonant: silent b (ambdós)
                } else {
                    let stop = initial || matches!(pv, 'm' | 'n' | 'p' | 't' | 'd' | 'k' | 'c' | 'g') || last
                        || (!next_vowel && !matches!(nx, 'l' | 'r' | 'h'));
                    segs.push(seg(if stop { "b" } else { "β" }));
                }
            }
            'd' => {
                if nx == 'd' {
                } else if nx == 'j' {
                    segs.push(seg("dʒ"));
                    i += 1;
                } else {
                    let stop = initial || matches!(pv, 'n' | 'l' | 'm' | 'p' | 'b' | 't' | 'k' | 'c' | 'g') || last
                        || (!next_vowel && !matches!(nx, 'r' | 'h'));
                    segs.push(seg(if stop { "d" } else { "ð" }));
                }
            }
            'g' => {
                if nx == 'g' && matches!(at(i as isize + 2), 'e' | 'i' | 'è' | 'é' | 'í') {
                    segs.push(seg("dʒ"));
                    i += 1;
                } else if nx == 'g' {
                } else if next_front {
                    segs.push(seg("ʒ"));
                } else if pv == 'i' && (last || (i + 2 == n && nx == 's'))
                    && !c.iter().any(|&x| matches!(x, 'à' | 'è' | 'é' | 'ò' | 'ó' | 'ú')) {
                    segs.push(seg("tʃ"));
                } else if nx == 'n' {
                    segs.push(seg("ŋ"));
                } else if nx == 'u' || nx == 'ü' {
                    let stop = initial || matches!(pv, 'n' | 'm' | 'p' | 'b' | 't' | 'd' | 'k' | 'c');
                    segs.push(seg(if stop { "ɡ" } else { "ɣ" }));
                } else {
                    let stop = initial || matches!(pv, 'n' | 'm' | 'p' | 'b' | 't' | 'd' | 'k' | 'c') || last
                        || (!next_vowel && !matches!(nx, 'l' | 'r' | 'h'));
                    segs.push(seg(if stop { "ɡ" } else { "ɣ" }));
                }
            }
            'c' => {
                if nx == 'h' {
                    segs.push(seg("k"));
                    i += 1;
                } else if nx == 'c' && matches!(at(i as isize + 2), 'e' | 'i' | 'è' | 'é' | 'í') {
                    segs.push(seg("k"));
                } else if next_front && pv == 'x' {
                    // xc + e/i: one s (excepte)
                } else if next_front {
                    segs.push(seg("s"));
                } else if nx == 'n' {
                    segs.push(seg("ŋ"));
                } else if nx == 'k' || nx == 'q' {
                } else {
                    segs.push(seg("k"));
                }
            }
            'ç' => segs.push(seg("s")),
            'q' => {
                segs.push(seg("k"));
                // the u is handled in the vowel cluster (silent before e/i, w before a/o)
            }
            'k' => segs.push(seg("k")),
            'z' => segs.push(seg("z")),
            's' => {
                if nx == 's' {
                    segs.push(seg("s"));
                    i += 1;
                } else if nx == 'h' && prev_vowel && is_vowel_letter(at(i as isize + 2)) {
                    segs.push(seg("z"));
                } else if nx == 'c' && matches!(at(i as isize + 2), 'e' | 'i' | 'è' | 'é' | 'í') {
                    // sc + e/i: one s
                    segs.push(seg("s"));
                    i += 1;
                } else if prev_vowel && next_vowel {
                    segs.push(seg("z"));
                } else if !initial && !prev_vowel && next_vowel && pv == 'n' && i == 4 && c.len() > 4
                    && c[..4].iter().map(|&x| base(x)).collect::<String>() == "tran" {
                    segs.push(seg("z"));
                } else {
                    segs.push(seg("s"));
                }
            }
            'j' => segs.push(seg("ʒ")),
            'h' => {}
            'l' => {
                if nx == 'l' {
                    segs.push(seg("ʎ"));
                    i += 1;
                } else {
                    segs.push(seg("l"));
                }
            }
            'ł' => {
                segs.push(seg("l"));
                segs.push(seg("l"));
            }
            'r' => {
                if nx == 'r' {
                    segs.push(seg("r"));
                    i += 1;
                } else if initial || matches!(pv, 'n' | 'l' | 's') || !next_vowel {
                    segs.push(seg("r"));
                } else {
                    segs.push(seg("ɾ"));
                }
            }
            'n' => {
                let n2 = at(i as isize + 2);
                let front2 = matches!(n2, 'e' | 'i' | 'è' | 'é' | 'í' | 'ï' | 'y');
                let n3 = at(i as isize + 3);
                let front3 = matches!(n3, 'e' | 'i' | 'è' | 'é' | 'í' | 'ï' | 'y');
                if nx == 'y' {
                    segs.push(seg("ɲ"));
                    i += 1;
                } else if nx == 't' && (matches!(n2, 'k' | 'q') || (n2 == 'c' && !front3) || (n2 == 'g' && !front3)) {
                    segs.push(seg("ŋ"));
                } else if nx == 't' && (n2 == 'j' || (n2 == 'g' && front3)) {
                    segs.push(seg("ɲ"));
                } else if matches!(nx, 'k' | 'q') || (nx == 'c' && !front2) || (nx == 'g' && !front2) {
                    segs.push(seg("ŋ"));
                } else if matches!(nx, 'b' | 'v' | 'p' | 'm') {
                    segs.push(seg("m"));
                } else if nx == 'f' {
                    segs.push(seg("ɱ"));
                } else if nx == 'x' || nx == 'j' || (nx == 'g' && front2) || (nx == 'l' && n2 == 'l') {
                    segs.push(seg("ɲ"));
                } else {
                    segs.push(seg("n"));
                }
            }
            'm' => {
                if nx == 'f' {
                    segs.push(seg("ɱ"));
                } else {
                    segs.push(seg("m"));
                }
            }
            'p' => {
                if initial && matches!(nx, 'n' | 't') {
                    // pneumàtic: silent (Wiktionary keeps the p of psicologia)
                } else if nx == 'h' {
                    segs.push(seg("f"));
                    i += 1;
                } else if nx == 'p' || (pv == 'm' && nx == 't') {
                } else {
                    segs.push(seg("p"));
                }
            }
            't' => {
                if nx == 'x' {
                    segs.push(seg("tʃ"));
                    i += 1;
                } else if nx == 'j' || (nx == 'g' && matches!(at(i as isize + 2), 'e' | 'i' | 'è' | 'é' | 'í')) {
                    segs.push(seg("dʒ"));
                    i += 1;
                } else if nx == 'z' {
                    segs.push(seg("dz"));
                    i += 1;
                } else if nx == 'l' && at(i as isize + 2) == 'l' {
                    segs.push(seg("ʎ"));
                    i += 2;
                } else if nx == 'l' && !initial {
                    segs.push(seg("l"));
                } else if nx == 'm' && !initial && lw.starts_with("setman") {
                    segs.push(seg("m"));
                } else if nx == 'h' {
                    segs.push(seg("t"));
                    i += 1;
                } else if pv == 'n' && !next_vowel && !matches!(nx, 'r' | 'l' | 'h' | 'z') && !last {
                    // Montcada, Montjuïc: silent t
                } else if nx == 't' {
                } else {
                    segs.push(seg("t"));
                }
            }
            'f' => {
                if nx != 'f' {
                    segs.push(seg("f"));
                }
            }
            'x' => {
                let ex_prefix = pv == 'e' || pv == 'è' || pv == 'é';
                let next_vowel = next_vowel || (nx == 'h' && is_vowel_letter(at(i as isize + 2)));
                let ex_initial = ex_prefix
                    && (i == 1
                        || lw.starts_with("inex")
                        || lw.starts_with("reex")
                        || lw.starts_with("hex")
                        || lw.starts_with("coex")
                        || lw.starts_with("preex"));
                if lw.starts_with("arxi") && i == 2 {
                    segs.push(seg("tʃ"));
                } else if pv == 'i' && at(i as isize - 2) == 'f' {
                    // fix, prefix, sufix
                    segs.push(seg("k"));
                    segs.push(seg("s"));
                } else if initial || pv == 'i' || !prev_vowel {
                    segs.push(seg("ʃ"));
                } else if next_vowel && ex_initial {
                    segs.push(seg("ɡ"));
                    segs.push(seg("z"));
                } else {
                    segs.push(seg("k"));
                    segs.push(seg("s"));
                }
            }
            'w' => segs.push(seg("w")),
            _ => {}
        }
        i += 1;
    }
    // --- stress
    let nuclei: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| s.nucleus).map(|(i, _)| i).collect();
    if nuclei.is_empty() {
        return segs.iter().map(|s| s.ph).collect();
    }
    let primary = nuclei.iter().position(|&ix| segs[ix].accent.is_some()).unwrap_or_else(|| {
        if nuclei.len() == 1 {
            return 0;
        }
        let last = c[n - 1];
        let pen = c[n - 2];
        let last_nucleus_idx = segs[*nuclei.last().unwrap()].idx;
        let ends_vowel = is_vowel_letter(last) && last_nucleus_idx == n - 1;
        let ends_vs = last == 's' && is_vowel_letter(pen) && last_nucleus_idx == n - 2;
        let ends_en_in = (last == 'n' && matches!(pen, 'e' | 'i')) && last_nucleus_idx == n - 2;
        if ends_vowel || ends_vs || ends_en_in {
            nuclei.len() - 2
        } else {
            nuclei.len() - 1
        }
    });
    // --- vowel quality
    for (k, &ix) in nuclei.iter().enumerate() {
        let s = &segs[ix];
        let stressed = k == primary && stress != Stress::None;
        let ph: &'static str = if stressed {
            match s.letter {
                'a' => "a",
                'i' => "i",
                'u' => "u",
                'e' => match s.accent {
                    Some(true) => "ɛ",
                    Some(false) => "e",
                    None => {
                        let eu_final = s.idx + 2 <= n && matches!(c[s.idx + 1], 'u' | 'i')
                            && (s.idx + 2 == n || (s.idx + 3 == n && c[n - 1] == 's'))
                            && !CLOSED_WORDS.contains(&lw);
                        if eu_final || is_open(&c, s.idx) { "ɛ" } else { "e" }
                    }
                },
                'o' => match s.accent {
                    Some(true) => "ɔ",
                    Some(false) => "o",
                    None => if is_open(&c, s.idx) { "ɔ" } else { "o" },
                },
                _ => "a",
            }
        } else {
            match s.letter {
                'a' => "ə",
                'e' => {
                    // unstressed e stays e before a (teatre, ideal, àrea)
                    let next_a = segs.get(ix + 1).map_or(false, |t| t.nucleus && t.letter == 'a');
                    if next_a { "e" } else { "ə" }
                }
                'o' => "u",
                'i' => "i",
                'u' => "u",
                _ => "ə",
            }
        };
        segs[ix].ph = ph;
    }
    for (pre, idx, phone) in UNREDUCED {
        if lw.starts_with(pre) {
            if let Some(sg) = segs.iter_mut().find(|sg| sg.nucleus && sg.idx == *idx) {
                sg.ph = phone;
            }
        }
    }
    // --- phone list and post-processing
    let mut ph: Vec<&'static str> = segs.iter().map(|s| s.ph).collect();
    let mut stress_at = nuclei[primary];
    // bl/gl after a stressed vowel: geminate (poble, regla)
    {
        let mut k = 0;
        while k + 1 < ph.len() {
            let stem = lw.starts_with("pobl") || lw.starts_with("publ") || lw.starts_with("públ");
            if matches!(ph[k], "β" | "ɣ") && ph[k + 1] == "l" && k > 0 && (k - 1 == nuclei[primary] || stem) {
                let stop = if ph[k] == "β" { "b" } else { "ɡ" };
                ph[k] = stop;
                ph.insert(k, stop);
                k += 1;
            }
            k += 1;
        }
    }
    // silent final r
    let rs = c.ends_with(&['r', 's']);
    if final_ && (c[n - 1] == 'r' || rs) && final_r_silent(&c, &nuclei, primary) {
        let pos = if rs { ph.len() - 2 } else { ph.len() - 1 };
        if matches!(ph[pos], "r" | "ɾ") {
            ph.remove(pos);
        }
    }
    if final_ {
        // final cluster simplification
        let l = ph.len();
        let ends = |p: &[&str], pat: &[&str]| p.len() >= pat.len() && &p[p.len() - pat.len()..] == pat;
        if ends(&ph, &["n", "t"]) || ends(&ph, &["l", "t"]) || ends(&ph, &["n", "d"]) {
            ph.truncate(l - 1);
        } else if ends(&ph, &["n", "t", "s"]) || ends(&ph, &["l", "t", "s"]) {
            ph.remove(l - 2);
        } else if ends(&ph, &["m", "p"]) || ends(&ph, &["m", "b"]) {
            ph.truncate(l - 1);
        } else if ends(&ph, &["m", "p", "s"]) {
            ph.remove(l - 2);
        } else if ends(&ph, &["ŋ", "k"]) || ends(&ph, &["ŋ", "ɡ"]) {
            ph.truncate(l - 1);
        } else if ends(&ph, &["ŋ", "k", "s"]) {
            ph.remove(l - 2);
        } else if ends(&ph, &["s", "t", "s"]) {
            ph.remove(l - 3);
        }
        // final devoicing
        if let Some(last) = ph.last_mut() {
            if kind(last) == Kind::Obs(true) {
                *last = voiceless_of(last);
            }
        }
    }
    // regressive voicing assimilation
    for k in (0..ph.len().saturating_sub(1)).rev() {
        let cur = ph[k];
        if !matches!(kind(cur), Kind::Obs(_)) {
            continue;
        }
        let next = ph[k + 1];
        let want = match kind(next) {
            Kind::Obs(v) => Some(v),
            Kind::Nasal => Some(true),
            Kind::Liquid if matches!(cur, "s" | "z") => Some(true),
            Kind::Liquid if matches!(cur, "t" | "d") && matches!(next, "l" | "ʎ") => Some(true),
            _ => None,
        };
        if let Some(v) = want {
            let rep = if v { voiced_of(cur) } else { voiceless_of(cur) };
            if !rep.is_empty() {
                ph[k] = rep;
            }
        }
    }
    // the stressed vowel's index may have shifted by the bl insertion
    if ph.len() != segs.len() {
        // recompute: count nuclei
        let mut cnt = 0;
        for (k, p) in ph.iter().enumerate() {
            if kind(p) == Kind::Vowel {
                if cnt == primary {
                    stress_at = k;
                    break;
                }
                cnt += 1;
            }
        }
    }
    // --- assemble
    let mut out = String::with_capacity(lw.len() * 2);
    // espeak puts the mark immediately before the stressed vowel (kˈazə, dəspɾˈes)
    let mark_at = if stress == Stress::None { usize::MAX } else { stress_at };
    for (k, p) in ph.iter().enumerate() {
        if k == mark_at {
            out.push('ˈ');
        }
        out.push_str(p);
    }
    out
}

fn first_phone(s: &str) -> Option<&str> {
    let s = s.trim_start_matches(['ˈ', 'ˌ']);
    let mut it = s.char_indices();
    let (_, c) = it.next()?;
    // two-character phones
    let two = &s[..c.len_utf8() + it.next().map_or(0, |(_, d)| d.len_utf8())];
    if matches!(two, "tʃ" | "dʒ" | "dz" | "ts" | "ks" | "ɡz") {
        return Some(two);
    }
    Some(&s[..c.len_utf8()])
}

fn last_phone(s: &str) -> Option<&str> {
    let mut it = s.char_indices().rev();
    let (i, _) = it.next()?;
    if let Some((j, _)) = it.next() {
        let two = &s[j..];
        if matches!(two, "tʃ" | "dʒ" | "dz" | "ts" | "ks") {
            return Some(two);
        }
    }
    Some(&s[i..])
}

fn replace_first(s: &str, old: &str, new: &str) -> String {
    let marks: String = s.chars().take_while(|&c| c == 'ˈ' || c == 'ˌ').collect();
    let rest = &s[marks.len()..];
    format!("{marks}{new}{}", &rest[old.len()..])
}

fn replace_last(s: &str, old: &str, new: &str) -> String {
    format!("{}{new}", &s[..s.len() - old.len()])
}

fn continuant(k: Kind, ph: &str) -> bool {
    matches!(k, Kind::Vowel | Kind::Glide | Kind::Liquid)
        || matches!(ph, "s" | "z" | "ʃ" | "ʒ" | "f" | "v" | "β" | "ð" | "ɣ")
}

/// Sandhi between `prev` and `cur` (no pause between them). `inside` = inside
/// one orthographic word (compound/enclitic), where stops stay stops.
fn sandhi(prev: &mut String, cur: &mut String, inside: bool) {
    let (Some(f), Some(l)) = (first_phone(cur).map(|s| s.to_string()), last_phone(prev).map(|s| s.to_string())) else {
        return;
    };
    let fk = kind(&f);
    let lk = kind(&l);
    // word-initial b/d/g soften after a vowel or a non-nasal consonant
    let soft = match f.as_str() {
        // after a vowel, glide, liquid or fricative; not after a stop or a nasal
        "b" => continuant(lk, &l),
        "d" => continuant(lk, &l) && l != "l",
        "ɡ" => continuant(lk, &l),
        _ => false,
    };
    if soft && !inside {
        let rep = match f.as_str() {
            "b" => "β",
            "d" => "ð",
            _ => "ɣ",
        };
        *cur = replace_first(cur, &f, rep);
    } else if soft && inside && lk == Kind::Vowel {
        let rep = match f.as_str() {
            "b" => "β",
            "d" => "ð",
            _ => "ɣ",
        };
        *cur = replace_first(cur, &f, rep);
    }
    // final fricatives/affricates voice before a vowel or a voiced consonant;
    // final stops only before a voiced consonant
    let next_voiced = matches!(fk, Kind::Vowel | Kind::Glide | Kind::Nasal | Kind::Liquid | Kind::Obs(true))
        || matches!(f.as_str(), "β" | "ð" | "ɣ");
    let next_voiced_cons = next_voiced && fk != Kind::Vowel && fk != Kind::Glide;
    let rep = match l.as_str() {
        "s" if next_voiced => "z",
        "ʃ" if next_voiced => "ʒ",
        "tʃ" if next_voiced => "dʒ",
        "ts" if next_voiced => "dz",
        "p" if next_voiced_cons => "b",
        "t" if next_voiced_cons => "d",
        "k" if next_voiced_cons => "ɡ",
        "f" if next_voiced_cons => "v",
        _ => "",
    };
    if !rep.is_empty() {
        *prev = replace_last(prev, &l, rep);
    }
    // nasal assimilation
    if l == "n" {
        let rep = match f.as_str() {
            "k" | "ɡ" | "ɣ" => "ŋ",
            "b" | "β" | "p" | "m" => "m",
            "f" => "ɱ",
            _ => "",
        };
        if !rep.is_empty() {
            *prev = replace_last(prev, "n", rep);
        }
    }
}

/// Sentence pass: voicing of word-final s/ʃ/tʃ and stops, spirant b/d/g after
/// a vowel, nasal assimilation (els amics -> əlz əmˈiks, un gat -> uŋ ɡˈat).
pub fn join(outs: &mut [Out]) {
    // l·l words come split at the middle dot (col + legi): re-merge the known prefixes
    for k in 1..outs.len() {
        let (a, b) = outs.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let ll = cur.src.starts_with('l')
            && !cur.src.starts_with("ll")
            && LL_PAIRS.iter().any(|(a, b)| *a == prev.src.as_str() && cur.src.starts_with(b));
        if ll {
            let merged = format!("{}·{}", prev.src, cur.src);
            cur.ph = word(&merged);
            cur.src = merged;
            prev.ph.clear();
        }
    }
    for k in 1..outs.len() {
        let (a, b) = outs.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        sandhi(&mut prev.ph, &mut cur.ph, false);
    }
}

/// (first half, start of the second half) of l·l words; "" = any second half.
/// Halves that are words themselves (pel, sol, al, ...) need a specific second half.
const LL_PAIRS: &[(&str, &str)] = &[
    ("col", ""), ("il", ""), ("intel", ""), ("instal", ""), ("excel", ""), ("novel", ""), ("rebel", ""),
    ("satel", ""), ("nul", ""), ("tranquil", ""), ("paral", ""), ("cristal", ""), ("metal", ""), ("flagel", ""),
    ("vacil", ""), ("oscil", ""), ("apel", ""), ("cancel", ""), ("xarel", ""), ("hel", ""), ("escil", ""),
    ("ampul", ""), ("medul", ""), ("sil", "lab"), ("til", "la"), ("pel", "lícul"), ("sol", "licit"),
    ("al", "leg"), ("al", "lusi"), ("al", "lèrg"), ("al", "lot"), ("al", "ludi"), ("al", "lucin"), ("mil", "lím"),
    ("mil", "lenni"), ("mil", "ligram"), ("mil", "lilitr"), ("cel", "lul"), ("cel", "la"), ("mol", "lusc"),
    ("mal", "leab"), ("pal", "li"), ("bel", "lic"), ("gal", "lès"), ("gal", "lic"), ("cal", "ligraf"),
    ("fal", "lera"), ("pol", "len"), ("pol", "luci"), ("libel", ""), ("ral", "li"), ("bal", "lest"),
];

const ONES: [&str; 20] = [
    "zero", "un", "dos", "tres", "quatre", "cinc", "sis", "set", "vuit", "nou", "deu", "onze", "dotze", "tretze",
    "catorze", "quinze", "setze", "disset", "divuit", "dinou",
];
const TENS: [&str; 10] = ["", "", "vint", "trenta", "quaranta", "cinquanta", "seixanta", "setanta", "vuitanta", "noranta"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = n / 100;
    let r = n % 100;
    if h == 1 {
        out.push("cent".into());
    } else if h > 1 {
        out.push(format!("{}-cents", ONES[h as usize]));
    }
    if r == 0 {
        return;
    }
    if r < 20 {
        out.push(ONES[r as usize].into());
    } else {
        let t = TENS[(r / 10) as usize];
        let u = r % 10;
        if u == 0 {
            out.push(t.into());
        } else if r < 30 {
            out.push(format!("vint-i-{}", ONES[u as usize]));
        } else {
            out.push(format!("{t}-{}", ONES[u as usize]));
        }
    }
}

/// Catalan cardinal number words ("vint-i-un", "dos-cents", "mil", "un milió", "mil milions").
pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n == 0 {
        out.push("zero".into());
        return out;
    }
    let scales: [(u128, &str, &str); 3] =
        [(1_000_000_000_000_000_000, "trilió", "trilions"), (1_000_000_000_000, "bilió", "bilions"), (1_000_000, "milió", "milions")];
    let mut rest = n;
    for (val, one, many) in scales {
        let q = rest / val;
        if q == 0 {
            continue;
        }
        rest %= val;
        if q == 1 {
            out.push("un".into());
            out.push(one.into());
        } else {
            out.extend(cardinal(q));
            out.push(many.into());
        }
    }
    let th = rest / 1000;
    rest %= 1000;
    if th > 0 {
        if th > 1 {
            out.extend(cardinal(th));
        }
        out.push("mil".into());
    }
    below_1000(rest, &mut out);
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "per cent",
        '$' => "dòlars",
        '€' => "euros",
        '£' => "lliures",
        '&' => "i",
        '+' => "més",
        '@' => "arrova",
        '°' => "graus",
        '=' => "igual a",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};
    #[test]
    fn basics() {
        for (w, p) in [
            ("casa", "kˈazə"),
            ("xocolata", "ʃukulˈatə"),
            ("examen", "əɡzˈamən"),
            ("text", "tˈekst"),
            ("cavall", "kəβˈaʎ"),
            ("col·legi", "kullˈɛʒi"),
            ("any", "ˈaɲ"),
            ("vaig", "bˈatʃ"),
            ("cotxe", "kˈotʃə"),
            ("platja", "plˈadʒə"),
            ("sant", "sˈan"),
            ("molt", "mˈol"),
            ("cantar", "kəntˈa"),
            ("primer", "pɾimˈe"),
            ("senyor", "səɲˈo"),
            ("mar", "mˈar"),
            ("futur", "futˈur"),
            ("però", "pəɾˈɔ"),
            ("història", "istˈɔɾiə"),
            ("ciutat", "siwtˈat"),
            ("aigua", "ˈajɣwə"),
            ("llengua", "ʎˈeŋɡwə"),
            ("veure", "bˈɛwɾə"),
            ("poble", "pˈɔbblə"),
            ("setmana", "səmmˈanə"),
            ("fred", "fɾˈɛt"),
            ("verd", "bˈɛrt"),
            ("caixa", "kˈaʃə"),
            ("baix", "bˈaʃ"),
            ("exemple", "əɡzˈemplə"),
            ("taxi", "tˈaksi"),
            ("camp", "kˈam"),
            ("temps", "tˈems"),
            ("amb", "əm"),
            ("cinc", "sˈiŋ"),
            ("teatre", "teˈatɾə"),
            ("ideal", "iðeˈal"),
            ("desembre", "dəzˈembɾə"),
            ("transició", "tɾənzisiˈo"),
            ("noia", "nˈɔjə"),
            ("diuen", "dˈiwən"),
            ("cuina", "kˈujnə"),
            ("ràpidament", "rˈapiðəmˈen"),
            ("finalment", "finˈalmˈen"),
            ("moment", "mumˈen"),
            ("ajuntament", "əʒuntəmˈen"),
            ("l'home", "lˈɔmə"),
            ("d'una", "dˈunə"),
            ("portar-se", "purtˈarsə"),
            ("fer-ho", "fˈeɾu"),
            ("vint-i-dos", "bˈintiðˈos"),
            ("se'n", "sən"),
            ("els", "əls"),
            ("de", "də"),
            ("hola", "ˈɔlə"),
            ("dona", "dˈɔnə"),
            ("quatre", "kwˈatɾə"),
            ("qüestió", "kwəstiˈo"),
            ("veïna", "bəˈinə"),
            ("absolut", "əpsulˈut"),
            ("dissabte", "disˈaptə"),
            ("psicologia", "psikuluʒˈiə"),
            ("anys", "ˈaɲs"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(21).join(" "), "vint-i-un");
        assert_eq!(cardinal(35).join(" "), "trenta-cinc");
        assert_eq!(cardinal(200).join(" "), "dos-cents");
        assert_eq!(cardinal(1001).join(" "), "mil un");
        assert_eq!(cardinal(2000).join(" "), "dos mil");
        assert_eq!(cardinal(1_000_000).join(" "), "un milió");
        assert_eq!(cardinal(3_000_000_000).join(" "), "tres mil milions");
    }

    #[test]
    fn sentences() {
        let g = crate::G2p::new(None);
        assert_eq!(g.phonemize("els amics", crate::Lang::Ca), "əlz əmˈiks");
        assert_eq!(g.phonemize("un gat", crate::Lang::Ca), "uŋ ɡˈat");
        assert_eq!(g.phonemize("molt bé", crate::Lang::Ca), "mˈol βˈe");
        assert_eq!(g.phonemize("cap vegada", crate::Lang::Ca), "kˈab bəɣˈaðə");
        assert_eq!(g.phonemize("vaig anar", crate::Lang::Ca), "bˈaʤ ənˈa");
    }
}
