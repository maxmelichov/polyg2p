//! Romanian grapheme-to-phoneme, espeak-style notation.
//!
//! Romanian spelling is nearly phonemic; the hard parts are the glide/hiatus
//! status of `i` and `u` next to other vowels and the (lexical) stress. This
//! module is rules only. Notation follows espeak-ng's `ro` voice:
//!
//! * vowels `a e i o u ə ɨ`; the diphthongs are written `ea` (`avˈea`), `ɔa`
//!   (`pˈɔate`), `aɪ eɪ uɪ əɪ ɨɪ iɪ` (`maɪ`, `lˈuɪ`, `ˈaniɪ`), `oj` (`dˈoj`),
//!   `aʊ eʊ oʊ əʊ iʊ` word-finally (`stˈaʊ`, `nˈoʊ`, `fˈiʊ`); `u` between vowels
//!   is `w` (`zˈiwa`, `dˈowə`), `i` between vowels is `j` (`femˈeje`).
//! * `ce ci ge gi` are `tʃ dʒ` (ligatured by `fold.rs`), `che chi ghe ghi` are
//!   `k ɡ`, `ș ț j x` are `ʃ ts ʒ ks` (`ɡz` in `ex-` + vowel), `h` is `h`.
//! * a word-final `i` after a consonant is the whispered palatal offglide: espeak
//!   writes `ʲʲ` after a plain consonant (`ˈanʲʲ`, `tˈotsʲʲ`) and `ʲ` after
//!   `tʃ dʒ k ɡ` (`nˈitʃʲ`, `ˈokʲ`); it is a full `i` after a consonant + liquid
//!   cluster (`mˈembri`) and in monosyllables (`ʃi`).
//! * `r` is a tap `ɾ` between vowels (`kˈaɾe`), a trill `r` elsewhere; `n` is
//!   `ŋ` before `g`.
//! * `e-` is `je-` in the forms of *a fi* and the personal pronouns (`jˈeste`,
//!   `jˈel`, `jˈa`).
//! * Stress `ˈ` goes right before the stressed vowel; secondary `ˌ` on every
//!   second syllable counted from the word start before the primary one and on
//!   the syllable two after it, as espeak does (`ˌɨntʃepˈut`, `asˈemenˌea`).
//!   Placement: penultimate for words ending in a syllabic vowel, final for
//!   words ending in a consonant or a glide, with morphological overrides
//!   (article and plural suffixes keep the stem's stress, `-ic(ă)`, `-ie` nouns,
//!   verb endings, `-ă` never stressed next to the end …) and a small list of
//!   exceptions. `join` moves the stress of an infinitive after the marker `a`
//!   to the ending (`a lukrˈa`, `a vorbˈi`).

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "virgulă";
/// Letter names a..z (phonemes).
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "be", "ʧe", "de", "e", "ef", "ʤe", "haʃ", "i", "ʒe", "ka", "el", "em", "en", "o", "pe", "kju", "er", "es",
    "te", "u", "ve", "dubluve", "iks", "iɡrek", "zet",
]);

/// Clitics and short function words espeak reads without any stress mark.
const UNSTRESSED: &[&str] = &[
    "de", "în", "și", "a", "la", "să", "din", "o", "pe", "cu", "mai", "un", "se", "ca", "al", "dar", "sau", "ne",
    "le", "prin", "când", "ale", "in", "te", "vă", "mi", "mă", "îl", "ți", "cam", "ni", "ție", "ci", "vi", "li",
    "s", "n", "l", "m", "v", "i",
];
/// Function words that carry a secondary stress only (espeak: `kˌaɾe`).
const SECONDARY: &[&str] = &[
    "care", "pentru", "dacă", "fie", "unui", "unei", "îi", "ceea", "orice", "unor", "ori", "îmi", "îți", "unele",
    "niște", "tocmai", "aceștia", "unii", "acești", "multor", "căreia", "căruia", "căror", "cărora", "fiecărui",
    "fiecărei", "alea", "acelor", "anumitor", "unuia", "uneia",
];

