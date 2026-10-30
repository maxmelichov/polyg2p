//! Italian grapheme-to-phoneme rules, espeak-style notation.
//!
//! Italian spelling fixes almost everything except stress and the open/closed
//! quality of stressed e/o. Those come from, in order: a written accent, the
//! optional lexicon (`it.lex`), and ending-based statistics (`data/it_*.tsv`).

use std::collections::HashMap;
use std::path::Path;

use crate::lexicon::Lexicon;
use crate::Out;

const SUFFIX_TSV: &str = include_str!("../data/it_suffix.tsv");
const OPEN_TSV: &str = include_str!("../data/it_open.tsv");

const UNSTRESSED: &[&str] = &[
    "il", "lo", "la", "i", "gli", "le", "un", "di", "a", "da", "in", "con", "su", "per", "tra", "fra", "e",
    "ed", "o", "od", "ma", "che", "non", "mi", "ti", "si", "ci", "vi", "ne", "del", "al", "dal", "nel", "sul",
    "col", "se", "è", "ha", "hai", "hanno", "dei", "ai", "dai", "nei", "sui", "coi", "me", "te", "ce", "ve",
];
/// Short words whose u before a final vowel is the stressed nucleus (sˈuo), not a glide.
/// Open stressed vowel where the ending statistics say closed.
const OPEN_WORDS: &[&str] = &["chiesa", "chiese", "chiesetta", "chiesastico"];
/// Closed stressed vowel where the ending statistics say open (articulated prepositions).
const CLOSED_WORDS: &[&str] = &["della", "delle", "dello", "degli", "nella", "nelle", "nello", "negli", "stella", "stelle"];
/// Enclitic pronouns that attach to imperatives, infinitives and gerunds (smettila, parlatemi).
const ENCLITIC: &[&str] = &[
    "glielo", "gliela", "glieli", "gliele", "gliene", "melo", "mela", "meli", "mele", "mene", "telo", "tela", "teli",
    "tele", "tene", "celo", "cela", "celi", "cele", "cene", "velo", "vela", "veli", "vele", "vene", "selo", "sela",
    "seli", "sele", "sene", "gli", "lo", "la", "li", "le", "mi", "ti", "ci", "vi", "si", "ne",
];
const IMPERFECT: &[&str] = &["evo", "evi", "eva", "evamo", "evate", "evano"];
const U_NUCLEUS: &[&str] = &["suo", "sua", "sue", "tuo", "tua", "tue", "due", "bue", "lui", "cui", "fui"];
/// zz that is voiced (mèzzo, azzurro); the default for zz is voiceless.
const VOICED_ZZ: &[&str] = &["mezz", "azzurr", "rozz", "brezz", "bizzarr", "razz", "zizzan", "gazz", "olezz", "orzz"];
/// English and other loanwords in everyday Italian (hand-written).
const LOAN: &[(&str, &str)] = &[
    ("computer", "kompjˈuter"), ("internet", "ˈinternet"), ("john", "ʤɔn"), ("york", "jˈɔrk"),     ("sushi", "sˈuʃi"), ("weekend", "wˈikɛnd"), ("baseball", "bˈɛizbol"), ("sandwich", "sˈɛndwiʧ"),
    ("halloween", "ˈallowin"), ("roma", "rˈoma"), ("europa", "eʊrˈɔpa"), ("hotel", "otˈɛl"), ("bangladesh", "baŋɡladˈɛʃ"), ("facebook", "fˈeɪsbuk"), ("son", "sˈon"), ("autobus", "ˈaʊtobʊs"), ("mary", "mˈɛri"), ("new", "njˈu"), ("motel", "motˈɛl"), ("privacy", "prˈaivasi"), ("garage", "ɡarˈaʒ"), ("champagne", "ʃampˈaɲ"), ("george", "ʤˈɔrʤ"), ("judo", "ʤˈudo"), ("routine", "rutˈin"), ("detective", "detˈɛktiv"), ("bush", "bˈuʃ"), ("shanghai", "ʃaŋɡˈai"), ("souvenir", "suvenˈir"), ("chicago", "ʧikˈaɡo"), ("jackson", "ʤˈɛkson"), ("dessert", "dessˈɛr"), ("laptop", "lˈaptop"), ("bill", "bˈil"), ("sciare", "ʃiˈare"), ("sciatore", "ʃiatˈore"), ("sciatori", "ʃiatˈori"), ("sciatrice", "ʃiatrˈiʧe"), ("sciato", "ʃiˈato"), ("sciava", "ʃiˈava"), ("scio", "ʃˈio"),
    ("ok", "okˈɛi"), ("okay", "okˈɛi"), ("jeans", "ʤins"), ("club", "klab"), ("email", "imˈɛil"), ("web", "wɛb"),
    ("online", "onlˈain"), ("smartphone", "zmartfˈon"), ("party", "pˈarti"), ("leader", "lˈider"),
    ("manager", "mˈɛneʤer"), ("meeting", "mˈitinɡ"), ("marketing", "mˈarketinɡ"), ("business", "bˈiznes"),
    ("show", "ʃɔ"), ("team", "tim"), ("test", "tɛst"), ("toast", "tost"), ("whisky", "wˈiski"), ("hobby", "ˈɔbbi"),
    ("shopping", "ʃˈɔppinɡ"), ("stress", "strɛs"), ("mouse", "mˈaus"), ("jazz", "ʤɛz"), ("rock", "rɔk"),
    ("blues", "bluz"), ("cowboy", "kˈauboi"), ("bye", "bai"), ("hamburger", "ambˈurɡer"), ("ketchup", "kˈɛʧap"),
    ("pub", "pab"), ("puzzle", "pˈazol"), ("quiz", "kwiz"), ("relax", "rilˈaks"), ("sexy", "sˈɛksi"), ("shock", "ʃɔk"),
    ("spray", "sprɛi"), ("staff", "staf"), ("standard", "stˈandard"), ("ticket", "tˈiket"), ("trend", "trɛnd"),
    ("yacht", "jɔt"), ("tv", "tivˈu"), ("wifi", "wˈaifai"), ("video", "vˈideo"), ("software", "sˈɔftwer"),
    ("hardware", "ˈardwer"), ("killer", "kˈiller"), ("look", "luk"), ("boss", "bɔs"), ("camper", "kˈamper"),
    ("cocktail", "kˈɔktɛil"), ("college", "kˈɔlleʤ"), ("design", "dizˈain"), ("fitness", "fˈitnes"),
    ("gay", "ɡɛi"), ("hostess", "ˈɔstes"), ("jogging", "ʤˈɔɡɡinɡ"), ("jungle", "ʤˈanɡol"), ("laser", "lˈɛizer"),
    ("nylon", "nˈailon"), ("outlet", "ˈautlet"), ("playboy", "plˈɛiboi"), ("rugby", "rˈaɡbi"), ("scooter", "skˈuter"),
    ("shampoo", "ʃampˈo"), ("skateboard", "skˈɛitbord"), ("snowboard", "znˈɔubord"), ("surf", "sɛrf"),
    ("teenager", "tinˈɛiʤer"), ("thriller", "trˈiller"), ("tunnel", "tˈunnel"), ("welfare", "wˈɛlfer"),
    ("yogurt", "jˈɔɡurt"), ("jet", "ʤɛt"), ("jolly", "ʤˈɔlli"), ("junior", "ʤˈunior"), ("gol", "ɡɔl"), ("goal", "ɡɔl"),
];
const SECONDARY: &[&str] = &[
    "della", "dello", "delle", "degli", "dalla", "dallo", "dalle", "dagli", "nella", "nello", "nelle", "negli",
    "sulla", "sullo", "sulle", "sugli", "alla", "allo", "alle", "agli", "una", "uno",
];

