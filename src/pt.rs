//! Brazilian Portuguese grapheme-to-phoneme rules, espeak-style (pt-br) notation.
//!
//! Rules only: orthographic stress (written accents, default penultimate, final
//! stress for words ending in r/l/z/i/u/ão...), nasal vowels and diphthongs,
//! palatalized t/d before i, l-vocalization in the coda, the r-sounds, x, and the
//! open/closed quality of stressed e/o. The last one is not predictable from
//! spelling, so stressed e/o without a written accent use ending statistics
//! derived from Wiktionary (`data/pt_open.tsv`, built by `tools/pt_open.py`).
//!
//! espeak's conventions that we follow: final unstressed a/e/o are `æ y ʊ`,
//! nasal a/e/o/u are `ɐ̃ eɪ o ũ` + `ŋ`/`m`, `ão`=`ɐ̃ʊ̃`, `ãe`=`ɐ̃y`, `õe`=`õj`,
//! `lh`=`lj`, `nh`=`ɲ`, strong r=`x`, flap=`ɾ`, coda r before a consonant=`ɾə`,
//! final r=`r`, coda l=`ʊ` (after o/u: `w`), `ou`=`ow`, stress marks `ˈ`/`ˌ`
//! immediately before the stressed vowel.

use crate::Out;
use std::collections::HashMap;
use std::sync::OnceLock;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "vírgula";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "be", "se", "de", "e", "ɛfy", "ʒe", "aɡa", "i", "ʒɔtæ", "ka", "ɛly", "eme", "ene", "ɔ", "pe", "ke", "ɛxy", "ɛsy",
    "te", "u", "ve", "dabljʊ", "ʃis", "ipsiloŋ", "ze",
]);

const OPEN_TSV: &str = include_str!("../data/pt_open.tsv");

