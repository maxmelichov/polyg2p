//! French grapheme-to-phoneme rules, espeak-style notation.
//!
//! French spelling is regular enough for rules once the silent final letters,
//! the `-ent` verb ending and the e-caduc are handled; a small built-in lexicon
//! covers the irregular residue (femme, fils, monsieur, ...). Notation follows
//! espeak-ng `fr-fr`: `ʁ`, nasal vowels `ɑ̃ ɛ̃ ɔ̃ œ̃` (combining tilde), the glide
//! /ɥ/ written `y`, `â` as `aː`, primary stress `ˈ` before the last vowel.
//! Function words carry no stress unless they end a phrase (as espeak does);
//! obligatory liaison is added by the sentence pass with the consonant attached
//! to the preceding word (`lez ɑ̃fɑ̃`).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::Out;

/// marker letters produced by `rewrite`: a nasal ɑ̃ vowel, and tʃ
const NAS_A: char = '\u{E000}';
const TCH: char = '\u{E001}';
const TH: char = '\u{E002}';

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "virgule";
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "be", "se", "de", "ə", "ɛf", "ʒe", "aʃ", "i", "ʒi", "ka", "ɛl", "ɛm", "ɛn", "o", "pe", "ky", "ɛʁ",
    "ɛs", "te", "y", "ve", "dubləve", "iks", "iɡʁɛk", "zɛd",
]);