/// Endings with fixed antepenultimate stress (Italian morphology; hand-written).
const SDRUCCIOLE: &[&str] = &[
    "abile", "abili", "ibile", "ibili", "evole", "evoli", "issimo", "issima", "issimi", "issime", "esimo",
    "esima", "esimi", "esime", "agine", "agini", "igine", "igini", "udine", "udini", "logo", "loga", "logi",
    "loghe", "grafo", "grafa", "grafi", "grafe", "fono", "fona", "foni", "metro", "metri", "ico", "ica",
    "ici", "iche", "fugo", "fuga", "colo", "cola", "coli", "cole", "avano", "evano", "ivano", "ebbero",
    "arono", "erono", "irono", "assero", "essero", "issero", "gliere",
];
/// Common proparoxytone words not covered by an ending rule (hand-written).
const SDRUCCIOLE_WORDS: &[&str] = &[
    "autobus",
    "vogliono", "devono", "debbono", "sogliono", "muoiono", "siano", "vadano", "facciano", "dicano", "vengano",
    "abbiano", "sappiano", "possano", "vogliano", "debbano", "diano", "stiano",
    "ultimo", "ultima", "ultimi", "ultime", "utile", "utili", "facile", "facili", "difficile", "difficili",
    "simile", "simili", "debole", "deboli", "nobile", "nobili", "mobile", "mobili", "stupido", "stupida",
    "rapido", "rapida", "solido", "timido", "lucido", "umido", "valido", "valida", "pallido", "tiepido",
    "limpido", "splendido", "splendida", "candido", "morbido", "morbida", "tavolo", "tavoli", "popolo",
    "popoli", "secolo", "secoli", "titolo", "titoli", "angolo", "angoli", "numero", "numeri", "camera",
    "camere", "opera", "opere", "epoca", "epoche", "pagina", "pagine", "macchina", "macchine", "monaco",
    "napoli", "genova", "padova", "modena", "taranto", "brindisi", "essere", "prendere", "vendere", "credere",
    "scrivere", "leggere", "chiudere", "perdere", "vivere", "ridere", "decidere", "chiedere", "rispondere",
    "correre", "muovere", "nascere", "piangere", "scegliere", "cogliere", "togliere", "spegnere", "mettere",
    "battere", "permettere", "promettere", "conoscere", "crescere", "piovere", "rompere", "scendere",
    "spendere", "accendere", "attendere", "difendere", "dipendere", "offendere", "pretendere", "rendere",
    "sorridere", "uccidere", "dividere", "giungere", "aggiungere", "raggiungere", "spingere", "dipingere",
    "fingere", "vincere", "convincere", "torcere", "cuocere", "nuocere", "tessere", "mungere", "ungere",
    "parlano", "mangiano", "lavorano", "abitano", "amano", "cantano", "giocano", "pensano", "guardano",
    "aspettano", "cercano", "trovano", "portano", "chiamano", "lasciano", "prendono", "vendono", "credono",
    "scrivono", "leggono", "vivono", "dormono", "partono", "sentono", "aprono", "offrono", "seguono",
    "vengono", "tengono", "dicono", "fanno", "vanno", "sanno", "conoscono", "mettono", "chiedono", "perdono",
    "erano", "furono", "ebbero", "dissero", "fecero", "vennero", "presero", "misero", "videro", "vollero",
    "sabato", "domenica", "lunedì", "medico", "medici", "tecnico", "tecnici", "subito", "abito", "abiti",
    "merito", "debito", "spirito", "limite", "limiti", "ospite", "ospiti", "giovane", "giovani", "ordine",
    "ordini", "immagine", "immagini", "origine", "margine", "termine", "termini", "polvere", "cenere",
    "lettera", "lettere", "regola", "regole", "favola", "isola", "isole", "nuvola", "parola", "tavola",
    "fragola", "pentola", "scatola", "vipera", "zucchero", "albero", "alberi", "fegato", "orfano", "sindaco",
    "stomaco", "cronaca", "dialogo", "catalogo", "monologo", "prologo", "piccolo", "piccola", "piccoli",
    "piccole", "povero", "povera", "poveri", "libero", "libera", "liberi", "tenero", "tenera", "celebre",
    "lacrima", "lacrime", "anima", "anime", "musica", "musiche", "fabbrica", "fabbriche", "replica",
    "unico", "unica", "pubblico", "pubblica", "politico", "politica", "economico", "economica", "storico",
    "storica", "pratico", "pratica", "classico", "classica", "magico", "magica", "tragico", "comico",
    "critico", "critica", "fisico", "fisica", "chimico", "chimica", "logico", "logica",
];
/// Words that look like they take an antepenultimate ending but are paroxytone.
const PIANE: &[&str] = &[
    "amico", "amica", "amici", "amiche", "nemico", "nemica", "nemici", "nemiche", "antico", "antica", "antichi",
    "antiche", "fico", "fichi", "pudico", "mendico", "lombrico", "formica", "formiche", "vescica", "ortica",
    "fatica", "fatiche", "lettica", "barbabietola", "colui", "ciascuno", "pacifica", "avere",
];