/// Words whose pronunciation is given outright (clitic groups, `e-` → `je-`,
/// abbreviations, foreign words). Unstressed ones carry no mark.
const FIXED: &[(&str, &str)] = &[
    ("e", "je"), ("el", "jˈel"), ("ea", "jˈa"), ("ei", "jˈeɪ"), ("ele", "jˈele"), ("eu", "jˈeʊ"),
    ("este", "jˈeste"), ("ești", "jˈeʃtʲ"), ("era", "jeɾˈa"), ("eram", "jeɾˈam"), ("erai", "jeɾˈaɪ"),
    ("erați", "jeɾˈatsʲʲ"), ("erau", "jeɾˈaʊ"), ("ești", "jˈeʃtʲ"),
    ("s-a", "sa"), ("s-au", "saʊ"), ("n-a", "na"), ("n-au", "naʊ"), ("n-am", "nam"), ("n-ai", "naɪ"),
    ("n-ar", "nar"), ("n-aș", "naʃ"), ("n-o", "no"), ("n-are", "nˈaɾe"), ("n-avea", "navˈea"),
    ("l-a", "la"), ("l-au", "laʊ"), ("l-am", "lam"), ("l-ai", "laɪ"), ("l-aș", "laʃ"), ("l-ar", "lar"),
    ("m-a", "ma"), ("m-am", "mam"), ("m-au", "maʊ"), ("m-ai", "maɪ"), ("m-ar", "mar"), ("m-aș", "maʃ"),
    ("v-a", "va"), ("v-am", "vam"), ("v-au", "vaʊ"), ("v-ați", "vatsʲʲ"), ("v-ar", "var"), ("v-o", "vo"),
    ("i-a", "ja"), ("i-au", "jaʊ"), ("i-am", "jam"), ("i-ai", "jaɪ"), ("i-ar", "jar"), ("i-aș", "jaʃ"),
    ("i-o", "jo"), ("i-e", "je"), ("i-l", "il"), ("i-i", "iɪ"),
    ("mi-a", "mja"), ("mi-au", "mjaʊ"), ("mi-am", "mjam"), ("mi-ai", "mjaɪ"), ("mi-ar", "mjar"),
    ("mi-aș", "mjaʃ"), ("mi-e", "mje"), ("mi-o", "mjo"), ("mi-l", "mil"), ("mi-i", "miɪ"), ("mi-s", "mis"),
    ("ți-a", "tsja"), ("ți-au", "tsjaʊ"), ("ți-am", "tsjam"), ("ți-ai", "tsjaɪ"), ("ți-ar", "tsjar"),
    ("ți-e", "tsje"), ("ți-o", "tsjo"), ("ți-l", "tsil"), ("ți-i", "tsiɪ"),
    ("și-a", "ʃja"), ("și-au", "ʃjaʊ"), ("și-am", "ʃjam"), ("și-ai", "ʃjaɪ"), ("și-ar", "ʃjar"), ("și-o", "ʃjo"),
    ("și-l", "ʃil"), ("și-i", "ʃiɪ"), ("și-e", "ʃje"),
    ("ni-a", "nja"), ("ni-au", "njaʊ"), ("ni-l", "nil"), ("ni-i", "niɪ"), ("ni-e", "nje"), ("ni-s", "nis"),
    ("vi-a", "vja"), ("vi-au", "vjaʊ"), ("vi-l", "vil"), ("vi-i", "viɪ"), ("vi-e", "vje"), ("vi-s", "vis"),
    ("li-a", "lja"), ("li-au", "ljaʊ"), ("li-e", "lje"), ("li-s", "lis"),
    ("ne-a", "nea"), ("ne-au", "neaʊ"), ("ne-am", "neam"), ("ne-ai", "neaɪ"), ("ne-ar", "near"), ("ne-o", "neo"),
    ("ne-ați", "neatsʲʲ"),
    ("le-a", "lea"), ("le-au", "leaʊ"), ("le-am", "leam"), ("le-ai", "leaɪ"), ("le-ar", "lear"), ("le-o", "leo"),
    ("se-a", "sea"), ("te-a", "tea"), ("te-au", "teaʊ"), ("te-am", "team"), ("te-ai", "teaɪ"), ("te-ar", "tear"),
    ("te-o", "teo"), ("ce-a", "tʃea"), ("ce-au", "tʃeaʊ"), ("ce-am", "tʃeam"), ("ce-ai", "tʃeaɪ"), ("ce-ar", "tʃear"),
    ("ce-i", "tʃeɪ"), ("ce-o", "tʃeo"), ("ce-l", "tʃel"), ("de-a", "dea"), ("de-al", "deal"), ("de-ai", "deaɪ"),
    ("de-ale", "deˈale"), ("de-o", "deo"), ("de-un", "deun"), ("pe-un", "peun"), ("pe-o", "peo"), ("c-o", "ko"),
    ("c-un", "kun"), ("d-o", "do"), ("d-un", "dun"), ("nu-i", "nuɪ"), ("nu-l", "nul"), ("nu-s", "nus"),
    ("nu-mi", "numʲʲ"), ("nu-ți", "nutsʲʲ"), ("nu-și", "nuʃʲʲ"), ("cu-n", "kun"), ("să-i", "səɪ"), ("să-l", "səl"),
    ("să-mi", "səmʲʲ"), ("să-ți", "sətsʲʲ"), ("să-și", "səʃʲʲ"), ("să-și", "səʃʲʲ"), ("să-i", "səɪ"),
    ("să-și", "səʃʲʲ"), ("că-i", "kəɪ"), ("că-l", "kəl"), ("că-s", "kəs"), ("dă-i", "dˈəɪ"), ("dă-mi", "dˈəmʲʲ"),
    ("într-o", "ˈɨntro"), ("într-un", "ˈɨntrun"), ("dintr-o", "dˈintro"), ("dintr-un", "dˈintrun"),
    ("printr-o", "prˈintro"), ("printr-un", "prˈintrun"), ("într-una", "ˈɨntruna"), ("într-adevăr", "ˌɨntradevˈər"),
    ("într-însa", "ɨntrˈɨnsa"), ("într-însul", "ɨntrˈɨnsul"), ("printr-însa", "printrˈɨnsa"),
    ("etc", "ettʃˈeteɾa"), ("nr", "numˈərul"), ("str", "strˈada"), ("dl", "dˈomnul"), ("dna", "dˈɔamna"),
    ("dnei", "dˈɔamneɪ"), ("km", "kilomˈetri"), ("kg", "kiloɡrˈame"), ("art", "artˈikolul"), ("tel", "telefˈon"),
    ("ok", "okˈeɪ"), ("ora", "ˈoɾa"),
    ("sunt", "sˈunt"), ("sînt", "sˈɨnt"), ("suntem", "sˈuntem"), ("sunteți", "sˈuntetsʲʲ"), ("aici", "aˈitʃʲ"),
    ("aidoma", "aˈidoma"), ("ioana", "iˈɔana"), ("ioan", "iˈɔan"), ("ion", "iˈon"), ("ionuț", "ionˈuts"),
    ("ionescu", "ionˈesku"), ("ionel", "ionˈel"), ("iov", "iˈov"), ("înainte", "ˌɨnaˈinte"), ("înaintea", "ˌɨnaˈintea"), ("uita", "uɪtˈa"), ("uită", "ˈuɪtə"),
    ("uitat", "uɪtˈat"), ("uitată", "uɪtˈatə"), ("uitați", "uɪtˈatsʲʲ"), ("uit", "ˈuɪt"), ("uiți", "ˈuɪtsʲʲ"),
    ("uitare", "uɪtˈaɾe"), ("uite", "ˈuɪte"), ("cuib", "kˈuɪb"), ("cuiburi", "kˈuɪbuɾʲʲ"), ("niciun", "nitʃiˈun"), ("nicio", "nˈitʃio"), ("niciuna", "nitʃiˈuna"),
    ("niciunul", "nitʃiˈunul"), ("niciunei", "nitʃiˈuneɪ"), ("niciunui", "nitʃiˈunuɪ"), ("aievea", "aˈjevea"), ("iie", "ˈije"),
];

