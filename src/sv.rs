//! Swedish grapheme-to-phoneme rules, espeak-style notation.
//!
//! Central Standard Swedish. Stress is lexical but predictable from the
//! morphology for most words: initial in native words, on the suffix in the
//! Romance loans (-tion, -era, -ik, -ell, -ent, ...), unstressed prefixes
//! be-/för-, stressed particle prefixes (an-, av-, in-, upp-, ut-, ...) with
//! a secondary stress on the rest, and a small table of compound tails
//! (-het, -dom, -skap, -bar, -lös, ...) that keep their own length. Vowel
//! length follows from stress and syllable structure: a stressed vowel is long
//! before at most one consonant (and before rd rn rl), short before clusters,
//! doubles, ŋ and j; unstressed vowels are short.
//!
//! Notation follows espeak-ng's `sv` voice: long `ɑː iː eː uː oː yː ɛː øː`,
//! long u written `ʉ` (no length mark), short `a ɪ ɛ ɔ ʊ ɵ y œ`, the sj-sound
//! written `sx`, tj/kj/k+front `ɕ`, `rs` -> `ʂ`, `ʃ` for sch/sh/French ge,
//! `ŋ`, unstressed e `ə` after the stress, `ˈ` right before the stressed
//! vowel, `ˌ` on the other parts of a compound. Tone accents are not marked.

use std::path::Path;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

pub const IMPLEMENTED: bool = true;

/// Optional data-pack lexicon `sv.lex` (WikiPron, CC BY-SA; built by `tools/sv_lex.py`),
/// loaded lazily from `$POLYG2P_DATA`. Consulted after the built-in tables, and as an
/// extra source of compound parts.
fn data_lex() -> Option<&'static Lexicon> {
    static LEX_FILE: OnceLock<Option<Lexicon>> = OnceLock::new();
    LEX_FILE
        .get_or_init(|| {
            let dir = std::env::var_os("POLYG2P_DATA")?;
            Lexicon::open(&Path::new(&dir).join("sv.lex")).ok()
        })
        .as_ref()
}
pub const DECIMAL: &str = "komma";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "ɑː", "beː", "seː", "deː", "eː", "ɛf", "ɡeː", "hoː", "iː", "jiː", "koː", "ɛl", "ɛm", "ɛn", "uː", "peː", "kʉ",
    "ɛr", "ɛs", "teː", "ʉ", "veː", "dɵbəlveː", "ɛks", "yː", "sɛːta",
]);

/// Function words read without stress (espeak style), with their spoken forms.
const FUNC: &[(&str, &str)] = &[
    ("och", "ɔk"), ("att", "at"), ("det", "deː"), ("en", "ɛn"), ("ett", "ɛt"), ("den", "dɛn"), ("de", "dɔm"),
    ("dem", "dɔm"), ("som", "sɔm"), ("till", "tɪl"), ("för", "føːr"), ("med", "meːd"), ("på", "poː"), ("av", "ɑːv"),
    ("om", "ɔm"), ("är", "ɛːr"), ("jag", "jɑːɡ"), ("han", "han"), ("hon", "hʊn"), ("vi", "viː"), ("ni", "niː"),
    ("du", "dʉ"), ("kan", "kan"), ("man", "man"), ("har", "hɑːr"), ("var", "vɑːr"), ("från", "froːn"),
    ("så", "soː"), ("men", "mɛn"), ("inte", "ɪntə"), ("ska", "ska"), ("skall", "skal"), ("i", "iː"), ("än", "ɛn"),
    ("sig", "sɛj"), ("mig", "mɛj"), ("dig", "dɛj"), ("sin", "sɪn"), ("sitt", "sɪt"), ("sina", "siːna"),
    ("min", "mɪn"), ("mitt", "mɪt"), ("din", "dɪn"), ("ditt", "dɪt"), ("hans", "hans"), ("er", "eːr"),
    ("ert", "eːrt"), ("vår", "voːr"), ("vårt", "voːrt"), ("ju", "jʉ"), ("nog", "nuːɡ"), ("väl", "vɛːl"),
    ("där", "dɛːr"), ("här", "hɛːr"), ("då", "doː"), ("nu", "nʉ"), ("ej", "ɛj"), ("ut", "ʉt"), ("in", "ɪn"),
    ("upp", "ɵp"), ("ner", "neːr"), ("hos", "huːs"), ("vid", "viːd"), ("mot", "muːt"), ("ur", "ʉr"),
    ("hur", "hʉr"), ("vad", "vɑː"), ("vem", "vɛm"), ("när", "nɛːr"), ("ha", "hɑː"), ("de", "dɔm"),
    ("bli", "bliː"), ("blir", "bliːr"), ("ska", "ska"), ("få", "foː"), ("får", "foːr"), ("fick", "fɪk"),
];

