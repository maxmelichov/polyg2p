//! Hungarian grapheme-to-phoneme rules, espeak-style notation.
//!
//! Hungarian spelling is phonemic apart from the consonant sandhi, so this is
//! rules only: vowel length by accent, the digraphs/trigraphs (`sz zs cs dzs gy
//! ty ny ly dz`) and their doubled forms (`ssz ggy nny…`), long consonants,
//! regressive voicing assimilation, j-assimilation (`tj → cː`, `dj → ɟː`,
//! `lj → jj`, `nj → ɲɲ`), sibilant assimilation (`ts → tʃː`, `tsz → tsː`,
//! `zs+s → ʃʃ`), degemination next to another consonant, the `egy-` words with
//! a long `ɟː`, and fixed initial stress.
//!
//! Notation follows espeak-ng's `hu` voice: `a` is `ɑ`, `á` is `aː`, `e` is
//! `ɛ`, `é` is `eː`, `ö ő ü ű` are `ø øː y yː`; `gy ty ny ly` are `ɟ c ɲ j`;
//! `sz s zs cs c dz dzs` are `s ʃ ʒ tʃ ts dz dʒ`; long stops, affricates and
//! palatals carry `ː` (`tː ɟː tsː`), long fricatives and sonorants are written
//! double (`ss ll ɲɲ`); `ˈ` stands right before the first vowel (`nˈɛm`,
//! `brˈinɡɑ`, `ˈɑz`). espeak lowers ~a quarter of all words to `ˌ` from a
//! word list; we always write the primary mark.

use crate::Out;

pub const IMPLEMENTED: bool = true;
/// "3,5" is read "három egész öt tized"; the comma word is "egész".
pub const DECIMAL: &str = "egész";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "ɑ", "beː", "tseː", "deː", "ɛ", "ɛff", "ɡeː", "haː", "i", "jeː", "kaː", "ɛll", "ɛmm", "ɛnn", "o", "peː", "ku",
    "ɛrr", "ɛʃʃ", "teː", "u", "veː", "duplɑveː", "iks", "ipsilon", "zeː",
]);

/// Words whose final written `h` is silent (méh, cseh…); it is pronounced in
/// suffixed forms (méhek).
const SILENT_H: &[&str] = &["méh", "cseh", "juh", "düh", "pléh", "rüh", "céh", "oláh", "keh"];

/// `egy` + vowel-initial suffix: short `ɟ` (otherwise the stem has a long `ɟː`:
/// egyik, egyet, együtt, egyes).
const EGY_SHORT: &[&str] = &[
    "egyedül", "egyedi#", "egyed#", "egyetem", "egyen", "egyén", "egyel", "egyéb", "egyeb", "egyip", "egyél#", "egye#",
];

/// Prefixes and number words before `egy` that keep its long `ɟː` (tizenegy,
/// huszonegy, mindegyik, megegyezik, mintegy).
const EGY_PREFIX: &[&str] = &[
    "tizen", "huszon", "harminc", "negyven", "ötven", "hatvan", "hetven", "nyolcvan", "kilencven", "száz", "ezer",
    "mind", "meg", "bele", "össze", "ki", "mint",
];

/// Abbreviations read as full words.
const ABBREV: &[(&str, &str)] = &[
    ("db", "darab"), ("kb", "körülbelül"), ("pl", "például"), ("stb", "satöbbi"), ("ill", "illetve"),
    ("ún", "úgynevezett"), ("ui", "ugyanis"), ("ld", "lásd"), ("vö", "vesd össze"), ("vmi", "valami"),
    ("vki", "valaki"), ("vhol", "valahol"), ("vmit", "valamit"), ("vkit", "valakit"), ("km", "kilométer"),
    ("kg", "kilogramm"), ("dkg", "dekagramm"), ("cm", "centiméter"), ("mm", "milliméter"), ("dl", "deciliter"),
    ("ft", "forint"), ("dr", "doktor"), ("kft", "káefté"), ("zrt", "zérté"), ("nyrt", "ennyérté"), ("bt", "béti"),
    ("szt", "szent"), ("am", "amúgy"),
    ("ea", "előadás"), ("tkp", "tulajdonképpen"), ("sztm", "szerintem"),
    ("ker", "kerület"), ("pu", "pályaudvar"), ("isk", "iskola"), ("ált", "általános"), ("kb", "körülbelül"),
    ("uő", "ugyanő"), ("ua", "ugyanaz"), ("hrsz", "helyrajzi szám"), ("ny", "enny"), ("ly", "ejj"),
    ("cs", "csé"), ("sz", "essz"), ("zs", "zsé"), ("gy", "gyé"), ("ty", "tyé"), ("dz", "dzé"), ("dzs", "dzsé"),
];

