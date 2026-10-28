//! Dutch (Standard, Netherlandic) grapheme-to-phoneme, espeak-style notation.
//!
//! Rules only: morphological layering (unstressed prefixes be-/ge-/ver-/…,
//! stressed separable prefixes, -heid/-lijk/-ig/-tie suffixes), syllable
//! openness for vowel length, schwa placement for unstressed `e`, final and
//! cluster devoicing, and a small built-in exception table for frequent loans
//! and irregular words. An optional lexicon (`nl.lex` in `POLYG2P_DATA`,
//! CC BY-SA pack built from WikiPron) is consulted first when present, and
//! also drives compound splitting the way `de.rs` does.
//!
//! Notation (espeak `nl`): aː eː oː øː / ɑ ɛ ɔ ɵ ɪ / i y u / ɛɪ œy ʌʊ / ə,
//! w = ʋ, g = ɣ (x when voiceless), stress `ˈ` directly before the vowel.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "komma";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "aː", "beː", "seː", "deː", "eː", "ɛf", "ɣeː", "haː", "i", "jeː", "kaː", "ɛl", "ɛm", "ɛn", "oː", "peː", "ky", "ɛr", "ɛs",
    "teː", "y", "veː", "ʋeː", "ɪks", "ɛɪ", "zɛt",
]);

const EREN_TSV: &str = include_str!("../data/nl_eren.tsv");

