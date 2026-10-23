//! Polish grapheme-to-phoneme rules, espeak-style notation.
//!
//! Polish spelling is close to phonemic, so this is rules only: digraphs
//! (sz cz rz dz dź dż ch), the soft series (ś ź ć ń dź and si/zi/ci/ni/dzi
//! before a vowel), palatalized labials and velars before i, the nasal vowels
//! ę/ą and their context realization, voicing assimilation and final
//! devoicing, and penultimate stress with the regular exceptions.
//!
//! Notation follows espeak-ng's `pl` voice: vowels `a ɛ i ɔ u ɨ`, nasals `ɛ̃ ɔ̃`,
//! sz/ż/cz/dż as `ʃ ʒ tʃ dʒ`, trz/drz as `tʃ dʒ`, the soft series as
//! `ɕ ʑ tɕ dʑ ɲ`, `ł` as `w`, h/ch as `x` (`ç` before i), and the primary stress
//! mark `ˈ` immediately before the stressed vowel.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "przecinek";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "bɛ", "tsɛ", "dɛ", "ɛ", "ɛf", "ɡʲɛ", "xa", "i", "jɔt", "ka", "ɛl", "ɛm", "ɛn", "ɔ", "pɛ", "ku", "ɛr", "ɛs", "tɛ",
    "u", "faw", "vu", "iks", "iɡrɛk", "zɛt",
]);

/// Clitics and short function words espeak reads without a stress mark.
const UNSTRESSED: &[&str] = &[
    "w", "i", "nie", "się", "to", "z", "a", "od", "dla", "czy", "mi", "mnie", "ich", "jej", "mu", "im", "we", "wam",
    "ni",
];

/// Whole words with stress on the antepenultimate syllable.
const ANTEPENULT_WORDS: &[&str] = &[
    "uniwersytet", "prezydent", "ryzyko", "opera", "czterysta", "siedemset", "osiemset", "dziewięćset", "maksimum",
    "minimum", "optimum", "rzeczpospolita", "reguła", "okolica", "nauka",
];

/// Greek/Latin stems in -yk-/-ik- whose nominal forms (stem + one vowel) stress
/// the antepenult: fizyka, matematyki, logiką...
const YKA_STEMS: &[&str] = &[
    "matematyk", "fizyk", "muzyk", "polityk", "logik", "technik", "mechanik", "gramatyk", "statystyk", "informatyk",
    "praktyk", "krytyk", "etyk", "estetyk", "taktyk", "optyk", "metryk", "fabryk", "rubryk", "afryk", "ameryk",
    "plastyk", "grafik", "botanik", "elektronik", "ceramik", "dydaktyk", "genetyk", "kosmetyk", "lingwistyk",
    "turystyk", "gimnastyk", "charakterystyk", "symbolik", "problematyk", "tematyk", "dramatyk", "akustyk", "mistyk",
    "metodyk", "dynamik", "ekonomik", "fonetyk", "poetyk", "semantyk", "stylistyk", "energetyk", "automatyk",
    "robotyk", "erotyk", "egzotyk", "heraldyk", "klinik", "bazylik", "republik", "arytmetyk", "geometryk", "atletyk",
    "publicystyk", "romantyk", "systematyk", "hydraulik", "mozaik", "kronik", "pediatryk", "psychiatryk", "retoryk",
];