/// Historical name spellings and other words the rules cannot read.
const PHON: &[(&str, &str)] = &[
    ("kossuth", "kˈoʃut"), ("batthyány", "bˈɑcːaːɲ"), ("thököly", "tˈøkøji"), ("dessewffy", "dˈɛʒøːfi"),
    ("weöres", "vˈørøʃ"), ("eötvös", "ˈøtvøʃ"), ("gaál", "ɡˈaːl"), ("soós", "ʃˈoːʃ"), ("kiss", "kˈiʃ"),
    ("papp", "pˈɑp"), ("vass", "vˈɑʃ"), ("tóth", "tˈoːt"), ("horváth", "hˈorvaːt"), ("németh", "nˈeːmɛt"),
    ("rácz", "rˈaːts"), ("takáts", "tˈɑkaːtʃ"), ("babits", "bˈɑbitʃ"), ("móricz", "mˈoːrits"),
    ("czuczor", "tsˈutsor"), ("zichy", "zˈitʃi"), ("széchenyi", "sˈeːtʃeːɲi"), ("széchényi", "sˈeːtʃeːɲi"),
    ("madách", "mˈɑdaːtʃ"), ("forgách", "fˈorɡaːtʃ"), ("kisebb", "kˈiʃʃɛbː"), ("ahhoz", "ˈɑhoz"),
    ("ehhez", "ˈɛhɛz"), ("hadd", "hˈɑd"), ("new", "njˈuː"), ("york", "jˈork"),
    ("the", "dø"), ("of", "ɑf"), ("and", "ɛnd"), ("facebook", "fˈeːjsbuk"), ("google", "ɡˈuːɡli"),
    ("youtube", "jˈutuːb"), ("twitter", "tvˈitːɛr"), ("online", "ˈonlaːjn"), ("show", "ʃˈoː"), ("ok", "ˈok"),
    ("okay", "ˈoːkeː"), ("e-mail", "ˈiːmɛjl"), ("email", "ˈiːmɛjl"), ("mail", "mˈɛjl"),
    ("klub", "klˈubː"), ("kösz", "kˈøss"), ("pizza", "pˈidzːɑ"), ("pizzát", "pˈidzːaːt"), ("pizzák", "pˈidzːaːk"),
    ("pizzéria", "pidzːˈeːriɑ"), ("rock", "rˈokː"), ("chip", "tʃˈip"), ("chips", "tʃˈips"), ("western", "vˈɛstɛrn"),
    ("stroke", "strˈoːk"), ("boy", "bˈoj"), ("cowboy", "kˈovboj"), ("bitcoin", "bˈitkojn"), ("podcast", "pˈotkɑst"),
    ("design", "dizˈaːjn"), ("pendrive", "pˈɛndraːjv"), ("blues", "blˈuːz"), ("fair", "fˈɛr"), ("thai", "tˈaːj"),
    ("house", "hˈɑuz"), ("musical", "mjˈuzikɛl"), ("baseball", "bˈeːzboːl"), ("halloween", "hˈɛloviːn"),
    ("rap", "rˈɛp"), ("puzzle", "pˈaːzɛl"), ("washington", "vˈɑʃinkton"), ("washingtoni", "vˈɑʃinktoni"),
    ("workshop", "vˈørkʃop"), ("soul", "sˈoːl"), ("go", "ɡˈoː"), ("texas", "tˈɛksɑs"), ("texasi", "tˈɛksɑsi"),
    ("web", "vˈɛb"), ("weboldal", "vˈɛboldɑl"), ("windows", "vˈindoːz"), ("microsoft", "mˈɑjkrosoft"),
    ("free", "frˈiː"), ("team", "tˈiːm"), ("design", "dizˈɑjn"), ("business", "bˈiznis"),
    ("manager", "mˈɛnɛdʒɛr"), ("live", "lˈɑjv"), ("like", "lˈɑjk"), ("cool", "kˈuːl"), ("jazz", "dʒˈɛz"),
    ("whisky", "vˈiski"), ("quiz", "kvˈiz"), ("baby", "bˈeːbi"), ("party", "pˈaːrti"), ("sexy", "sˈɛksi"),
    ("hobby", "hˈobːi"), ("hobbi", "hˈobːi"), ("szexi", "sˈɛksi"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Vowel,
    /// nasal, liquid, glide
    Son,
    /// obstruent: `voiced`, and whether it triggers assimilation (v and h don't)
    Obs { voiced: bool },
}

#[derive(Clone, Copy, Debug)]
struct Seg {
    ph: &'static str,
    kind: Kind,
    long: bool,
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'á' | 'e' | 'é' | 'i' | 'í' | 'o' | 'ó' | 'ö' | 'ő' | 'u' | 'ú' | 'ü' | 'ű')
}