/// Whole-word pronunciations: function words (reduced forms), frequent loans
/// and words the rules get wrong.
const EXC: &[(&str, &str)] = &[
    ("de", "də"), ("het", "ət"), ("een", "ən"), ("één", "ˈeːn"), ("er", "ər"), ("je", "jə"), ("ze", "zə"), ("we", "ʋə"),
    ("me", "mə"), ("te", "tə"), ("ik", "ɪk"), ("en", "ɛn"), ("'t", "ət"), ("'n", "ən"), ("'s", "s"), ("'k", "ək"),
    ("'m", "əm"), ("z'n", "zən"), ("m'n", "mən"), ("d'r", "dər"), ("zo'n", "zoːn"), ("hun", "hɵn"), ("hen", "hɛn"),
    ("mijn", "mɛɪn"), ("zijn", "zɛɪn"), ("zich", "zɪx"), ("ons", "ɔns"), ("niet", "nit"), ("wat", "ʋɑt"), ("dat", "dɑt"),
    ("dit", "dɪt"), ("die", "di"), ("deze", "dˈeːzə"), ("hij", "hɛɪ"), ("zij", "zɛɪ"), ("wij", "ʋɛɪ"), ("jij", "jɛɪ"),
    ("u", "y"), ("uw", "yw"), ("men", "mɛn"), ("van", "vɑn"), ("in", "ɪn"), ("op", "ɔp"), ("om", "ɔm"), ("aan", "aːn"),
    ("uit", "œyt"), ("bij", "bɛɪ"), ("tot", "tɔt"), ("met", "mɛt"), ("of", "ɔf"), ("als", "ɑls"), ("dan", "dɑn"),
    ("is", "ɪs"), ("was", "ʋɑs"), ("zal", "zɑl"), ("kan", "kɑn"), ("heb", "hɛp"), ("ben", "bɛn"), ("bent", "bɛnt"),
    ("al", "ɑl"), ("nog", "nɔx"), ("ook", "oːk"), ("wel", "ʋɛl"), ("maar", "maːr"), ("want", "ʋɑnt"), ("dus", "dɵs"),
    ("toch", "tɔx"), ("nu", "ny"), ("zo", "zoː"), ("ja", "jaː"), ("nee", "neː"), ("hé", "heː"), ("oké", "oːkˈeː"),
    // irregular / loans
    ("bijzonder", "bizˈɔndər"), ("bijzondere", "bizˈɔndərə"), ("bijzonders", "bizˈɔndərs"), ("meteen", "mɛtˈeːn"),
    ("beneden", "bənˈeːdən"), ("bevel", "bəvˈɛl"), ("benzine", "bɛnzˈinə"), ("generaal", "ɣeːnərˈaːl"), ("versie", "vˈɛrzi"),
    ("versies", "vˈɛrzis"), ("baby", "bˈeːbi"), ("baby's", "bˈeːbis"), ("inderdaad", "ɪndərdˈaːt"), ("afrika", "ˈaːfrikaː"),
    ("amerika", "ɑmˈeːrikaː"), ("europa", "øːrˈoːpaː"), ("ingenieur", "ɪnʒeːnjˈøːr"), ("internet", "ˈɪntərnɛt"),
    ("computer", "kɔmpjˈutər"), ("computers", "kɔmpjˈutərs"), ("team", "tim"), ("teams", "tims"), ("manager", "mˈɛnədʒər"),
    ("weekend", "ʋˈikɛnt"), ("online", "ɔnlˈɑin"), ("website", "ʋˈɛpsɑit"), ("websites", "ʋˈɛpsɑits"), ("show", "ʃoː"),
    ("shows", "ʃoːs"), ("design", "dizˈɑɪn"), ("game", "ɡeːm"), ("games", "ɡeːms"), ("email", "ˈimeːl"), ("e-mail", "ˈimeːl"),
    ("mail", "meːl"), ("mails", "meːls"), ("club", "klɵp"), ("clubs", "klɵps"), ("jazz", "dʒɛs"), ("jeans", "dʒins"),
    ("juice", "dʒus"), ("jungle", "dʒˈɵŋəl"), ("jury", "ʒyːrˈi"), ("journalist", "ʒurnaːlˈɪst"), ("journalisten", "ʒurnaːlˈɪstən"),
    ("journaal", "ʒurnˈaːl"), ("route", "rˈutə"), ("routes", "rˈutəs"), ("retour", "rətˈur"), ("tour", "tur"), ("douche", "duʃ"),
    ("douane", "duʋˈaːnə"), ("souvenir", "suvənˈir"), ("coach", "koːtʃ"), ("lunch", "lɵnʃ"), ("match", "mɛtʃ"), ("check", "tʃɛk"),
    ("chat", "tʃɛt"), ("chips", "ʃɪps"), ("chef", "ʃɛf"), ("chaos", "xˈaːɔs"), ("chemie", "xeːmˈi"), ("chemisch", "xˈeːmis"),
    ("christelijk", "krˈɪstələk"), ("christen", "krˈɪstən"), ("christenen", "krˈɪstənən"), ("cholera", "kˈoːləraː"),
    ("toilet", "tʋaːlˈɛt"), ("detail", "deːtˈɑi"), ("details", "deːtˈɑis"), ("mais", "mɑis"), ("museum", "myzˈeːɵm"),
    ("muziek", "myzˈik"), ("genre", "ʒˈɑnrə"), ("horloge", "hɔrlˈoːʒə"), ("college", "kɔlˈeːʒə"), ("colleges", "kɔlˈeːʒəs"),
    ("regisseur", "reːʒisˈøːr"), ("regie", "reːʒˈi"), ("regime", "reːʒˈim"), ("prestige", "prɛstˈiʒə"), ("logeren", "loːʒˈeːrən"),
    ("energie", "eːnərʒˈi"), ("bagage", "baːɣˈaːʒə"), ("garage", "ɣaːrˈaːʒə"), ("etage", "eːtˈaːʒə"), ("beige", "bɛːʒə"),
    ("rouge", "ruʒ"), ("loge", "lˈoːʒə"), ("rage", "rˈaːʒə"), ("budget", "bɵdʒˈɛt"), ("champagne", "ʃɑmpˈɑnjə"),
    ("cognac", "kɔnjˈɑk"), ("lasagne", "laːzˈɑnjə"), ("signaal", "sɪɲˈaːl"), ("hotel", "hoːtˈɛl"), ("hotels", "hoːtˈɛls"),
    ("model", "moːdˈɛl"), ("modellen", "moːdˈɛlən"), ("kapel", "kaːpˈɛl"), ("expert", "ɛkspˈɛrt"), ("experts", "ɛkspˈɛrts"),
    ("concert", "kɔnsˈɛrt"), ("concerten", "kɔnsˈɛrtən"), ("modern", "moːdˈɛrn"), ("moderne", "moːdˈɛrnə"),
    ("intens", "ɪntˈɛns"), ("mens", "mɛns"), ("mensen", "mˈɛnsən"), ("grens", "ɣrɛns"), ("zeventien", "zˈeːvəntin"),
    ("negentien", "nˈeːɣəntin"), ("zeventig", "zˈeːvəntəx"), ("negentig", "nˈeːɣəntəx"), ("zeventiende", "zˈeːvəntində"),
    ("negentiende", "nˈeːɣəntində"), ("zeventigste", "zˈeːvəntəxstə"), ("negentigste", "nˈeːɣəntəxstə"),
    ("opent", "ˈoːpənt"), ("oefent", "ˈufənt"), ("wapent", "ˈʋaːpənt"), ("ordent", "ˈɔrdənt"), ("tekent", "tˈeːkənt"),
    ("rekent", "rˈeːkənt"), ("betekent", "bətˈeːkənt"), ("zegent", "zˈeːɣənt"), ("regent", "rˈeːɣənt"),
    ("dergelijk", "dˈɛrɣələk"), ("dergelijke", "dˈɛrɣələkə"), ("pensioen", "pɛnʃˈun"), ("station", "staːʃˈɔn"),
    ("stations", "staːʃˈɔns"), ("politie", "poːlˈitsi"), ("kwestie", "kʋˈɛsti"), ("kwesties", "kʋˈɛstis"),
    ("tevreden", "təvrˈeːdən"), ("terecht", "tərˈɛxt"), ("terug", "tərˈɵx"), ("tegelijk", "təɣəlˈɛɪk"),
    ("tegelijkertijd", "təɣəlˈɛɪkərtɛɪt"), ("tegemoet", "təɣəmˈut"), ("tekort", "təkˈɔrt"), ("teveel", "təvˈeːl"),
    ("tevens", "tˈeːvəns"), ("tenslotte", "tɛnslˈɔtə"), ("tenminste", "tɛnmˈɪnstə"), ("terwijl", "tɛrʋˈɛɪl"),
    ("tehuis", "təhˈœys"), ("tezamen", "təzˈaːmən"), ("tenzij", "tɛnzˈɛɪ"), ("tegenwoordig", "teːɣənʋˈoːrdəx"),
    ("tegenover", "teːɣənˈoːvər"), ("tegenwoordige", "teːɣənʋˈoːrdəɣə"), ("telefoon", "teːləfˈoːn"),
    ("telefoons", "teːləfˈoːns"), ("februari", "fˌeːbryˈaːri"), ("januari", "jˌɑnyˈaːri"), ("atlas", "ˈɑtlɑs"),
    ("atleet", "ɑtlˈeːt"), ("atleten", "ɑtlˈeːtən"), ("cadeau", "kaːdˈoː"), ("cadeaus", "kaːdˈoːs"), ("bureau", "byrˈoː"),
    ("niveau", "nivˈoː"), ("niveaus", "nivˈoːs"), ("restaurant", "rɛstoːrˈɑnt"), ("restaurants", "rɛstoːrˈɑnts"),
    ("tunnel", "tˈɵnəl"), ("label", "lˈeːbəl"), ("appel", "ˈɑpəl"), ("appels", "ˈɑpəls"), ("kasteel", "kɑstˈeːl"),
    ("yoghurt", "jˈɔxɵrt"), ("pyjama", "pijˈaːmaː"), ("pizza", "pˈitsaː"), ("taxi", "tˈɑksi"), ("tram", "trɛm"),
    ("flat", "flɛt"), ("tank", "tɛŋk"), ("racket", "rˈɛkət"), ("basket", "bˈɑskət"), ("ticket", "tˈɪkət"),
    ("tickets", "tˈɪkəts"), ("cent", "sɛnt"), ("centen", "sˈɛntən"), ("euro", "ˈøːroː"), ("euro's", "ˈøːroːs"),
    ("dollar", "dˈɔlɑr"), ("dollars", "dˈɔlɑrs"), ("procent", "proːsˈɛnt"), ("miljoen", "mɪljˈun"), ("miljard", "mɪljˈɑrt"),
    ("miljoenen", "mɪljˈunən"), ("miljarden", "mɪljˈɑrdən"), ("biljoen", "bɪljˈun"), ("triljoen", "trɪljˈun"),
    ("elf", "ɛlf"), ("twaalf", "tʋaːlf"), ("dertien", "dɛrtˈin"), ("veertien", "veːrtˈin"), ("dertig", "dˈɛrtəx"),
    ("veertig", "vˈeːrtəx"), ("tachtig", "tˈɑxtəx"), ("honderd", "hˈɔndərt"), ("duizend", "dˈœyzənt"), ("nul", "nɵl"),
    ("komma", "kˈɔmaː"), ("plus", "plɵs"), ("min", "mɪn"), ("apenstaartje", "ˈaːpənstaːrtjə"), ("graden", "ɣrˈaːdən"),
    ("pond", "pɔnt"), ("iemand", "ˈimɑnt"), ("niemand", "nˈimɑnt"), ("jullie", "jˈɵli"), ("misschien", "mɪsxˈin"),
    ("wereld", "ʋˈeːrəlt"), ("werelden", "ʋˈeːrəldən"), ("video", "vˈidijoː"), ("video's", "vˈidijoːs"),
    ("radio", "rˈaːdijoː"), ("piano", "pijˈaːnoː"), ("idee", "idˈeː"), ("ideeën", "idˈeːən"), ("bioscoop", "bijɔskˈoːp"),
    ("opera", "ˈoːpəraː"), ("camera", "kˈaːməraː"), ("camera's", "kˈaːməraːs"), ("kalender", "kaːlˈɛndər"),
    ("menu", "mənˈy"), ("menu's", "mənˈys"), ("mango", "mˈɑŋɡoː"), ("tango", "tˈɑŋɡoː"), ("engels", "ˈɛŋəls"),
    ("engelse", "ˈɛŋəlsə"), ("engeland", "ˈɛŋəlɑnt"), ("ritme", "rˈɪtmə"), ("thuis", "tœys"), ("thans", "tɑns"),
    ("ochtend", "ˈɔxtənt"), ("avond", "ˈaːvɔnt"), ("avonds", "ˈaːvɔnts"), ("avonden", "ˈaːvɔndən"), ("maand", "maːnt"),
    ("maanden", "mˈaːndən"), ("vriend", "vrint"), ("vrienden", "vrˈindən"), ("vriendin", "vrindˈɪn"), ("duidelijk", "dˈœydələk"),
    ("eindelijk", "ˈɛɪndələk"), ("gelijk", "ɣəlˈɛɪk"), ("ongelijk", "ɔnɣəlˈɛɪk"), ("vergelijken", "vərɣəlˈɛɪkən"),
    ("vergelijking", "vərɣəlˈɛɪkɪŋ"), ("vergelijkbaar", "vərɣəlˈɛɪkbaːr"), ("soortgelijke", "sˈoːrtxəlɛɪkə"),
    ("lelijk", "lˈeːlək"), ("huwelijk", "hˈyʋələk"), ("vrolijk", "vrˈoːlək"), ("mogelijk", "mˈoːɣələk"),
    ("vijftien", "vˈɛɪftin"), ("zestien", "zˈɛstin"), ("achttien", "ˈɑxtin"), ("achtste", "ˈɑxstə"), ("eerste", "ˈeːrstə"),
    ("derde", "dˈɛrdə"), ("elfde", "ˈɛlvdə"), ("twaalfde", "tʋˈaːlvdə"), ("honderdste", "hˈɔndərtstə"),
    ("duizendste", "dˈœyzəntstə"), ("ooit", "oːit"), ("nooit", "noːit"), ("wie", "ʋi"), ("hoe", "hu"), ("toen", "tun"),
    ("doen", "dun"), ("moet", "mut"), ("goed", "ɣut"), ("water", "ʋˈaːtər"), ("land", "lɑnt"), ("hand", "hɑnt"),
    ("kind", "kɪnt"), ("mond", "mɔnt"), ("hond", "hɔnt"), ("bed", "bɛt"), ("web", "ʋˈɛp"), ("heeft", "heːft"),
    ("wordt", "ʋɔrt"), ("werd", "ʋɛrt"), ("had", "hɑt"), ("zei", "zɛɪ"), ("ga", "ɣaː"), ("sta", "staː"), ("sla", "slaː"),
    ("na", "naː"), ("zie", "zi"), ("doe", "du"), ("mee", "meː"), ("toe", "tu"), ("daar", "daːr"), ("waar", "ʋaːr"),
    ("hier", "hir"), ("vóór", "voːr"), ("voor", "voːr"), ("door", "doːr"), ("naar", "naːr"), ("over", "ˈoːvər"),
    ("onder", "ˈɔndər"), ("zonder", "zˈɔndər"), ("tegen", "tˈeːɣən"), ("tussen", "tˈɵsən"), ("achter", "ˈɑxtər"),
    ("sinds", "sɪnts"), ("tijdens", "tˈɛɪdəns"), ("wegens", "ʋˈeːɣəns"), ("volgens", "vˈɔlɣəns"), ("ondanks", "ɔndˈɑŋks"),
    ("behalve", "bəhˈɑlvə"), ("nadat", "naːdˈɑt"), ("omdat", "ɔmdˈɑt"), ("voordat", "voːrdˈɑt"), ("zodat", "zoːdˈɑt"),
    ("hoewel", "huʋˈɛl"), ("zoals", "zoːˈɑls"), ("alsof", "ɑlsˈɔf"), ("echter", "ˈɛxtər"), ("immers", "ˈɪmərs"),
    ("ergens", "ˈɛrɣəns"), ("nergens", "nˈɛrɣəns"), ("overal", "oːvərˈɑl"), ("altijd", "ˈɑltɛɪt"), ("vaak", "vaːk"),
    ("sommige", "sˈɔməɣə"), ("enige", "ˈeːnəɣə"), ("enigszins", "ˈeːnəxsɪns"), ("weinig", "ʋˈɛɪnəx"), ("weinige", "ʋˈɛɪnəɣə"),
    ("koningin", "koːnɪŋˈɪn"), ("koninklijk", "kˈoːnɪŋklək"), ("koninklijke", "kˈoːnɪŋkləkə"), ("koninkrijk", "kˈoːnɪŋkrɛɪk"),
    ("cijfer", "sˈɛɪfər"), ("cijfers", "sˈɛɪfərs"), ("asociaal", "ˈaːsoːʃaːl"), ("ratio", "rˈaːtsijoː"), ("patio", "pˈaːtijoː"),
    ("regio", "rˈeːɣijoː"), ("regio's", "rˈeːɣijoːs"), ("vooral", "voːrˈɑl"), ("vooraf", "voːrˈɑf"), ("alleen", "ɑlˈeːn"),
    ("elkaar", "ɛlkˈaːr"), ("mekaar", "məkˈaːr"), ("sowieso", "zoːʋizˈoː"), ("überhaupt", "ybərhˈʌʊpt"),
    ("nederland", "nˈeːdərlɑnt"), ("nederlands", "nˈeːdərlɑnts"), ("nederlandse", "nˈeːdərlɑntsə"), ("amsterdam", "ɑmstərdˈɑm"),
    ("rotterdam", "rɔtərdˈɑm"), ("utrecht", "ˈytrɛxt"), ("belgië", "bˈɛlɣijə"), ("belgisch", "bˈɛlɣis"), ("belgische", "bˈɛlɣisə"),
    ("vlaanderen", "vlˈaːndərən"), ("vlaams", "vlaːms"), ("brussel", "brˈɵsəl"), ("limburg", "lˈɪmbɵrx"),
    ("duitsland", "dˈœytslɑnt"), ("duits", "dœyts"), ("frankrijk", "frˈɑŋkrɛɪk"), ("frans", "frɑns"), ("italië", "itˈaːlijə"),
    ("spanje", "spˈɑnjə"), ("china", "ʃˈinaː"), ("chinees", "ʃinˈeːs"), ("chinese", "ʃinˈeːsə"), ("rusland", "rˈɵslɑnt"),
    ("turkije", "tɵrkˈɛɪə"), ("engel", "ˈɛŋəl"), ("engelen", "ˈɛŋələn"), ("honger", "hˈɔŋər"), ("vinger", "vˈɪŋər"),
    ("vingers", "vˈɪŋərs"), ("jongen", "jˈɔŋən"), ("jongens", "jˈɔŋəns"), ("zingen", "zˈɪŋən"), ("lezen", "lˈeːzən"),
    ("tv", "teːvˈeː"), ("pc", "peːsˈeː"), ("ok", "oːkˈeː"), ("oh", "oː"), ("ah", "aː"), ("eh", "ɛ"), ("hm", "hm"),
    ("thee", "tˈeː"), ("theater", "teːˈaːtər"), ("theorie", "teːoːrˈi"), ("thema", "tˈeːmaː"), ("methode", "meːtˈoːdə"),
    ("katholiek", "kɑtoːlˈik"), ("katholieke", "kɑtoːlˈikə"), ("bibliotheek", "bibliˌoːtˈeːk"), ("apotheek", "aːpoːtˈeːk"),
    ("sympathiek", "sɪmpaːtˈik"), ("atheïst", "aːteːˈɪst"), ("interview", "ˈɪntərvjuː"), ("interviews", "ˈɪntərvjuːs"),
    ("unie", "ˈyni"), ("universiteit", "ˌynivɛrsitˈɛɪt"), ("universiteiten", "ˌynivɛrsitˈɛɪtən"), ("minister", "minˈɪstər"),
    ("ministers", "minˈɪstərs"), ("minuut", "minˈyt"), ("minuten", "minˈytən"), ("bank", "bˈɑŋk"), ("banken", "bˈɑŋkən"),
    ("film", "fɪlm"), ("films", "fɪlms"), ("foto", "fˈoːtoː"), ("foto's", "fˈoːtoːs"), ("auto", "ˈʌʊtoː"), ("auto's", "ˈʌʊtoːs"),
    ("tweede", "tʋˈeːdə"), ("goede", "ɣˈudə"), ("eigen", "ˈɛɪɣən"), ("eigenlijk", "ˈɛɪɣənlək"), ("natuurlijk", "naːtˈyrlək"),
    ("bezig", "bˈeːzəx"), ("lijk", "lɛɪk"), ("lijken", "lˈɛɪkən"), ("lijkt", "lɛɪkt"), ("bevelen", "bəvˈeːlən"),
    ("nadelen", "nˈaːdeːlən"), ("oordelen", "ˈoːrdeːlən"), ("kastelen", "kɑstˈeːlən"), ("kamelen", "kaːmˈeːlən"),
    ("tafereel", "taːfərˈeːl"), ("reclame", "rəklˈaːmə"), ("probleem", "proːblˈeːm"), ("problemen", "proːblˈeːmən"),
    ("systeem", "sistˈeːm"), ("systemen", "sistˈeːmən"), ("item", "ˈɑitəm"), ("items", "ˈɑitəms"), ("sorry", "sˈɔri"),
    ("hobby", "hˈɔbi"), ("hobby's", "hˈɔbis"), ("party", "pˈɑːrti"), ("hockey", "hˈɔki"), ("baby'tje", "bˈeːbitjə"),
    ("single", "sˈɪŋɡəl"), ("gym", "ɣɪm"), ("goal", "ɡoːl"), ("goals", "ɡoːls"), ("playstation", "plˈeːsteːʃən"),
    ("gratis", "ɣrˈaːtɪs"), ("crisis", "krˈizɪs"), ("basis", "bˈaːzɪs"), ("praktijk", "prɑktˈɛɪk"), ("mozaïek", "moːzaːˈik"),
    ("huis", "hˈœys"), ("huizen", "hˈœyzən"), ("thuis", "tœys"), ("ieder", "ˈidər"), ("iedere", "ˈidərə"), ("iedereen", "ˌidərˈeːn"),
    ("elkaars", "ɛlkˈaːrs"), ("iets", "its"), ("niets", "nits"), ("niks", "nɪks"), ("even", "ˈeːvən"), ("eventjes", "ˈeːvəntjəs"),
    ("alles", "ˈɑləs"), ("tenminste", "tɛnmˈɪnstə"), ("intussen", "ɪntˈɵsən"), ("inmiddels", "ɪnmˈɪdəls"),
    ("uiteraard", "œytərˈaːrt"), ("vanzelf", "vɑnzˈɛlf"), ("vanmiddag", "vɑnmˈɪdɑx"), ("vanavond", "vɑnˈaːvɔnt"),
    ("vanmorgen", "vɑnmˈɔrɣən"), ("vandaag", "vɑndˈaːx"), ("vannacht", "vɑnˈɑxt"), ("vanwege", "vɑnʋˈeːɣə"),
    ("vanuit", "vɑnˈœyt"), ("vandaar", "vɑndˈaːr"), ("nabij", "naːbˈɛɪ"), ("voorbij", "voːrbˈɛɪ"), ("dichtbij", "dɪxtbˈɛɪ"),
    ("bijna", "bˈɛɪnaː"), ("bijvoorbeeld", "bɛɪvˈoːrbeːlt"), ("tenslotte", "tɛnslˈɔtə"), ("eerst", "eːrst"),
    ("eens", "eːns"), ("meest", "meːst"), ("meeste", "mˈeːstə"), ("meestal", "mˈeːstɑl"), ("liever", "lˈivər"),
    ("liefst", "lifst"), ("vrouw", "vrʌʊ"), ("vrouwen", "vrˈʌʊən"), ("man", "mˈɑn"), ("mannen", "mˈɑnən"), ("hoor", "hoːr"),
    ("leuk", "løːk"), ("leuke", "lˈøːkə"), ("oei", "ui"), ("au", "ʌʊ"), ("zeg", "zɛx"), ("zegt", "zˈɛxt"), ("lag", "lɑx"),
    ("dag", "dˈɑx"), ("weg", "ʋɛx"), ("nog", "nɔx"), ("og", "ɔx"), ("og", "ɔx"), ("erg", "ɛrx"), ("mag", "mɑx"),
    ("wraak", "vraːk"), ("wreed", "vrˈeːt"), ("wrijven", "vrˈɛɪvən"), ("wringen", "vrˈɪŋən"), ("sjaal", "ʃaːl"),
    ("sjabloon", "ʃaːblˈoːn"), ("tjonge", "tʃˈɔŋə"), ("tsjechië", "tʃˈɛxijə"), ("tsjechisch", "tʃˈɛxis"),
    ("zee", "zeː"), ("zeeën", "zˈeːən"), ("twee", "tʋeː"), ("drie", "dri"), ("vier", "vir"), ("vijf", "vɛɪf"), ("zes", "zɛs"),
    ("zeven", "zˈeːvən"), ("acht", "ɑxt"), ("negen", "nˈeːɣən"), ("tien", "tin"), ("twintig", "tʋˈɪntəx"),
    ("vijftig", "vˈɛɪftəx"), ("zestig", "zˈɛstəx"), ("sex", "sɛks"), ("seks", "sɛks"), ("sexy", "sˈɛksi"), ("x", "ɪks"),
    ("ex", "ɛks"), ("extra", "ˈɛkstraː"), ("examen", "ɛksˈaːmən"), ("examens", "ɛksˈaːməns"), ("exact", "ɛksˈɑkt"),
    ("jou", "jʌʊ"), ("jouw", "jʌʊ"), ("nou", "nʌʊ"), ("zou", "zʌʊ"), ("zouden", "zˈʌʊdən"), ("oud", "ˈʌʊt"), ("oude", "ˈʌʊdə"),
    ("ouder", "ˈʌʊdər"), ("ouders", "ˈʌʊdərs"), ("goud", "ɣʌʊt"), ("koud", "kʌʊt"), ("hout", "hʌʊt"), ("fout", "fʌʊt"),
    ("blauw", "blˈʌʊ"), ("vrouwtje", "vrˈʌʊtjə"), ("bouw", "bʌʊ"), ("bouwen", "bˈʌʊən"), ("trouwen", "trˈʌʊən"),
    ("dezelfde", "dəzˈɛlfdə"), ("hetzelfde", "hətzˈɛlfdə"), ("diezelfde", "dizˈɛlfdə"), ("datzelfde", "dɑtzˈɛlfdə"), ("zelfde", "zˈɛlfdə"),
    ("mezelf", "məzˈɛlf"), ("jezelf", "jəzˈɛlf"), ("onszelf", "ɔnszˈɛlf"), ("uzelf", "yzˈɛlf"), ("hemzelf", "hɛmzˈɛlf"), ("haarzelf", "haːrzˈɛlf"),
    ("henzelf", "hɛnzˈɛlf"), ("mijnheer", "mənˈeːr"), ("meneer", "mənˈeːr"), ("mevrouw", "məvrˈʌʊ"), ("weleens", "ʋɛlˈeːns"), ("weliswaar", "ʋɛlɪsʋˈaːr"),
    ("namelijk", "nˈaːmələk"), ("nameloos", "nˈaːməloːs"), ("plezier", "pləzˈir"), ("plezierig", "pləzˈirəx"), ("chip", "tʃɪp"), ("chips", "tʃɪps"),
    ("checken", "tʃˈɛkən"), ("chill", "tʃɪl"), ("chillen", "tʃˈɪlən"), ("serieus", "seːriˈøːs"), ("serieuze", "seːriˈøːzə"),
    ("oke", "oːkˈeː"), ("collega", "kɔlˈeːɣaː"), ("collega's", "kɔlˈeːɣaːs"), ("precies", "prəsˈis"), ("april", "ɑprˈɪl"), ("categorie", "kɑtəɣoːrˈi"), ("categorieën", "kɑtəɣoːrˈiən"),
    ("record", "rˈɛkɔrt"), ("records", "rˈɛkɔrts"), ("stage", "stˈaːʒə"), ("stages", "stˈaːʒəs"), ("nogal", "nɔɣˈɑl"),
    ("zoiets", "zoːˈits"), ("luxe", "lˈyksə"), ("luxueus", "lyksyˈøːs"), ("concurrent", "kɔŋkyrˈɛnt"), ("concurrentie", "kɔŋkyrˈɛnsi"),
    ("concurrenten", "kɔŋkyrˈɛntən"), ("drugs", "drɵks"), ("milieu", "mɪljˈøː"), ("origineel", "oːriʒinˈeːl"), ("originele", "oːriʒinˈeːlə"),
    ("enthousiast", "ɛntuzijˈɑst"), ("enthousiaste", "ɛntuzijˈɑstə"), ("enthousiasme", "ɛntuzijˈɑsmə"), ("gouverneur", "ɣuvərnˈøːr"),
    ("boulevard", "buləvˈaːr"), ("coupon", "kupˈɔn"), ("bouillon", "bujˈɔn"), ("goulash", "ɣˈulɑʃ"), ("limousine", "limuzˈinə"),
    ("mousse", "mus"), ("routine", "rutˈinə"), ("courant", "kurˈɑnt"), ("courgette", "kurʒˈɛtə"), ("troubadour", "trubaːdˈur"),
    ("souper", "supˈeː"), ("toernooi", "turnˈoːi"), ("coupe", "kup"), ("couvert", "kuvˈɛːr"), ("bourgeois", "burʒʋˈaː"),
    ("amoureus", "ɑmurˈøːs"), ("touché", "tuʃˈeː"), ("roulette", "rulˈɛtə"), ("silhouet", "silʋˈɛt"), ("jaloers", "jaːlˈurs"),
    // English loans
    ("site", "sɑit"), ("sites", "sɑits"), ("update", "ˈɵpdeːt"), ("updates", "ˈɵpdeːts"), ("facebook", "fˈeːsbuk"),
    ("google", "ɣˈuɣəl"), ("twitter", "tʋˈɪtər"), ("youtube", "jˈutjub"), ("cool", "kul"), ("tweet", "tʋit"), ("tweets", "tʋits"),
    ("feedback", "fˈidbɛk"), ("finish", "fˈɪnɪʃ"), ("cash", "kɛʃ"), ("crash", "krɛʃ"), ("sushi", "sˈuʃi"), ("download", "dˈɑunloːt"),
    ("downloads", "dˈɑunloːts"), ("downloaden", "dˈɑunloːdən"), ("app", "ɛp"), ("apps", "ɛps"), ("plastic", "plˈɛstɪk"),
    ("webcam", "ʋˈɛpkɛm"), ("chatten", "tʃˈɛtən"), ("race", "reːs"), ("races", "rˈeːsəs"), ("date", "deːt"), ("dates", "deːts"),
    ("timing", "tˈɑimɪŋ"), ("privacy", "prˈɑivəsi"), ("fulltime", "fˈultɑim"), ("parttime", "pˈɑrttɑim"), ("performance", "pərfˈoːrməns"),
    ("relaxed", "rilˈɛkst"), ("relaxen", "rilˈɛksən"), ("volleybal", "vˈɔlibɑl"), ("community", "kɔmjˈunɪti"), ("living", "lˈɪvɪŋ"),
    ("journalistiek", "ʒurnaːlɪstˈik"), ("journaliste", "ʒurnaːlˈɪstə"), ("jam", "ʃɛm"), ("deal", "dil"), ("deals", "dils"),
    ("marketing", "mˈɑrkətɪŋ"), ("meeting", "mˈitɪŋ"), ("meetings", "mˈitɪŋs"), ("business", "bˈɪsnɪs"), ("service", "sˈɵrvɪs"),
    ("hardware", "hˈɑrtʋɛːr"), ("software", "sˈɔftʋɛːr"), ("image", "ˈɪmɪdʒ"), ("trend", "trɛnt"), ("trends", "trɛnts"),
    ("fan", "fɛn"), ("fans", "fɛns"), ("camping", "kˈɛmpɪŋ"), ("caravan", "kˈɛrəvɛn"), ("printer", "prˈɪntər"), ("laptop", "lˈɛptɔp"),
    ("smartphone", "smˈɑrtfoːn"), ("mobiel", "moːbˈil"), ("mobiele", "moːbˈilə"), ("blog", "blɔx"), ("blogs", "blɔxs"), ("podcast", "pˈɔtkɑst"),
    ("playlist", "plˈeːlɪst"), ("dance", "dɛns"), ("house", "hɑus"), ("party's", "pˈɑːrtis"), ("show", "ʃoː"), ("shoppen", "ʃˈɔpən"),
    ("shop", "ʃɔp"), ("shops", "ʃɔps"), ("fashion", "fˈɛʃən"), ("outfit", "ˈɑutfɪt"), ("sneakers", "snˈikərs"), ("cake", "keːk"),
    ("cocktail", "kˈɔkteːl"), ("cocktails", "kˈɔkteːls"), ("fastfood", "fˈɑstfut"), ("burger", "bˈɵrɣər"), ("sandwich", "sˈɛntʋɪtʃ"),
    ("brunch", "brɵnʃ"), ("trip", "trɪp"), ("creditcard", "krˈɛdɪtkaːrt"), ("job", "dʒɔp"), ("jobs", "dʒɔps"), ("freelance", "frˈilɑns"),
    ("freelancer", "frˈilɑnsər"), ("fitness", "fˈɪtnəs"), ("workout", "ʋˈɵrkɑut"), ("yoga", "jˈoːɣaː"), ("score", "skˈoːrə"), ("scores", "skˈoːrəs"),
    ("finale", "finˈaːlə"), ("box", "bɔks"), ("display", "dɪsplˈeː"), ("gamer", "ɡˈeːmər"), ("gamers", "ɡˈeːmərs"), ("level", "lˈɛvəl"),
    ("levels", "lˈɛvəls"), ("upgrade", "ˈɵpɡreːt"), ("upload", "ˈɵploːt"), ("uploaden", "ˈɵploːdən"), ("file", "fɑil"), ("files", "fɑils"),
    ("folder", "fˈɔldər"), ("desktop", "dˈɛsktɔp"), ("browser", "brˈɑuzər"), ("link", "lɪŋk"), ("links", "lɪŋks"), ("offline", "ɔflˈɑin"),
    ("post", "poːst"), ("posts", "poːsts"), ("like", "lɑik"), ("likes", "lɑiks"), ("followers", "fˈɔloːʋərs"), ("account", "ɑkˈɑunt"),
    ("accounts", "ɑkˈɑunts"), ("password", "pˈɑsʋɵrt"), ("login", "lˈɔɣɪn"), ("user", "jˈuzər"), ("users", "jˈuzərs"), ("cookie", "kˈuki"),
    ("cookies", "kˈukis"), ("spam", "spɛm"), ("hacker", "hˈɛkər"), ("hackers", "hˈɛkərs"), ("hack", "hɛk"), ("virus", "vˈirɵs"),
    ("backup", "bˈɛkɵp"), ("server", "sˈɵrvər"), ("servers", "sˈɵrvərs"), ("cloud", "klɑut"), ("data", "dˈaːtaː"), ("bug", "bɵx"),
    ("bugs", "bɵxs"), ("reset", "risˈɛt"), ("install", "ɪnstˈɔl"), ("mode", "mˈoːdə"), ("live", "lɑif"), ("event", "ivˈɛnt"),
    ("events", "ivˈɛnts"), ("ranking", "rˈɛŋkɪŋ"), ("single", "sˈɪŋɡəl"), ("singles", "sˈɪŋɡəls"), ("beach", "bitʃ"), ("club", "klɵp"),
    ("pub", "pɵp"), ("drinks", "drɪŋks"), ("boys", "bɔis"), ("girls", "ɡɵrls"), ("baby's", "bˈeːbis"), ("teddy", "tˈɛdi"), ("teddybeer", "tˈɛdibeːr"),
    ("hobby's", "hˈɔbis"), ("comedy", "kˈɔmədi"), ("thriller", "trˈɪlər"), ("tv-serie", "teːvˈeː sˈeːri"), ("serie", "sˈeːri"), ("series", "sˈeːris"),
    ("exemplaar", "ɛksəmplˈaːr"), ("exemplaren", "ɛksəmplˈaːrən"), ("liefde", "lˈifdə"), ("liefdes", "lˈifdəs"), ("geliefde", "ɣəlˈifdə"),
    ("geliefden", "ɣəlˈifdən"), ("verenigd", "vərˈeːnəxt"), ("verenigde", "vərˈeːnɪɣdə"), ("mama", "mˈɑmaː"), ("papa", "pˈɑpaː"), ("advocaat", "ɑdvoːkˈaːt"), ("advocaten", "ɑdvoːkˈaːtən"),
    ("antwerpen", "ˈɑntʋɛrpən"), ("antwerps", "ˈɑntʋɛrps"), ("premier", "prəmjˈeː"), ("degene", "dəɣˈeːnə"), ("diegene", "diɣˈeːnə"),
    ("datgene", "dɑtxˈeːnə"), ("hetgeen", "hətxˈeːn"), ("alsjeblieft", "ɑlsjəblˈift"), ("alstublieft", "ɑlstyblˈift"), ("machine", "mɑʃˈinə"),
    ("machines", "mɑʃˈinəs"), ("wasmachine", "ʋˈɑsmɑʃinə"), ("zoekmachine", "zˈukmɑʃinə"), ("haarlem", "hˈaːrlɛm"), ("arnhem", "ˈɑrnɛm"),
    ("recept", "rəsˈɛpt"), ("recepten", "rəsˈɛptən"), ("seconde", "səkˈɔndə"), ("seconden", "səkˈɔndən"), ("medaille", "meːdˈɑljə"),
    ("medailles", "meːdˈɑljəs"), ("diner", "dinˈeː"), ("thailand", "tˈɑilɑnt"), ("hoogleraar", "hˈoːxleːraːr"), ("weigeren", "ʋˈɛɪɣərən"),
    ("weigert", "ʋˈɛɪɣərt"), ("weigerde", "ʋˈɛɪɣərdə"), ("geweigerd", "ɣəʋˈɛɪɣərt"), ("hersenen", "hˈɛrsənən"), ("hersens", "hˈɛrsəns"),
    ("high", "hɑi"), ("down", "dɑun"), ("wow", "ʋɑu"), ("live", "lɑif"), ("my", "mɑi"), ("shirt", "ʃɵrt"), ("shirts", "ʃɵrts"),
    ("nationaal", "naːʃoːnˈaːl"), ("nationale", "naːʃoːnˈaːlə"), ("natie", "nˈaːtsi"), ("naties", "nˈaːtsis"), ("ai", "ɑi"),
    ("waardering", "ʋaːrdˈeːrɪŋ"), ("waarderen", "ʋaːrdˈeːrən"), ("gewaardeerd", "ɣəʋaːrdˈeːrt"), ("desondanks", "dɛsɔndˈɑŋks"),
    ("schiphol", "sxˈɪphɔl"), ("uitgeverij", "ˈœytxeːvərɛɪ"), ("beoordeling", "bəˈoːrdeːlɪŋ"), ("beoordelen", "bəˈoːrdeːlən"),
    ("tabel", "taːbˈɛl"), ("tabellen", "taːbˈɛlən"), ("première", "prəmjˈɛːrə"), ("speech", "spitʃ"), ("douchen", "dˈuʃən"),
    ("bridge", "brɪdʒ"), ("circuit", "sɪrkʋˈi"), ("musical", "mjˈuzikəl"), ("producer", "proːdjˈusər"), ("academie", "aːkaːdˈeːmi"),
    ("strategie", "straːteːɣˈi"), ("horeca", "hoːrˈeːkaː"), ("arena", "aːrˈeːnaː"), ("federatie", "feːdeːrˈaːtsi"),
    ("herinneren", "hɛrˈɪnərən"), ("herinnering", "hɛrˈɪnərɪŋ"), ("herinneringen", "hɛrˈɪnərɪŋən"), ("herinnerd", "hɛrˈɪnərt"),
    ("herinnert", "hɛrˈɪnərt"), ("herinnerde", "hɛrˈɪnərdə"),
    ("intelligent", "ɪntəliɣˈɛnt"), ("intelligente", "ɪntəliɣˈɛntə"), ("intelligentie", "ɪntəliɣˈɛnsi"), ("ademen", "ˈaːdəmən"),
    ("tour", "tur"), ("tours", "turs"), ("flat", "flɛt"), ("flats", "flɛts"), ("tram", "trɛm"), ("trams", "trɛms"),
    ("truck", "trɵk"), ("trucks", "trɵks"), ("jeep", "dʒip"), ("scooter", "skˈutər"), ("scooters", "skˈutərs"), ("camera", "kˈaːməraː"),
    ("zover", "zoːvˈɛr"), ("verdere", "vˈɛrdərə"), ("verderop", "vɛrdərˈɔp"), ("uiterlijk", "ˈœytərlək"), ("uiterlijke", "ˈœytərləkə"),
    ("uiterst", "ˈœytərst"), ("uiterste", "ˈœytərstə"), ("althans", "ɑltˈɑns"), ("drenthe", "drˈɛntə"), ("esther", "ˈɛstər"),
    ("logisch", "lˈoːxis"), ("logische", "lˈoːxisə"), ("cover", "kˈɔvər"), ("special", "spˈɛʃɑl"), ("initiatief", "inisjaːtˈif"),
    ("initiatieven", "inisjaːtˈivən"), ("initiatiefnemer", "inisjaːtˈifneːmər"), ("initieel", "iniʃˈeːl"), ("regering", "rəɣˈeːrɪŋ"),
    ("regeringen", "rəɣˈeːrɪŋən"), ("regeren", "rəɣˈeːrən"), ("religie", "rəlˈiɣi"), ("religies", "rəlˈiɣis"), ("reserve", "rəzˈɛrvə"),
    ("agenda", "aːɣˈɛndaː"), ("assistent", "ɑsistˈɛnt"), ("assistente", "ɑsistˈɛntə"), ("systeem", "sistˈeːm"), ("zichzelf", "zɪxzˈɛlf"),
    ("nieuw", "nˈiw"), ("nieuws", "niws"), ("eeuw", "eːw"), ("eeuwen", "ˈeːwən"), ("leeuw", "lˈeːw"),
    ("sneeuw", "sneːw"), ("schuw", "sxyw"), ("ruw", "ryw"), ("duw", "dyw"), ("zwaluw", "zʋˈaːlyw"), ("kieuw", "kiw"),
];