pub struct Italian {
    lex: Option<Lexicon>,
    suffix: HashMap<String, u8>,
    open: HashMap<String, f32>,
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'à' | 'è' | 'é' | 'ì' | 'ò' | 'ó' | 'ù' | 'í' | 'ú')
}
fn base(c: char) -> char {
    match c {
        'à' => 'a',
        'è' | 'é' => 'e',
        'ì' | 'í' => 'i',
        'ò' | 'ó' => 'o',
        'ù' | 'ú' => 'u',
        _ => c,
    }
}
fn accented(c: char) -> bool {
    matches!(c, 'à' | 'è' | 'é' | 'ì' | 'ò' | 'ó' | 'ù' | 'í' | 'ú')
}

impl Italian {
    pub fn load(lex: Option<&Path>) -> Self {
        let mut suffix = HashMap::new();
        for line in SUFFIX_TSV.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() >= 2 {
                if let Ok(st) = f[1].parse() {
                    suffix.insert(f[0].to_string(), st);
                }
            }
        }
        let mut open = HashMap::new();
        for line in OPEN_TSV.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() >= 2 {
                if let Ok(p) = f[1].parse() {
                    open.insert(f[0].to_string(), p);
                }
            }
        }
        Italian { lex: lex.and_then(|p| Lexicon::open(p).ok()), suffix, open }
    }

    pub fn word(&self, w: &str) -> String {
        let lw: String = w.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect();
        if lw.is_empty() {
            return String::new();
        }
        if !w.contains('\'') {
            if let Some((_, p)) = LOAN.iter().find(|(k, _)| *k == lw) {
                return p.to_string();
            }
        }
        let level = if UNSTRESSED.contains(&lw.as_str()) {
            0
        } else if SECONDARY.contains(&lw.as_str()) {
            1
        } else {
            2
        };
        // l'uomo, dell'anno: apostrophe-joined words come in as one token
        if let Some((a, b)) = w.split_once('\'') {
            if !a.is_empty() && !b.is_empty() {
                let al = a.to_lowercase();
                let right = self.word(b);
                if !al.chars().any(is_v) {
                    // c'era, c'è, sc'è: the elided ce/ci/sce keeps its soft consonant
                    let front = b.chars().next().is_some_and(|ch| matches!(base(ch.to_lowercase().next().unwrap_or(ch)), 'e' | 'i'));
                    let left = match (al.as_str(), front) {
                        ("c", true) => "ʧ",
                        ("c", false) => "k",
                        ("g", true) => "ʤ",
                        ("g", false) => "ɡ",
                        ("sc", true) => "ʃ",
                        ("sc", false) => "sk",
                        ("gl", _) => "ʎ",
                        ("gn", _) => "ɲ",
                        ("z", _) => "ʦ",
                        ("ch", _) => "k",
                        ("gh", _) => "ɡ",
                        ("qu", _) => "kw",
                        _ => al.as_str(),
                    };
                    // c'è, v'è: the verb keeps its stress even though è alone is a clitic
                    let right = if b.to_lowercase() == "è" { "ˈɛ".to_string() } else { right };
                    return format!("{left}{right}");
                }
                let mut left = self.realize(&al, 0);
                let bl = b.to_lowercase();
                // cos'hai, com'è, dov'è: the verb is stressed; cosa's s is intervocalic (kozˈai)
                let right = if UNSTRESSED.contains(&bl.as_str()) { self.realize(&bl, 2) } else { right };
                if left.ends_with('s') && bl.chars().next().is_some_and(|ch| is_v(ch) || ch == 'h') {
                    left.pop();
                    left.push('z');
                }
                return format!("{left}{right}");
            }
        }
        self.realize(&lw, level)
    }

    /// (stress index from the end, open vowel, voiced z) from the lexicon.
    fn lex_info(&self, lw: &str) -> (Option<u8>, Option<bool>, Option<bool>) {
        let Some(l) = &self.lex else { return (None, None, None) };
        // proper nouns are capitalized in the lexicon (Roma, Europa)
        let cap: String = lw.chars().next().map(|c| c.to_uppercase().chain(lw.chars().skip(1)).collect()).unwrap_or_default();
        let parse = |v: &str| -> (Option<u8>, Option<bool>, Option<bool>) {
            let mut it = v.split('|');
            let st = it.next().and_then(|s| s.parse().ok());
            let op = match it.next() {
                Some("o") => Some(true),
                Some("c") => Some(false),
                _ => None,
            };
            let z = match it.next() {
                Some("v") => Some(true),
                Some("u") => Some(false),
                _ => None,
            };
            (st, op, z)
        };
        let cs: Vec<char> = lw.chars().collect();
        let n = cs.len();
        // giocano, pensano: a 3pl verb has the vowel quality of its 3sg (gioca), not its stress
        let sg3 = |op: Option<bool>| -> Option<bool> {
            if op.is_some() || n < 5 {
                return op;
            }
            let s3: String = cs[..n - 3].iter().collect();
            let sg = if lw.ends_with("ano") { Some(format!("{s3}a")) } else if lw.ends_with("ono") { Some(format!("{s3}e")) } else { None };
            sg.and_then(|k| l.get(&k).or_else(|| l.get(&format!("{s3}o")))).and_then(|v| parse(&v).1)
        };
        if let Some(v) = l.get(lw).or_else(|| l.get(&cap)) {
            let (st, op, z) = parse(&v);
            return (st, sg3(op), z);
        }
        let (mut op, mut z) = (None, None);
        // an inflected form that Wiktionary lacks takes stress and vowel quality from the
        // form it has: dollari -> dollaro, felici -> felice, fattorie -> fattoria, ricorda -> ricordo
        if n >= 3 {
            let stem: String = cs[..n - 1].iter().collect();
            let alts: &[char] = match cs[n - 1] {
                'i' if n >= 4 => &['o', 'e', 'a'],
                'e' if n >= 4 => &['a', 'o'],
                'e' => &['a'],
                'a' if n >= 4 => &['o', 'e'],
                'o' if n >= 4 => &['a'],
                _ => &[],
            };
            for &a in alts {
                if let Some(v) = l.get(&format!("{stem}{a}")) {
                    let (st2, op2, z2) = parse(&v);
                    op = op.or(op2);
                    z = z.or(z2);
                    // the stressed vowel must not be the one that changed
                    if let Some(s) = st2.filter(|&s| s >= 1) {
                        return (Some(s), op, z);
                    }
                }
            }
            return (None, sg3(op), z);
        }
        (None, op, z)
    }

    /// smettila, parlatemi, vederlo: a verb form with an enclitic pronoun keeps its own stress.
    fn enclitic(&self, c: &[char], lw: &str, vp: &[usize]) -> Option<(usize, String)> {
        let l = self.lex.as_ref()?;
        if vp.len() < 3 {
            return None;
        }
        // dammelo, fallo, dicci, vattene: a monosyllabic imperative doubles the clitic's consonant
        for pre in ["da", "fa", "di", "va", "sta"] {
            let k = pre.len();
            if lw.starts_with(pre) && c.len() > k + 2 && c[k] == c[k + 1] && !is_v(c[k]) {
                let tail: String = c[k + 2..].iter().collect();
                let with: String = c[k + 1..].iter().collect();
                if ENCLITIC.contains(&tail.as_str()) || ENCLITIC.contains(&with.as_str()) {
                    return Some((vp[0], lw.to_string()));
                }
            }
        }
        let known = l.contains(lw);
        for cl in ENCLITIC {
            let Some(base) = lw.strip_suffix(cl) else { continue };
            let bc: Vec<char> = base.chars().collect();
            // gerunds, plural imperatives and infinitives are unmistakable; a 2sg imperative
            // plus clitic is only read into a word Wiktionary doesn't know (generale, animale
            // are not genera+le, anima+le), and never with mi/ti/si/vi (amavi, finiti, salami)
            let unambiguous = base.ends_with('r') || ["ndo", "ate", "ete", "ite"].iter().any(|e| base.ends_with(e));
            if !unambiguous && (known || matches!(*cl, "mi" | "ti" | "si" | "vi")) {
                continue;
            }
            if bc.len() < 3 {
                continue;
            }
            let bvp: Vec<usize> = (0..bc.len()).filter(|&i| is_v(bc[i])).collect();
            if bvp.len() < 2 {
                continue;
            }
            // infinitives drop the e (far-lo, veder-lo); imperatives and gerunds are verb forms
            // only if their infinitive exists (guarda -> guardare, but settima-ne is not one)
            let key = if base.ends_with('r') { format!("{base}e") } else { base.to_string() };
            if base.ends_with('r') {
                if !l.contains(&key) {
                    continue;
                }
            } else {
                let inf = |suf: &str, ends: &[&str]| -> Vec<String> {
                    base.strip_suffix(suf).map_or(vec![], |st| ends.iter().map(|e| format!("{st}{e}")).collect())
                };
                let mut cands = vec![];
                cands.extend(inf("ando", &["are"]));
                cands.extend(inf("endo", &["ere", "ire"]));
                cands.extend(inf("ate", &["are"]));
                cands.extend(inf("ete", &["ere"]));
                cands.extend(inf("ite", &["ire"]));
                cands.extend(inf("iamo", &["are", "ere", "ire"]));
                cands.extend(inf("isci", &["ire"]));
                cands.extend(inf("a", &["are"]));
                cands.extend(inf("i", &["ere", "ire"]));
                if !cands.iter().any(|i| l.contains(i)) {
                    continue;
                }
            }
            let kc: Vec<char> = key.chars().collect();
            let kvp: Vec<usize> = (0..kc.len()).filter(|&i| is_v(kc[i])).collect();
            let (st, _, _) = if l.contains(&key) { self.lex_info(&key) } else { (None, None, None) };
            let p = match st {
                Some(s) if (s as usize) < kvp.len() => kvp[kvp.len() - 1 - s as usize],
                _ => {
                    if base.ends_with('r') {
                        // -are/-ire infinitives stress the vowel before the r (parlar-lo, finir-la);
                        // an -ere infinitive of unknown stress (scriver-ci, annetter-si) is
                        // stressed the way the rules stress the infinitive itself
                        if base.ends_with("er") {
                            let (kst, kop, _) = self.lex_info(&key);
                            let kp = self.stress_pos(&kc, &key, &kvp, kst, kop == Some(false));
                            if kp >= bc.len() {
                                continue;
                            }
                            kp
                        } else {
                            *bvp.last().unwrap()
                        }
                    } else {
                        let syl: Vec<usize> = bvp.iter().copied().filter(|&p| syllabic(&bc, p)).collect();
                        if syl.len() >= 2 { syl[syl.len() - 2] } else { continue }
                    }
                }
            };
            if p < bc.len() && c.get(p) == bc.get(p) {
                return Some((p, base.to_string()));
            }
        }
        None
    }

    /// Index (into `c`) of the stressed vowel letter.
    fn stress_pos(&self, c: &[char], lw: &str, vp: &[usize], lex_st: Option<u8>, lex_closed: bool) -> usize {
        let nv = vp.len();
        if let Some(&p) = vp.iter().rev().find(|&&p| accented(c[p])) {
            return p;
        }
        if nv == 1 {
            return vp[0];
        }
        // suo, tuo, due: the u is the stressed nucleus, not a glide (unlike buono, può)
        if U_NUCLEUS.contains(&lw) {
            return vp[0];
        }
        if let Some(st) = lex_st {
            if (st as usize) < nv {
                return vp[nv - 1 - st as usize];
            }
        }
        if let Some((p, _)) = self.enclitic(c, lw, vp) {
            return p;
        }
        let n = c.len();
        if PIANE.contains(&lw) {
            let syl: Vec<usize> = vp.iter().copied().filter(|&p| syllabic(c, p)).collect();
            return if syl.len() >= 2 { syl[syl.len() - 2] } else { vp[nv - 1] };
        }
        if nv >= 3 && SDRUCCIOLE_WORDS.contains(&lw) {
            let syl: Vec<usize> = vp.iter().copied().filter(|&p| syllabic(c, p)).collect();
            return if syl.len() >= 3 { syl[syl.len() - 3] } else { vp[nv - 3] };
        }
        if !PIANE.contains(&lw) && nv >= 3 && SDRUCCIOLE.iter().any(|e| lw.len() > e.len() + 1 && lw.ends_with(e)) {
            let syl: Vec<usize> = vp.iter().copied().filter(|&p| syllabic(c, p)).collect();
            return if syl.len() >= 3 { syl[syl.len() - 3] } else { vp[nv - 3] };
        }
        if let Some(p) = self.verb_3pl(c, lw, vp) {
            return p;
        }
        for k in (2..=7).rev() {
            if n > k {
                let start = lw.char_indices().nth(n - k).map(|(b, _)| b).unwrap_or(0);
                if let Some(&st) = self.suffix.get(&lw[start..]) {
                    if (st as usize) < nv {
                        let p = vp[nv - 1 - st as usize];
                        // ending statistics come from open-vowel words; a word known to have
                        // no open vowel can't be stressed on an e/o there by that evidence
                        if st >= 2 && lex_closed && matches!(c[p], 'e' | 'o') {
                            break;
                        }
                        // a glide can't carry the stress (pieno, fieno: -eno counts letters)
                        if !syllabic(c, p) {
                            break;
                        }
                        return p;
                    }
                }
            }
        }
        // default: penultimate syllabic vowel (skip glide / silent letters)
        let syl: Vec<usize> = vp.iter().copied().filter(|&p| syllabic(c, p)).collect();
        if syl.len() >= 2 {
            syl[syl.len() - 2]
        } else {
            *syl.last().unwrap_or(&vp[nv - 1])
        }
    }

    /// Third-person-plural present forms (ricordano, prendono, finiscono) are
    /// proparoxytone; recognise them by finding their infinitive in the lexicon.
    fn verb_3pl(&self, c: &[char], lw: &str, vp: &[usize]) -> Option<usize> {
        let l = self.lex.as_ref()?;
        if vp.len() < 3 {
            return None;
        }
        let infs: Vec<String> = if let Some(stem) = lw.strip_suffix("ano") {
            vec![format!("{stem}are")]
        } else if let Some(stem) = lw.strip_suffix("iscono") {
            vec![format!("{stem}ire")]
        } else if let Some(stem) = lw.strip_suffix("ono") {
            let mut v = vec![format!("{stem}ere"), format!("{stem}ire")];
            if let Some(s) = stem.strip_suffix("lg") {
                v.push(format!("{s}gliere"));
            }
            if let Some(s) = stem.strip_suffix("ng") {
                v.push(format!("{s}nere"));
            }
            v
        } else {
            return None;
        };
        if !infs.iter().any(|i| l.contains(i)) {
            return None;
        }
        let syl: Vec<usize> = vp.iter().copied().filter(|&p| syllabic(c, p)).collect();
        if syl.len() >= 3 { Some(syl[syl.len() - 3]) } else { None }
    }

    fn is_open(&self, c: &[char], sp: usize, lex_op: Option<bool>) -> bool {
        match c[sp] {
            'è' | 'ò' => return true,
            'é' | 'ó' => return false,
            _ => {}
        }
        if let Some(o) = lex_op {
            return o;
        }
        let lw: String = c.iter().collect();
        if OPEN_WORDS.contains(&lw.as_str()) {
            return true;
        }
        if CLOSED_WORDS.contains(&lw.as_str()) {
            return false;
        }
        // the uo diphthong is always open (buona, nuovo, scuola)
        if base(c[sp]) == 'o' && sp > 0 && c[sp - 1] == 'u' {
            return true;
        }
        let tail: String = c[sp..].iter().collect();
        // viene, tiene, appartiene, contiene (but pieno/piene is closed)
        if tail == "ene" && sp > 0 && c[sp - 1] == 'i' && !lw.starts_with("pien") {
            return true;
        }
        // posto, risposto, proposta, imposte: the participle family is closed
        if sp >= 2 && c[sp - 1] == 'p' && matches!(tail.as_str(), "osto" | "osta" | "osti" | "oste") {
            return false;
        }
        // the -mento suffix has a closed e (allevamenti, movimento); vento/cento are too short to be it
        if sp >= 3 && c[sp - 1] == 'm' && matches!(tail.as_str(), "ento" | "enti") {
            return false;
        }
        // imperfect indicative of -ere verbs keeps a closed e (vedevano, credeva, dovevi)
        if sp > 0 && IMPERFECT.iter().any(|e| tail == *e) {
            return false;
        }
        let full = format!("{tail}#");
        if let Some(&p) = self.open.get(&full) {
            return p >= 0.5;
        }
        let tc: Vec<char> = tail.chars().collect();
        for k in (2..tc.len()).rev() {
            let key: String = tc[..k].iter().collect();
            if let Some(&p) = self.open.get(&key) {
                return p >= 0.5;
            }
        }
        let key = format!("{}*", base(c[sp]));
        self.open.get(&key).map_or(false, |&p| p >= 0.5)
    }

    fn realize(&self, lw: &str, level: u8) -> String {
        let c: Vec<char> = lw.chars().collect();
        let n = c.len();
        let vp: Vec<usize> = (0..n).filter(|&i| is_v(c[i])).collect();
        if vp.is_empty() {
            return spell(lw);
        }
        let (lex_st, lex_op, lex_z) = self.lex_info(lw);
        let sp = self.stress_pos(&c, lw, &vp, lex_st, lex_op == Some(false));
        // smettila, vedetelo: vowel quality comes from the verb, not from the clitic ending
        let (oc, oop) = match self.enclitic(&c, lw, &vp) {
            Some((_, b)) if lex_st.is_none() => (b.chars().collect::<Vec<char>>(), lex_op.or(self.lex_info(&b).1)),
            _ => (c.clone(), lex_op),
        };
        // a grave accent spells the open vowel even on an unstressed clitic (è, c'è, ciò)
        let open = matches!(c[sp], 'è' | 'ò')
            || (level > 0 && matches!(base(c[sp]), 'e' | 'o') && self.is_open(&oc, sp, oop));
        let at = |i: isize| -> char { if i < 0 || i as usize >= n { '#' } else { c[i as usize] } };
        let mut out = String::with_capacity(n * 2);
        let mut i = 0usize;
        while i < n {
            let ch = c[i];
            let nx = at(i as isize + 1);
            let pv = at(i as isize - 1);
            let front = matches!(base(nx), 'e' | 'i');
            if is_v(ch) {
                let stressed = i == sp;
                if stressed && level > 0 {
                    out.push(if level == 2 { 'ˈ' } else { 'ˌ' });
                }
                let b = base(ch);
                match b {
                    'a' => out.push('a'),
                    'e' => out.push(if stressed && open { 'ɛ' } else { 'e' }),
                    'o' => out.push(if stressed && open { 'ɔ' } else { 'o' }),
                    'i' => {
                        if !stressed && is_v(nx) {
                            let silent = matches!(pv, 'c' | 'g') || (pv == 'l' && at(i as isize - 2) == 'g')
                                || (pv == 'c' && at(i as isize - 2) == 's');
                            if !silent {
                                // espeak: a glide before the stress, a full vowel after it;
                                // reversed after z (violazione / grazie); ri- prefix keeps i
                                let before = i < sp;
                                let zion = pv == 'z' && base(nx) == 'o' && at(i as isize + 2) == 'n';
                                let glide = if before {
                                    !matches!(pv, 'r' | 'l') && !zion
                                } else {
                                    pv == 'z'
                                };
                                out.push(if glide { 'j' } else { 'i' });
                            }
                        } else if !stressed && i + 1 == n && vp.len() > 1 && !is_v(pv) {
                            // final unstressed i is reduced (amici), but not a diphthong offglide (sei, poi)
                            out.push('ɪ');
                        } else if i + 1 == n && level == 0 && n > 1 {
                            // ci, si, di, mi: an unstressed clitic ends in the reduced vowel (the article i keeps i)
                            out.push('ɪ');
                        } else {
                            out.push('i');
                        }
                    }
                    _ => {
                        // u
                        if pv == 'q' || (pv == 'g' && is_v(nx) && !stressed) {
                            out.push('w');
                        } else if !stressed && is_v(nx) && i == 0 {
                            out.push('w');
                        } else if level < 2 || (!stressed && !(i + 1 == n && vp.len() == 1)) {
                            // un, una, su, sul: espeak writes the reduced vowel in unstressed function words
                            out.push('ʊ');
                        } else {
                            out.push('u');
                        }
                    }
                }
                i += 1;
                continue;
            }
            let double = nx == ch && ch != 'h';
            match ch {
                'c' => {
                    if nx == 'h' {
                        out.push('k');
                        i += 2;
                        continue;
                    }
                    if nx == 'q' {
                        out.push_str("kː");
                        i += 2;
                        continue;
                    }
                    if double {
                        let f = matches!(base(at(i as isize + 2)), 'e' | 'i');
                        out.push_str(if f { "ʧː" } else { "kː" });
                        if at(i as isize + 2) == 'h' {
                            i += 1;
                        }
                        i += 2;
                        continue;
                    }
                    if front {
                        out.push('ʧ');
                    } else {
                        out.push('k');
                    }
                }
                'g' => {
                    if nx == 'h' {
                        out.push('ɡ');
                        i += 2;
                        continue;
                    }
                    if nx == 'n' {
                        // espeak writes intervocalic gn long
                        out.push_str(if i > 0 && i == sp + 1 { "ɲɲ" } else { "ɲ" });
                        i += 2;
                        continue;
                    }
                    if nx == 'l' && base(at(i as isize + 2)) == 'i' {
                        out.push('ʎ');
                        // "gli" + vowel: the i is silent; "gli" alone keeps it
                        i += 2;
                        continue;
                    }
                    if double {
                        let f = matches!(base(at(i as isize + 2)), 'e' | 'i');
                        out.push_str(if f { "ʤː" } else { "ɡː" });
                        if at(i as isize + 2) == 'h' {
                            i += 1;
                        }
                        i += 2;
                        continue;
                    }
                    out.push(if front { 'ʤ' } else { 'ɡ' });
                }
                's' => {
                    if nx == 'c' && matches!(base(at(i as isize + 2)), 'e' | 'i') {
                        out.push('ʃ');
                        i += 2;
                        continue;
                    }
                    if nx == 's' {
                        out.push_str("ss");
                        i += 2;
                        continue;
                    }
                    let voiced_next = matches!(nx, 'b' | 'd' | 'g' | 'l' | 'm' | 'n' | 'r' | 'v');
                    let intervocalic = is_v(pv) && is_v(nx);
                    out.push(if voiced_next || intervocalic { 'z' } else { 's' });
                }
                'z' => {
                    if double {
                        // -izzare verbs are voiced in espeak
                        let vowels_before = c[..i].iter().filter(|&&x| is_v(x)).count();
                        let voiced = lex_z.unwrap_or_else(|| {
                            (pv == 'i' && vowels_before >= 2 && !lw.starts_with("indirizz"))
                                || VOICED_ZZ.iter().any(|z| lw.starts_with(z))
                        });
                        out.push_str(if voiced { "ʣː" } else { "tsː" });
                        i += 2;
                        continue;
                    }
                    // negozi, servizi, grazie: a single z before i is voiceless
                    let before_i = base(nx) == 'i';
                    let voiced = lex_z.unwrap_or(i == 0 || (is_v(pv) && is_v(nx) && !before_i));
                    out.push_str(if voiced { "ʣ" } else { "ts" });
                }
                'h' => {}
                'q' => out.push('k'),
                'x' => out.push_str("ks"),
                'j' | 'y' => out.push(if is_v(nx) { 'j' } else { 'i' }),
                'k' => out.push('k'),
                'w' => out.push('w'),
                'p' | 't' | 'b' | 'd' => {
                    if double {
                        out.push(ch);
                        out.push('ː');
                        i += 2;
                        continue;
                    }
                    out.push(ch);
                }
                'n' => out.push(if nx == 'g' && !front_after(&c, i + 1) && at(i as isize + 2) != 'n' { 'ŋ' } else { 'n' }),
                'r' => {
                    if nx == 'r' {
                        out.push_str("rɾ");
                        i += 2;
                        continue;
                    }
                    out.push('r');
                }
                'f' | 'l' | 'm' | 'v' => out.push(ch),
                _ => {}
            }
            i += 1;
        }
        out
    }
}