/// Frequent English loans and abbreviations, respelled in Polish orthography
/// (ł = w, dż = dʒ, ej/aj = diphthongs) and fed back through the rules.
const RESPELL: &[(&str, &str)] = &[
    ("weekend", "łikend"), ("weekendu", "łikendu"), ("weekendy", "łikendy"), ("show", "szoł"), ("online", "onlajn"),
    ("offline", "oflajn"), ("like", "lajk"), ("house", "hałs"), ("jazz", "dżez"), ("quiz", "kłiz"),
    ("business", "biznes"), ("design", "dizajn"), ("team", "tim"), ("news", "njus"), ("manager", "menedżer"),
    ("lunch", "lancz"), ("live", "lajw"), ("cool", "kul"), ("rock", "rok"), ("rockowy", "rokowy"), ("nick", "nik"),
    ("windows", "łindołs"), ("mail", "mejl"), ("fair", "fer"), ("disco", "disko"), ("fantasy", "fantazy"),
    ("college", "koledż"), ("cover", "kawer"), ("content", "kontent"), ("country", "kantry"), ("punk", "pank"),
    ("trust", "trast"), ("dubbing", "dabing"), ("bug", "bag"), ("whisky", "łiski"), ("event", "iwent"),
    ("update", "apdejt"), ("gate", "gejt"), ("joker", "dżoker"), ("judo", "dżudo"), ("zombie", "zombi"),
    ("selfie", "selfi"), ("spoiler", "spojler"), ("bitcoin", "bitkojn"), ("hardcore", "hardkor"),
    ("podcast", "podkast"), ("stream", "strim"), ("deal", "dil"), ("tweet", "tłit"), ("twitter", "tłiter"),
    ("facebook", "fejsbuk"), ("google", "gugl"), ("youtube", "jutub"), ("york", "jork"), ("boy", "boj"),
    ("toyota", "tojota"), ("chili", "czili"), ("chilli", "czili"), ("chip", "czip"), ("sushi", "suszi"),
    ("hashtag", "hasztag"), ("shot", "szot"), ("flow", "floł"), ("router", "ruter"), ("trailer", "trejler"),
    ("wow", "łał"), ("ok", "okej"), ("okay", "okej"),
    ("thriller", "triler"), ("happening", "hepening"), ("einstein", "ajnsztajn"), ("voodoo", "wudu"),
    ("soundtrack", "sałntrak"), ("down", "dałn"), ("popcorn", "popkorn"), ("software", "softłer"),
    ("hardware", "hardłer"), ("microsoft", "majkrosoft"), ("chicago", "szikago"), ("georgia", "dżordżja"),
    ("jessica", "dżesika"), ("sydney", "sydnej"), ("disney", "disnej"), ("melbourne", "melbern"),
    ("cambridge", "kembrydż"), ("victoria", "wiktorja"), ("halloween", "halołin"), ("spaghetti", "spageti"),
    ("cola", "kola"), ("gender", "dżender"), ("ninja", "nindża"), ("lee", "li"), ("street", "strit"),
    ("free", "fri"), ("blues", "blus"), ("image", "imidż"), ("point", "pojnt"),
    ("playlista", "plejlista"), ("playlisty", "plejlisty"), ("cheeseburger", "czizburger"),
    ("jeep", "dżip"), ("jeans", "dżins"), ("jingle", "dżingiel"), ("gin", "dżin"),
    ("bike", "bajk"), ("mix", "miks"), ("remix", "remiks"), ("box", "boks"), ("xbox", "iksboks"),
    ("notebook", "nołtbuk"), ("email", "imejl"), ("e-mail", "imejl"), 
    ("outlet", "ałtlet"), ("outdoor", "ałtdor"), ("freestyle", "fristajl"), ("lifestyle", "lajfstajl"),
    ("timeline", "tajmlajn"), ("deadline", "dedlajn"), ("fitness", "fitnes"), ("wellness", "łelnes"),
    ("dream", "drim"), ("beat", "bit"), ("heavy", "hewi"), ("boss", "bos"),
    ("cash", "kesz"), ("flash", "flesz"), ("crash", "kresz"), ("trash", "tresz"), ("fresh", "fresz"),
    ("casting", "kasting"), ("catering", "katering"), ("coach", "kołcz"),
    ("coaching", "kołczing"), ("challenge", "czalendż"), ("vintage", "wintydż"), ("garage", "garaż"),
    ("fake", "fejk"), ("fail", "fejl"), ("hate", "hejt"), ("sale", "sejl"),
    ("make", "mejk"), ("game", "gejm"), ("games", "gejms"), ("gameplay", "gejmplej"), ("play", "plej"),
    ("player", "plejer"), ("display", "displej"), ("steam", "stim"), ("ice", "ajs"), ("nice", "najs"),
    ("white", "łajt"), ("light", "lajt"), ("night", "najt"), ("high", "haj"), ("five", "fajw"),
    ("drive", "drajw"), ("pendrive", "pendrajw"), ("time", "tajm"), ("prime", "prajm"), ("line", "lajn"),
    ("wine", "łajn"), ("style", "stajl"), ("site", "sajt"), ("website", "łebsajt"), ("web", "łeb"),
    ("tweetować", "tłitować"), ("trio", "trijo"),
    // abbreviations
    ("nr", "numer"), ("dr", "doktor"), ("dra", "doktora"), ("zł", "złoty"), ("wg", "według"), ("mgr", "magister"),
    ("mjr", "major"), ("bp", "biskup"), ("abp", "arcybiskup"), ("gr", "grosz"), ("ul", "ulica"), ("al", "aleja"),
    ("pl", "plac"), ("tel", "telefon"), ("godz", "godzina"), ("prof", "profesor"), ("św", "święty"),
    ("ks", "ksiądz"), ("str", "strona"), ("tys", "tysięcy"), ("mln", "milionów"), ("mld", "miliardów"),
    ("np", "na przykład"), ("itd", "i tak dalej"), ("itp", "i tym podobne"), ("tzw", "tak zwany"),
    ("tzn", "to znaczy"), ("min", "minut"), ("ww", "wyżej wymieniony"), ("pkt", "punkt"),
    ("szt", "sztuk"), ("proc", "procent"), ("woj", "województwo"), ("płn", "północny"), ("płd", "południowy"),
    ("wsch", "wschodni"), ("zach", "zachodni"), ("jw", "jak wyżej"), ("cd", "ciąg dalszy"),
    ("dn", "dnia"), ("ds", "do spraw"), ("sms", "esemes"), ("pdf", "pedeef"), ("abc", "abece"),
    ("pm", "pe em"), ("usa", "u es a"),
];