/// Words with irregular stress: the stressed vowel letter is upper-cased.
const STRESS: &[&str] = &[
    "Oameni", "Oamenii", "copIi", "copIii", "nUmai", "sÂmbătă", "dumInică", "vIneri", "lIniște", "lAcrimă",
    "pAgină", "pAgini", "mArgine", "mArgini", "drAgoste", "Oaspete", "Oaspeți", "Iepure", "Iepuri", "pEpene",
    "rEpede", "cAmeră", "cAmera", "cAmere", "Operă", "Opera", "Opere", "lImită", "lImita", "lImite", "Ancoră",
    "fOrmulă", "fOrmula", "fOrmule", "vevEriță", "vevErița", "sEcol", "sEcole", "nimIc", "amIc", "voinIc",
    "pitIc", "inamIc", "pisIcă", "pisIca", "pisIci", "furnIcă", "furnIca", "furnIci", "bunIcă", "bunIca",
    "bunIci", "bunIc", "ridIcă", "aplIcă", "explIcă", "indIcă", "implIcă", "complIcă", "dedIcă", "predIcă",
    "mânEcă", "adevĂr", "arĂt", "Iunie", "Iulie", "August", "polItică", "polItica", "polItici", "polItice",
    "Ultimul", "Ultima", "Ultimii", "Ultime", "Ultim", "Ultimă", "Ultimei", "Altfel", "Astfel", "Astăzi",
    "dejA", "tOtuși", "Orice", "cevA", "cinevA", "altcevA", "cumvA", "cândvA", "undevA", "câtevA", "câțivA",
    "Ăștia", "acEștia", "desIgur", "aurIe", "aurIi", "argintIe", "cenușIe", "cafenIe", "luA", "vIzită", "vIzita", "vIzite", "bUcur", "lIder", "lIderi", "lIderul",
    "lIderii", "Insula", "Insulă", "Insule", "modEle", "modElele", "gAlben", "gAlbenă", "gAlbeni", "gAlbene",
    "tErmină", "tErmin", "tErmini", "prEot", "prEotul", "prEoți", "prImei", "femEi", "idEi", "evrEi", "Isus",
    "Epocă", "Epoca", "Epoci", "tEnis", "asIgură", "asIgur", "asIguri", "asIgure", "descrIe", "înscrIe",
    "prescrIe", "descrIere", "descrIu", "grAtis", "cÂntec", "cÂntece", "cÂntecul", "detErmină", "detErmin",
    "detErmini", "considerA", "spIrit", "spIritul", "spIrite", "epIscop", "epIscopul", "opEră", "opEre",
    "Ieri", "sUflă", "mĂnâncă", "mĂnânc", "mĂnânci", "lUcrează", "cUmpără", "cUmpăr", "cUmperi", "cUmpere", "obicEi", "obicEiul", "obicEiuri", "cĂrei", "cElei", "mErită", "mErit",
    "mEriți", "pAgina", "servIcii", "mUreș", "sprIjin", "sprIjinul", "sprIjină", "sUper", "tUdor", "rObert",
    "dAvid", "mAximă", "mAxim", "mAxime", "mInimă", "mInim", "mInime", "rEgulă", "rEgula", "rEguli", "ilIe",
    "mOscova", "nEutru", "nEutră", "Euro", "împreUnă", "Opt", "cOntra", "Împotrivă", "vasIle", "zIlei", "cAsei", "fEtei", "mAmei", "destUl", "destUlă",
    "destUle", "destUi", "acEstei", "acEstuia", "acEstora", "acEsteia", "acEstor", "acElei", "acEluia", "acElora",
    "acEleia", "acElor", "ghEorghe", "gEorge", "vIdeo", "rAdio", "stUdio", "aUdio", "lIber", "lIberă", "lIberi",
    "lIbere", "cAncer", "acOper", "descOper", "Inimă", "Inima", "Inimi", "consIderă", "consIder", "consIderi",
    "mIjloc", "mIjlocul", "mIjloace", "artIcol", "artIcole", "profEsor", "profEsori", "profEsoară", "profEsoare",
    "tErmen", "tErmene", "Aer", "Aerul", "sIngura", "sIngurii", "sIngurele", "sIngurei", "actOr", "actOri",
    "ridicA", "aplicA", "explicA", "indicA", "implicA", "complicA", "comunicA", "dedicA", "predicA", "ridicAt",
    "pOate", "trEbuie", "lUmii", "lUnii", "țĂrii", "pÂinii", "mÂinii", "cĂrții", "vIeții", "mOrții", "nOpții",
    "pĂrții", "fEței", "cUrții", "gUrii", "dUpă", "cÂnd", "fIindcă", "orIcât", "cÂt", "atÂt", "Aur", "Aurul", "Aurului", "sIngur", "sIngură", "sIngure", "sIngurul", "sIgur", "sIgură", "sIguri", "sIgure", "Ungur", "Unguri", "priEten", "priEteni", "priEtenă", "priEtene", "alĂturi", "asEmenea", "nImeni", "dIncolo", "dOctor", "dOctori", "Ordin",
    "Ordine", "Ordinea", "sUflet", "sUflete", "Înger", "Îngeri", "Umeri", "Umerii", "nUmere", "tIneri",
    "tInerii", "tInere", "sUferă", "zgOmot", "zgOmote", "lIngură", "mĂtură", "pĂcură", "pUlbere", "pUlberea",
    "pUlberi", "Aripă", "Aripa", "Aripi", "mObilă", "mObila", "mObile", "sIlabă", "sIlaba", "sIlabe",
    "pEștera", "pEșteri", "cUmpăna", "cUmpene", "lEbăda", "lEbede", "pĂsări", "vEsel", "vEseli",
    "Optulea", "pAtrulea", "cIncilea", "Șaselea", "Șaptelea", "nOuălea", "zEcelea", "sUtelea", "mIilea",
    "cafeA", "cureA", "măseA", "nuiA", "perdeA", "podeA", "șoseA", "rețeA", "cafeneA", "basmA", "vopseA",
    "pijamA", "canapeA", "aveA", "puteA", "vedeA", "păreA", "plăceA", "cădeA", "ședeA", "tăceA", "zăceA",
    "prevedeA", "revedeA", "dureA", "încăpeA", "apăreA", "dispăreA", "veneA", "vorbeA", "trăiA", "scriA",
    "citeA", "iubeA", "simțeA", "doreA", "munceA", "deveneA", "primeA", "găseA", "priveA", "mureA", "opreA",
    "porneA", "soseA", "reușeA", "foloseA", "auzeA", "plăteA", "ieșeA", "dormeA", "fugeA", "numeA",
    "trebuiA", "trEbuie", "gIurgiu", "pustIu", "pustIul", "târzIu", "sibIu", "sibIului", "aurIu", "argintIu",
    "arămIu", "cenușIu", "cafenIu", "castanIu", "vișinIu", "portocalIu", "ruginIu", "stacojIu", "trandafirIu",
    "fIii", "relIgie", "relIgia", "relIgii", "chEstie", "chEstia", "chEstii", "bEstie", "modEstie", "custOdie",
    "cOpie", "cOpia", "germAnia", "germAniei", "spAnia", "spAniei", "polOnia", "polOniei", "sErbia", "japOnia",
    "India", "bElgia", "suEdia", "norvEgia", "soțIe", "soțIa", "soțIi", "soțIile", "soțIilor", "tărIe",
    "tărIa", "mândrIe", "mândrIa", "bogățIe", "bogățIa", "datorIe", "datorIa", "teorIe", "teorIa",
    "categorIe", "categorIa", "poezIe", "poezIa", "bucurIe", "bucurIa", "călătorIe", "călătorIa", "meserIe",
    "meserIa", "mărturIe", "mărturIa", "veselIe", "veselIa", "frizerIe", "frizerIa", "galerIe", "galerIa",
    "loterIe", "loterIa", "bijuterIe", "bijuterIa", "periferIe", "periferIa", "fantezIe", "fantezIa",
    "burghezIe", "burghezIa", "galaxIe", "galaxIa", "melancolIe", "melancolIa", "sărăcIe", "sărăcIa",
    "căsnicIe", "căsnicIa", "amicIe", "feerIe", "istorIa", "istorIe", "glorIe", "glorIa",
];

/// Second/third-conjugation verb stems whose 1pl/2pl endings `-em -eți` keep the stem's
/// stress (`fˈatʃem`, `spˈunetsʲʲ`); other `-em/-eți` words are stressed on the ending.
/// Their imperfect `-ea` is stressed on the ending (`spunˈea`).
const STEM_STRESS_EM: &[&str] = &[
    "fac", "merg", "spun", "zic", "duc", "pun", "cred", "trec", "cer", "scri", "țin", "bat", "vind", "ajung",
    "plâng", "cunoașt", "nașt", "rămân", "pierd", "râd", "prind", "cuprind", "deschid", "închid", "aprind",
    "trimit", "permit", "promit", "admit", "înțeleg", "aleg", "culeg", "strâng", "ating", "împing", "cresc",
    "cunosc", "rup", "umpl", "împart", "ard", "întrec", "petrec", "frâng", "înving", "sting", "ling", "mulg",
    "ucid", "decid", "creșt", "făc", "ofer", "acoper", "descoper", "sufer", "întoarc", "convinc", "trag", "distrug", "ajung",
    "încep", "concep", "pricep", "încap", "desprind", "surprind", "întind", "vând", "pătrund", "răspund",
    "ascund", "întrerup", "corup", "fierb", "scad", "zbat", "străbat", "dezbat", "combat", "abat",
];