/// Irregular words and frequent loans (stressed), in espeak notation.
const LEX: &[(&str, &str)] = &[
    ("säga", "sˈɛja"), ("säger", "sˈɛjər"), ("sagt", "sˈakt"), ("sade", "sˈɑː"), ("sa", "sˈɑː"), ("säg", "sˈɛj"),
    ("någon", "nˈoːɡɔn"), ("något", "nˈoːɡɔt"), ("några", "nˈoːɡra"), ("sådan", "sˈoːdan"), ("sådant", "sˈoːdant"),
    ("mycket", "mˈykə"), ("sedan", "sˈeːdan"), ("sen", "sˈɛn"), ("igen", "ɪjˈɛn"), ("också", "ˈɔksoː"),
    ("hade", "hˈadə"), ("gjorde", "jˈʊrdə"), ("gjort", "jˈʊrt"), ("gör", "jˈøːr"), ("göra", "jˈøːra"),
    ("ge", "jˈeː"), ("ger", "jˈeːr"), ("gav", "ɡˈɑːv"), ("gett", "jˈɛt"), ("gick", "jˈɪk"), ("gå", "ɡˈoː"),
    ("mer", "mˈeːr"), ("mera", "mˈeːra"), ("mest", "mˈɛst"), ("fler", "flˈeːr"), ("flera", "flˈeːra"),
    ("många", "mˈɔŋa"), ("mindre", "mˈɪndrə"), ("minst", "mˈɪnst"), ("bättre", "bˈɛtrə"), ("bäst", "bˈɛst"),
    ("stor", "stˈuːr"), ("stora", "stˈuːra"), ("stort", "stˈuːrt"), ("större", "stˈœrə"), ("störst", "stˈœʂt"),
    ("bra", "brˈɑː"), ("dålig", "dˈoːlɪɡ"), ("god", "ɡˈuːd"), ("gott", "ɡˈɔt"), ("goda", "ɡˈuːda"),
    ("dag", "dˈɑːɡ"), ("dagen", "dˈɑːɡən"), ("dagar", "dˈɑːɡar"), ("idag", "ɪdˈɑːɡ"), ("imorgon", "ɪmˈɔrɔn"),
    ("morgon", "mˈɔrɔn"), ("morgonen", "mˈɔrɔnən"), ("kväll", "kvˈɛl"), ("år", "ˈoːr"), ("året", "ˈoːrət"),
    ("tillbaka", "tɪlbˈɑːka"), ("tillsammans", "tɪlsˈamans"), ("tillräckligt", "tɪlrˈɛklɪkt"),
    ("tillräcklig", "tɪlrˈɛklɪɡ"), ("tillbaks", "tɪlbˈɑːks"), ("alldeles", "ˈaldeːləs"), ("särskilt", "sˈɛːʂɪlt"),
    ("särskild", "sˈɛːʂɪld"), ("särskilda", "sˈɛːʂɪlda"), ("kanske", "kˈanʃə"), ("människa", "mˈɛnɪʃa"),
    ("människor", "mˈɛnɪʃʊr"), ("människan", "mˈɛnɪʃan"), ("människors", "mˈɛnɪʃʊʂ"), ("bort", "bˈɔrt"),
    ("borta", "bˈɔrta"), ("kort", "kˈɔrt"), ("ort", "ˈuːrt"), ("art", "ˈɑːrt"), ("fart", "fˈɑːrt"),
    ("part", "pˈɑːrt"), ("snart", "snˈɑːrt"), ("karta", "kˈɑːrta"), ("arton", "ˈɑːrtɔn"), ("sort", "sˈɔrt"),
    ("hjärta", "jˈɛrta"), ("hjärtat", "jˈɛrtat"), ("hjärtan", "jˈɛrtan"), ("svart", "svˈart"), ("start", "stˈart"),
    ("förr", "fˈœr"), ("förra", "fˈœra"), ("först", "fˈœʂt"), ("första", "fˈœʂta"), ("förste", "fˈœʂtə"),
    ("före", "fˈøːrə"), ("föra", "fˈøːra"), ("förde", "fˈøːrdə"), ("fört", "fˈøːrt"), ("för", "fˈøːr"),
    ("hört", "hˈøːrt"), ("kört", "ɕˈøːrt"), ("lärt", "lˈɛːrt"), ("rört", "rˈøːrt"), ("hårt", "hˈoːrt"),
    ("lejon", "lˈeːjɔn"), ("kamera", "kˈɑːmɛra"), ("opera", "ˈuːpɛra"), ("resultat", "rɛsɵltˈɑːt"),
    ("gammal", "ɡˈamal"), ("gamla", "ɡˈamla"), ("antal", "ˈantɑːl"), ("tiotal", "tˈiːʊtɑːl"),
    ("hundratal", "hˈɵndratɑːl"), ("tusental", "tˈʉsəntɑːl"), ("paket", "pakˈeːt"), ("raket", "rakˈeːt"),
    ("planet", "planˈeːt"), ("budget", "bˈɵdjɛt"), ("hotell", "hʊtˈɛl"), ("och", "ˈɔk"), ("orm", "ˈuːrm"),
    ("son", "sˈoːn"), ("sova", "sˈoːva"), ("sov", "sˈoːv"), ("sover", "sˈoːvər"), ("lov", "lˈoːv"), ("lova", "lˈoːva"), ("lovar", "lˈoːvar"), ("kol", "kˈoːl"),
    ("ost", "ˈʊst"), ("ond", "ˈʊnd"), ("onda", "ˈʊnda"), ("bott", "bˈʊt"), ("onsdag", "ˈʊnsdɑːɡ"),
    ("blomma", "blˈʊma"), ("blommor", "blˈʊmʊr"), ("tom", "tˈuːm"), ("rolig", "rˈuːlɪɡ"), ("roligt", "rˈuːlɪkt"),
    ("bo", "bˈuː"), ("bor", "bˈuːr"), ("bodde", "bˈʊdə"), ("bott", "bˈʊt"), ("dom", "dˈɔm"), ("kom", "kˈɔm"),
    ("komma", "kˈɔma"), ("kommer", "kˈɔmər"), ("hem", "hˈɛm"), ("hemma", "hˈɛma"), ("rum", "rˈɵm"),
    ("dum", "dˈɵm"), ("fram", "frˈam"), ("sam", "sˈam"), ("kam", "kˈam"), ("dam", "dˈɑːm"), ("lim", "lˈiːm"),
    ("vän", "vˈɛn"), ("män", "mˈɛn"), ("mun", "mˈɵn"), ("nån", "nˈɔn"), ("nåt", "nˈɔt"), ("nåra", "nˈɔra"),
    ("energi", "ɛnɛrʃˈiː"), ("allergi", "alɛrʃˈiː"), ("kirurgi", "ɕɪrɵrʃˈiː"), ("orgel", "ˈɔrɡəl"),
    ("organ", "ɔrɡˈɑːn"), ("borgare", "bˈɔrjarə"), ("medborgare", "mˈeːdbɔrjarə"), ("sverige", "svˈɛrjə"),
    ("norge", "nˈɔrjə"), ("europa", "ɛʉrˈuːpa"), ("europeisk", "ɛʉrʊpˈeːɪsk"), ("eu", "ˈeːʉ"),
    ("teori", "tɛʊrˈiː"), ("video", "vˈiːdɛʊ"), ("radio", "rˈɑːdɪʊ"), ("bio", "bˈiːʊ"), ("idé", "ɪdˈeː"),
    ("ide", "ˈiːdə"), ("museum", "mʉsˈeːɵm"), ("piano", "pɪˈɑːnʊ"), ("union", "ʉnɪˈuːn"), ("region", "rɛɡɪˈuːn"), ("regioner", "rɛɡɪˈuːnər"),
    ("religion", "rɛlɪjˈuːn"), ("miljon", "mɪljˈuːn"), ("miljoner", "mɪljˈuːnər"), ("miljard", "mɪljˈɑːrd"),
    ("miljarder", "mɪljˈɑːrdər"), ("familj", "famˈɪlj"), ("familjen", "famˈɪljən"), ("linje", "lˈɪnjə"),
    ("social", "sɔsɪˈɑːl"), ("sociala", "sɔsɪˈɑːla"), ("speciell", "spɛsɪˈɛl"), ("speciellt", "spɛsɪˈɛlt"),
    ("potential", "pʊtɛnsɪˈɑːl"), ("officiell", "ɔfɪsɪˈɛl"), ("aktie", "ˈaktsɪə"), ("aktier", "ˈaktsɪər"),
    ("nation", "nasxˈuːn"), ("station", "stasxˈuːn"), ("situation", "sɪtʉasxˈuːn"), ("version", "vɛʂˈuːn"),
    ("pension", "pansxˈuːn"), ("diskussion", "dɪskɵsxˈuːn"), ("mission", "mɪsxˈuːn"), ("vision", "vɪsxˈuːn"),
    ("chef", "ʃˈeːf"), ("chefen", "ʃˈeːfən"), ("chans", "ʃˈans"), ("choklad", "ʃʊklˈɑːd"), ("check", "tʃˈɛk"),
    ("chatt", "tʃˈat"), ("chips", "tʃˈɪps"), ("match", "mˈatʃ"), ("matchen", "mˈatʃən"), ("lunch", "lˈɵnʃ"),
    ("jobb", "jˈɔb"), ("jobba", "jˈɔba"), ("jobbar", "jˈɔbar"), ("jobbet", "jˈɔbət"), ("hopp", "hˈɔp"),
    ("hoppas", "hˈɔpas"), ("kopp", "kˈɔp"), ("lott", "lˈɔt"), ("bonde", "bˈʊndə"), ("bönder", "bˈœndər"),
    ("juice", "jˈuːs"), ("jeans", "jˈiːns"), ("jazz", "jˈas"), ("jury", "jˈʉry"), ("sj", "ˈɛsjiː"),
    ("tjugo", "ɕˈʉɡʊ"), ("nio", "nˈiːʊ"), ("tio", "tˈiːʊ"), ("fyra", "fˈyːra"), ("sju", "sxˈʉ"), ("sex", "sˈɛks"),
    ("åtta", "ˈɔta"), ("elva", "ˈɛlva"), ("tolv", "tˈɔlv"), ("nitton", "nˈɪtɔn"), ("sjutton", "sxˈɵtɔn"),
    ("fjorton", "fjˈuːrtɔn"), ("tretton", "trˈɛtɔn"), ("femton", "fˈɛmtɔn"), ("sexton", "sˈɛkstɔn"),
    ("trettio", "trˈɛtɪʊ"), ("fyrtio", "fˈœrtɪʊ"), ("femtio", "fˈɛmtɪʊ"), ("sextio", "sˈɛkstɪʊ"),
    ("sjuttio", "sxˈɵtɪʊ"), ("åttio", "ˈɔtɪʊ"), ("nittio", "nˈɪtɪʊ"), ("hundra", "hˈɵndra"), ("tusen", "tˈʉsən"),
    ("två", "tvˈoː"), ("tre", "trˈeː"), ("fem", "fˈɛm"), ("noll", "nˈɔl"), ("procent", "prʊsˈɛnt"),
    ("dollar", "dˈɔlar"), ("euro", "ˈɛʉrʊ"), ("pund", "pˈɵnd"), ("grader", "ɡrˈɑːdər"), ("snabela", "snˈɑːbəlˌɑː"),
    ("plus", "plˈɵs"), ("minus", "mˈiːnɵs"), ("lika", "lˈiːka"), ("kronor", "krˈuːnʊr"), ("krona", "krˈuːna"),
    ("öre", "ˈøːrə"), ("ja", "jˈɑː"), ("nej", "nˈɛj"), ("hej", "hˈɛj"), ("tack", "tˈak"), ("bara", "bˈɑːra"),
    ("måste", "mˈɔstə"), ("vill", "vˈɪl"), ("ville", "vˈɪlə"), ("skulle", "skˈɵlə"), ("kunde", "kˈɵndə"),
    ("kunna", "kˈɵna"), ("kunnat", "kˈɵnat"), ("varit", "vˈɑːrɪt"), ("vara", "vˈɑːra"), ("blev", "blˈeːv"),
    ("blivit", "blˈiːvɪt"), ("hela", "hˈeːla"), ("helt", "hˈeːlt"), ("hel", "hˈeːl"), ("själv", "sxˈɛlv"),
    ("själva", "sxˈɛlva"), ("samma", "sˈama"), ("mellan", "mˈɛlan"), ("under", "ˈɵndər"), ("utan", "ˈʉtan"),
    ("efter", "ˈɛftər"), ("över", "ˈøːvər"), ("genom", "jˈeːnɔm"), ("sedan", "sˈeːdan"), ("alla", "ˈala"),
    ("allt", "ˈalt"), ("all", "ˈal"), ("annan", "ˈanan"), ("annat", "ˈanat"), ("andra", "ˈandra"),
    ("annars", "ˈanaʂ"), ("varje", "vˈarjə"), ("vilken", "vˈɪlkən"), ("vilket", "vˈɪlkət"), ("vilka", "vˈɪlka"),
    ("sådana", "sˈoːdana"), ("sådant", "sˈoːdant"), ("ingen", "ˈɪŋən"), ("inget", "ˈɪŋət"), ("inga", "ˈɪŋa"),
    ("ingenting", "ˈɪŋəntɪŋ"), ("någonting", "nˈoːɡɔntɪŋ"), ("allting", "ˈaltɪŋ"), ("nästan", "nˈɛstan"),
    ("redan", "rˈeːdan"), ("ännu", "ˈɛnʉ"), ("aldrig", "ˈaldrɪɡ"), ("ibland", "ɪblˈand"),
    ("ofta", "ˈɔfta"), ("kanhända", "kanhˈɛnda"), ("emot", "ɛmˈuːt"), ("emellan", "ɛmˈɛlan"), ("bland", "blˈand"),
    ("omkring", "ɔmkrˈɪŋ"), ("ungefär", "ɵnjɛfˈɛːr"), ("ihop", "ɪhˈuːp"), ("iväg", "ɪvˈɛːɡ"), ("ihåg", "ɪhˈoːɡ"),
    ("istället", "ɪstˈɛlət"), ("itu", "ɪtˈʉ"), ("isär", "ɪsˈɛːr"), ("ikväll", "ɪkvˈɛl"), ("emedan", "ɛmˈeːdan"),
    ("hennes", "hˈɛnəs"), ("henne", "hˈɛnə"), ("honom", "hˈɔnɔm"), ("deras", "dˈeːras"), ("oss", "ˈɔs"),
    ("sju", "sxˈʉ"), ("sjuk", "sxˈʉk"), ("sjukhus", "sxˈʉkhʉs"), ("kyrka", "ɕˈyrka"), ("kyrkan", "ɕˈyrkan"),
    ("djur", "jˈʉr"), ("djuret", "jˈʉrət"), ("ljus", "jˈʉs"), ("ljud", "jˈʉd"), ("hjul", "jˈʉl"),
    ("och", "ˈɔk"), ("brev", "brˈeːv"), ("liv", "lˈiːv"), ("livet", "lˈiːvət"), ("sol", "sˈuːl"), ("mor", "mˈuːr"),
    ("far", "fˈɑːr"), ("bror", "brˈuːr"), ("syster", "sˈystər"), ("vatten", "vˈatːən"), ("kvinna", "kvˈɪna"),
    ("kvinnor", "kvˈɪnʊr"), ("hus", "hˈʉs"), ("ord", "ˈuːrd"), ("orden", "ˈuːrdən"), ("ordet", "ˈuːrdət"),
    ("jord", "jˈuːrd"), ("jorden", "jˈuːrdən"), ("bord", "bˈuːrd"), ("bordet", "bˈuːrdət"), ("barn", "bˈɑːrn"),
    ("barnen", "bˈɑːrnən"), ("barnet", "bˈɑːrnət"), ("karl", "kˈɑːrl"), ("världen", "vˈɛːrdən"), ("värld", "vˈɛːrd"),
    ("nord", "nˈuːrd"), ("norden", "nˈuːrdən"), ("hård", "hˈoːrd"), ("hårda", "hˈoːrda"), ("gård", "ɡˈoːrd"),
    ("fjärde", "fjˈɛːrdə"), ("värde", "vˈɛːrdə"), ("värdet", "vˈɛːrdət"), ("ordning", "ˈoːrdnɪŋ"), ("mord", "mˈuːrd"),
    ("sju", "sxˈʉ"), ("sjö", "sxˈøː"), ("sjön", "sxˈøːn"), ("skjorta", "sxˈʊrta"), ("skjuta", "sxˈʉta"),
    ("sked", "sxˈeːd"), ("skön", "sxˈøːn"), ("skinn", "sxˈɪn"), ("sköta", "sxˈøːta"), ("stjärna", "sxˈɛːrna"),
    ("stjärnor", "sxˈɛːrnʊr"), ("tjej", "ɕˈɛj"), ("tjejer", "ɕˈɛjər"), ("kille", "kˈɪlə"), ("killar", "kˈɪlar"),
    ("kilo", "ɕˈiːlʊ"), ("kemi", "ɕɛmˈiː"), ("kedja", "ɕˈeːdja"), ("kö", "kˈøː"), ("kön", "ɕˈøːn"),
    ("kör", "ɕˈøːr"), ("köra", "ɕˈøːra"), ("köpa", "ɕˈøːpa"), ("köpte", "ɕˈœptə"), ("köp", "ɕˈøːp"),
    ("kött", "ɕˈœt"), ("kök", "ɕˈøːk"), ("kind", "ɕˈɪnd"), ("kina", "ɕˈiːna"), ("kinesisk", "ɕɪnˈeːsɪsk"),
    ("kex", "kˈɛks"), ("kiss", "kˈɪs"), ("keps", "kˈɛps"), ("kenya", "kˈeːnya"), ("gel", "ɡˈeːl"), ("gen", "ɡˈeːn"),
    ("gener", "ɡˈeːnər"), ("genetik", "ɡɛnɛtˈiːk"), ("general", "ɡɛnɛrˈɑːl"), ("generell", "ɡɛnɛrˈɛl"),
    ("generation", "ɡɛnɛrasxˈuːn"), ("genre", "ʃˈaŋər"), ("gelé", "ʃɛlˈeː"), ("geni", "ʃɛnˈiː"),
    ("gest", "ʃˈɛst"), ("gin", "jˈɪn"), ("gigant", "jɪɡˈant"), ("gigantisk", "jɪɡˈantɪsk"), ("gymnasium", "jymnˈɑːsɪɵm"),
    ("gäst", "jˈɛst"), ("gäster", "jˈɛstər"), ("regering", "rɛjˈeːrɪŋ"), ("regeringen", "rɛjˈeːrɪŋən"),
    ("regera", "rɛjˈeːra"), ("begära", "bɛjˈɛːra"), ("begärde", "bɛjˈɛːrdə"), ("begär", "bɛjˈɛːr"),
    ("angelägen", "ˈanjɛlɛːɡən"), ("angående", "ˈanɡoːəndə"), ("fågel", "fˈoːɡəl"), ("fåglar", "fˈoːɡlar"),
    ("mage", "mˈɑːɡə"), ("magen", "mˈɑːɡən"), ("lugn", "lˈɵŋn"), ("lugnt", "lˈɵŋt"), ("lugna", "lˈɵŋna"),
    ("vagn", "vˈaŋn"), ("regn", "rˈɛŋn"), ("ugn", "ˈɵŋn"), ("signal", "sɪŋnˈɑːl"), ("magnet", "maŋnˈeːt"),
    ("vägg", "vˈɛɡ"), ("väggen", "vˈɛɡən"), ("berg", "bˈɛrj"), ("berget", "bˈɛrjət"), ("borg", "bˈɔrj"),
    ("torg", "tˈɔrj"), ("torget", "tˈɔrjət"), ("helg", "hˈɛlj"), ("helgen", "hˈɛljən"), ("älg", "ˈɛlj"),
    ("färg", "fˈɛrj"), ("färger", "fˈɛrjər"), ("arg", "ˈarj"), ("varg", "vˈarj"), ("sorg", "sˈɔrj"),
    ("göteborg", "jœtɛbˈɔrj"), ("stockholm", "stˈɔkhɔlm"), ("malmö", "mˈalmøː"), ("uppsala", "ˈɵpsɑːla"),
    ("sverige", "svˈɛrjə"), ("svensk", "svˈɛnsk"), ("svenska", "svˈɛnska"), ("svenskt", "svˈɛnskt"),
    ("engelska", "ˈɛŋəlska"), ("engelsk", "ˈɛŋəlsk"), ("england", "ˈɛŋland"), ("tyska", "tˈyska"),
    ("tyskland", "tˈyskland"), ("finland", "fˈɪnland"), ("danmark", "dˈanmark"), ("norge", "nˈɔrjə"),
    ("usa", "ˌʉɛsˈɑː"), ("amerika", "amˈeːrɪka"), ("afrika", "ˈɑːfrɪka"), ("asien", "ˈɑːsɪən"),
    ("ryssland", "rˈysland"), ("frankrike", "frˈaŋkrɪkə"), ("spanien", "spˈɑːnɪən"), ("italien", "ɪtˈɑːlɪən"),
    ("london", "lˈɔndɔn"), ("paris", "parˈiːs"), ("berlin", "bɛrlˈiːn"), ("new", "njˈʉ"), ("york", "jˈɔrk"),
    ("facebook", "fˈɛjsbʊk"), ("google", "ɡˈʉɡəl"), ("youtube", "jˈʉtjʉb"), ("twitter", "tvˈɪtər"),
    ("internet", "ˈɪntərnɛt"), ("mail", "mˈɛjl"), ("online", "ˈɔnlajn"), ("team", "tˈiːm"), ("show", "ʃˈɔʊ"),
    ("design", "dɪsˈajn"), ("business", "bˈɪsnɪs"), ("ok", "ˌoːkˈɛj"), ("okej", "ˌoːkˈɛj"), ("hockey", "hˈɔkɪ"),
    ("whisky", "vˈɪskɪ"), ("weekend", "vˈiːkɛnd"), ("coach", "kˈoːtʃ"), ("baby", "bˈɛjbɪ"), ("hej", "hˈɛj"),
    ("hejdå", "hɛjdˈoː"), ("tjena", "ɕˈeːna"), ("sjuksköterska", "sxˈʉksxøːtɛʂka"), ("precis", "prɛsˈiːs"),
    ("exakt", "ɛksˈakt"), ("example", "ɛɡsˈampəl"), ("exempel", "ɛksˈɛmpəl"), ("examen", "ɛksˈɑːmən"),
    ("faktiskt", "fˈaktɪskt"), ("faktisk", "fˈaktɪsk"), ("politik", "pʊlɪtˈiːk"), ("politiker", "pʊlˈiːtɪkər"),
    ("politisk", "pʊlˈiːtɪsk"), ("politiska", "pʊlˈiːtɪska"), ("musik", "mʉsˈiːk"), ("fysik", "fysˈiːk"),
    ("teknik", "tɛknˈiːk"), ("mobil", "mʊbˈiːl"), ("bil", "bˈiːl"), ("bilen", "bˈiːlən"), ("bilar", "bˈiːlar"),
    ("april", "aprˈɪl"), ("juli", "jˈʉlɪ"), ("juni", "jˈʉnɪ"), ("maj", "mˈaj"), ("mars", "mˈaʂ"),
    ("augusti", "aɡˈɵstɪ"), ("januari", "janʉˈɑːrɪ"), ("februari", "fɛbrʉˈɑːrɪ"), ("september", "sɛptˈɛmbər"),
    ("oktober", "ɔktˈuːbər"), ("november", "nʊvˈɛmbər"), ("december", "dɛsˈɛmbər"), ("måndag", "mˈɔndɑːɡ"),
    ("tisdag", "tˈiːsdɑːɡ"), ("torsdag", "tˈuːʂdɑːɡ"), ("fredag", "frˈeːdɑːɡ"), ("lördag", "lˈœːrdɑːɡ"),
    ("söndag", "sˈœndɑːɡ"), ("vecka", "vˈɛka"), ("veckan", "vˈɛkan"), ("veckor", "vˈɛkʊr"), ("månad", "mˈoːnad"),
    ("timme", "tˈɪmə"), ("timmar", "tˈɪmar"), ("minut", "mɪnˈʉt"), ("minuter", "mɪnˈʉtər"), ("sekund", "sɛkˈɵnd"),
    ("natt", "nˈat"), ("natten", "nˈatːən"), ("vinter", "vˈɪntər"), ("sommar", "sˈɔmar"), ("höst", "hˈœst"),
    ("vår", "vˈoːr"), ("kväll", "kvˈɛl"), ("kvällen", "kvˈɛlən"), ("eftermiddag", "ˈɛftərmɪdɑːɡ"),
    ("förmiddag", "fˈœrmɪdɑːɡ"), ("middag", "mˈɪdɑːɡ"), ("frukost", "frˈʉkɔst"), ("mat", "mˈɑːt"),
    ("kaffe", "kˈafə"), ("te", "tˈeː"), ("öl", "ˈøːl"), ("vin", "vˈiːn"), ("mjölk", "mjˈœlk"), ("bröd", "brˈøːd"),
    ("smör", "smˈœːr"), ("socker", "sˈɔkər"), ("salt", "sˈalt"), ("fisk", "fˈɪsk"), ("kyckling", "ɕˈyklɪŋ"),
    ("potatis", "pʊtˈɑːtɪs"), ("tomat", "tʊmˈɑːt"), ("banan", "banˈɑːn"), ("apelsin", "apɛlsˈiːn"),
    ("äpple", "ˈɛplə"), ("frukt", "frˈɵkt"), ("grönsaker", "ɡrˈøːnsɑːkər"), ("restaurang", "rɛstʊrˈaŋ"),
    ("hotell", "hʊtˈɛl"), ("sjö", "sxˈøː"), ("skog", "skˈuːɡ"), ("skogen", "skˈuːɡən"), ("träd", "trˈɛːd"),
    ("blommor", "blˈʊmʊr"), ("hund", "hˈɵnd"), ("katt", "kˈat"), ("häst", "hˈɛst"), ("ko", "kˈuː"),
    ("fågel", "fˈoːɡəl"), ("människa", "mˈɛnɪʃa"), ("pojke", "pˈɔjkə"), ("flicka", "flˈɪkːa"), ("mamma", "mˈama"),
    ("pappa", "pˈapa"), ("dotter", "dˈɔtər"), ("fru", "frˈʉ"), ("herr", "hˈɛr"), ("fröken", "frˈøːkən"),
    ("student", "stʉdˈɛnt"), ("lärare", "lˈɛːrarə"), ("läkare", "lˈɛːkarə"), ("polis", "pʊlˈiːs"),
    ("doktor", "dˈɔktɔr"), ("professor", "prʊfˈɛsɔr"), ("direktör", "dɪrɛktˈøːr"), ("ingenjör", "ɪnʃɛnjˈøːr"),
    ("journalist", "ʃʊrnalˈɪst"), ("jour", "ʃˈʉr"), ("journal", "ʃʊrnˈɑːl"), ("projekt", "prʊʃˈɛkt"),
    ("projektet", "prʊʃˈɛktət"), ("objekt", "ɔbjˈɛkt"), ("subjekt", "sɵbjˈɛkt"), ("adjektiv", "ˈadjɛktiːv"),
    ("jag", "jˈɑːɡ"), ("vi", "vˈiː"), ("ni", "nˈiː"), ("du", "dˈʉ"), ("han", "hˈan"), ("hon", "hˈʊn"),
    ("program", "prʊɡrˈam"), ("programmet", "prʊɡrˈamət"), ("system", "systˈeːm"), ("systemet", "systˈeːmət"),
    ("problem", "prʊblˈeːm"), ("problemet", "prʊblˈeːmət"), ("kontakt", "kɔntˈakt"), ("kontor", "kɔntˈuːr"),
    ("kontroll", "kɔntrˈɔl"), ("konsert", "kɔnsˈɛːr"), ("intresse", "ɪntrˈɛsə"), ("intressant", "ɪntrɛsˈant"),
    ("intresserad", "ɪntrɛsˈeːrad"), ("person", "pɛʂˈuːn"), ("personer", "pɛʂˈuːnər"), ("personal", "pɛʂʊnˈɑːl"),
    ("privat", "prɪvˈɑːt"), ("natur", "natˈʉr"), ("kultur", "kɵltˈʉr"), ("figur", "fɪɡˈʉr"),
    ("litteratur", "lɪtɛratˈʉr"), ("struktur", "strɵktˈʉr"), ("universitet", "ʉnɪvɛʂɪtˈeːt"),
    ("kvalitet", "kvalɪtˈeːt"), ("aktivitet", "aktɪvɪtˈeːt"), ("identitet", "ɪdɛntɪtˈeːt"), ("majoritet", "majʊrɪtˈeːt"),
    ("mobil", "mʊbˈiːl"), ("hotel", "hʊtˈɛl"), ("modell", "mʊdˈɛl"), ("motor", "mˈuːtɔr"), ("motiv", "mʊtˈiːv"),
    ("aktiv", "ˈaktiːv"), ("positiv", "pʊsɪtˈiːv"), ("negativ", "nɛɡatˈiːv"), ("relativ", "rɛlatˈiːv"),
    ("alternativ", "altɛrnatˈiːv"), ("effektiv", "ɛfɛktˈiːv"), ("kollektiv", "kɔlɛktˈiːv"), ("arkiv", "arkˈiːv"),
    ("fotboll", "fˈuːtbɔl"), ("boll", "bˈɔl"), ("bollen", "bˈɔlən"), ("sport", "spˈɔrt"), ("tennis", "tˈɛnɪs"),
    ("golf", "ɡˈɔlf"), ("sex", "sˈɛks"), ("text", "tˈɛkst"), ("texten", "tˈɛkstən"), ("exempel", "ɛksˈɛmpəl"),
    ("examen", "ɛksˈɑːmən"), ("extra", "ˈɛkstra"), ("taxi", "tˈaksɪ"), ("max", "mˈaks"), ("fax", "fˈaks"),
    ("sex", "sˈɛks"), ("ex", "ˈɛks"), ("zon", "sˈoːn"), ("zoo", "sˈuː"), ("zink", "sˈɪŋk"), ("pizza", "pˈɪtsa"),
    ("jazz", "jˈas"), ("quiz", "kvˈɪs"), ("wc", "vˌeːsˈeː"), ("vd", "vˌeːdˈeː"), ("tv", "tˌeːvˈeː"),
    ("pc", "pˌeːsˈeː"), ("cd", "sˌeːdˈeː"), ("dvd", "dˌeːvˌeːdˈeː"), ("sms", "ˌɛsˌɛmˈɛs"), ("mm", "mˌɛmˈɛm"),
    ("kg", "kˈiːlʊ"), ("km", "kˌoːˈɛm"), ("cm", "sˌeːˈɛm"), ("kr", "krˈuːnʊr"), ("st", "stˈykən"),
    ("bl", "blˈand"), ("dvs", "dˌeːvˌeːˈɛs"), ("osv", "ˌoːˌɛsvˈeː"), ("tex", "tˌeːˈɛks"), ("ca", "sˈɪrka"),
    ("cirka", "sˈɪrka"), ("etc", "ɛtsˈeːtɛra"), ("nr", "nˈɵmər"), ("kl", "klˈɔkan"), ("tel", "tɛlɛfˈoːn"),
    ("mvh", "mˌɛmvˌeːhˈoː"), ("fr", "frˈoːn"), ("eg", "ˈeːɡ"), ("iq", "ˌiːkˈʉ"),
    ("varför", "vˈarfœr"), ("därför", "dˈɛːrfœr"), ("härför", "hˈɛːrfœr"), ("varandra", "varˈandra"),
    ("arbete", "ˈarbeːtə"), ("arbeta", "ˈarbeːta"), ("arbetar", "ˈarbeːtar"), ("arbetade", "ˈarbeːtadə"),
    ("arbetat", "ˈarbeːtat"), ("arbetet", "ˈarbeːtət"), ("arbetare", "ˈarbeːtarə"), ("arbeten", "ˈarbeːtən"),
    ("kilometer", "ɕɪlʊmˈeːtər"), ("meter", "mˈeːtər"), ("liter", "lˈiːtər"), ("huvud", "hˈʉvɵd"), ("huvudet", "hˈʉvɵdət"),
    ("egentligen", "ɛjˈɛntlɪɡən"), ("egentlig", "ɛjˈɛntlɪɡ"), ("egen", "ˈeːɡən"), ("eget", "ˈeːɡət"), ("egna", "ˈɛŋna"),
    ("igenom", "ɪjˈeːnɔm"), ("ihjäl", "ɪjˈɛːl"), ("ihåg", "ɪhˈoːɡ"), ("gratis", "ɡrˈɑːtiːs"), ("artikel", "artˈiːkəl"),
    ("artiklar", "artˈiːklar"), ("musiker", "mʉsˈiːkər"), ("telefon", "tɛlɛfˈoːn"), ("ledsen", "lˈɛsən"),
    ("from", "frˈʊm"), ("order", "ˈɔrdər"), ("smart", "smˈɑːrt"), ("rakt", "rˈɑːkt"), ("klart", "klˈɑːrt"),
    ("jätte", "jˈɛtə"), ("bibliotek", "bɪblɪʊtˈeːk"), ("princip", "prɪnsˈiːp"), ("reklam", "rɛklˈɑːm"),
    ("period", "pɛrɪˈuːd"), ("perioden", "pɛrɪˈuːdən"), ("rejäl", "rɛjˈɛːl"), ("inse", "ˈɪnseː"), ("japan", "japˈɑːn"),
    ("irak", "ɪrˈɑːk"), ("iran", "ɪrˈɑːn"), ("agenda", "aɡˈɛnda"), ("miljö", "mɪljˈøː"), ("miljön", "mɪljˈøːn"),
    ("israel", "ˈɪsraɛːl"), ("idiot", "ɪdɪˈuːt"), ("hallå", "halˈoː"), ("ändå", "ɛndˈoː"), ("alltså", "ˈaltsoː"),
    ("låtsas", "lˈɔtsas"), ("hänsyn", "hˈɛːnsyːn"), ("cup", "kˈɵp"), ("usch", "ˈɵʃ"), ("maria", "marˈiːa"),
    ("sofia", "sʊfˈiːa"), ("linköping", "lɪnɕˈøːpɪŋ"), ("jönköping", "jœnɕˈøːpɪŋ"), ("norrköping", "nɔrɕˈøːpɪŋ"),
    ("växjö", "vˈɛksxøː"), ("gotland", "ɡˈɔtland"), ("polen", "pˈoːlən"), ("örebro", "ˈœrɛbruː"), ("västerås", "vɛstɛrˈoːs"),
    ("borås", "bʊrˈoːs"), ("island", "ˈiːsland"), ("oscar", "ˈɔskar"), ("oskar", "ˈɔskar"), ("robert", "rˈoːbɛrt"),
    ("johan", "jˈuːan"), ("tvärtom", "tvˈɛːrtɔm"), ("frukt", "frˈɵkt"), ("lukt", "lˈɵkt"), ("bukt", "bˈɵkt"), ("fukt", "fˈɵkt"),
    ("vikt", "vˈɪkt"), ("sikt", "sˈɪkt"), ("dikt", "dˈɪkt"), ("plikt", "plˈɪkt"), ("skikt", "ʃˈɪkt"), ("strikt", "strˈɪkt"),
    ("sent", "sˈeːnt"), ("rent", "rˈeːnt"), ("fint", "fˈiːnt"), ("brunt", "brˈʉnt"), ("runt", "rˈɵnt"), ("tunt", "tˈɵnt"),
    ("ont", "ˈʊnt"), ("vilt", "vˈɪlt"), ("salt", "sˈalt"), ("halt", "hˈalt"), ("valt", "vˈɑːlt"), ("cykel", "sˈykəl"),
    ("cyklar", "sˈyklar"), ("cykla", "sˈykla"), ("titel", "tˈɪtəl"), ("titlar", "tˈɪtlar"), ("kapitel", "kapˈɪtəl"),
    ("kapitlet", "kapˈɪtlət"), ("person", "pɛʂˈuːn"), ("citron", "sɪtrˈuːn"), ("balkong", "balkˈɔŋ"), ("kalkon", "kalkˈuːn"),
    ("betong", "bɛtˈɔŋ"), ("kartong", "kartˈɔŋ"), ("salong", "salˈɔŋ"), ("patron", "patrˈuːn"), ("kanon", "kanˈuːn"),
    ("ozon", "ʊsˈoːn"), ("operation", "ʊpɛrasxˈuːn"), ("omelett", "ɔmɛlˈɛt"), ("oliv", "ʊlˈiːv"), ("oral", "ʊrˈɑːl"),
    ("orange", "ʊrˈanʃ"), ("original", "ʊrɪɡɪnˈɑːl"), ("orient", "ʊrɪˈɛnt"), ("tango", "tˈaŋɡʊ"), ("mango", "mˈaŋɡʊ"),
    ("sommaren", "sˈɔmarən"), ("vintern", "vˈɪntərn"), ("medlem", "mˈeːdlɛm"), ("medlemmar", "mˈeːdlɛmar"),
    ("vecka", "vˈɛka"), ("kväll", "kvˈɛl"), ("stad", "stˈɑːd"), ("staden", "stˈɑːdən"), ("städer", "stˈɛːdər"),
    ("land", "lˈand"), ("landet", "lˈandət"), ("länder", "lˈɛndər"), ("hand", "hˈand"), ("händer", "hˈɛndər"),
    ("tid", "tˈiːd"), ("tiden", "tˈiːdən"), ("tider", "tˈiːdər"), ("del", "dˈeːl"), ("delen", "dˈeːlən"), ("delar", "dˈeːlar"),
    ("väg", "vˈɛːɡ"), ("vägen", "vˈɛːɡən"), ("vägar", "vˈɛːɡar"), ("dag", "dˈɑːɡ"), ("fall", "fˈal"), ("fallet", "fˈalət"),
    ("sätt", "sˈɛt"), ("sättet", "sˈɛtət"), ("plats", "plˈats"), ("platsen", "plˈatsən"), ("platser", "plˈatsər"),
    ("man", "mˈan"), ("mannen", "mˈanən"), ("män", "mˈɛn"), ("männen", "mˈɛnən"), ("bok", "bˈuːk"), ("boken", "bˈuːkən"),
    ("böcker", "bˈœkər"), ("hus", "hˈʉs"), ("huset", "hˈʉsət"), ("skola", "skˈuːla"), ("skolan", "skˈuːlan"),
    ("skolor", "skˈuːlʊr"), ("liv", "lˈiːv"), ("gång", "ɡˈɔŋ"), ("gången", "ɡˈɔŋən"), ("gånger", "ɡˈɔŋər"),
    ("rum", "rˈɵm"), ("rummet", "rˈɵmət"), ("bord", "bˈuːrd"), ("verk", "vˈɛrk"), ("verket", "vˈɛrkət"), ("form", "fˈɔrm"),
    ("formen", "fˈɔrmən"), ("former", "fˈɔrmər"), ("mat", "mˈɑːt"), ("maten", "mˈɑːtən"), ("mark", "mˈark"),
    ("marken", "mˈarkən"), ("kraft", "krˈaft"), ("makt", "mˈakt"), ("makten", "mˈaktən"), ("rätt", "rˈɛt"),
    ("rätten", "rˈɛtən"), ("namn", "nˈamn"), ("namnet", "nˈamnət"), ("folk", "fˈɔlk"), ("folket", "fˈɔlkət"),
    ("spel", "spˈeːl"), ("spelet", "spˈeːlət"), ("spelare", "spˈeːlarə"), ("klass", "klˈas"), ("klassen", "klˈasən"),
    ("nummer", "nˈɵmər"), ("bild", "bˈɪld"), ("bilden", "bˈɪldən"), ("bilder", "bˈɪldər"), ("film", "fˈɪlm"),
    ("filmen", "fˈɪlmən"), ("filmer", "fˈɪlmər"), ("tåg", "tˈoːɡ"), ("tåget", "tˈoːɡət"), ("fält", "fˈɛlt"),
    ("vård", "vˈoːrd"), ("vården", "vˈoːrdən"), ("hjälp", "jˈɛlp"), ("hjälpa", "jˈɛlpa"), ("hjälper", "jˈɛlpər"),
    ("kaka", "kˈɑːka"), ("kakor", "kˈɑːkʊr"), ("stund", "stˈɵnd"), ("mål", "mˈoːl"), ("målet", "mˈoːlət"),
    ("tal", "tˈɑːl"), ("talet", "tˈɑːlət"), ("lag", "lˈɑːɡ"), ("lagen", "lˈɑːɡən"), ("laget", "lˈɑːɡət"),
    ("ledare", "lˈeːdarə"), ("sida", "sˈiːda"), ("sidan", "sˈiːdan"), ("sidor", "sˈiːdʊr"), ("vara", "vˈɑːra"),
    ("varor", "vˈɑːrʊr"), ("ljus", "jˈʉs"), ("ljuset", "jˈʉsət"), ("båt", "bˈoːt"), ("båten", "bˈoːtən"),
    ("fisk", "fˈɪsk"), ("fisken", "fˈɪskən"), ("fiskar", "fˈɪskar"), ("sjö", "sxˈøː"), ("hund", "hˈɵnd"),
    ("hunden", "hˈɵndən"), ("hundar", "hˈɵndar"), ("gård", "ɡˈoːrd"), ("gården", "ɡˈoːrdən"), ("barn", "bˈɑːrn"),
    ("kort", "kˈɔrt"), ("kortet", "kˈɔrtət"), ("boll", "bˈɔl"), ("lek", "lˈeːk"), ("leken", "lˈeːkən"),
    ("skog", "skˈuːɡ"), ("träd", "trˈɛːd"), ("trädet", "trˈɛːdət"), ("mjölk", "mjˈœlk"), ("bröd", "brˈøːd"),
    ("brödet", "brˈøːdət"), ("stor", "stˈuːr"), ("små", "smˈoː"), ("ny", "nˈyː"), ("nya", "nˈyːa"), ("nytt", "nˈyt"),
    ("hel", "hˈeːl"), ("halv", "hˈalv"), ("halva", "hˈalva"), ("halvt", "hˈalft"), ("hem", "hˈɛm"), ("hemmet", "hˈɛmət"),
    ("fri", "frˈiː"), ("fritt", "frˈɪt"), ("fria", "frˈiːa"), ("bo", "bˈuː"), ("en", "ˈeːn"), ("ko", "kˈuː"),
    ("ö", "ˈøː"), ("ön", "ˈøːn"), ("å", "ˈoː"), ("ån", "ˈoːn"), ("by", "bˈyː"), ("byn", "bˈyːn"), ("tro", "trˈuː"),
    ("tror", "trˈuːr"), ("trodde", "trˈʊdə"), ("trott", "trˈʊt"), ("se", "sˈeː"), ("ser", "sˈeːr"), ("såg", "sˈoːɡ"),
    ("sett", "sˈɛt"), ("gå", "ɡˈoː"), ("går", "ɡˈoːr"), ("gått", "ɡˈɔt"), ("stå", "stˈoː"), ("står", "stˈoːr"),
    ("stod", "stˈuːd"), ("stått", "stˈɔt"), ("få", "fˈoː"), ("fått", "fˈɔt"), ("ta", "tˈɑː"), ("tar", "tˈɑːr"),
    ("tog", "tˈuːɡ"), ("tagit", "tˈɑːɡɪt"), ("dö", "dˈøː"), ("dör", "dˈøːr"), ("dog", "dˈuːɡ"), ("dött", "dˈœt"),
    ("le", "lˈeː"), ("ler", "lˈeːr"), ("log", "lˈuːɡ"), ("be", "bˈeː"), ("ber", "bˈeːr"), ("bad", "bˈɑːd"), ("bett", "bˈɛt"),
    ("nå", "nˈoː"), ("når", "nˈoːr"), ("nådde", "nˈɔdə"), ("nått", "nˈɔt"), ("må", "mˈoː"), ("mår", "mˈoːr"),
    ("sy", "sˈyː"), ("syr", "sˈyːr"), ("spå", "spˈoː"), ("så", "sˈoː"), ("sår", "sˈoːr"), ("såra", "sˈoːra"),
    ("bli", "blˈiː"), ("blev", "blˈeːv"), ("ro", "rˈuː"), ("ror", "rˈuːr"), ("oro", "ˈuːruː"), ("klo", "klˈuː"),
    ("bro", "brˈuː"), ("bron", "brˈuːn"), ("tå", "tˈoː"), ("rå", "rˈoː"), ("grå", "ɡrˈoː"), ("blå", "blˈoː"),
    ("blått", "blˈɔt"), ("blåa", "blˈoːa"), ("snö", "snˈøː"), ("snön", "snˈøːn"), ("hö", "hˈøː"), ("mö", "mˈøː"),
    ("frö", "frˈøː"), ("fröet", "frˈøːət"), ("bi", "bˈiː"), ("bin", "bˈiːn"), ("sko", "skˈuː"), ("skor", "skˈuːr"),
    ("skon", "skˈuːn"), ("ske", "sxˈeː"), ("sker", "sxˈeːr"), ("skedde", "sxˈɛdə"), ("skett", "sxˈɛt"),
    ("tre", "trˈeː"), ("fe", "fˈeː"), ("te", "tˈeː"), ("ve", "vˈeː"), ("kli", "klˈiː"), ("ni", "nˈiː"),
    ("tjo", "ɕˈuː"), ("sjå", "sxˈoː"), ("ju", "jˈʉ"), ("nu", "nˈʉ"), ("du", "dˈʉ"), ("sju", "sxˈʉ"),
    ("gnu", "ɡnˈʉ"), ("tabu", "tabˈʉ"), ("hustru", "hˈɵstrʉ"), ("intervju", "ɪntɛrvjˈʉ"), ("meny", "mɛnˈyː"),
    ("parti", "partˈiː"), ("partiet", "partˈiːət"), ("partier", "partˈiːər"), ("kemi", "ɕɛmˈiː"), ("energi", "ɛnɛrʃˈiː"),
    ("ekonomi", "ɛkʊnʊmˈiː"), ("ekonomin", "ɛkʊnʊmˈiːn"), ("ekonomisk", "ɛkʊnˈuːmɪsk"), ("ekonomiska", "ɛkʊnˈuːmɪska"),
    ("industri", "ɪndɵstrˈiː"), ("industrin", "ɪndɵstrˈiːn"), ("demokrati", "dɛmʊkratˈiː"), ("demokratisk", "dɛmʊkrˈɑːtɪsk"),
    ("teknik", "tɛknˈiːk"), ("tekniken", "tɛknˈiːkən"), ("teknisk", "tˈɛknɪsk"), ("tekniska", "tˈɛkniska"),
    ("fysisk", "fˈyːsɪsk"), ("fysiska", "fˈyːsɪska"), ("psykisk", "psˈyːkɪsk"), ("typisk", "tˈyːpɪsk"), ("typiskt", "tˈyːpɪskt"),
    ("praktisk", "prˈaktɪsk"), ("praktiskt", "prˈaktɪskt"), ("historia", "hɪstˈuːrɪa"), ("historisk", "hɪstˈuːrɪsk"),
    ("historien", "hɪstˈuːrɪən"), ("logisk", "lˈuːɡɪsk"), ("logiskt", "lˈuːɡɪskt"), ("komisk", "kˈuːmɪsk"), ("tragisk", "trˈɑːɡɪsk"),
    ("elektrisk", "ɛlˈɛktrɪsk"), ("elektriska", "ɛlˈɛktrɪska"), ("nordisk", "nˈuːrdɪsk"), ("nordiska", "nˈuːrdɪska"),
    ("svenska", "svˈɛnska"), ("danska", "dˈanska"), ("norska", "nˈɔʂka"), ("finska", "fˈɪnska"), ("ryska", "rˈyska"),
    ("franska", "frˈanska"), ("spanska", "spˈanska"), ("isländska", "ˈiːslɛnska"), ("grekiska", "ɡrˈeːkɪska"),
    ("arabiska", "arˈɑːbɪska"), ("latin", "latˈiːn"), ("asiatisk", "asɪˈɑːtɪsk"), ("afrikansk", "afrɪkˈɑːnsk"),
    ("amerikansk", "amɛrɪkˈɑːnsk"), ("amerikanska", "amɛrɪkˈɑːnska"), ("europeisk", "ɛʉrʊpˈeːɪsk"),
    ("europeiska", "ɛʉrʊpˈeːɪska"), ("internationell", "ɪntɛrnasxʊnˈɛl"), ("internationella", "ɪntɛrnasxʊnˈɛla"),
    ("nationell", "nasxʊnˈɛl"), ("nationella", "nasxʊnˈɛla"), ("global", "ɡlʊbˈɑːl"), ("globala", "ɡlʊbˈɑːla"),
    ("lokal", "lʊkˈɑːl"), ("lokala", "lʊkˈɑːla"), ("normal", "nɔrmˈɑːl"), ("normalt", "nɔrmˈɑːlt"), ("total", "tʊtˈɑːl"),
    ("totalt", "tʊtˈɑːlt"), ("central", "sɛntrˈɑːl"), ("centrala", "sɛntrˈɑːla"), ("centrum", "sˈɛntrɵm"),
    ("kapital", "kapɪtˈɑːl"), ("material", "matɛrɪˈɑːl"), ("materialet", "matɛrɪˈɑːlət"), ("special", "spɛsɪˈɑːl"),
    ("kanal", "kanˈɑːl"), ("final", "fɪnˈɑːl"), ("moral", "mʊrˈɑːl"), ("ideal", "ɪdɛˈɑːl"), ("real", "rɛˈɑːl"),
    ("signal", "sɪŋnˈɑːl"), ("skandal", "skandˈɑːl"), ("portal", "pɔrtˈɑːl"), ("metall", "mɛtˈal"), ("hotell", "hʊtˈɛl"),
    ("modell", "mʊdˈɛl"), ("aktuell", "aktʉˈɛl"), ("aktuella", "aktʉˈɛla"), ("aktuellt", "aktʉˈɛlt"),
    ("kulturell", "kɵltʉrˈɛl"), ("formell", "fɔrmˈɛl"), ("reell", "rɛˈɛl"), ("industriell", "ɪndɵstrɪˈɛl"),
    ("professionell", "prʊfɛsxʊnˈɛl"), ("eventuell", "ɛvɛntʉˈɛl"), ("eventuellt", "ɛvɛntʉˈɛlt"),
    ("intellektuell", "ɪntɛlɛktʉˈɛl"), ("individuell", "ɪndɪvɪdʉˈɛl"), ("manuell", "manʉˈɛl"), ("visuell", "vɪsʉˈɛl"),
    ("sexuell", "sɛksʉˈɛl"), ("sexuella", "sɛksʉˈɛla"), ("sexuellt", "sɛksʉˈɛlt"), ("aktiv", "ˈaktiːv"), ("aktiva", "ˈaktiːva"),
    ("aktivt", "ˈaktiːft"), ("positiv", "pʊsɪtˈiːv"), ("positiva", "pʊsɪtˈiːva"), ("positivt", "pʊsɪtˈiːft"),
    ("negativ", "nɛɡatˈiːv"), ("negativa", "nɛɡatˈiːva"), ("negativt", "nɛɡatˈiːft"), ("kreativ", "krɛatˈiːv"),
    ("effektiv", "ɛfɛktˈiːv"), ("effektiva", "ɛfɛktˈiːva"), ("effektivt", "ɛfɛktˈiːft"), ("intensiv", "ɪntɛnsˈiːv"),
    ("massiv", "masˈiːv"), ("naiv", "naˈiːv"), ("passiv", "pasˈiːv"), ("objektiv", "ɔbjɛktˈiːv"), ("subjektiv", "sɵbjɛktˈiːv"),
    ("alternativ", "altɛrnatˈiːv"), ("alternativet", "altɛrnatˈiːvət"), ("alternativa", "altɛrnatˈiːva"),
    ("initiativ", "ɪnɪtsɪatˈiːv"), ("perspektiv", "pɛʂpɛktˈiːv"), ("relativt", "rɛlatˈiːft"), ("definitivt", "dɛfɪnɪtˈiːft"),
    ("exklusiv", "ɛksklʉsˈiːv"), ("defensiv", "dɛfɛnsˈiːv"), ("offensiv", "ɔfɛnsˈiːv"), ("instinktiv", "ɪnstɪŋktˈiːv"),
    ("ide", "ˈiːdə"), ("idé", "ɪdˈeː"), ("idéer", "ɪdˈeːər"), ("idén", "ɪdˈeːn"), ("kafé", "kafˈeː"), ("café", "kafˈeː"),
    ("armé", "armˈeː"), ("kommitté", "kɔmɪtˈeː"), ("succé", "sɵksˈeː"), ("entré", "antrˈeː"), ("turné", "tɵrnˈeː"),
    ("filé", "fɪlˈeː"), ("allé", "alˈeː"), ("supé", "sʉpˈeː"), ("cv", "sˌeːvˈeː"), ("pm", "pˌeːˈɛm"),
    ("välkommen", "vˈɛːlkɔmən"), ("välkomna", "vˈɛːlkɔmna"), ("gravid", "ɡravˈiːd"), ("semester", "sɛmˈɛstər"),
    ("recept", "rɛsˈɛpt"), ("metod", "mɛtˈuːd"), ("metoder", "mɛtˈuːdər"), ("teater", "tɛˈɑːtər"), ("amanda", "amˈanda"),
    ("kanada", "kˈanada"), ("japanska", "japˈɑːnska"), ("japansk", "japˈɑːnsk"), ("advokat", "advʊkˈɑːt"),
    ("attityd", "atɪtˈyːd"), ("lagom", "lˈɑːɡɔm"), ("ovan", "ˈoːvan"), ("ovanför", "ˈoːvanfœr"), ("utom", "ˈʉtɔm"),
    ("dessutom", "dɛsˈʉtɔm"), ("kommun", "kɔmˈʉn"), ("kommunen", "kɔmˈʉnən"), ("kommuner", "kɔmˈʉnər"),
    ("kommunal", "kɔmʉnˈɑːl"), ("fortsätta", "fˈʊrtsɛta"), ("fortsätter", "fˈʊrtsɛtər"), ("fortsatte", "fˈʊrtsatə"),
    ("fortsatt", "fˈʊrtsat"), ("fortfarande", "fˈʊrtfɑːrandə"), ("fort", "fˈʊrt"), ("okej", "ɔkˈɛj"), ("ort", "ˈʊrt"),
    ("orter", "ˈʊrtər"), ("von", "fˈɔn"), ("allvarlig", "ˈalvɑːrlɪɡ"), ("allvar", "ˈalvɑːr"), ("allvarligt", "ˈalvɑːrlɪkt"),
    ("sven", "svˈɛn"), ("solna", "sˈoːlna"), ("kvart", "kvˈɑːrt"), ("museum", "mɵsˈeːɵm"), ("museet", "mɵsˈeːət"),
    ("österrike", "ˈœstɛrˌriːkə"), ("portugal", "pˈɔrtɵɡal"), ("förening", "fœrˈeːnɪŋ"), ("föreningen", "fœrˈeːnɪŋən"),
    ("föreningar", "fœrˈeːnɪŋar"), ("förälder", "fœrˈɛldər"), ("föräldrar", "fœrˈɛldrar"), ("föräldrarna", "fœrˈɛldrarna"),
    ("förändra", "fœrˈɛndra"), ("förändring", "fœrˈɛndrɪŋ"), ("förändringar", "fœrˈɛndrɪŋar"), ("föremål", "fˈøːrɛmoːl"),
    ("företag", "fˈøːrɛtɑːɡ"), ("företaget", "fˈøːrɛtɑːɡət"), ("företagen", "fˈøːrɛtɑːɡən"), ("företagare", "fˈøːrɛtɑːɡarə"),
    ("föreställning", "fˈøːrɛstɛlnɪŋ"), ("föredrag", "fˈøːrɛdrɑːɡ"), ("fördel", "fˈøːrdeːl"), ("fördelar", "fˈøːrdeːlar"),
    ("förlag", "fˈœrlɑːɡ"), ("förord", "fˈøːruːrd"), ("förnamn", "fˈøːrnamn"), ("förbund", "fœrbˈɵnd"), ("förbundet", "fœrbˈɵndət"),
    ("förr", "fˈœr"), ("förut", "fœrˈʉt"), ("förutom", "fœrˈʉtɔm"), ("förorter", "fˈøːrʊrtər"), ("förort", "fˈøːrʊrt"),
    ("förare", "fˈøːrarə"), ("föraren", "fˈøːrarən"), ("förmån", "fˈœrmoːn"),
    ("förråd", "fœrˈoːd"), ("förbud", "fœrbˈʉd"), ("förlust", "fœrlˈɵst"), ("förslag", "fœʂlˈɑːɡ"), ("förslaget", "fœʂlˈɑːɡət"),
    ("försök", "fœʂˈøːk"), ("försöka", "fœʂˈøːka"), ("försöker", "fœʂˈøːkər"), ("försökte", "fœʂˈøːktə"), ("försökt", "fœʂˈøːkt"),
    ("förhållande", "fœrhˈɔlandə"), ("förhållanden", "fœrhˈɔlandən"), ("egentligen", "ɪjˈɛntlɪɡən"), ("egentlig", "ɪjˈɛntlɪɡ"),
    ("sj", "sxˈɛː"), ("acceptera", "aksɛptˈeːra"), ("accepterar", "aksɛptˈeːrar"), ("succé", "sɵksˈeː"), ("tanke", "tˈaŋkə"),
    ("tankar", "tˈaŋkar"), ("tänka", "tˈɛŋka"), ("tänker", "tˈɛŋkər"), ("tänkte", "tˈɛŋktə"), ("tänkt", "tˈɛŋkt"),
    ("dom", "dˈʊm"), ("fattig", "fˈatɪɡ"), ("mattias", "matˈiːas"), ("johanna", "jʊhˈana"), ("johan", "jˈuːan"),
    ("leif", "lˈɛjf"), ("schweiz", "ʃvˈɛjts"), ("kaos", "kˈɑːɔs"), ("oslo", "ˈʊslʊ"), ("irland", "ˈɪrland"),
    ("barcelona", "baʂɛlˈoːna"), ("koppling", "kˈɔplɪŋ"), ("design", "dɛsˈajn"), ("green", "ɡrˈiːn"),
    ("engagemang", "aŋɡaʃɛmˈaŋ"), ("drottning", "drˈɔtnɪŋ"), ("drottningen", "drˈɔtnɪŋən"), ("vinna", "vˈɪna"),
    ("vinner", "vˈɪnər"), ("vann", "vˈan"), ("vunnit", "vˈɵnɪt"), ("vinst", "vˈɪnst"), ("lycklig", "lˈyklɪɡ"),
    ("lyckligt", "lˈyklɪkt"), ("lycka", "lˈyka"), ("lyckas", "lˈykas"), ("lyckades", "lˈykadəs"), ("tidning", "tˈiːdnɪŋ"),
    ("tidningen", "tˈiːdnɪŋən"), ("tidningar", "tˈiːdnɪŋar"), ("meddela", "mˈeːdˌdeːla"), ("meddelande", "mˈeːdˌdeːlandə"),
    ("meddelar", "mˈeːdˌdeːlar"), ("meddelade", "mˈeːdˌdeːladə"), ("eskilstuna", "ˈɛskɪlsˌtʉna"), ("halmstad", "hˈalmstɑːd"),
    ("ett", "ˈɛt"), ("karlskrona", "kɑːrlskrˈuːna"), ("kalifornien", "kalɪfˈɔrnɪən"), ("småningom", "smˈoːnɪŋɔm"),
    ("universitet", "ɵnɪvɛʂɪtˈeːt"), ("universitetet", "ɵnɪvɛʂɪtˈeːtət"), ("januari", "janɵˈɑːrɪ"), ("februari", "fɛbrɵˈɑːrɪ"),
    ("situation", "sɪtɵasxˈuːn"), ("situationen", "sɪtɵasxˈuːnən"), ("student", "stɵdˈɛnt"), ("studenter", "stɵdˈɛntər"),
    ("studera", "stɵdˈeːra"), ("studerar", "stɵdˈeːrar"), ("studerade", "stɵdˈeːradə"), ("studerat", "stɵdˈeːrat"),
    ("musik", "mɵsˈiːk"), ("musiken", "mɵsˈiːkən"), ("musiker", "mɵsˈiːkər"), ("kultur", "kɵltˈʉr"), ("kulturen", "kɵltˈʉrən"),
    ("låtsas", "lˈɔsas"), ("alkohol", "alkʊhˈoːl"), ("åtminstone", "oːtmˈɪnstʊnə"), ("yes", "jˈɛs"), ("allihopa", "alɪhˈuːpa"),
    ("såvida", "soːvˈiːda"), ("moskva", "mˈʊskva"), ("slogs", "slˈuːks"), ("olle", "ˈʊlə"), ("journalistik", "ʃʊrnalɪstˈiːk"),
    ("journalister", "ʃʊrnalˈɪstər"), ("journalisten", "ʃʊrnalˈɪstən"), ("arbets", "ˈarbeːts"), ("euro", "ˈɛʉrʊ"),
    ("med", "mˈeːd"), ("till", "tˈɪl"), ("mot", "mˈuːt"), ("när", "nˈɛːr"), ("här", "hˈɛːr"), ("där", "dˈɛːr"),
    ("vår", "vˈoːr"), ("hos", "hˈuːs"), ("ner", "nˈeːr"), ("ned", "nˈeːd"), ("upp", "ˈɵp"), ("in", "ˈɪn"), ("ut", "ˈʉt"),
    ("av", "ˈɑːv"), ("om", "ˈɔm"), ("över", "ˈøːvər"), ("under", "ˈɵndər"), ("efter", "ˈɛftər"), ("genom", "jˈeːnɔm"),
    ("doktor", "dˈɔktɔr"), ("professor", "prʊfˈɛsɔr"), ("traktor", "trˈaktɔr"), ("sektor", "sˈɛktɔr"), ("faktor", "fˈaktɔr"),
    ("rektor", "rˈɛktɔr"), ("lektor", "lˈɛktɔr"), ("humor", "hˈʉmɔr"), ("motor", "mˈuːtɔr"), ("senior", "sˈeːnɪɔr"),
    ("junior", "jˈʉnɪɔr"), ("major", "majˈuːr"), ("tenor", "tɛnˈuːr"), ("kontor", "kɔntˈuːr"), ("kontoret", "kɔntˈuːrət"),
    ("tredje", "trˈeːdjə"), ("period", "pɛrɪˈuːd"), ("check", "tʃˈɛk"), ("chips", "tʃˈɪps"), ("charlie", "tʃˈɑːlɪ"),
    ("gitarr", "jɪtˈar"), ("kopia", "kʊpˈiːa"), ("kopior", "kʊpˈiːʊr"), ("varsågod", "vɑːʂoːɡˈuːd"), ("talang", "talˈaŋ"),
    ("patient", "pasɪˈɛnt"), ("patienter", "pasɪˈɛntər"), ("set", "sˈɛt"), ("anonym", "anʊnˈyːm"), ("muslim", "mɵslˈiːm"),
    ("civil", "sɪvˈiːl"), ("apotek", "apʊtˈeːk"), ("ordentlig", "ɔrdˈɛntlɪɡ"), ("ordentligt", "ɔrdˈɛntlɪkt"), ("race", "rˈɛjs"),
    ("varann", "varˈan"), ("mona", "mˈoːna"), ("passagerare", "pasaʃˈeːrarə"), ("religiös", "rɛlɪʃˈøːs"), ("religiösa", "rɛlɪʃˈøːsa"),
    ("fult", "fˈʉlt"), ("gult", "ɡˈʉlt"), ("sålt", "sˈoːlt"), ("skylt", "sxˈylt"), ("plan", "plˈɑːn"), ("planen", "plˈɑːnən"),
    ("planer", "plˈɑːnər"), ("port", "pˈɔrt"), ("porten", "pˈɔrtən"), ("dusch", "dˈɵʃ"), ("duscha", "dˈɵʃa"),
    ("aktuell", "aktɵˈɛl"), ("aktuella", "aktɵˈɛla"), ("aktuellt", "aktɵˈɛlt"), ("eventuell", "ɛvɛntɵˈɛl"), ("eventuellt", "ɛvɛntɵˈɛlt"),
    ("intellektuell", "ɪntɛlɛktɵˈɛl"), ("individuell", "ɪndɪvɪdɵˈɛl"), ("manuell", "manɵˈɛl"), ("visuell", "vɪsɵˈɛl"),
    ("sexuell", "sɛksɵˈɛl"), ("sexuella", "sɛksɵˈɛla"), ("sexuellt", "sɛksɵˈɛlt"), ("ritual", "rɪtɵˈɑːl"), ("manual", "manɵˈɑːl"),
    ("aktualitet", "aktɵalɪtˈeːt"), ("januari", "janɵˈɑːrɪ"), ("hemma", "hˈɛma"), ("uttal", "ˈʉtˌtɑːl"), ("uttala", "ˈʉtˌtɑːla"),
];