/// Function words with a fixed (mostly unstressed) espeak form.
const FIXED: &[(&str, &str)] = &[
    ("o", "ʊ"), ("os", "ʊs"), ("a", "a"), ("as", "as"), ("ao", "aʊ"), ("aos", "aʊs"), ("à", "a"), ("às", "as"),
    ("do", "dʊ"), ("dos", "dʊs"), ("da", "da"), ("das", "das"), ("no", "nʊ"), ("nos", "nʊs"), ("na", "na"), ("nas", "nas"),
    ("de", "dʒy"), ("que", "ky"), ("se", "sy"), ("me", "my"), ("te", "tʃy"), ("lhe", "ʎy"), ("lhes", "ʎys"),
    ("e", "i"), ("em", "eɪŋ"), ("um", "ũŋ"), ("com", "koŋ"), ("por", "por"), ("mas", "mas"), ("ou", "ow"),
    ("sou", "sow"), ("foi", "foɪ"), ("fui", "fuɪ"), ("tem", "teɪŋ"), ("são", "sɐ̃ʊ̃"), ("ter", "ter"), ("ser", "ser"),
    ("num", "nũŋ"), ("dum", "dũŋ"), ("lo", "lʊ"), ("la", "la"), ("los", "lʊs"), ("las", "las"), ("vos", "vʊs"),
    ("muito", "mwˈiŋtʊ"), ("muita", "mwˈiŋtæ"), ("muitos", "mwˈiŋtʊs"), ("muitas", "mwˈiŋtæs"),
    ("é", "ˈɛ"), ("és", "ˈɛs"), ("há", "ˈa"), ("ele", "ˌely"), ("eles", "ˌelys"), ("esse", "ˈesi"), ("esses", "ˈesis"),
    ("este", "ˈestʃy"), ("estes", "ˈestʃys"), ("dele", "dˈely"), ("deles", "dˈelys"), ("nele", "nˈely"),
    ("nesse", "nˈesi"), ("nesses", "nˈesis"), ("desse", "dˈesi"), ("desses", "dˈesis"), ("neste", "nˈestʃy"),
    ("nestes", "nˈestʃys"), ("deste", "dˈestʃy"), ("destes", "dˈestʃys"), ("pra", "pra"), ("uns", "ũŋs"),
    ("umas", "ˌumæs"), ("pro", "prʊ"), ("pros", "prʊs"), ("pras", "pras"), ("porque", "pˌuɾəky"), ("porquê", "puɾəkˈe"),
    ("somente", "sˌɔmˈeɪŋtʃy"), ("vc", "vosˈe"), ("vcs", "vosˈes"), ("pq", "pˌuɾəky"), ("tb", "tɐ̃mbˈeɪŋ"),
    ("tbm", "tɐ̃mbˈeɪŋ"), ("mt", "mwˈiŋtʊ"), ("mto", "mwˈiŋtʊ"), ("blz", "belˈezæ"), ("sozinho", "sɔzˈiɲʊ"),
    ("sozinha", "sɔzˈiɲæ"), ("sozinhos", "sɔzˈiɲʊs"), ("sozinhas", "sɔzˈiɲæs"), ("novinha", "nɔvˈiɲæ"),
    ("novinho", "nɔvˈiɲʊ"), ("live", "lˈaɪvi"), ("black", "blˈɛki"), ("story", "istˈɔɾi"), ("hall", "xˈɔw"),
    ("light", "lˈaɪtʃi"), ("night", "nˈaɪtʃi"), ("designer", "dezˈaɪner"), ("call", "kˈɔw"), ("full", "fˈuw"),
    ("single", "sˈiŋɡʊ"), ("country", "kˈɐ̃ŋtri"), ("trailer", "trˈeɪler"), ("website", "wɛbisˈaɪtʃi"),
    ("hit", "xˈitʃi"), ("tour", "tˈur"), ("impeachment", "ˌimpˈitʃimeɪŋtʃi"), ("performance", "pˌeɾəfˈɔɾəmɐ̃ŋsi"),
    ("hollywood", "xˈɔliwudʒi"), ("washington", "wˈɔʃiŋtoŋ"), ("walter", "vˈaʊter"), ("william", "wˈiliɐ̃"),
    ("taylor", "tˈeɪlor"), ("kevin", "kˈɛviŋ"), ("jason", "ʒˈeɪzoŋ"), ("boston", "bˈɔstoŋ"), ("miami", "maɪˈɐ̃mi"),
    ("john", "dʒˈoŋ"), ("jack", "dʒˈɛki"), ("bob", "bˈɔbi"), ("make", "mˈeɪki"), ("record", "xˈɛkoɾədʒi"),
    ("alguma", "aʊɡˈumæ"), ("algumas", "aʊɡˈumæs"), ("nenhuma", "neɲˈumæ"), ("nenhumas", "neɲˈumæs"),
    ("big", "bˈiɡi"), ("fitness", "fˈitʃinɛs"), ("delivery", "delˈiveɾi"), ("smart", "izmˈaɾətʃi"), ("stock", "istˈɔki"),
    ("test", "tˈɛstʃi"), ("free", "frˈi"), ("fashion", "fˈɛʃoŋ"), ("hotel", "otˈɛʊ"), ("hotéis", "otˈɛɪs"),
    ("ex", "ˈes"), ("dez", "dˈɛs"), ("internet", "ˌiŋteɾənˈɛtʃi"), ("ideia", "ˌidˈɛɪæ"), ("ideias", "ˌidˈɛɪæs"),
    // frequent loanwords (English spelling)
    ("site", "sˈaɪtʃi"), ("sites", "sˈaɪtʃis"), ("online", "ˌoŋlˈaɪni"), ("blog", "blˈɔɡi"), ("blogs", "blˈɔɡis"),
    ("web", "wˈɛbi"), ("rock", "xˈɔki"), ("facebook", "fˌeɪsibˈuki"), ("google", "ɡˈuɡʊ"), ("software", "sˈɔftʃiwɛr"),
    ("hardware", "xˈaɾədʒiwɛr"), ("show", "ʃˈow"), ("shows", "ʃˈows"), ("link", "lˈiŋki"), ("links", "lˈiŋkis"),
    ("email", "imˈeɪʊ"), ("emails", "imˈeɪʊs"), ("app", "ˈɛpi"), ("apps", "ˈɛpis"), ("design", "dʒizˈaɪni"),
    ("download", "daʊnlˈowdʒi"), ("layout", "leɪˈaʊtʃi"), ("smartphone", "ˌizmaɾətʃifˈoni"), ("whatsapp", "watʃizˈapi"),
    ("youtube", "jutˈubi"), ("instagram", "ˌiŋstaɡrˈɐ̃"), ("twitter", "twˈiter"), ("windows", "wˈiŋdows"),
    ("iphone", "aɪfˈoni"), ("microsoft", "maɪkrosˈɔftʃi"), ("apple", "ˈɛpʊ"), ("netflix", "nˈɛtfliks"),
    ("game", "ɡˈeɪmi"), ("games", "ɡˈeɪmis"), ("shopping", "ʃˈɔpiŋ"), ("marketing", "mˈaɾəketʃiŋ"),
    ("ranking", "xˈɐ̃ŋkiŋ"), ("look", "lˈuki"), ("stop", "istˈɔpi"), ("club", "klˈubi"), ("rap", "xˈɛpi"),
    ("funk", "fˈɐ̃ŋki"), ("skate", "iskˈeɪtʃi"), ("surf", "sˈuɾəfi"), ("pub", "pˈabi"), ("sexy", "sˈɛksi"),
    ("ok", "okˈeɪ"), ("notebook", "nˌowtʃibˈuki"), ("tablet", "tˈabletʃi"), ("mouse", "mˈaʊzi"), ("wifi", "waɪfˈaɪ"),
    ("login", "lɔɡˈiŋ"), ("post", "pˈostʃi"), ("posts", "pˈostʃis"), ("tweet", "twˈitʃi"), ("hashtag", "xɛʃtˈɛɡi"),
    ("selfie", "sˈɛʊfi"), ("feed", "fˈidʒi"), ("like", "lˈaɪki"), ("likes", "lˈaɪkis"), ("john", "ʒˈoŋ"),
    ("harry", "xˈɛɾi"), ("trump", "trˈɐ̃mpi"), ("gay", "ɡˈeɪ"), ("jazz", "ʒˈɛs"), ("pop", "pˈɔpi"), ("chat", "ʃˈatʃi"),
    ("new", "nˈiʊ"), ("news", "nˈiʊs"), ("fake", "fˈeɪki"), ("top", "tˈɔpi"), ("flash", "flˈɛʃi"), ("drive", "drˈaɪvi"),
    ("player", "plˈeɪer"), ("bike", "bˈaɪki"), ("chip", "ʃˈipi"), ("clip", "klˈipi"), ("kit", "kˈitʃi"),
    ("pizza", "pˈitsæ"), ("hobby", "ˈɔbi"), ("jeans", "ʒˈiŋs"), ("york", "ɪˈɔɾəki"), ("hip", "xˈipi"), ("hop", "xˈɔpi"),
];
/// Frequent words whose stressed e/o the ending statistics get wrong.
const OPEN: &[&str] = &[
    "agora", "maior", "maiores", "pior", "piores", "menor", "menores", "foto", "fotos", "merda", "mulher", "mulheres",
    "belo", "nossa", "nosso", "nossas", "nossos", "posso", "vosso", "vossa", "pedra", "pedras", "festa", "guerra",
    "terra", "perto", "certo", "certa", "aberto", "aberta", "abertos", "abertas", "coberto", "morte", "forte", "fortes",
    "sorte", "corpos", "porta", "portas", "hora", "horas", "fora", "nove", "novamente", "seria", "série", "senhora",
    "senhoras", "novos", "novas", "nova", "querem", "podem", "perde", "inverno", "regra", "regras", "pele", "sozinho",
    "sozinha", "sozinhos", "sozinhas", "novinha", "novinho", "modos", "inveja", "evangelho", "arquiteto", "redor",
    "major", "dezenove", "nojo", "jogos", "olhos", "ovos", "fogos", "porcos", "esforços", "esposos", "postos",
    "logo", "olha", "olhe", "olhem", "pega", "pegam", "seca", "cerca", "sincera", "sincero", "sinceros", "sinceras",
    "quiser", "puder", "fizer", "souber", "tiver", "estiver", "houver", "trouxer", "vier", "der", "couber", "disser",
    "quiserem", "puderem", "fizerem", "souberem", "tiverem", "estiverem", "houverem", "vierem", "derem", "disserem",
    "ela", "elas", "essa", "esta", "aquela", "nela", "dela", "nessa", "nesta", "dessa", "desta",
    "sete", "dezessete", "pé", "velho", "velha", "velhos", "velhas", "escola", "hotel", "papel", "sol",
    "jornal", "metro", "zero", "bola", "cola", "mola", "pobre", "nobre", "obra", "obras", "lógica", "jovem",
];
const CLOSED: &[&str] = &[
    "hoje", "senhor", "senhores", "cabeça", "teve", "esteve", "preto", "preta", "pretos", "pretas", "pelo",
    "pela", "pelos", "pelas", "povo", "novo", "ovo", "medo", "cedo", "dedo", "começo", "almoço", "esforço",
    "esquerda", "esquerdo", "perda", "sopa", "grosso", "osso", "acordo", "acordos", "sorvete", "escolher", "recolher",
    "corpo", "pessoa", "pessoas", "gol", "for", "forem", "fomos", "foram", "jogo", "olho", "fogo", "porco", "posto",
    "fosse", "fossem", "força", "forças", "interesse", "interesses", "preso", "presa", "presos", "presas",
    "pescoço", "poço", "esposo", "esposa", "mesmo", "mesma", "mesmos", "mesmas", "esse", "este", "aquele", "nome",
    "fome", "homem", "homens", "boca", "sono", "todo", "toda", "todos", "todas", "boa", "boas", "gosto", "outro",
    "outra", "outros", "outras", "pouco", "pouca", "poucos", "poucas", "lobo", "bolo", "soco", "coco", "coisa",
    "vez", "vezes", "mês", "três", "inglês", "você", "vocês", "dele", "deles", "nele", "deste", "desse", "neste",
    "nesse", "verde", "perdeu", "perder", "sede", "rede", "redes", "selo", "cabelo", "modelo", "modelos", "sobre",
];
/// Function words espeak reads with a secondary stress only.
const SECONDARY: &[&str] = &[
    "como", "para", "até", "está", "estou", "estão", "estamos", "era", "eram", "foram", "onde", "desde", "uma", "numa",
    "duma", "tenho", "temos", "teve", "seria",
];