/// Frequent 4th-conjugation infinitives (present in -esc, so the form is unambiguous):
/// their final i is a full stressed vowel (vorbˈi), not the whispered offglide.
const INFINITIVES_I: &[&str] = &[
    "vorbi", "citi", "iubi", "găsi", "privi", "muri", "dori", "opri", "porni", "sosi", "gândi", "reuși", "sfârși",
    "folosi", "veni", "deveni", "reveni", "primi", "mulțumi", "plăti", "stabili", "împlini", "ghici", "trăi",
    "lovi", "hotărî", "clădi", "zidi", "munci", "izbuti", "sfătui", "povesti", "pregăti", "pricepe",
    "trezi", "sosi", "greși", "glumi", "grăbi", "rosti", "umili", "goli", "mări", "mâhni", "lipi", "topi",
    "orbi", "robi", "slăbi", "înțepeni", "pomeni", "amăgi", "părăsi", "pedepsi", "răni", "zări", "jigni",
    "ocoli", "cuceri", "zdrobi", "hrăni", "ferici", "deosebi", "vesti", "cinsti", "pândi", "sorbi",
    "dobândi", "înmulți", "împărți", "numi", "potrivi", "lipsi", "înlocui", "locui", "construi", "contribui",
    "constitui", "atribui", "bănui", "chinui", "alcătui", "prăbuși", "obosi", "osteni",
    "nimeri", "împărtăși", "înfăptui", "zugrăvi", "ucenici", "roși", "albi", "învechi", "îmbogăți",
    "îmbătrâni", "întineri", "înnoi", "înflori", "înveli", "închipui", "înghiți", "înlesni",
    "înnebuni", "însoți", "însuși", "întări", "întâlni", "întocmi", "întregi", "înțelepți",
    "învălui", "învârti", "lămuri", "liniști", "mărturisi", "mântui", "mistui", "năpusti",
    "necăji", "nenoroci", "obișnui", "ocroti", "odihni", "oglindi", "orândui", "osândi", "pârî", "plictisi",
    "potoli", "prăpădi", "prelungi", "pricinui", "prigoni", "răsplăti", "răzbuni", "rătăci", "regăsi", "risipi",
    "sili", "smeri", "socoti", "stărui", "străluci", "stăpâni", "tăinui", "tâlcui", "tocmi", "tulbura",
    "urmări", "urzi", "vrăji", "zăbovi", "citi", "clipi", "coji", "dărui", "dobori", "domoli",
    "dospi", "dumiri", "făgădui", "găzdui", "gâtui", "hăitui", "hăpăi", "hoinări", "ieftini", "iscăli", "iscodi",
    "isprăvi", "împodobi", "împotrivi", "împuternici", "înăbuși", "înălți", "încălzi", "încălți", "înfiora",
    "îngriji", "îngrozi", "înjosi", "înrăi", "înveseli",
    "învinui", "învinovăți", "mângâi", "mâzgăli", "mocni", "moțăi", "mucezi", "murdări", "obloji", "pălmui",
    "păzi", "pipăi", "plesni", "pomeni", "porunci", "poticni", "preamări", "prețui", "prihăni",
    "scrânti", "sfinți", "slobozi", "sluji", "smuci", "sprijini", "stabilì",
    "strivi", "suci", "șovăi", "tămădui", "tânji", "trebui", "trudi", "zgâlțâi", "zori",
    "zvârli",
];

/// A fused clitic group (`s-a`, `într-o`) when one side is this short.
fn clitic_part(p: &str) -> bool {
    let n = p.chars().count();
    n <= 2 || !p.chars().any(is_v) || matches!(p, "într" | "dintr" | "printr")
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'ă' | 'â' | 'e' | 'i' | 'î' | 'o' | 'u')
}

/// Normalise a token: cedilla ș/ț, y → i, letters only.
fn norm(w: &str) -> String {
    w.chars()
        .filter(|c| c.is_alphabetic())
        .map(|c| match c {
            'ş' => 'ș',
            'ţ' => 'ț',
            'y' => 'i',
            'ã' => 'ă',
            _ => c,
        })
        .collect()
}

fn fixed() -> &'static HashMap<&'static str, &'static str> {
    static M: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    M.get_or_init(|| FIXED.iter().copied().collect())
}

/// Lexical stress: word -> letter index of the stressed vowel.
fn lexicon() -> &'static HashMap<String, usize> {
    static M: OnceLock<HashMap<String, usize>> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::with_capacity(STRESS.len());
        for e in STRESS {
            let mut idx = 0;
            let mut lw = String::with_capacity(e.len());
            for (k, c) in e.chars().enumerate() {
                if c.is_uppercase() {
                    idx = k;
                    lw.extend(c.to_lowercase());
                } else {
                    lw.push(c);
                }
            }
            m.entry(lw).or_insert(idx);
        }
        m
    })
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    let w = w.trim_matches(|c: char| !c.is_alphanumeric());
    let w = w.replace('ş', "ș").replace('ţ', "ț");
    if let Some(p) = fixed().get(w.as_str()) {
        return (*p).to_string();
    }
    // hyphen / apostrophe: a clitic group fuses into one word, a compound is read in parts
    if w.contains('-') || w.contains('\'') {
        let parts: Vec<&str> = w.split(['-', '\'']).filter(|p| p.chars().any(|c| c.is_alphabetic())).collect();
        if parts.is_empty() {
            return String::new();
        }
        if parts.len() == 2 && (clitic_part(parts[0]) || clitic_part(parts[1])) {
            return word(&parts.concat());
        }
        let outs: Vec<String> = parts.iter().map(|p| word(p)).filter(|p| !p.is_empty()).collect();
        return outs.join(" ");
    }
    let lw = norm(&w);
    if lw.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = lw.chars().collect();
    if !chars.iter().any(|&c| is_v(c)) {
        return spell_consonants(&chars);
    }
    let level = if UNSTRESSED.contains(&lw.as_str()) {
        Level::None
    } else if SECONDARY.contains(&lw.as_str()) {
        Level::Secondary
    } else {
        Level::Primary
    };
    let mut segs = segment(&chars);
    let nuclei: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| s.nucleus).map(|(i, _)| i).collect();
    if nuclei.is_empty() {
        return segs.iter().map(|s| s.ph).collect();
    }
    let letter = stress_letter(&chars, &segs);
    let stressed = nucleus_at(&segs, &nuclei, letter);
    // an unstressed hiatus i before a final u is a glide (mˈedju); stressed it stays (tɨrzˈiʊ)
    let mut primary = stressed;
    if let Some(k) = segs.iter().position(|s| s.iu) {
        if k != nuclei[stressed] {
            segs[k].ph = "j";
            segs[k].nucleus = false;
            segs[k + 1].ph = "u";
            primary = segs[..nuclei[stressed]].iter().filter(|s| s.nucleus).count();
        } else {
            segs[k + 1].ph = "ʊ";
            segs[k + 1].nucleus = false;
        }
    }
    render(&segs, primary, level)
}

#[derive(Clone, Copy, PartialEq)]
enum Level {
    Primary,
    Secondary,
    None,
}

#[derive(Clone, Debug)]
struct Seg {
    ph: &'static str,
    nucleus: bool,
    /// letter index this segment came from
    letter: usize,
    /// hiatus `i` before a word-final `u`: resolved by stress
    iu: bool,
}

/// consonant
fn c(ph: &'static str, letter: usize) -> Seg {
    Seg { ph, nucleus: false, letter, iu: false }
}
/// glide (j w ɪ ʊ) or the palatal offglide
fn g(ph: &'static str, letter: usize) -> Seg {
    c(ph, letter)
}
/// nucleus
fn v(ph: &'static str, letter: usize) -> Seg {
    Seg { ph, nucleus: true, letter, iu: false }
}

/// Index of the nucleus containing letter `letter` (or the last one before it).
fn nucleus_at(segs: &[Seg], nuclei: &[usize], letter: usize) -> usize {
    let mut best = 0;
    for (k, &ix) in nuclei.iter().enumerate() {
        if segs[ix].letter <= letter {
            best = k;
        } else {
            break;
        }
    }
    best
}