fn seg(ph: &'static str) -> Seg {
    let kind = match ph {
        "ɑ" | "aː" | "ɛ" | "eː" | "i" | "iː" | "o" | "oː" | "ø" | "øː" | "u" | "uː" | "y" | "yː" => Kind::Vowel,
        "m" | "n" | "ɲ" | "l" | "r" | "j" => Kind::Son,
        "b" | "d" | "ɡ" | "v" | "z" | "ʒ" | "dz" | "dʒ" | "ɟ" => Kind::Obs { voiced: true },
        _ => Kind::Obs { voiced: false },
    };
    Seg { ph, kind, long: false }
}

fn voiced_of(ph: &str) -> &'static str {
    match ph {
        "p" => "b",
        "t" => "d",
        "k" => "ɡ",
        "f" => "v",
        "s" => "z",
        "ʃ" => "ʒ",
        "ts" => "dz",
        "tʃ" => "dʒ",
        "c" => "ɟ",
        "b" => "b",
        "d" => "d",
        "ɡ" => "ɡ",
        "v" => "v",
        "z" => "z",
        "ʒ" => "ʒ",
        "dz" => "dz",
        "dʒ" => "dʒ",
        "ɟ" => "ɟ",
        "h" => "h",
        _ => "",
    }
}

fn voiceless_of(ph: &str) -> &'static str {
    match ph {
        "b" => "p",
        "d" => "t",
        "ɡ" => "k",
        "v" => "f",
        "z" => "s",
        "ʒ" => "ʃ",
        "dz" => "ts",
        "dʒ" => "tʃ",
        "ɟ" => "c",
        "p" => "p",
        "t" => "t",
        "k" => "k",
        "f" => "f",
        "s" => "s",
        "ʃ" => "ʃ",
        "ts" => "ts",
        "tʃ" => "tʃ",
        "c" => "c",
        "h" => "h",
        _ => "",
    }
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    if let Some((_, ph)) = PHON.iter().find(|(k, _)| *k == w) {
        return ph.to_string();
    }
    // hyphenated compounds and numbers: each part keeps its own stress, except a
    // case suffix after a hyphen (BKV-nál, 25-től), which is glued unstressed
    if w.contains('-') {
        let mut out = String::new();
        for part in w.split('-') {
            let p = word(part);
            if p.is_empty() {
                continue;
            }
            if !out.is_empty() && SUFFIXES.contains(&part) {
                out.extend(p.chars().filter(|&c| c != 'ˈ' && c != 'ˌ'));
            } else {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&p);
            }
        }
        return out;
    }
    let lw: String = w.chars().filter(|c| c.is_alphabetic()).collect();
    if lw.is_empty() {
        return String::new();
    }
    if let Some((_, ph)) = PHON.iter().find(|(k, _)| *k == lw) {
        return ph.to_string();
    }
    if let Some((_, full)) = ABBREV.iter().find(|(k, _)| *k == lw) {
        return full.split(' ').map(word).collect::<Vec<_>>().join(" ");
    }
    let c: Vec<char> = lw.chars().collect();
    // a lone letter is read by its name (s = és is a letter name too: ɛʃ)
    if c.len() == 1 {
        if let Some(name) = letter_name(c[0]) {
            return stress_first(name);
        }
    }
    let mut segs = letters_to_segs(&c);
    if segs.iter().all(|s| s.kind != Kind::Vowel) {
        // consonant-only token (abbreviation): spell it, primary stress on the last letter
        let names: Vec<&str> = c.iter().filter_map(|&ch| letter_name(ch)).collect();
        let mut out = String::new();
        for (k, name) in names.iter().enumerate() {
            let s = stress_first(name);
            if k + 1 < names.len() {
                out.push_str(&s.replace('ˈ', "ˌ"));
            } else {
                out.push_str(&s);
            }
        }
        return out;
    }
    if SILENT_H.contains(&lw.as_str()) {
        segs.pop();
    }
    egy_long(&lw, &mut segs);
    assimilate(&mut segs);
    render(&segs)
}

fn letter_name(ch: char) -> Option<&'static str> {
    let table = LETTERS.unwrap();
    Some(match ch {
        'a'..='z' => table[(ch as u8 - b'a') as usize],
        'á' => "aː",
        'é' => "eː",
        'í' => "iː",
        'ó' => "oː",
        'ö' => "ø",
        'ő' => "øː",
        'ú' => "uː",
        'ü' => "y",
        'ű' => "yː",
        _ => return None,
    })
}