/// Letters before -er(en) where the e is a schwa (counts from data/nl_eren.tsv).
fn eren_table() -> &'static HashMap<&'static str, (u32, u32)> {
    static T: OnceLock<HashMap<&'static str, (u32, u32)>> = OnceLock::new();
    T.get_or_init(|| {
        EREN_TSV
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| {
                let mut it = l.split('\t');
                let k = it.next()?;
                let s: u32 = it.next()?.parse().ok()?;
                let e: u32 = it.next()?.parse().ok()?;
                Some((k, (s, e)))
            })
            .collect()
    })
}

/// Is the `e` of -eren a schwa after these two letters (vergad-eren vs stud-eren)?
fn eren_schwa(before: &str) -> bool {
    let t = eren_table();
    let key: String = before.chars().rev().take(2).collect::<Vec<_>>().into_iter().rev().collect();
    if let Some(&(s, e)) = t.get(key.as_str()) {
        return s > e;
    }
    // back off to the last letter
    let last = key.chars().last().unwrap_or('#');
    let (mut s, mut e) = (0, 0);
    for (k, &(ks, ke)) in t.iter() {
        if k.ends_with(last) {
            s += ks;
            e += ke;
        }
    }
    if s + e == 0 {
        return true;
    }
    s > e
}

fn exc() -> &'static HashMap<&'static str, &'static str> {
    static M: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    M.get_or_init(|| EXC.iter().copied().collect())
}