/// Irregular words (phones without stress). Everything else is rules.
const LEX: &[(&str, &str)] = &[
    ("et", "e"), ("est", "ɛ"), ("es", "ɛ"), ("ai", "e"), ("as", "a"), ("eu", "y"), ("eus", "y"), ("eut", "y"),
    ("eût", "y"), ("eue", "y"), ("eues", "y"), ("eurent", "yʁ"), ("eusse", "ys"), ("eussent", "ys"),
    ("les", "le"), ("des", "de"), ("mes", "me"), ("tes", "te"), ("ses", "se"), ("ces", "se"),
    ("monsieur", "məsjø"), ("messieurs", "mesjø"), ("femme", "fam"), ("femmes", "fam"), ("fils", "fis"),
    ("œil", "œj"), ("yeux", "jø"), ("pays", "pei"), ("paysage", "peizaʒ"), ("paysages", "peizaʒ"),
    ("paysan", "peizɑ̃"), ("paysans", "peizɑ̃"), ("paysanne", "peizan"), ("paysannes", "peizan"),
    ("abbaye", "abei"), ("examen", "ɛɡzamɛ̃"), ("examens", "ɛɡzamɛ̃"), ("oignon", "ɔɲɔ̃"), ("oignons", "ɔɲɔ̃"),
    ("second", "səɡɔ̃"), ("seconde", "səɡɔ̃d"), ("secondes", "səɡɔ̃d"), ("seconds", "səɡɔ̃"),
    ("secondaire", "səɡɔ̃dɛʁ"), ("secondaires", "səɡɔ̃dɛʁ"), ("solennel", "sɔlanɛl"), ("solennelle", "sɔlanɛl"),
    ("solennellement", "sɔlanɛlmɑ̃"), ("ennemi", "ɛnmi"), ("ennemis", "ɛnmi"), ("ennemie", "ɛnmi"),
    ("aujourd'hui", "oʒuʁdyi"), ("quelqu'un", "kɛlkœ̃"), ("tous", "tus"), ("plus", "plys"), ("os", "ɔs"),
    ("sens", "sɑ̃s"), ("gens", "ʒɑ̃"), ("mars", "maʁs"), ("ours", "uʁs"), ("mœurs", "mœʁs"), ("six", "sis"),
    ("dix", "dis"), ("huit", "yit"), ("vingt", "vɛ̃"), ("cinq", "sɛ̃k"), ("sept", "sɛt"), ("neuf", "nœf"),
    ("but", "byt"), ("août", "ut"), ("clef", "kle"), ("clefs", "kle"), ("nerf", "nɛʁ"), ("nerfs", "nɛʁ"),
    ("cerf", "sɛʁ"), ("cerfs", "sɛʁ"), ("œufs", "ø"), ("bœufs", "bø"), ("porc", "pɔʁ"), ("porcs", "pɔʁ"),
    ("tabac", "taba"), ("estomac", "ɛstɔma"), ("caoutchouc", "kautʃu"), ("gars", "ɡa"), ("soixante", "swasɑ̃t"),
    ("bruxelles", "bʁysɛl"), ("auxerre", "osɛʁ"), ("poêle", "pwal"), ("moelle", "mwal"), ("moelleux", "mwalø"),
    ("faisant", "fəzɑ̃"), ("faisait", "fəzɛ"), ("faisais", "fəzɛ"), ("faisons", "fəzɔ̃"), ("faisiez", "fəzje"),
    ("faisions", "fəzjɔ̃"), ("faisaient", "fəzɛ"), ("bienfaisant", "bjɛ̃fəzɑ̃"), ("malfaisant", "malfəzɑ̃"),
    ("automne", "otɔn"), ("fusil", "fyzi"), ("fusils", "fyzi"), ("gentil", "ʒɑ̃ti"), ("gentils", "ʒɑ̃ti"),
    ("outil", "uti"), ("outils", "uti"), ("persil", "pɛʁsi"), ("sourcil", "suʁsi"), ("sourcils", "suʁsi"),
    ("almanach", "almana"), ("hier", "jɛʁ"), ("hiver", "ivɛʁ"), ("amer", "amɛʁ"), ("enfer", "ɑ̃fɛʁ"),
    ("cancer", "kɑ̃sɛʁ"), ("cancers", "kɑ̃sɛʁ"), ("super", "sypɛʁ"), ("hyper", "ipɛʁ"), ("éther", "etɛʁ"),
    ("cuiller", "kyijɛʁ"), ("revolver", "ʁevɔlvɛʁ"), ("fer", "fɛʁ"), ("mer", "mɛʁ"), ("cher", "ʃɛʁ"),
    ("leader", "lidœʁ"), ("leaders", "lidœʁ"), ("speaker", "spikœʁ"), ("dealer", "dilœʁ"), ("manager", "manadʒœʁ"),
    ("managers", "manadʒœʁ"), ("designer", "dizajnœʁ"), ("trader", "tʁɛdœʁ"), ("scooter", "skutœʁ"),
    ("reporter", "ʁəpɔʁtɛʁ"), ("container", "kɔ̃tɛnɛʁ"), ("poker", "pɔkɛʁ"), ("gangster", "ɡɑ̃ɡstɛʁ"),
    ("hamster", "amstɛʁ"), ("master", "mastɛʁ"), ("cluster", "klœstœʁ"), ("bunker", "bunkɛʁ"), ("joker", "ʒɔkɛʁ"),
    ("sticker", "stikœʁ"), ("tanker", "tɑ̃kœʁ"), ("week-end", "wikɛnd"), ("football", "futbol"), ("foot", "fut"),
    ("match", "matʃ"), ("matchs", "matʃ"), ("jazz", "dʒaz"), ("jean", "ʒɑ̃"), ("jeans", "dʒin"), ("job", "dʒɔb"),
    ("jogging", "dʒɔɡiŋ"), ("wagon", "vaɡɔ̃"), ("wagons", "vaɡɔ̃"), ("sandwich", "sɑ̃dwitʃ"), ("gym", "ʒim"),
    ("parfum", "paʁfœ̃"), ("parfums", "paʁfœ̃"), ("rouen", "ʁwɑ̃"), ("caen", "kɑ̃"), ("paul", "pɔl"),
    ("rome", "ʁɔm"), ("donc", "dɔ̃k"), ("zinc", "zɛ̃ɡ"), ("legs", "lɛ"), ("asthme", "asm"), ("isthme", "ism"),
    ("gaz", "ɡaz"), ("quiz", "kwiz"), ("metz", "mɛs"), ("brut", "bʁyt"), ("chut", "ʃyt"), ("zut", "zyt"),
    ("scout", "skut"), ("ouest", "wɛst"), ("est-ce", "ɛs"), ("dot", "dɔt"), ("mat", "mat"), ("net", "nɛt"),
    ("internet", "ɛ̃tɛʁnɛt"), ("tibet", "tibɛ"), ("set", "sɛt"), ("huit", "yit"), ("accessit", "aksesit"),
    ("transit", "tʁɑ̃zit"), ("déficit", "defisit"), ("granit", "ɡʁanit"), ("audit", "odit"), ("brexit", "bʁɛɡzit"),
    ("sud", "syd"), ("bled", "blɛd"), ("oued", "wɛd"), ("raid", "ʁɛd"), ("caïd", "kaid"), ("david", "david"),
    ("madrid", "madʁid"), ("alfred", "alfʁɛd"), ("mohamed", "mɔamɛd"), ("bagdad", "baɡdad"), ("tchad", "tʃad"),
    ("djihad", "dʒiad"), ("fjord", "fjɔʁd"), ("stop", "stɔp"), ("top", "tɔp"), ("clip", "klip"), ("slip", "slip"),
    ("cap", "kap"), ("cep", "sɛp"), ("handicap", "ɑ̃dikap"), ("ketchup", "kɛtʃœp"), ("drap", "dʁa"), ("draps", "dʁa"),
    ("iceberg", "isbɛʁɡ"), ("bourg", "buʁ"), ("croc", "kʁo"), ("escroc", "ɛskʁo"), ("accroc", "akʁo"),
    ("broc", "bʁo"), ("clerc", "klɛʁ"), ("marc", "maʁ"), ("respect", "ʁɛspɛ"), ("aspect", "aspɛ"),
    ("aspects", "aspɛ"), ("suspect", "syspɛ"), ("instinct", "ɛ̃stɛ̃"), ("distinct", "distɛ̃"), ("succinct", "syksɛ̃"),
    ("exempt", "ɛɡzɑ̃"), ("prompt", "pʁɔ̃"), ("biceps", "bisɛps"), ("forceps", "fɔʁsɛps"), ("laps", "laps"),
    ("chips", "ʃips"), ("bus", "bys"), ("sus", "sys"), ("us", "ys"), ("lis", "lis"), ("lys", "lis"), ("vis", "vis"),
    ("bis", "bis"), ("jadis", "ʒadis"), ("gratis", "ɡʁatis"), ("hélas", "elas"), ("atlas", "atlas"),
    ("alias", "aljas"), ("maïs", "mais"), ("cassis", "kasis"), ("oasis", "ɔazis"), ("tennis", "tenis"),
    ("iris", "iʁis"), ("anus", "anys"), ("virus", "viʁys"), ("bonus", "bɔnys"), ("campus", "kɑ̃pys"),
    ("cactus", "kaktys"), ("lotus", "lɔtys"), ("rébus", "ʁebys"), ("terminus", "tɛʁminys"), ("autobus", "otɔbys"),
    ("consensus", "kɔ̃sɛ̃sys"), ("hiatus", "jatys"), ("corpus", "kɔʁpys"), ("focus", "fɔkys"), ("fœtus", "fetys"),
    ("lapsus", "lapsys"), ("sinus", "sinys"), ("cumulus", "kymylys"), ("stimulus", "stimylys"),
    ("prospectus", "pʁɔspɛktys"), ("papyrus", "papiʁys"), ("radius", "ʁadjys"), ("cursus", "kyʁsys"),
    ("processus", "pʁɔsesys"), ("cosmos", "kɔsmos"), ("albatros", "albatʁos"), ("rhinocéros", "ʁinoseʁɔs"),
    ("pathos", "patos"), ("tétanos", "tetanos"), ("ès", "ɛs"), ("palmarès", "palmaʁɛs"), ("herpès", "ɛʁpɛs"),
    ("faciès", "fasjɛs"), ("express", "ɛkspʁɛs"), ("dessus", "dəsy"), ("dessous", "dəsu"), ("pouls", "pu"),
    ("cul", "ky"), ("index", "ɛ̃dɛks"), ("félix", "feliks"), ("astérix", "asteʁiks"), ("phénix", "feniks"),
    ("aix", "ɛks"), ("flux", "fly"), ("reflux", "ʁəfly"), ("afflux", "afly"), ("influx", "ɛ̃fly"),
    ("munich", "mynik"), ("zurich", "zyʁik"), ("bach", "bak"), ("krach", "kʁak"), ("yacht", "jɔt"),
    ("gnocchi", "ɲɔki"), ("adam", "adɑ̃"), ("tram", "tʁam"), ("islam", "islam"), ("vietnam", "vjɛtnam"),
    ("macadam", "makadam"), ("ville", "vil"), ("villes", "vil"), ("mille", "mil"), ("tranquille", "tʁɑ̃kil"),
    ("tranquilles", "tʁɑ̃kil"), ("lille", "lil"), ("gilles", "ʒil"), ("achille", "aʃil"), ("bacille", "basil"),
    ("ubiquité", "ybikyite"), ("équilatéral", "ekyilateʁal"), ("linguistique", "lɛ̃ɡyistik"),
    ("linguistiques", "lɛ̃ɡyistik"), ("linguiste", "lɛ̃ɡyist"), ("linguistes", "lɛ̃ɡyist"), ("aiguille", "eɡyij"),
    ("aiguilles", "eɡyij"), ("aiguiller", "eɡyije"), ("aiguillon", "eɡyijɔ̃"), ("guy", "ɡi"), ("quelque", "kɛlkə"), ("cet", "sɛt"), ("relai", "ʁəlɛ"), ("relais", "ʁəlɛ"), ("cobaye", "kɔbaj"), ("bidonville", "bidɔ̃vil"), ("bidonvilles", "bidɔ̃vil"), ("péréquation", "peʁekwasjɔ̃"), ("quadrant", "kadʁɑ̃"), ("inuit", "inyit"), ("inuits", "inyit"), ("chiite", "ʃiit"), ("chiites", "ʃiit"),
    ("lesdits", "ledi"), ("lesdites", "ledit"), ("desdits", "dedi"), ("desdites", "dedit"), ("ledit", "lədi"), ("ladite", "ladit"),
    ("ennuyant", "ɑ̃nyijɑ̃"), ("outremer", "utʁəmɛʁ"), ("outre-mer", "utʁəmɛʁ"), ("contresens", "kɔ̃tʁəsɑ̃s"),
    ("nounours", "nunuʁs"), ("bisounours", "bizunuʁs"), ("appendicite", "apɛ̃disit"), ("appendice", "apɛ̃dis"),
    ("hautbois", "obwa"), ("montgolfière", "mɔ̃ɡɔlfjɛʁ"), ("gentilshommes", "ʒɑ̃tizɔm"), ("satiété", "sasjete"),
    ("vénitienne", "venisjɛn"), ("machiavélique", "makjavelik"), ("machiavel", "makjavɛl"), ("dichotomie", "dikɔtɔmi"),
    ("eucharistie", "økaʁisti"), ("kalachnikov", "kalaʃnikɔv"), ("loch", "lɔk"), ("hindi", "indi"), ("rut", "ʁyt"),
    ("zig", "zi"), ("instit", "ɛ̃stit"), ("aprèm", "apʁɛm"), ("yoyo", "jojo"), ("liber", "libɛʁ"), ("oculus", "ɔkylys"),
    ("humus", "ymys"), ("typhus", "tifys"), ("tonus", "tɔnys"), ("minus", "minys"), ("minibus", "minibys"),
    ("eucalyptus", "økaliptys"), ("syphilis", "sifilis"), ("clitoris", "klitɔʁis"), ("pastis", "pastis"), ("ibis", "ibis"),
    ("albinos", "albinos"), ("talmud", "talmyd"), ("mazout", "mazut"), ("prout", "pʁut"), ("peps", "pɛps"),
    ("merguez", "mɛʁɡɛz"), ("quidam", "kidam"), ("caméraman", "kameʁaman"), ("schtroumpf", "ʃtʁumf"),
    ("coyote", "kɔjɔt"), ("pitbull", "pitbul"), ("boomerang", "bumʁɑ̃ɡ"), ("m'sieur", "msjø"), ("mb", "mb"),
    ("chrétienté", "kʁetjɛ̃te"), ("clercs", "klɛʁ"), ("couscous", "kuskus"), ("tournevis", "tuʁnəvis"),
    ("summum", "sɔmɔm"), ("tacos", "takos"), ("yaourts", "jauʁt"), ("haïtiens", "aisjɛ̃"), ("haïtienne", "aisjɛn"),
    ("hospital", "ɔspital"), ("psyché", "psiʃe"), ("blockchain", "blɔktʃɛn"), ("cricket", "kʁikɛt"), ("racket", "ʁakɛt"),
    ("fix", "fiks"), ("hammam", "amam"), ("smiley", "smajlɛ"), ("gin", "dʒin"), ("yard", "jaʁd"), ("plug", "plœɡ"),
    ("pizzeria", "pidzeʁja"), ("munster", "mœ̃stɛʁ"), ("tong", "tɔ̃ɡ"), ("tongs", "tɔ̃ɡ"), ("waouh", "waw"),
    ("quatuor", "kwatyɔʁ"), ("utérus", "yteʁys"), ("chelem", "ʃlɛm"), ("canyon", "kaɲɔn"), ("alsacien", "alzasjɛ̃"),
    ("alsacienne", "alzasjɛn"), ("bruts", "bʁyt"), ("bruxellois", "bʁysɛlwa"), ("bruxelloise", "bʁysɛlwaz"),
    ("lithium", "litjɔm"), ("bienheureux", "bjɛ̃nøʁø"), ("transat", "tʁɑ̃zat"), ("flirt", "flœʁt"), ("flirter", "flœʁte"),
    ("placebo", "plasebo"), ("omnibus", "ɔmnibys"), ("ressusciter", "ʁesysite"), ("ressuscité", "ʁesysite"),
    ("alien", "aljɛn"), ("aliens", "aljɛn"), ("imam", "imam"), ("spaghetti", "spaɡeti"), ("ghetto", "ɡeto"),
    ("empathie", "ɑ̃pati"), ("psychique", "psiʃik"), ("psychiatre", "psikjatʁ"), ("psychiatrie", "psikjatʁi"),
    ("charisme", "kaʁism"), ("charismatique", "kaʁismatik"), ("choléra", "kɔleʁa"), ("cholestérol", "kɔlɛsteʁɔl"),
    ("schizophrénie", "skizɔfʁeni"), ("schizophrène", "skizɔfʁɛn"), ("zénith", "zenit"), ("élisabeth", "elizabɛt"),
    ("édith", "edit"), ("maestro", "maɛstʁo"), ("graal", "ɡʁal"), ("fasciste", "faʃist"), ("fascisme", "faʃism"),
    ("gentleman", "dʒɛntləman"), ("barman", "baʁman"), ("superman", "sypɛʁman"), ("batman", "batman"),
    ("sus", "sy"), ("ut", "yt"), ("dixit", "diksit"), ("clim", "klim"), ("slalom", "slalɔm"), ("versus", "vɛʁsys"),
    ("métis", "metis"), ("infarctus", "ɛ̃faʁktys"), ("yuan", "jyan"), ("thaï", "taj"), ("thaïlandais", "tajlɑ̃dɛ"),
    ("thaïlande", "tajlɑ̃d"), ("miam", "mjam"), ("oups", "ups"), ("aïe", "aj"), ("gus", "ɡys"), ("matos", "matos"),
    ("gratos", "ɡʁatos"), ("synopsis", "sinɔpsis"), ("maya", "maja"), ("fayard", "fajaʁ"), ("badminton", "badmintɔn"),
    ("newton", "njutɔn"), ("saoudien", "saudjɛ̃"), ("saoudite", "saudit"), ("cockpit", "kɔkpit"), ("gadget", "ɡadʒɛt"),
    ("gadgets", "ɡadʒɛt"), ("jackpot", "ʒakpɔt"), ("kebab", "kebab"), ("veto", "veto"), ("apartheid", "apaʁtɛd"),
    ("nets", "nɛt"), ("let", "lɛt"), ("pat", "pat"), ("spot", "spɔt"), ("shot", "ʃɔt"), ("drop", "dʁɔp"), ("kid", "kid"),
    ("soft", "sɔft"), ("hard", "aʁd"), ("bit", "bit"), ("bits", "bit"), ("shit", "ʃit"), ("speed", "spid"), ("deal", "dil"),
    ("light", "lajt"), ("sky", "skaj"), ("stone", "ston"), ("cloud", "klawd"), ("dead", "dɛd"), ("beach", "bitʃ"),
    ("geek", "ɡik"), ("check", "tʃɛk"), ("cheese", "tʃiz"), ("cake", "kɛk"), ("skate", "skɛt"), ("joke", "dʒok"),
    ("trade", "tʁɛd"), ("trading", "tʁediŋ"), ("quick", "kwik"), ("quid", "kwid"), ("bluff", "blœf"), ("rush", "ʁœʃ"),
    ("punch", "pœntʃ"), ("must", "mœst"), ("trust", "tʁœst"), ("truck", "tʁœk"), ("running", "ʁœniŋ"),
    ("remake", "ʁimɛk"), ("remix", "ʁəmiks"), ("mix", "miks"), ("cover", "kɔvœʁ"), ("driver", "dʁajvœʁ"),
    ("roller", "ʁɔlœʁ"), ("sniper", "snajpœʁ"), ("gamer", "ɡɛmœʁ"), ("spoiler", "spɔjlœʁ"), ("water", "watɛʁ"),
    ("weber", "vebɛʁ"), ("spencer", "spɛnsɛʁ"), ("bacon", "bekɔn"), ("random", "ʁɑ̃dɔm"), ("copyright", "kɔpiʁajt"),
    ("streaming", "stʁimiŋ"), ("leadership", "lidœʁʃip"), ("building", "bildiŋ"), ("lobbying", "lɔbiiŋ"),
    ("crowdfunding", "kʁawdfœndiŋ"), ("underground", "œndœʁɡʁawnd"), ("cowboys", "kobɔj"), ("flow", "flo"),
    ("slow", "slo"), ("low", "lo"), ("round", "ʁawnd"), ("bingo", "biŋɡo"), ("yang", "jaŋ"), ("mustang", "mystɑ̃ɡ"),
    ("bang", "bɑ̃ɡ"), ("because", "bikoz"), ("hype", "ajp"), ("strike", "stʁajk"), ("because", "bikoz"),
    ("spin", "spin"), ("ciao", "tʃao"), ("jihad", "dʒiad"), ("chum", "tʃɔm"), ("putsch", "putʃ"), ("wu", "wu"),
    ("siemens", "simɛns"), ("fantasy", "fɑ̃tezi"), ("soccer", "sɔkœʁ"), ("bitcoin", "bitkɔjn"), ("challenger", "tʃalɛndʒœʁ"),
    ("abc", "abese"), ("dsl", "dezɔle"), ("bcp", "boku"), ("qqn", "kɛlkœ̃"), ("mdr", "ɛmdeɛʁ"), ("tramways", "tʁamwɛ"),
    ("emmanuel", "emanyɛl"), ("emma", "ema"), ("mesdames", "medam"), ("mesdemoiselles", "medmwazɛl"), ("saône", "son"),
    ("subsiste", "sybzist"), ("subsistent", "sybzist"), ("subsister", "sybziste"), ("subsistance", "sybzistɑ̃s"),
    ("parasol", "paʁasɔl"), ("leclerc", "ləklɛʁ"), ("agnès", "aɲɛs"), ("tunis", "tynis"), ("régis", "ʁeʒis"),
    ("vénus", "venys"), ("opus", "ɔpys"), ("pénis", "penis"), ("blocus", "blɔkys"), ("istanbul", "istɑ̃bul"),
    ("beyrouth", "beʁut"), ("deschamps", "deʃɑ̃"), ("senior", "senjɔʁ"), ("seniors", "senjɔʁ"), ("sénior", "senjɔʁ"),
    ("interviews", "ɛ̃tɛʁvju"), ("boycott", "bɔjkɔt"), ("penalty", "penalti"), ("modem", "mɔdɛm"), ("zen", "zɛn"),
    ("rom", "ʁɔm"), ("strasbourg", "stʁazbuʁ"), ("israël", "izʁaɛl"), ("mons", "mɔ̃s"), ("lens", "lɑ̃s"),
    ("florent", "flɔʁɑ̃"), ("laurent", "lɔʁɑ̃"), ("dylan", "dilan"), ("abraham", "abʁaam"), ("xi", "ksi"),
    ("lesquels", "lekɛl"), ("lesquelles", "lekɛl"), ("auxquels", "okɛl"), ("auxquelles", "okɛl"), ("desquels", "dekɛl"),
    ("desquelles", "dekɛl"), ("duquel", "dykɛl"), ("mme", "madam"), ("mlle", "madmwazɛl"), ("dr", "dɔktœʁ"),
    ("mathieu", "matjø"), ("matthieu", "matjø"), ("kevin", "kevin"), ("inter", "ɛ̃tɛʁ"), ("alsace", "alzas"),
    ("dan", "dan"), ("smith", "smit"), ("baby", "bebi"), ("flash", "flaʃ"), ("benjamin", "bɛ̃ʒamɛ̃"), ("appart", "apaʁt"),
    ("francis", "fʁɑ̃sis"), ("reims", "ʁɛ̃s"), ("pizza", "pidza"), ("weekend", "wikɛnd"), ("stand", "stɑ̃d"),
    ("cannabis", "kanabis"), ("oeufs", "ø"), ("boeufs", "bø"), ("oeil", "œj"), ("oeuf", "œf"), ("boeuf", "bœf"),
    ("oeuvre", "œvʁ"), ("oeuvres", "œvʁ"), ("coeur", "kœʁ"), ("coeurs", "kœʁ"), ("soeur", "sœʁ"), ("soeurs", "sœʁ"),
    ("voeu", "vø"), ("voeux", "vø"), ("noeud", "nø"), ("marketing", "maʁketiŋ"), ("ed", "ɛd"), ("vincent", "vɛ̃sɑ̃"), ("buts", "byt"), ("derrière", "dɛʁjɛʁ"),
    ("quelques", "kɛlkə"), ("presque", "pʁɛskə"), ("puisque", "pyiskə"), ("lorsque", "lɔʁskə"), ("jusque", "ʒyskə"),
    ("quoique", "kwakə"), ("jusques", "ʒysk"), ("monsieur", "məsjø"), ("oui", "wi"), ("ouïe", "wi"), ("mil", "mil"),
    ("jeudi", "ʒødi"), ("jeûne", "ʒøn"), ("meule", "møl"), ("veule", "vøl"), ("eux", "ø"), ("ceux", "sø"),
    ("fleurir", "flœʁiʁ"), ("fleuriste", "flœʁist"), ("fleuri", "flœʁi"), ("fleurie", "flœʁi"), ("fleuron", "flœʁɔ̃"),
    ("chrome", "kʁom"), ("gnome", "ɡnom"), ("axiome", "aksjom"), ("idiome", "idjom"), ("zone", "zon"),
    ("zones", "zon"), ("clone", "klon"), ("drone", "dʁon"), ("cyclone", "siklon"), ("amazone", "amazon"),
    ("grosse", "ɡʁos"), ("grosses", "ɡʁos"), ("fosse", "fos"), ("fosses", "fos"), ("grossir", "ɡʁosiʁ"),
    ("grossesse", "ɡʁosɛs"), ("alcool", "alkɔl"), ("alcools", "alkɔl"), ("alcoolique", "alkɔlik"),
    ("zoo", "zo"), ("zoos", "zo"), ("mayonnaise", "majɔnɛz"), ("kayak", "kajak"), ("papaye", "papaj"),
    ("bayonne", "bajɔn"), ("bayard", "bajaʁ"), ("cayenne", "kajɛn"), ("mayotte", "majɔt"), ("ennui", "ɑ̃nyi"),
    ("ennuis", "ɑ̃nyi"), ("ennuyer", "ɑ̃nyije"), ("ennuyeux", "ɑ̃nyijø"), ("ennuyeuse", "ɑ̃nyijøz"),
    ("ennuyé", "ɑ̃nyije"), ("ennuie", "ɑ̃nyi"), ("ennoblir", "ɑ̃nɔbliʁ"), ("emmener", "ɑ̃məne"), ("emmène", "ɑ̃mɛn"),
    ("emménager", "ɑ̃menaʒe"), ("emmagasiner", "ɑ̃maɡazine"), ("emmêler", "ɑ̃mele"), ("condamner", "kɔ̃dane"),
    ("condamné", "kɔ̃dane"), ("condamnée", "kɔ̃dane"), ("condamnés", "kɔ̃dane"), ("condamnation", "kɔ̃danasjɔ̃"),
    ("condamne", "kɔ̃dan"), ("damner", "dane"), ("damné", "dane"), ("automnal", "otɔnal"), ("jeune", "ʒœn"),
    ("jeunes", "ʒœn"), ("jeunesse", "ʒœnɛs"), ("déjeuner", "deʒœne"), ("neuvième", "nœvjɛm"), ("veuvage", "vœvaʒ"),
    ("meubler", "mœble"), ("meublé", "mœble"), ("dix-sept", "disɛt"), ("dix-huit", "dizyit"), ("dix-neuf", "diznœf"),
    ("interview", "ɛ̃tɛʁvju"), ("whisky", "wiski"), ("sweat", "swit"), ("steak", "stɛk"), ("break", "bʁɛk"),
    ("team", "tim"), ("cheval", "ʃəval"), ("chevaux", "ʃəvo"), ("premier", "pʁəmje"), ("premiers", "pʁəmje"),
    ("premières", "pʁəmjɛʁ"), ("première", "pʁəmjɛʁ"), ("hockey", "ɔkɛ"), ("jockey", "ʒɔkɛ"), ("poney", "pɔnɛ"),
    ("volley", "vɔlɛ"), ("ananas", "anana"), ("moins", "mwɛ̃"), ("longtemps", "lɔ̃tɑ̃"), ("doigt", "dwa"),
    ("doigts", "dwa"), ("vingts", "vɛ̃"), ("exact", "ɛɡzakt"), ("exacts", "ɛɡzakt"), ("chef-d'œuvre", "ʃɛdœvʁ"),
    ("monsieurs", "məsjø"), ("sculpter", "skylte"), ("sculpture", "skyltyʁ"), ("sculpteur", "skyltœʁ"),
    ("baptême", "batɛm"), ("baptiser", "batize"), ("baptisé", "batize"), ("baptême", "batɛm"), ("compter", "kɔ̃te"),
    ("dompter", "dɔ̃te"), ("sept", "sɛt"), ("septième", "sɛtjɛm"), ("cinquième", "sɛ̃kjɛm"), ("ananas", "anana"),
    ("entier", "ɑ̃tje"), ("coefficient", "kɔefisjɑ̃"), ("efficient", "efisjɑ̃"), ("conscient", "kɔ̃sjɑ̃"),
    ("inconscient", "ɛ̃kɔ̃sjɑ̃"), ("subconscient", "sypkɔ̃sjɑ̃"), ("omniscient", "ɔmnisjɑ̃"), ("omniscience", "ɔmnisjɑ̃s"),
    ("conscience", "kɔ̃sjɑ̃s"), ("consciences", "kɔ̃sjɑ̃s"), ("inconscience", "ɛ̃kɔ̃sjɑ̃s"), ("science", "sjɑ̃s"),
    ("sciences", "sjɑ̃s"), ("hyène", "jɛn"), ("ailleurs", "ajœʁ"), ("faon", "fɑ̃"), ("paon", "pɑ̃"), ("taon", "tɑ̃"),
    ("laon", "lɑ̃"), ("saoul", "su"), ("saoule", "sul"), ("fût", "fy"), ("fûmes", "fym"), ("fûtes", "fyt"),
    ("thym", "tɛ̃"), ("lynx", "lɛ̃ks"), ("sphinx", "sfɛ̃ks"), ("larynx", "laʁɛ̃ks"), ("pharynx", "faʁɛ̃ks"),
    ("sprint", "spʁint"), ("yaourt", "jauʁt"), ("yoga", "jɔɡa"), ("yoghourt", "jɔɡuʁt"), ("shampoing", "ʃɑ̃pwɛ̃"),
    ("coing", "kwɛ̃"), ("poing", "pwɛ̃"), ("poings", "pwɛ̃"), ("gong", "ɡɔ̃ɡ"), ("ping-pong", "piŋpɔ̃ɡ"),
    ("punk", "pœ̃k"), ("junk", "dʒœ̃k"), ("brunch", "bʁœnʃ"), ("lunch", "lœnʃ"), ("bungalow", "bœ̃ɡalo"),
    ("club", "klœb"), ("clubs", "klœb"), ("pub", "pœb"), ("bug", "bœɡ"), ("bugs", "bœɡ"), ("rhum", "ʁɔm"),
    ("rugby", "ʁyɡbi"), ("puzzle", "pœzl"), ("fun", "fœn"), ("fan", "fan"), ("fans", "fan"), ("gang", "ɡɑ̃ɡ"),
    ("ranch", "ʁɑ̃tʃ"), ("lobby", "lɔbi"), ("hobby", "ɔbi"), ("hall", "ol"), ("halls", "ol"), ("goal", "ɡol"),
    ("coach", "kotʃ"), ("coachs", "kotʃ"), ("road", "ʁod"), ("toast", "tost"), ("boat", "bot"), ("show", "ʃo"),
    ("shows", "ʃo"), ("bowling", "boliŋ"), ("snowboard", "snobɔʁd"), ("blockbuster", "blɔkbœstœʁ"),
    ("business", "biznɛs"), ("fitness", "fitnɛs"), ("stress", "stʁɛs"), ("miss", "mis"), ("boss", "bɔs"),
    ("baseball", "bɛzbol"), ("basket", "baskɛt"), ("basket-ball", "baskɛtbol"), ("volley-ball", "vɔlɛbol"),
    ("handball", "ɑ̃dbal"), ("cocktail", "kɔktɛl"), ("e-mail", "imɛl"), ("mail", "mɛl"), ("mails", "mɛl"),
    ("email", "imɛl"), ("emails", "imɛl"), ("détail", "detaj"), ("détails", "detaj"), ("portail", "pɔʁtaj"),
    ("cookie", "kuki"), ("cookies", "kuki"), ("smartphone", "smaʁtfɔn"), ("iphone", "ajfon"), ("téléphone", "telefɔn"),
    ("microphone", "mikʁɔfɔn"), ("saxophone", "saksɔfɔn"), ("francophone", "fʁɑ̃kɔfɔn"), ("casting", "kastiŋ"),
    ("design", "dizajn"), ("designs", "dizajn"), ("cowboy", "kobɔj"), ("boy", "bɔj"), ("boys", "bɔj"),
    ("playboy", "plɛbɔj"), ("roy", "ʁwa"), ("tramway", "tʁamwɛ"), ("gay", "ɡɛ"), ("gays", "ɡɛ"), ("okay", "ɔkɛ"),
    ("spray", "spʁɛ"), ("ok", "ɔkɛ"), ("rock", "ʁɔk"), ("clown", "klun"), ("clowns", "klun"), ("town", "taun"),
    ("down", "daun"), ("brown", "bʁawn"), ("bowl", "bol"), ("pipeline", "pajplajn"), ("online", "ɔnlajn"),
    ("timeline", "tajmlajn"), ("skyline", "skajlajn"), ("deadline", "dɛdlajn"), ("live", "lajv"), ("like", "lajk"),
    ("likes", "lajk"), ("bike", "bajk"), ("nike", "najk"), ("mike", "majk"), ("drive", "dʁajv"), ("five", "fajv"),
    ("wifi", "wifi"), ("wi-fi", "wifi"), ("web", "wɛb"), ("www", "dubləvedubləvedubləve"), ("iphone", "ajfon"),
    ("ipad", "ajpad"), ("pop", "pɔp"), ("hip-hop", "ipɔp"), ("rap", "ʁap"), ("tip", "tip"), ("tips", "tips"),
    ("chip", "tʃip"), ("blog", "blɔɡ"), ("blogs", "blɔɡ"), ("gag", "ɡaɡ"), ("tag", "taɡ"), ("tags", "taɡ"),
    ("zigzag", "ziɡzaɡ"), ("grog", "ɡʁɔɡ"), ("mug", "mœɡ"), ("golf", "ɡɔlf"), ("surf", "sœʁf"), ("turf", "tyʁf"),
    ("œuvre", "œvʁ"), ("œuvres", "œvʁ"), ("œuf", "œf"), ("bœuf", "bœf"), ("cœur", "kœʁ"), ("cœurs", "kœʁ"),
    ("sœur", "sœʁ"), ("sœurs", "sœʁ"), ("vœu", "vø"), ("vœux", "vø"), ("nœud", "nø"), ("nœuds", "nø"),
    ("œdème", "edɛm"), ("œsophage", "ezɔfaʒ"), ("œcuménique", "ekymenik"), ("œdipe", "edip"),
    ("pneu", "pnø"), ("pneus", "pnø"), ("rythme", "ʁitm"), ("rythmes", "ʁitm"), ("algorithme", "alɡɔʁitm"),
    ("logarithme", "lɔɡaʁitm"), ("asthmatique", "asmatik"), ("second", "səɡɔ̃"), ("tienne", "tjɛn"), ("tiennes", "tjɛn"),
    ("tiennent", "tjɛn"), ("tiens", "tjɛ̃"), ("tient", "tjɛ̃"), ("tien", "tjɛ̃"), ("chrétien", "kʁetjɛ̃"),
    ("chrétiens", "kʁetjɛ̃"), ("chrétienne", "kʁetjɛn"), ("chrétiennes", "kʁetjɛn"), ("égyptien", "eʒipsjɛ̃"),
    ("égyptienne", "eʒipsjɛn"), ("égyptiens", "eʒipsjɛ̃"), ("vénitien", "venisjɛ̃"), ("haïtien", "aisjɛ̃"),
    ("tahitien", "taisjɛ̃"), ("capétien", "kapesjɛ̃"), ("martien", "maʁsjɛ̃"), ("martiens", "maʁsjɛ̃"),
    ("tiers", "tjɛʁ"), ("fier", "fjɛʁ"), ("fiers", "fjɛʁ"), ("fière", "fjɛʁ"), ("fièrement", "fjɛʁmɑ̃"),
    ("ver", "vɛʁ"), ("vers", "vɛʁ"), ("mers", "mɛʁ"), ("fers", "fɛʁ"), ("chers", "ʃɛʁ"), ("hivers", "ivɛʁ"),
    ("enfers", "ɑ̃fɛʁ"), ("amers", "amɛʁ"), ("tiers", "tjɛʁ"), ("anvers", "ɑ̃vɛʁs"), ("envers", "ɑ̃vɛʁ"),
    ("cyber", "sibɛʁ"), ("laser", "lazɛʁ"), ("lasers", "lazɛʁ"), ("polyester", "pɔljɛstɛʁ"), ("ester", "ɛstɛʁ"),
    ("jupiter", "ʒypitɛʁ"), ("lucifer", "lysifɛʁ"), ("magister", "maʒistɛʁ"), ("sweater", "swɛtœʁ"),
    ("hamburger", "ɑ̃buʁɡœʁ"), ("thriller", "sʁilœʁ"), ("killer", "kilœʁ"), ("partner", "paʁtnɛʁ"),
    ("partners", "paʁtnɛʁ"), ("teaser", "tizœʁ"), ("best-seller", "bɛstsɛlœʁ"), ("starter", "staʁtɛʁ"),
    ("scanner", "skanɛʁ"), ("scanners", "skanɛʁ"), ("tuner", "tynɛʁ"), ("mixer", "miksœʁ"), ("shaker", "ʃɛkœʁ"),
    ("gluten", "ɡlytɛn"), ("pollen", "pɔlɛn"), ("lichen", "likɛn"), ("abdomen", "abdɔmɛn"), ("spécimen", "spesimɛn"),
    ("amen", "amɛn"), ("cyclamen", "siklamɛn"), ("dolmen", "dɔlmɛn"), ("hymen", "imɛn"), ("carmen", "kaʁmɛn"),
    ("eden", "edɛn"), ("open", "ɔpɛn"), ("golden", "ɡɔldɛn"), ("ben", "bɛn"), ("amen", "amɛn"), ("sen", "sɛn"),
    ("yen", "jɛn"), ("yens", "jɛn"), ("citoyen", "sitwajɛ̃"), ("citoyens", "sitwajɛ̃"), ("moyen", "mwajɛ̃"),
    ("moyens", "mwajɛ̃"), ("doyen", "dwajɛ̃"), ("mitoyen", "mitwajɛ̃"), ("item", "itɛm"), ("items", "itɛm"),
    ("tandem", "tɑ̃dɛm"), ("totem", "tɔtɛm"), ("requiem", "ʁekyijɛm"), ("harem", "aʁɛm"), ("idem", "idɛm"),
    ("jérusalem", "ʒeʁyzalɛm"), ("bethléem", "bɛtleɛm"), ("intérim", "ɛ̃teʁim"), ("joachim", "ʒɔaʃim"),
    ("ibrahim", "ibʁaim"), ("selim", "selim"), ("karim", "kaʁim"), ("album", "albɔm"), ("albums", "albɔm"),
    ("maximum", "maksimɔm"), ("minimum", "minimɔm"), ("forum", "fɔʁɔm"), ("forums", "fɔʁɔm"), ("aluminium", "alyminjɔm"),
    ("médium", "medjɔm"), ("opium", "ɔpjɔm"), ("uranium", "yʁanjɔm"), ("sodium", "sɔdjɔm"), ("calcium", "kalsjɔm"),
    ("stadium", "stadjɔm"), ("podium", "pɔdjɔm"), ("référendum", "ʁefeʁɑ̃dɔm"), ("quorum", "kɔʁɔm"),
    ("muséum", "myzeɔm"), ("curriculum", "kyʁikylɔm"), ("sérum", "seʁɔm"), ("géranium", "ʒeʁanjɔm"),
    ("symposium", "sɛ̃pozjɔm"), ("consortium", "kɔ̃sɔʁsjɔm"), ("atrium", "atʁijɔm"), ("latium", "lasjɔm"),
    ("rhum", "ʁɔm"), ("album", "albɔm"), ("humour", "ymuʁ"), ("humeur", "ymœʁ"), ("humain", "ymɛ̃"),
    ("mathématiques", "matematik"), ("maths", "mat"), ("math", "mat"), ("william", "wiljam"), ("sam", "sam"),
    ("spam", "spam"), ("jam", "dʒam"), ("ram", "ʁam"), ("cam", "kam"), ("webcam", "wɛbkam"), ("amsterdam", "amstɛʁdam"),
    ("rotterdam", "ʁɔtɛʁdam"), ("siam", "sjam"), ("nom", "nɔ̃"), ("noms", "nɔ̃"), ("prénom", "pʁenɔ̃"),
    ("prénoms", "pʁenɔ̃"), ("pronom", "pʁɔnɔ̃"), ("renom", "ʁənɔ̃"), ("surnom", "syʁnɔ̃"), ("tom", "tɔm"),
    ("cd-rom", "sedeʁɔm"), ("intercom", "ɛ̃tɛʁkɔm"), ("dom", "dɔm"), ("boom", "bum"), ("zoom", "zum"),
    ("mnémonique", "mnemɔnik"), ("mnémotechnique", "mnemɔtɛknik"), ("pneumonie", "pnømɔni"), ("pneumatique", "pnømatik"),
    ("psychologie", "psikɔlɔʒi"), ("psychologue", "psikɔlɔɡ"), ("psychologique", "psikɔlɔʒik"), ("ptérodactyle", "pteʁɔdaktil"),
    ("sceptique", "sɛptik"), ("septique", "sɛptik"), ("septembre", "sɛptɑ̃bʁ"), ("sceau", "so"), ("sceaux", "so"),
    ("scène", "sɛn"), ("scènes", "sɛn"), ("ascension", "asɑ̃sjɔ̃"), ("asseoir", "aswaʁ"), ("rasseoir", "ʁaswaʁ"),
    ("surseoir", "syʁswaʁ"), ("suer", "sye"), ("sueur", "syœʁ"), ("muet", "myɛ"), ("muette", "myɛt"),
    ("muets", "myɛ"), ("fuel", "fjul"), ("duel", "dyɛl"), ("cruel", "kʁyɛl"), ("cruelle", "kʁyɛl"),
    ("cruellement", "kʁyɛlmɑ̃"), ("ruelle", "ʁyɛl"), ("nuée", "nye"), ("tuer", "tye"), ("tué", "tye"), ("tués", "tye"),
    ("tuée", "tye"), ("tuées", "tye"), ("tue", "ty"), ("tuent", "ty"), ("jésus", "ʒezy"), ("jésus-christ", "ʒezykʁi"),
    ("christ", "kʁist"), ("antéchrist", "ɑ̃tekʁist"), ("chaos", "kao"), ("chaotique", "kaɔtik"), ("chœur", "kœʁ"),
    ("chœurs", "kœʁ"), ("choral", "kɔʁal"), ("chorale", "kɔʁal"), ("chorales", "kɔʁal"), ("chorégraphie", "kɔʁeɡʁafi"),
    ("chorégraphe", "kɔʁeɡʁaf"), ("chorizo", "ʃɔʁizo"), ("écho", "eko"), ("échos", "eko"), ("orchestre", "ɔʁkɛstʁ"),
    ("orchestres", "ɔʁkɛstʁ"), ("orchidée", "ɔʁkide"), ("orchidées", "ɔʁkide"), ("archéologie", "aʁkeɔlɔʒi"),
    ("archéologue", "aʁkeɔlɔɡ"), ("archéologique", "aʁkeɔlɔʒik"), ("archaïque", "aʁkaik"), ("archange", "aʁkɑ̃ʒ"),
    ("archétype", "aʁketip"), ("lichen", "likɛn"), ("varech", "vaʁɛk"), ("machisme", "matʃism"), ("macho", "matʃo"),
    ("michel", "miʃɛl"), ("sandwichs", "sɑ̃dwitʃ"), ("sandwiches", "sɑ̃dwitʃ"), ("coach", "kotʃ"), ("tchèque", "tʃɛk"),
    ("tchèques", "tʃɛk"), ("tchat", "tʃat"), ("chat", "ʃa"), ("chats", "ʃa"), ("chatte", "ʃat"), ("yiddish", "jidiʃ"),
    ("sushi", "suʃi"), ("sushis", "suʃi"), ("shampooing", "ʃɑ̃pwɛ̃"), ("lingual", "lɛ̃ɡwal"), ("jaguar", "ʒaɡwaʁ"),
    ("iguane", "iɡwan"), ("guano", "ɡwano"), ("guadeloupe", "ɡwadlup"), ("guatemala", "ɡwatemala"),
    ("guacamole", "ɡwakamɔl"), ("aquarium", "akwaʁjɔm"), ("aquariums", "akwaʁjɔm"), ("aquatique", "akwatik"),
    ("aquatiques", "akwatik"), ("aquarelle", "akwaʁɛl"), ("aqueux", "akø"), ("équateur", "ekwatœʁ"),
    ("équation", "ekwasjɔ̃"), ("équations", "ekwasjɔ̃"), ("équatorial", "ekwatɔʁjal"), ("adéquat", "adekwa"),
    ("adéquate", "adekwat"), ("adéquation", "adekwasjɔ̃"), ("quadrilatère", "kwadʁilatɛʁ"), ("quadruple", "kwadʁypl"),
    ("quadrature", "kwadʁatyʁ"), ("quartz", "kwaʁts"), ("square", "skwaʁ"), ("quasar", "kwazaʁ"),
    ("quaker", "kwɛkœʁ"), ("aquaculture", "akwakyltyʁ"), ("quota", "kɔta"), ("quotas", "kɔta"),
    ("quotidien", "kɔtidjɛ̃"), ("quotidienne", "kɔtidjɛn"), ("quotidiens", "kɔtidjɛ̃"), ("quotient", "kɔsjɑ̃"),
    ("quiétude", "kjetyd"), ("inquiétude", "ɛ̃kjetyd"), ("ubiquité", "ybikyite"), ("aquifère", "akyifɛʁ"),
    ("équidistant", "ekyidistɑ̃"), ("équivalent", "ekivalɑ̃"), ("équivalents", "ekivalɑ̃"), ("équivalente", "ekivalɑ̃t"),
    ("équivaloir", "ekivalwaʁ"), ("loquace", "lɔkwas"), ("in-quarto", "inkwaʁto"), ("quasi", "kazi"),
    ("quasiment", "kazimɑ̃"), ("squash", "skwaʃ"), ("squat", "skwat"), ("squatter", "skwate"), ("jacquard", "ʒakaʁ"),
    ("jacques", "ʒak"), ("acquérir", "akeʁiʁ"), ("acquis", "aki"), ("acquise", "akiz"), ("acquisition", "akizisjɔ̃"),
    ("acquitter", "akite"), ("pied", "pje"), ("pieds", "pje"), ("sied", "sje"), ("assied", "asje"),
    ("nez", "ne"), ("chez", "ʃe"), ("assez", "ase"), ("rez", "ʁe"), ("riz", "ʁi"), ("raz", "ʁa"), ("berlioz", "bɛʁljoz"),
    ("fez", "fɛz"), ("suez", "syɛz"), ("ils", "il"), ("il", "il"), ("fil", "fil"), ("fils", "fis"), ("avril", "avʁil"),
    ("cil", "sil"), ("civil", "sivil"), ("exil", "ɛɡzil"), ("profil", "pʁɔfil"), ("péril", "peʁil"), ("subtil", "syptil"),
    ("vil", "vil"), ("nil", "nil"), ("gril", "ɡʁil"), ("grésil", "ɡʁezil"), ("nombril", "nɔ̃bʁil"), ("baril", "baʁil"),
    ("chenil", "ʃənil"), ("fournil", "fuʁni"), ("coutil", "kuti"), ("cerveau", "sɛʁvo"), ("cerveaux", "sɛʁvo"),
    ("wagon-lit", "vaɡɔ̃li"), ("hollande", "ɔlɑ̃d"), ("vosges", "voʒ"), ("lyon", "ljɔ̃"), ("lyonnais", "ljɔnɛ"),
    ("yankee", "jɑ̃ki"), ("yankees", "jɑ̃ki"), ("yeti", "jeti"), ("yéti", "jeti"), ("yéménite", "jemenit"), ("yemen", "jemɛn"),
    ("yémen", "jemɛn"), ("ypres", "ipʁ"), ("yves", "iv"), ("yvonne", "ivɔn"), ("yvelines", "ivlin"), ("yvan", "ivɑ̃"),
    ("ysengrin", "izɑ̃ɡʁɛ̃"), ("x", "iks"), ("y", "i"), ("w", "dubləve"), ("kg", "kilɔɡʁam"), ("km", "kilɔmɛtʁ"), ("cm", "sɑ̃timɛtʁ"), ("mm", "milimɛtʁ"), ("kgs", "kilɔɡʁam"), ("k", "ka"), ("q", "ky"),
    ("h", "aʃ"), ("z", "zɛd"), ("g", "ʒe"), ("j", "ʒi"), ("b", "be"), ("c", "se"), ("d", "de"), ("f", "ɛf"),
    ("l", "ɛl"), ("m", "ɛm"), ("n", "ɛn"), ("p", "pe"), ("r", "ɛʁ"), ("s", "ɛs"), ("t", "te"), ("v", "ve"),
    ("a", "a"), ("e", "ə"), ("i", "i"), ("o", "o"), ("u", "y"), ("à", "a"), ("ô", "o"), ("ç", "sesedij"),
    ("pm", "peɛm"), ("am", "aɛm"), ("ok", "ɔkɛ"), ("ô", "o"), ("ah", "a"), ("oh", "o"), ("eh", "e"), ("hé", "e"),
    ("hein", "ɛ̃"), ("euh", "ø"), ("bah", "ba"), ("ouf", "uf"), ("bof", "bɔf"), ("hum", "ɔm"), ("hop", "ɔp"),
    ("youpi", "jupi"), ("ouais", "wɛ"), ("eh", "e"), ("ô", "o"), ("ben", "bɛ̃"), ("mec", "mɛk"), ("mecs", "mɛk"),
    ("truc", "tʁyk"), ("trucs", "tʁyk"), ("cinq", "sɛ̃k"), ("brin", "bʁɛ̃"), ("moelleuse", "mwaløz"),
    ("poêlon", "pwalɔ̃"), ("poêlée", "pwale"), ("poêles", "pwal"), ("aulne", "on"), ("aulnes", "on"),
    ("renault", "ʁəno"), ("thibault", "tibo"), ("perrault", "pɛʁo"), ("arnault", "aʁno"), ("foucault", "fuko"),
    ("hérault", "eʁo"), ("legault", "ləɡo"), ("pouls", "pu"), ("soûl", "su"), ("soûle", "sul"), ("soûls", "su"),
    ("fusil-mitrailleur", "fyzimitʁajœʁ"), ("outillage", "utijaʒ"), ("outiller", "utije"), ("gentille", "ʒɑ̃tij"),
    ("gentilles", "ʒɑ̃tij"), ("gentillesse", "ʒɑ̃tijɛs"), ("gentiment", "ʒɑ̃timɑ̃"), ("gentilhomme", "ʒɑ̃tijɔm"),
    ("million", "miljɔ̃"), ("millions", "miljɔ̃"), ("milliard", "miljaʁ"), ("milliards", "miljaʁ"), ("millier", "milje"),
    ("milliers", "milje"), ("millième", "miljɛm"), ("millimètre", "milimɛtʁ"), ("millimètres", "milimɛtʁ"),
    ("milligramme", "miliɡʁam"), ("millénaire", "milenɛʁ"), ("millésime", "milezim"), ("millet", "mijɛ"),
    ("milliardaire", "miljaʁdɛʁ"), ("millionnaire", "miljɔnɛʁ"), ("million", "miljɔ̃"), ("village", "vilaʒ"),
    ("villages", "vilaʒ"), ("villa", "vila"), ("villas", "vila"), ("villageois", "vilaʒwa"), ("villégiature", "vileʒjatyʁ"),
    ("ville-marie", "vilmaʁi"), ("pupille", "pypij"), ("osciller", "ɔsile"), ("oscillation", "ɔsilasjɔ̃"),
    ("oscillations", "ɔsilasjɔ̃"), ("oscillateur", "ɔsilatœʁ"), ("distiller", "distile"), ("distillerie", "distilʁi"),
    ("distillation", "distilasjɔ̃"), ("instiller", "ɛ̃stile"), ("capillaire", "kapilɛʁ"), ("capillaires", "kapilɛʁ"),
    ("cyrillique", "siʁilik"), ("pénicilline", "penisilin"), ("codicille", "kɔdisil"), ("pusillanime", "pyzilanim"),
    ("armillaire", "aʁmilɛʁ"), ("maxillaire", "maksilɛʁ"), ("axillaire", "aksilɛʁ"), ("bacillaire", "basilɛʁ"),
    ("vaudeville", "vodvil"), ("illusion", "ilyzjɔ̃"), ("illusions", "ilyzjɔ̃"), ("bille", "bij"), ("billes", "bij"),
    ("billard", "bijaʁ"), ("villeneuve", "vilnœv"), ("villefranche", "vilfʁɑ̃ʃ"), ("villeurbanne", "vilœʁban"),
    ("trafic", "tʁafik"), ("sommes", "sɔm"), ("comme", "kɔm"), ("homme", "ɔm"), ("hommes", "ɔm"), ("pomme", "pɔm"),
    ("pommes", "pɔm"), ("somme", "sɔm"), ("gomme", "ɡɔm"), ("nomme", "nɔm"), ("tome", "tɔm"), ("tomes", "tɔm"),
    ("atome", "atom"), ("atomes", "atom"), ("génome", "ʒenom"), ("dôme", "dom"), ("arôme", "aʁom"), ("syndrome", "sɛ̃dʁom"),
    ("hippodrome", "ipɔdʁom"), ("aérodrome", "aeʁɔdʁom"), ("vélodrome", "velɔdʁom"), ("palindrome", "palɛ̃dʁom"),
    ("monochrome", "mɔnɔkʁom"), ("polychrome", "pɔlikʁom"), ("psychologie", "psikɔlɔʒi"),
];