/// `ˈ` right before the first vowel of a phoneme string.
fn stress_first(name: &str) -> String {
    let pos = name.find(|ch: char| "ɑaɛeioøuy".contains(ch)).unwrap_or(0);
    format!("{}ˈ{}", &name[..pos], &name[pos..])
}

/// Greedy longest-match of the Hungarian (multi)graphs.
fn letters_to_segs(c: &[char]) -> Vec<Seg> {
    let n = c.len();
    let at = |i: usize| -> char { if i < n { c[i] } else { '#' } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 2);
    let mut i = 0usize;
    // (letters, phone, long)
    const MULTI: &[(&str, &str, bool)] = &[
        ("ddzs", "dʒ", true), ("dzs", "dʒ", false),
        ("ccs", "tʃ", true), ("cs", "tʃ", false),
        ("ssz", "s", true), ("szsz", "s", true), ("sz", "s", false),
        ("zzs", "ʒ", true), ("zszs", "ʒ", true), ("zs", "ʒ", false),
        ("ggy", "ɟ", true), ("gygy", "ɟ", true), ("gy", "ɟ", false),
        ("tty", "c", true), ("tyty", "c", true), ("ty", "c", false),
        ("nny", "ɲ", true), ("nyny", "ɲ", true), ("ny", "ɲ", false),
        ("lly", "j", true), ("lyly", "j", true), ("ly", "j", false),
        ("ddz", "dz", true), ("dzdz", "dz", true), ("dz", "dz", false),
        ("ch", "h", false), ("cz", "ts", false), ("qu", "kv", false),
    ];
    while i < n {
        let ch = c[i];
        let mut hit = None;
        if matches!(ch, 'c' | 'd' | 'g' | 'l' | 'n' | 'q' | 's' | 't' | 'z') {
            for &(letters, ph, long) in MULTI {
                // the table is ASCII, so bytes compare with chars directly
                let lb = letters.as_bytes();
                if i + lb.len() <= n && lb.iter().zip(&c[i..]).all(|(&b, &x)| x == b as char) {
                    hit = Some((lb.len(), ph, long));
                    break;
                }
            }
        }
        // z + the suffix -ság/-ség (igazság, község) and z + sz (közszolgálat,
        // százszor) are not the digraph zs; c + sz (nyolcszor) is not cs
        if ch == 'z' && at(i + 1) == 's' && ((matches!(at(i + 2), 'á' | 'é') && at(i + 3) == 'g') || at(i + 2) == 'z') {
            hit = None;
        }
        if ch == 'c' && at(i + 1) == 's' && at(i + 2) == 'z' {
            hit = None;
        }
        // superlative leg- and ország- before gy: g + gy (leggyakoribb, országgyűlés)
        if ch == 'g' && at(i + 1) == 'g' && at(i + 2) == 'y' {
            let prefix: String = c[..=i].iter().collect();
            let rest: String = c[i + 3..].iter().collect();
            // meg- + gy-verb (meggyőz, meggyógyul), not the fruit meggy (meggyes, meggyet)
            let meg_verb = prefix == "meg" && rest.chars().count() >= 2 && !rest.starts_with('e');
            if prefix == "leg" || prefix.ends_with("ország") || meg_verb {
                segs.push(seg("ɡ"));
                i += 1;
                continue;
            }
        }
        if let Some((len, ph, long)) = hit {
            // ch: h in Greek/Latin loans (technika), tʃ at the start (chip) and at the end of names (Madách)
            let ph = if ph == "h" && len == 2 && ((i + 2 == n && i > 0) || i == 0) { "tʃ" } else { ph };
            if ph == "kv" {
                segs.push(seg("k"));
                segs.push(seg("v"));
            } else {
                let mut s = seg(ph);
                s.long = long;
                segs.push(s);
            }
            i += len;
            continue;
        }
        let ph: &'static str = match ch {
            'a' => "ɑ",
            'á' => "aː",
            'e' => "ɛ",
            'é' => "eː",
            'i' | 'í' => if ch == 'i' { "i" } else { "iː" },
            'o' => "o",
            'ó' => "oː",
            'ö' => "ø",
            'ő' => "øː",
            'u' => "u",
            'ú' => "uː",
            'ü' => "y",
            'ű' => "yː",
            'b' => "b",
            'c' => "ts",
            'd' => "d",
            'f' => "f",
            'g' => "ɡ",
            'h' => "h",
            'j' => "j",
            'k' => "k",
            'l' => "l",
            'm' => "m",
            'n' => "n",
            'p' => "p",
            'q' => "k",
            'r' => "r",
            's' => "ʃ",
            't' => "t",
            'v' | 'w' => "v",
            'x' => {
                segs.push(seg("k"));
                segs.push(seg("s"));
                i += 1;
                continue;
            }
            'y' => if i == 0 && is_vowel_letter(at(1)) { "j" } else { "i" },
            'z' => "z",
            _ => {
                i += 1;
                continue;
            }
        };
        let mut s = seg(ph);
        // doubled consonant letter = long consonant
        if s.kind != Kind::Vowel && at(i + 1) == ch {
            s.long = true;
            i += 1;
        }
        segs.push(s);
        i += 1;
    }
    // dz and dzs are long between vowels and after a vowel at the end (edző, bodza, bridzs)
    let m = segs.len();
    for i in 1..m {
        if matches!(segs[i].ph, "dz" | "dʒ") && segs[i - 1].kind == Kind::Vowel && (i + 1 == m || segs[i + 1].kind == Kind::Vowel) {
            segs[i].long = true;
        }
    }
    segs
}

