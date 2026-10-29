//! Czech grapheme-to-phoneme rules, espeak-style notation.
//!
//! Czech spelling is nearly phonemic: fixed initial stress (`ˈ` before the
//! first nucleus, `ˌ` on odd non-final syllables from the third on, the way
//! espeak-ng writes it), vowel length from diacritics, the palatal series
//! `ɟ c ɲ` from d/t/n before i/í/ě, `ě` after labials, final devoicing and
//! regressive voicing assimilation, syllabic r/l (/m). The sentence pass
//! merges monosyllabic prepositions with the following word (one stress unit,
//! as espeak does) and voices word-final obstruents before a voiced one.
//!
//! Notation follows espeak's `cs` voice: `h` for the voiced glottal
//! fricative (Wiktionary writes ɦ), `x` for ch, `r̝`/`r̝̥` for ř (voiceless mark U+0325; espeak writes U+030A, which is outside the BlueTTS vocabulary), `oʊ aʊ eʊ`
//! for the diphthongs, `eɪ` for ej, `ts tʃ` for c č (folded to ligatures at
//! the end by `fold.rs`), `r̩ l̩ m̩` for syllabic consonants.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "celá";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "aː", "beː", "tseː", "deː", "eː", "ef", "ɡeː", "haː", "iː", "jeː", "kaː", "el", "em", "en", "oː", "peː", "kveː",
    "er", "es", "teː", "uː", "veː", "dvojiteːveː", "iks", "ipsilon", "zet",
]);

/// Clitics and prepositions espeak reads without a stress mark.
const UNSTRESSED: &[&str] = &[
    "se", "si", "na", "je", "že", "do", "jsem", "pro", "za", "po", "by", "od", "jsou", "mi", "jsme", "ze", "při",
    "před", "bych", "jste", "ti", "nad", "pod", "jsi", "bys", "dle", "ke", "ku", "ve", "ob", "u", "o", "v", "k",
    "s", "z", "bez", "skrz", "mu", "ho", "tě", "mě", "nám", "vám", "jim", "jej", "ji", "jí", "to", "a", "i",
];
/// Function words that only get a secondary stress.
const SECONDARY: &[&str] = &[
    "nebo", "bude", "bylo", "byla", "proti", "budou", "byly", "budu", "byli", "budeme", "budete", "bychom",
    "byste", "budeš", "ode", "skrze", "nade", "pode", "beze", "přede",
];
/// Monosyllabic prepositions that take the stress of the following word.
const PREPS: &[&str] = &[
    "na", "do", "pro", "za", "po", "od", "ze", "při", "před", "nad", "pod", "ke", "ku", "ve", "u", "o", "v", "k",
    "s", "z", "bez", "přes", "skrz", "ob", "ode", "nade", "pode", "beze", "přede", "mezi", "dle", "kol",
];

/// Foreign stems in which d/t/n + i keep the hard consonant (dɪ tɪ nɪ).
/// A stem may occur anywhere in the word; it must cover the consonant.
const HARD: &[&str] = &[
    "minist", "mini", "uni", "komuni", "imuni", "techni", "mechani", "elektroni", "kroni", "botani", "harmoni",
    "politi", "prakti", "kriti", "statisti", "matemati", "automati", "systemati", "dramati", "romanti",
    "gymnasti", "atleti", "geneti", "kosmeti", "esteti", "takti", "erotik", "gramati", "akusti", "analyti",
    "logisti", "turisti", "informati", "charakteristi", "problemati", "temati", "klimati", "pragmati",
    "diplomati", "mysti", "plasti", "elasti", "fantasti", "skepti", "identi", "autenti", "apati",
    "panik", "klini", "organi", "moderniz", "defini", "fini", "veroni", "moni", "domini", "nikol", "daniel", "inici", "haiti", "etiop", "investi", "institu", "justi",
    "petic", "prostitu", "konstitu", "substitu", "restitu", "titu", "vanil", "kogni", "etic", "prestiž", "dimenz",
    "koordin", "kondic", "teoreti", "teoretic", "titul", "tip", "tik", "semit",
    "nikot", "nikar", "geni", "senil", "infantil", "subtil", "projektil", "ventil", "artikl", "stipendi",
    "kardi", "radik", "radio", "rádio", "radiá", "kandid", "individ", "idio", "ident", "edito", "editac", "edic",
    "kredit", "audi", "dilem", "diář", "stadi", "studi", "medi", "bandit", "entit", "kontinu", "kontinent",
    "disk", "dispo", "distr", "disci", "diskv", "diskut", "diskus", "disident", "dislok", "disp", "distan",
    "disfunk", "disharm", "disput", "marketin", "flotil", "kompatibil", "preventiv", "inventiv",
    "certifik", "anim", "nig", "koloni", "vernis", "ánist", "unist", "onist", "optim", "intim", "ultim",
    "legitim", "centim", "sentim", "estim", "viktim", "tipov", "antib", "antik", "antis", "antif", "antig",
    "antip", "antit", "antir", "antim", "antil", "antiv", "antid", "antih", "antio", "antia", "antie",
    "titan", "parti", "tibet", "tid", "argentin", "palestin", "kristin",
    "kristi", "rutin", "platin", "pervitin", "latin", "valentin", "tinkt", "destin", "satin",
    "putin", "tenis", "penis", "denis", "humanit", "manipul", "etik", "estet", "diplom", "tradi",
    "festiv", "tunis", "dinosaur", "dinár", "nikotin", "arktid", "milion",
    "tiket", "tipov", "kretin", "retin", "benedikt", "judit", "diesel", "díz", "dividend", "divizn", "diviz",
    "nitrát", "nitrog", "kyanid", "oxid", "hybrid", "fluid", "liquid", "disney", "indián", "indi", "sodík",
    "kanadi", "hindi", "diet", "multi", "inti", "senti", "lenti", "konti", "magneti",
    "chroni", "elektri", "cyni", "tyrani", "sympati", "apati", "empati", "telepati", "homeopati",
    "demokrati", "aristokrati", "byrokrati", "teokrati", "dynasti", "amnesti", "bestie", "tiger", "tiara",
];
/// Native words that one of the stems or generic rules above would get wrong.
const HARD_SOFT: &[&str] = &[
    "tito", "tisk", "tich", "štik", "štic", "protiv", "unikn", "unikat", "unikl", "mánie", "edisk", "odisk", "vtip", "tipl", "tipn", "protip",
    "nikam", "nikd", "nikoli", "nikom", "nikoh", "omani", "románi", "slovani", "germáni", "hostin", "cestin",
    "křtin", "štin", "patin", "dětin", "četin", "kostin", "ostin", "otin", "vtip", "stimul", "ctim", "tipl",
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Vowel,
    Son,
    /// obstruent: voiced?, weak (v, ř: assimilates but does not spread)?
    Obs(bool, bool),
}