/// qu/gu before e/i where the u is pronounced (kw/ɡw); the word must start with one of these.
const QU_W: &[&str] = &[
    "tranquil", "frequen", "frequên", "sequen", "sequên", "consequen", "consequên", "subsequen", "eloquen", "eloquên",
    "cinquent", "quinquen", "quinquên", "equestr", "delinquen", "delinquên", "aquíf", "ubíqu", "iníqu", "obliqu",
    "sequel", "equidad", "liquid", "líquid", "quinquag", "equidist", "equilát",
];
const GU_W: &[&str] = &[
    "aguent", "linguiç", "linguíst", "linguist", "pinguim", "argui", "redargui", "ambigui", "contigui", "exigui",
    "sagui", "unguent", "sanguín", "sanguin", "consanguin", "linguet", "bilingu", "trilingu", "multilingu",
];

#[derive(Clone, Copy, PartialEq)]
enum Stress {
    Primary,
    Secondary,
    None,
}

#[derive(Clone, Debug)]
struct Seg {
    ph: String,
    nucleus: bool,
    v: char,
    /// 0 none, 1 acute (open), 2 circumflex (closed), 3 tilde
    acc: u8,
    /// second letter of a falling/nasal diphthong, '\0' if none
    glide: char,
    /// nasal coda after this vowel: '\0', 'm', 'ŋ', 'F' (word-final m/n)
    nasal: char,
    /// an m/n/nh onset follows
    nasal_onset: bool,
    /// nh follows (a is nasal even unstressed: manhã)
    nh: bool,
    /// epenthetic vowel (not counted for stress placement)
    epen: bool,
    pos: usize,
    len: usize,
}

fn cons(ph: &str) -> Seg {
    Seg { ph: ph.to_string(), nucleus: false, v: '\0', acc: 0, glide: '\0', nasal: '\0', nasal_onset: false, nh: false, epen: false, pos: 0, len: 0 }
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'â' | 'ê' | 'ô' | 'ã' | 'õ' | 'ü')
}
fn base(c: char) -> char {
    match c {
        'á' | 'â' | 'ã' => 'a',
        'é' | 'ê' => 'e',
        'í' => 'i',
        'ó' | 'ô' | 'õ' => 'o',
        'ú' | 'ü' => 'u',
        _ => c,
    }
}
fn acc_of(c: char) -> u8 {
    match c {
        'á' | 'é' | 'í' | 'ó' | 'ú' => 1,
        'â' | 'ê' | 'ô' => 2,
        'ã' | 'õ' => 3,
        _ => 0,
    }
}
fn norm_char(c: char) -> char {
    match c {
        'y' => 'i',
        'w' => 'u',
        'à' => 'a',
        'ï' | 'ì' => 'i',
        'è' => 'é',
        'ò' => 'ó',
        'ù' => 'u',
        'ñ' => 'ɲ',
        _ => c,
    }
}

fn open_table() -> &'static HashMap<&'static str, f32> {
    static T: OnceLock<HashMap<&'static str, f32>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m = HashMap::new();
        for line in OPEN_TSV.lines() {
            let mut f = line.split('\t');
            if let (Some(k), Some(p)) = (f.next(), f.next()) {
                if let Ok(p) = p.parse() {
                    m.insert(k, p);
                }
            }
        }
        m
    })
}

/// Is the stressed e/o at `pos` open? Longest matching ending wins, with two,
/// one or no letters of left context.
fn is_open(lw: &str, c: &[char], pos: usize) -> bool {
    if OPEN.contains(&lw) {
        return true;
    }
    if CLOSED.contains(&lw) {
        return false;
    }
    let t = open_table();
    let tail = &c[pos..];
    let prev = if pos > 0 { c[pos - 1] } else { '#' };
    let prev2: String = if pos > 1 { c[pos - 2..pos].iter().collect() } else { format!("#{prev}") };
    let full: String = tail.iter().collect();
    let mut k = tail.len();
    while k >= 1 {
        let key = if k == tail.len() { format!("{full}#") } else { tail[..k].iter().collect::<String>() };
        if k >= 2 || k == tail.len() {
            for cand in [format!("{prev2}|{key}"), format!("{prev}|{key}"), key] {
                if let Some(&p) = t.get(cand.as_str()) {
                    return p >= 0.5;
                }
            }
        }
        k -= 1;
    }
    t.get(format!("{}*", c[pos]).as_str()).map_or(false, |&p| p >= 0.5)
}