/// Clitics and function words that espeak leaves unstressed inside a phrase.
/// Those in NEVER_STRESSED stay unstressed even at the end of a phrase.
const UNSTRESSED: &[&str] = &[
    "le", "la", "les", "un", "une", "des", "du", "de", "au", "aux", "à", "en", "dans", "sur", "sous", "chez",
    "par", "pour", "sans", "vers", "entre", "contre", "je", "tu", "il", "elle", "on", "nous", "vous", "ils",
    "elles", "me", "te", "se", "leur", "y", "ce", "ça", "celle", "celles", "mon", "ton", "son", "ma", "ta", "sa",
    "mes", "tes", "ses", "notre", "votre", "leurs", "nos", "vos", "cet", "cette", "ces", "quel", "quelle",
    "quels", "quelles", "chaque", "tout", "toute", "tous", "toutes", "qui", "que", "dont", "où", "et", "ou", "ni",
    "mais", "car", "comme", "quand", "est", "a", "es", "ne", "pas", "plus", "très", "trop", "peu", "tel", "telle",
    "tels", "telles", "autre", "autres",  "ceux", "quant", "dès",
    "lorsque", "puisque", "jusque", "presque", "quelque", "quelques", "afin", "alors", "ainsi", "aussi", "avec",
    "depuis", "lui", "cela", "ceci", "jamais", "rien", "assez", "beaucoup", "encore", "toujours", "plusieurs",
    "quoi", "suis", "lorsqu", "puisqu", "jusqu", "presqu", "quelqu", "quoiqu", "quoique",
];
/// Multi-syllable function words: `ˌ` inside a phrase, `ˈ` at its end (espeak).
const SECONDARY: &[&str] = &[
    "avec", "depuis", "lui", "cela", "ceci", "quelque", "quelques", "lorsque", "puisque", "afin", "alors", "ainsi",
    "aussi", "jamais", "rien", "assez", "beaucoup", "encore", "toujours", "plusieurs", "quoi", "suis", "celui",
    "quoique", "jusque", "presque",
];
const NEVER_STRESSED: &[&str] = &[
    "de", "des", "du", "je", "me", "te", "se", "ne", "ma", "ta", "sa", "mes", "tes", "ses", "nos", "vos", "ces",
    "cet", "cette", "mon", "ton", "notre", "votre", "leur", "car", "dont", "ou", "ni", "qu", "l", "d", "j", "n",
    "m", "t", "s", "c", "lorsqu", "puisqu", "jusqu", "presqu", "quelqu", "quoiqu",
];