/// Word-initial k before a front vowel that stays hard (loans).
const K_HARD: &[&str] = &[
    "kille", "killar", "kex", "kiss", "keps", "kebab", "kennel", "kemtvätt", "kilt", "kiwi", "kimono", "kiosk",
    "ketchup", "keso", "kefir", "kilo", "kyl", "kymr", "kevlar", "kerub", "ket", "keff", "kis", "kirgiz", "kiril",
    "kirsch", "kitsch", "kick", "kid", "kit", "kim", "ken", "key", "kevin", "kenn", "kerry", "kiev",
];
/// Word-initial g before a front vowel that stays hard (loans).
const G_HARD: &[&str] = &[
    "gel", "gen", "gener", "genet", "genom", "genu", "geo", "gedi", "gess", "gibb", "giffel", "gigg", "gill",
    "gimm", "gips", "girl", "gitarr", "gyn", "geisha", "gecko", "gem", "gentl", "germ",
];
/// Word-initial g read ʃ (French loans).
const G_SH: &[&str] = &["genre", "gelé", "geni", "generös", "gest", "gentil", "gelatin", "gest"];
/// Native stems where "gen"/"geo"/"genom" prefixes keep j: checked before G_HARD.
const G_SOFT: &[&str] = &["genom", "genast", "gensvar", "gentemot", "genväg", "genast", "genomsnitt", "genomföra"];