/// Length of the orthographic nucleus starting at a vowel letter `i`, and its glide letter.
fn nucleus_at(c: &[char], i: usize) -> (usize, char) {
    let n = c.len();
    let ch = c[i];
    let at = |k: usize| if k < n { c[k] } else { '#' };
    let d = at(i + 1);
    let e = at(i + 2);
    if d == '#' {
        return (1, '\0');
    }
    if matches!(d, 'i' | 'u') && base(ch) != d {
        // a-iu / u-iu / a-ui: the i/u begins a final diphthong of its own
        let own = matches!(e, 'i' | 'u') && e != d && (i + 3 == n || !is_v(at(i + 3)));
        // sair, raiz, raul, ruim, ruins, ainda, rainha, reunir
        let hiatus = (matches!(e, 'r' | 'z' | 'l') && i + 3 == n)
            || (matches!(e, 'm' | 'n') && (i + 3 == n || (at(i + 3) == 's' && i + 4 == n) || !is_v(at(i + 3))))
            || (ch == 'e' && d == 'u' && i == 1 && c[0] == 'r' && matches!(e, 'n' | 't'));
        if !own && !hiatus {
            return (2, d);
        }
        return (1, '\0');
    }
    if (ch == 'ã' && matches!(d, 'e' | 'o')) || (ch == 'õ' && d == 'e') {
        return (2, d);
    }
    (1, '\0')
}

/// What does this x sound like? (ʃ, ks, s, z)
fn x_kind(c: &[char], i: usize) -> &'static str {
    let n = c.len();
    let w: String = c.iter().map(|&x| base(x)).collect();
    let before: String = c[..i].iter().map(|&x| base(x)).collect();
    let next = if i + 1 < n { c[i + 1] } else { '#' };
    let prev = if i > 0 { c[i - 1] } else { '#' };
    if i == 0 {
        return "ʃ";
    }
    if next == '#' {
        return if w == "ex" { "s" } else { "ks" };
    }
    if !is_v(next) {
        return "s";
    }
    // initial ex- (+ a few prefixes) before a vowel: z
    for p in ["e", "ine", "ree", "coe", "dese", "he"] {
        if before == p {
            return if p == "he" { "ks" } else { "z" };
        }
    }
    for s in ["pro", "ma", "au"] {
        if before.ends_with(s) && !(s == "ma" && (before != "ma" || c[i - 1] != 'á')) {
            return "s"; // próximo, máximo, auxílio
        }
    }
    // after a diphthong (caixa, peixe, frouxo) or n (enxergar): ʃ
    if matches!(prev, 'i' | 'u') && i >= 2 && is_v(c[i - 2]) {
        return if before.ends_with("trou") { "s" } else { "ʃ" };
    }
    if prev == 'n' {
        return "ʃ";
    }
    // stems (the letters before the x) read with ks; roxo/coxa/lixo/taxa keep ʃ
    for s in [
        "fi", "ne", "fle", "ple", "le", "de", "se", "ta", "ma", "to", "bo", "do", "sa", "gala", "sinta", "flu", "pi",
        "mi", "ni", "proli", "asfi", "conve", "te", "o", "mar",
    ] {
        if before.ends_with(s) {
            if s == "te" && !(before == "te" || before == "late") {
                continue;
            }
            if (s == "de" || s == "le" || s == "ta") && !matches!(c[i - 1], 'é' | 'á') && before.len() <= 3 && !(s == "ta" && next == 'i') {
                continue; // mexer/relaxar/taxa are ʃ, léxico/táxi/galáxia are ks
            }
            if s == "o" && before != "o" {
                continue; // roxo keeps ʃ, oxigênio is ks
            }
            return "ks";
        }
    }
    if before == "o" || before == "a" {
        return "ks"; // oxigênio, axila
    }
    "ʃ"
}

/// Phonemize one lower-case word (hyphen/apostrophe parts are read one by one).
pub fn word(lw: &str) -> String {
    let parts: Vec<&str> = lw.split(['-', '\'', '’']).filter(|p| p.chars().any(char::is_alphabetic)).collect();
    let mut out = String::new();
    for (k, p) in parts.iter().enumerate() {
        // d'água, n'água: an elided single consonant is read as that consonant, not its letter name
        let elided = p.chars().count() == 1 && k + 1 < parts.len() && !p.chars().any(is_v) && lw.contains(['\'', '’']);
        if elided {
            out.push_str(match *p {
                "d" => "d",
                "n" => "n",
                "l" => "l",
                "m" => "m",
                "s" | "c" => "s",
                "t" => "t",
                _ => "",
            });
        } else {
            out.push_str(&word_one(p));
        }
    }
    out
}