/// Words beginning with an aspirated h: no liaison or elision before them.
const H_ASPIRE: &[&str] = &[
    "hache", "haie", "haine", "haïr", "hall", "halle", "halte", "hamac", "hamburger", "hameau", "hanche",
    "handicap", "hangar", "hanter", "harceler", "hardi", "harem", "hareng", "haricot", "harpe", "hasard",
    "hâte", "hausse", "haut", "hauteur", "havre", "hennir", "hérisson", "hernie", "héron", "héros", "hêtre",
    "hibou", "hideux", "hiérarchie", "hisser", "hockey", "hollande", "homard", "hongrie", "honte", "hoquet",
    "hors", "hotte", "houle", "housse", "hublot", "huche", "huer", "huit", "hurler", "hussard", "hutte",
    "hamster", "hockey", "hobby", "hip", "hop", "hall", "halo", "hamas", "hanoï", "hanovre", "harlem",
    "hollywood", "hongkong", "houston",
];

fn lex() -> &'static HashMap<&'static str, &'static str> {
    static L: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    L.get_or_init(|| LEX.iter().copied().collect())
}
fn set(list: &'static [&'static str], cell: &'static OnceLock<HashSet<&'static str>>) -> &'static HashSet<&'static str> {
    cell.get_or_init(|| list.iter().copied().collect())
}
fn unstressed() -> &'static HashSet<&'static str> {
    static S: OnceLock<HashSet<&'static str>> = OnceLock::new();
    set(UNSTRESSED, &S)
}
fn secondary() -> &'static HashSet<&'static str> {
    static S: OnceLock<HashSet<&'static str>> = OnceLock::new();
    set(SECONDARY, &S)
}
fn never_stressed() -> &'static HashSet<&'static str> {
    static S: OnceLock<HashSet<&'static str>> = OnceLock::new();
    set(NEVER_STRESSED, &S)
}