/// Letters -> segments (no stress yet).
fn segment(ch: &[char]) -> Vec<Seg> {
    let n = ch.len();
    let at = |i: usize| -> char { if i < n { ch[i] } else { '#' } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 4);
    let word: String = ch.iter().collect();
    // 4th-conjugation infinitives (-esc verbs): the final i is a full stressed vowel
    let inf_i = INFINITIVES_I.contains(&word.as_str());
    let mut i = 0usize;
    while i < n {
        let c0 = ch[i];
        let nx = at(i + 1);
        // doubled consonant letters (înnoda, Bill): one phone; cc/gg before e/i differ (accent)
        if i > 0 && ch[i - 1] == c0 && !is_v(c0) && !(matches!(c0, 'c' | 'g') && matches!(nx, 'e' | 'i')) {
            i += 1;
            continue;
        }
        if is_v(c0) {
            let start = i;
            let mut j = i;
            while j < n && is_v(ch[j]) {
                j += 1;
            }
            // consonant + liquid cluster right before the run (membri, patrie)
            let cluster = start >= 2 && matches!(ch[start - 1], 'r' | 'l') && !is_v(ch[start - 2]) && ch[start - 2] != 'h';
            let prev_letter = if start > 0 { ch[start - 1] } else { '#' };
            // real, creat, ideal, teatru, ocean: e-a in hiatus, not the diphthong
            let ea_hiatus = (start == 1 && (word.starts_with("rea") || word.starts_with("teatr")))
                || (start == 2 && (word.starts_with("crea") || word.starts_with("idea") || word.starts_with("ocean")));
            if ea_hiatus && ch[start] == 'e' && at(start + 1) == 'a' {
                segs.push(v("e", start));
                vowel_run(&ch[start + 1..j], start + 1, j == n, 'e', false, &mut segs);
                i = j;
                continue;
            }
            vowel_run(&ch[start..j], start, j == n, prev_letter, cluster || (j == n && inf_i), &mut segs);
            i = j;
            continue;
        }
        match c0 {
            'c' | 'g' => {
                let hard: &'static str = if c0 == 'c' { "k" } else { "ɡ" };
                let soft: &'static str = if c0 == 'c' { "tʃ" } else { "dʒ" };
                if nx == 'h' {
                    segs.push(c(hard, i));
                    i += 2;
                    // chi + vowel: the i is a glide (kjˈar); che + a: diphthong handled by the run
                    if at(i) == 'i' && is_v(at(i + 1)) {
                        segs.push(g("j", i));
                        i += 1;
                    }
                    continue;
                }
                if matches!(nx, 'e' | 'i') {
                    segs.push(c(soft, i));
                    // non-final ci/gi + vowel (the gold's majority readings): ciu/cio → tʃu tʃo
                    // (tʃudˈat, pitʃˈor), gia/gio/giu → dʒjV, cia → tʃia (hiatus, by the vowel run)
                    if nx == 'i' && matches!(at(i + 2), 'a' | 'o' | 'u') && i + 3 != n {
                        if c0 == 'c' && at(i + 2) != 'a' {
                            i += 2;
                            continue;
                        }
                        if c0 == 'g' {
                            segs.push(g("j", i + 1));
                            i += 2;
                            continue;
                        }
                    }
                } else {
                    segs.push(c(hard, i));
                }
            }
            'x' => {
                if i == 1 && ch[0] == 'e' && is_v(nx) {
                    segs.push(c("ɡ", i));
                    segs.push(c("z", i));
                } else {
                    segs.push(c("k", i));
                    segs.push(c("s", i));
                }
            }
            'n' => segs.push(c(if nx == 'g' { "ŋ" } else { "n" }, i)),
            'ș' => segs.push(c("ʃ", i)),
            'ț' => segs.push(c("ts", i)),
            'j' => segs.push(c("ʒ", i)),
            'q' => segs.push(c("k", i)),
            'w' => segs.push(g("w", i)),
            'k' => segs.push(c("k", i)),
            'b' => segs.push(c("b", i)),
            'd' => segs.push(c("d", i)),
            'f' => segs.push(c("f", i)),
            'h' => segs.push(c("h", i)),
            'l' => segs.push(c("l", i)),
            'm' => segs.push(c("m", i)),
            'p' => segs.push(c("p", i)),
            'r' => segs.push(c("r", i)),
            's' => segs.push(c("s", i)),
            't' => segs.push(c("t", i)),
            'v' => segs.push(c("v", i)),
            'z' => segs.push(c("z", i)),
            _ => {}
        }
        i += 1;
    }
    // r is a tap between a vowel and a vowel or palatal glide
    for k in 1..segs.len().saturating_sub(1) {
        if segs[k].ph == "r" && segs[k - 1].nucleus && (segs[k + 1].nucleus || matches!(segs[k + 1].ph, "j" | "ʲ" | "ʲʲ")) {
            segs[k].ph = "ɾ";
        }
    }
    segs
}

fn vowel_run(v_: &[char], start: usize, final_: bool, prev: char, cluster: bool, segs: &mut Vec<Seg>) {
    let m = v_.len();
    let at = |k: usize| -> char { if k < m { v_[k] } else { '#' } };
    let after_cons = prev != '#';
    let mut k = 0usize;
    while k < m {
        let ch = v_[k];
        let nx = at(k + 1);
        let li = start + k;
        match ch {
            'a' => segs.push(v("a", li)),
            'ă' => segs.push(v("ə", li)),
            'â' | 'î' => segs.push(v("ɨ", li)),
            'e' => {
                if nx == 'a' {
                    segs.push(v("ea", li)); // diphthong; aceea: e + ea
                    k += 2;
                    continue;
                }
                segs.push(v("e", li));
            }
            'o' => {
                if nx == 'a' {
                    segs.push(v("ɔa", li));
                    k += 2;
                    continue;
                }
                segs.push(v("o", li));
            }
            'i' => {
                if is_v(nx) && nx != 'i' {
                    if k > 0 || !after_cons {
                        // between vowels or word-initial: glide (femˈeje, jˈar)
                        segs.push(g("j", li));
                    } else if final_ && m == 2 {
                        // final -ia -ie -io: hiatus (famˈilie); -iu: decided by stress
                        let mut s = v("i", li);
                        s.iu = nx == 'u';
                        segs.push(s);
                    } else if final_ && m == 3 && at(1) == 'e' && at(2) == 'i' {
                        segs.push(v("i", li)); // -iei
                    } else if !cluster && matches!(nx, 'a' | 'e' | 'ă') && matches!(prev, 'p' | 'b' | 'f' | 'v' | 'm') {
                        segs.push(g("j", li)); // piatră, fier, viață
                    } else {
                        segs.push(v("i", li)); // hiatus: prieten, național, știut, italian
                    }
                } else if nx == 'i' {
                    segs.push(v("i", li));
                    k += 1;
                    if at(k + 1) == 'i' {
                        // copiii: i j i
                        segs.push(g("j", li + 1));
                        segs.push(v("i", li + 2));
                        k += 2;
                        continue;
                    }
                    if k + 1 == m && final_ {
                        segs.push(g("ɪ", li + 1));
                    } else {
                        segs.push(v("i", li + 1)); // fiind, știință: hiatus
                    }
                    k += 1;
                    continue;
                } else if k > 0 {
                    // after a vowel: offglide, or hiatus in -ui/-ăi verbs
                    let pv = v_[k - 1];
                    let last = final_ && k + 1 == m;
                    let nuclei_before = segs.iter().filter(|s| s.nucleus).count();
                    let verb_hiatus = last && nuclei_before >= 2 && (pv == 'u' && prev != 'l' || pv == 'ă');
                    // trăit, construit, ruină: ă/u + i + consonant is a hiatus
                    let medial_hiatus = !last && matches!(pv, 'ă' | 'u') && !(pv == 'u' && nuclei_before == 0);
                    if verb_hiatus || medial_hiatus {
                        segs.push(v("i", li));
                    } else if let Some(&Seg { ph: "ea", .. }) = segs.last() {
                        segs.push(g("ɪ", li));
                    } else {
                        segs.push(g(if pv == 'o' { "j" } else { "ɪ" }, li));
                    }
                } else if final_ && m == 1 && after_cons {
                    let nuclei_before = segs.iter().filter(|s| s.nucleus).count();
                    if nuclei_before == 0 || cluster || prev == 'x' {
                        segs.push(v("i", li));
                    } else {
                        let pal = matches!(segs.last().map(|s| s.ph), Some("tʃ") | Some("dʒ") | Some("k") | Some("ɡ"));
                        segs.push(g(if pal { "ʲ" } else { "ʲʲ" }, li));
                    }
                } else {
                    segs.push(v("i", li));
                }
            }
            'u' => {
                if is_v(nx) && nx != 'i' {
                    if k > 0 {
                        segs.push(g("w", li)); // ziua, două
                    } else {
                        segs.push(v("u", li)); // lua, continua
                    }
                } else if nx == 'i' {
                    segs.push(v("u", li));
                } else if k > 0 && final_ && k + 1 == m {
                    // final diphthong: aʊ eʊ oʊ əʊ iʊ, but eaw; -iu after a consonant waits for the stress
                    if segs.last().map_or(false, |s| s.iu) {
                        segs.push(v("u", li));
                    } else if let Some(&Seg { ph: "ea", .. }) = segs.last() {
                        segs.push(g("w", li));
                    } else {
                        segs.push(g("ʊ", li));
                    }
                } else {
                    segs.push(v("u", li)); // aur, neutru, european: hiatus
                }
            }
            _ => {}
        }
        k += 1;
    }
}