#[derive(Clone, Debug)]
struct Seg {
    ph: &'static str,
    kind: Kind,
}

fn seg(ph: &'static str) -> Seg {
    Seg { ph, kind: seg_kind(ph) }
}

fn voiced_of(ph: &str) -> &'static str {
    match ph {
        "p" => "b",
        "t" => "d",
        "c" => "ɟ",
        "k" => "ɡ",
        "f" => "v",
        "s" => "z",
        "ʃ" => "ʒ",
        "x" => "h",
        "ts" => "dz",
        "tʃ" => "dʒ",
        "r̝̥" => "r̝",
        "b" => "b",
        "d" => "d",
        "ɟ" => "ɟ",
        "ɡ" => "ɡ",
        "v" => "v",
        "z" => "z",
        "ʒ" => "ʒ",
        "h" => "h",
        "dz" => "dz",
        "dʒ" => "dʒ",
        "r̝" => "r̝",
        _ => "",
    }
}
fn voiceless_of(ph: &str) -> &'static str {
    match ph {
        "b" => "p",
        "d" => "t",
        "ɟ" => "c",
        "ɡ" => "k",
        "v" => "f",
        "z" => "s",
        "ʒ" => "ʃ",
        "h" => "x",
        "dz" => "ts",
        "dʒ" => "tʃ",
        "r̝" => "r̝̥",
        "p" => "p",
        "t" => "t",
        "c" => "c",
        "k" => "k",
        "f" => "f",
        "s" => "s",
        "ʃ" => "ʃ",
        "x" => "x",
        "ts" => "ts",
        "tʃ" => "tʃ",
        "r̝̥" => "r̝̥",
        _ => "",
    }
}

fn set_voicing(s: &mut Seg, voiced: bool) {
    let p = if voiced { voiced_of(s.ph) } else { voiceless_of(s.ph) };
    if !p.is_empty() {
        s.ph = p;
        if let Kind::Obs(_, weak) = s.kind {
            s.kind = Kind::Obs(voiced, weak);
        }
    }
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ů' | 'ý' | 'ě' | 'ä' | 'ö' | 'ü')
}

fn vowel_ph(c: char) -> &'static str {
    match c {
        'a' | 'ä' => "a",
        'á' => "aː",
        'e' | 'ě' | 'ö' => "e",
        'é' => "eː",
        'i' | 'y' => "i",
        'í' | 'ý' => "iː",
        'o' => "o",
        'ó' => "oː",
        'u' | 'ü' => "u",
        'ú' | 'ů' => "uː",
        _ => "",
    }
}

/// Does one of `stems` occur in `c` covering index `i`?
fn covered(c: &[char], i: usize, stems: &[&str]) -> bool {
    for st in stems {
        let sc: Vec<char> = st.chars().collect();
        let m = sc.len();
        if m > c.len() {
            continue;
        }
        let lo = (i + 1).saturating_sub(m);
        for start in lo..=i.min(c.len() - m) {
            if c[start..start + m] == sc[..] {
                return true;
            }
        }
    }
    false
}

/// Is a d/t/n at letter index `i` followed by i/í hard (a foreign stem)?
fn hard_at(c: &[char], i: usize) -> bool {
    if covered(c, i, HARD_SOFT) {
        return false;
    }
    if covered(c, i, HARD) {
        return true;
    }
    let at = |k: usize| c.get(k).copied().unwrap_or('#');
    let (n1, n2) = (at(i + 1), at(i + 2));
    let prev = if i > 0 { c[i - 1] } else { '#' };
    if n1 != 'i' {
        return false;
    }
    match c[i] {
        't' => {
            // -tiv- (aktivní, motiv); native lstivý, chtivý keep the palatal
            if n2 == 'v' && (is_vowel_letter(prev) || matches!(prev, 'k' | 'p' | 'n' | 'r')) {
                return true;
            }
            // -tik- / -tick- / -tiz- (politika, praktický, privatizace)
            if matches!(n2, 'k' | 'z') && i > 0 && prev != 'š' {
                return true;
            }
            // -tick- (gotický, optický)
            if n2 == 'c' && at(i + 3) == 'k' && i > 0 && prev != 'š' {
                return true;
            }
        }
        'd' => {
            if matches!(n2, 'f' | 'g' | 'k' | 'p' | 'r' | 'a' | 'o' | 'u' | 'e') {
                return true;
            }
        }
        'n' => {
            if matches!(n2, 'a' | 'o' | 'u') {
                return true;
            }
        }
        _ => {}
    }
    // foreign nouns in -ie (Británie, komedie, sympatie) and their inflections
    let tail: String = c[i + 2..].iter().collect();
    matches!(tail.as_str(), "e" | "í" | "i" | "em" | "emi" | "ím" | "ích")
}