/// `egy` as a word or stem is `ɛɟː` (egy, egyik, egyet, együtt, huszonegy)
/// except in the derivatives listed in EGY_SHORT.
fn egy_long(lw: &str, segs: &mut [Seg]) {
    if let Some(stem) = lw.strip_suffix("egy") {
        if stem.is_empty() || EGY_PREFIX.iter().any(|p| stem.ends_with(p)) {
            if let Some(last) = segs.last_mut() {
                if last.ph == "ɟ" {
                    last.long = true;
                }
            }
        }
    }
    if let Some(p) = lw.find("egy") {
        let prefix = &lw[..p];
        let base = &lw[p..];
        let short = EGY_SHORT.iter().any(|s| if let Some(w) = s.strip_suffix('#') { base == w } else { base.starts_with(s) });
        if (prefix.is_empty() || EGY_PREFIX.contains(&prefix)) && base.chars().nth(3).map_or(false, is_vowel_letter) && !short {
            // the ɟ is the segment after the (k+1)-th vowel, k = vowels in the prefix
            let k = prefix.chars().filter(|&c| is_vowel_letter(c)).count();
            let idx = segs.iter().enumerate().filter(|(_, s)| s.kind == Kind::Vowel).nth(k).map(|(i, _)| i + 1);
            if let Some(s) = idx.and_then(|i| segs.get_mut(i)) {
                if s.ph == "ɟ" {
                    s.long = true;
                }
            }
        }
    }
    // millió and its forms have a short l (milliárd keeps the long one)
    if lw.starts_with("millió") {
        if let Some(s) = segs.iter_mut().find(|s| s.ph == "l") {
            s.long = false;
        }
    }
    // kisebb and its derivatives: long ʃ
    if lw.contains("kisebb") {
        if let Some(s) = segs.iter_mut().find(|s| s.ph == "ʃ") {
            s.long = true;
        }
    }
}

fn is_sib_fric(ph: &str) -> bool {
    matches!(ph, "s" | "z" | "ʃ" | "ʒ")
}