// ---------------------------------------------------------------- optional lexicon

fn lexicon() -> Option<&'static Lexicon> {
    static L: OnceLock<Option<Lexicon>> = OnceLock::new();
    L.get_or_init(|| {
        let p = crate::data_dir()?.join("nl.lex");
        Lexicon::open(&p).ok()
    })
    .as_ref()
}

fn lookup(w: &str) -> Option<String> {
    lexicon()?.get(w).map(|p| p.to_string())
}

/// Split an unknown word into lexicon words (German-style, with s/en/e linkers).
fn compound(lw: &str) -> Option<String> {
    let lex = lexicon()?;
    let c: Vec<char> = lw.chars().collect();
    let n = c.len();
    if n < 6 {
        return None;
    }
    const LINKS: &[(&str, &str)] = &[("", ""), ("s", "s"), ("en", "ən"), ("e", "ə")];
    let s = |a: usize, b: usize| -> String { c[a..b].iter().collect() };
    let mut best: Vec<Option<(usize, String)>> = vec![None; n + 1];
    best[n] = Some((0, String::new()));
    for i in (0..n).rev() {
        let mut cand: Option<(usize, String)> = None;
        for j in (i + 3)..=n {
            let Some((parts, rest)) = best[j].clone() else { continue };
            let piece = s(i, j);
            let mut options: Vec<(String, &str)> = Vec::new();
            if j == n {
                if piece.chars().count() >= 4 {
                    options.push((piece.clone(), ""));
                }
            } else {
                for (link, lp) in LINKS {
                    if let Some(stem) = piece.strip_suffix(link) {
                        if stem.chars().count() >= 4 || (i == 0 && stem.chars().count() >= 3) {
                            options.push((stem.to_string(), lp));
                        }
                    }
                }
            }
            for (stem, lp) in options {
                let Some(p) = lex.get(&stem) else { continue };
                if stem.chars().count() < 4 && exc().contains_key(stem.as_str()) {
                    continue;
                }
                let p = p.split('\t').next().unwrap_or("").to_string();
                let pron = format!("{}{lp}{rest}", if i == 0 { p } else { p.replace('ˈ', "ˌ") });
                let score = parts + 1;
                if cand.as_ref().map_or(true, |(k, _)| score < *k) {
                    cand = Some((score, pron));
                }
            }
        }
        best[i] = cand;
    }
    best[0].clone().filter(|(k, _)| *k >= 2).map(|(_, p)| finalize(&p))
}

// ---------------------------------------------------------------- public API

pub fn word(lw: &str) -> String {
    let w: String = lw.chars().filter(|c| c.is_alphabetic() || matches!(c, '\'' | '-' | '’')).collect::<String>().replace('’', "'");
    if !w.chars().any(|c| c.is_alphabetic()) {
        return String::new();
    }
    if let Some(p) = exc().get(w.as_str()) {
        return p.to_string();
    }
    if w.chars().count() == 1 {
        if let Some(i) = w.chars().next().filter(char::is_ascii_lowercase).map(|c| c as usize - 'a' as usize) {
            return LETTERS.unwrap()[i].to_string();
        }
    }
    if w.contains('-') {
        let parts: Vec<String> = w.split('-').filter(|p| !p.is_empty()).map(word).filter(|p| !p.is_empty()).collect();
        return parts.join(" ");
    }
    if let Some((a, b)) = w.split_once('\'') {
        if a.is_empty() {
            return word(b);
        }
        if b == "s" || b.is_empty() {
            return format!("{}s", word(a));
        }
        return word(&format!("{a}{b}"));
    }
    if let Some(p) = lookup(&w) {
        return p;
    }
    if let Some(p) = compound(&w) {
        return p;
    }
    finalize(&analyze(&w, 0))
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "nul", "één", "twee", "drie", "vier", "vijf", "zes", "zeven", "acht", "negen", "tien", "elf", "twaalf", "dertien",
    "veertien", "vijftien", "zestien", "zeventien", "achttien", "negentien",
];
const TENS: [&str; 10] = ["", "tien", "twintig", "dertig", "veertig", "vijftig", "zestig", "zeventig", "tachtig", "negentig"];