fn is_vl(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'à' | 'â' | 'ä' | 'é' | 'è' | 'ê' | 'ë' | 'î' | 'ï' | 'ô' | 'ö' | 'ù' | 'û' | 'ü' | 'ÿ' | 'œ' | 'æ')
}
/// Letters that never take part in a vowel digraph (they start a new nucleus).
fn breaks_run(c: char) -> bool {
    matches!(c, 'ï' | 'ë' | 'ü' | 'ÿ' | 'ä' | 'ö')
}
fn is_front(c: char) -> bool {
    matches!(c, 'e' | 'i' | 'y' | 'é' | 'è' | 'ê' | 'ë' | 'î' | 'ï' | 'ÿ' | 'æ')
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stress {
    Primary,
    Secondary,
    None,
}

/// Phonemize one lower-case token (may contain apostrophes and hyphens).
pub fn word(lw: &str) -> String {
    let w: String = lw.chars().filter(|c| c.is_alphabetic() || *c == '\'' || *c == '-' || *c == '’').map(|c| if c == '’' { '\'' } else { c }).collect();
    let w = w.trim_matches(|c| c == '\'' || c == '-').to_string();
    if !w.chars().any(|c| c.is_alphabetic()) {
        return String::new();
    }
    if let Some(p) = lex().get(w.as_str()) {
        return apply_stress(p, stress_class(&w));
    }
    if w.contains('-') {
        return hyphenated(&w);
    }
    if let Some((a, b)) = w.split_once('\'') {
        return apostrophe(a, b);
    }
    base(&w, stress_class(&w))
}

fn stress_class(w: &str) -> Stress {
    if unstressed().contains(w) {
        if secondary().contains(w) {
            Stress::Secondary
        } else {
            Stress::None
        }
    } else {
        Stress::Primary
    }
}

/// Apply the stress mark to a stress-free phone string: before the last nucleus.
fn apply_stress(p: &str, st: Stress) -> String {
    let mark = match st {
        Stress::Primary => "ˈ",
        Stress::Secondary => "ˌ",
        Stress::None => return p.to_string(),
    };
    let chars: Vec<char> = p.chars().collect();
    let mut last = None;
    for (i, &c) in chars.iter().enumerate() {
        let nucleus = match c {
            'a' | 'e' | 'ɛ' | 'i' | 'o' | 'ɔ' | 'u' | 'ø' | 'œ' | 'ə' | 'ɑ' => true,
            'y' => !chars.get(i + 1).map_or(false, |&n| matches!(n, 'a' | 'e' | 'ɛ' | 'i' | 'o' | 'ɔ' | 'u' | 'ø' | 'œ' | 'ɑ')),
            _ => false,
        };
        if nucleus {
            last = Some(i);
        }
    }
    let Some(mut ix) = last else { return p.to_string() };
    if chars[ix] == 'ə' && ix + 1 == chars.len() {
        if let Some(prev) = chars[..ix].iter().rposition(|&c| matches!(c, 'a' | 'e' | 'ɛ' | 'i' | 'o' | 'ɔ' | 'u' | 'ø' | 'œ' | 'ɑ' | 'y')) {
            ix = prev;
        }
    }
    let mut out = String::with_capacity(p.len() + 2);
    for (i, &c) in chars.iter().enumerate() {
        if i == ix {
            out.push_str(mark);
        }
        out.push(c);
    }
    out
}

const NUM_WORDS: &[&str] = &[
    "zéro", "un", "une", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze", "douze",
    "treize", "quatorze", "quinze", "seize", "vingt", "vingts", "trente", "quarante", "cinquante", "soixante",
    "cent", "cents", "mille", "million", "millions", "milliard", "milliards", "et",
];

fn hyphenated(w: &str) -> String {
    let parts: Vec<&str> = w.split('-').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2 && parts.iter().all(|p| NUM_WORDS.contains(p)) {
        return number_compound(&parts);
    }
    let mut out = String::new();
    let n = parts.len();
    for (k, p) in parts.iter().enumerate() {
        let ph = match *p {
            "t" if k > 0 => "t".to_string(),
            "ce" if k > 0 => "s".to_string(),
            "ci" | "là" => apply_stress(if *p == "ci" { "si" } else { "la" }, Stress::Primary),
            _ => {
                let st = if k + 1 == n || !unstressed().contains(p) { Stress::Primary } else { stress_class(p) };
                if let Some((a, b)) = p.split_once('\'') {
                    apostrophe(a, b)
                } else {
                    base(p, st)
                }
            }
        };
        out.push_str(&ph);
        // liaison inside the compound (peut-être, grand-oncle)
        if k + 1 < n {
            let next = parts[k + 1];
            let next_ph = base(next, Stress::None);
            if starts_with_vowel_phone(&next_ph) && !h_aspire(next) {
                if let Some(cons) = liaison(p) {
                    out.push_str(cons);
                }
            }
        }
    }
    out
}

/// Pronounce a hyphenated number (`vingt-et-un`, `dix-huit`, `quatre-vingt-dix-neuf`).
fn number_compound(parts: &[&str]) -> String {
    let mut out = String::new();
    let n = parts.len();
    for (k, p) in parts.iter().enumerate() {
        let next = parts.get(k + 1).copied().unwrap_or("");
        let prev = if k > 0 { parts[k - 1] } else { "" };
        let next_vowel = matches!(next, "un" | "une" | "onze" | "et" | "huit");
        let next_cons = !next.is_empty() && !next_vowel;
        let ph: String = match *p {
            "zéro" => "zeʁo".into(),
            "un" => "œ̃".into(),
            "une" => "yn".into(),
            "deux" => if next_vowel { "døz" } else { "dø" }.into(),
            "trois" => if next_vowel { "tʁwaz" } else { "tʁwa" }.into(),
            "quatre" => if next.is_empty() { "katʁ" } else { "katʁə" }.into(),
            "cinq" => "sɛ̃k".into(),
            "six" => if next_vowel { "siz" } else if next_cons { "si" } else { "sis" }.into(),
            "sept" => "sɛt".into(),
            "huit" => if next_cons { "yi" } else { "yit" }.into(),
            "neuf" => "nœf".into(),
            "dix" => if next == "sept" { "dis" } else if next_vowel || next == "neuf" { "diz" } else if next_cons { "di" } else { "dis" }.into(),
            "onze" => "ɔ̃z".into(),
            "douze" => "duz".into(),
            "treize" => "tʁɛz".into(),
            "quatorze" => "katɔʁz".into(),
            "quinze" => "kɛ̃z".into(),
            "seize" => "sɛz".into(),
            "vingt" | "vingts" => {
                let t = prev != "quatre" && matches!(next, "et" | "deux" | "trois" | "quatre" | "cinq" | "six" | "sept" | "huit" | "neuf");
                if t { "vɛ̃t" } else { "vɛ̃" }.into()
            }
            "trente" => "tʁɑ̃t".into(),
            "quarante" => "kaʁɑ̃t".into(),
            "cinquante" => "sɛ̃kɑ̃t".into(),
            "soixante" => "swasɑ̃t".into(),
            "cent" | "cents" => "sɑ̃ ".into(),
            "mille" => if k > 0 { " mil " } else { "mil " }.into(),
            "million" | "millions" => "miljɔ̃ ".into(),
            "milliard" | "milliards" => "miljaʁ ".into(),
            "et" => "e".into(),
            _ => String::new(),
        };
        if k + 1 == n {
            out.push_str(&apply_stress(ph.trim_end(), Stress::Primary));
        } else {
            out.push_str(&ph);
        }
    }
    out
}

fn apostrophe(a: &str, b: &str) -> String {
    let pre = match a {
        "l" => "l",
        "d" => "d",
        "j" => "ʒ",
        "n" => "n",
        "m" => "m",
        "t" => "t",
        "s" => "s",
        "c" | "ç" => "s",
        "qu" => "k",
        "jusqu" => "ʒysk",
        "lorsqu" => "lɔʁsk",
        "puisqu" => "pyisk",
        "quoiqu" => "kwak",
        "presqu" => "pʁɛsk",
        "quelqu" => "kɛlk",
        "entr" => "ɑ̃tʁ",
        "p" => "p",
        "aujourd" => "oʒuʁd",
        _ => "",
    };
    if pre.is_empty() {
        // unknown apostrophe: treat as a word boundary without a pause
        return format!("{}{}", base(a, Stress::None), word(b));
    }
    format!("{pre}{}", word(b))
}

// ---------------------------------------------------------------------------
// the rule engine

#[derive(Clone, Copy)]
struct Seg {
    ph: &'static str,
    /// vowel nucleus
    v: bool,
    /// unstable e (e-caduc candidate)
    schwa: bool,
}
fn seg(ph: &'static str) -> Seg {
    Seg { ph, v: false, schwa: false }
}
fn vow(ph: &'static str) -> Seg {
    Seg { ph, v: true, schwa: false }
}

/// Rules-only pronunciation of a plain word (letters only).
fn base(w: &str, st: Stress) -> String {
    if let Some(p) = lex().get(w) {
        return apply_stress(p, st);
    }
    let mut segs = rules(w);
    let mut s = String::with_capacity(w.len() * 2);
    if let Some(last_v) = segs.iter().rposition(|g| g.v) {
        for g in segs[..last_v].iter_mut() {
            g.ph = match g.ph {
                "aː" => "a",
                "oː" => "o",
                "yː" => "y",
                p => p,
            };
        }
    }
    let nuclei: Vec<usize> = segs.iter().enumerate().filter(|(_, g)| g.v).map(|(i, _)| i).collect();
    let last = nuclei.last().copied();
    for (i, g) in segs.iter().enumerate() {
        if Some(i) == last {
            s.push_str(match st {
                Stress::Primary => "ˈ",
                Stress::Secondary => "ˌ",
                Stress::None => "",
            });
        }
        s.push_str(g.ph);
    }
    s
}

/// Orthographic normalizations that make the scan simpler.
fn rewrite(w: &str) -> String {
    let mut s = w.to_string();
    for (a, b) in [
        ("oeu", "œu"), ("oei", "œi"), ("æ", "é"), ("ph", "f"), ("th", "\u{E002}"), ("ck", "k"), ("cqu", "qu"), ("rh", "r"),
        ("kh", "k"), ("gh", "g"), ("sch", "sh"), ("sseoi", "ssoi"), ("paysa", "péisa"), ("payse", "péise"), ("second", "segond"), ("damn", "dan"), ("automn", "auton"), ("guë", "gü"), ("güe", "gü"), ("oê", "oi"),
        ("ress", "rəss"), ("tch", "\u{E001}"),
    ] {
        if s.contains(a) {
            // "ress" only at the start of the word (ressembler, ressource)
            if a == "ress" {
                if s.starts_with("ress") {
                    s = format!("rəss{}", &s[4..]);
                }
                continue;
            }
            s = s.replace(a, b);
        }
    }
    if s.starts_with("seoi") {
        s = format!("soi{}", &s[4..]);
    }
    if s.ends_with("cemment") {
        s.truncate(s.len() - 7);
        s.push_str("çamment");
    } else if s.ends_with("gemment") {
        s.truncate(s.len() - 7);
        s.push_str("jamment");
    } else if s.ends_with("emment") {
        s.truncate(s.len() - 6);
        s.push_str("amment");
    }
    if s.starts_with("emm") && !s.starts_with("emmental") {
        s = format!("{NAS_A}m{}", &s[3..]);
    } else if s.starts_with("enn") && s.starts_with("ennui") || s.starts_with("ennob") {
        s = format!("{NAS_A}n{}", &s[3..]);
    }
    s
}

/// How many trailing letters are silent (final consonants, `-ent` of verbs...).
fn silent_tail(c: &[char]) -> usize {
    let n = c.len();
    if n < 2 {
        return 0;
    }
    let s: String = c.iter().collect();
    let last = c[n - 1];
    let prev = c[n - 2];
    let ends = |x: &str| s.ends_with(x);
    let vowels = c.iter().filter(|&&x| is_vl(x)).count();
    match last {
        's' => {
            if prev == 's' {
                return 0;
            }
            if ends("ès") {
                return 1;
            }
            if n >= 3 && c[n - 2] == 'e' && !is_vl(c[n - 3]) && c[n - 3] != 'ə' {
                // -es: silent e (the e rule handles it) + silent s; -ées, -ies etc. too
                return 1;
            }
            if ends("ents") {
                return 2;
            }
            if ends("rps") {
                return 2;
            }
            if ends("vers") {
                return 1;
            }
            if ends("virus") {
                return 0;
            }
            let base = &c[..n - 1];
            if is_vl(c[n - 2]) {
                return 1;
            }
            1 + silent_tail(base)
        }
        'x' => {
            if is_vl(prev) && !matches!(prev, 'e' | 'a' | 'o' | 'y') {
                return 1;
            }
            if ends("aix") || ends("eux") || ends("aux") || ends("eaux") || ends("oux") || ends("oix") {
                return 1;
            }
            if prev == 'e' || prev == 'a' || prev == 'o' || prev == 'y' {
                return 0;
            }
            0
        }
        't' => {
            if ends("ent") {
                let stem = &s[..s.len() - 3];
                if vowels <= 1 {
                    return 1;
                }
                return if ent_is_verb(&s, stem) { 2 } else { 1 };
            }
            if ends("ct") {
                return if ends("spect") || ends("nct") { 2 } else { 0 };
            }
            if ends("st") || ends("pt") || ends("lt") || ends("kt") {
                if ends("mpt") {
                    return 2;
                }
                if ends("ault") || ends("oult") {
                    return 2;
                }
                return 0;
            }
            1
        }
        'd' => {
            if ends("nd") || ends("rd") || is_vl(prev) {
                return 1;
            }
            1
        }
        'p' => {
            if prev == 'm' {
                return 1;
            }
            if ends("oup") || ends("rop") || ends("rop") || ends("lop") || ends("sirop") {
                return 1;
            }
            if ends("amp") || ends("emp") {
                return 1;
            }
            0
        }
        'g' => {
            if ends("ing") && n >= 4 && !is_vl(c[n - 4]) {
                return 0;
            }
            if prev == 'n' || ends("bourg") || ends("rg") && ends("bourg") {
                return 1;
            }
            0
        }
        'c' => {
            if prev == 'n' {
                return 1;
            }
            0
        }
        'r' => {
            if ends("er") && vowels >= 2 {
                return 1;
            }
            0
        }
        'z' => {
            if ends("ez") || ends("iz") {
                return 1;
            }
            0
        }
        'b' => {
            if prev == 'm' {
                1
            } else {
                0
            }
        }
        'h' => {
            if matches!(prev, 'c' | 's' | 't' | 'p') {
                0
            } else {
                1
            }
        }
        _ => 0,
    }
}

/// Is a word in `-ent` a third-person-plural verb form (silent ending)?
fn ent_is_verb(w: &str, stem: &str) -> bool {
    const NOUNS: &[&str] = &[
        "moment", "comment", "argent", "agent", "urgent", "client", "orient", "parent", "apparent", "transparent",
        "serment", "sarment", "présent", "absent", "omniprésent", "content", "mécontent", "patent", "latent",
        "compétent", "incompétent", "intermittent", "omnipotent", "impotent", "ardent", "serpent", "arpent",
        "vent", "souvent", "couvent", "fervent", "auvent", "lent", "violent", "excellent", "talent", "équivalent",
        "insolent", "succulent", "turbulent", "somnolent", "corpulent", "truculent", "purulent", "virulent",
        "opulent", "indolent", "ambivalent", "prévalent", "polyvalent", "bivalent", "univalent", "relent",
        "dolent", "féculent", "flatulent", "fraudulent", "éminent", "imminent", "pertinent", "impertinent",
        "continent", "proéminent", "permanent", "immanent", "rémanent", "fréquent", "éloquent", "conséquent",
        "subséquent", "grandiloquent", "onguent", "adjacent", "innocent", "décent", "indécent", "récent",
        "adolescent", "accent", "concurrent", "récurrent", "torrent", "laurent", "tangent", "contingent",
        "intelligent", "diligent", "négligent", "indigent", "indulgent", "détergent", "astringent", "sergent",
        "régent", "gent", "divergent", "convergent", "insurgent", "prudent", "imprudent", "décadent", "occident",
        "accident", "incident", "président", "résident", "évident", "dissident", "confident", "trident", "strident",
        "précédent", "antécédent", "excédent", "coïncident", "dent", "cent", "sent", "ment", "vincent", "consent",
        "ressent", "pressent", "tourment", "paravent", "auvent", "ingrédient",
        "expédient", "obédient", "récipient", "inconvénient", "coefficient", "conscient", "efficient", "quotient",
        "patient", "impatient", "omniscient", "déficient", "émollient", "escient", "gradient", "dividende",
        "ponent", "opposant", "spent", "trident", "ciment", "piment", "aliment", "segment", "fragment", "pigment",
        "firmament", "testament", "médicament", "parlement", "ornement", "tempérament", "armement", "élément",
        "sentiment", "compliment", "document", "argument", "monument", "instrument", "vêtement", "mouvement",
        "bâtiment", "jugement", "régiment", "sédiment", "supplément", "complément", "détriment", "ferment",
        "serment", "dément", "clément", "inclément", "véhément", "abstinent", "incontinent", "rent",
    ];
    const VERBS: &[&str] = &[
        "aiment", "estiment", "expriment", "suppriment", "animent", "raniment", "riment", "abîment", "déciment",
        "réclament", "proclament", "acclament", "déclament", "entament", "affament", "diffament", "assument",
        "résument", "fument", "allument", "consument", "parfument", "écument", "plument", "hument", "sèment",
        "nomment", "consomment", "dénomment", "renomment", "assomment", "étudient", "remédient", "répudient",
        "apprécient", "déprécient", "remercient", "associent", "négocient", "dissocient", "copient", "expient",
        "envient", "dévient", "oublient", "publient", "multiplient", "plient", "replient", "supplient", "lient",
        "relient", "allient", "rallient", "humilient", "concilient", "réconcilient", "crient", "écrient", "prient",
        "rient", "sourient", "varient", "marient", "parient", "trient", "charient", "communient", "rassasient",
        "décident", "aident", "guident", "plaident", "vident", "élucident", "intimident", "liquident",
        "consolident", "valident", "invalident", "lapident", "dilapident", "oxydent", "éludent", "dénudent",
        "extrudent", "reculent", "calculent", "circulent", "accumulent", "articulent", "annulent", "basculent",
        "coagulent", "simulent", "stimulent", "formulent", "manipulent", "spéculent", "brûlent", "hurlent",
        "appellent", "rappellent", "excellent", "nivellent", "planent", "émanent", "ricanent", "déterminent",
        "imaginent", "dominent", "terminent", "examinent", "combinent", "dessinent", "exigent", "dirigent",
        "corrigent", "obligent", "rédigent", "affligent", "fustigent", "infligent", "voltigent", "émergent",
        "nagent", "partagent", "engagent", "voyagent", "dégagent", "encouragent", "ménagent", "déménagent",
        "commencent", "lancent", "avancent", "annoncent", "prononcent", "renoncent", "dénoncent", "influencent",
        "financent", "placent", "effacent", "remplacent", "menacent", "tracent", "déplacent", "exercent",
        "forcent", "percent", "amorcent", "divorcent", "coincent", "pincent", "grincent", "rincent", "bercent",
        "forment", "dorment", "ferment", "affirment", "enferment", "confirment", "alarment", "calment", "filment",
        "infirment", "informent", "transforment", "réforment", "déforment", "conforment", "charment", "arment",
        "désarment", "endorment", "réaffirment", "mentent", "sentent", "ressentent", "pressentent", "consentent",
        "présentent", "représentent", "inventent", "absentent", "patientent", "contentent", "mécontentent",
        "orientent", "désorientent", "argumentent", "documentent", "complimentent", "alimentent", "cimentent",
        "fermentent", "augmentent", "commentent", "fragmentent", "lamentent", "tourmentent", "régentent",
        "serpentent", "arpentent", "ventent", "éventent", "inventent", "chantent", "plantent", "tentent",
        "vantent", "rentent", "parent", "préparent", "séparent", "comparent", "réparent", "déclarent",
        "accaparent", "emparent", "garent", "égarent", "tarent", "effarent", "comparent", "disparent",
    ];
    if NOUNS.contains(&w) {
        return false;
    }
    if VERBS.contains(&w) {
        return true;
    }
    let sc: Vec<char> = stem.chars().collect();
    let k = sc.len();
    if k == 0 {
        return false;
    }
    let last = sc[k - 1];
    let prev = if k >= 2 { sc[k - 2] } else { '#' };
    let ends = |x: &str| stem.ends_with(x);
    match last {
        'm' => {
            if ends("mm") {
                return false; // -amment / -emment adverbs
            }
            if matches!(prev, 'r' | 'l') {
                return true;
            }
            if is_vl(prev) {
                return false; // -Vment nouns and adverbs
            }
            false
        }
        'i' => {
            if matches!(prev, 'a' | 'o' | 'u' | 'e' | 'ï') {
                return true; // -aient, -oient, -uient
            }
            // -ient: nouns unless a known -ier verb stem
            ends("fi") || ends("pli") || ends("bli") || ends("cri") || ends("pri") || ends("tri") || ends("vari")
                || ends("mari") || ends("pari") || ends("tudi") || ends("rci") || ends("éci") || ends("oci") || ends("ici")
        }
        'a' | 'o' | 'u' | 'é' | 'e' | 'y' | 'ï' | 'ë' | 'ô' | 'û' => true,
        'c' => ends("nc") || ends("rc") || ends("ac"),
        'g' => !ends("ng") || true,
        't' => true,
        's' => true,
        'd' => {
            if ends("nd") || ends("rd") || ends("od") || ends("ad") || ends("èd") || ends("oud") || ends("aud") {
                return true;
            }
            if ends("id") || ends("ud") || ends("éd") {
                return false;
            }
            true
        }
        'l' => {
            if ends("ul") && !ends("oul") && !ends("eul") {
                return false;
            }
            true
        }
        'n' => true,
        'r' => {
            if ends("ér") || ends("rr") {
                return false;
            }
            true
        }
        'p' | 'b' | 'f' | 'v' | 'h' | 'x' | 'z' | 'ç' => true,
        _ => true,
    }
}

/// The core letter scan.
fn rules(w: &str) -> Vec<Seg> {
    let rw = rewrite(w);
    let c: Vec<char> = rw.chars().collect();
    let n = c.len();
    let tail = silent_tail(&c);
    let m = n - tail; // pronounced region
    // the last pronounced letter is a silent e? (for syllable structure)
    let total_vowels = c[..m].iter().filter(|&&x| is_vl(x) || x == NAS_A || x == 'ə').count();
    let final_e_silent = m >= 2 && c[m - 1] == 'e' && (!is_vl(c[m - 2]) || (c[m - 2] == 'u' && m >= 3 && matches!(c[m - 3], 'q' | 'g'))) && total_vowels >= 2 && (tail == 0 || matches!(c[m], 's' | 'n'));
    let me = if final_e_silent { m - 1 } else { m };
    let at = |i: usize| -> char { if i < n { c[i] } else { '#' } };
    let in_region = |i: usize| i < m;
    let tail_first = at(m);
    // onset cluster (obstruent + liquid)
    let onset = |a: char, b: char| matches!(a, 'p' | 'b' | 't' | 'd' | 'k' | 'g' | 'f' | 'v' | 'c') && matches!(b, 'l' | 'r');
    // length of the consonant unit starting at j (0 if none / vowel / silent h)
    let cons_len = |j: usize| -> usize {
        if j >= me {
            return 0;
        }
        let a = c[j];
        if is_vl(a) || a == 'h' || a == 'ə' || a == NAS_A {
            return 0;
        }
        let b = at(j + 1);
        if j + 1 < me && (a == b) && !matches!(a, TCH) {
            return 2;
        }
        match (a, b) {
            ('c', 'h') | ('s', 'h') | ('g', 'n') => 2,
            ('q', 'u') => 2,
            ('g', 'u') if is_front(at(j + 2)) => 2,
            _ => 1,
        }
    };
    // is the vowel ending right before j in a closed syllable?
    let closed = |j: usize| -> bool {
        let l1 = cons_len(j);
        if l1 == 0 {
            return false;
        }
        let k = j + l1;
        if k >= me {
            return true;
        }
        let l2 = cons_len(k);
        if l2 == 0 {
            return false;
        }
        let a = c[j];
        let kk = k + l2;
        if onset(a, c[k]) && kk < me && (is_vl(c[kk]) || c[kk] == 'ə') {
            return false;
        }
        true
    };
    let vowel_after = |j: usize| -> bool {
        // a vowel letter follows j (looking through a silent h)
        let mut k = j;
        while k < m && c[k] == 'h' {
            k += 1;
        }
        k < m && (is_vl(c[k]) || c[k] == 'ə' || c[k] == NAS_A)
    };
    let mut out: Vec<Seg> = Vec::with_capacity(n + 2);
    let mut i = 0usize;
    while i < m {
        let ch = c[i];
        if ch == NAS_A {
            out.push(vow("ɑ̃"));
            i += 1;
            continue;
        }
        if ch == 'ə' {
            out.push(Seg { ph: "ə", v: true, schwa: true });
            i += 1;
            continue;
        }
        if is_vl(ch) {
            // collect the vowel run
            let start = i;
            let mut j = i;
            while j < m && is_vl(c[j]) && (j == start || !breaks_run(c[j])) {
                j += 1;
            }
            // split off a final silent e: it belongs to no run
            let run: Vec<char> = c[start..j].to_vec();
            let next_is_vowel_after_h = vowel_after(j);
            vowel_run(&run, &c, start, j, m, me, tail_first, &closed, next_is_vowel_after_h, &mut out, &mut i, w);
            continue;
        }
        let nx = at(i + 1);
        let nx2 = at(i + 2);
        let pv = if i > 0 { c[i - 1] } else { '#' };
        let prev_vowel_letter = i > 0 && (is_vl(c[i - 1]) || c[i - 1] == 'ə' || c[i - 1] == NAS_A);
        let next_front = is_front(nx) && in_region(i + 1);
        match ch {
            'b' => {
                if nx == 'b' {
                    i += 1;
                }
                let voiceless = matches!(at(i + 1), 't' | 's' | 'c' | 'p' | 'f' | 'k' | 'q') && in_region(i + 1);
                out.push(seg(if voiceless { "p" } else { "b" }));
            }
            'c' => {
                if nx == 'h' {
                    let k = ch_is_k(&c, i);
                    out.push(seg(if k { "k" } else { "ʃ" }));
                    i += 1;
                } else if nx == 'c' {
                    if is_front(nx2) {
                        out.push(seg("k"));
                        out.push(seg("s"));
                    } else {
                        out.push(seg("k"));
                    }
                    i += 1;
                } else if next_front {
                    out.push(seg("s"));
                } else if nx == 'k' {
                    // handled by rewrite; defensive
                } else {
                    out.push(seg("k"));
                }
            }
            'ç' => out.push(seg("s")),
            TCH => {
                out.push(seg("t"));
                out.push(seg("ʃ"));
            }
            'd' => {
                if nx == 'd' {
                    i += 1;
                }
                out.push(seg("d"));
            }
            'f' => {
                if nx == 'f' {
                    i += 1;
                }
                out.push(seg("f"));
            }
            'g' => {
                if nx == 'g' {
                    if is_front(nx2) {
                        out.push(seg("ɡ"));
                        // next g becomes ʒ
                    } else {
                        i += 1;
                        out.push(seg("ɡ"));
                    }
                } else if nx == 'n' && in_region(i + 1) {
                    if gn_is_hard(&c, i) {
                        out.push(seg("ɡ"));
                    } else {
                        out.push(seg("ɲ"));
                        i += 1;
                    }
                } else if next_front {
                    out.push(seg("ʒ"));
                    // silent e in ge + a/o/u
                    if nx == 'e' && matches!(nx2, 'a' | 'o' | 'â' | 'ô' | NAS_A) && in_region(i + 2) {
                        i += 1;
                    }
                } else if nx == 'u' {
                    let gueil = nx2 == 'e' && at(i + 3) == 'i' && at(i + 4) == 'l';
                    let guy = nx2 == 'y' && is_vl(at(i + 3));
                    if matches!(nx2, 'e' | 'i' | 'y' | 'é' | 'è' | 'ê' | 'a' | 'o' | 'â' | 'ô' | NAS_A) && in_region(i + 2) && !gueil && !guy && !w.contains("lingui") {
                        out.push(seg("ɡ"));
                        i += 1; // silent u
                    } else {
                        out.push(seg("ɡ"));
                    }
                } else if nx == 't' {
                    // doigt, longtemps: silent g
                } else {
                    out.push(seg("ɡ"));
                }
            }
            'h' => {}
            'j' => out.push(seg("ʒ")),
            'k' => {
                if nx == 'k' {
                    i += 1;
                }
                out.push(seg("k"));
            }
            'l' => {
                if nx == 'l' {
                    i += 1;
                }
                out.push(seg("l"));
            }
            'm' => {
                if nx == 'm' {
                    i += 1;
                }
                out.push(seg("m"));
            }
            'n' => {
                if nx == 'n' {
                    i += 1;
                }
                if nx == 'g' && i + 2 == m && pv == 'i' && i >= 2 && !is_vl(c[i - 2]) {
                    out.push(seg("ŋ"));
                    i += 1;
                } else {
                    out.push(seg("n"));
                }
            }
            'p' => {
                if nx == 'p' {
                    i += 1;
                }
                if pv == 'm' && nx == 't' && (w.contains("compt") || w.contains("dompt") || w.contains("prompt") || (w.starts_with("exempt") && !w.contains("exempti"))) {
                    // compter, prompt: silent p
                } else if pv == 'a' && nx == 't' && w.starts_with("bapt") {
                    // baptême
                } else if pv == 'l' && nx == 't' && w.starts_with("sculpt") {
                    // sculpter
                } else {
                    out.push(seg("p"));
                }
            }
            'q' => {
                out.push(seg("k"));
                if nx == 'u' && in_region(i + 1) {
                    if kw_qua(&c, i) {
                        out.push(seg("w"));
                    }
                    i += 1;
                }
            }
            'r' => {
                if nx == 'r' {
                    i += 1;
                }
                out.push(seg("ʁ"));
            }
            's' => {
                if nx == 's' {
                    i += 1;
                    out.push(seg("s"));
                } else if nx == 'h' && !w.starts_with("désh") {
                    i += 1;
                    out.push(seg("ʃ"));
                } else if nx == 'c' && is_front(nx2) {
                    i += 1;
                    out.push(seg("s"));
                } else if prev_vowel_letter && (vowel_after(i + 1) || nx == 'y' || nx == 'ə') && in_region(i + 1) {
                    out.push(seg(if s_stays_voiceless(&c, i) { "s" } else { "z" }));
                } else if pv == 'n' && i >= 4 && c[i - 4..i] == ['t', 'r', 'a', 'n'] && vowel_after(i + 1) && !(nx == 'e' && i + 2 >= m) {
                    out.push(seg("z"));
                } else {
                    out.push(seg("s"));
                }
            }
            't' => {
                if nx == 't' {
                    i += 1;
                }
                if nx == 'i' && ti_is_s(&c, i, m) {
                    out.push(seg("s"));
                } else if nx == 'z' {
                    out.push(seg("t"));
                    out.push(seg("s"));
                    i += 1;
                } else {
                    out.push(seg("t"));
                }
            }
            TH => out.push(seg("t")),
            'v' => out.push(seg("v")),
            'w' => out.push(seg("w")),
            'x' => {
                if i == 0 {
                    out.push(seg(if matches!(nx, 'i' | 'y') { "k" } else { "ɡ" }));
                    out.push(seg(if matches!(nx, 'i' | 'y') { "s" } else { "z" }));
                } else if pv == 'e' && (i == 1 || (i == 3 && (w.starts_with("ine") || w.starts_with("coe") || w.starts_with("pre") || w.starts_with("rée")) ) || (i == 2 && w.starts_with("he"))) && vowel_after(i + 1) {
                    out.push(seg("ɡ"));
                    out.push(seg("z"));
                } else if nx == 'c' && is_front(nx2) {
                    out.push(seg("k"));
                } else if nx == 'i' && w.contains("xième") {
                    out.push(seg("z"));
                } else if w.starts_with("soix") && i == 3 {
                    out.push(seg("s"));
                } else {
                    out.push(seg("k"));
                    out.push(seg("s"));
                }
            }
            'z' => {
                if nx == 'z' {
                    i += 1;
                }
                out.push(seg("z"));
            }
            'ñ' => out.push(seg("ɲ")),
            _ => {}
        }
        i += 1;
    }
    // a word with no nucleus at all but a final silent e ("le", "que"): ə
    if !out.iter().any(|g| g.v) && m >= 1 && c[m - 1] == 'e' {
        out.push(vow("ə"));
    }
    finish(out)
}

/// Does `ch` read /k/ here (Greek words)?
fn ch_is_k(c: &[char], i: usize) -> bool {
    let rest: String = c[i..].iter().collect();
    let before: String = c[..i].iter().collect();
    let nx2 = c.get(i + 2).copied().unwrap_or('#');
    if matches!(nx2, 'r' | 'l' | 'n' | 't') {
        return true;
    }
    for p in ["chao", "chor", "chœ", "chlo", "chrom", "chron", "chir", "chim"] {
        if rest.starts_with(p) && !rest.starts_with("chose") && !rest.starts_with("choix") && !rest.starts_with("chou") && !rest.starts_with("chois") && !rest.starts_with("chirurg") && !rest.starts_with("chimi") && !rest.starts_with("chimère") {
            return p != "chir" && p != "chim";
        }
    }
    if before.ends_with("te") && (rest.starts_with("chn") || rest == "ch") {
        return true;
    }
    if before.ends_with("psy") || before == "ar" && (rest.starts_with("ché") || rest.starts_with("cha")) && !rest.starts_with("chal") && !rest.starts_with("chan") && !rest.starts_with("char") {
        return true;
    }
    if before == "é" && rest.starts_with("cho") && !rest.starts_with("chou") || before.ends_with("or") && rest.starts_with("ch") && (rest.starts_with("che") && before == "or") {
        return true;
    }
    false
}

fn gn_is_hard(c: &[char], i: usize) -> bool {
    let rest: String = c[i..].iter().collect();
    let before: String = c[..i].iter().collect();
    if i == 0 {
        return true;
    }
    if before == "i" && rest.starts_with("gn") {
        return true;
    }
    for (b, r) in [("dia", "gn"), ("a", "gnos"), ("co", "gniti"), ("co", "gnition"), ("sta", "gn"), ("ma", "gnat"), ("ma", "gnum"),  ("pro", "gnath"), ("pu", "gna"), ("inexpu", "gn"), ("", "gnom"), ("", "gnos"), ("", "gnou")] {
        if (b.is_empty() || before.ends_with(b)) && rest.starts_with(r) && !(r == "gnos" && b.is_empty()) {
            return true;
        }
    }
    false
}

fn kw_qua(c: &[char], i: usize) -> bool {
    let rest: String = c[i..].iter().collect();
    let before: String = c[..i].iter().collect();
    rest.starts_with("qua")
        && (before == "a" || before == "é" || before == "s" || before == "adé" || rest.starts_with("quadr") || rest.starts_with("quartz"))
}

/// `ti` + vowel read as /sj/ (nation, patient, initial, essentiel, ambitieux)?
fn ti_is_s(c: &[char], i: usize, m: usize) -> bool {
    let rest: String = c[i..m].iter().collect();
    let before: String = c[..i].iter().collect();
    if before.ends_with('s') || before.ends_with('x') {
        return false;
    }
    if rest == "tion" && before.ends_with('é') && m < c.len() && c[m] == 's' {
        return false;
    }
    if rest.starts_with("tio") && !rest.starts_with("tiot") {
        return true;
    }
    if rest.starts_with("tia") && !rest.starts_with("tiar") && !rest.starts_with("tiag") {
        return true;
    }
    if rest.starts_with("tium") {
        return true;
    }
    if rest.starts_with("tiel") || rest.starts_with("tieu") || rest.starts_with("tienc") || rest.starts_with("tiens") && false {
        return true;
    }
    let word_final = m == c.len() || c[m] == 's';
    if rest == "tie" && word_final && before.chars().last().map_or(false, |l| is_vl(l) && l != 'â') {
        return true;
    }
    if rest == "tie" && !word_final {
        return before.ends_with("ini");
    }
    if rest == "tie" && (before.ends_with("cra") || before.ends_with("ma") || before.ends_with("éti") || before.ends_with("inu") || before.ends_with("iner") || before.ends_with("ine") || before.ends_with("ophé") || before.ends_with("péripé") || before.ends_with("facé") || before.ends_with("supréma") || before.ends_with("prima") || before.ends_with("iner") || before.ends_with("inep")) {
        return true;
    }
    if rest == "tien" && (before.ends_with('a') || before.ends_with('i') || before.ends_with('y')) {
        return true;
    }
    if rest.starts_with("tient") && rest.len() <= 6 && is_vl(c[i - 1]) && !matches!(c[i - 1], 'e' | 'é' | 'ə') {
        return true;
    }
    false
}

fn s_stays_voiceless(c: &[char], i: usize) -> bool {
    let before: String = c[..i].iter().collect();
    for p in ["anti", "mono", "poly", "foto", "micro", "contre", "entre", "tourne", "vrai", "pré", "a", "re", "auto", "télé", "co", "bio", "éco", "multi", "ultra", "hyper", "super", "homo", "hétéro", "pseudo", "proto", "mi", "di"] {
        if (before == p || (p == "vrai" && before == "invrai")) && !matches!(p, "a" | "re" | "co" | "mi" | "di" | "pré") {
            return true;
        }
    }
    if before == "a" && (c[i + 1] == 'y' || c[i + 1..].starts_with(&['o', 'c']) || c[i + 1..].starts_with(&['e', 'p']) || c[i + 1..].starts_with(&['e', 'x'])) {
        return true;
    }
    if before == "pré" && (c[i + 1..].starts_with(&['u', 'p']) || c[i + 1..].starts_with(&['é', 'a'])) {
        return true;
    }
    if before == "re" && (c[i + 1..].starts_with(&['a', 'l']) || c[i + 1..].starts_with(&['u', 'r']) || c[i + 1..].starts_with(&['o', 'u'])) {
        return true;
    }
    false
}

/// Pronounce a vowel run `run` (letters c[start..j]) and advance `i` past it
/// (and past a consumed `ll`/`l` or nasal consonant). All vowel quality
/// decisions live here.
#[allow(clippy::too_many_arguments)]
fn vowel_run(
    run_in: &[char],
    c: &[char],
    start: usize,
    j: usize,
    m: usize,
    me: usize,
    tail_first: char,
    closed: &dyn Fn(usize) -> bool,
    vowel_follows: bool,
    out: &mut Vec<Seg>,
    i: &mut usize,
    w: &str,
) {
    let n = c.len();
    let at = |k: usize| -> char { if k < n { c[k] } else { '#' } };
    let prev = if start > 0 { c[start - 1] } else { '#' };
    let prev2 = if start > 1 { c[start - 2] } else { '#' };
    // after an obstruent+liquid cluster, i/u stay vowels (client, cruel)
    let after_cluster = matches!(prev, 'l' | 'r') && matches!(prev2, 'p' | 'b' | 't' | 'd' | 'k' | 'g' | 'f' | 'v' | 'c');
    let word_initial = start == 0;
    let push_v = |out: &mut Vec<Seg>, p: &'static str| out.push(vow(p));

    // ---- ill / il: a yod closes the run (travail, soleil, fille, feuille, accueil)
    let mut run: Vec<char> = run_in.to_vec();
    let mut yod_l = 0usize;
    if j < me && c[j] == 'l' && run.last() == Some(&'i') {
        let ll = j + 1 < me && c[j + 1] == 'l';
        let l_final = j + 1 >= m;
        let stem_ok = run.len() >= 2 && !(run.len() == 2 && matches!(run[0], 'u' | 'o'));
        if ll || (l_final && stem_ok) {
            if run.len() == 1 && ill_is_l(w, start) {
                // ville, mille, illusion: plain l
            } else {
                yod_l = if ll { 2 } else { 1 };
            }
        }
    }
    if yod_l > 0 {
        if run.len() == 1 {
            push_v(out, "i");
        } else {
            run.pop();
            let stem: String = run.iter().collect();
            match stem.as_str() {
                "a" | "ai" => push_v(out, "a"),
                "e" | "ei" => push_v(out, "ɛ"),
                "eu" | "œu" | "ue" | "œ" | "oeu" => push_v(out, "œ"),
                "ou" => push_v(out, "u"),
                "u" => {
                    out.push(seg("y"));
                    push_v(out, "i");
                }
                "ie" | "iei" => {
                    out.push(seg("j"));
                    push_v(out, "ɛ");
                }
                "oa" => {
                    push_v(out, "ɔ");
                    push_v(out, "a");
                }
                _ => {
                    // rare: pronounce the stem then i
                    let mut dummy = start;
                    vowel_run(&run, c, start, j, m, me, tail_first, closed, false, out, &mut dummy, w);
                    push_v(out, "i");
                }
            }
        }
        out.push(seg("j"));
        *i = j + yod_l;
        return;
    }

    // nasal consonant after the run?
    let nasal_after = {
        let k = j;
        if k < m && matches!(c[k], 'n' | 'm') {
            let a = at(k + 1);
            if k + 1 >= m {
                true
            } else if a == c[k] || (c[k] == 'm' && a == 'n') {
                false
            } else if a == 'h' {
                !(k + 2 < m && is_vl(at(k + 2)))
            } else if is_vl(a) || a == 'ə' {
                false
            } else if a == 'g' && k + 2 == m && run.last() == Some(&'i') && start > 0 && !is_vl(c[start - 1]) {
                false // -ing
            } else {
                true
            }
        } else {
            false
        }
    };
    let nasal_letter = if nasal_after { c[j] } else { '#' };
    let at_end = j >= m; // the run reaches the end of the pronounced region
    let no_tail = j >= n; // nothing at all follows (not even silent letters)
    let end_after_nasal = nasal_after && j + 1 >= m;
    let closed_after = closed(j);
    let r = run.len();
    let mut k = 0usize;
    // the previous element emitted a glide (j/w/ɥ): the next e is a real vowel
    let mut prev_glide = false;
    // the previous element was i/y/é: a following en is ɛ̃
    let mut prev_front = false;
    let nvowels = w.chars().filter(|&x| is_vl(x)).count();
    // is a run-final e (at run index kpos) silent?  joue/jouent/licenciement yes; jouer/pierre no
    let e_silent = |kpos: usize| -> bool {
        kpos + 1 == r
            && run[kpos] == 'e'
            && ((at_end && !matches!(tail_first, 'r' | 'z' | 'd' | 't'))
                || (!at_end && !nasal_after && !closed_after && j < me && !(j + 1 < m && c[j + 1] == c[j])))
    };
    while k < r {
        let a = run[k];
        let b = if k + 1 < r { run[k + 1] } else { '#' };
        let c2 = if k + 2 < r { run[k + 2] } else { '#' };
        let last_elem = |k: usize, len: usize| k + len >= r;
        let more_after = |k: usize, len: usize| k + len < r;
        // ---- three-letter
        if a == 'e' && b == 'a' && c2 == 'u' {
            push_v(out, "o");
            prev_glide = false;
            k += 3;
            continue;
        }
        if a == 'a' && b == 'o' && (c2 == 'u' || c2 == 'û') {
            push_v(out, "u");
            prev_glide = false;
            k += 3;
            continue;
        }
        // ---- two-letter
        let mut done = true;
        match (a, b) {
            ('a', 'u') => {
                push_v(out, if at(j) == 'r' && last_elem(k, 2) { "ɔ" } else { "o" });
                k += 2;
            }
            ('a', 'i') | ('a', 'î') | ('a', 'y') | ('e', 'i') | ('e', 'y') | ('e', 'î') => {
                let is_y = b == 'y';
                let more = more_after(k, 2);
                if last_elem(k, 2) && nasal_after {
                    push_v(out, "ɛ̃");
                    k += 2;
                    continue;
                }
                if a == 'a' && b == 'i' && prev == 'f' && at(j) == 's' && j + 1 < m && (is_vl(at(j + 1)) || at(j + 1) == 'ə') {
                    out.push(Seg { ph: "ə", v: true, schwa: false });
                    k += 2;
                    continue;
                }
                // -rai (future) and ai: e
                let noun_ai = matches!(w, "balai" | "délai" | "essai" | "minerai" | "remblai" | "étai" | "vrai" | "quai" | "mai" | "gai" | "geai" | "chai" | "lai" | "rai" | "bai" | "ai");
                let ph = if last_elem(k, 2) && no_tail && a == 'a' && !is_y && ((nvowels >= 2 && !noun_ai) || w == "ai") {
                    "e"
                } else {
                    "ɛ"
                };
                push_v(out, ph);
                if is_y && (more || vowel_follows) {
                    out.push(seg("j"));
                    prev_glide = true;
                    k += 2;
                    continue;
                }
                k += 2;
            }
            ('e', 'u') | ('e', 'û') | ('œ', 'u') => {
                if last_elem(k, 2) && nasal_after {
                    push_v(out, "œ̃");
                    k += 2;
                    continue;
                }
                let more = more_after(k, 2);
                let ph = if b == 'û' || more { "ø" } else { eu_quality(c, j, m, me, closed_after) };
                push_v(out, ph);
                k += 2;
            }
            ('o', 'i') | ('o', 'î') | ('o', 'y') => {
                let is_y = b == 'y';
                let more = more_after(k, 2);
                out.push(seg("w"));
                if last_elem(k, 2) && nasal_after && !is_y {
                    push_v(out, "ɛ̃");
                    k += 2;
                    continue;
                }
                push_v(out, "a");
                if is_y && (more || vowel_follows) {
                    out.push(seg("j"));
                    prev_glide = true;
                    k += 2;
                    continue;
                }
                k += 2;
            }
            ('o', 'u') | ('o', 'û') | ('o', 'ù') => {
                let more = more_after(k, 2);
                // ou + vowel -> w (oui, jouer), but not before a final silent e (joue, jouent)
                let next_e_silent = more && e_silent(k + 2);
                if (more && !next_e_silent) || (vowel_follows && !more && !after_cluster) {
                    if after_cluster {
                        push_v(out, "u");
                        prev_glide = true;
                        prev_front = false;
                        k += 2;
                        continue;
                    } else {
                        out.push(seg("w"));
                        prev_glide = true;
                        prev_front = false;
                        k += 2;
                        continue;
                    }
                } else {
                    push_v(out, "u");
                }
                k += 2;
            }
            ('u', 'i') | ('u', 'y') | ('û', 'i') => {
                let is_y = b == 'y';
                let more = more_after(k, 2);
                if after_cluster {
                    push_v(out, "y");
                } else {
                    out.push(seg("y"));
                }
                if last_elem(k, 2) && nasal_after {
                    push_v(out, "ɛ̃");
                    k += 2;
                    continue;
                }
                push_v(out, "i");
                if is_y && (more || vowel_follows) {
                    out.push(seg("j"));
                    prev_glide = true;
                    k += 2;
                    continue;
                }
                prev_front = true;
                prev_glide = false;
                k += 2;
                continue;
            }
            ('e', 'a') if matches!(prev, 'g' | 'j') => {
                // Jean, mangeant, geai: the e is silent
                k += 1;
            }
            ('e', 'e') => {
                push_v(out, "i");
                k += 2;
            }
            ('o', 'o') => {
                let before: String = c[..start].iter().collect();
                if (before == "c" && matches!(at(j), 'p' | 'r' | 'c')) || before == "z" {
                    push_v(out, "ɔ");
                    push_v(out, "ɔ");
                } else if before == "alc" {
                    push_v(out, "ɔ");
                } else {
                    push_v(out, "u");
                }
                k += 2;
            }
            _ => done = false,
        }
        if done {
            prev_glide = false;
            prev_front = false;
            continue;
        }
        // ---- single letters
        match a {
            'a' | 'à' | 'ä' | 'â' => {
                if last_elem(k, 1) && nasal_after {
                    push_v(out, "ɑ̃");
                } else {
                    push_v(out, if a == 'â' { "aː" } else { "a" });
                }
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            'é' => {
                push_v(out, "e");
                prev_glide = false;
                prev_front = true;
                k += 1;
            }
            'è' | 'ê' | 'ë' => {
                if last_elem(k, 1) && nasal_after {
                    push_v(out, "ɛ̃");
                } else {
                    push_v(out, "ɛ");
                }
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            'e' => {
                let is_last = last_elem(k, 1);
                let after_vowel = k > 0 && !prev_glide && !matches!(run[k - 1], 'é' | 'o');
                if is_last && nasal_after {
                    let ph = if prev_front || (k > 0 && matches!(run[k - 1], 'i' | 'y' | 'é')) {
                        // ien / yen / éen
                        if end_after_nasal {
                            // -ient: noun (client, patient) -> ɑ̃, verb (vient, tient) -> ɛ̃
                            let t_follows = j + 1 < n && c[j + 1] == 't';
                            if t_follows && !ient_is_tenir(w) { "ɑ̃" } else { "ɛ̃" }
                        } else if w.starts_with("bien") || w.starts_with("rien") || (at(j + 1) == 'd' && at(j + 2) == 'r') {
                            "ɛ̃"
                        } else {
                            "ɑ̃"
                        }
                    } else if end_after_nasal && !after_vowel && !prev_glide && nasal_letter == 'n' && !is_vl(prev) && prev != '#' && nvowels >= 2 && j + 1 >= n {
                        // -en after a consonant (abdomen, pollen): ɛn
                        push_v(out, "ɛ");
                        out.push(seg("n"));
                        *i = j + 1;
                        return;
                    } else {
                        "ɑ̃"
                    };
                    push_v(out, ph);
                    k += 1;
                    continue;
                }
                if after_vowel {
                    // e after a full vowel: silent (paiement, joue), unless it carries the
                    // final syllable before a silent r/z/d/t (créer, jouer, jouet)
                    if at_end {
                        match tail_first {
                            'r' | 'z' | 'd' => push_v(out, "e"),
                            't' => push_v(out, "ɛ"),
                            _ => {}
                        }
                    }
                    k += 1;
                    continue;
                }
                if !is_last {
                    // e before another vowel letter (eo, ea not after g): e
                    push_v(out, "e");
                    prev_glide = false;
                    prev_front = true;
                    k += 1;
                    continue;
                }
                if at_end {
                    // word-final e (possibly followed by silent letters)
                    let ph = match tail_first {
                        'r' | 'z' | 'd' => Some("e"),
                        't' | 'c' | 'p' => Some("ɛ"),
                        '#' | 's' | 'n' => None,
                        _ => Some("ɛ"),
                    };
                    match ph {
                        Some(p) => push_v(out, p),
                        None => {
                            // silent final e; monosyllables (le, que) get ə
                            if !out.iter().any(|g| g.v) {
                                out.push(Seg { ph: "ə", v: true, schwa: false });
                            }
                        }
                    }
                    k += 1;
                    continue;
                }
                let nx = c[j];
                if nx == 'x' {
                    push_v(out, "ɛ");
                } else if vowel_follows && !prev_glide {
                    // dehors
                    out.push(Seg { ph: "ə", v: true, schwa: true });
                } else if closed_after {
                    push_v(out, "ɛ");
                } else if j + 1 < me && c[j + 1] == nx {
                    // e before a double consonant + vowel: e (effet, cellule, terrain)
                    push_v(out, "e");
                } else if word_initial || prev_glide {
                    push_v(out, "e");
                } else if nx == 'r' && start == 3 && (w.starts_with("inter") || w.starts_with("hyper") || w.starts_with("super")) {
                    push_v(out, "ɛ");
                } else {
                    out.push(Seg { ph: "ə", v: true, schwa: true });
                }
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            'i' | 'î' | 'ï' | 'y' | 'ÿ' => {
                let is_y = a == 'y';
                let more = more_after(k, 1);
                let next_e_silent = more && e_silent(k + 1);
                if last_elem(k, 1) && nasal_after {
                    push_v(out, "ɛ̃");
                    prev_front = false;
                    k += 1;
                    continue;
                }
                if (more && !next_e_silent) || (vowel_follows && !more) {
                    // glide
                    if after_cluster && !is_y {
                        push_v(out, "i");
                    }
                    out.push(seg("j"));
                    prev_glide = true;
                    prev_front = true;
                    k += 1;
                    continue;
                }
                push_v(out, "i");
                prev_glide = false;
                prev_front = true;
                k += 1;
            }
            'o' | 'ö' => {
                if last_elem(k, 1) && nasal_after {
                    push_v(out, "ɔ̃");
                } else {
                    push_v(out, o_quality(c, j, m, me, w, more_after(k, 1)));
                }
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            'ô' => {
                if last_elem(k, 1) && nasal_after {
                    push_v(out, "ɔ̃");
                } else {
                    push_v(out, "oː");
                }
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            'u' | 'û' | 'ù' | 'ü' => {
                let more = more_after(k, 1);
                let next_e_silent = more && e_silent(k + 1);
                if last_elem(k, 1) && nasal_after {
                    // um final -> ɔm
                    if nasal_letter == 'm' && end_after_nasal && j + 1 >= n {
                        push_v(out, "ɔ");
                        out.push(seg("m"));
                        *i = j + 1;
                        return;
                    }
                    push_v(out, "œ̃");
                    k += 1;
                    continue;
                }
                if (more && !next_e_silent) || (vowel_follows && !more) {
                    if after_cluster {
                        push_v(out, "y");
                    } else {
                        out.push(seg("y"));
                    }
                    prev_glide = true;
                    prev_front = false;
                    k += 1;
                    continue;
                }
                push_v(out, if a == 'û' { "yː" } else { "y" });
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            'œ' => {
                push_v(out, if j < me && !is_vl(at(j)) && at(j) != 'u' { "e" } else { "œ" });
                prev_glide = false;
                prev_front = false;
                k += 1;
            }
            _ => {
                k += 1;
            }
        }
    }
    *i = j;
    // consumed nasal consonant
    if nasal_after && out.last().map_or(false, |g| g.v && g.ph.ends_with('̃')) {
        *i = j + 1;
    }
}

fn ient_is_tenir(w: &str) -> bool {
    if w.ends_with("vient") {
        return true;
    }
    if let Some(stem) = w.strip_suffix("tient") {
        return stem.is_empty()
            || ["sou", "main", "con", "ob", "re", "dé", "appar", "entre", "abs", "dis"].iter().any(|p| stem.ends_with(p));
    }
    false
}

/// ill after a consonant: /il/ in ville, mille, tranquille, million, ...
fn ill_is_l(w: &str, start: usize) -> bool {
    if start == 0 || (start == 3 && w.starts_with("dés")) {
        return true; // illusion, illégal, désillusion
    }
    for p in ["vill", "mill", "tranquill", "bacill", "oscill", "distill", "instill", "scintill" , "achill", "gill", "lill", "codicill", "pénicill", "cyrill", "capill", "pupill", "armill", "maxill", "axill", "vacill", "titill", "pusill", "imbécill"] {
        if w.starts_with(p) && !(p == "scintill" || p == "vacill" || p == "titill") {
            return true;
        }
        if let Some(pos) = w.find(p) {
            if pos > 0 && (p == "vill" || p == "mill") && (w.starts_with("vaude") || w.starts_with("deux")) {
                return true;
            }
        }
    }
    false
}

/// eu / œu: ø or œ by what follows.
fn eu_quality(c: &[char], j: usize, m: usize, _me: usize, closed_after: bool) -> &'static str {
    if j >= m {
        return "ø";
    }
    let a = c[j];
    let b = if j + 1 < m { c[j + 1] } else { '#' };
    match a {
        'z' | 'x' => return "ø",
        's' if b != 's' && (is_vl(b) || b == 'ə') => return "ø",
        't' | 'd' if j + 1 >= m || (b == 'e' && j + 2 >= m) || (b == 'r' && j + 2 < m && c[j + 2] == 'e' && j + 3 >= m) => return "ø",
        _ => {}
    }
    if closed_after {
        return "œ";
    }
    match a {
        'r' => if b == 'r' { "œ" } else { "ø" },
        'v' | 'n' | 'f' | 'b' | 'p' | 'g' | 'l' | 'j' => "œ",
        _ => "ø",
    }
}

/// o: ɔ or o by what follows.
fn o_quality(c: &[char], j: usize, m: usize, me: usize, w: &str, more_vowels: bool) -> &'static str {
    if more_vowels {
        return "ɔ";
    }
    if j >= me {
        return "o"; // final open syllable (mot, dos, trop, numéro)
    }
    let a = c[j];
    let rest: String = c[j..m].iter().collect();
    if a == 'z' || (a == 's' && j + 1 < m && (is_vl(c[j + 1]) || c[j + 1] == 'ə') && c[j + 1] != 's') {
        return "o";
    }
    if rest.starts_with("tion") || rest.starts_with("me") && j + 2 >= me && !w.ends_with("mme") && false {
        return "o";
    }
    if rest.starts_with("ss") && (w.starts_with("gross") || w.starts_with("foss")) {
        return "o";
    }
    "ɔ"
}

/// Post-processing: e-caduc deletion, geminate consonant merging.
fn finish(mut segs: Vec<Seg>) -> Vec<Seg> {
    // drop an unstable e inside the word when it follows a single consonant
    // that follows a vowel (samedi -> samdi), never in the first syllable
    let n = segs.len();
    let mut drop = vec![false; n];
    let mut first_nucleus = true;
    for i in 0..n {
        if !segs[i].v {
            continue;
        }
        if segs[i].schwa && !first_nucleus && i + 1 < n {
            // preceding consonant phones
            let mut k = i;
            let mut cons = 0;
            while k > 0 && !segs[k - 1].v {
                k -= 1;
                cons += 1;
            }
            let prev_is_vowel = k > 0 && segs[k - 1].v && !drop[k - 1];
            // following: must be a consonant then a vowel (not two consonants)
            let next_cons = !segs[i + 1].v;
            let next_next_vowel = i + 2 < n && (segs[i + 2].v || (matches!(segs[i + 2].ph, "j" | "w" | "y") && i + 3 < n && segs[i + 3].v));
            if cons == 1 && prev_is_vowel && next_cons && next_next_vowel && !matches!(segs[i - 1].ph, "ɲ") {
                drop[i] = true;
            }
        }
        first_nucleus = false;
    }
    let mut out: Vec<Seg> = Vec::with_capacity(n);
    let mut dropped_before = false;
    for (i, s) in segs.drain(..).enumerate() {
        if drop[i] {
            dropped_before = true;
            continue;
        }
        if let Some(last) = out.last() {
            if !last.v && !s.v && last.ph == s.ph && !dropped_before {
                continue;
            }
        }
        dropped_before = false;
        out.push(s);
    }
    out
}

// ---------------------------------------------------------------------------
// sentence pass

fn starts_with_vowel_phone(p: &str) -> bool {
    let first = p.chars().find(|&c| c != 'ˈ' && c != 'ˌ');
    matches!(first, Some('a' | 'e' | 'ɛ' | 'i' | 'o' | 'ɔ' | 'u' | 'ø' | 'œ' | 'ə' | 'ɑ' | 'y'))
        && !p.trim_start_matches(['ˈ', 'ˌ']).starts_with("yi")
        || p.trim_start_matches(['ˈ', 'ˌ']).starts_with("yi") && false
}

fn h_aspire(src: &str) -> bool {
    let w = src.split('\'').last().unwrap_or(src);
    if !w.starts_with('h') {
        return false;
    }
    H_ASPIRE.iter().any(|h| w.starts_with(h))
}

/// Liaison consonant a word contributes before a vowel (obligatory cases only).
fn liaison(src: &str) -> Option<&'static str> {
    let w = src.rsplit(['\'', '-']).next().unwrap_or(src);
    let z = Some("z");
    let t = Some("t");
    let n = Some("n");
    match w {
        "les" | "des" | "mes" | "tes" | "ses" | "ces" | "aux" | "leurs" | "nos" | "vos" | "quelques" | "plusieurs"
        | "deux" | "trois" | "six" | "dix" | "cents" | "vingts" | "ils" | "elles" | "nous" | "vous" | "très"
        | "plus" | "chez" | "dans" | "sans" | "sous" | "pas" | "petits" | "grands" | "gros" | "autres" | "beaux"
        | "vieux" | "nouveaux" | "bons" | "mauvais" | "tous" | "moins" | "premiers"
        | "derniers" | "certains" | "quels" | "quelles" | "tels" | "telles" | "uns" | "unes" => z,
        "est" | "c'est" | "sont" | "ont" | "vont" | "font" | "vingt" | "cent" | "petit" | "grand" | "quand" | "dont"
        | "tout" | "peut"
        | "cet" | "sept" | "huit" | "fort" | "sont-ils" => t,
        "un" | "mon" | "ton" | "son" | "bon" | "en" | "on" | "rien" | "bien" | "aucun" | "certain" | "ancien" | "prochain"
        | "plein" | "moyen" | "divin" | "commun" | "vilain" | "lointain" | "hautain" | "souverain" | "sain" => n,
        _ => None,
    }
}

/// Sentence pass: phrase-final stress on function words, number sandhi
/// (six/dix/huit/plus/tous), and obligatory liaison.
pub fn join(outs: &mut [Out]) {
    let n = outs.len();
    for k in 0..n {
        let src = outs[k].src.clone();
        let phrase_final = k + 1 == n || outs[k].pause_after || outs[k + 1].ph.is_empty();
        let next_src = if k + 1 < n { outs[k + 1].src.clone() } else { String::new() };
        let next_ph = if k + 1 < n { outs[k + 1].ph.clone() } else { String::new() };
        let next_vowel = !phrase_final && starts_with_vowel_phone(&next_ph) && !h_aspire(&next_src);
        let next_cons = !phrase_final && !next_ph.is_empty() && !next_vowel;
        // function words get their stress back at the end of a phrase
        if phrase_final && unstressed().contains(src.as_str()) && !never_stressed().contains(src.as_str()) {
            let bare: String = outs[k].ph.chars().filter(|&c| c != 'ˌ').collect();
            outs[k].ph = apply_stress(&bare, Stress::Primary);
        }
        // number and quantifier sandhi
        let last = src.rsplit('-').next().unwrap_or(&src).to_string();
        let stressed = outs[k].ph.contains('ˈ');
        let re = |p: &str, st: bool| if st { apply_stress(p, Stress::Primary) } else { p.to_string() };
        match last.as_str() {
            "six" | "dix" if src == last => {
                let b = if src == "six" { "si" } else { "di" };
                if next_cons && !matches!(next_src.as_str(), "et" | "ou") {
                    outs[k].ph = re(b, stressed);
                } else if next_vowel {
                    outs[k].ph = re(&format!("{b}z"), stressed);
                } else {
                    outs[k].ph = re(&format!("{b}s"), stressed);
                }
                continue;
            }
            "huit" if src == last => {
                outs[k].ph = re(if next_cons { "yi" } else { "yit" }, stressed);
                continue;
            }
            "neuf" if src == last => {
                if matches!(next_src.as_str(), "ans" | "heures" | "autres" | "hommes") {
                    outs[k].ph = re("nœv", stressed);
                }
                continue;
            }
            "plus" if src == last => {
                if next_vowel {
                    outs[k].ph = re("plyz", stressed);
                } else if next_cons {
                    outs[k].ph = re("ply", stressed);
                } else {
                    outs[k].ph = re("plys", stressed);
                }
                continue;
            }
            "tous" if src == last => {
                if matches!(next_src.as_str(), "les" | "ces" | "mes" | "tes" | "ses" | "nos" | "vos" | "leurs" | "deux" | "trois" | "quatre" | "cinq" | "six" | "sept" | "huit" | "neuf" | "dix" | "la" | "le" | "ceux") {
                    outs[k].ph = re("tu", stressed);
                } else if next_vowel {
                    outs[k].ph = re("tuz", stressed);
                }
                continue;
            }
            _ => {}
        }
        if next_vowel && !outs[k].ph.is_empty() {
            if let Some(cons) = liaison(&src) {
                if !(src == "et") && !next_src.starts_with("et") || next_src.len() > 2 {
                    if !outs[k].ph.ends_with(cons) {
                        outs[k].ph.push_str(cons);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// numbers and symbols

/// Number words (1990 spelling: hyphens everywhere below a million).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["zéro".into()];
    }
    let mut out: Vec<String> = Vec::new();
    let mut rest = n;
    let big: [(u128, &str, &str); 4] = [
        (1_000_000_000_000_000_000, "trillion", "trillions"),
        (1_000_000_000_000, "billion", "billions"),
        (1_000_000_000, "milliard", "milliards"),
        (1_000_000, "million", "millions"),
    ];
    for (unit, sg, pl) in big {
        if rest >= unit {
            let q = rest / unit;
            rest %= unit;
            out.push(below_million(q, false));
            out.push(if q == 1 { sg } else { pl }.into());
        }
    }
    if rest > 0 {
        out.push(below_million(rest, true));
    }
    out
}

fn below_million(n: u128, final_part: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    let thousands = n / 1000;
    let r = n % 1000;
    if thousands > 0 {
        if thousands > 1 {
            parts.push(below_thousand(thousands, false));
        }
        parts.push("mille".into());
    }
    if r > 0 {
        parts.push(below_thousand(r, final_part));
    }
    parts.join("-")
}

fn below_thousand(n: u128, plural_ok: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    let h = n / 100;
    let r = n % 100;
    if h > 0 {
        if h > 1 {
            parts.push(unit(h));
        }
        parts.push(if h > 1 && r == 0 && plural_ok { "cents" } else { "cent" }.into());
    }
    if r > 0 {
        parts.push(below_hundred(r, plural_ok));
    }
    parts.join("-")
}

fn unit(n: u128) -> String {
    const U: [&str; 17] = ["zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze", "douze", "treize", "quatorze", "quinze", "seize"];
    if (n as usize) < U.len() {
        U[n as usize].to_string()
    } else {
        format!("dix-{}", U[n as usize - 10])
    }
}

fn below_hundred(n: u128, plural_ok: bool) -> String {
    if n < 17 {
        return unit(n);
    }
    if n < 20 {
        return unit(n);
    }
    let (tens, name) = match n / 10 {
        2 => (20, "vingt"),
        3 => (30, "trente"),
        4 => (40, "quarante"),
        5 => (50, "cinquante"),
        6 | 7 => (60, "soixante"),
        _ => (80, "quatre-vingt"),
    };
    let r = n - tens;
    if r == 0 {
        return if tens == 80 && plural_ok { "quatre-vingts".into() } else { name.into() };
    }
    if tens == 80 {
        return format!("{name}-{}", unit(r));
    }
    if r == 1 || (tens == 60 && r == 11) {
        return format!("{name}-et-{}", unit(r));
    }
    format!("{name}-{}", unit(r))
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "pour cent",
        '$' => "dollars",
        '€' => "euros",
        '£' => "livres",
        '&' => "et",
        '+' => "plus",
        '@' => "arobase",
        '°' => "degrés",
        '=' => "égale",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    fn w(s: &str) -> String {
        word(s).replace(['ˈ', 'ˌ'], "")
    }

    #[test]
    fn basics() {
        for (s, p) in [
            ("maison", "mɛzɔ̃"),
            ("parlent", "paʁl"),
            ("moment", "mɔmɑ̃"),
            ("vraiment", "vʁɛmɑ̃"),
            ("enfant", "ɑ̃fɑ̃"),
            ("bien", "bjɛ̃"),
            ("science", "sjɑ̃s"),
            ("client", "klijɑ̃"),
            ("vient", "vjɛ̃"),
            ("patient", "pasjɑ̃"),
            ("un", "œ̃"),
            ("brun", "bʁœ̃"),
            ("nation", "nasjɔ̃"),
            ("question", "kɛstjɔ̃"),
            ("eau", "o"),
            ("beaucoup", "boku"),
            ("chose", "ʃoz"),
            ("homme", "ɔm"),
            ("école", "ekɔl"),
            ("petit", "pəti"),
            ("samedi", "samdi"),
            ("gouvernement", "ɡuvɛʁnəmɑ̃"),
            ("fille", "fij"),
            ("ville", "vil"),
            ("travail", "tʁavaj"),
            ("soleil", "sɔlɛj"),
            ("accueil", "akœj"),
            ("œil", "œj"),
            ("cœur", "kœʁ"),
            ("deux", "dø"),
            ("peur", "pœʁ"),
            ("heureux", "øʁø"),
            ("lui", "lyi"),
            ("nuit", "nyi"),
            ("fruit", "fʁyi"),
            ("oui", "wi"),
            ("jouer", "ʒwe"),
            ("voir", "vwaʁ"),
            ("loin", "lwɛ̃"),
            ("voyage", "vwajaʒ"),
            ("payer", "pɛje"),
            ("agneau", "aɲo"),
            ("gnome", "ɡnom"),
            ("guerre", "ɡɛʁ"),
            ("langue", "lɑ̃ɡ"),
            ("quand", "kɑ̃"),
            ("chat", "ʃa"),
            ("chaos", "kao"),
            ("technique", "tɛknik"),
            ("exemple", "ɛɡzɑ̃pl"),
            ("taxi", "taksi"),
            ("prix", "pʁi"),
            ("dix", "dis"),
            ("parler", "paʁle"),
            ("fer", "fɛʁ"),
            ("premier", "pʁəmje"),
            ("hiver", "ivɛʁ"),
            ("femme", "fam"),
            ("évidemment", "evidamɑ̃"),
            ("fils", "fis"),
            ("temps", "tɑ̃"),
            ("grand", "ɡʁɑ̃"),
            ("blanc", "blɑ̃"),
            ("sac", "sak"),
            ("respect", "ʁɛspɛ"),
            ("direct", "diʁɛkt"),
            ("album", "albɔm"),
            ("parfum", "paʁfœ̃"),
            ("l'homme", "lɔm"),
            ("aujourd'hui", "oʒuʁdyi"),
            ("qu'il", "kil"),
            ("peut-être", "pøtɛtʁ"),
            ("est-ce", "ɛs"),
            ("va-t-il", "vatil"),
            ("monsieur", "məsjø"),
            ("pays", "pei"),
            ("second", "səɡɔ̃"),
            ("mettre", "mɛtʁ"),
            ("cellule", "selyl"),
            ("effet", "efɛ"),
            ("terre", "tɛʁ"),
            ("dessus", "dəsy"),
            ("ressembler", "ʁəsɑ̃ble"),
            ("transition", "tʁɑ̃zisjɔ̃"),
            ("ensemble", "ɑ̃sɑ̃bl"),
            ("poison", "pwazɔ̃"),
            ("poisson", "pwasɔ̃"),
            ("ancienne", "ɑ̃sjɛn"),
            ("européen", "øʁɔpeɛ̃"),
            ("moyen", "mwajɛ̃"),
            ("juin", "ʒyɛ̃"),
            ("point", "pwɛ̃"),
            ("faim", "fɛ̃"),
            ("plein", "plɛ̃"),
            ("pleine", "plɛn"),
            ("automne", "otɔn"),
            ("immense", "imɑ̃s"),
            ("innocent", "inɔsɑ̃"),
            ("nez", "ne"),
            ("ballet", "balɛ"),
            ("pied", "pje"),
            ("tables", "tabl"),
            ("étudient", "etydi"),
            ("étaient", "etɛ"),
            ("voient", "vwa"),
            ("jouent", "ʒu"),
            ("créer", "kʁee"),
            ("théâtre", "teaːtʁ"),
            ("mangeons", "mɑ̃ʒɔ̃"),
            ("pigeon", "piʒɔ̃"),
            ("ciel", "sjɛl"),
            ("pierre", "pjɛʁ"),
            ("dieu", "djø"),
            ("vieux", "vjø"),
            ("yeux", "jø"),
            ("aiguille", "eɡyij"),
            ("lentement", "lɑ̃tmɑ̃"),
            ("appartement", "apaʁtəmɑ̃"),
            ("médecin", "medsɛ̃"),
            ("maintenant", "mɛ̃tnɑ̃"),
            ("développement", "devlɔpmɑ̃"),
            ("cheval", "ʃəval"),
            ("gens", "ʒɑ̃"),
            ("sens", "sɑ̃s"),
            ("tous", "tus"),
            ("os", "ɔs"),
            ("fin", "fɛ̃"),
            ("fine", "fin"),
            ("bonne", "bɔn"),
            ("bon", "bɔ̃"),
            ("sept", "sɛt"),
            ("huit", "yit"),
            ("vingt", "vɛ̃"),
            ("cent", "sɑ̃"),
            ("mille", "mil"),
            ("vingt-et-un", "vɛ̃teœ̃"),
            ("dix-huit", "dizyit"),
            ("quatre-vingt-dix-neuf", "katʁəvɛ̃diznœf"),
            ("deux-cents", "døsɑ̃"),
            ("soixante-et-onze", "swasɑ̃teɔ̃z"),
        ] {
            assert_eq!(w(s), p, "{s}");
        }
    }

    #[test]
    fn stress_marks() {
        assert_eq!(word("maison"), "mɛzˈɔ̃");
        assert_eq!(word("être"), "ˈɛtʁ");
        assert_eq!(word("lui"), "lyˌi");
        assert_eq!(word("de"), "də");
        assert_eq!(word("petite"), "pətˈit");
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(21).join(" "), "vingt-et-un");
        assert_eq!(cardinal(80).join(" "), "quatre-vingts");
        assert_eq!(cardinal(81).join(" "), "quatre-vingt-un");
        assert_eq!(cardinal(71).join(" "), "soixante-et-onze");
        assert_eq!(cardinal(200).join(" "), "deux-cents");
        assert_eq!(cardinal(201).join(" "), "deux-cent-un");
        assert_eq!(cardinal(1999).join(" "), "mille-neuf-cent-quatre-vingt-dix-neuf");
        assert_eq!(cardinal(2_000_000).join(" "), "deux millions");
        assert_eq!(cardinal(1_000_000).join(" "), "un million");
        assert_eq!(w("mille-neuf-cent-quatre-vingt-dix-neuf"), "mil nœfsɑ̃ katʁəvɛ̃diznœf");
    }

    #[test]
    fn sentence() {
        let g = crate::G2p::new(None);
        let p = g.phonemize("Les enfants arrivent.", crate::Lang::Fr);
        assert_eq!(p, "lez ɑ̃fˈɑ̃ aʁˈiv.");
        let p = g.phonemize("Il est arrivé à la maison.", crate::Lang::Fr);
        assert_eq!(p, "il ɛt aʁivˈe a la mɛzˈɔ̃.");
        let p = g.phonemize("C'est un grand ami.", crate::Lang::Fr);
        assert_eq!(p, "sɛt œ̃ ɡʁˈɑ̃t amˈi.");
        let p = g.phonemize("six mois, six ans, six", crate::Lang::Fr);
        assert_eq!(p, "sˈi mwˈa, sˈiz ˈɑ̃, sˈis");
        let p = g.phonemize("25 personnes", crate::Lang::Fr);
        assert_eq!(p, "vɛ̃tsˈɛ̃k pɛʁsˈɔn");
    }
}