/// Letter index of the stressed vowel.
fn stress_letter(w: &[char], segs: &[Seg]) -> usize {
    let nuc: Vec<usize> = segs.iter().filter(|s| s.nucleus).map(|s| s.letter).collect();
    let n = w.len();
    if nuc.len() <= 1 {
        return nuc.first().copied().unwrap_or(0);
    }
    let s: String = w.iter().collect();
    if let Some(&p) = lexicon().get(&s) {
        return p;
    }
    let ends = |suf: &str| s.ends_with(suf);
    let stem = |cut: usize| -> Option<usize> {
        if cut >= n {
            return None;
        }
        let st = &w[..n - cut];
        if !st.iter().any(|&c| is_v(c)) {
            return None;
        }
        let ss = segment(st);
        Some(stress_letter(st, &ss))
    };
    let count_v = |cut: usize| w[..n.saturating_sub(cut)].iter().filter(|&&c| is_v(c)).count();
    // definite article / plural suffixes keep the stem's stress
    if ends("ului") && n > 4 && !is_v(w[n - 5]) {
        if let Some(p) = stem(4) {
            return p;
        }
    }
    if ends("lui") && n > 3 && is_v(w[n - 4]) {
        if let Some(p) = stem(3) {
            return p;
        }
    }
    if ends("ul") && n > 2 {
        // muzeul, fiul, noul: the u belongs to the stem
        if let Some(p) = stem(if is_v(w[n - 3]) { 1 } else { 2 }) {
            return p;
        }
    }
    if ends("ilor") || ends("elor") || (ends("lor") && n > 3 && is_v(w[n - 4])) {
        if let Some(p) = stem(3) {
            return p;
        }
    }
    if ends("urile") || ends("urilor") {
        let cut = if ends("urile") { 5 } else { 6 };
        if let Some(p) = stem(cut) {
            return p;
        }
    }
    if ends("uri") && n > 3 && !is_v(w[n - 4]) {
        if let Some(p) = stem(3) {
            return p;
        }
    }
    if (ends("ele") || ends("ile")) && count_v(2) >= 2 {
        if let Some(p) = stem(2) {
            return p;
        }
    }
    if ends("ii") && n > 2 && (!is_v(w[n - 3]) || w[n - 3] == 'o') {
        // apărării, dezvoltării: genitive of an -are noun; fotografii: plural of an -ie noun
        if ends("ării") || ie_suffix_stressed(&s[..s.len() - 2]) {
            return nuc[nuc.len() - 2];
        }
        if let Some(p) = stem(2) {
            return p;
        }
    }
    if ends("iei") && n > 3 && !is_v(w[n - 4]) {
        // genitive of an -ie noun: as the noun
        let mut st: Vec<char> = w[..n - 2].to_vec();
        st.push('e');
        let ss = segment(&st);
        return stress_letter(&st, &ss);
    }
    if ends("sprezece") {
        return nuc[0];
    }
    if INFINITIVES_I.contains(&s.as_str()) {
        return nuc[nuc.len() - 1];
    }
    if ends("î") {
        return nuc[nuc.len() - 1];
    }
    // -ic adjectives: penultimate; -ică/-ice/-ici: antepenultimate
    if ends("ic") || ends("bil") || ends("ctor") || (ends("col") && !ends("tocol")) {
        return nuc[nuc.len() - 2];
    }
    if ends("gine") && nuc.len() >= 3 {
        return nuc[nuc.len() - 3];
    }
    if ends("ifica") {
        return nuc[nuc.len() - 1];
    }
    if ends("ica") && nuc.len() >= 3 {
        return nuc[nuc.len() - 3];
    }
    // -ere nouns of 3rd-conjugation verbs: kondˈutʃere, krˈeʃterea, trˈetʃeri
    if ends("ere") || ends("erea") || ends("eri") {
        let cut = if ends("erea") { 4 } else { 3 };
        let st: String = w[..n - cut].iter().collect();
        if STEM_STRESS_EM.iter().any(|v| st.ends_with(v)) {
            if let Some(p) = stem(cut) {
                return p;
            }
        }
    }
    // feminine genitive -ei (ekˈipei, eurˈopei); two-syllable words are plurals (femˈei, idˈei)
    if ends("ei") && !ends("iei") && nuc.len() >= 2 && !is_v(w[n - 3]) {
        return nuc[nuc.len() - 2];
    }
    if (ends("ică") || ends("ice") || ends("cole")) && nuc.len() >= 3 {
        return nuc[nuc.len() - 3];
    }
    if ends("ici") && nuc.len() >= 2 {
        return nuc[nuc.len() - 2];
    }
    // hiatus -ie -ia -io -iu after a consonant
    if n >= 3 && w[n - 2] == 'i' && matches!(w[n - 1], 'e' | 'a' | 'o' | 'u') && !is_v(w[n - 3]) && nuc.len() >= 3 {
        let i_stressed = if w[n - 1] == 'u' { iu_stressed(&s) } else { ie_stressed(&s) };
        return if i_stressed { nuc[nuc.len() - 2] } else { nuc[nuc.len() - 3] };
    }
    // -ea: verbs and a few nouns on the ending, articulated nouns before it
    if ends("ea") && nuc.len() >= 2 {
        // articulated noun whose base is in the lexicon (drˈaɡostea, mˈardʒinea)
        if let Some(&p) = lexicon().get(&s[..s.len() - 1]) {
            return p;
        }
        let st = &s[..s.len() - 2];
        if STEM_STRESS_EM.iter().any(|v| st.ends_with(v) && v.len() >= 3) {
            return nuc[nuc.len() - 1];
        }
        // dˈunəɾea, pˈasəɾea: ă before the ending is not stressed
        if nuc.len() >= 3 && w[nuc[nuc.len() - 2]] == 'ă' {
            return nuc[nuc.len() - 3];
        }
        return nuc[nuc.len() - 2];
    }
    // final syllable with ă is unstressed (nˈumər, ˈumər)
    if n >= 3 && w[n - 2] == 'ă' && matches!(w[n - 1], 'r' | 'n' | 't' | 'l') && !is_v(w[n - 3]) {
        return nuc[nuc.len() - 2];
    }
    // 2nd/3rd conjugation -em/-eți keep the stem stress
    if ends("em") || ends("eți") {
        let cut = if ends("em") { 2 } else { 3 };
        let st: String = w[..n - cut].iter().collect();
        if STEM_STRESS_EM.iter().any(|v| st.ends_with(v)) {
            if let Some(p) = stem(cut) {
                return p;
            }
        }
    }
    // default: penultimate if the last letter is a syllabic vowel, else final
    let last_syllabic = *nuc.last().unwrap() == n - 1;
    if last_syllabic {
        let mut k = nuc.len() - 2;
        // ă is not stressed in the penult of a longer word (pˈasəɾe, sˈɨmbətə) except -ăște
        if w[nuc[k]] == 'ă' && nuc.len() >= 3 && !ends("ăște") && !ends("ăsc") && !ends("ăsem") && !ends("ăseși") {
            // ɨnvətsˈa, kumpəɾˈa: ă before a final -a marks an infinitive
            if w[n - 1] == 'a' {
                return nuc[nuc.len() - 1];
            }
            k -= 1;
        }
        // kˈauzə, lˈaudə, nˈeutru: the u of a hiatus au/eu/ou is not stressed
        if nuc.len() >= 3 && w[nuc[k]] == 'u' && nuc[k] > 0 && matches!(w[nuc[k] - 1], 'a' | 'o') && (nuc[k] == 1 || !is_v(w[nuc[k] - 2])) {
            k -= 1;
        }
        nuc[k]
    } else {
        nuc[nuc.len() - 1]
    }
}