/// Consonant sandhi inside the word: j-assimilation, sibilant assimilation,
/// regressive voicing, identical-cluster merge and degemination.
fn assimilate(segs: &mut Vec<Seg>) {
    // j-assimilation: tj dj gyj tyj nyj nj lj
    let mut i = 0;
    while i + 1 < segs.len() {
        if segs[i + 1].ph == "j" && !segs[i + 1].long {
            let to = match segs[i].ph {
                "t" | "c" => Some("c"),
                "d" | "ɟ" => Some("ɟ"),
                "n" | "ɲ" => Some("ɲ"),
                "l" => Some("j"),
                _ => None,
            };
            if let Some(to) = to {
                let mut s = seg(to);
                s.long = true;
                segs[i] = s;
                segs.remove(i + 1);
                continue;
            }
        }
        i += 1;
    }
    // sibilant assimilation: t/d + s/sz/c/cs -> long affricate; sibilant fricative + sibilant fricative -> the second, long;
    // l + r -> rr (balra), n + m -> mm (önmaga)
    let mut i = 0;
    while i + 1 < segs.len() {
        let (a, b) = (segs[i].ph, segs[i + 1].ph);
        if (a == "l" && !segs[i].long && b == "r") || (a == "n" && b == "m") {
            let mut s = segs[i + 1];
            s.long = true;
            segs[i] = s;
            segs.remove(i + 1);
            continue;
        }
        if matches!(a, "t" | "d") && !segs[i].long {
            let to = match b {
                "ʃ" | "tʃ" => Some("tʃ"),
                "s" | "ts" => Some("ts"),
                "ʒ" | "dʒ" => Some("dʒ"),
                "z" | "dz" => Some("dz"),
                _ => None,
            };
            if let Some(to) = to {
                let mut s = seg(to);
                s.long = true;
                segs[i] = s;
                segs.remove(i + 1);
                continue;
            }
        }
        if is_sib_fric(a) && is_sib_fric(b) && !segs[i].long {
            let mut s = segs[i + 1];
            s.long = true;
            segs[i] = s;
            segs.remove(i + 1);
            continue;
        }
        i += 1;
    }
    // regressive voicing: an obstruent takes the voicing of the following obstruent;
    // v and h undergo but do not trigger (hatvan, adhat -> ɑthɑt)
    let n = segs.len();
    for i in (0..n.saturating_sub(1)).rev() {
        let next = segs[i + 1];
        let (Kind::Obs { .. }, Kind::Obs { voiced }) = (segs[i].kind, next.kind) else { continue };
        if next.ph == "v" {
            continue;
        }
        let cur = segs[i].ph;
        if cur == "h" {
            continue;
        }
        let to = if voiced { voiced_of(cur) } else { voiceless_of(cur) };
        if !to.is_empty() && to != cur {
            segs[i].ph = to;
            segs[i].kind = Kind::Obs { voiced };
        }
    }
    // identical adjacent consonants merge into a long one (tudtam -> tutːɑm)
    let mut i = 0;
    while i + 1 < segs.len() {
        if segs[i].kind != Kind::Vowel && segs[i].ph == segs[i + 1].ph {
            segs[i].long = true;
            segs.remove(i + 1);
            continue;
        }
        i += 1;
    }
    // degemination: a long consonant next to another consonant is short (hallgat, jobbra, mondd)
    let n = segs.len();
    for i in 0..n {
        if !segs[i].long || segs[i].kind == Kind::Vowel {
            continue;
        }
        let before = i > 0 && segs[i - 1].kind != Kind::Vowel;
        let after = i + 1 < n && segs[i + 1].kind != Kind::Vowel;
        if before || after {
            segs[i].long = false;
        }
    }
}

fn render(segs: &[Seg]) -> String {
    let mut out = String::with_capacity(segs.len() * 3 + 2);
    let mut stressed = false;
    for s in segs {
        if s.kind == Kind::Vowel && !stressed {
            out.push('ˈ');
            stressed = true;
        }
        out.push_str(s.ph);
        if s.long {
            // espeak: stops, affricates and palatals with ː, fricatives and sonorants doubled
            if matches!(s.ph, "p" | "b" | "t" | "d" | "k" | "ɡ" | "ts" | "dz" | "tʃ" | "dʒ" | "c" | "ɟ") {
                out.push('ː');
            } else {
                out.push_str(s.ph);
            }
        }
    }
    out
}

/// Case suffixes written with a hyphen after a number or an acronym (1848-ban,
/// 25-től, BKV-nál); the tokenizer hands them over as separate words.
const SUFFIXES: &[&str] = &[
    "ban", "ben", "ba", "be", "ból", "ből", "nak", "nek", "ra", "re", "ról", "ről", "tól", "től", "hoz", "hez",
    "höz", "ig", "on", "en", "ön", "án", "én", "at", "et", "ot", "öt", "val", "vel", "as", "es", "os", "ös", "ás",
    "és", "ig", "kor", "nál", "nél", "ként", "szor", "szer", "ször", "tel", "mal", "mel", "gyel", "ttel", "ssal",
    "ssel", "cal", "cel", "ikén", "ike", "iki", "ei", "ai", "i", "ét", "át", "ért", "ik", "edik", "adik", "odik",
    "ödik", "es", "ös", "os", "as", "ának", "ének", "ára", "ére", "ában", "ében", "ából", "éből",
];

/// Sentence pass: a hyphenated suffix after a number or acronym is glued to it
/// without its own stress (espeak: `nɛɟvɛnɲoltsbˌɑn`); otherwise every word
/// stands alone (espeak does no cross-word voicing either).
pub fn join(outs: &mut [Out]) {
    const NUMBER_ENDS: &[&str] = &[
        "egy", "kettő", "három", "négy", "öt", "hat", "hét", "nyolc", "kilenc", "tíz", "húsz", "harminc", "negyven",
        "ötven", "hatvan", "hetven", "nyolcvan", "kilencven", "száz", "ezer", "millió", "milliárd", "nulla",
    ];
    for k in 1..outs.len() {
        let (a, b) = outs.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &mut b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() || !SUFFIXES.contains(&cur.src.as_str()) {
            continue;
        }
        // only after a number word or a consonant-only acronym (BKV)
        let number = NUMBER_ENDS.iter().any(|n| prev.src.ends_with(n));
        let acronym = !prev.src.chars().any(is_vowel_letter) && prev.src.chars().all(|c| c.is_alphabetic());
        if !number && !acronym {
            continue;
        }
        let tail: String = cur.ph.chars().filter(|&c| c != 'ˈ' && c != 'ˌ').collect();
        prev.ph.push_str(&tail);
        prev.punct = std::mem::take(&mut cur.punct);
        prev.pause_after = cur.pause_after;
        cur.ph.clear();
    }
}