/// Frequent loanwords whose spelling is not Czech: respelled in Czech orthography.
const RESPELL: &[(&str, &str)] = &[
    ("design", "dyzajn"), ("designer", "dyzajnér"), ("software", "softvér"), ("hardware", "hardvér"),
    ("party", "párty"), ("gay", "gej"), ("rock", "rok"), ("business", "byznys"), ("asie", "ázije"),
    ("green", "grín"), ("sydney", "sydny"), ("back", "bek"), ("interview", "intervjů"), ("rally", "rely"),
    ("rallye", "rely"), ("notebook", "noutbuk"), ("whisky", "visky"), ("chile", "čile"), ("alexander", "alexandr"),
    ("pizza", "pica"), ("chicago", "čikágo"), ("poker", "pokr"), ("müller", "myler"), ("vietnam", "vjetnam"),
    ("liverpool", "livrpúl"), ("laser", "lejzr"), ("minus", "mínus"), ("amsterdam", "amstrdam"),
    ("thriller", "triler"), ("diesel", "dízl"), ("marseille", "marsej"), ("gin", "džin"), ("provence", "prováns"),
    ("caesar", "cézar"), ("puzzle", "pazl"), ("hamburger", "hamburgr"), ("steak", "stejk"), ("judo", "džudo"),
    ("gray", "grej"), ("grey", "grej"), ("beck", "bek"), ("hacker", "hekr"),
    ("newton", "ňútn"), ("baseball", "bejzbol"), ("jazz", "džez"), ("john", "džon"), ("james", "džejms"),
    ("george", "džordž"), ("show", "šou"), ("facebook", "fejsbuk"), ("youtube", "jutúb"),
    ("google", "gúgl"), ("twitter", "tvitr"), ("windows", "vindous"), ("iphone", "ajfoun"), ("online", "onlajn"),
    ("email", "emajl"), ("web", "veb"), ("team", "tým"), ("manager", "menedžr"), ("leader", "lídr"),
    ("shop", "šop"), ("cool", "kúl"), ("smartphone", "smartfoun"), ("image", "imidž"),
    ("meeting", "mítynk"), ("manchester", "menčestr"), ("new", "ňjů"), ("york", "jork"),
    ("williams", "viljems"), ("washington", "vašinkton"), ("hockey", "hokej"), ("jeans", "džíny"),
    ("playstation", "plejstejšn"), ("mainstream", "mejnstrým"), ("fair", "fér"), ("know", "nou"),
    ("how", "hau"), ("workshop", "vorkšop"), ("second", "sekond"), ("hand", "hend"), ("cash", "keš"),
    ("outlet", "autlet"), ("boy", "boj"), ("girl", "grl"), ("boys", "bojs"), ("girls", "grls"),
    ("baby", "bejby"), ("happy", "hepy"), ("single", "singl"), ("live", "lajv"),
    ("street", "strít"), ("time", "tajm"), ("free", "frí"), ("game", "gejm"), ("games", "gejms"),
    ("player", "plejer"), ("file", "fajl"), ("mail", "mejl"), ("club", "klab"), ("dance", "dens"),
    ("house", "haus"), ("light", "lajt"), ("night", "najt"), ("white", "vajt"), ("black", "blek"),
    ("blue", "blú"), ("joe", "džou"), ("jack", "džek"), ("charlie", "čárlí"),
    ("filosof", "filozof"), ("filosofie", "filozofije"), ("filosofický", "filozofický"), ("josef", "jozef"),
    ("josefa", "jozefa"), ("josefem", "jozefem"), ("josefu", "jozefu"), ("josefov", "jozefov"),
    ("indonésie", "indonézije"), ("malajsie", "malajzije"), ("siemens", "símens"), ("else", "elze"),
    ("virginie", "virdžínie"), ("stadium", "stádium"), ("textilie", "textýlije"), ("bůhví", "bůchví"),
    ("afghánistán", "afgánistán"), ("afghánský", "afgánský"), ("afghánistánu", "afgánistánu"),
    ("bordeaux", "bordó"), ("peugeot", "pežo"), ("renault", "renó"), ("citroën", "citroen"),
    ("bruxelles", "brysel"), ("picasso", "pikaso"), ("chipsy", "čipsy"), ("remake", "rimejk"),
    ("rhodos", "rodos"), ("müsli", "mysli"), ("persie", "perzie"), ("pizzerie", "picerie"), ("uruguay", "urugvaj"),
    ("schneider", "šnajder"), ("laserový", "lejzrový"), ("laserem", "lejzrem"), ("laseru", "lejzru"), ("hollywood", "holyvúd"), ("coca", "koka"), ("cola", "kola"),
];