/// Loans whose sounds the Polish spelling rules cannot express (si/ci as [s]/[ts]).
const PHON: &[(&str, &str)] = &[
    ("singiel", "sˈinɡjɛl"), ("pepsi", "pˈɛpsi"), ("sinus", "sˈinus"), ("maksimum", "mˈaksimum"),
    ("intercity", "ˌintɛrsˈiti"), ("dancing", "dˈansink"), ("sic", "sˈik"), ("sexy", "sˈɛksi"),
    ("city", "sˈiti"), ("sitcom", "sˈitkɔm"), ("cv", "tsˌɛfˈaw"), ("pizza", "pˈitstsa"), ("pizzy", "pˈitstsɨ"),
    ("pizzę", "pˈitstsɛ"), ("pizze", "pˈitstsɛ"), ("pizzerii", "pitstsˈɛrji"), ("pizzeria", "pitstsˈɛrja"),
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Vowel,
    /// ę (realized by the following segment)
    NasalE,
    /// ą
    NasalO,
    /// sonorant consonant: m n ɲ ŋ r l w j
    Son,
    /// palatalization marker ʲ / j glued to the preceding consonant
    Pal,
    /// obstruent, voiced or voiceless
    Obs(bool),
}

#[derive(Clone, Debug)]
struct Seg {
    ph: &'static str,
    kind: Kind,
    /// v from w, ʒ from rz: devoiced after a voiceless obstruent (progressive)
    soft_voice: bool,
}

fn seg(ph: &'static str, kind: Kind) -> Seg {
    Seg { ph, kind, soft_voice: false }
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'ą' | 'ę' | 'ó')
}

fn voiced_of(ph: &str) -> &'static str {
    match ph {
        "p" => "b",
        "t" => "d",
        "k" => "ɡ",
        "f" => "v",
        "s" => "z",
        "ʃ" => "ʒ",
        "ɕ" => "ʑ",
        "ts" => "dz",
        "tʃ" => "dʒ",
        "tɕ" => "dʑ",
        "x" | "ç" => "ɣ",
        "b" => "b",
        "d" => "d",
        "ɡ" => "ɡ",
        "v" => "v",
        "z" => "z",
        "ʒ" => "ʒ",
        "ʑ" => "ʑ",
        "dz" => "dz",
        "dʒ" => "dʒ",
        "dʑ" => "dʑ",
        "ɣ" => "ɣ",
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
        "ʑ" => "ɕ",
        "dz" => "ts",
        "dʒ" => "tʃ",
        "dʑ" => "tɕ",
        "ɣ" => "x",
        "p" => "p",
        "t" => "t",
        "k" => "k",
        "f" => "f",
        "s" => "s",
        "ʃ" => "ʃ",
        "ɕ" => "ɕ",
        "ts" => "ts",
        "tʃ" => "tʃ",
        "tɕ" => "tɕ",
        "x" => "x",
        "ç" => "ç",
        _ => "",
    }
}

fn obs(ph: &'static str) -> Seg {
    let voiced = matches!(ph, "b" | "d" | "ɡ" | "v" | "z" | "ʒ" | "ʑ" | "dz" | "dʒ" | "dʑ" | "ɣ");
    seg(ph, Kind::Obs(voiced))
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    let lw = w.to_lowercase();
    // hyphenated compounds: each part keeps its own stress
    if lw.contains('-') {
        let parts: Vec<String> = lw.split('-').map(word).filter(|p| !p.is_empty()).collect();
        return parts.join(" ");
    }
    let lw: String = lw.chars().filter(|c| c.is_alphabetic()).collect();
    if lw.is_empty() {
        return String::new();
    }
    if let Some((_, ph)) = PHON.iter().find(|(k, _)| *k == lw) {
        return ph.to_string();
    }
    if let Some((_, re)) = RESPELL.iter().find(|(k, _)| *k == lw) {
        if *re != lw {
            return re.split(' ').map(word).collect::<Vec<_>>().join(" ");
        }
    }
    let c: Vec<char> = lw.chars().collect();
    // a lone consonant letter (abbreviations: r., s., m) is read by its name
    if c.len() == 1 && (!is_vowel_letter(c[0]) || c[0] == 'y') && !matches!(c[0], 'w' | 'z') && c[0].is_ascii_alphabetic() {
        let name = LETTERS.unwrap()[(c[0] as u8 - b'a') as usize];
        let pos = name.find(|ch: char| matches!(ch, 'a' | 'ɛ' | 'i' | 'ɔ' | 'u')).unwrap_or(0);
        return format!("{}ˈ{}", &name[..pos], &name[pos..]);
    }
    let mut segs = letters_to_segs(&c);
    realize_nasals(&mut segs);
    voicing(&mut segs);
    // ć/dź + dzi simplifies to one affricate (pięćdziesiąt, sześćdziesiąt)
    let mut i = 0;
    while i + 1 < segs.len() {
        if segs[i].ph == "dʑ" && segs[i + 1].ph == "dʑ" {
            segs.remove(i);
        } else {
            i += 1;
        }
    }
    let stressed = !UNSTRESSED.contains(&lw.as_str());
    render(&segs, &lw, stressed)
}