/// Is the c/g at `p` soft (followed by e/i)?
fn front_after(c: &[char], p: usize) -> bool {
    c.get(p + 1).map_or(false, |&x| matches!(base(x), 'e' | 'i'))
}

/// Can vowel letter at p carry stress by default (not a glide / silent letter)?
fn syllabic(c: &[char], p: usize) -> bool {
    let b = base(c[p]);
    let next_v = c.get(p + 1).map_or(false, |&x| is_v(x));
    if !next_v || accented(c[p]) {
        return true;
    }
    match b {
        'i' => false,
        // eu is a falling diphthong (Europa, neutro), au/ou keep the u (paura, pausa)
        'u' => p > 0 && base(c[p - 1]) == 'e',
        _ => true,
    }
}

fn spell(w: &str) -> String {
    // vowel-less tokens (tv, mp) are passed through; foreign letters and symbols are dropped
    w.chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

pub fn join(_w: &mut [Out]) {}

#[cfg(test)]
mod tests {
    use super::Italian;
    #[test]
    fn basics() {
        let it = Italian::load(None);
        for (w, p) in [
            ("casa", "kˈaza"),
            ("pizza", "pˈitsːa"),
            ("città", "ʧitːˈa"),
            ("gnocchi", "ɲˈɔkːɪ"),
            ("figlio", "fˈiʎo"),
            ("ciao", "ʧˈao"),
            ("chiave", "kjˈave"),
            ("quando", "kwˈando"),
            ("acqua", "ˈakːwa"),
            ("università", "ʊniversitˈa"), ("amici", "amˈiʧɪ"), ("telefono", "telˈɛfono"),
            ("perché", "perkˈe"),
            ("sbaglio", "zbˈaʎo"),
            ("bello", "bˈɛllo"),
        ] {
            assert_eq!(it.word(w), p, "{w}");
        }
    }
}