fn word_one(w: &str) -> String {
    let raw: String = w.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect();
    if let Some((_, p)) = FIXED.iter().find(|(k, _)| *k == raw) {
        return p.to_string();
    }
    let lw: String = raw.chars().map(norm_char).collect();
    if lw.is_empty() {
        return String::new();
    }
    // single consonants and vowel-less tokens (tv, mp3) are spelled out
    if !lw.chars().any(is_v) {
        if let Some(sp) = crate::spell::spell(&lw, crate::Lang::Pt) {
            return sp;
        }
    }
    // adverbs in -mente keep the adjective's open/closed vowel (nova -> nɔvamente)
    if let Some(stem) = lw.strip_suffix("mente") {
        if stem.chars().filter(|&c| is_v(c)).count() >= 2 && stem != "sim" {
            let mut stem = stem.to_string();
            if let Some(adj) = stem.strip_suffix("vel") {
                if adj.ends_with('a') || adj.ends_with('i') {
                    let acc = if adj.ends_with('a') { "á" } else { "í" };
                    stem = format!("{}{acc}vel", &adj[..adj.len() - 1]);
                }
            } else if let Some(adj) = stem.strip_suffix("ia") {
                // séria-mente, própria-mente, diária-mente: the accent is dropped in writing
                let ch: Vec<char> = adj.chars().collect();
                if ch.len() >= 2 && !is_v(ch[ch.len() - 1]) && ch.iter().filter(|&&x| is_v(x)).count() >= 1 {
                    if let Some(k) = ch.iter().rposition(|&x| is_v(x)) {
                        let acc = match ch[k] {
                            'e' => 'é', 'a' => 'á', 'o' => 'ó', 'i' => 'í', 'u' => 'ú', x => x,
                        };
                        let mut ch2 = ch.clone();
                        ch2[k] = acc;
                        stem = format!("{}ia", ch2.iter().collect::<String>());
                    }
                }
            }
            let (_, quality) = word_full(&stem, Stress::Primary, None);
            return word_full(&lw, Stress::Primary, Some(quality)).0;
        }
    }
    let st = if SECONDARY.contains(&lw.as_str()) { Stress::Secondary } else { Stress::Primary };
    word_inner(&lw, st)
}

fn word_inner(lw: &str, stress: Stress) -> String {
    word_full(lw, stress, None).0
}