fn letters_to_segs(c: &[char]) -> Vec<Seg> {
    let n = c.len();
    let at = |i: usize| -> char { if i < n { c[i] } else { '#' } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 4);
    let mut i = 0usize;
    // marz- as r+z: zamarzać, marznąć, zmarzł, Tarzan, erzac
    let rz_split = |i: usize| -> bool {
        let rest: String = c[i..].iter().take(4).collect();
        let before: String = c[..i].iter().collect();
        (before.ends_with("ma") && (rest.starts_with("rzn") || rest.starts_with("rzł") || before.ends_with("zama") || before.ends_with("zma")))
            || (before == "ta" && rest.starts_with("rzan"))
            || (before == "e" && rest.starts_with("rzac"))
            || at(i + 2) == 'i'
    };
    while i < n {
        let ch = c[i];
        let nx = at(i + 1);
        let nx2 = at(i + 2);
        let prev = if i > 0 { c[i - 1] } else { '#' };
        // consonant + i + vowel  (soft consonant / palatalization)
        let i_vowel = nx == 'i' && is_vowel_letter(nx2) && nx2 != 'i';
        let i_i = nx == 'i' && nx2 == 'i';
        let i_cons = nx == 'i' && !is_vowel_letter(nx2);
        match ch {
            'a' => {
                let prefix = i == 1 && matches!(c[0], 'n' | 'z');
                if nx == 'u' && !prefix {
                    segs.push(seg("a", Kind::Vowel));
                    segs.push(seg("w", Kind::Son));
                    i += 2;
                    continue;
                }
                segs.push(seg("a", Kind::Vowel));
            }
            'e' => {
                // eu diphthong in loans (Europa, neutralny), not ie-u or final -eum/-eus(z)
                let tail: String = c.get(i + 2..).unwrap_or(&[]).iter().collect();
                let latin = tail == "m" || tail == "s" || tail == "sz";
                if nx == 'u' && prev != 'i' && !latin {
                    segs.push(seg("ɛ", Kind::Vowel));
                    segs.push(seg("w", Kind::Son));
                    i += 2;
                    continue;
                }
                segs.push(seg("ɛ", Kind::Vowel));
            }
            'i' => segs.push(seg("i", Kind::Vowel)),
            'o' => segs.push(seg("ɔ", Kind::Vowel)),
            'ó' | 'u' => segs.push(seg("u", Kind::Vowel)),
            'y' => segs.push(seg("ɨ", Kind::Vowel)),
            'ą' => segs.push(seg("ɔ", Kind::NasalO)),
            'ę' => segs.push(seg("ɛ", Kind::NasalE)),
            'b' => {
                segs.push(obs("b"));
                if i_vowel {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                } else if i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                } else if i_cons {
                    segs.push(seg("ʲ", Kind::Pal));
                }
            }
            'p' => {
                segs.push(obs("p"));
                if i_vowel {
                    segs.push(seg("ʲ", Kind::Pal));
                    i += 1;
                } else if i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'w' | 'v' => {
                let mut s = obs("v");
                s.soft_voice = true;
                segs.push(s);
                if i_vowel {
                    segs.push(seg("ʲ", Kind::Pal));
                    i += 1;
                } else if i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'f' => {
                segs.push(obs("f"));
                if i_vowel || i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'm' => {
                segs.push(seg("m", Kind::Son));
                if i_vowel {
                    // espeak: mʲ after a consonant or word-initially, mj after a vowel
                    let after_cons = i == 0 || !is_vowel_letter(prev);
                    segs.push(seg(if after_cons { "ʲ" } else { "j" }, Kind::Pal));
                    i += 1;
                } else if i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'k' => {
                segs.push(obs("k"));
                if i_vowel {
                    segs.push(seg("ʲ", Kind::Pal));
                    i += 1;
                } else if i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'g' => {
                segs.push(obs("ɡ"));
                if i_vowel {
                    segs.push(seg("ʲ", Kind::Pal));
                    i += 1;
                } else if i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                } else if i_cons {
                    segs.push(seg("ʲ", Kind::Pal));
                }
            }
            'c' => {
                if nx == 'h' {
                    // ch
                    let v2 = at(i + 2) == 'i' && is_vowel_letter(at(i + 3)) && at(i + 3) != 'i';
                    let ii = at(i + 2) == 'i' && at(i + 3) == 'i';
                    if v2 || ii {
                        segs.push(obs("x"));
                        segs.push(seg("j", Kind::Pal));
                        i += 3;
                        continue;
                    } else if at(i + 2) == 'i' {
                        segs.push(obs("ç"));
                    } else {
                        segs.push(obs("x"));
                    }
                    i += 2;
                    continue;
                } else if nx == 'z' {
                    segs.push(obs("tʃ"));
                    i += 2;
                    continue;
                } else if i_vowel {
                    segs.push(obs("tɕ"));
                    i += 2;
                    continue;
                } else if i_cons {
                    segs.push(obs("tɕ"));
                } else {
                    segs.push(obs("ts"));
                }
            }
            'ć' => segs.push(obs("tɕ")),
            's' => {
                if nx == 'z' {
                    segs.push(obs("ʃ"));
                    i += 2;
                    continue;
                } else if i_vowel {
                    segs.push(obs("ɕ"));
                    i += 2;
                    continue;
                } else if i_cons {
                    segs.push(obs("ɕ"));
                } else {
                    segs.push(obs("s"));
                }
            }
            'ś' => segs.push(obs("ɕ")),
            'z' => {
                if nx == 'z' {
                    // pizza, mozzarella
                    segs.push(obs("ts"));
                    segs.push(obs("ts"));
                    i += 2;
                    continue;
                }
                if i_vowel {
                    segs.push(obs("ʑ"));
                    i += 2;
                    continue;
                } else if i_cons {
                    segs.push(obs("ʑ"));
                } else {
                    segs.push(obs("z"));
                }
            }
            'ź' => segs.push(obs("ʑ")),
            'ż' => segs.push(obs("ʒ")),
            'd' => {
                if nx == 'z' {
                    let v2 = at(i + 2) == 'i' && is_vowel_letter(at(i + 3)) && at(i + 3) != 'i';
                    if v2 {
                        segs.push(obs("dʑ"));
                        i += 3;
                        continue;
                    } else if at(i + 2) == 'i' {
                        segs.push(obs("dʑ"));
                    } else {
                        segs.push(obs("dz"));
                    }
                    i += 2;
                    continue;
                } else if nx == 'ź' {
                    segs.push(obs("dʑ"));
                    i += 2;
                    continue;
                } else if nx == 'ż' {
                    segs.push(obs("dʒ"));
                    i += 2;
                    continue;
                }
                segs.push(obs("d"));
                if i_vowel || i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            't' => {
                segs.push(obs("t"));
                if i_vowel || i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'r' => {
                if nx == 'z' && !rz_split(i) {
                    let mut s = obs("ʒ");
                    s.soft_voice = true;
                    segs.push(s);
                    i += 2;
                    continue;
                }
                segs.push(seg("r", Kind::Son));
                if i_vowel || i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'l' => {
                segs.push(seg("l", Kind::Son));
                // lie- is two syllables (klient), lia/lio/liu a glide (Julia, milion)
                if (i_vowel && nx2 != 'e') || i_i {
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                }
            }
            'ł' => segs.push(seg("w", Kind::Son)),
            'n' => {
                if i_vowel {
                    segs.push(seg("ɲ", Kind::Son));
                    segs.push(seg("ʲ", Kind::Pal));
                    i += 2;
                    continue;
                } else if i_i {
                    segs.push(seg("ɲ", Kind::Son));
                    segs.push(seg("j", Kind::Pal));
                    i += 2;
                    continue;
                } else if i_cons {
                    segs.push(seg("ɲ", Kind::Son));
                } else if matches!(nx, 'k' | 'g') {
                    segs.push(seg("ŋ", Kind::Son));
                } else {
                    segs.push(seg("n", Kind::Son));
                }
            }
            'ń' => segs.push(seg("ɲ", Kind::Son)),
            'j' => segs.push(seg("j", Kind::Son)),
            'h' => {
                if i_vowel || i_i {
                    segs.push(obs("x"));
                    segs.push(seg("j", Kind::Pal));
                    i += 1;
                } else if i_cons {
                    segs.push(obs("ç"));
                } else {
                    segs.push(obs("x"));
                }
            }
            'q' => {
                segs.push(obs("k"));
                if nx == 'u' && is_vowel_letter(nx2) {
                    let mut s = obs("v");
                    s.soft_voice = true;
                    segs.push(s);
                    i += 1;
                }
            }
            'x' => {
                segs.push(obs("k"));
                segs.push(obs("s"));
            }
            _ => {}
        }
        i += 1;
    }
    segs
}

/// ę/ą: oral vowel + homorganic nasal before a stop or affricate, oral before
/// l/ł, nasal vowel before fricatives; word-final ę is plain ɛ.
fn realize_nasals(segs: &mut Vec<Seg>) {
    let mut i = 0;
    while i < segs.len() {
        let e = match segs[i].kind {
            Kind::NasalE => true,
            Kind::NasalO => false,
            _ => {
                i += 1;
                continue;
            }
        };
        let next = segs.get(i + 1).map(|s| (s.ph, s.kind));
        let (vowel, nasal): (&'static str, Option<&'static str>) = match next {
            None => (if e { "ɛ" } else { "ɔ̃" }, None),
            Some((ph, Kind::Obs(_))) => match ph {
                "p" | "b" => (if e { "ɛ" } else { "ɔ" }, Some("m")),
                "t" | "d" | "ts" | "dz" | "tʃ" | "dʒ" => (if e { "ɛ" } else { "ɔ" }, Some("n")),
                "tɕ" | "dʑ" | "ɕ" | "ʑ" => (if e { "ɛ" } else { "ɔ" }, Some("ɲ")),
                "k" | "ɡ" => (if e { "ɛ" } else { "ɔ" }, Some("ŋ")),
                _ => (if e { "ɛ̃" } else { "ɔ̃" }, None),
            },
            Some(("l", _)) | Some(("w", _)) => (if e { "ɛ" } else { "ɔ" }, None),
            Some(_) => (if e { "ɛ̃" } else { "ɔ̃" }, None),
        };
        segs[i].ph = vowel;
        segs[i].kind = Kind::Vowel;
        if let Some(nc) = nasal {
            segs.insert(i + 1, seg(nc, Kind::Son));
            i += 1;
        }
        i += 1;
    }
}

/// Final devoicing, progressive devoicing of w/rz after a voiceless obstruent,
/// and regressive voicing assimilation inside obstruent clusters.
fn voicing(segs: &mut [Seg]) {
    let n = segs.len();
    if n == 0 {
        return;
    }
    // n before a soft affricate
    for i in 0..n - 1 {
        if segs[i].ph == "n" && matches!(segs[i + 1].ph, "tɕ" | "dʑ") {
            segs[i].ph = "ɲ";
        }
    }
    // final devoicing (not for the vowelless prepositions w, z)
    let has_vowel = segs.iter().any(|s| s.kind == Kind::Vowel);
    if let (Kind::Obs(true), true) = (segs[n - 1].kind, has_vowel) {
        segs[n - 1].ph = voiceless_of(segs[n - 1].ph);
        segs[n - 1].kind = Kind::Obs(false);
    }
    // w / rz after a voiceless obstruent (also across r/l: trwać, krwi)
    for i in 0..n {
        if !segs[i].soft_voice || segs[i].kind != Kind::Obs(true) {
            continue;
        }
        let mut j = i;
        while j > 0 {
            j -= 1;
            match segs[j].kind {
                Kind::Son if matches!(segs[j].ph, "r" | "l") => continue,
                Kind::Obs(false) => {
                    segs[i].ph = voiceless_of(segs[i].ph);
                    segs[i].kind = Kind::Obs(false);
                }
                _ => {}
            }
            break;
        }
    }
    // regressive: a cluster takes the voicing of its last obstruent
    for i in (0..n - 1).rev() {
        if let (Kind::Obs(_), Kind::Obs(v)) = (segs[i].kind, segs[i + 1].kind) {
            segs[i].ph = if v { voiced_of(segs[i].ph) } else { voiceless_of(segs[i].ph) };
            segs[i].kind = Kind::Obs(v);
        }
    }
}

/// Index (among vowels) of the stressed syllable, counting from the end:
/// 1 = penultimate, 2 = antepenultimate.
fn stress_from_end(lw: &str, nvowels: usize) -> usize {
    if nvowels <= 1 {
        return 0;
    }
    if ANTEPENULT_WORDS.contains(&lw) {
        return 2;
    }
    // -yka/-ika nouns: stem + one vowel letter
    if let Some(last) = lw.chars().last() {
        if matches!(last, 'a' | 'i' | 'ą' | 'ę' | 'o' | 'e' | 'u') {
            let stem = &lw[..lw.len() - last.len_utf8()];
            if YKA_STEMS.iter().any(|s| stem.ends_with(s)) {
                return 2;
            }
            // dative/locative -yce/-ice: the k softens to c
            if last == 'e' && stem.ends_with('c') {
                let kstem = format!("{}k", &stem[..stem.len() - 1]);
                if YKA_STEMS.iter().any(|s| kstem.ends_with(s)) {
                    return 2;
                }
            }
        }
    }
    // verb clitics: -liśmy/-łyśmy/-liście, conditional -by(m/ś/śmy/ście)
    let mut base = lw;
    let mut extra = 0usize;
    for cl in ["byśmy", "byście", "bym", "byś", "by"] {
        if let Some(stem) = base.strip_suffix(cl) {
            if stem.ends_with('ł') || stem.ends_with("li") || stem.ends_with("ły") || stem.ends_with("ła") || stem.ends_with("ło") {
                extra += cl.chars().filter(|c| matches!(c, 'y' | 'e')).count();
                base = stem;
            }
            break;
        }
    }
    for cl in ["śmy", "ście"] {
        if let Some(stem) = base.strip_suffix(cl) {
            if stem.ends_with("li") || stem.ends_with("ły") || stem.ends_with("by") {
                extra += 1;
            }
            break;
        }
    }
    let base_vowels = nvowels - extra;
    if extra > 0 {
        return if base_vowels >= 2 { extra + 1 } else { extra.min(nvowels - 1) };
    }
    1
}

fn render(segs: &[Seg], lw: &str, stressed: bool) -> String {
    let nuclei: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| s.kind == Kind::Vowel).map(|(i, _)| i).collect();
    let nv = nuclei.len();
    let mut out = String::with_capacity(lw.len() * 2 + 4);
    if nv == 0 || !stressed {
        for s in segs {
            out.push_str(s.ph);
        }
        return out;
    }
    let from_end = stress_from_end(lw, nv).min(nv - 1);
    let primary = nv - 1 - from_end;
    let mut k = 0usize;
    for s in segs {
        if s.kind == Kind::Vowel {
            if k == primary {
                out.push('ˈ');
            } else if (k < primary && k + 1 < primary && k % 2 == 0) || (k > primary && (k - primary) % 2 == 0) {
                out.push('ˌ');
            }
            k += 1;
        }
        out.push_str(s.ph);
    }
    out
}

/// Sentence pass: a word-final obstruent is voiced before a word starting with a
/// voiced obstruent (other than w/rz), and the prepositions w/z take the voicing
/// of the next word's first consonant.
pub fn join(words: &mut [Out]) {
    for k in 1..words.len() {
        let (a, b) = words.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() {
            continue;
        }
        let first = first_phone(&cur.ph);
        let voiced_next = matches!(first, "b" | "d" | "ɡ" | "z" | "ʒ" | "ʑ" | "dz" | "dʒ" | "dʑ" | "ɣ")
            && !(cur.src.starts_with('w') || cur.src.starts_with("rz"));
        let voiceless_next = matches!(first, "p" | "t" | "k" | "f" | "s" | "ʃ" | "ɕ" | "ts" | "tʃ" | "tɕ" | "x" | "ç");
        if voiced_next {
            prev.ph = revoice_final(&prev.ph, true);
        } else if voiceless_next && (prev.src == "w" || prev.src == "z") {
            prev.ph = revoice_final(&prev.ph, false);
        }
    }
}

fn first_phone(s: &str) -> &str {
    let s = s.trim_start_matches(|c| c == 'ˈ' || c == 'ˌ');
    for p in ["dz", "dʒ", "dʑ", "ts", "tʃ", "tɕ"] {
        if s.starts_with(p) {
            return p;
        }
    }
    let mut it = s.char_indices();
    match (it.next(), it.next()) {
        (Some(_), Some((e, _))) => &s[..e],
        (Some(_), None) => s,
        _ => "",
    }
}

/// Re-voice (or devoice) the final run of obstruents of a phoneme string.
fn revoice_final(ph: &str, voiced: bool) -> String {
    const OBS: &[&str] = &[
        "dz", "dʒ", "dʑ", "ts", "tʃ", "tɕ", "p", "b", "t", "d", "k", "ɡ", "f", "v", "s", "z", "ʃ", "ʒ", "ɕ", "ʑ", "x", "ɣ", "ç",
    ];
    let mut rest = ph;
    let mut tail: Vec<&'static str> = Vec::new();
    loop {
        let mut hit = None;
        for p in OBS {
            if rest.ends_with(p) {
                hit = Some(*p);
                break;
            }
        }
        match hit {
            Some(p) => {
                tail.push(if voiced { voiced_of(p) } else { voiceless_of(p) });
                rest = &rest[..rest.len() - p.len()];
            }
            None => break,
        }
    }
    let mut out = String::from(rest);
    for p in tail.iter().rev() {
        out.push_str(p);
    }
    out
}

const UNITS: [&str; 20] = [
    "zero", "jeden", "dwa", "trzy", "cztery", "pięć", "sześć", "siedem", "osiem", "dziewięć", "dziesięć", "jedenaście",
    "dwanaście", "trzynaście", "czternaście", "piętnaście", "szesnaście", "siedemnaście", "osiemnaście",
    "dziewiętnaście",
];
const TENS: [&str; 10] = [
    "", "", "dwadzieścia", "trzydzieści", "czterdzieści", "pięćdziesiąt", "sześćdziesiąt", "siedemdziesiąt",
    "osiemdziesiąt", "dziewięćdziesiąt",
];
const HUNDREDS: [&str; 10] = [
    "", "sto", "dwieście", "trzysta", "czterysta", "pięćset", "sześćset", "siedemset", "osiemset", "dziewięćset",
];
/// (singular, paucal 2-4, plural genitive) for 10^3, 10^6, 10^9, ...
const SCALES: [(&str, &str, &str); 6] = [
    ("tysiąc", "tysiące", "tysięcy"),
    ("milion", "miliony", "milionów"),
    ("miliard", "miliardy", "miliardów"),
    ("bilion", "biliony", "bilionów"),
    ("biliard", "biliardy", "biliardów"),
    ("trylion", "tryliony", "trylionów"),
];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = n % 100;
    if h > 0 {
        out.push(HUNDREDS[h].into());
    }
    if r >= 20 {
        out.push(TENS[(r / 10) as usize].into());
        if r % 10 > 0 {
            out.push(UNITS[(r % 10) as usize].into());
        }
    } else if r > 0 || n == 0 {
        out.push(UNITS[r as usize].into());
    }
}

pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n < 1000 {
        below_1000(n, &mut out);
        return out;
    }
    // split into groups of three digits, highest first
    let mut groups: Vec<u128> = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push(m % 1000);
        m /= 1000;
    }
    for (gi, &g) in groups.iter().enumerate().rev() {
        if g == 0 {
            continue;
        }
        if gi == 0 {
            below_1000(g, &mut out);
            continue;
        }
        let Some(&(one, few, many)) = SCALES.get(gi - 1) else {
            below_1000(g, &mut out);
            continue;
        };
        if g == 1 {
            out.push(one.into());
            continue;
        }
        below_1000(g, &mut out);
        let few_form = (2..=4).contains(&(g % 10)) && !(12..=14).contains(&(g % 100));
        out.push(if few_form { few } else { many }.into());
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "procent",
        '$' => "dolarów",
        '€' => "euro",
        '£' => "funtów",
        '&' => "i",
        '+' => "plus",
        '@' => "małpa",
        '°' => "stopni",
        '=' => "równa się",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn basics() {
        for (w, p) in [
            // digraphs and the soft series
            ("szkoła", "ʃkˈɔwa"),
            ("czas", "tʃˈas"),
            ("rzeka", "ʒˈɛka"),
            ("dżem", "dʒˈɛm"),
            ("dzień", "dʑˈɛɲ"),
            ("ciocia", "tɕˈɔtɕa"),
            ("siano", "ɕˈanɔ"),
            ("ziemia", "ʑˈɛmja"),
            ("miasto", "mʲˈastɔ"),
            ("niebo", "ɲʲˈɛbɔ"),
            ("koń", "kˈɔɲ"),
            ("chleb", "xlˈɛp"),
            ("Chiny", "çˈinɨ"),
            // palatalized labials and velars
            ("pies", "pʲˈɛs"),
            ("bieg", "bjˈɛk"),
            ("kiedy", "kʲˈɛdɨ"),
            ("drugi", "drˈuɡʲi"),
            ("radio", "rˈadjɔ"),
            ("Anglii", "ˈaŋɡlji"),
            // nasal vowels
            ("będę", "bˈɛndɛ"),
            ("będą", "bˈɛndɔ̃"),
            ("ząb", "zˈɔmp"),
            ("pięć", "pʲˈɛɲtɕ"),
            ("ręka", "rˈɛŋka"),
            ("mąż", "mˈɔ̃ʃ"),
            ("wziął", "vʑˈɔw"),
            ("tysiąc", "tˈɨɕɔnts"),
            // voicing
            ("wszystko", "fʃˈɨstkɔ"),
            ("twój", "tfˈuj"),
            ("przez", "pʃˈɛs"),
            ("trzeba", "tʃˈɛba"),
            ("jakże", "jˈaɡʒɛ"),
            ("prośba", "prˈɔʑba"),
            ("trwać", "trfˈatɕ"),
            ("bóg", "bˈuk"),
            ("bardzo", "bˈardzɔ"),
            ("zamarzać", "zamˈarzatɕ"),
            // vowels, ł, diphthongs
            ("stół", "stˈuw"),
            ("auto", "ˈawtɔ"),
            ("nauka", "nˈaukˌa"),
            ("fizyce", "fˈizɨtsˌɛ"),
            ("koncie", "kˈɔɲtɕɛ"),
            ("część", "tʃˈɛɲɕtɕ"),
            ("w", "v"),
            ("pięćdziesiąt", "pʲɛɲdʑˈɛɕɔnt"),
            ("sześćdziesiąt", "ʃɛʑdʑˈɛɕɔnt"),
            ("weekend", "wˈikɛnt"),
            ("nr", "nˈumɛr"),
            ("pizza", "pˈitstsa"),
            ("klient", "klˈiɛnt"),
            ("r", "ˈɛr"),
            ("biało-czerwony", "bjˈawɔ tʃɛrvˈɔnɨ"),
            ("muzeum", "muzˈɛum"),
            ("Europa", "ɛwrˈɔpa"),
            // stress
            ("matematyka", "mˌatɛmˈatɨkˌa"),
            ("robiliśmy", "rɔbʲˈiliɕmˌɨ"),
            ("robilibyśmy", "rɔbʲˈilibˌɨɕmɨ"),
            ("uniwersytet", "ˌuɲivˈɛrsɨtˌɛt"),
            ("jedenaście", "jˌɛdɛnˈaɕtɕɛ"),
            ("się", "ɕɛ"),
            ("nie", "ɲʲɛ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn sandhi() {
        use crate::Out;
        let mk = |src: &str| Out { ph: word(src), punct: String::new(), pause_after: false, src: src.to_string() };
        let mut outs = vec![mk("w"), mk("szkole"), mk("jak"), mk("dobrze"), mk("bez"), mk("ciebie"), mk("w"), mk("domu")];
        super::join(&mut outs);
        let got: Vec<&str> = outs.iter().map(|o| o.ph.as_str()).collect();
        assert_eq!(got, ["f", "ʃkˈɔlɛ", "jˈaɡ", "dˈɔbʒɛ", "bˈɛs", "tɕˈɛbjɛ", "v", "dˈɔmu"]);
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(21).join(" "), "dwadzieścia jeden");
        assert_eq!(cardinal(1000).join(" "), "tysiąc");
        assert_eq!(cardinal(2000).join(" "), "dwa tysiące");
        assert_eq!(cardinal(5000).join(" "), "pięć tysięcy");
        assert_eq!(cardinal(12000).join(" "), "dwanaście tysięcy");
        assert_eq!(cardinal(1_000_000).join(" "), "milion");
        assert_eq!(cardinal(3_000_000).join(" "), "trzy miliony");
        assert_eq!(cardinal(1999).join(" "), "tysiąc dziewięćset dziewięćdziesiąt dziewięć");
    }
}