/// Number below 1000 as one orthographic word (drieënveertig, tweehonderdvijf).
fn below_thousand(n: u128) -> String {
    let mut s = String::new();
    let h = n / 100;
    let r = n % 100;
    if h > 0 {
        if h > 1 {
            s.push_str(ONES[h as usize].replace('é', "e").as_str());
        }
        s.push_str("honderd");
    }
    if r >= 20 {
        let u = r % 10;
        if u > 0 {
            let one = ONES[u as usize].replace('é', "e");
            s.push_str(&one);
            s.push_str(if one.ends_with('e') { "ën" } else { "en" });
        }
        s.push_str(TENS[(r / 10) as usize]);
    } else if r > 0 || n == 0 {
        s.push_str(&if h > 0 { ONES[r as usize].replace('é', "e") } else { ONES[r as usize].to_string() });
    }
    s
}

pub fn cardinal(n: u128) -> Vec<String> {
    if n < 1000 {
        return vec![below_thousand(n)];
    }
    let mut out = Vec::new();
    let scales: [(u128, &str, &str); 5] = [
        (1_000_000_000_000_000_000, "triljoen", "triljoen"),
        (1_000_000_000_000_000, "biljard", "biljard"),
        (1_000_000_000_000, "biljoen", "biljoen"),
        (1_000_000_000, "miljard", "miljard"),
        (1_000_000, "miljoen", "miljoen"),
    ];
    let mut rest = n;
    for (v, name, _) in scales {
        if rest >= v {
            let k = rest / v;
            rest %= v;
            out.extend(cardinal(k));
            out.push(name.to_string());
        }
    }
    if rest >= 1000 {
        let k = rest / 1000;
        rest %= 1000;
        if k == 1 {
            out.push("duizend".to_string());
        } else if k < 1000 {
            out.push(format!("{}duizend", below_thousand(k).replace('é', "e")));
        } else {
            out.extend(cardinal(k));
            out.push("duizend".to_string());
        }
    }
    if rest > 0 {
        out.push(below_thousand(rest));
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "procent",
        '$' => "dollar",
        '€' => "euro",
        '£' => "pond",
        '&' => "en",
        '+' => "plus",
        '@' => "apenstaartje",
        '°' => "graden",
        '=' => "is",
        _ => return None,
    })
}

// ---------------------------------------------------------------- morphology

/// Unstressed prefixes (pronunciation given) — the stem keeps the main stress.
const PREFIX_UNSTRESSED: &[(&str, &str)] =
    &[("inter", "ɪntər"), ("ver", "vər"), ("her", "hɛr"), ("ont", "ɔnt"), ("be", "bə"), ("ge", "ɣə"), ("er", "ɛr")];

/// Separable / compound-like prefixes that carry the primary stress.
const PREFIX_STRESSED: &[&str] = &[
    "tegen", "samen", "terug", "achter", "binnen", "buiten", "boven", "onder", "tussen", "voort", "over", "door", "voor",
    "weer", "rond", "langs", "opper", "aan", "uit", "mee", "toe", "weg", "oer", "bij", "mis", "op", "af", "in", "om", "na", "on",
];
/// Stressed prefixes that may be followed by a vowel-initial stem (onafhankelijk, uiteindelijk).
const PREFIX_VOWEL_OK: &[&str] =
    &["on", "uit", "in", "af", "aan", "om", "bij", "achter", "onder", "boven", "binnen", "buiten", "tussen", "tegen", "voor", "over"];
/// Native words in -ele whose e is a schwa (the rest are inflected loans in -eel).
const ELE_NATIVE: &[&str] = &["enkele", "edele", "dubbele", "wankele", "ijdele", "vele", "hele", "stekele", "nevele", "regele"];
const TIE_HEADS: &[(&str, &str)] = &[
    ("politie", "poːlˈitsi"), ("actie", "ˈɑksi"), ("informatie", "ɪnfɔrmˈaːtsi"), ("vakantie", "vaːkˈɑnsi"), ("relatie", "reːlˈaːtsi"),
    ("organisatie", "ɔrɣaːnizˈaːtsi"), ("communicatie", "kɔmynikˈaːtsi"), ("administratie", "ɑtministrˈaːtsi"),
    ("registratie", "reːɣistrˈaːtsi"), ("productie", "proːdˈɵksi"), ("inspiratie", "ɪnspirˈaːtsi"), ("traditie", "traːdˈitsi"),
    ("justitie", "jɵstˈitsi"), ("ambitie", "ɑmbˈitsi"), ("coalitie", "koːaːlˈitsi"), ("positie", "poːzˈitsi"),
    ("recreatie", "reːkreːjˈaːtsi"), ("motivatie", "moːtivˈaːtsi"), ("emigratie", "eːmiɣrˈaːtsi"), ("immigratie", "ɪmiɣrˈaːtsi"),
    ("integratie", "ɪntəɣrˈaːtsi"), ("evacuatie", "eːvaːkyʋˈaːtsi"), ("prestatie", "prɛstˈaːtsi"), ("locatie", "loːkˈaːtsi"),
    ("formatie", "fɔrmˈaːtsi"), ("isolatie", "izoːlˈaːtsi"), ("ventilatie", "vɛntilˈaːtsi"), ("notitie", "noːtˈitsi"),
];
/// er- as a pronominal adverb (erop, ervan) or in a few verbs.
const ER_STEMS: &[&str] = &[
    "op", "af", "aan", "in", "uit", "om", "onder", "over", "achter", "bij", "van", "voor", "door", "mee", "naar", "toe",
    "tegen", "tussen", "heen", "langs", "na", "ken", "var", "vaar", "af",
];

fn is_v(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'é' | 'è' | 'ê' | 'ë' | 'ï' | 'ö' | 'ü' | 'ä' | 'â' | 'ô' | 'û' | 'î' | 'á' | 'à' | 'ó' | 'ú' | 'í'
    )
}

/// Can a Dutch word start with the consonant letters `rem` begins with?
fn valid_onset(rem: &str) -> bool {
    let cl: String = rem.chars().take_while(|&c| !is_v(c)).collect();
    if cl.chars().count() <= 1 {
        return true;
    }
    const ONSETS: &[&str] = &[
        "bl", "br", "ch", "chr", "chl", "cl", "cr", "dr", "dw", "fl", "fr", "gl", "gn", "gr", "kl", "kn", "kr", "kw", "pl", "pn",
        "pr", "ps", "ph", "phr", "sc", "sch", "schr", "schw", "scr", "sf", "sj", "sk", "sl", "sm", "sn", "sp", "spl", "spr", "st",
        "str", "sw", "th", "thr", "tj", "tr", "ts", "tsj", "tw", "vl", "vr", "wr", "zw", "ʃ", "dj", "sh", "qu", "squ",
    ];
    ONSETS.contains(&cl.as_str())
}

fn secondary(p: &str) -> String {
    p.replace('ˈ', "ˌ")
}

fn nvowels(w: &str) -> usize {
    let c: Vec<char> = w.chars().collect();
    units(&c).iter().filter(|u| u.v).count()
}

/// Does `rem` (the part after a prefix) hold a full (non-schwa) syllable? A consonant-initial
/// stem of two or more reduced syllables also counts (ver-delen, ver-velend): its first one is full.
fn has_full_syllable(rem: &str) -> bool {
    real_full_syllable(rem) || (nvowels(rem) >= 2 && !rem.starts_with(is_v))
}

fn real_full_syllable(rem: &str) -> bool {
    let c: Vec<char> = rem.chars().collect();
    let u = units(&c);
    let vi: Vec<usize> = (0..u.len()).filter(|&k| u[k].v).collect();
    let m = vi.len();
    for (k, &ui) in vi.iter().enumerate() {
        let letters = u[ui].s.as_str();
        let onset = if ui > 0 && !u[ui - 1].v { u[ui - 1].s.as_str() } else { "" };
        let coda = u.get(ui + 1).filter(|x| !x.v).map(|x| x.s.as_str()).unwrap_or("");
        let last = k + 1 == m;
        let schwa = match letters {
            "e" => {
                if m == 1 {
                    // a monosyllabic stem after a prefix is full unless it is a bare -e/-en/-er/-el (be-ter, ge-ven)
                    matches!(coda, "" | "n" | "r" | "m") || (matches!(coda, "l" | "s") && final_schwa(onset, coda))
                } else if last {
                    final_schwa(onset, coda)
                } else {
                    matches!(coda, "r" | "l" | "n" | "tj")
                }
            }
            "i" => coda.starts_with('g') && (last || coda == "g"),
            "ij" => onset.ends_with('l') && coda.starts_with('k'),
            _ => false,
        };
        if !schwa {
            return true;
        }
    }
    false
}

/// Phonemize a word with its morphology: prefixes first, then the stem.
const FUNC: &[&str] = &["de", "het", "een", "er", "je", "ze", "we", "me", "te", "en", "u", "ge", "op", "in", "om", "aan", "uit", "bij", "na", "mee", "toe", "af", "weg", "voor", "door", "over", "onder", "tegen"];

fn analyze(w: &str, depth: usize) -> String {
    if let Some(p) = exc().get(w) {
        if depth == 0 || !FUNC.contains(&w) {
            return p.to_string();
        }
    }
    let n = w.chars().count();
    // frequent compound heads whose final -tie would otherwise read as ti- (politiebureau)
    for (head, ph) in TIE_HEADS {
        if let Some(rem) = w.strip_prefix(head) {
            if rem.chars().count() >= 3 && has_full_syllable(rem) && valid_onset(rem) {
                return format!("{ph}{}", secondary(&analyze(rem, depth + 1)));
            }
        }
    }
    if depth < 4 {
        for suf in ["atie", "itie", "utie", "otie", "etie"] {
            if let Some(pos) = w.find(suf) {
                let cut = pos + suf.len();
                let (head, rem) = w.split_at(cut);
                if pos >= 3 && rem.chars().count() >= 3 && !rem.starts_with('k') && !rem.starts_with('s') && valid_onset(rem) && has_full_syllable(rem) {
                    return format!("{}{}", analyze(head, depth + 1), secondary(&analyze(rem, depth + 1)));
                }
            }
        }
    }
    if depth < 4 && n >= 4 {
        for (p, ph) in PREFIX_UNSTRESSED {
            let Some(rem) = w.strip_prefix(p) else { continue };
            if n < 5 && *p != "er" {
                continue;
            }
            if !valid_onset(rem) {
                continue;
            }
            let rc = rem.chars().next().unwrap_or('#');
            let first_ok = !is_v(rc)
                || matches!(rc, 'ë' | 'ï' | 'ö' | 'ü' | 'ä')
                || matches!(*p, "ver" | "her" | "ont" | "er" | "inter")
                || (matches!(*p, "be" | "ge") && matches!(rc, 'a' | 'o'));
            if !first_ok || rem.chars().count() < if *p == "er" { 2 } else { 3 } || !has_full_syllable(rem) {
                continue;
            }
            if *p == "er" && !ER_STEMS.iter().any(|s| rem.starts_with(s)) {
                continue;
            }
            if *p == "ge" && (rem.starts_with("ving") || rem.starts_with("ner")) {
                continue; // om-geving, wet-geving
            }
            if *p == "ont" && rem.starts_with('v') {
                return format!("{ph}f{}", analyze(&rem[1..], depth + 1)); // ontvangen: ɔntf
            }
            return format!("{ph}{}", analyze(rem, depth + 1));
        }
        for p in PREFIX_STRESSED {
            let Some(rem) = w.strip_prefix(p) else { continue };
            if !valid_onset(rem) {
                continue;
            }
            let rc = rem.chars().next().unwrap_or('#');
            if is_v(rc) {
                if !PREFIX_VOWEL_OK.contains(p) {
                    continue;
                }
                // a digraph would have formed across the boundary (mee + r, na + a)
                let pl = p.chars().last().unwrap();
                if is_v(pl) && DIGRAPHS.iter().any(|d| d.starts_with(pl) && d[pl.len_utf8()..].starts_with(base(rc))) {
                    continue;
                }
            }
            if rem.chars().count() < 3 || !has_full_syllable(rem) {
                continue;
            }
            // in-nemen, op-peren: across a doubled consonant only a stem with a full vowel counts
            if p.ends_with(rc) && !real_full_syllable(rem) {
                continue;
            }
            let head = match (*p, rem.chars().next().unwrap_or('#')) {
                ("mis", 'd') => "mˈɪz".to_string(),
                ("af", 'd') => "ˈɑv".to_string(),
                ("aan", 'g' | 'k') => "ˈaːŋ".to_string(),
                ("in", 'g' | 'k') => "ˈɪŋ".to_string(),
                ("on", 'g' | 'k') => "ˈɔŋ".to_string(),
                _ => stem(p),
            };
            let tail = analyze(rem, depth + 1);
            return format!("{head}{}", secondary(&tail));
        }
    }
    // -heid / -heden keep the stem intact (zekerheid); h is a morpheme boundary
    for (suf, ph) in [("heden", "heːdən"), ("heids", "hɛɪts"), ("heid", "hɛɪt")] {
        if let Some(stem) = w.strip_suffix(suf) {
            if nvowels(stem) >= 1 {
                return format!("{}{ph}", analyze(stem, depth + 1));
            }
        }
    }
    stem(w)
}