/// Medial "sk" + front vowel read as the sj-sound (not after a prefix).
const SK_SOFT: &[&str] = &["maskin", "människ", "kansk", "marskalk", "fallskärm", "avsk", "besk", "försk", "ansk", "utsk", "omsk", "insk", "uppsk", "misssk", "nysk", "mellansk"];

/// Stressed particle prefixes: primary stress on the prefix, secondary on the rest.
const PREFIX_STRESSED: &[&str] = &[
    "anti", "efter", "genom", "under", "åter", "över", "mellan", "själv", "inter", "före", "utom", "inom", "bort",
    "fram", "från", "miss", "ned", "ner", "sam", "till", "upp", "van", "ut", "an", "av", "in", "om", "på", "hem",
    "sär", "små", "stor", "ur", "mot", "med", "o", "sam", "hel", "halv", "jämn", "ny", "inne", "er",
];
/// Prefixes that may be followed by a vowel-initial stem (anordna, utan-, uppe-).
const PREFIX_BEFORE_VOWEL: &[&str] = &["an", "in", "ut", "upp", "om", "av", "över", "under", "efter", "genom", "mellan", "själv", "små", "stor", "ny", "hel", "halv", "sam", "o", "mot", "fram"];
/// Words beginning like a stressed prefix that are not prefixed.
const PREFIX_STOP: &[&str] = &[
    "andra", "annan", "annat", "annars", "anda", "ande", "anden", "ankel", "anka", "ankar", "ansa", "anta", "antal",
    "antog", "antar", "antagen", "anledning", "inte", "innan", "inner", "ingen", "inget", "inga", "ingå", "ingång",
    "insekt", "intern", "intervju", "intresse", "intressant", "omkring", "under", "undan", "undra", "utan", "ute",
    "utom", "avsk", "avsikt", "medel", "medan", "medlem", "metal", "medi", "sammet", "samla", "samma", "samt",
    "samtal", "samhälle", "samvete", "pågå", "påse", "påstå", "upper", "oss", "ost", "ond", "oro", "ord", "orm",
    "okej", "olja", "olika", "orsak", "organ", "orgel", "oval", "omega", "framför", "frame", "från",
    "motor", "motion", "motiv", "mota", "motsats", "hemma", "hemsk", "hemlig", "hela", "helt", "hel", "helg",
    "halva", "halvera", "urin", "urval", "nyhet", "nytt", "nyss", "nya", "ny", "urban", "natur", "efter",
    "stora", "stort", "storm", "före", "föredra", "ned", "ner", "nere", "nedre", "undvik", "underbar",
];
/// Stressed suffixes of loanwords; the upper-case letter is the stressed vowel.
const SUFFIX_STRESSED: &[&str] = &[
    "tiOnerna", "tiOnen", "tiOner", "tiOns", "tiOn", "siOnen", "siOner", "siOn", "iOnen", "iOner", "iOn", "ionEll",
    "ionElla", "ionEllt", "Eringarna", "Eringar", "Eringen", "Erings", "Ering", "Erande", "Erades", "Erade", "Erats",
    "Erat", "Erar", "Eras", "Erad", "Era", "Iken", "Ikens", "Ik", "Isterna", "Isten", "Ister", "Ists", "Ist", "Ismen",
    "Ismer", "Ism", "Ellt", "Ella", "Ellen", "Eller", "Ell", "Enterna", "Enten", "Enter", "Ents", "Ent", "Anterna",
    "Anten", "Anter", "Ants", "Ant", "Ansen", "Anser", "Ans", "Uren", "Urer", "Urerna", "Ur", "Yren", "Yrer", "Yr",
    "Ören", "Örer", "Örerna", "Ör", "Ösa", "Öst", "Ös", "tEten", "tEter", "tEt", "erI", "erIer", "erIet", "erIerna",
    "I", "Ier", "Iet", "In", "Inen", "Iner", "Inerna", "It", "Iten", "Iter", "Iv", "Ivt", "Iva", "Ivet", "Age", "Aget",
    "Al", "Alt", "Ala", "Alen", "Aler", "Alerna", "Är", "Ärt", "Ära", "Ären", "Ess", "Essen", "Esser", "Ett", "Etten",
    "Etter", "É", "Én", "Éer", "Itt", "Att", "Atten", "Atter", "Ea", "Ean", "Inas", "Ut", "Uten", "Uter", "Og", "Ogen", "Oger", "Os", "Osen", "Oser",
    "Om", "Omen", "Omer", "Ör", "Ös",
];
/// Loan suffixes with three or more syllables in the word: -On (telefon) but not -son names.
const SUFFIX_STRESSED_LONG: &[&str] = &["On", "Onen", "Oner", "Onerna", "Ong", "Ongen", "Onger", "Ina", "stAn"];
/// Suffixes stressing the syllable before them.
const SUFFIX_PRE: &[&str] = &["iskt", "iska", "isk", "ium", "ien", "ikern", "ikerna", "iker", "iskan", "iskor", "iskas"];
/// Compound tails with secondary stress and their own vowel length.
const TAILS: &[&str] = &[
    "ås", "sjön", "sjö", "ön", "ö", "ån", "å", "byn", "by", "hult", "borg", "berg", "holm", "torp", "vik", "sund", "näs",
    "ström", "fors", "dalen", "dal", "lund", "löv", "kvist", "qvist", "gren", "strand", "hamn", "köping", "tuna", "stad",
    "heterna", "heten", "heter", "hets", "het", "domen", "dom", "skapen", "skaper", "skapet", "skap", "leken", "lek",
    "bara", "bart", "bar", "lösa", "löst", "lös", "fulla", "fullt", "full", "samma", "samt", "sam", "vis", "mässig",
    "mässiga", "mässigt", "dagen", "dagar", "dag", "tiden", "tider", "tid", "huset", "hus", "landet", "land",
    "staden", "stad", "platsen", "plats", "mannen", "man", "boken", "bok", "sättet", "sätt", "gången", "gång",
    "verket", "verk", "bordet", "bord", "rummet", "rum", "vägen", "väg", "livet", "liv", "delen", "del", "sidan",
    "sida", "skolan", "skola", "maten", "mat", "marken", "mark", "formen", "form", "kraft", "makt", "rätt",
    "sjön", "sjö", "fisk", "båt", "ljus", "mål", "tal", "lag", "ledare", "vatten", "stund", "fall", "spel", "kort",
    "namn", "folk", "barn", "gård", "hem", "lös", "klass", "nummer", "bild", "film", "spelare", "hund", "tåg",
    "boll", "lek", "fält", "skog", "träd", "vård", "hjälp", "kaka", "mjölk", "bröd", "varor", "vara",
];