/// Phonemize; also returns (letter index, is open) of the stressed vowel. `force`
/// fixes the quality of the vowel at a letter index (the adjective inside -mente).
fn word_full(lw: &str, stress: Stress, force: Option<(usize, bool)>) -> (String, (usize, bool)) {
    let c: Vec<char> = lw.chars().collect();
    let n = c.len();
    let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { c[i as usize] } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 2);
    let mut i = 0usize;
    while i < n {
        let ch = c[i];
        let nx = at(i as isize + 1);
        let nx2 = at(i as isize + 2);
        let pv = at(i as isize - 1);
        if is_v(ch) {
            let (len, glide) = nucleus_at(&c, i);
            let mut s = cons("");
            s.nucleus = true;
            s.v = base(ch);
            s.acc = acc_of(ch);
            s.glide = glide;
            s.pos = i;
            s.len = len;
            segs.push(s);
            i += len;
            continue;
        }
        let next_front = matches!(nx, 'e' | 'i' | 'é' | 'ê' | 'í');
        let next_v = is_v(nx);
        let mark_nasal_onset = |segs: &mut Vec<Seg>| {
            if let Some(l) = segs.last_mut() {
                if l.nucleus {
                    l.nasal_onset = true;
                }
            }
        };
        match ch {
            'b' => segs.push(cons("b")),
            'p' => segs.push(cons("p")),
            'f' => segs.push(cons("f")),
            'v' => segs.push(cons("v")),
            'k' => segs.push(cons("k")),
            't' | 'd' => {
                segs.push(cons(if ch == 't' { "t" } else { "d" }));
                let cluster = !next_v && nx != '#' && !matches!(nx, 'l' | 'r' | 'h' | 's' | 'z' | 'c' | 'x' | 'ɲ') && nx != ch;
                if cluster && i > 0 {
                    let mut e = cons("i");
                    e.nucleus = true;
                    e.v = 'i';
                    e.epen = true;
                    e.pos = i;
                    segs.push(e);
                }
            }
            'ɲ' => segs.push(cons("ɲ")),
            'c' => {
                if nx == 'h' {
                    segs.push(cons("ʃ"));
                    i += 1;
                } else if next_front {
                    segs.push(cons("s"));
                } else if nx == 'k' || nx == 'c' && !matches!(nx2, 'e' | 'i' | 'é' | 'ê' | 'í') {
                    // ck, cc+a: one k
                } else {
                    segs.push(cons("k"));
                }
            }
            'ç' => segs.push(cons("s")),
            's' => {
                if nx == 's' {
                    segs.push(cons("s"));
                    i += 1;
                } else if nx == 'h' {
                    segs.push(cons("ʃ"));
                    i += 1;
                } else if nx == 'ç' || (nx == 'c' && matches!(nx2, 'e' | 'i' | 'é' | 'ê' | 'í')) {
                    // sç, sce, sci: one s (from the ç / c)
                } else if i > 0 && is_v(pv) && next_v {
                    segs.push(cons("z"));
                } else if matches!(nx, 'b' | 'd' | 'g' | 'v' | 'm' | 'n' | 'l' | 'r' | 'z' | 'j') {
                    segs.push(cons("z"));
                } else if pv == 'n' && next_v && i == 4 && c[..4].iter().map(|&x| base(x)).collect::<String>() == "tran" {
                    segs.push(cons("z"));
                } else {
                    segs.push(cons("s"));
                }
            }
            'z' => segs.push(cons(if nx == '#' { "s" } else { "z" })),
            'x' => {
                let k = x_kind(&c, i);
                if k == "ks" {
                    segs.push(cons("k"));
                    segs.push(cons("s"));
                } else {
                    segs.push(cons(k));
                }
                if nx == 'c' && matches!(nx2, 'e' | 'i' | 'é' | 'ê' | 'í') || nx == 's' {
                    i += 1; // exceto, exsudar: one s
                }
            }
            'g' => {
                if next_front {
                    segs.push(cons("ʒ"));
                } else if nx == 'u' && matches!(nx2, 'e' | 'i' | 'é' | 'ê' | 'í') {
                    segs.push(cons("ɡ"));
                    if GU_W.iter().any(|p| lw.starts_with(p)) {
                        segs.push(cons("w"));
                    }
                    i += 1;
                } else if nx == 'ü' || (nx == 'u' && is_v(nx2)) {
                    segs.push(cons("ɡ"));
                    segs.push(cons("w"));
                    i += 1;
                } else {
                    segs.push(cons("ɡ"));
                }
            }
            'q' => {
                segs.push(cons("k"));
                if nx == 'u' || nx == 'ü' {
                    let front = matches!(nx2, 'e' | 'i' | 'é' | 'ê' | 'í');
                    if nx == 'ü' || !front || QU_W.iter().any(|p| lw.starts_with(p)) {
                        segs.push(cons("w"));
                    }
                    i += 1;
                }
            }
            'j' => segs.push(cons("ʒ")),
            'h' => {}
            'l' => {
                if nx == 'h' {
                    segs.push(cons("lj"));
                    i += 1;
                } else if nx == 'l' {
                    segs.push(cons("l"));
                    i += 1;
                } else if !next_v && nx != 'h' {
                    segs.push(cons(if matches!(base(pv), 'o' | 'u') { "w" } else { "ʊ" }));
                } else {
                    segs.push(cons("l"));
                }
            }
            'r' => {
                if nx == 'r' {
                    segs.push(cons("x"));
                    i += 1;
                } else if i == 0 || matches!(pv, 'n' | 'l' | 's' | 'z') {
                    segs.push(cons("x"));
                } else if !is_v(pv) {
                    segs.push(cons("r"));
                } else if nx == '#' {
                    segs.push(cons("r"));
                } else if !next_v && nx != 'h' {
                    segs.push(cons("ɾə"));
                } else {
                    segs.push(cons("ɾ"));
                }
            }
            'm' | 'n' => {
                if ch == 'n' && nx == 'h' {
                    mark_nasal_onset(&mut segs);
                    if let Some(l) = segs.last_mut() {
                        l.nh = true;
                    }
                    segs.push(cons("ɲ"));
                    i += 1;
                } else if next_v {
                    mark_nasal_onset(&mut segs);
                    segs.push(cons(if ch == 'm' { "m" } else { "n" }));
                } else if segs.last().map_or(false, |s| s.nucleus) {
                    let l = segs.last_mut().unwrap();
                    l.nasal = if ch == 'm' && (nx == '#' || (nx == 's' && nx2 == '#')) {
                        'F'
                    } else if matches!(nx, 'p' | 'b') || (ch == 'm' && nx == 'n') {
                        'm'
                    } else {
                        'ŋ'
                    };
                } else {
                    segs.push(cons(if ch == 'm' { "m" } else { "n" }));
                }
            }
            _ => {}
        }
        i += 1;
    }
    // ---- stress ----
    let nuclei = |segs: &[Seg]| -> Vec<usize> { segs.iter().enumerate().filter(|(_, s)| s.nucleus).map(|(i, _)| i).collect() };
    let nk: Vec<usize> = nuclei(&segs).into_iter().filter(|&i| !segs[i].epen).collect();
    if nk.is_empty() {
        return (segs.iter().map(|s| s.ph.as_str()).collect(), (0, false));
    }
    let primary = nk.iter().position(|&ix| matches!(segs[ix].acc, 1 | 2)).or_else(|| {
        nk.iter().position(|&ix| {
            let s = &segs[ix];
            s.acc == 3 && !c[s.pos + s.len..].starts_with(&['z', 'i', 'n', 'h'])
        })
    });
    let primary = primary.unwrap_or_else(|| {
        if nk.len() == 1 {
            return 0;
        }
        let last = &segs[nk[nk.len() - 1]];
        let tail = &c[last.pos + last.len..];
        let lv = c[last.pos + last.len - 1];
        let penult = ((tail.is_empty() || tail == ['s']) && matches!(lv, 'a' | 'e' | 'o'))
            || ((tail == ['m'] || tail == ['n', 's']) && last.len == 1 && matches!(lv, 'a' | 'e'));
        if penult { nk.len() - 2 } else { nk.len() - 1 }
    });
    // post-tonic i/u before a vowel become glides (história -> ɔɾjæ); so does i before ê
    let pidx = nk[primary];
    if pidx >= 1 && segs[pidx - 1].nucleus && segs[pidx - 1].v == 'i' && segs[pidx - 1].acc == 0 && segs[pidx].v == 'e' && segs[pidx].acc == 2 {
        segs[pidx - 1] = cons("j");
    }
    for k in (pidx + 1)..segs.len() {
        let s = &segs[k];
        if s.nucleus && s.acc == 0 && s.glide == '\0' && s.nasal == '\0' && matches!(s.v, 'i' | 'u' | 'e') {
            if let Some(nxt) = segs.get(k + 1) {
                if nxt.nucleus && matches!(nxt.v, 'a' | 'e' | 'o') && nxt.acc != 3 && !(s.v == 'e' && nxt.v == 'e') {
                    let g = if s.v == 'u' { "w" } else { "j" };
                    segs[k] = cons(g);
                }
            }
        }
    }
    let nk = nuclei(&segs);
    let total = nk.len();
    let primary = nk.iter().position(|&x| x == pidx).unwrap_or(0);
    // ---- vowel rendering ----
    let last_idx = nk[total - 1];
    let final_open = segs[last_idx + 1..].iter().all(|s| s.ph == "s");
    for (k, &ix) in nk.iter().enumerate() {
        let stressed = k == primary && stress != Stress::None;
        let post = k > primary;
        let fin = ix == last_idx && final_open;
        let s = &segs[ix];
        let forced = force.filter(|&(p, _)| p == s.pos && matches!(s.v, 'e' | 'o'));
        let open = match (s.acc, forced) {
            (_, Some((_, o))) => o,
            (1, _) => true,
            (2, _) => false,
            _ => !s.nasal_onset && s.nasal == '\0' && stressed && matches!(s.v, 'e' | 'o') && is_open(lw, &c, s.pos),
        };
        let qual = stressed || s.acc != 0 || forced.is_some();
        let mut out = String::new();
        if s.acc == 3 {
            out.push_str(match (s.v, s.glide) {
                ('a', 'o') => "ɐ̃ʊ̃",
                ('a', 'e') => "ɐ̃y",
                ('a', 'i') => "ɐ̃ɪ",
                ('o', 'e') => "õj",
                ('o', _) => "õ",
                _ => "ɐ̃",
            });
        } else if s.nasal != '\0' {
            let core = match s.v {
                'a' => "ɐ̃",
                'e' => "eɪ",
                'i' => "i",
                'o' => "o",
                _ => "ũ",
            };
            if s.nasal == 'F' && s.v == 'a' {
                out.push_str("ɐ̃ʊ̃");
            } else {
                out.push_str(core);
                out.push(if s.nasal == 'm' { 'm' } else { 'ŋ' });
            }
        } else {
            let red = fin && s.glide == '\0' && !stressed;
            let q = match s.v {
                'a' => {
                    if stressed || (s.nh && s.glide == '\0') {
                        if s.nasal_onset { "ɐ̃" } else { "a" }
                    } else if (s.nasal_onset && s.glide == '\0') || red || (post && (ix != last_idx || final_open)) {
                        "æ"
                    } else {
                        "a"
                    }
                }
                'e' => if qual { if open { "ɛ" } else { "e" } } else if red { "y" } else { "e" },
                'o' => if qual { if open { "ɔ" } else { "o" } } else if red { "ʊ" } else { "o" },
                'i' => "i",
                _ => if stressed && s.nasal_onset { "ũ" } else { "u" },
            };
            out.push_str(q);
            out.push_str(match (s.v, s.glide) {
                (_, 'i') => "ɪ",
                ('o', 'u') => "w",
                (_, 'u') => "ʊ",
                _ => "",
            });
        }
        segs[ix].ph = out;
    }
    // t/d before i (or a j glide): tʃ dʒ; word-final t/d get an epenthetic i (internet)
    for k in 0..segs.len() {
        if segs[k].ph != "t" && segs[k].ph != "d" {
            continue;
        }
        let pal = match segs.get(k + 1) {
            Some(nx) => (nx.nucleus && nx.ph.starts_with(['i', 'y'])) || nx.ph == "j",
            None => true,
        };
        if pal {
            segs[k].ph = if segs[k].ph == "t" { "tʃ".into() } else { "dʒ".into() };
            if k + 1 == segs.len() {
                segs[k].ph.push('i');
            }
        }
    }
    if segs.len() > 1 {
        if let Some(l) = segs.last_mut() {
            if matches!(l.ph.as_str(), "b" | "p" | "k" | "ɡ" | "f" | "v") {
                l.ph.push('i');
            }
        }
    }
    // ---- stress marks ----
    let pidx = nk[primary];
    let mut out = String::with_capacity(lw.len() * 2);
    for (ix, s) in segs.iter().enumerate() {
        match stress {
            Stress::Primary => {
                if ix == pidx {
                    out.push('ˈ');
                }
                if s.nucleus {
                    let k = nk.iter().position(|&x| x == ix).unwrap();
                    if k < primary && k % 2 == 0 && (primary - k >= 2 || (k == 0 && total >= 3)) {
                        out.push('ˌ');
                    }
                }
            }
            Stress::Secondary => {
                if ix == pidx {
                    out.push('ˌ');
                }
            }
            Stress::None => {}
        }
        out.push_str(&s.ph);
    }
    let p = &segs[pidx];
    (out, (p.pos, p.ph.starts_with(['ɛ', 'ɔ'])))
}