/// Abbreviations read as words.
const ABBREV: &[(&str, &str)] = &[
    ("km", "kilometrů"), ("cm", "centimetrů"), ("mm", "milimetrů"), ("kg", "kilogramů"), ("tzv", "takzvaný"),
    ("tj", "to jest"), ("tzn", "to znamená"), ("mgr", "magistr"), ("dr", "doktor"), ("sv", "svatý"),
    ("např", "například"), ("atd", "a tak dále"), ("apod", "a podobně"), ("mld", "miliard"), ("kč", "korun"),
    ("ing", "inženýr"), ("ks", "kusů"), ("str", "strana"), ("ml", "mililitrů"), ("resp", "respektive"),
    ("tis", "tisíc"), ("popř", "popřípadě"), ("př", "před"), ("č", "číslo"), ("čs", "československý"),
    ("phdr", "pé há dé er"), ("rndr", "er en dé er"), ("judr", "jů dé er"), ("mudr", "em ú dé er"),
    ("st", "svatý"), ("ch", "chá"), ("hod", "hodin"), ("min", "minut"),
    ("sec", "sekund"), ("mil", "milionů"), ("m", "metrů"), ("g", "gramů"), ("l", "litrů"), ("ha", "hektarů"),
];

/// Vowel-less Czech words (syllabic r/l/m), as opposed to initialisms to spell.
const VOWELLESS: &[&str] = &[
    "hm", "hmm", "mhm", "pst", "brr", "vrr", "krk", "vlk", "smrt", "prst", "trh", "vrch", "vln", "srst", "čtvrt",
    "čtvrť", "drž", "skrz", "prd", "krb", "hr", "blb", "brk", "chrt", "drn", "hrb", "hrst", "krč", "mlč", "mls",
    "mrz", "plch", "pln", "plň", "plť", "prš", "srp", "strč", "trn", "trs", "vrt", "vrh", "zrn", "slz", "smrk",
    "sprch", "strh", "tvrz", "vrb", "vrč", "mlh", "mrv", "krm", "krt", "drb", "hlt", "plk", "plž", "trp", "trž",
    "brzd", "mrkl", "prchl", "trhl", "vrhl", "zdrž", "sprchl", "svrhl", "strhl", "zhltl", "cvrk", "frk", "chrp",
    "skvrn", "zkrm", "brň", "frč", "drť", "mrť", "prť", "brť", "crč", "chrl", "splň", "srn", "strn", "strp",
    "trč", "trň", "vrš", "vrť", "tvrď", "strď", "smrď", "prď", "brzď", "chrň", "hrň", "plsť", "prsť", "trkl",
    "krkl", "srkl", "mrskl", "prskl", "prdl", "mrzl", "zblbl", "blbl", "tvrdl", "drhl", "zdrhl", "vtrhl", "vlhl",
    "vlh", "vlhk", "drnkl", "k", "s", "v", "z", "zmrd", "smrd", "brnkl", "frnkl", "cvrnk", "krch", "mrch", "srch", "srh", "prh", "brh", "brd", "brt",
    "brv", "mrd", "prs", "prč", "drs", "trš", "chrz", "plst", "klk", "mlž", "smrž", "obr", "ptr",
];

/// Czech letter names (for initialisms the generic speller does not catch, e.g. ČR, ČSSD).
fn letter_name(c: char) -> &'static str {
    match c {
        'a' | 'á' => "aː",
        'b' => "beː",
        'c' => "tseː",
        'd' => "deː",
        'e' | 'é' => "eː",
        'f' => "ef",
        'g' => "ɡeː",
        'h' => "haː",
        'i' | 'í' | 'ý' => "iː",
        'j' => "jeː",
        'k' => "kaː",
        'l' => "el",
        'm' => "em",
        'n' => "en",
        'o' | 'ó' => "oː",
        'p' => "peː",
        'q' => "kveː",
        'r' => "er",
        's' => "es",
        't' => "teː",
        'u' | 'ú' | 'ů' => "uː",
        'v' => "veː",
        'w' => "dvojiteːveː",
        'x' => "iks",
        'y' => "ipsilon",
        'z' => "zet",
        'č' => "tʃeː",
        'š' => "eʃ",
        'ž' => "ʒet",
        'ř' => "er̝̥",
        'ď' => "ɟeː",
        'ť' => "ceː",
        'ň' => "eɲ",
        'ě' => "e",
        _ => "",
    }
}