/// Words ending in a single n with a short vowel.
const SHORT_N: &[&str] = &["man", "kan", "han", "hon", "min", "din", "sin", "den", "en", "in", "an", "mun", "vän", "män", "nån", "san", "fan", "kön"];
/// Words ending in a single m with a long vowel.
const LONG_M: &[&str] = &["dam", "lim", "krim", "team", "kram", "ram", "rim", "program", "system", "problem", "extrem", "poem", "harem", "totem", "chrom", "diadem", "atom"];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stress {
    Primary,
    Secondary,
    None,
}

#[derive(Clone, Debug)]
struct Seg {
    ph: String,
    vowel: bool,
    /// letter index of the vowel (for stress targeting)
    li: usize,
    part: usize,
    /// consonant weight for the vowel-length rule
    w: u8,
    /// the vowel letter (vowels only)
    letter: char,
}

fn is_v(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'å' | 'ä' | 'ö' | 'é' | 'è' | 'ê' | 'à' | 'á' | 'ü' | 'ï' | 'ë' | 'ô' | 'æ' | 'ø')
}
fn is_front(c: char) -> bool {
    matches!(c, 'e' | 'i' | 'y' | 'ä' | 'ö' | 'é' | 'ü')
}

fn cons(ph: &str, w: u8, part: usize) -> Seg {
    Seg { ph: ph.to_string(), vowel: false, li: usize::MAX, part, w, letter: ' ' }
}
fn vow(c: char, li: usize, part: usize) -> Seg {
    Seg { ph: String::new(), vowel: true, li, part, w: 0, letter: c }
}