/// Sentence pass: a word-final s is voiced before a vowel or voiced consonant
/// (os amigos -> ʊz ˌæmˈiɡʊs, mais ou menos -> mˈaɪz ow).
pub fn join(words: &mut [Out]) {
    for k in 1..words.len() {
        let (a, b) = words.split_at_mut(k);
        let prev = &mut a[k - 1];
        let cur = &b[0];
        if prev.pause_after || prev.ph.is_empty() || cur.ph.is_empty() || !prev.ph.ends_with('s') {
            continue;
        }
        let first = cur.ph.chars().find(|&c| c != 'ˈ' && c != 'ˌ').unwrap_or(' ');
        let voiced = "aeiouɐɛɔæʊɪyõũbdɡvmnlʒzxɾrjwɲ".contains(first);
        if voiced {
            prev.ph.pop();
            prev.ph.push('z');
        }
    }
}

const UNITS: [&str; 20] = [
    "zero", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito", "nove", "dez", "onze", "doze", "treze",
    "quatorze", "quinze", "dezesseis", "dezessete", "dezoito", "dezenove",
];
const TENS: [&str; 10] = ["", "", "vinte", "trinta", "quarenta", "cinquenta", "sessenta", "setenta", "oitenta", "noventa"];
const HUNDREDS: [&str; 10] = [
    "", "cento", "duzentos", "trezentos", "quatrocentos", "quinhentos", "seiscentos", "setecentos", "oitocentos",
    "novecentos",
];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let (h, r) = (n / 100, n % 100);
    if h > 0 {
        if h == 1 && r == 0 {
            out.push("cem".into());
            return;
        }
        out.push(HUNDREDS[h as usize].into());
        if r > 0 {
            out.push("e".into());
        }
    }
    if r >= 20 {
        out.push(TENS[(r / 10) as usize].into());
        if r % 10 > 0 {
            out.push("e".into());
            out.push(UNITS[(r % 10) as usize].into());
        }
    } else if r > 0 || h == 0 {
        out.push(UNITS[r as usize].into());
    }
}