const ONES: [&str; 10] = ["nulla", "egy", "kettő", "három", "négy", "öt", "hat", "hét", "nyolc", "kilenc"];
/// Forms used inside compounds (kétszáz, kétezer, kétmillió).
const ONES_C: [&str; 10] = ["", "egy", "két", "három", "négy", "öt", "hat", "hét", "nyolc", "kilenc"];
const TENS: [&str; 10] = ["", "tíz", "húsz", "harminc", "negyven", "ötven", "hatvan", "hetven", "nyolcvan", "kilencven"];
const TEENS: [&str; 10] = ["tizen", "huszon", "harminc", "negyven", "ötven", "hatvan", "hetven", "nyolcvan", "kilencven", ""];

/// `last`: the final group of the number, where 2 is "kettő" (tizenkettő)
/// instead of the compound form "két" (tizenkétezer).
fn below_1000(n: u128, last: bool) -> String {
    let mut s = String::new();
    let h = (n / 100) as usize;
    let r = (n % 100) as usize;
    let one = |d: usize| if d == 2 && last { "kettő" } else { ONES_C[d] };
    if h > 0 {
        if h > 1 {
            s.push_str(ONES_C[h]);
        }
        s.push_str("száz");
    }
    if r >= 10 {
        if r % 10 == 0 {
            s.push_str(TENS[r / 10]);
        } else {
            s.push_str(TEENS[r / 10 - 1]);
            s.push_str(one(r % 10));
        }
    } else if r > 0 {
        s.push_str(one(r));
    }
    s
}