#[derive(Clone, Copy, Debug)]
struct Part {
    start: usize,
    end: usize,
    stress: Stress,
}

/// Negating o- (olika, omöjlig, otrolig, oavsett): the stem must look like a word.
fn o_prefix_ok(c: &[char]) -> bool {
    let n = c.len();
    if n < 5 {
        return false;
    }
    let (c1, c2, c3) = (c[1], c[2], c[3]);
    if is_v(c1) {
        let rest: String = c[1..].iter().collect();
        return ["av", "an", "in", "er", "ut", "upp", "en", "om", "ef", "ö", "ä"].iter().any(|p| rest.starts_with(p));
    }
    if matches!(c1, 'b' | 'd' | 'f' | 'g' | 'h' | 'j' | 'k' | 'l' | 'p' | 't' | 'v' | 's' | 'm' | 'n' | 'r') && is_v(c2) && c2 != c1 {
        return true;
    }
    if matches!(c1, 'b' | 'd' | 'f' | 'g' | 'k' | 'p' | 't' | 'v') && matches!(c2, 'l' | 'r') && is_v(c3) {
        return true;
    }
    if c1 == 's' && matches!(c2, 'k' | 't' | 'p' | 'l' | 'n' | 'v' | 'm' | 'j') && (is_v(c3) || matches!(c3, 'r' | 'l' | 'j')) {
        return true;
    }
    false
}

/// Split a word into prefix / core / tail parts.
fn parts_of(c: &[char], lw: &str) -> Vec<Part> {
    let n = c.len();
    let mut parts: Vec<Part> = Vec::new();
    let nvow = |a: usize, b: usize| c[a..b].iter().filter(|&&x| is_v(x)).count();
    let mut core_start = 0;
    let mut core_stress = Stress::Primary;
    if !PREFIX_STOP.iter().any(|p| lw.starts_with(p)) {
        // unstressed be-/för-
        for p in ["be", "för"] {
            let pl = p.chars().count();
            let short_ok = p == "be" && n == pl + 2 && matches!(c[n - 1], 'å' | 'o' | 'ö' | 'u');
            if (n >= pl + 3 || short_ok) && c[..pl].iter().collect::<String>() == p && !is_v(c[pl]) && nvow(pl, n) >= 1 {
                let rest: String = c[pl..].iter().collect();
                if (p == "för" && !rest.starts_with('r'))
                    || (p == "be" && !(rest.starts_with("rg") || rest.starts_with("ck") || rest.starts_with("te") && n <= 5 || rest.starts_with("bis")))
                {
                    parts.push(Part { start: 0, end: pl, stress: Stress::None });
                    core_start = pl;
                }
            }
        }
        if core_start == 0 {
            let mut best: Option<usize> = None;
            for p in PREFIX_STRESSED {
                let pl = p.chars().count();
                let short_ok = n == pl + 2 && !is_v(c[pl]) && is_v(c[n - 1]) && p.len() > 1 && c[pl] != c[pl - 1];
                if (n >= pl + 3 || short_ok) && c[..pl].iter().collect::<String>() == *p && nvow(pl, n) >= 1 {
                    if is_v(c[pl]) && !PREFIX_BEFORE_VOWEL.contains(p) {
                        continue;
                    }
                    if *p == "o" && !o_prefix_ok(c) {
                        continue;
                    }
                    if *p == "om" && is_v(c[pl]) {
                        continue;
                    }
                    if best.map_or(true, |b| pl > b) {
                        best = Some(pl);
                    }
                }
            }
            if let Some(pl) = best {
                parts.push(Part { start: 0, end: pl, stress: Stress::Primary });
                core_start = pl;
                core_stress = Stress::Secondary;
            }
        }
    }
    // compound tail
    let mut core_end = n;
    let mut tail: Option<usize> = None;
    for t in TAILS {
        let tl = t.chars().count();
        let core_len = n.saturating_sub(tl + core_start);
        if n > tl + core_start && c[n - tl..].iter().collect::<String>() == *t && nvow(core_start, n - tl) >= 1
            && (core_len >= 3 || (core_len == 2 && is_v(c[n - tl - 1])))
            && tail.map_or(true, |b| tl > b)
        {
            tail = Some(tl);
        }
    }
    if let Some(tl) = tail {
        core_end = n - tl;
    }
    parts.push(Part { start: core_start, end: core_end, stress: core_stress });
    if core_end < n {
        parts.push(Part { start: core_end, end: n, stress: Stress::Secondary });
    }
    parts
}