/// Greek/Romance suffixes whose -ie noun is stressed on the i (biolodʒˈie, bukətəɾˈie).
fn ie_suffix_stressed(st: &str) -> bool {
    ["ăr", "log", "graf", "craț", "nom", "sof", "metr", "ped", "pat", "fon", "tom", "terap", "arh", "urg", "erg",
     "dem", "nol", "scop", "fob", "fil"]
        .iter()
        .any(|suf| st.ends_with(suf))
}

/// Is the `i` of a final -ie/-ia/-io stressed? (prostˈie vs famˈilie)
fn ie_stressed(s: &str) -> bool {
    let st = &s[..s.len() - 2]; // before the i + vowel
    let last = st.chars().last().unwrap_or('#');
    if ie_suffix_stressed(st) {
        return true;
    }
    // ț s z l r (and st): before the i; m f g d t n h p b: on the i
    !matches!(last, 'ț' | 's' | 'z' | 'l' | 'r' | 'x' | 'v' | 'c')
}

/// Is the `i` of a final -iu stressed (tɨrzˈiʊ) or a glide (mˈedju)?
fn iu_stressed(s: &str) -> bool {
    let st = &s[..s.len() - 2];
    st.ends_with("ăr") || st.ends_with("ăl") || st.ends_with("str") || st.ends_with("iz") || st.ends_with("ch")
        || st.ends_with('g') && !st.ends_with("ag") && !st.ends_with("eg")
}

/// Output with stress marks.
fn render(segs: &[Seg], primary: usize, level: Level) -> String {
    let mut out = String::with_capacity(segs.len() * 2 + 4);
    let total = segs.iter().filter(|s| s.nucleus).count();
    let mut nk = 0usize;
    for s in segs {
        if s.nucleus {
            let mark = match level {
                Level::None => "",
                Level::Secondary => if nk == primary { "ˌ" } else { "" },
                Level::Primary => {
                    if nk == primary {
                        "ˈ"
                    } else if (nk + 1 < primary && nk % 2 == 0) || (nk == primary + 2 && nk < total) {
                        "ˌ"
                    } else {
                        ""
                    }
                }
            };
            out.push_str(mark);
            nk += 1;
        }
        out.push_str(s.ph);
    }
    out
}