/// Cardinal number words. Up to 2000 one word (ezernyolcszáznegyvennyolc);
/// above, the thousand-groups are hyphenated (kétezer-huszonnégy), and `word`
/// stresses each part.
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["nulla".into()];
    }
    if n < 10 {
        return vec![ONES[n as usize].into()];
    }
    const SCALES: [&str; 6] = ["ezer", "millió", "milliárd", "billió", "billiárd", "trillió"];
    let mut groups: Vec<u128> = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push(m % 1000);
        m /= 1000;
    }
    let mut parts: Vec<String> = Vec::new();
    for (gi, &g) in groups.iter().enumerate().rev() {
        if g == 0 {
            continue;
        }
        let mut p = String::new();
        if gi == 0 {
            p.push_str(&below_1000(g, true));
        } else {
            let scale = SCALES.get(gi - 1).copied().unwrap_or("trillió");
            // "ezer" alone for 1000; the others take "egy"
            if !(g == 1 && gi == 1) {
                p.push_str(&below_1000(g, false));
            }
            p.push_str(scale);
        }
        parts.push(p);
    }
    if n <= 2000 {
        vec![parts.concat()]
    } else {
        parts
    }
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "százalék",
        '$' => "dollár",
        '€' => "euró",
        '£' => "font",
        '&' => "és",
        '+' => "plusz",
        '@' => "kukac",
        '°' => "fok",
        '=' => "egyenlő",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn basics() {
        for (w, p) in [
            // vowels and stress
            ("nem", "nˈɛm"),
            ("az", "ˈɑz"),
            ("három", "hˈaːrom"),
            ("első", "ˈɛlʃøː"),
            ("ürügy", "ˈyryɟ"),
            ("bringa", "brˈinɡɑ"),
            ("miért", "mˈieːrt"),
            ("hosszú", "hˈossuː"),
            ("tűz", "tˈyːz"),
            // digraphs and their doubled forms
            ("szép", "sˈeːp"),
            ("zsák", "ʒˈaːk"),
            ("csak", "tʃˈɑk"),
            ("dzsungel", "dʒˈunɡɛl"),
            ("magyar", "mˈɑɟɑr"),
            ("kutya", "kˈucɑ"),
            ("nyár", "ɲˈaːr"),
            ("lyuk", "jˈuk"),
            ("meggy", "mˈɛɟː"),
            ("hattyú", "hˈɑcːuː"),
            ("könnyű", "kˈøɲɲyː"),
            ("friss", "frˈiʃʃ"),
            ("öccs", "ˈøtʃː"),
            ("vicces", "vˈitsːɛʃ"),
            ("edző", "ˈɛdzːøː"),
            ("egészség", "ˈɛɡeːʃʃeːɡ"),
            ("kettő", "kˈɛtːøː"),
            ("ellen", "ˈɛllɛn"),
            ("taxi", "tˈɑksi"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn sandhi() {
        for (w, p) in [
            // j-assimilation
            ("tudja", "tˈuɟːɑ"),
            ("látja", "lˈaːcːɑ"),
            ("anyja", "ˈɑɲɲɑ"),
            ("bánja", "bˈaːɲɲɑ"),
            ("teljes", "tˈɛjjɛʃ"),
            ("hallja", "hˈɑjjɑ"),
            ("hagyj", "hˈɑɟː"),
            ("mondja", "mˈonɟɑ"),
            // sibilants
            ("tanítson", "tˈɑniːtʃːon"),
            ("szabadság", "sˈɑbɑtʃːaːɡ"),
            ("kétszer", "kˈeːtsːɛr"),
            ("igazság", "ˈiɡɑʃʃaːɡ"),
            ("község", "kˈøʃʃeːɡ"),
            // voicing
            ("azt", "ˈɑst"),
            ("népdal", "nˈeːbdɑl"),
            ("rakd", "rˈɑɡd"),
            ("szívtelen", "sˈiːftɛlɛn"),
            ("hatvan", "hˈɑtvɑn"),
            ("adhat", "ˈɑthɑt"),
            ("egyszer", "ˈɛcsɛr"),
            ("nagyság", "nˈɑcʃaːɡ"),
            ("tudtam", "tˈutːɑm"),
            ("kezdte", "kˈɛstɛ"),
            // degemination, egy-, silent h
            ("hallgat", "hˈɑlɡɑt"),
            ("jobbra", "jˈobrɑ"),
            ("egyik", "ˈɛɟːik"),
            ("egy", "ˈɛɟː"),
            ("huszonegy", "hˈusonɛɟː"),
            ("egyedül", "ˈɛɟɛdyl"),
            ("egyértelmű", "ˈɛɟːeːrtɛlmyː"),
            ("tizenegyedik", "tˈizɛnɛɟːɛdik"),
            ("méh", "mˈeː"),
            ("méhek", "mˈeːhɛk"),
            ("kossuth", "kˈoʃut"),
            ("s", "ˈɛʃʃ"),
            ("mintha", "mˈinthɑ"),
            ("balra", "bˈɑrrɑ"),
            ("közszolgálat", "kˈøssolɡaːlɑt"),
            ("leggyakoribb", "lˈɛɡɟɑkoribː"),
            ("megy", "mˈɛɟ"),
            ("mindegyik", "mˈindɛɟːik"),
            ("megegyezik", "mˈɛɡɛɟːɛzik"),
            ("meggyőz", "mˈɛɡɟøːz"),
            ("meggyes", "mˈɛɟːɛʃ"),
            ("legkisebb", "lˈɛkːiʃʃɛbː"),
            ("önmaga", "ˈømmɑɡɑ"),
            ("db", "dˈɑrɑb"),
            ("cs", "tʃˈeː"),
            ("kétezer-huszonnégy", "kˈeːtɛzɛr hˈusonneːɟ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn suffix_join() {
        use crate::Out;
        let mk = |src: &str| Out { ph: word(src), punct: String::new(), pause_after: false, src: src.to_string() };
        let mut outs = vec![mk("ezernyolcszáznegyvennyolc"), mk("ban"), mk("és"), mk("ma"), mk("tóth"), mk("és"), mk("bkv"), mk("nál")];
        super::join(&mut outs);
        let got: Vec<&str> = outs.iter().map(|o| o.ph.as_str()).collect();
        assert_eq!(got, ["ˈɛzɛrɲoltssaːznɛɟvɛɲɲoltsbɑn", "", "ˈeːʃ", "mˈɑ", "tˈoːt", "ˈeːʃ", "bˌeːkˌaːvˈeːnaːl", ""]);
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), vec!["nulla"]);
        assert_eq!(cardinal(2), vec!["kettő"]);
        assert_eq!(cardinal(25), vec!["huszonöt"]);
        assert_eq!(cardinal(200), vec!["kétszáz"]);
        assert_eq!(cardinal(1000), vec!["ezer"]);
        assert_eq!(cardinal(1848), vec!["ezernyolcszáznegyvennyolc"]);
        assert_eq!(cardinal(2000), vec!["kétezer"]);
        assert_eq!(cardinal(2024), vec!["kétezer", "huszonnégy"]);
        assert_eq!(cardinal(1_000_000), vec!["egymillió"]);
        assert_eq!(cardinal(3_500_012), vec!["hárommillió", "ötszázezer", "tizenkettő"]);
    }
}