/// Letters -> segments.
fn segment(c: &[char], parts: &[Part], lw: &str) -> Vec<Seg> {
    let n = c.len();
    let at = |i: usize| -> char { if i < n { c[i] } else { '#' } };
    let part_of = |i: usize| parts.iter().position(|p| i >= p.start && i < p.end).unwrap_or(parts.len() - 1);
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 4);
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let p = part_of(i);
        let initial = parts[p].start == i;
        let nx = at(i + 1);
        let nx2 = at(i + 2);
        let pv = if i > 0 { c[i - 1] } else { '#' };
        let rest = |k: usize| -> String { c[k.min(n)..].iter().collect() };
        if is_v(ch) {
            // diphthongs in loans
            if ch == 'a' && nx == 'u' {
                segs.push(vow('a', i, p));
                segs.push(cons("ʊ", 2, p));
                i += 2;
                continue;
            }
            if ch == 'e' && nx == 'u' && !(i + 2 < n && is_v(nx2)) {
                segs.push(vow('e', i, p));
                segs.push(cons("ʉ", 2, p));
                i += 2;
                continue;
            }
            if ch == 'e' && nx == 'i' && !is_v(nx2) && i + 2 < n {
                segs.push(vow('e', i, p));
                segs.push(cons("j", 2, p));
                i += 2;
                continue;
            }
            segs.push(vow(ch, i, p));
            i += 1;
            if ch == 'o' && nx == ch {
                i += 1;
            }
            continue;
        }
        // -ssion -> sj-sound
        if ch == 's' && rest(i).starts_with("ssion") && i > 0 {
            segs.push(cons("sx", 1, p));
            i += 3;
            continue;
        }
        // doubled consonants: one phone, short vowel before
        if ch == nx && ch != 'j' && part_of(i + 1) == p {
            let between = i > 0 && is_v(pv) && is_v(nx2);
            let ph = match ch {
                't' => if between { "tː" } else { "t" },
                'k' => if between { "kː" } else { "k" },
                'g' => "ɡ",
                'c' => if is_front(nx2) { "ks" } else { "k" },
                'z' => "ts",
                'w' => "v",
                _ => "",
            };
            if ph.is_empty() {
                segs.push(cons(&ch.to_string(), 2, p));
            } else {
                segs.push(cons(ph, 2, p));
            }
            i += 2;
            // ttj, ckj etc.
            continue;
        }
        match ch {
            'b' | 'd' | 'f' | 'h' | 'l' | 'm' | 'n' | 'p' | 'r' | 's' | 't' | 'v' | 'k' | 'g' | 'j' | 'c' | 'x' | 'z' | 'w' | 'q' => {}
            'ç' => {
                segs.push(cons("s", 1, p));
                i += 1;
                continue;
            }
            'ñ' => {
                segs.push(cons("nj", 2, p));
                i += 1;
                continue;
            }
            'ß' => {
                segs.push(cons("s", 2, p));
                i += 1;
                continue;
            }
            _ => {
                i += 1;
                continue;
            }
        }
        match ch {
            'c' => {
                if nx == 'h' {
                    segs.push(cons("ʃ", 1, p));
                    i += 2;
                } else if nx == 'k' {
                    let between = i > 0 && is_v(pv) && is_v(nx2);
                    segs.push(cons(if between { "kː" } else { "k" }, 2, p));
                    i += 2;
                } else if is_front(nx) {
                    segs.push(cons("s", 1, p));
                    i += 1;
                } else {
                    segs.push(cons("k", 1, p));
                    i += 1;
                }
            }
            'd' => {
                if nx == 'j' && initial {
                    i += 1; // j follows
                } else {
                    segs.push(cons("d", 1, p));
                    i += 1;
                }
            }
            'g' => {
                if nx == 'j' && initial {
                    i += 1;
                } else if nx == 'n' && !initial && i > 0 && is_v(pv) {
                    segs.push(cons("ŋ", 2, p));
                    segs.push(cons("n", 1, p));
                    i += 2;
                } else if initial && is_front(nx) {
                    let r = rest(i);
                    if G_SOFT.iter().any(|s| r.starts_with(s)) {
                        segs.push(cons("j", 1, p));
                    } else if G_SH.iter().any(|s| r.starts_with(s)) {
                        segs.push(cons("ʃ", 1, p));
                    } else if G_HARD.iter().any(|s| r.starts_with(s)) {
                        segs.push(cons("ɡ", 1, p));
                    } else {
                        segs.push(cons("j", 1, p));
                    }
                    i += 1;
                } else if matches!(pv, 'r' | 'l') && (nx == '#' || matches!(nx, 's' | 'e' | 'ä' | 'ö' | 'y') || (nx == 'i' && (lw.ends_with("ig") || lw.ends_with("igt") || lw.ends_with("iga") || lw.ends_with("igare") || lw.ends_with("igast")))) {
                    segs.push(cons("j", 1, p));
                    i += 1;
                } else if i + 2 >= n && lw.ends_with("age") && lw.chars().count() >= 5 && !lw.ends_with("lage") && !lw.ends_with("dage") {
                    segs.push(cons("ʃ", 1, p));
                    i += 1;
                } else {
                    segs.push(cons("ɡ", 1, p));
                    i += 1;
                }
            }
            'h' => {
                if nx == 'j' && initial {
                    i += 1;
                } else {
                    segs.push(cons("h", 1, p));
                    i += 1;
                }
            }
            'j' => {
                segs.push(cons("j", 2, p));
                i += 1;
            }
            'k' => {
                if nx == 'j' {
                    segs.push(cons("ɕ", 1, p));
                    i += 2;
                } else if initial && is_front(nx) && !K_HARD.iter().any(|s| rest(i).starts_with(s)) {
                    segs.push(cons("ɕ", 1, p));
                    i += 1;
                } else {
                    segs.push(cons("k", 1, p));
                    i += 1;
                }
            }
            'l' => {
                if nx == 'j' && initial {
                    i += 1;
                } else {
                    segs.push(cons("l", 1, p));
                    i += 1;
                }
            }
            'n' => {
                if nx == 'g' && !(i + 2 < n && at(i + 2) == 'j') && part_of(i + 1) == p {
                    segs.push(cons("ŋ", 2, p));
                    i += 2;
                    // loans: tango, fungera, ungern
                    let tail: String = c[i..].iter().collect();
                    if tail == "o" || tail == "on" || (tail.starts_with("er") && (lw.ends_with("era") || lw.ends_with("erar") || lw.ends_with("erade") || lw.ends_with("erat") || lw.ends_with("ering") || lw.ends_with("eringen") || lw.ends_with("erande"))) || lw.starts_with("ungern") || lw.starts_with("ungersk") {
                        segs.push(cons("ɡ", 1, p));
                    }
                } else if nx == 'k' && part_of(i + 1) == p {
                    segs.push(cons("ŋ", 1, p));
                    i += 1;
                } else {
                    segs.push(cons("n", 1, p));
                    i += 1;
                }
            }
            'r' => {
                if nx == 's' && !(nx2 == 'j') && !(nx2 == 'k' && is_front(at(i + 3)) && part_of(i + 2) != p) {
                    segs.push(cons("ʂ", if part_of(i + 1) == p { 2 } else { 1 }, p));
                    i += 2;
                } else {
                    segs.push(cons("r", 1, p));
                    i += 1;
                }
            }
            's' => {
                let r = rest(i);
                if r.starts_with("sch") {
                    segs.push(cons("ʃ", if initial { 1 } else { 2 }, p));
                    i += 3;
                } else if r.starts_with("skj") || r.starts_with("stj") {
                    segs.push(cons("sx", 1, p));
                    i += 3;
                } else if r.starts_with("sj") {
                    segs.push(cons("sx", 1, p));
                    i += 2;
                } else if r.starts_with("sh") && !is_v(at(i + 2)) {
                    segs.push(cons("s", 1, p));
                    i += 1;
                } else if r.starts_with("sh") {
                    segs.push(cons("ʃ", 1, p));
                    i += 2;
                } else if nx == 'k' && is_front(nx2) && (initial || SK_SOFT.iter().any(|s| lw.contains(s))) {
                    segs.push(cons("sx", 1, p));
                    i += 2;
                } else if r.starts_with("sion") && i > 0 {
                    segs.push(cons("sx", 1, p));
                    i += 2;
                } else {
                    segs.push(cons("s", 1, p));
                    i += 1;
                }
            }
            't' => {
                if nx == 'j' {
                    segs.push(cons("ɕ", 1, p));
                    i += 2;
                } else if nx == 'i' && nx2 == 'o' && at(i + 3) == 'n' && i > 0 {
                    segs.push(cons("sx", 1, p));
                    i += 2;
                } else if nx == 'h' {
                    segs.push(cons("t", 1, p));
                    i += 2;
                } else if nx == 'z' {
                    segs.push(cons("ts", 2, p));
                    i += 2;
                } else {
                    segs.push(cons("t", 1, p));
                    i += 1;
                }
            }
            'x' => {
                segs.push(cons("ks", 2, p));
                i += 1;
            }
            'z' => {
                segs.push(cons("s", 1, p));
                i += 1;
            }
            'w' => {
                segs.push(cons("v", 1, p));
                i += 1;
            }
            'q' => {
                segs.push(cons("k", 1, p));
                i += 1;
                if nx == 'u' && is_v(nx2) {
                    segs.push(cons("v", 1, p));
                    i += 1;
                }
            }
            'b' | 'f' | 'm' | 'p' | 'v' => {
                segs.push(cons(&ch.to_string(), 1, p));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    segs
}

/// b d g are voiceless before t and s (snabbt, lagt, dags).
fn devoice(segs: &mut [Seg]) {
    for k in 0..segs.len().saturating_sub(1) {
        let last = k + 2 == segs.len();
        if matches!(segs[k + 1].ph.as_str(), "t" | "tː") || (segs[k + 1].ph == "s" && last) {
            let v = match segs[k].ph.as_str() {
                "b" => "p",
                "d" => "t",
                "ɡ" => "k",
                _ => continue,
            };
            segs[k].ph = v.to_string();
        }
    }
}

/// Which vowel segment carries the stress of a part (index into segs).
fn stressed_vowel(segs: &[Seg], c: &[char], part: &Part, last: bool) -> Option<usize> {
    let vowels: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| s.vowel && s.part_eq(part)).map(|(i, _)| i).collect();
    if vowels.is_empty() {
        return None;
    }
    let s: String = c[part.start..part.end].iter().collect();
    if last && vowels.len() >= 2 {
        let nv = vowels.len();
        // longest stressed suffix (upper-case letter = stressed vowel)
        let mut best: Option<(usize, usize)> = None; // (suffix len, letter index)
        let consider = |suf: &str, best: &mut Option<(usize, usize)>| {
            let lower = suf.to_lowercase();
            if !s.ends_with(&lower) {
                return;
            }
            let sl = lower.chars().count();
            let total = s.chars().count();
            if sl >= total {
                return;
            }
            let stem: String = s.chars().take(total - sl).collect();
            if !stem.chars().any(is_v) {
                return;
            }
            let Some(pos) = suf.chars().position(|ch| ch.is_uppercase()) else { return };
            let li = part.end - sl + pos;
            if best.map_or(true, |(l, _)| sl > l) {
                *best = Some((sl, li));
            }
        };
        for suf in SUFFIX_STRESSED {
            consider(suf, &mut best);
        }
        if nv >= 3 && !s.ends_with("son") && !s.ends_with("sson") {
            for suf in SUFFIX_STRESSED_LONG {
                consider(suf, &mut best);
            }
        }
        if let Some((_, li)) = best {
            if let Some(k) = vowels.iter().find(|&&k| segs[k].li == li) {
                return Some(*k);
            }
        }
        if nv >= 4 && s.ends_with("ia") {
            if let Some(k) = vowels.iter().rev().find(|&&k| segs[k].li + 2 < part.end) {
                return Some(*k);
            }
        }
        if nv >= 3 {
            for suf in SUFFIX_PRE {
                if s.ends_with(suf) {
                    let sl = suf.chars().count();
                    let stem_end = part.end - sl;
                    // last vowel before stem_end
                    if let Some(k) = vowels.iter().rev().find(|&&k| segs[k].li < stem_end) {
                        return Some(*k);
                    }
                }
            }
        }
    }
    if last && vowels.len() >= 3 && part.start == 0 && c[0] == 'a' && c.len() >= 5 && !is_v(c[1]) && is_v(c[2]) && c[1] != c[2]
        && !matches!(c[1], 'n' | 'v' | 'l' | 'r' | 'j' | 'x')
        && !A_INITIAL_STRESSED.iter().any(|w| s.starts_with(w))
    {
        return Some(vowels[1]);
    }
    Some(vowels[0])
}

/// Native words in a + consonant + vowel with initial stress.
const A_INITIAL_STRESSED: &[&str] = &["apa", "aga", "aka", "asa", "ata", "ama", "aba", "afa", "aha", "ask", "ada", "aza"];

impl Seg {
    fn part_eq(&self, p: &Part) -> bool {
        self.li != usize::MAX && self.li >= p.start && self.li < p.end
    }
}

/// Consonant run after vowel index k inside the same part: (weight, phones).
fn run_after(segs: &[Seg], k: usize) -> (u8, Vec<&str>) {
    let part = segs[k].part;
    let mut w = 0u8;
    let mut phs = Vec::new();
    for s in &segs[k + 1..] {
        if s.vowel || s.part != part {
            break;
        }
        w = w.saturating_add(s.w);
        phs.push(s.ph.as_str());
    }
    (w, phs)
}

fn long_ph(c: char) -> &'static str {
    match c {
        'a' | 'à' | 'á' => "ɑː",
        'e' | 'é' | 'è' | 'ê' | 'ë' => "eː",
        'i' | 'ï' => "iː",
        'o' | 'ô' => "uː",
        'u' => "ʉ",
        'y' | 'ü' => "yː",
        'å' => "oː",
        'ä' | 'æ' => "ɛː",
        'ö' | 'ø' => "øː",
        _ => "",
    }
}
fn short_ph(c: char) -> &'static str {
    match c {
        'a' | 'à' | 'á' => "a",
        'e' | 'é' | 'è' | 'ê' | 'ë' => "ɛ",
        'i' | 'ï' => "ɪ",
        'o' | 'ô' => "ɔ",
        'u' => "ɵ",
        'y' | 'ü' => "y",
        'å' => "ɔ",
        'ä' | 'æ' => "ɛ",
        'ö' | 'ø' => "œ",
        _ => "",
    }
}

/// Assign vowel qualities given the stressed vowel per part.
fn qualities(segs: &mut [Seg], c: &[char], parts: &[Part], stressed: &[Option<usize>]) {
    let n = segs.len();
    for k in 0..n {
        if !segs[k].vowel {
            continue;
        }
        let p = segs[k].part;
        let part = parts[p];
        let is_stressed = stressed[p] == Some(k) && part.stress != Stress::None;
        let letter = segs[k].letter;
        let (w, run) = run_after(segs, k);
        let final_in_part = run.len() as usize + k + 1 >= n || segs[k + 1 + run.len()].part != p;
        let mut weight = w;
        if run.len() >= 2 && run[0] == "r" && matches!(run[1], "d" | "n" | "l") {
            weight = 1;
        }
        if run.len() == 2 && run[0] == "r" && run[1] == "t" && matches!(letter, 'å' | 'ö' | 'ä' | 'u') {
            weight = 1;
        }
        if run.len() == 1 && run[0] == "ʂ" && final_in_part && matches!(letter, 'å' | 'ö' | 'ä') {
            weight = 1;
        }
        let pstr: String = c[part.start..part.end].iter().collect();
        if run.len() == 1 && final_in_part && run[0] == "m" && !LONG_M.contains(&pstr.as_str()) {
            weight = 2;
        }
        if run.len() == 1 && final_in_part && run[0] == "n" && SHORT_N.contains(&pstr.as_str()) {
            weight = 2;
        }
        // inflected forms keep the stem's long vowel: sjukt, djupt, högt, grovt
        if run.len() == 2 && w == 2 && final_in_part && run[1] == "t" && matches!(letter, 'u' | 'o' | 'y' | 'ö' | 'å')
            && (matches!(run[0], "k" | "p" | "v" | "d") || (run[0] == "n" && letter == 'ö'))
        {
            weight = 1;
        }
        // stop + liquid/j + vowel: syncopated stems keep the long vowel (jävla, fåglar, vakna, tredje)
        if run.len() == 2 && (w == 2 || (w == 3 && run[1] == "j")) && !final_in_part && ((matches!(run[0], "b" | "d" | "ɡ" | "k" | "p" | "v" | "t" | "f") && matches!(run[1], "l" | "r" | "n" | "j")) || (run[0] == "s" && run[1] == "n")) {
            weight = 1;
        }
        let ph: String = if letter == 'o' && (pstr == "dom" || pstr == "domen") {
            "ʊ".to_string() // the compound tail -dom (ungdom, sjukdom)
        } else if is_stressed {
            if weight <= 1 { long_ph(letter).to_string() } else { short_ph(letter).to_string() }
        } else {
            // unstressed: short quality; e -> ə after the stress, open o -> ʊ, u -> ɵ (final ʉ)
            let after_stress = parts[..=p].iter().enumerate().any(|(q, pp)| pp.stress == Stress::Primary && stressed[q].map_or(false, |s| s < k));
            let open = weight == 0 || (run.len() == 1 && weight == 1 && !final_in_part);
            match letter {
                'e' if after_stress => "ə".to_string(),
                'e' if p == 0 && part.stress == Stress::None => "ə".to_string(),
                'ö' | 'å' | 'y' if weight == 0 && final_in_part => long_ph(letter).to_string(),
                'o' if open => "ʊ".to_string(),
                'o' if after_stress && run.len() >= 1 && run[0] == "r" && final_in_part && (c[part.end - 1] == 'r' || c[part.end - 1] == 's' || c[part.end - 1] == 'a') => "ʊ".to_string(),
                'u' if weight == 0 && final_in_part => "ʉ".to_string(),
                'é' => "eː".to_string(),
                _ => short_ph(letter).to_string(),
            }
        };
        segs[k].ph = ph;
    }
}