/// Letter names for a vowel-less token.
fn spell_consonants(chars: &[char]) -> String {
    let names = LETTERS.unwrap();
    let mut out = String::new();
    let n = chars.len();
    for (k, &ch) in chars.iter().enumerate() {
        let name = match ch {
            'ș' => "ʃe",
            'ț' => "tse",
            'a'..='z' => names[(ch as u8 - b'a') as usize],
            _ => continue,
        };
        let mark = if k + 1 == n { "ˈ" } else { "ˌ" };
        let pos = name.find(|c: char| "aeiou".contains(c)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    out
}

/// Sentence pass: after the infinitive marker `a`, a word ending in `-a` (two
/// or more syllables) or in consonant + `i` is an infinitive stressed on the
/// ending (`a lukrˈa`, `a vorbˈi`).
pub fn join(outs: &mut [Out]) {
    for k in 1..outs.len() {
        let (a, b) = outs.split_at_mut(k);
        let prev = &a[k - 1];
        let cur = &mut b[0];
        if prev.src != "a" || prev.pause_after || cur.ph.is_empty() {
            continue;
        }
        let src = norm(&cur.src);
        let chars: Vec<char> = src.chars().collect();
        let n = chars.len();
        if n < 3 || lexicon().contains_key(&src) && !src.ends_with('a') {
            continue;
        }
        let last = chars[n - 1];
        let before = chars[n - 2];
        if last == 'a' && !is_v(before) && before != 'i' && chars.iter().filter(|&&c| is_v(c)).count() >= 2 {
            // infinitive -a: final stress
            let segs = segment(&chars);
            let nn = segs.iter().filter(|s| s.nucleus).count();
            cur.ph = render(&segs, nn - 1, Level::Primary);
        } else if last == 'i' && !is_v(before) && !matches!(before, 'r' | 'l') && !src.ends_with("ii") {
            // infinitive -i: syllabic stressed i
            let mut segs = segment(&chars[..n - 1]);
            segs.push(v("i", n - 1));
            let nn = segs.iter().filter(|s| s.nucleus).count();
            if nn >= 2 {
                cur.ph = render(&segs, nn - 1, Level::Primary);
            }
        }
    }
}

const ONES: [&str; 20] = [
    "zero", "unu", "doi", "trei", "patru", "cinci", "șase", "șapte", "opt", "nouă", "zece", "unsprezece",
    "doisprezece", "treisprezece", "paisprezece", "cincisprezece", "șaisprezece", "șaptesprezece",
    "optsprezece", "nouăsprezece",
];
const TENS: [&str; 10] = ["", "", "douăzeci", "treizeci", "patruzeci", "cincizeci", "șaizeci", "șaptezeci", "optzeci", "nouăzeci"];

fn below_100(n: u128, fem: bool, out: &mut Vec<String>) {
    if n < 20 {
        let w = match (n, fem) {
            (1, true) => "una",
            (2, true) => "două",
            (12, true) => "douăsprezece",
            _ => ONES[n as usize],
        };
        out.push(w.into());
        return;
    }
    out.push(TENS[(n / 10) as usize].into());
    if n % 10 > 0 {
        out.push("și".into());
        below_100(n % 10, fem, out);
    }
}

fn below_1000(n: u128, fem: bool, out: &mut Vec<String>) {
    let h = n / 100;
    let r = n % 100;
    if h == 1 {
        out.push("o".into());
        out.push("sută".into());
    } else if h > 1 {
        below_100(h, true, out);
        out.push("sute".into());
    }
    if r > 0 || n == 0 {
        below_100(r, fem, out);
    }
}

/// Cardinal number words (masculine: `doi`, `douăzeci și unu`).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["zero".into()];
    }
    let mut out = Vec::new();
    const SCALES: [(u128, &str, &str, bool); 6] = [
        (1_000_000_000_000_000_000, "trilion", "trilioane", false),
        (1_000_000_000_000_000, "cvadrilion", "cvadrilioane", false),
        (1_000_000_000_000, "trilion", "trilioane", false),
        (1_000_000_000, "miliard", "miliarde", false),
        (1_000_000, "milion", "milioane", false),
        (1_000, "mie", "mii", true),
    ];
    let mut rest = n;
    for (val, one, many, fem) in SCALES {
        let q = rest / val;
        if q > 0 {
            if q == 1 {
                out.push(if fem { "o" } else { "un" }.into());
                out.push(one.into());
            } else {
                below_1000(q, fem, &mut out);
                if q >= 20 {
                    out.push("de".into());
                }
                out.push(many.into());
            }
            rest %= val;
        }
    }
    if rest > 0 {
        below_1000(rest, false, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "la sută",
        '$' => "dolari",
        '€' => "euro",
        '£' => "lire",
        '&' => "și",
        '+' => "plus",
        '@' => "a rond",
        '°' => "grade",
        '=' => "egal",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn basics() {
        for (w, p) in [
            ("casă", "kˈasə"),
            ("când", "kɨnd"),
            ("încă", "ˈɨnkə"),
            ("ce", "tʃˈe"),
            ("cinci", "tʃˈintʃʲ"),
            ("geam", "dʒˈeam"),
            ("ghid", "ɡˈid"),
            ("chiar", "kjˈar"),
            ("ochi", "ˈokʲ"),
            ("ședință", "ʃedˈintsə"),
            ("joc", "ʒˈok"),
            ("exact", "eɡzˈakt"),
            ("text", "tˈekst"),
            ("singur", "sˈiŋɡur"),
            ("bancă", "bˈankə"),
            ("fără", "fˈəɾə"),
            ("este", "jˈeste"),
            ("ea", "jˈa"),
            ("ei", "jˈeɪ"),
            ("era", "jeɾˈa"),
            ("exemplu", "eɡzˈemplu"),
            ("ţară", "tsˈaɾə"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn glides() {
        for (w, p) in [
            ("poate", "pˈɔate"),
            ("avea", "avˈea"),
            ("lumea", "lˈumea"),
            ("mai", "maɪ"),
            ("dau", "dˈaʊ"),
            ("nou", "nˈoʊ"),
            ("doi", "dˈoj"),
            ("lui", "lˈuɪ"),
            ("său", "sˈəʊ"),
            ("câine", "kˈɨɪne"),
            ("ziua", "zˈiwa"),
            ("două", "dˈowə"),
            ("femeie", "femˈeje"),
            ("ploaie", "plˈɔaje"),
            ("iarnă", "jˈarnə"),
            ("piatră", "pjˈatrə"),
            ("fier", "fjˈer"),
            ("prieten", "priˈeten"),
            ("național", "nˌatsionˈal"),
            ("familie", "famˈiliˌe"),
            ("familia", "famˈiliˌa"),
            ("prostie", "prostˈie"),
            ("mediu", "mˈedju"),
            ("târziu", "tɨrzˈiʊ"),
            ("fiu", "fˈiʊ"),
            ("copii", "kopˈiɪ"),
            ("anii", "ˈaniɪ"),
            ("copiii", "kopˈiji"),
            ("aur", "ˈaur"),
            ("cauză", "kˈauzˌə"),
            ("fiind", "fiˈind"),
            ("construit", "kˌonstruˈit"),
            ("muzeul", "muzˈeul"),
            ("european", "ˌeuɾopˈean"),
            ("lua", "luˈa"),
            ("bănui", "bˌənuˈi"),
            ("ciudat", "tʃudˈat"),
            ("cioban", "tʃobˈan"),
            ("social", "sˌotʃiˈal"),
            ("serviciu", "servˈitʃju"),
            ("membri", "mˈembri"),
            ("mergeau", "merdʒˈeaw"),
            ("aceea", "atʃˈeea"),
            ("României", "rˌomɨnˈieɪ"),
        ] {
            assert_eq!(word(&w.to_lowercase()), p, "{w}");
        }
    }

    #[test]
    fn stress() {
        for (w, p) in [
            ("oameni", "ˈɔamenʲʲ"),
            ("oamenilor", "ˈɔamenˌilor"),
            ("lucrurile", "lˈukruɾˌile"),
            ("anului", "ˈanulˌuɪ"),
            ("numărul", "nˈuməɾˌul"),
            ("politică", "polˈitikˌə"),
            ("public", "pˈublik"),
            ("nimic", "nimˈik"),
            ("duminică", "dumˈinikˌə"),
            ("sâmbătă", "sˈɨmbətˌə"),
            ("pasăre", "pˈasəɾˌe"),
            ("facem", "fˈatʃem"),
            ("faceți", "fˈatʃetsʲʲ"),
            ("lucrați", "lukrˈatsʲʲ"),
            ("vorbim", "vorbˈim"),
            ("hotărî", "hˌotəɾˈɨ"),
            ("început", "ˌɨntʃepˈut"),
            ("românia", "rˌomɨnˈia"),
            ("activitatea", "ˌaktivitˈatea"),
            ("asemenea", "asˈemenˌea"),
            ("internațional", "ˌinternˌatsionˈal"),
            ("care", "kˌaɾe"),
            ("de", "de"),
            ("într-o", "ˈɨntro"),
            ("s-a", "sa"),
            ("nord-vest", "nˈord vˈest"),
            ("douăzeci", "dˌowəzˈetʃʲ"),
            ("probabil", "probˈabil"),
            ("biserica", "bisˈeɾikˌa"),
            ("conducerea", "kondˈutʃeɾˌea"),
            ("echipei", "ekˈipeɪ"),
            ("veni", "venˈi"),
            ("unsprezece", "ˈunsprezˌetʃe"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), vec!["zero"]);
        assert_eq!(cardinal(21), vec!["douăzeci", "și", "unu"]);
        assert_eq!(cardinal(100), vec!["o", "sută"]);
        assert_eq!(cardinal(212), vec!["două", "sute", "doisprezece"]);
        assert_eq!(cardinal(1000), vec!["o", "mie"]);
        assert_eq!(cardinal(2000), vec!["două", "mii"]);
        assert_eq!(cardinal(25_000), vec!["douăzeci", "și", "cinci", "de", "mii"]);
        assert_eq!(cardinal(1_000_000), vec!["un", "milion"]);
        assert_eq!(cardinal(3_000_000), vec!["trei", "milioane"]);
    }
}