// ---------------------------------------------------------------- letter groups

#[derive(Clone, Debug)]
struct U {
    v: bool,
    /// letters, accents stripped
    s: String,
    /// the accented/diaeresis letter the group starts with, or ' '
    acc: char,
}

const DIGRAPHS: &[&str] = &[
    "eeuw", "ieuw", "aai", "ooi", "oei", "eau", "ouw", "auw", "ieu", "aa", "ee", "oo", "uu", "ie", "oe", "eu", "ui", "ij",
    "ei", "ou", "au", "ai", "oi", "uw", "ay", "ey", "oy", "uy",
];

fn base(c: char) -> char {
    match c {
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'ï' | 'î' | 'í' => 'i',
        'ö' | 'ô' | 'ó' => 'o',
        'ü' | 'û' | 'ú' => 'u',
        'ä' | 'â' | 'á' | 'à' => 'a',
        _ => c,
    }
}

/// Split a word into vowel and consonant letter groups.
fn units(c: &[char]) -> Vec<U> {
    let n = c.len();
    let at = |k: usize| -> char { if k < n { c[k] } else { '#' } };
    // y is a consonant word-initially before a vowel and between vowels; u after q is a consonant
    let is_cons = |k: usize| -> bool {
        let ch = c[k];
        if !is_v(ch) {
            return true;
        }
        if ch == 'y' && is_v(at(k + 1)) && (k == 0 || is_v(c[k - 1])) {
            return true;
        }
        ch == 'u' && k > 0 && c[k - 1] == 'q'
    };
    let mut out: Vec<U> = Vec::new();
    let mut i = 0;
    while i < n {
        if is_cons(i) {
            let start = i;
            while i < n && is_cons(i) {
                i += 1;
            }
            out.push(U { v: false, s: c[start..i].iter().collect(), acc: ' ' });
            continue;
        }
        let ch = c[i];
        let acc = if base(ch) != ch { ch } else { ' ' };
        let mut taken = 1;
        // -eum (museum): e + u, not eu
        let eum = ch == 'e' && at(i + 1) == 'u' && at(i + 2) == 'm' && i + 3 == n && i >= 2;
        let ieu = ch == 'i' && at(i + 1) == 'e' && ((at(i + 2) == 'u' && at(i + 3) != 'w' && i + 3 < n) || at(i + 2) == 'e');
        if !eum && !ieu {
            for d in DIGRAPHS {
                let dc: Vec<char> = d.chars().collect();
                if dc[0] != base(ch) || i + dc.len() > n {
                    continue;
                }
                let mut ok = true;
                for (k, &dk) in dc.iter().enumerate().skip(1) {
                    // later letters must be plain (a diaeresis starts a new group)
                    if c[i + k] != dk {
                        ok = false;
                        break;
                    }
                }
                // a y before a vowel is a consonant (mayo), a u after q too
                let lastc = dc[dc.len() - 1];
                if ok && lastc == 'y' && is_v(at(i + dc.len())) {
                    ok = false;
                }
                if ok {
                    taken = dc.len();
                    break;
                }
            }
        }
        let s: String = c[i..i + taken].iter().map(|&x| base(x)).collect();
        out.push(U { v: true, s, acc });
        i += taken;
    }
    out
}

/// How many consonant "slots" a cluster fills for syllable weight.
fn weight(cl: &str) -> usize {
    let c: Vec<char> = cl.chars().collect();
    let mut i = 0;
    let mut w = 0;
    while i < c.len() {
        let two: String = c[i..(i + 2).min(c.len())].iter().collect();
        let three: String = c[i..(i + 3).min(c.len())].iter().collect();
        if three == "sch" {
            w += 3;
            i += 3;
        } else if matches!(two.as_str(), "ch" | "ng" | "nk" | "ck" | "sj" | "tj" | "dj") {
            w += 2;
            i += 2;
        } else if matches!(two.as_str(), "th" | "ph" | "qu") {
            w += 1;
            i += 2;
        } else if c[i] == 'x' {
            w += 2;
            i += 1;
        } else {
            w += 1;
            i += 1;
        }
    }
    w
}