/// Render segments with espeak-style stress marks.
fn render(segs: &[Seg], parts: &[Part], stressed: &[Option<usize>]) -> String {
    let mut out = String::with_capacity(segs.len() * 3);
    // mark position: before the consonant run preceding the stressed vowel within the part
    let mut marks: Vec<(usize, &str)> = Vec::new();
    for (p, part) in parts.iter().enumerate() {
        let Some(k) = stressed[p] else { continue };
        let m = match part.stress {
            Stress::Primary => "ˈ",
            Stress::Secondary => "ˌ",
            Stress::None => continue,
        };
        marks.push((k, m));
    }
    for (i, s) in segs.iter().enumerate() {
        if let Some((_, m)) = marks.iter().find(|(p, _)| *p == i) {
            out.push_str(m);
        }
        out.push_str(&s.ph);
    }
    out
}

fn rules(lw: &str) -> String {
    let c: Vec<char> = lw.chars().collect();
    if c.is_empty() {
        return String::new();
    }
    let parts = parts_of(&c, lw);
    let mut segs = segment(&c, &parts, lw);
    if std::env::var_os("POLYG2P_DEBUG").is_some() {
        eprintln!("{lw}: {parts:?} {:?}", segs.iter().map(|s| (s.ph.clone(), s.letter, s.w)).collect::<Vec<_>>());
    }
    let np = parts.len();
    let stressed: Vec<Option<usize>> = parts.iter().enumerate().map(|(i, p)| stressed_vowel(&segs, &c, p, i + 1 == np)).collect();
    qualities(&mut segs, &c, &parts, &stressed);
    devoice(&mut segs);
    render(&segs, &parts, &stressed)
}

/// Spell letter by letter (initialisms without vowels).
fn spell(c: &[char]) -> String {
    let mut out = String::new();
    for (k, &ch) in c.iter().enumerate() {
        let idx = (ch as u32).wrapping_sub('a' as u32) as usize;
        let name = if idx < 26 { LETTERS.unwrap()[idx] } else {
            match ch {
                'å' => "oː",
                'ä' => "ɛː",
                'ö' => "øː",
                _ => "",
            }
        };
        if name.is_empty() {
            continue;
        }
        let mark = if k + 1 == c.len() { "ˈ" } else { "ˌ" };
        let pos = name.find(|x: char| "aeiouyɑɛɪɔʊɵœøʉ".contains(x)).unwrap_or(0);
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
        let mut out = String::new();
        for (k, p) in parts.iter().enumerate() {
            if k == 0 {
                out.push_str(p);
            } else {
                out.push_str(&p.replace('ˈ', "ˌ"));
            }
        }
        return out;
    }
    let c: Vec<char> = lw.chars().filter(|c| c.is_alphabetic()).collect();
    if c.is_empty() {
        return String::new();
    }
    let lw: String = c.iter().collect();
    if let Some((_, p)) = FUNC.iter().find(|(k, _)| *k == lw) {
        return p.to_string();
    }
    if let Some(p) = lex_get(&lw) {
        return p;
    }
    if !c.iter().any(|&x| is_v(x)) {
        return spell(&c);
    }
    // number compounds built by cardinal(): tjugoett, trettiofem
    for tens in ["tjugo", "trettio", "fyrtio", "femtio", "sextio", "sjuttio", "åttio", "nittio"] {
        if let Some(unit) = lw.strip_prefix(tens) {
            if !unit.is_empty() {
                if let (Some(u), Some(t)) = (lex_get(unit), lex_get(tens)) {
                    return format!("{t}{}", u.replace('ˈ', "ˌ"));
                }
            }
        }
    }
    if let Some(p) = inflected(&lw) {
        return p;
    }
    if let Some(p) = compound(&lw) {
        return p;
    }
    rules(&lw)
}

/// Native inflectional / derivational endings and their phonemes, longest first.
const INFLECT: &[(&str, &str)] = &[
    ("ligheterna", "lɪɡheːtərna"), ("ligheten", "lɪɡheːtən"), ("ligheter", "lɪɡheːtər"), ("lighet", "lɪɡheːt"),
    ("ligaste", "lɪɡastə"), ("ligare", "lɪɡarə"), ("ligast", "lɪɡast"), ("ligen", "lɪɡən"), ("liga", "lɪɡa"),
    ("ligt", "lɪkt"), ("lig", "lɪɡ"), ("ningarna", "nɪŋarna"), ("ningar", "nɪŋar"), ("ningen", "nɪŋən"),
    ("nings", "nɪŋs"), ("ning", "nɪŋ"), ("skapet", "skɑːpət"), ("skaper", "skɑːpər"), ("skap", "skɑːp"),
    ("heterna", "heːtərna"), ("heten", "heːtən"), ("heter", "heːtər"), ("het", "heːt"), ("ande", "andə"),
    ("ende", "əndə"), ("andet", "andət"), ("endet", "əndət"), ("arnas", "arnas"), ("ernas", "ərnas"),
    ("ornas", "ʊrnas"), ("arna", "arna"), ("erna", "ərna"), ("orna", "ʊrna"), ("ades", "adəs"), ("ade", "adə"),
    ("ats", "ats"), ("at", "at"), ("ens", "əns"), ("ets", "əts"), ("ers", "əʂ"), ("ars", "aʂ"), ("ors", "ʊʂ"),
    ("en", "ən"), ("et", "ət"), ("er", "ər"), ("ar", "ar"), ("or", "ʊr"), ("as", "as"), ("es", "əs"),
    ("na", "na"), ("ns", "ns"), ("de", "də"), ("s", "s"), ("n", "n"), ("t", "t"), ("a", "a"), ("e", "ə"),
];

/// Built-in table (later entries override earlier ones), then the data-pack lexicon.
fn lex_get(w: &str) -> Option<String> {
    if let Some(p) = LEX.iter().rev().find(|(k, _)| *k == w).map(|x| x.1) {
        return Some(p.to_string());
    }
    data_lex().and_then(|l| l.get(w)).map(|p| p.to_string())
}

/// A lexicon word plus a native ending (person-lig, hotell-et, bil-arna).
fn inflected(lw: &str) -> Option<String> {
    for (suf, ph) in INFLECT {
        if let Some(stem) = lw.strip_suffix(suf) {
            if stem.chars().count() < 3 {
                continue;
            }
            let last = stem.chars().last().unwrap_or(' ');
            let first = suf.chars().next().unwrap_or(' ');
            if last == first {
                continue; // vin+na, mat+t: the doubled consonant means a different stem
            }
            if !is_v(last) && (suf.chars().count() == 1 || *suf == "de") {
                continue;
            }
            if is_v(last) && is_v(first) && !(last == 'o' || last == 'u') {
                continue;
            }
            if let Some(p) = lex_get(stem) {
                return Some(format!("{p}{ph}"));
            }
        }
    }
    None
}

/// Two-letter words allowed as the second part of a compound.
const SHORT_WORDS: &[&str] = &["gå", "se", "ta", "ge", "bo", "by", "ko", "ro", "så", "nå", "må", "dö", "le", "sy", "tå", "ny", "bi", "te", "ö", "å"];

/// Two lexicon words (optionally joined by -s-): primary stress on the first.
fn compound(lw: &str) -> Option<String> {
    let c: Vec<char> = lw.chars().collect();
    let n = c.len();
    if n < 5 {
        return None;
    }
    for i in (3..=n - 2).rev() {
        let left: String = c[..i].iter().collect();
        let Some(lp) = lex_get(&left) else { continue };
        if !left.chars().any(is_v) || matches!(left.as_str(), "be" | "för" | "er") {
            continue;
        }
        for (link, lph) in [("", ""), ("s", "s")] {
            if i + link.len() + 2 > n {
                continue;
            }
            let right: String = c[i + link.len()..].iter().collect();
            if right.chars().count() == 2 && !SHORT_WORDS.contains(&right.as_str()) {
                continue;
            }
            if !right.starts_with(|ch: char| !is_v(ch)) && link.is_empty() && !is_v(c[i - 1]) {
                // consonant + vowel joint is ambiguous (bil+en): require a lexicon hit with a long right part
                if right.chars().count() < 4 {
                    continue;
                }
            }
            let rp = lex_get(&right).or_else(|| inflected(&right));
            if let Some(rp) = rp {
                return Some(format!("{lp}{lph}{}", rp.replace('ˈ', "ˌ")));
            }
        }
    }
    None
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "noll", "ett", "två", "tre", "fyra", "fem", "sex", "sju", "åtta", "nio", "tio", "elva", "tolv", "tretton",
    "fjorton", "femton", "sexton", "sjutton", "arton", "nitton",
];
const TENS: [&str; 10] = ["", "", "tjugo", "trettio", "fyrtio", "femtio", "sextio", "sjuttio", "åttio", "nittio"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = n / 100;
    let r = n % 100;
    if h > 0 {
        if h > 1 {
            out.push(ONES[h as usize].into());
        }
        out.push("hundra".into());
    }
    if r == 0 {
        return;
    }
    if r < 20 {
        out.push(ONES[r as usize].into());
    } else if r % 10 == 0 {
        out.push(TENS[(r / 10) as usize].into());
    } else {
        out.push(format!("{}{}", TENS[(r / 10) as usize], ONES[(r % 10) as usize]));
    }
}

/// Swedish cardinal number words (neuter "ett").
pub fn cardinal(n: u128) -> Vec<String> {
    let mut out = Vec::new();
    if n == 0 {
        out.push("noll".into());
        return out;
    }
    let scales: [(u128, &str, &str); 4] = [
        (1_000_000_000_000_000_000, "triljon", "triljoner"),
        (1_000_000_000_000, "biljon", "biljoner"),
        (1_000_000_000, "miljard", "miljarder"),
        (1_000_000, "miljon", "miljoner"),
    ];
    let mut rest = n;
    for (val, one, many) in scales {
        let q = rest / val;
        if q == 0 {
            continue;
        }
        rest %= val;
        if q == 1 {
            out.push("en".into());
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
        out.push("tusen".into());
    }
    below_1000(rest, &mut out);
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "procent",
        '$' => "dollar",
        '€' => "euro",
        '£' => "pund",
        '&' => "och",
        '+' => "plus",
        '@' => "snabela",
        '°' => "grader",
        '=' => "lika med",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};
    #[test]
    fn basics() {
        for (w, p) in [
            // vowel length from syllable structure
            ("tala", "tˈɑːla"),
            ("tall", "tˈal"),
            ("vatten", "vˈatːən"),
            ("flicka", "flˈɪkːa"),
            ("gata", "ɡˈɑːta"),
            ("lång", "lˈɔŋ"),
            ("många", "mˈɔŋa"),
            ("mage", "mˈɑːɡə"),
            ("fågel", "fˈoːɡəl"),
            ("vakna", "vˈɑːkna"),
            ("jävla", "jˈɛːvla"),
            ("sjukt", "sxˈʉkt"),
            ("högt", "hˈœkt"),
            ("snabbt", "snˈapt"),
            // sj-sound, tj-sound, softening
            ("sju", "sxˈʉ"),
            ("stjärna", "sxˈɛːrna"),
            ("skina", "sxˈiːna"),
            ("skola", "skˈuːla"),
            ("station", "stasxˈuːn"),
            ("nation", "nasxˈuːn"),
            ("tjej", "ɕˈɛj"),
            ("köpa", "ɕˈøːpa"),
            ("kyrka", "ɕˈyrka"),
            ("göra", "jˈøːra"),
            ("regering", "rɛjˈeːrɪŋ"),
            ("djur", "jˈʉr"),
            ("hjälp", "jˈɛlp"),
            ("berg", "bˈɛrj"),
            ("helg", "hˈɛlj"),
            ("lugn", "lˈɵŋn"),
            // retroflexes / r clusters
            ("bord", "bˈuːrd"),
            ("barn", "bˈɑːrn"),
            ("fors", "fˈɔʂ"),
            ("kort", "kˈɔrt"),
            ("förstå", "fœʂtˈoː"),
            // stress: loan suffixes, unstressed prefixes, particles, compounds
            ("studera", "stɵdˈeːra"),
            ("musik", "mɵsˈiːk"),
            ("hotell", "hʊtˈɛl"),
            ("natur", "natˈʉr"),
            ("telefon", "tɛlɛfˈoːn"),
            ("kemi", "ɕɛmˈiː"),
            ("rabatt", "rabˈat"),
            ("betala", "bətˈɑːla"),
            ("ansvar", "ˈansvˌɑːr"),
            ("inköp", "ˈɪnɕˌøːp"),
            ("olika", "ˈuːlɪka"),
            ("skönhet", "sxˈøːnheːt"),
            ("ungdom", "ˈɵŋdˌʊm"),
            ("medarbetare", "mˈeːdˌarbeːtarə"),
            ("bilarna", "bˈiːlarna"),
            ("personlig", "pɛʂˈuːnlɪɡ"),
            ("flickor", "flˈɪkːʊr"),
            // function words and lexicon
            ("och", "ɔk"),
            ("är", "ɛːr"),
            ("de", "dɔm"),
            ("mig", "mɛj"),
            ("sverige", "svˈɛrjə"),
            ("göteborg", "jœtɛbˈɔrj"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(21).join(" "), "tjugoett");
        assert_eq!(cardinal(135).join(" "), "hundra trettiofem");
        assert_eq!(cardinal(2000).join(" "), "två tusen");
        assert_eq!(cardinal(1_000_000).join(" "), "en miljon");
        assert_eq!(cardinal(3_000_000_000).join(" "), "tre miljarder");
        assert_eq!(word("tjugoett"), "ɕˈʉɡʊˌɛt");
        assert_eq!(word("trettiofem"), "trˈɛtɪʊfˌɛm");
    }

    #[test]
    fn sentences() {
        let g = crate::G2p::new(None);
        assert_eq!(g.phonemize("Jag talar svenska.", crate::Lang::Sv), "jɑːɡ tˈɑːlar svˈɛnska.");
        assert_eq!(g.phonemize("3,5 %", crate::Lang::Sv), "trˈeː kˈɔma fˈɛm prʊsˈɛnt");
        assert_eq!(g.phonemize("SVT", crate::Lang::Sv), "ˌɛsvˌeːtˈeː");
    }
}