/// Spell letter by letter: secondary stress on each letter, primary on the last.
fn spell(c: &[char]) -> String {
    let mut out = String::new();
    for (k, &ch) in c.iter().enumerate() {
        let name = letter_name(ch);
        if name.is_empty() {
            continue;
        }
        let mark = if k + 1 == c.len() { "ˈ" } else { "ˌ" };
        let pos = name.find(|x: char| "aeiou".contains(x)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    out
}

/// Phonemize one lower-case word (no sentence context).
pub fn word(w: &str) -> String {
    let lw = w.to_lowercase();
    if lw.contains('-') {
        let parts: Vec<String> = lw.split('-').filter(|p| !p.is_empty()).map(word).filter(|p| !p.is_empty()).collect();
        return parts.join("");
    }
    let c: Vec<char> = lw.chars().filter(|c| c.is_alphabetic()).collect();
    if c.is_empty() {
        return String::new();
    }
    if let Some((_, r)) = RESPELL.iter().find(|(k, _)| *k == lw) {
        return word(r);
    }
    if !c.iter().any(|&x| is_vowel_letter(x)) {
        if let Some((_, r)) = ABBREV.iter().find(|(k, _)| *k == lw) {
            return r.split(' ').map(word).collect::<Vec<_>>().join(" ");
        }
        let inner_liquid = c.len() >= 3 && c[1..c.len() - 1].iter().any(|&x| matches!(x, 'r' | 'l'));
        if !VOWELLESS.contains(&lw.as_str()) && !inner_liquid && c.len() <= 5 {
            return spell(&c);
        }
    }
    let st = if UNSTRESSED.contains(&lw.as_str()) {
        Stress::None
    } else if SECONDARY.contains(&lw.as_str()) {
        Stress::Secondary
    } else {
        Stress::Primary
    };
    let segs = segments(&c);
    render(&segs, st)
}

#[derive(Clone, Copy, PartialEq)]
enum Stress {
    Primary,
    Secondary,
    None,
}

/// Letters -> phone segments, with assimilation applied.
fn segments(c: &[char]) -> Vec<Seg> {
    let n = c.len();
    let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { c[i as usize] } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 2);
    let mut i = 0usize;
    // hiatus rules for prefix + u (ne-u, na-u, za-u, po-u)
    let starts = |p: &str| p.chars().zip(c.iter()).all(|(a, &b)| a == b) && p.chars().count() <= n;
    let hiatus_neu = starts("neu") && !(starts("neutr") || starts("neuro") || starts("neuron"));
    let hiatus_au = starts("nau") || starts("zau");
    let hiatus_pou = starts("použ") || starts("pouč") || starts("pouk") || starts("douč");
    while i < n {
        let ch = c[i];
        let nx = at(i as isize + 1);
        let nx2 = at(i as isize + 2);
        let pv = at(i as isize - 1);
        if is_vowel_letter(ch) {
            // diphthongs
            if ch == 'o' && nx == 'u' && !(i == 1 && hiatus_pou) {
                segs.push(seg("oʊ"));
                i += 2;
                continue;
            }
            if ch == 'a' && nx == 'u' && !(i == 1 && hiatus_au) {
                segs.push(seg("aʊ"));
                i += 2;
                continue;
            }
            if ch == 'e' && nx == 'u' && !(i == 1 && hiatus_neu) {
                segs.push(seg("eʊ"));
                i += 2;
                continue;
            }
            let v = vowel_ph(ch);
            segs.push(seg(v));
            // ej -> eɪ (espeak), i + vowel -> ij + vowel
            if ch == 'e' && nx == 'j' && !is_vowel_letter(nx2) {
                segs.pop();
                segs.push(seg("eɪ"));
                i += 2;
                continue;
            }
            if matches!(ch, 'i' | 'í' | 'y' | 'ý') && is_vowel_letter(nx) {
                segs.push(seg("j"));
            }
            i += 1;
            continue;
        }
        let soft_next = matches!(nx, 'i' | 'í');
        // doubled consonant letters are one phone (Anna, činnost, měkký), except when the
        // second one palatalizes (oddělení: d + ɟ)
        if ch == nx && !is_vowel_letter(ch) && !(matches!(ch, 'd' | 't' | 'n') && matches!(nx2, 'ě' | 'i' | 'í')) {
            i += 1;
            continue;
        }
        match ch {
            'b' | 'p' | 'v' | 'f' if nx == 'ě' => {
                segs.push(seg(match ch {
                    'b' => "b",
                    'p' => "p",
                    'v' => "v",
                    _ => "f",
                }));
                segs.push(seg("j"));
                segs.push(seg("e"));
                i += 2;
                continue;
            }
            'm' if nx == 'ě' => {
                segs.push(seg("m"));
                segs.push(seg("ɲ"));
                segs.push(seg("e"));
                i += 2;
                continue;
            }
            'd' | 't' | 'n' if nx == 'ě' => {
                segs.push(seg(match ch {
                    'd' => "ɟ",
                    't' => "c",
                    _ => "ɲ",
                }));
                segs.push(seg("e"));
                i += 2;
                continue;
            }
            'd' | 't' | 'n' if soft_next && !hard_at(c, i) => {
                segs.push(seg(match ch {
                    'd' => "ɟ",
                    't' => "c",
                    _ => "ɲ",
                }));
            }
            'd' => {
                if nx == 'z' {
                    segs.push(seg("dz"));
                    i += 1;
                } else if nx == 'ž' {
                    segs.push(seg("dʒ"));
                    i += 1;
                } else if (nx == 'c' && nx2 != 'h') || nx == 'č' {
                    // dc, dč -> one affricate (dcera, srdce, rozhodčí)
                } else {
                    segs.push(seg("d"));
                }
            }
            't' => {
                // tc: one affricate (otce, letci), as Wiktionary transcribes it
                if !(nx == 'c' && nx2 != 'h') {
                    segs.push(seg("t"));
                }
            }
            'n' if nx == 'n' && matches!(nx2, 'ě' | 'i' | 'í') => {
                // nn + ě/i: one ɲ (denně, ranní)
            }
            'n' => {
                if matches!(nx, 'k' | 'g') {
                    segs.push(seg("ŋ"));
                } else {
                    segs.push(seg("n"));
                }
            }
            'ď' => segs.push(seg("ɟ")),
            'ť' => segs.push(seg("c")),
            'ň' => segs.push(seg("ɲ")),
            'c' => {
                if nx == 'h' {
                    segs.push(seg("x"));
                    i += 1;
                } else {
                    segs.push(seg("ts"));
                }
            }
            'č' => segs.push(seg("tʃ")),
            'š' => segs.push(seg("ʃ")),
            'ž' => segs.push(seg("ʒ")),
            'ř' => segs.push(seg("r̝")),
            'h' => segs.push(seg("h")),
            'j' => segs.push(seg("j")),
            'g' => segs.push(seg("ɡ")),
            'k' => segs.push(seg("k")),
            'b' => segs.push(seg("b")),
            'p' => segs.push(seg("p")),
            'v' => segs.push(seg("v")),
            'f' => segs.push(seg("f")),
            's' => {
                // -ismus / -asmus and inflections: organizmus, sarkazmu
                let ism = matches!(pv, 'i' | 'a') && nx == 'm' && nx2 == 'u' && at(i as isize + 3) == 's' && i + 4 == n;
                segs.push(seg(if ism { "z" } else { "s" }));
            }
            'z' => segs.push(seg("z")),
            'm' => segs.push(seg("m")),
            'l' => segs.push(seg("l")),
            'r' => segs.push(seg("r")),
            'w' => segs.push(seg("v")),
            'q' => {
                segs.push(seg("k"));
                segs.push(seg("v"));
                if nx == 'u' {
                    i += 1;
                }
            }
            'x' => {
                // ex + vowel -> eɡz (exil, existence); otherwise ks
                let ne = i == 3 && c[0] == 'n' && c[1] == 'e';
                if (i == 1 || ne) && pv == 'e' && is_vowel_letter(nx) {
                    segs.push(seg("ɡ"));
                    segs.push(seg("z"));
                } else {
                    segs.push(seg("k"));
                    segs.push(seg("s"));
                }
            }
            'ł' => segs.push(seg("l")),
            'ß' => segs.push(seg("s")),
            'ç' => segs.push(seg("s")),
            _ => {}
        }
        i += 1;
    }
    syllabics(&mut segs);
    assimilate(&mut segs);
    segs
}

/// Mark syllabic r/l (and final m) between consonants.
fn syllabics(segs: &mut [Seg]) {
    let n = segs.len();
    for i in 0..n {
        let ph = segs[i].ph;
        if !matches!(ph, "r" | "l" | "m") {
            continue;
        }
        let prev_cons = i > 0 && segs[i - 1].kind != Kind::Vowel && !matches!(segs[i - 1].ph, "r" | "l" | "j" | "r̩" | "l̩");
        let next_cons = i + 1 == n || segs[i + 1].kind != Kind::Vowel;
        if prev_cons && next_cons {
            let syl = match ph {
                "r" => "r̩",
                "l" => "l̩",
                _ => {
                    if i + 1 == n {
                        "m̩"
                    } else {
                        continue;
                    }
                }
            };
            segs[i].ph = syl;
        }
    }
}

/// Final devoicing and regressive voicing assimilation; ř also takes the
/// voicing of a preceding obstruent.
fn assimilate(segs: &mut [Seg]) {
    let n = segs.len();
    let mut target: Option<bool> = Some(false);
    for i in (0..n).rev() {
        match segs[i].kind {
            Kind::Obs(v, weak) => {
                if let Some(t) = target {
                    set_voicing(&mut segs[i], t);
                }
                if weak {
                    // v/ř keep the target of what follows for the next obstruent to the left? No:
                    // "svatba" keeps s, so a weak obstruent stops the spread.
                    target = None;
                } else {
                    let now = if let Kind::Obs(v2, _) = segs[i].kind { v2 } else { v };
                    target = Some(now);
                }
            }
            _ => target = None,
        }
    }
    // ř after a voiceless obstruent (tři, přes)
    for i in 1..n {
        if segs[i].ph == "r̝" {
            if let Kind::Obs(false, _) = segs[i - 1].kind {
                set_voicing(&mut segs[i], false);
            }
        }
    }
}

/// Render segments with espeak-style stress marks.
fn render(segs: &[Seg], stress: Stress) -> String {
    let nuclei: Vec<usize> =
        segs.iter().enumerate().filter(|(_, s)| s.kind == Kind::Vowel || matches!(s.ph, "r̩" | "l̩" | "m̩")).map(|(i, _)| i).collect();
    let mut out = String::with_capacity(segs.len() * 2);
    let last = nuclei.len();
    for (i, s) in segs.iter().enumerate() {
        if let Some(k) = nuclei.iter().position(|&x| x == i) {
            let mark = match stress {
                Stress::None => "",
                Stress::Secondary => if k == 0 { "ˌ" } else { "" },
                Stress::Primary => {
                    if k == 0 {
                        "ˈ"
                    } else if k >= 2 && k % 2 == 0 && k + 1 < last {
                        "ˌ"
                    } else {
                        ""
                    }
                }
            };
            out.push_str(mark);
        }
        out.push_str(s.ph);
    }
    out
}

fn first_phone(s: &str) -> Option<&str> {
    let t = s.trim_start_matches(['ˈ', 'ˌ', '\u{0}']);
    let mut it = t.chars();
    let c = it.next()?;
    let mut end = c.len_utf8();
    // affricates and diacritics
    let rest = &t[end..];
    if (c == 't' && (rest.starts_with('s') || rest.starts_with('ʃ'))) || (c == 'd' && (rest.starts_with('z') || rest.starts_with('ʒ'))) {
        end += rest.chars().next().unwrap().len_utf8();
    }
    for d in t[end..].chars() {
        if crate::text::is_mark(d) {
            end += d.len_utf8();
        } else {
            break;
        }
    }
    Some(&t[..end])
}

fn obstruent_voicing(ph: &str) -> Option<(bool, bool)> {
    match seg_kind(ph) {
        Kind::Obs(v, w) => Some((v, w)),
        _ => None,
    }
}

fn seg_kind(ph: &str) -> Kind {
    match ph {
        "a" | "aː" | "e" | "eː" | "i" | "iː" | "o" | "oː" | "u" | "uː" | "oʊ" | "aʊ" | "eʊ" | "eɪ" => Kind::Vowel,
        "m" | "n" | "ɲ" | "ŋ" | "l" | "r" | "j" | "r̩" | "l̩" | "m̩" => Kind::Son,
        "v" | "r̝" => Kind::Obs(true, true),
        "b" | "d" | "ɟ" | "ɡ" | "z" | "ʒ" | "h" | "dz" | "dʒ" => Kind::Obs(true, false),
        _ => Kind::Obs(false, false),
    }
}

/// Split a phoneme string into phones (affricates and base+diacritics kept together).
fn phones(s: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for ch in s.chars() {
        if crate::text::is_mark(ch) || ch == 'ː' {
            if let Some(l) = v.last_mut() {
                l.push(ch);
                continue;
            }
        }
        if matches!(ch, 's' | 'ʃ' | 'z' | 'ʒ') {
            if let Some(l) = v.last() {
                if (l == "t" && matches!(ch, 's' | 'ʃ')) || (l == "d" && matches!(ch, 'z' | 'ʒ')) {
                    v.last_mut().unwrap().push(ch);
                    continue;
                }
            }
        }
        if matches!(ch, 'ʊ' | 'ɪ') {
            if let Some(l) = v.last_mut() {
                l.push(ch);
                continue;
            }
        }
        v.push(ch.to_string());
    }
    v
}

/// Voice the final obstruent cluster of `s` (word-final before a voiced obstruent).
fn voice_final(s: &str) -> String {
    let mut ph = phones(s);
    let mut i = ph.len();
    while i > 0 {
        let p = ph[i - 1].trim_matches(['ˈ', 'ˌ']);
        match seg_kind(p) {
            Kind::Obs(_, weak) => {
                let v = voiced_of(p);
                if !v.is_empty() {
                    ph[i - 1] = v.to_string();
                }
                if weak {
                    break;
                }
            }
            _ => break,
        }
        i -= 1;
    }
    ph.concat()
}

/// Sentence pass: a monosyllabic preposition and the following word form one
/// stress unit (espeak writes them as one token); word-final obstruents are
/// voiced before a word starting with a voiced obstruent.
pub fn join(outs: &mut [Out]) {
    let n = outs.len();
    // 1. prepositions
    for k in 0..n.saturating_sub(1) {
        let (a, b) = outs.split_at_mut(k + 1);
        let prev = &mut a[k];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let src = prev.src.as_str();
        let is_prep = PREPS.contains(&src)
            || (src == "se" && cur.src.starts_with(['s', 'z', 'š', 'ž']) || src == "se" && cur.src == "mnou");
        if !is_prep || PREPS.contains(&cur.src.as_str()) && cur.src.chars().count() <= 2 {
            continue;
        }
        let p = prev.ph.replace(['ˈ', 'ˌ'], "");
        let merged = if matches!(src, "v" | "k" | "s" | "z") {
            let rest = cur.ph.clone();
            // non-syllabic: assimilate to the first obstruent of the next word
            let mut cons = p.clone();
            if let Some(f) = first_phone(&rest) {
                if let Some((voiced, weak)) = obstruent_voicing(f) {
                    if !weak {
                        cons = if voiced { voiced_of(&p).to_string() } else { voiceless_of(&p).to_string() };
                    }
                }
            }
            format!("{cons}{rest}")
        } else {
            let rest = cur.ph.replace('ˈ', "");
            // stress on the preposition's vowel
            let mut s = String::new();
            let mut done = false;
            for ph in phones(&p) {
                if !done && seg_kind(&ph) == Kind::Vowel {
                    s.push('ˈ');
                    done = true;
                }
                s.push_str(&ph);
            }
            // final obstruent of the preposition before the word (bez -> bes vody, od -> ot)
            let s = if let Some(f) = first_phone(&rest) {
                match obstruent_voicing(f) {
                    Some((true, false)) => voice_final(&s),
                    _ => s,
                }
            } else {
                s
            };
            format!("{s}{rest}")
        };
        cur.ph = merged;
        prev.ph.clear();
    }
    // 2. cross-word voicing assimilation
    for k in 0..n.saturating_sub(1) {
        let (a, b) = outs.split_at_mut(k + 1);
        let prev = &mut a[k];
        let cur = &b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        if let Some(f) = first_phone(&cur.ph) {
            if let Some((true, false)) = obstruent_voicing(f) {
                prev.ph = voice_final(&prev.ph);
            }
        }
    }
}

const ONES: [&str; 20] = [
    "nula", "jedna", "dva", "tři", "čtyři", "pět", "šest", "sedm", "osm", "devět", "deset", "jedenáct", "dvanáct",
    "třináct", "čtrnáct", "patnáct", "šestnáct", "sedmnáct", "osmnáct", "devatenáct",
];
const TENS: [&str; 10] = ["", "", "dvacet", "třicet", "čtyřicet", "padesát", "šedesát", "sedmdesát", "osmdesát", "devadesát"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = n / 100;
    let r = n % 100;
    match h {
        0 => {}
        1 => out.push("sto".into()),
        2 => {
            out.push("dvě".into());
            out.push("stě".into());
        }
        3 | 4 => {
            out.push(ONES[h as usize].into());
            out.push("sta".into());
        }
        _ => {
            out.push(ONES[h as usize].into());
            out.push("set".into());
        }
    }
    if r == 0 {
        return;
    }
    if r < 20 {
        out.push(ONES[r as usize].into());
    } else {
        out.push(TENS[(r / 10) as usize].into());
        if r % 10 != 0 {
            out.push(ONES[(r % 10) as usize].into());
        }
    }
}

/// Czech cardinal number words (nominative; "dvě stě", "tři tisíce", "pět milionů").
pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n == 0 {
        out.push("nula".into());
        return out;
    }
    // (value, singular, 2-4 form, 5+ form, feminine?)
    let scales: [(u128, &str, &str, &str); 5] = [
        (1_000_000_000_000_000_000, "trilion", "triliony", "trilionů"),
        (1_000_000_000_000_000, "biliarda", "biliardy", "biliard"),
        (1_000_000_000_000, "bilion", "biliony", "bilionů"),
        (1_000_000_000, "miliarda", "miliardy", "miliard"),
        (1_000_000, "milion", "miliony", "milionů"),
    ];
    let mut rest = n;
    for (val, one, few, many) in scales {
        let q = rest / val;
        if q == 0 {
            continue;
        }
        rest %= val;
        let fem = one.ends_with('a');
        if q == 1 {
            out.push(if fem { "jedna".into() } else { "jeden".into() });
            out.push(one.into());
        } else {
            let mut qw = cardinal(q);
            if fem {
                if let Some(l) = qw.last_mut() {
                    if l == "dva" {
                        *l = "dvě".into();
                    }
                }
            }
            out.extend(qw);
            out.push(if (2..=4).contains(&(q % 10)) && !(12..=14).contains(&(q % 100)) { few } else { many }.into());
        }
    }
    let th = rest / 1000;
    rest %= 1000;
    if th > 0 {
        if th == 1 {
            out.push("tisíc".into());
        } else {
            out.extend(cardinal(th));
            out.push(if (2..=4).contains(&(th % 10)) && !(12..=14).contains(&(th % 100)) { "tisíce" } else { "tisíc" }.into());
        }
    }
    below_1000(rest, &mut out);
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "procent",
        '$' => "dolarů",
        '€' => "eur",
        '£' => "liber",
        '&' => "a",
        '+' => "plus",
        '@' => "zavináč",
        '°' => "stupňů",
        '=' => "rovná se",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};
    #[test]
    fn basics() {
        for (w, p) in [
            ("praha", "prˈaha"),
            ("děti", "ɟˈeci"),
            ("město", "mɲˈesto"),
            ("pět", "pjˈet"),
            ("věda", "vjˈeda"),
            ("tělo", "cˈelo"),
            ("něco", "ɲˈetso"),
            ("chléb", "xlˈeːp"),
            ("bůh", "bˈuːx"),
            ("tři", "tr̝̥ˈi"),
            ("řeka", "r̝ˈeka"),
            ("keř", "kˈer̝̥"),
            ("hřbet", "hr̝bˈet"),
            ("kdo", "ɡdˈo"),
            ("vstup", "fstˈup"),
            ("zpět", "spjˈet"),
            ("sbor", "zbˈor"),
            ("svatba", "svˈadba"),
            ("lev", "lˈef"),
            ("krk", "kˈr̩k"),
            ("vlk", "vˈl̩k"),
            ("nesl", "nˈesl̩"),
            ("sedm", "sˈedm̩"),
            ("čtyři", "tʃtˈir̝i"),
            ("stejně", "stˈeɪɲe"),
            ("nejsou", "nˈeɪsoʊ"),
            ("auto", "ˈaʊto"),
            ("naučit", "nˈautʃit"),
            ("použít", "pˈouʒiːt"),
            ("neuvěřitelně", "nˈeuvjˌer̝itˌelɲe"),
            ("politika", "pˈolitˌika"),
            ("dialog", "dˈijalok"),
            ("existuje", "ˈeɡzistˌuje"),
            ("srdce", "sˈr̩tse"),
            ("dcera", "tsˈera"),
            ("shoda", "zhˈoda"),
            ("otce", "ˈotse"),
            ("rozhodčí", "rˈozhotʃiː"),
            ("denně", "dˈeɲe"),
            ("organismus", "ˈorɡanˌizmus"),
            ("ředitel", "r̝ˈeɟitel"),
            ("tradice", "trˈaditse"),
            ("čssd", "tʃˌeːˌesˌesdˈeː"),
            ("km", "kˈilomˌetruː"),
            ("venku", "vˈeŋku"),
            ("společnosti", "spˈoletʃnˌosci"),
            ("nejpravděpodobnější", "nˈeɪpravɟˌepodˌobɲejʃiː"),
            ("se", "se"),
            ("nebo", "nˌebo"),
            ("tchán", "txˈaːn"),
            ("ďábel", "ɟˈaːbel"),
            ("loď", "lˈoc"),
            ("džus", "dʒˈus"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(21).join(" "), "dvacet jedna");
        assert_eq!(cardinal(200).join(" "), "dvě stě");
        assert_eq!(cardinal(2001).join(" "), "dva tisíce jedna");
        assert_eq!(cardinal(5000).join(" "), "pět tisíc");
        assert_eq!(cardinal(1_000_000).join(" "), "jeden milion");
        assert_eq!(cardinal(3_000_000_000).join(" "), "tři miliardy");
    }

    #[test]
    fn sentences() {
        let g = crate::G2p::new(None);
        assert_eq!(g.phonemize("v Praze", crate::Lang::Cs), "fprˈaze");
        assert_eq!(g.phonemize("na stole", crate::Lang::Cs), "nˈastole");
        assert_eq!(g.phonemize("k domu", crate::Lang::Cs), "ɡdˈomu");
        assert_eq!(g.phonemize("hrad byl", crate::Lang::Cs), "hrˈad bˈil");
        assert_eq!(g.phonemize("bez vody", crate::Lang::Cs), "bˈesvodi");
    }
}