/// Number words (masculine forms).
pub fn cardinal(n: u128) -> Vec<String> {
    if n < 1000 {
        let mut out = Vec::new();
        below_1000(n, &mut out);
        return out;
    }
    const SCALE: [(&str, &str); 5] =
        [("milhão", "milhões"), ("bilhão", "bilhões"), ("trilhão", "trilhões"), ("quatrilhão", "quatrilhões"), ("quintilhão", "quintilhões")];
    // split into groups of three digits
    let mut groups = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push(m % 1000);
        m /= 1000;
    }
    let mut out = Vec::new();
    for gi in (0..groups.len()).rev() {
        let g = groups[gi];
        if g == 0 {
            continue;
        }
        if gi >= 2 {
            let (sg, pl) = SCALE[(gi - 2).min(SCALE.len() - 1)];
            below_1000(g, &mut out);
            out.push(if g == 1 { sg } else { pl }.into());
        } else {
            // "e" before the last non-zero group when it is < 100 or a round hundred
            let last = groups[..gi].iter().all(|&x| x == 0);
            if last && gi + 1 < groups.len() && (g < 100 || g % 100 == 0) {
                out.push("e".into());
            }
            if gi == 1 {
                if g > 1 {
                    below_1000(g, &mut out);
                }
                out.push("mil".into());
            } else {
                below_1000(g, &mut out);
            }
        }
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "por cento",
        '$' => "dólar",
        '€' => "euro",
        '£' => "libra",
        '&' => "e",
        '+' => "mais",
        '@' => "arroba",
        '°' => "graus",
        '=' => "igual",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, join, word, Out};

    #[test]
    fn basics() {
        for (w, p) in [
            ("casa", "kˈazæ"),
            ("tempo", "tˈeɪmpʊ"),
            ("porta", "pˈɔɾətæ"),
            ("cidade", "sˌidˈadʒy"),
            ("tia", "tʃˈiæ"),
            ("não", "nˈɐ̃ʊ̃"),
            ("mãe", "mˈɐ̃y"),
            ("ações", "asˈõjs"),
            ("bem", "bˈeɪŋ"),
            ("também", "tɐ̃mbˈeɪŋ"),
            ("falam", "fˈalɐ̃ʊ̃"),
            ("homem", "ˈomeɪŋ"),
            ("jardim", "ʒaɾədʒˈiŋ"),
            ("comum", "komˈũŋ"),
            ("bom", "bˈoŋ"),
            ("filho", "fˈiljʊ"),
            ("banho", "bˈɐ̃ɲʊ"),
            ("carro", "kˈaxʊ"),
            ("rato", "xˈatʊ"),
            ("honra", "ˈoŋxæ"),
            ("mal", "mˈaʊ"),
            ("brasil", "brazˈiʊ"),
            ("sol", "sˈɔw"),
            ("fácil", "fˈasiʊ"),
            ("caixa", "kˈaɪʃæ"),
            ("exame", "ˌezˈɐ̃my"),
            ("texto", "tˈestʊ"),
            ("táxi", "tˈaksi"),
            ("próximo", "prˈɔsimʊ"),
            ("fixo", "fˈiksʊ"),
            ("chave", "ʃˈavy"),
            ("gente", "ʒˈeɪŋtʃy"),
            ("guerra", "ɡˈɛxæ"),
            ("água", "ˈaɡwæ"),
            ("quase", "kwˈazy"),
            ("quem", "kˈeɪŋ"),
            ("cinquenta", "sˌiŋkwˈeɪŋtæ"),
            ("mesmo", "mˈezmʊ"),
            ("casas", "kˈazæs"),
            ("luz", "lˈus"),
            ("nascer", "nasˈer"),
            ("história", "ˌistˈɔɾjæ"),
            ("dia", "dʒˈiæ"),
            ("sair", "saˈir"),
            ("rainha", "xˌaˈiɲæ"),
            ("ruim", "xuˈiŋ"),
            ("cuidado", "kˌuɪdˈadʊ"),
            ("avó", "avˈɔ"),
            ("avô", "avˈo"),
            ("pé", "pˈɛ"),
            ("você", "vosˈe"),
            ("ela", "ˈɛlæ"),
            ("ele", "ˌely"),
            ("coração", "kˌoɾasˈɐ̃ʊ̃"),
            ("comunicação", "kˌomunˌikasˈɐ̃ʊ̃"),
            ("agora", "ˌaɡˈɔɾæ"),
            ("escola", "ˌeskˈɔlæ"),
            ("importante", "ˌimpoɾətˈɐ̃ŋtʃy"),
            ("de", "dʒy"),
            ("os", "ʊs"),
            ("muito", "mwˈiŋtʊ"),
            ("pouco", "pˈowkʊ"),
            ("primeiro", "prˌimˈeɪɾʊ"),
            ("ideia", "ˌidˈɛɪæ"),
            ("pessoa", "pˌesˈoæ"),
            ("bêbado", "bˈebædʊ"),
            ("órgão", "ˈɔɾəɡɐ̃ʊ̃"),
            ("sobre-humano", "sˈobryˌumˈɐ̃nʊ"),
            ("novamente", "nˌɔvæmˈeɪŋtʃy"),
            ("provavelmente", "prˌovaveʊmˈeɪŋtʃy"),
            ("seriamente", "sˌɛɾiæmˈeɪŋtʃy"),
            ("máximo", "mˈasimʊ"),
            ("maxilar", "mˌaksilˈar"),
            ("peixe", "pˈeɪʃy"),
            ("auxílio", "ˌaʊsˈiljʊ"),
            ("exceto", "ˌesˈɛtʊ"),
            ("enxergar", "ˌeɪŋʃeɾəɡˈar"),
            ("internet", "ˌiŋteɾənˈɛtʃi"),
            ("advogado", "ˌadʒivoɡˈadʊ"),
            ("show", "ʃˈow"),
            ("tranquilo", "trˌɐ̃ŋkwˈilʊ"),
            ("linguiça", "lˌiŋɡwˈisæ"),
            ("quiser", "kizˈɛr"),
            ("número", "nˈũmeɾʊ"),
            ("espanhol", "ˌespɐ̃ɲˈɔw"),
            ("óleo", "ˈɔljʊ"),
            ("tv", "tˌevˈe"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn sentence_pass() {
        let mut outs: Vec<Out> = ["os", "amigos", "mais", "ou", "menos", "dez", "casas"]
            .iter()
            .map(|w| Out { ph: word(w), punct: String::new(), pause_after: false, src: w.to_string() })
            .collect();
        join(&mut outs);
        let got: Vec<&str> = outs.iter().map(|o| o.ph.as_str()).collect();
        assert_eq!(got, ["ʊz", "ˌæmˈiɡʊz", "mˈaɪz", "ow", "mˈenʊz", "dˈɛs", "kˈazæs"]);
    }

    #[test]
    fn numbers() {
        let j = |n| cardinal(n).join(" ");
        assert_eq!(j(21), "vinte e um");
        assert_eq!(j(100), "cem");
        assert_eq!(j(101), "cento e um");
        assert_eq!(j(1000), "mil");
        assert_eq!(j(1001), "mil e um");
        assert_eq!(j(1234), "mil duzentos e trinta e quatro");
        assert_eq!(j(2000), "dois mil");
        assert_eq!(j(1_000_000), "um milhão");
        assert_eq!(j(2_500_000), "dois milhões e quinhentos mil");
        assert_eq!(j(1_000_000_000), "um bilhão");
    }
}