/// Is a vowel followed by this cluster in an open syllable?
fn open_before(cl: &str, next_vowel: bool) -> bool {
    let w = weight(cl);
    if w == 0 {
        return true;
    }
    if !next_vowel {
        return false;
    }
    if w == 1 {
        return true;
    }
    // loan clusters: C + r, and b/p/k/g/f + l (zebra, publiek), but not tl/dl (atlas)
    let c: Vec<char> = cl.chars().collect();
    if c.len() == 2 {
        let (a, b) = (c[0], c[1]);
        if b == 'r' && matches!(a, 'b' | 'd' | 'g' | 'k' | 'p' | 't' | 'f' | 'v' | 'c') {
            return true;
        }
        if b == 'l' && matches!(a, 'b' | 'p' | 'k' | 'f' | 'c') {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------- stem rules

/// Stress-attracting suffixes: (suffix, index of the stressed vowel group counted from the end).
const STRESS_SUFFIX: &[(&str, usize)] = &[
    ("ië", 3), ("iteiten", 2), ("iteit", 1), ("aties", 2), ("atie", 2), ("ische", 3), ("isch", 2), ("ismen", 2), ("isme", 2),
    ("isten", 2), ("ist", 1), ("uren", 2), ("uur", 1), ("eringen", 3), ("ering", 2), ("eren", 2), ("erijen", 2), ("erij", 1),
    ("ties", 2), ("tie", 2), ("sies", 2), ("sie", 2), ("cie", 2), ("ieën", 2), ("ieuze", 2), ("ieus", 1), ("ieke", 2),
    ("iek", 1), ("iers", 1), ("ier", 1), ("eel", 1), ("ele", 2), ("eurs", 1), ("eur", 1), ("euse", 2), ("euze", 2),
    ("eus", 1), ("onen", 2), ("oon", 1), ("aal", 1), ("ale", 2), ("aten", 2), ("aat", 1), ("ine", 2), ("ijnen", 2),
    ("ijn", 1), ("ein", 1), ("oenen", 2), ("oen", 1), ("eau", 1), ("ese", 2), ("ees", 1), ("eten", 2), ("eet", 1),
    ("anten", 2), ("ant", 1), ("enten", 2), ("ent", 1), ("ijen", 2), ("ij", 1), ("aire", 2), ("air", 1), ("age", 2),
    ("ade", 2), ("ure", 2), ("ose", 2), ("oot", 1), ("oom", 1), ("iet", 1), ("ique", 2), ("ie", 1),
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum K {
    Full,
    Schwa,
    Ig,
    Lijk,
}

/// Rewrite loan spellings whose sound is not derivable letter by letter (ti/ci + vowel -> ʃ).
fn rewrite(w: &str) -> String {
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    let mut out = String::with_capacity(n + 2);
    let mut i = 0;
    while i < n {
        let rest: String = c[i..].iter().collect();
        let prev = if i > 0 { c[i - 1] } else { '#' };
        let prev_ok = (i > 0 && prev != 's') || (i == 0 && rest.starts_with("tio"));
        if rest.starts_with("ssio") {
            out.push_str("ʃʃ");
            i += 3;
            continue;
        }
        if prev_ok && (c[i] == 't' || c[i] == 'c') && i + 2 < n && c[i + 1] == 'i' {
            let after: String = c[i + 2..].iter().collect();
            let sh = after.starts_with('a')
                || after.starts_with('o')
                || after.starts_with("ee")
                || after.starts_with("eu")
                || (c[i] == 't' && after.starts_with("ënt"));
            // but not -tie(s)/-cie(s) and not ti + vowel digraphs like "tieu(w)"
            if sh && !after.starts_with("ieu") {
                out.push('ʃ');
                if after.starts_with("ënt") {
                    out.push('ʃ'); // pa-tiënt: the a stays short
                }
                i += 2;
                continue;
            }
        }
        if rest.starts_with("sion") && prev_ok && !after_vowel(&c, i) {
            out.push_str("ʃon");
            i += 4;
            continue;
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

fn after_vowel(c: &[char], i: usize) -> bool {
    i > 0 && is_v(c[i - 1])
}

fn stem(w: &str) -> String {
    let w = rewrite(w);
    let c: Vec<char> = w.chars().collect();
    let u = units(&c);
    let vi: Vec<usize> = (0..u.len()).filter(|&k| u[k].v).collect();
    let m = vi.len();
    if m == 0 {
        // no vowel letters: an abbreviation read letter by letter (sms, dvd)
        let names = LETTERS.unwrap();
        let mut out = String::new();
        for (i, ch) in c.iter().enumerate() {
            if ch.is_ascii_lowercase() {
                let name = names[*ch as usize - 'a' as usize];
                if i + 1 == c.len() {
                    let (a, b) = name.split_at(name.find(|x: char| "aeiouyɛɔɪɵəø".contains(x)).unwrap_or(0));
                    out.push_str(a);
                    out.push('ˈ');
                    out.push_str(b);
                } else {
                    out.push_str(name);
                }
            }
        }
        return out;
    }
    let letters = |k: usize| -> &str { u[vi[k]].s.as_str() };
    let coda = |k: usize| -> &str { if vi[k] + 1 < u.len() && !u[vi[k] + 1].v { u[vi[k] + 1].s.as_str() } else { "" } };
    let onset = |k: usize| -> &str { if vi[k] > 0 && !u[vi[k] - 1].v { u[vi[k] - 1].s.as_str() } else { "" } };
    let before = |k: usize| -> String { c[..unit_start(&u, vi[k])].iter().collect() };
    let plain_e = |k: usize| -> bool { letters(k) == "e" && (u[vi[k]].acc == ' ' || (u[vi[k]].acc == 'ë' && (coda(k).is_empty() || coda(k).starts_with('n')))) };

    // -- trailing chain of unstressed syllables (-en, -er, -el, -ig, -lijk, -eren …)
    let mut kind = vec![K::Full; m];
    let last_e_open = plain_e(m - 1) && coda(m - 1).is_empty();
    let mut ing_at: Option<usize> = None;
    let mut ij_at: Option<usize> = None;
    let mut k = m - 1;
    while k > 0 {
        let cd = coda(k);
        let last = k == m - 1;
        let on = onset(k);
        let next_schwa = !last && (kind[k + 1] != K::Full || ing_at == Some(k + 1));
        // -ing / -ingen is unstressed but not a schwa: the chain continues past it (ver-eni-ging)
        if letters(k) == "ij" && last && on.ends_with('r') && k >= 2 {
            ij_at = Some(k);
            k -= 1;
            continue;
        }
        let nis = letters(k) == "i" && on.ends_with('n') && (cd == "s" || (cd == "ss" && !last)) && (last || kind[k + 1] == K::Schwa) && k >= 2;
        if ((letters(k) == "i" && cd.starts_with("ng")) || nis) && (last || kind[k + 1] == K::Schwa) && k >= 1 {
            ing_at = Some(k);
            k -= 1;
            continue;
        }
        let tail_kind = if plain_e(k) {
            let erij = cd == "r" && ij_at == Some(k + 1);
            let is_tail = if last {
                final_schwa(on, cd)
            } else {
                (next_schwa || erij)
                    && (matches!(cd, "r" | "l" | "n" | "tj") || (cd == "ll" && !last && kind[k + 1] == K::Lijk))
                    && !(cd == "r" && ((kind[k + 1] == K::Schwa && coda(k + 1) != "") || ing_at == Some(k + 1)) && !eren_schwa(&before(k)))
                    // -ele is mostly the inflected loan suffix -eel (originele); native -ele words are listed
                    && !(cd == "l" && last_e_open && k + 2 == m && m >= 3 && !ELE_NATIVE.iter().any(|x| w.ends_with(x)))
            };
            if is_tail { Some(K::Schwa) } else { None }
        } else if letters(k) == "i" && u[vi[k]].acc == ' ' && cd.starts_with('g') {
            let ig = (last && matches!(cd, "g" | "gs" | "gst" | "gt" | "gd"))
                || (next_schwa && matches!(cd, "g" | "gst" | "gd"))
                || (!last && cd == "g" && ing_at == Some(k + 1));
            if ig { Some(K::Ig) } else { None }
        } else if letters(k) == "ij" && on.ends_with('l') && cd.starts_with('k') {
            let b = before(k);
            let ok = ((last && matches!(cd, "k" | "ks" | "kst")) || (next_schwa && matches!(cd, "k" | "kst")))
                && b != "ge"
                && b.chars().any(is_v);
            if ok { Some(K::Lijk) } else { None }
        } else {
            None
        };
        let Some(tk) = tail_kind else { break };
        kind[k] = tk;
        k -= 1;
    }

    // -- stress
    let mut stress: Option<usize> = None;
    let wb: String = c.iter().map(|&x| if x == 'ë' { x } else { base(x) }).collect();
    for (suf, from_end) in STRESS_SUFFIX {
        if wb.len() <= suf.len() + 1 || !wb.ends_with(suf) {
            continue;
        }
        if *from_end >= m {
            break;
        }
        let cand = m - from_end;
        if cand == 0 {
            break;
        }
        if *suf == "ië" {
            if kind[cand] != K::Full {
                break;
            }
        } else if *suf == "ie" || *suf == "ieën" {
            if !(letters(m - 1) == "ie" || (*suf == "ieën" && m >= 2 && letters(m - 2) == "ie")) {
                break;
            }
            let b = before(m - 1);
            if b.ends_with('l') || b.ends_with("er") {
                break;
            }
        } else if kind[cand] != K::Full {
            break;
        }
        stress = Some(cand);
        break;
    }
    if u[vi[m - 1]].acc == 'é' {
        stress = Some(m - 1);
    }
    let stress = stress.unwrap_or_else(|| (0..m).find(|&k| kind[k] == K::Full).unwrap_or(0));
    if kind[stress] != K::Full {
        kind[stress] = K::Full;
    }

    // -- emit
    let mut out = String::new();
    let mut prev_reduced = false;
    for (idx, un) in u.iter().enumerate() {
        if !un.v {
            let prev_v = idx > 0;
            let next_v = idx + 1 < u.len();
            let next_letters: &str = if next_v { u[idx + 1].s.as_str() } else { "" };
            let first = idx == 0;
            let last = idx + 1 == u.len();
            let mut s: String = un.s.clone();
            // -tie / -ties: t -> ts after a vowel, s after a consonant; -sie after r/n -> zi
            let final_ie = next_v
                && ((next_letters == "ie" && (idx + 2 == u.len() || (idx + 3 == u.len() && u[idx + 2].s == "s")))
                    || (next_letters == "i" && idx + 3 == u.len() && u[idx + 2].acc == 'ë'));
            if final_ie {
                if s.ends_with('t') && !s.ends_with("st") {
                    s.pop();
                    s.push_str(if prev_v && s.is_empty() { "ts" } else { "s" });
                } else if s == "rs" && !first {
                    s.pop();
                    s.push('z');
                }
            }
            let prev_letters: &str = if prev_v { u[idx - 1].s.as_str() } else { "" };
            // -isch(e): sch -> s, the i stays i
            if s.starts_with("sch") && prev_v && prev_letters == "i" && (last || (next_letters == "e" && idx + 2 == u.len())) {
                out.push('s');
                out.push_str(&cons(&s[3..], Ctx { prev_v: false, next_v, first: false, last, next_letters, prev_letters: "" }));
                continue;
            }
            if s == "gn" && prev_v && ((next_letters == "e" && idx + 2 == u.len()) || (next_letters == "ie" && idx + 2 == u.len())) {
                out.push_str("nj");
                continue;
            }
            // -age (massage, percentage): g -> ʒ in words of three or more syllables
            if s == "g" && prev_letters == "a" && m >= 3 && (next_letters == "e" && (idx + 2 == u.len() || (idx + 3 == u.len() && u[idx + 2].s == "s"))) {
                out.push('ʒ');
                continue;
            }
            out.push_str(&cons(&s, Ctx { prev_v, next_v, first, last, next_letters, prev_letters }));
            continue;
        }
        let k = vi.iter().position(|&x| x == idx).unwrap();
        let cd = coda(k);
        let has_next = k + 1 < m;
        let open = open_before(cd, has_next);
        let l = un.s.as_str();
        let stressed = k == stress;
        let hiatus = has_next && cd.is_empty();
        let mut ph: String = match kind[k] {
            K::Schwa | K::Ig | K::Lijk => "ə".to_string(),
            K::Full => {
                if plain_e(k) {
                    let next_i = k + 1 < m && letters(k + 1).starts_with('i');
                    let next_ing = ing_at == Some(k + 1);
                    if hiatus {
                        "eː".to_string()
                    } else {
                        e_quality(k, m, stressed, open, &kind, cd, next_i, next_ing, prev_reduced)
                    }
                } else if l == "ai" && cd.starts_with('r') {
                    "ɛː".to_string()
                } else if l == "i" && cd.starts_with("sch") && (!has_next || (k + 2 == m && kind[k + 1] == K::Schwa)) {
                    "i".to_string()
                } else {
                    vowel(l, open, un.acc).to_string()
                }
            }
        };
        prev_reduced = ph == "ə";
        if hiatus {
            // glide into the next vowel: ide-j-aal, draai-j-en, mooi-j-er, nieu-ʋ-e
            match l {
                "ieuw" => {
                    ph.pop();
                    ph.push('ʋ');
                }
                "e" | "ee" if letters(k + 1).starts_with('a') => ph.push('j'),
                "aai" | "ooi" | "oei" => {
                    ph.pop();
                    ph.push('j');
                }
                _ => {}
            }
        }
        if stressed {
            out.push('ˈ');
        }
        out.push_str(&ph);
    }
    out
}

fn unit_start(u: &[U], idx: usize) -> usize {
    u[..idx].iter().map(|x| x.s.chars().count()).sum()
}

/// Is a word-final `e` syllable with this coda a schwa (-el, -en, -er, -es …)?
fn final_schwa(onset: &str, coda: &str) -> bool {
    let last = onset.chars().last().unwrap_or('#');
    match coda {
        "" | "n" | "r" | "nd" | "rd" | "rs" | "ls" | "ms" | "rt" | "nds" | "rds" | "lds" | "nts" | "tj" | "tjs" => true,
        "m" => last != 'h',
        "nt" => last == 'k',
        "l" => !matches!(last, 'c' | 'w' | 'n' | '#') && !onset.ends_with("st") && !onset.ends_with("sp"),
        "s" => {
            let oc: Vec<char> = onset.chars().collect();
            match oc.len() {
                0 => false,
                1 => !matches!(oc[0], 'r' | 'c' | 's'),
                2 if oc[1] == 'j' => true,
                _ => oc[oc.len() - 1] == oc[oc.len() - 2] && oc[oc.len() - 1] != 'c',
            }
        }
        "ns" => !matches!(last, 'm' | 'p' | 'l') && !onset.ends_with("gr"),
        "ld" => !matches!(last, 'v' | 'g' | 'w' | 'h'),
        "lt" => !matches!(last, 'v' | 'b'),
        _ => false,
    }
}

/// Quality of a full (non-tail) `e`.
#[allow(clippy::too_many_arguments)]
fn e_quality(k: usize, m: usize, stressed: bool, open: bool, kind: &[K], cd: &str, next_i: bool, next_ing: bool, prev_reduced: bool) -> String {
    if k == 0 || stressed || k == m - 1 {
        return (if open { "eː" } else { "ɛ" }).to_string();
    }
    // medial unstressed
    if open {
        let next_final_tail = k + 1 == m - 1 && (kind[k + 1] != K::Full || next_ing);
        if next_final_tail || (cd == "r" && next_i) {
            return "eː".to_string();
        }
        return "ə".to_string();
    }
    let c0 = cd.chars().next().unwrap_or('#');
    let c1 = cd.chars().nth(1).unwrap_or('#');
    // a closed er/el/en syllable is reduced after a full syllable (Nederland, boekenkast),
    // not after a reduced one (medewerker)
    if prev_reduced && matches!(cd.chars().next(), Some('r' | 'l')) {
        return "ɛ".to_string();
    }
    if c0 == c1 {
        return "ɛ".to_string();
    }
    match c0 {
        'r' | 'l' => "ə".to_string(),
        'm' => (if matches!(c1, 'b' | 'p') { "ɛ" } else { "ə" }).to_string(),
        'n' => {
            let next_tail = kind[k + 1] != K::Full;
            let loan = matches!(cd, "nt" | "ns" | "nc" | "nz" | "ntr" | "nts" | "nsi");
            (if cd.starts_with("ng") || (loan && !(next_tail && matches!(cd, "nd" | "ns"))) { "ɛ" } else { "ə" }).to_string()
        }
        _ => "ɛ".to_string(),
    }
}

/// Phones of a vowel group.
fn vowel(l: &str, open: bool, acc: char) -> &'static str {
    match acc {
        'é' => return "eː",
        'è' => return "ɛ",
        'ê' => return "ɛː",
        _ => {}
    }
    match l {
        "a" => if open { "aː" } else { "ɑ" },
        "e" => if open { "eː" } else { "ɛ" },
        "i" => if open { "i" } else { "ɪ" },
        "o" => if open { "oː" } else { "ɔ" },
        "u" => if open { "y" } else { "ɵ" },
        "y" => if open { "i" } else { "ɪ" },
        "aa" => "aː",
        "ee" => "eː",
        "oo" => "oː",
        "uu" => "y",
        "ie" => "i",
        "oe" => "u",
        "eu" => "øː",
        "ui" | "uy" => "œy",
        "ij" | "ei" => "ɛɪ",
        "ou" | "au" | "ouw" | "auw" => "ʌʊ",
        "aai" => "aːi",
        "ooi" => "oːi",
        "oei" => "ui",
        "eeuw" => "eːw",
        "ieuw" => "iw",
        "uw" => "yw",
        "eau" => "oː",
        "ai" | "ay" => "eː",
        "ey" => "ɛɪ",
        "oi" | "oy" => "ɔi",
        "ieu" => "jøː",
        _ => "ə",
    }
}

// ---------------------------------------------------------------- consonants

struct Ctx<'a> {
    prev_v: bool,
    next_v: bool,
    first: bool,
    last: bool,
    next_letters: &'a str,
    prev_letters: &'a str,
}

/// Phones of a consonant letter group (voicing is settled later in `finalize`).
fn cons(s: &str, cx: Ctx) -> String {
    let c: Vec<char> = s.chars().collect();
    let n = c.len();
    let at = |k: usize| -> char { if k < n { c[k] } else { '#' } };
    let front_next = cx.next_v && cx.next_letters.starts_with(|x: char| matches!(x, 'e' | 'i' | 'y'));
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = at(i + 1);
        let n2 = at(i + 2);
        match ch {
            's' => {
                if nx == 'c' && n2 == 'h' {
                    out.push_str(if cx.last && i + 3 == n { "s" } else { "sx" });
                    i += 3;
                    continue;
                }
                if nx == 'j' {
                    out.push('ʃ');
                    i += 2;
                    continue;
                }
                if nx == 'h' && cx.first {
                    out.push('ʃ');
                    i += 2;
                    continue;
                }
                if nx == 'h' && i + 2 == n && (cx.last || cx.next_letters.starts_with('i')) {
                    out.push('ʃ');
                    i += 2;
                    continue;
                }
                let diph = matches!(cx.prev_letters, "ei" | "ij" | "ui" | "ou" | "au" | "ouw" | "auw" | "aai" | "ooi" | "oei" | "eeuw" | "ieuw" | "uw" | "ai" | "oi" | "uy" | "ay" | "ey" | "oy");
                let ese = cx.prev_letters == "ee" && cx.next_letters == "e";
                if n == 1 && cx.prev_v && cx.next_v && !cx.first && !diph && !ese {
                    out.push('z');
                } else {
                    out.push('s');
                }
            }
            'c' => {
                if nx == 'h' {
                    if cx.first && i == 0 {
                        out.push(if matches!(n2, 'r' | 'l') { 'k' } else { 'ʃ' });
                    } else if i > 0 && c[i - 1] == 't' {
                        out.push('ʃ');
                    } else {
                        out.push('x');
                    }
                    i += 2;
                    continue;
                }
                if nx == 'k' {
                    i += 1;
                    continue;
                }
                if nx == 'c' {
                    out.push('k');
                    i += 1;
                    continue;
                }
                let front = i + 1 == n && front_next;
                out.push(if front { 's' } else { 'k' });
            }
            'g' => {
                if nx == 'h' {
                    out.push('ɣ');
                    i += 2;
                    continue;
                }
                out.push('ɣ');
            }
            'n' => {
                if nx == 'g' {
                    out.push('ŋ');
                    // loans: congres, mango, Hongarije keep the g
                    if i + 2 == n && cx.next_letters.starts_with(|x: char| matches!(x, 'a' | 'o' | 'u')) {
                        out.push('ɣ');
                    }
                    i += 2;
                    continue;
                }
                if nx == 'k' || (nx == 'c' && !(i + 2 == n && front_next) && n2 != 'h') {
                    out.push('ŋ');
                } else {
                    out.push('n');
                }
            }
            't' => {
                if nx == 'h' && i == 0 {
                    out.push('t');
                    i += 2;
                    continue;
                }
                if nx == 's' && n2 == 'j' {
                    out.push_str("tʃ");
                    i += 3;
                    continue;
                }
                out.push('t');
            }
            'p' => {
                if nx == 'h' {
                    out.push('f');
                    i += 2;
                    continue;
                }
                out.push('p');
            }
            'q' => {
                out.push_str("kʋ");
                if nx == 'u' {
                    i += 1;
                }
            }
            'x' => out.push_str("ks"),
            'w' => out.push(if cx.first && i == 0 && nx == 'r' { 'v' } else { 'ʋ' }),
            'y' => out.push('j'),
            'h' => {
                if !(cx.last && i + 1 == n && cx.prev_v) {
                    out.push('h');
                }
            }
            'ʃ' => out.push('ʃ'),
            'ç' => out.push('s'),
            'b' | 'd' | 'f' | 'j' | 'k' | 'l' | 'm' | 'r' | 'v' | 'z' => out.push(ch),
            _ => {}
        }
        i += 1;
    }
    out
}

fn voiced(c: char) -> bool {
    matches!(c, 'b' | 'd' | 'v' | 'z' | 'ɣ' | 'ʒ' | 'ɡ')
}
fn voiceless(c: char) -> bool {
    matches!(c, 'p' | 't' | 'k' | 'f' | 's' | 'x' | 'ʃ')
}
fn devoice(c: char) -> char {
    match c {
        'b' => 'p',
        'd' => 't',
        'v' => 'f',
        'z' => 's',
        'ɣ' => 'x',
        'ʒ' => 'ʃ',
        'ɡ' => 'k',
        _ => c,
    }
}
fn is_cons_ph(c: char) -> bool {
    matches!(c, 'p' | 't' | 'k' | 'b' | 'd' | 'f' | 'v' | 's' | 'z' | 'x' | 'ɣ' | 'ʃ' | 'ʒ' | 'ɡ' | 'm' | 'n' | 'ŋ' | 'l' | 'r' | 'ʋ' | 'j' | 'h')
}

/// Final devoicing, cluster voicing assimilation, doubled consonants, dj -> tj.
fn finalize(p: &str) -> String {
    let mut c: Vec<char> = Vec::with_capacity(p.len());
    for ch in p.chars() {
        if is_cons_ph(ch) && c.last() == Some(&ch) {
            continue; // doubled letters (bedden, hebben) and boundary geminates (uittrekken)
        }
        c.push(ch);
    }
    let n = c.len();
    if n == 0 {
        return String::new();
    }
    // word-final obstruent
    let last = n - 1;
    c[last] = devoice(c[last]);
    // syllable-final obstruents are voiceless: before a voiceless obstruent, and before any
    // consonant that cannot start a syllable with it (wereld-wijd, gezond-heid, ad-vies)
    for i in (0..n - 1).rev() {
        if voiced(c[i]) && is_cons_ph(c[i + 1]) {
            let long_before = i >= 1 && c[i - 1] == 'ː';
            let onset = i == 0
                || (matches!((c[i], c[i + 1]), ('b' | 'd' | 'ɣ' | 'v', 'r') | ('b' | 'ɣ' | 'v', 'l') | ('z', 'ʋ') | ('ɡ', 'r' | 'l')) && !(c[i] == 'ɣ' && c[i + 1] == 'l' && long_before))
                || (c[i] == 'd' && c[i + 1] == 'ʋ' && matches!(c[i - 1], 'ə' | 'r'))
                || (matches!(c[i], 'ɣ' | 'v' | 'z' | 'b') && c[i + 1] == 'd');
            if !onset {
                c[i] = devoice(c[i]);
            }
        }
    }
    // progressive: fricative after a voiceless obstruent
    for i in 1..n {
        if matches!(c[i], 'v' | 'z' | 'ɣ') && voiceless(c[i - 1]) && !(c[i] == 'z' && c[i - 1] == 'x') {
            c[i] = devoice(c[i]);
        }
        if c[i] == 'j' && c[i - 1] == 'd' && i + 1 < n && c[i + 1] == 'ə' && i + 2 == n {
            c[i - 1] = 't';
        }
    }
    let mut out = String::with_capacity(p.len());
    let mut prev = '#';
    for (i, &ch) in c.iter().enumerate() {
        if ch == prev && is_cons_ph(ch) {
            continue;
        }
        if ch == 't' && i + 1 < n && c[i + 1] == 'd' {
            continue; // tot-dat, ont-dekken: t merges into d
        }
        out.push(ch);
        prev = ch;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn vowel_length() {
        for (w, p) in [
            ("maken", "mˈaːkən"), ("man", "mˈɑn"), ("mannen", "mˈɑnən"), ("water", "ʋˈaːtər"), ("tafel", "tˈaːfəl"),
            ("lopen", "lˈoːpən"), ("bollen", "bˈɔlən"), ("minuut", "minˈyt"), ("muren", "mˈyrən"), ("bus", "bˈɵs"),
            ("lezen", "lˈeːzən"), ("bedden", "bˈɛdən"), ("liter", "lˈitər"), ("dik", "dˈɪk"), ("kachel", "kˈɑxəl"),
            ("jongen", "jˈɔŋən"), ("koning", "kˈoːnɪŋ"), ("zebra", "zˈeːbraː"), ("publiek", "pyblˈik"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn digraphs_and_diphthongs() {
        for (w, p) in [
            ("huis", "hˈœys"), ("tijd", "tˈɛɪt"), ("klein", "klˈɛɪn"), ("oud", "ˈʌʊt"), ("blauw", "blˈʌʊ"), ("deur", "dˈøːr"),
            ("boek", "bˈuk"), ("dier", "dˈir"), ("mooi", "mˈoːi"), ("draai", "drˈaːi"), ("groei", "ɣrˈui"), ("nieuw", "nˈiw"),
            ("leeuw", "lˈeːw"), ("vrouwen", "vrˈʌʊən"), ("ideeën", "idˈeːən"), ("zeeën", "zˈeːən"), ("reünie", "reːynˈi"),
            ("ruïne", "ryˈinə"), ("coördinatie", "koːɔrdinˈaːtsi"), ("museum", "myzˈeːɵm"), ("café", "kaːfˈeː"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn consonants() {
        for (w, p) in [
            ("school", "sxˈoːl"), ("schrijven", "sxrˈɛɪvən"), ("logisch", "lˈoːxis"), ("lachen", "lˈɑxən"), ("dag", "dˈɑx"),
            ("dagen", "dˈaːɣən"), ("zegt", "zˈɛxt"), ("hoofd", "hˈoːft"), ("abt", "ˈɑpt"), ("web", "ʋˈɛp"), ("werk", "ʋˈɛrk"),
            ("wreed", "vrˈeːt"), ("quiz", "kʋˈɪs"), ("taxi", "tˈɑksi"), ("centrum", "sˈɛntrɵm"), ("cultuur", "kɵltˈyr"),
            ("bank", "bˈɑŋk"), ("zingen", "zˈɪŋən"), ("meisje", "mˈɛɪʃə"), ("hondje", "hˈɔntjə"), ("thee", "tˈeː"),
            ("afzien", "ˈɑfsˌin"), ("ontvangen", "ɔntfˈɑŋən"), ("afval", "ˈɑfˌɑl"), ("nationaal", "naːʃoːnˈaːl"), ("internationaal", "ɪntərnaːʃoːnˈaːl"),
            ("sociaal", "soːʃˈaːl"), ("politie", "poːlˈitsi"), ("actie", "ˈɑksi"), ("functie", "fˈɵŋksi"), ("relatie", "reːlˈaːtsi"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn schwa_and_morphology() {
        for (w, p) in [
            ("gegeven", "ɣəɣˈeːvən"), ("beter", "bˈeːtər"), ("gevel", "ɣˈeːvəl"), ("verder", "vˈɛrdər"), ("bezoek", "bəzˈuk"),
            ("werkgever", "ʋˈɛrkxeːvər"), ("werknemer", "ʋˈɛrkneːmər"), ("wetenschap", "ʋˈeːtənsxɑp"), ("medewerker", "mˈeːdəʋɛrkər"),
            ("leveren", "lˈeːvərən"), ("studeren", "stydˈeːrən"), ("kinderen", "kˈɪndərən"), ("regelen", "rˈeːɣələn"),
            ("vriendelijk", "vrˈindələk"), ("mogelijk", "mˈoːɣələk"), ("eigenlijk", "ˈɛɪɣənlək"), ("gelijk", "ɣəlˈɛɪk"),
            ("twintig", "tʋˈɪntəx"), ("weinig", "ʋˈɛɪnəx"), ("nodige", "nˈoːdəɣə"), ("mogelijkheid", "mˈoːɣələkhɛɪt"),
            ("ontwikkeling", "ɔntʋˈɪkəlɪŋ"), ("onafhankelijk", "ˈɔnˌɑfhˌɑŋkələk"), ("ongeveer", "ˈɔŋɣəvˌeːr"), ("aangeven", "ˈaːŋɣˌeːvən"),
            ("interessant", "ɪntərɛsˈɑnt"), ("universiteit", "ˌynivɛrsitˈɛɪt"), ("telefoon", "teːləfˈoːn"),
            ("generaal", "ɣeːnərˈaːl"), ("materiaal", "maːteːriˈaːl"), ("student", "stydˈɛnt"), ("energie", "eːnərʒˈi"),
            ("economie", "eːkoːnoːmˈi"), ("familie", "fˈaːmili"), ("regering", "rəɣˈeːrɪŋ"), ("verandering", "vərˈɑndərɪŋ"),
            ("eenentwintig", "ˈeːnəntʋɪntəx"), ("zevenentwintig", "zˈeːvənəntʋɪntəx"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), vec!["nul"]);
        assert_eq!(cardinal(21), vec!["eenentwintig"]);
        assert_eq!(cardinal(22), vec!["tweeëntwintig"]);
        assert_eq!(cardinal(123), vec!["honderddrieëntwintig"]);
        assert_eq!(cardinal(2500), vec!["tweeduizend", "vijfhonderd"]);
        assert_eq!(cardinal(1_000_000), vec!["één", "miljoen"]);
        assert_eq!(cardinal(3_000_000_000), vec!["drie", "miljard"]);
        assert_eq!(word("tweeëntwintig"), "tʋˈeːəntʋɪntəx");
    }
}
