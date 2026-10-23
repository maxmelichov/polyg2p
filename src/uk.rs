//! Ukrainian grapheme-to-phoneme, espeak-style notation.
//!
//! Ukrainian spelling is close to phonemic: the letter rules (iotation vs
//! softening of я/ю/є/ї/і/ь, the apostrophe, дз/дж affricates, щ, regressive
//! voicing and softness assimilation, the з-/роз- sibilant clusters) are all
//! here. What spelling does not give is the stress, which is lexical and
//! mobile. It comes from, in order: a built-in table of function words and
//! numerals, the optional lexicon `uk.lex` (CC BY-SA data pack, loaded from
//! `$POLYG2P_DATA`), ending statistics (`data/uk_suffix.tsv`, aggregate counts
//! derived from Wiktionary), and finally the penultimate syllable.
//!
//! Notation follows espeak-ng's `uk` voice: vowels `a o u ɛ ɪ iː`, `β` for в in
//! every position, `h` for г, `t̪` for т (`d̪` when voiced by assimilation),
//! `ʎ ɲ` for soft л/н and `ʲ` on the other soft dentals, `ʃʧ` for щ, and the
//! stress mark directly before the stressed vowel. Unlike espeak, soft
//! consonants are marked wherever Ukrainian has them, obstruents never devoice
//! before voiceless consonants or word-finally, and в never becomes `f`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use crate::lexicon::Lexicon;
use crate::Out;

const SUFFIX_TSV: &str = include_str!("../data/uk_suffix.tsv");
const STEM_TSV: &str = include_str!("../data/uk_stem.tsv");

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "кома";
/// Names of the Latin letters a..z, for spelling acronyms (NATO, USA).
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "bɛ", "tsɛ", "dɛ", "ɛ", "ɛf", "ɡɛ", "aʃ", "iː", "jot̪", "ka", "ɛl", "ɛm", "ɛn", "o", "pɛ", "ku", "ɛr", "ɛs",
    "t̪ɛ", "u", "βɛ", "dubʎβɛ", "iːks", "iːɡrɛk", "zɛt̪",
]);

/// Common abbreviations read as the full word.
const ABBREV: &[(&str, &str)] = &[
    ("грн", "гривень"), ("кг", "кілограм"), ("км", "кілометрів"), ("млн", "мільйонів"), ("млрд", "мільярдів"),
    ("тис", "тисяч"), ("вул", "вулиця"), ("обл", "область"), ("ім", "імені"), ("св", "святий"),
];

/// Proclitics espeak reads without a stress mark.
const CLITICS: &[&str] = &["в", "з", "й", "ж", "б", "у"];

/// Stress (vowel index from the end) of function words, numerals and other
/// frequent words whose stress the ending statistics get wrong. Hand-written.
const STRESS_TABLE: &[(&str, u8)] = &[
    // pronouns
    ("мене", 0), ("мені", 0), ("тебе", 0), ("тобі", 0), ("себе", 0), ("собі", 0), ("його", 0), ("йому", 0),
    ("нього", 0), ("ньому", 1), ("вона", 0), ("вони", 0), ("воно", 0), ("вона", 0), ("нами", 1), ("вами", 1),
    ("нього", 0), ("свою", 0), ("свого", 0), ("своє", 0), ("своєї", 1), ("своїм", 0), ("своїх", 0), ("своїй", 0),
    ("свої", 0), ("мого", 0), ("моя", 0), ("моє", 0), ("мої", 0), ("моєї", 1), ("моїм", 0), ("моїх", 0),
    ("моїй", 0), ("мою", 0), ("твого", 0), ("твоя", 0), ("твоє", 0), ("твої", 0), ("твою", 0), ("твоїх", 0),
    ("цього", 0), ("цьому", 0), ("того", 0), ("тому", 0), ("така", 0), ("таке", 0), ("такі", 0), ("такий", 0),
    ("таку", 0), ("таким", 0), ("таких", 0), ("такої", 1), ("усі", 0), ("усіх", 0), ("усім", 0), ("усе", 0),
    ("уся", 0), ("усю", 0), ("вся", 0), ("все", 0), ("всі", 0), ("всіх", 0), ("всім", 0), ("всю", 0),
    ("сама", 0), ("саме", 1), ("самі", 0), ("самий", 1), ("самого", 2), ("який", 0), ("яка", 0), ("яке", 0),
    ("які", 0), ("якого", 1), ("якої", 1), ("якому", 1), ("яким", 0), ("яких", 0), ("яку", 0), ("котрий", 0),
    ("одна", 0), ("одне", 0), ("одні", 0), ("одного", 1), ("одній", 0), ("одну", 0), ("одним", 0), ("одних", 0),
    ("інша", 1), ("інше", 1), ("інші", 0), ("інший", 1), ("іншого", 2), ("іншій", 0), ("ніхто", 0), ("ніщо", 0),
    ("ніколи", 1), ("ніде", 0), ("ніяк", 0), ("нічого", 1), ("нікого", 1), ("нікому", 1), ("хтось", 0),
    ("щось", 0), ("десь", 0), ("колись", 0), ("якось", 0), ("хтозна", 1), ("абищо", 1),
    // adverbs, conjunctions, particles
    ("також", 1), ("тепер", 0), ("тоді", 0), ("коли", 0), ("завжди", 0), ("зараз", 1), ("потім", 1),
    ("тільки", 1), ("лише", 0), ("тому", 0), ("чому", 0), ("щоби", 1), ("хоча", 0), ("хоч", 0), ("якщо", 0),
    ("якби", 0), ("навіть", 1), ("дуже", 1), ("майже", 1), ("вже", 0), ("уже", 0), ("однак", 0), ("проте", 0),
    ("адже", 0), ("отже", 1), ("тобто", 1), ("також", 1), ("тут", 0), ("там", 0), ("туди", 0), ("сюди", 0),
    ("звідки", 1), ("куди", 0), ("ніби", 1), ("наче", 1), ("нехай", 0), ("хай", 0), ("нема", 0), ("немає", 1),
    ("було", 0), ("була", 0), ("були", 0), ("буде", 1), ("буду", 1), ("будеш", 1), ("будуть", 1), ("будемо", 2),
    ("бути", 1), ("можна", 1), ("треба", 1), ("можу", 1), ("може", 1), ("можуть", 1), ("можеш", 1),
    ("може", 1), ("хочу", 1), ("хоче", 1), ("хочуть", 1), ("маю", 1), ("має", 1), ("мають", 1), ("далі", 1),
    ("перед", 1), ("після", 1), ("через", 1), ("проти", 1), ("серед", 1), ("поміж", 0), ("між", 0),
    ("понад", 0), ("попри", 1), ("задля", 1), ("біля", 1), ("коло", 1), ("повз", 0), ("крізь", 0),
    ("щодо", 1), ("окрім", 0), ("крім", 0), ("завдяки", 1), ("близько", 1), ("навколо", 1), ("довкола", 1),
    ("разом", 1), ("знову", 1), ("знов", 0), ("часто", 1), ("іноді", 2), ("інколи", 2), ("досить", 1),
    ("дуже", 1), ("багато", 1), ("мало", 1), ("трохи", 1), ("зовсім", 1), ("взагалі", 0), ("загалом", 1),
    ("особливо", 1), ("зокрема", 0), ("наприклад", 1), ("звичайно", 1), ("справді", 1), ("дійсно", 1),
    ("добре", 1), ("погано", 1), ("краще", 1), ("гірше", 1), ("більше", 1), ("менше", 1), ("більш", 0),
    ("менш", 0), ("найбільш", 0), ("сьогодні", 1), ("вчора", 1), ("учора", 1), ("завтра", 1), ("ранок", 1),
    ("спочатку", 1), ("нарешті", 1), ("поки", 1), ("доки", 1), ("відразу", 1), ("одразу", 1), ("зараз", 1),
    ("ось", 0), ("осьде", 1), ("оце", 0), ("отак", 0), ("так", 0), ("отож", 0), ("авжеж", 0), ("ага", 0),
    ("дякую", 2), ("будь", 0), ("ласка", 1), ("добридень", 1), ("вітаю", 1), ("привіт", 0), ("вибач", 1),
    ("пане", 1), ("пані", 1), ("люди", 1), ("людей", 0), ("людям", 1), ("людини", 1), ("людина", 1),
    ("дітей", 0), ("діти", 1), ("дітям", 1), ("гроші", 1), ("грошей", 1), ("років", 0), ("роки", 0),
    ("року", 1), ("рік", 0), ("разів", 0), ("рази", 0), ("разу", 1), ("часу", 1), ("часи", 0), ("часів", 0),
    ("країна", 1), ("країни", 1), ("країні", 1), ("країну", 1), ("країн", 0), ("держава", 1), ("місто", 1),
    ("міста", 1), ("міст", 0), ("голова", 0), ("голови", 0), ("голову", 2), ("життя", 0), ("життю", 0),
    ("робота", 1), ("роботи", 1), ("роботу", 1), ("робіт", 0), ("слово", 1), ("слова", 1), ("слів", 0),
    ("сказав", 0), ("сказала", 1), ("сказали", 1), ("каже", 1), ("кажуть", 1), ("казав", 0), ("казала", 1),
    ("знаю", 1), ("знає", 1), ("знаєш", 1), ("знають", 1), ("знав", 0), ("знала", 1), ("думаю", 2),
    ("думає", 2), ("думають", 2), ("хотів", 0), ("хотіла", 1), ("хотіли", 1), ("пішов", 0), ("пішла", 0),
    ("прийшов", 0), ("прийшла", 0), ("прийшли", 0), ("дати", 1), ("дала", 0), ("дали", 0), ("взяти", 1),
    ("взяв", 0), ("взяла", 0), ("взяли", 1), ("стати", 1), ("став", 0), ("стала", 1), ("стали", 1),
    ("нова", 0), ("нове", 0), ("нові", 0), ("новий", 0), ("нового", 1), ("нових", 0), ("новим", 0),
    ("стара", 0), ("старе", 0), ("старі", 0), ("старий", 0), ("старого", 1), ("мала", 1), ("малі", 0),
    ("малий", 0), ("велика", 1), ("велике", 1), ("великі", 1), ("великий", 1), ("великого", 2),
    ("українська", 1), ("українське", 1), ("українські", 1), ("український", 1), ("українською", 2),
    ("українців", 1), ("українці", 1), ("україна", 1), ("україни", 1), ("україні", 1), ("україну", 1),
    ("україною", 2), ("київ", 1), ("києва", 2), ("києві", 2), ("львів", 0), ("львові", 1), ("одеса", 1),
    ("харків", 1), ("дніпро", 0), ("донецьк", 0), ("європа", 1), ("європи", 1), ("росія", 1), ("росії", 1),
    // numerals
    ("нуль", 0), ("один", 0), ("два", 0), ("дві", 0), ("три", 0), ("чотири", 1), ("шість", 0), ("сім", 0),
    ("вісім", 1), ("десять", 1), ("одинадцять", 1), ("дванадцять", 1), ("тринадцять", 1), ("чотирнадцять", 1),
    ("п'ятнадцять", 1), ("шістнадцять", 1), ("сімнадцять", 1), ("вісімнадцять", 1), ("дев'ятнадцять", 1),
    ("двадцять", 1), ("тридцять", 1), ("сорок", 1), ("п'ятдесят", 0), ("шістдесят", 0), ("сімдесят", 0),
    ("вісімдесят", 0), ("дев'яносто", 1), ("сто", 0), ("двісті", 1), ("триста", 1), ("чотириста", 2),
    ("п'ятсот", 0), ("шістсот", 0), ("сімсот", 0), ("вісімсот", 0), ("дев'ятсот", 0), ("тисяча", 2),
    ("тисячі", 2), ("тисяч", 1), ("мільйон", 0), ("мільйони", 1), ("мільйонів", 1), ("мільярд", 0),
    ("мільярди", 1), ("мільярдів", 1), ("трильйон", 0), ("трильйони", 1), ("трильйонів", 1),
    ("перший", 1), ("перша", 1), ("перше", 1), ("другий", 1), ("друга", 1), ("третій", 1), ("третя", 1),
    ("кома", 1), ("відсотків", 1), ("відсоток", 1), ("відсотки", 1), ("доларів", 2), ("долар", 1),
    ("євро", 1), ("фунтів", 1), ("гривень", 1), ("гривня", 1), ("градусів", 2), ("плюс", 0),
    ("мінус", 1), ("дорівнює", 2), ("собака", 1), ("просьба", 1), ("розчарований", 2), ("кілограм", 0), ("кілограмів", 1),
    ("кілометр", 0), ("кілометрів", 1), ("область", 1), ("вулиця", 2), ("мільйонів", 1),

    // forms of pronouns and frequent words whose stress the statistics miss
    ("такого", 1), ("такому", 1), ("такою", 1), ("такими", 1), ("такій", 0), ("якій", 0), ("якою", 1),
    ("якими", 1), ("якась", 0), ("якесь", 0), ("якісь", 0), ("якомусь", 1), ("одному", 0), ("однією", 1),
    ("одних", 0), ("одної", 1), ("однієї", 1), ("усього", 0), ("всього", 0), ("кого", 0), ("чого", 0),
    ("когось", 0), ("чогось", 0), ("комусь", 0), ("чомусь", 0), ("кимось", 1), ("чимось", 1), ("кожен", 1),
    ("кожна", 1), ("кожне", 1), ("кожні", 1), ("кожного", 2), ("кожному", 2), ("кожної", 2), ("кожну", 1),
    ("кожним", 1), ("кожних", 1), ("деякі", 2), ("деяких", 2), ("деякий", 2), ("деяка", 2), ("деяке", 2),
    ("деяку", 2), ("деяким", 2), ("деякими", 3), ("інших", 1), ("іншим", 1), ("іншими", 2), ("іншу", 1),
    ("іншою", 2), ("іншої", 2), ("іншому", 2), ("місце", 1), ("місця", 1), ("місці", 1), ("давно", 0),
    ("здається", 1), ("таки", 0), ("подобається", 3), ("працює", 1), ("розвитку", 2), ("числі", 0),
    ("вулиці", 2), ("вулиця", 2), ("вулицю", 2), ("вулиць", 1), ("розумію", 1), ("розуміє", 1), ("аби", 0),
    ("війна", 0), ("війну", 0), ("війни", 0), ("війні", 0), ("війною", 1), ("народився", 1), ("сергій", 0), ("словами", 1), ("кінця", 0), ("метою", 1), ("стає", 0),
    ("депутати", 1), ("захисту", 2), ("уряду", 2), ("дає", 0), ("вигляді", 2), ("декілька", 2),
    ("війська", 0), ("засідання", 2), ("місяців", 2), ("місяці", 2), ("місяць", 1), ("головний", 0),
    ("авто", 0), ("боротьби", 0), ("зрозумів", 0), ("пам'яті", 2), ("працюють", 1), ("вибір", 1),
    ("живе", 0), ("людину", 1), ("потрібен", 1), ("галузі", 2), ("номер", 1), ("селі", 0), ("виборів", 2),
    ("нову", 0), ("інтерв'ю", 0), ("вдалося", 1), ("дороги", 1), ("вирішив", 2), ("вирішили", 3),
    ("вирішила", 3), ("вирішити", 3), ("готовий", 1), ("зустрічі", 2), ("зустріч", 1), ("обов'язково", 1),
    ("піти", 0), ("чудово", 1), ("вийшов", 1), ("вийшла", 1), ("вийшли", 1), ("вийшло", 1), ("вийти", 1),
    ("напрямку", 2), ("поблизу", 0), ("працював", 0), ("ціна", 0), ("бойових", 0), ("матері", 2),
    ("батьки", 0), ("вибачте", 2), ("вода", 0), ("бізнесу", 2), ("представників", 0), ("путін", 1),
    ("свобода", 1), ("мешканців", 2), ("науково", 1), ("петра", 0), ("стаття", 0), ("василь", 0),
    ("використовують", 2), ("ані", 0), ("велику", 1), ("відбудеться", 2), ("відбулася", 1), ("засобів", 2),
    ("руках", 0), ("невже", 0), ("пісня", 1), ("водночас", 0), ("олексій", 0), ("статтю", 0), ("москви", 0),
    ("бере", 0), ("виключно", 2), ("заходи", 2), ("земля", 0), ("наслідки", 2), ("павло", 0), ("послуг", 1),
    ("продаж", 1), ("суддя", 0), ("годину", 1), ("добу", 0), ("заходу", 2), ("місцях", 0), ("їхати", 2),
    ("виборах", 2), ("випадків", 2), ("відбулося", 1), ("лікар", 1), ("стосується", 2), ("музей", 0),
    ("містах", 0), ("суді", 0), ("вперед", 0), ("дають", 0), ("навпаки", 0), ("поняття", 1),
    ("тимчасово", 1), ("відносини", 2), ("звання", 0), ("засіданні", 1), ("марія", 1), ("хлопець", 1),
    ("мета", 0), ("поряд", 1), ("причому", 1), ("визначення", 3), ("обласної", 1), ("руками", 1),
    ("річний", 0), ("забув", 0), ("святого", 1), ("живу", 0), ("пов'язані", 2), ("чудовий", 1),
    ("заході", 2), ("москва", 0), ("глава", 0), ("боротьбу", 0), ("брата", 1), ("вогню", 0),
    ("головна", 0), ("наприкінці", 1), ("щодня", 0), ("випадково", 1), ("знання", 0), ("показує", 2),
    ("транспорту", 2), ("хочемо", 2), ("майно", 0), ("приїхав", 1), ("прокурор", 0), ("піду", 0),
    ("стоять", 0), ("частково", 1), ("любові", 1), ("повідомили", 2), ("іде", 0), ("дніпра", 0),
    ("кафедри", 2), ("методи", 2), ("поступово", 1), ("працювали", 1), ("визнати", 2), ("вирішення", 3),
    ("засоби", 2), ("надія", 1), ("областей", 0), ("опитування", 3), ("сидіти", 1), ("ціну", 0),
    ("виконує", 2), ("відмовився", 2), ("жили", 0), ("зброєю", 2), ("успіху", 2), ("великою", 2),
    ("довелося", 1), ("лежить", 0), ("машин", 0), ("надії", 1), ("шевченко", 1), ("янукович", 1),
    ("зберегти", 0), ("коштує", 2), ("міська", 0), ("судом", 0), ("висновок", 2), ("новина", 0),
    ("виявили", 3), ("ворога", 2), ("мешканці", 2), ("молодих", 0), ("число", 0), ("активісти", 1),
    ("неодноразово", 2), ("обласного", 1), ("обслуговування", 3), ("пісень", 0), ("ворота", 1),
    ("декількох", 2), ("злочину", 2), ("лідерів", 2), ("приїхали", 2), ("тренер", 1), ("увесь", 0),
    ("висновки", 2), ("зате", 0), ("нібито", 2), ("призначений", 2), ("харкова", 2), ("виграв", 1),
    ("використовує", 2), ("дівчинка", 2), ("метод", 1), ("рота", 1), ("весело", 2), ("дома", 1),
    ("злочинів", 2), ("комплексу", 2), ("листа", 0), ("обставин", 1), ("дмитра", 0), ("друзями", 2),
    ("бояться", 1), ("виготовлення", 4), ("дочка", 0), ("житло", 0), ("максим", 0), ("чимало", 1),
    ("восени", 0), ("літака", 0), ("ідею", 1), ("брехня", 0), ("важливий", 1), ("воювати", 1),
    ("зобов'язання", 2), ("міський", 0), ("напад", 1), ("нападу", 2), ("неподалік", 0), ("прийом", 0),
    ("відбулися", 1), ("дозволити", 2), ("кудись", 0), ("майор", 0), ("біда", 0), ("кафе", 0),
    ("колишніх", 1), ("котра", 0), ("листи", 0), ("півтора", 0), ("реклами", 1), ("актор", 0),
    ("живий", 0), ("молода", 0), ("працювала", 1), ("святий", 0), ("іще", 0), ("визначити", 3),
    ("відстані", 2), ("вінниці", 2), ("долари", 2), ("однаково", 2), ("будете", 2), ("будуть", 1),
    ("але", 0), ("або", 0), ("відео", 2), ("березня", 2), ("протягом", 2), ("вересня", 2), ("особи", 1),
    ("мовою", 2), ("здоров'я", 1), ("говорить", 1), ("допомоги", 1), ("освіти", 1), ("допомогу", 1),
    ("повідомлення", 2), ("допомога", 1), ("використовувати", 3), ("поліція", 2), ("особа", 1),
    ("відновлення", 2), ("повідомив", 1), ("відділення", 2), ("новому", 1), ("допоможе", 1),
    ("територію", 2), ("відміну", 1), ("освіта", 1), ("офіційний", 1), ("хороший", 1), ("освіту", 1),
    ("використовується", 3), ("відносин", 1), ("підготовки", 1), ("говорять", 1), ("обговорення", 2),
    ("остаточно", 1), ("розмови", 1), ("регіону", 1), ("заборону", 1), ("боротися", 2), ("особу", 1),
    ("особою", 2), ("осіб", 0), ("квітня", 1), ("травня", 1), ("червня", 1), ("липня", 1), ("серпня", 1),
    ("жовтня", 1), ("листопада", 1), ("грудня", 1), ("січня", 1), ("лютого", 2), ("мови", 1), ("мова", 1),
    ("мову", 1), ("мові", 1), ("повідомляє", 1), ("повідомили", 2), ("повідомила", 2), ("допомагати", 1),
    ("допомагає", 1), ("допомогли", 0), ("допоміг", 0), ("хорошого", 2), ("хороша", 1), ("хороше", 1),
    ("хороші", 1), ("офіційно", 1), ("офіційного", 2), ("території", 2), ("територія", 2),
    ("використання", 1), ("відношення", 2), ("підготовку", 1), ("підготовка", 1), ("регіон", 0),
    ("регіони", 1), ("регіонів", 1), ("заборона", 1), ("заборони", 1), ("розмова", 1), ("розмову", 1),
    ("говорив", 0), ("говорила", 1), ("говорили", 1), ("говорю", 0), ("говорити", 1), ("казати", 1),
    ("сказати", 1), ("скажу", 0), ("скаже", 1), ("скажіть", 0), ("прошу", 1), ("просив", 0),
    ("хотіти", 1), ("хочеш", 1), ("хочете", 2), ("можете", 2), ("можемо", 2), ("можливо", 1),
    ("можливість", 1), ("можливості", 2), ("необхідно", 1), ("потрібно", 1), ("потрібна", 1),
    ("потрібні", 1), ("зрозуміло", 1), ("зрозуміти", 1), ("розуміти", 1), ("розуміють", 1),
    ("питання", 1), ("питань", 0), ("відповідь", 2), ("відповіді", 2), ("відповідно", 1),
    ("відповідає", 1), ("відповів", 0), ("відповіла", 0), ("рішення", 2), ("рішень", 1), ("вирішив", 2),
];

/// Lexicon + ending statistics, loaded once.
struct Res {
    /// ending -> stress index from the end
    suffix: HashMap<String, u8>,
    /// word start -> stress index from the start (stems with fixed stress)
    stem: HashMap<String, u8>,
    lex: Option<Lexicon>,
}

fn load_table(tsv: &str) -> HashMap<String, u8> {
    let mut t = HashMap::new();
    for line in tsv.lines() {
        let mut f = line.split('\t');
        if let (Some(e), Some(st)) = (f.next(), f.next()) {
            if let Ok(st) = st.parse() {
                t.insert(e.to_string(), st);
            }
        }
    }
    t
}

static RES: OnceLock<Res> = OnceLock::new();

fn res() -> &'static Res {
    RES.get_or_init(|| {
        let suffix = load_table(SUFFIX_TSV);
        let stem = load_table(STEM_TSV);
        // the optional stress lexicon lives in the data pack next to it.lex etc.
        let lex = std::env::var_os("POLYG2P_DATA")
            .map(|d| Path::new(&d).join("uk.lex"))
            .filter(|p| p.exists())
            .and_then(|p| Lexicon::open(&p).ok());
        Res { suffix, stem, lex }
    })
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'а' | 'е' | 'є' | 'и' | 'і' | 'ї' | 'о' | 'у' | 'ю' | 'я')
}
fn is_cyr(c: char) -> bool {
    matches!(c, 'а'..='я' | 'і' | 'ї' | 'є' | 'ґ' | 'ё')
}

/// Stress as vowel index from the end for a word with `nv` vowel letters.
fn stress_from_end(lw: &str, nv: usize) -> usize {
    if nv <= 1 {
        return 0;
    }
    if let Some(&(_, st)) = STRESS_TABLE.iter().find(|(w, _)| *w == lw) {
        if (st as usize) < nv {
            return st as usize;
        }
    }
    let r = res();
    if let Some(l) = &r.lex {
        if let Some(v) = l.get(lw) {
            let first = v.split(',').next().unwrap_or(v);
            if let Ok(st) = first.parse::<usize>() {
                if st < nv {
                    return st;
                }
            }
        }
    }
    // reflexive verbs: the base verb's ending decides when the -ся form is unknown
    if let Some(base) = lw.strip_suffix("ся").or_else(|| lw.strip_suffix("сь")) {
        let bnv = base.chars().filter(|&c| is_vowel_letter(c)).count();
        if bnv >= 1 {
            if let Some(st) = stem_stress(&r.stem, lw, nv) {
                return st;
            }
            if let Some(st) = suffix_stress(&r.suffix, lw, nv) {
                return st;
            }
            if let Some(st) = suffix_stress(&r.suffix, base, bnv) {
                return st + (nv - bnv);
            }
        }
    }
    if let Some(st) = stem_stress(&r.stem, lw, nv) {
        return st;
    }
    if let Some(st) = suffix_stress(&r.suffix, lw, nv) {
        return st;
    }
    1
}

fn stem_stress(table: &HashMap<String, u8>, lw: &str, nv: usize) -> Option<usize> {
    let idx: Vec<usize> = lw.char_indices().map(|(b, _)| b).collect();
    let n = idx.len();
    if n < 5 {
        return None;
    }
    for k in (4..=8.min(n - 1)).rev() {
        if let Some(&st) = table.get(&lw[..idx[k]]) {
            if (st as usize) < nv {
                return Some(nv - 1 - st as usize);
            }
        }
    }
    None
}

fn suffix_stress(table: &HashMap<String, u8>, lw: &str, nv: usize) -> Option<usize> {
    let idx: Vec<usize> = lw.char_indices().map(|(b, _)| b).collect();
    let n = idx.len();
    for k in (1..=9).rev() {
        if n > k {
            if let Some(&st) = table.get(&lw[idx[n - k]..]) {
                if (st as usize) < nv {
                    return Some(st as usize);
                }
            }
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum K {
    Vowel,
    Cons,
}

#[derive(Clone, Debug)]
struct Seg {
    k: K,
    /// base phone: vowels a o u ɛ ɪ iː; consonants b p β m f d t z s ts ʣ n l r h ɡ k x ʒ ʃ ʧ ʤ j
    p: &'static str,
    soft: bool,
    /// labial/velar + я/ю/є: rendered C + j
    jglide: bool,
    /// labial/velar/husher before і or a soft vowel: semi-palatalized (not written,
    /// but it softens a preceding dental)
    semi: bool,
    /// т voiced by assimilation (espeak writes d̪)
    from_t: bool,
    /// long consonant (the -ться affricate)
    long: bool,
    stressed: bool,
}

fn cons(p: &'static str) -> Seg {
    Seg { k: K::Cons, p, soft: false, jglide: false, semi: false, from_t: false, long: false, stressed: false }
}
fn vowel(p: &'static str) -> Seg {
    Seg { k: K::Vowel, p, soft: false, jglide: false, semi: false, from_t: false, long: false, stressed: false }
}

fn is_dental(p: &str) -> bool {
    matches!(p, "t" | "d" | "s" | "z" | "ts" | "ʣ" | "n" | "l")
}
fn softenable(p: &str) -> bool {
    is_dental(p) || p == "r"
}
fn is_husher(p: &str) -> bool {
    matches!(p, "ʃ" | "ʒ" | "ʧ" | "ʤ")
}
fn is_voiced_obstruent(p: &str) -> bool {
    matches!(p, "b" | "d" | "ɡ" | "z" | "ʒ" | "ʣ" | "ʤ" | "h")
}
fn voice(p: &str) -> Option<&'static str> {
    Some(match p {
        "p" => "b",
        "t" => "d",
        "k" => "ɡ",
        "s" => "z",
        "ʃ" => "ʒ",
        "ts" => "ʣ",
        "ʧ" => "ʤ",
        "x" => "h",
        _ => return None,
    })
}
fn is_voiceless(p: &str) -> bool {
    matches!(p, "p" | "t" | "k" | "s" | "ʃ" | "ts" | "ʧ" | "x" | "f")
}

/// Latin letters inside Ukrainian text: a rough reading by Cyrillic equivalents.
fn latin_to_cyr(c: char) -> &'static str {
    match c {
        'a' => "а", 'b' => "б", 'c' => "к", 'd' => "д", 'e' => "е", 'f' => "ф", 'g' => "ґ", 'h' => "г", 'i' => "і",
        'j' => "й", 'k' => "к", 'l' => "л", 'm' => "м", 'n' => "н", 'o' => "о", 'p' => "п", 'q' => "к", 'r' => "р",
        's' => "с", 't' => "т", 'u' => "у", 'v' => "в", 'w' => "в", 'x' => "кс", 'y' => "і", 'z' => "з",
        _ => "",
    }
}

/// Phonemize one lower-case word (no context).
pub fn word(lw: &str) -> String {
    let norm: String = lw
        .chars()
        .map(|c| match c {
            '’' | 'ʼ' | '`' => '\'',
            'ё' => 'е',
            'ы' => 'и',
            'э' => 'е',
            'ъ' => '\'',
            _ => c,
        })
        .collect();
    // hyphenated compounds: each part keeps its own stress (будь-який, по-перше)
    if norm.contains('-') {
        let parts: Vec<String> = norm.split('-').filter(|p| p.chars().any(char::is_alphabetic)).map(word).collect();
        return parts.concat();
    }
    let mut clean = String::with_capacity(norm.len());
    for c in norm.chars() {
        if is_cyr(c) || c == 'ь' || c == '\'' {
            clean.push(c);
        } else if c.is_ascii_alphabetic() {
            clean.push_str(latin_to_cyr(c));
        }
    }
    let clean = clean.trim_matches('\'').to_string();
    if clean.chars().all(|c| !is_cyr(c)) {
        return String::new();
    }
    if let Some(&(_, full)) = ABBREV.iter().find(|(a, _)| *a == clean) {
        return word(full);
    }
    let nv = clean.chars().filter(|&c| is_vowel_letter(c)).count();
    let st = stress_from_end(&clean, nv);
    let level = if nv == 1 && CLITICS.contains(&clean.as_str()) { 0 } else { 2 };
    realize(&clean, nv, st, level)
}

fn realize(lw: &str, nv: usize, st_from_end: usize, level: u8) -> String {
    let c: Vec<char> = lw.chars().collect();
    let n = c.len();
    let at = |i: usize| -> char { if i < n { c[i] } else { '#' } };
    let mut segs: Vec<Seg> = Vec::with_capacity(n + 4);
    // a soft sign or apostrophe stands between the last consonant and the next vowel
    let mut sep = false;
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let nx = at(i + 1);
        let prev_cons = !sep && segs.last().map_or(false, |s| s.k == K::Cons && s.p != "j");
        match ch {
            'а' => segs.push(vowel("a")),
            'о' => segs.push(vowel("o")),
            'у' => segs.push(vowel("u")),
            'е' => segs.push(vowel("ɛ")),
            'и' => segs.push(vowel("ɪ")),
            'і' => {
                if prev_cons {
                    let last = segs.last_mut().unwrap();
                    if softenable(last.p) {
                        last.soft = true;
                    } else {
                        last.semi = true;
                    }
                }
                segs.push(vowel("iː"));
            }
            'ї' => {
                segs.push(cons("j"));
                segs.push(vowel("iː"));
            }
            'я' | 'ю' | 'є' => {
                let v = match ch {
                    'я' => "a",
                    'ю' => "u",
                    _ => "ɛ",
                };
                if prev_cons {
                    let last = segs.last_mut().unwrap();
                    if softenable(last.p) {
                        last.soft = true;
                    } else if is_husher(last.p) {
                        last.semi = true;
                    } else {
                        last.jglide = true;
                    }
                } else {
                    segs.push(cons("j"));
                }
                segs.push(vowel(v));
            }
            'ь' => {
                if let Some(last) = segs.last_mut() {
                    if last.k == K::Cons && softenable(last.p) {
                        last.soft = true;
                    }
                }
                // ьо: soft consonant + o; ья/ью/ьє: soft consonant + j + vowel
                if matches!(nx, 'я' | 'ю' | 'є' | 'ї') {
                    sep = true;
                }
                i += 1;
                continue;
            }
            '\'' => {
                sep = true;
                i += 1;
                continue;
            }
            'б' => segs.push(cons("b")),
            'п' => segs.push(cons("p")),
            'в' => segs.push(cons("β")),
            'м' => segs.push(cons("m")),
            'ф' => segs.push(cons("f")),
            'д' => {
                if nx == 'з' {
                    segs.push(cons("ʣ"));
                    i += 1;
                } else if nx == 'ж' {
                    segs.push(cons("ʤ"));
                    i += 1;
                } else {
                    segs.push(cons("d"));
                }
            }
            'т' => segs.push(cons("t")),
            'з' => segs.push(cons("z")),
            'с' => segs.push(cons("s")),
            'ц' => segs.push(cons("ts")),
            'н' => segs.push(cons("n")),
            'л' => segs.push(cons("l")),
            'р' => segs.push(cons("r")),
            'г' => segs.push(cons("h")),
            'ґ' => segs.push(cons("ɡ")),
            'к' => segs.push(cons("k")),
            'х' => segs.push(cons("x")),
            'ж' => segs.push(cons("ʒ")),
            'ш' => segs.push(cons("ʃ")),
            'ч' => segs.push(cons("ʧ")),
            'щ' => {
                segs.push(cons("ʃ"));
                segs.push(cons("ʧ"));
            }
            'й' => segs.push(cons("j")),
            _ => {}
        }
        sep = false;
        i += 1;
    }
    // stress
    let vidx: Vec<usize> = segs.iter().enumerate().filter(|(_, s)| s.k == K::Vowel).map(|(i, _)| i).collect();
    if let Some(&ix) = vidx.get(nv.saturating_sub(1).saturating_sub(st_from_end)) {
        if level > 0 && !vidx.is_empty() {
            segs[ix].stressed = true;
        }
    }
    assimilate(&mut segs);
    render(&segs)
}

/// Regressive assimilations: softness through dental clusters, voicing before
/// voiced obstruents, sibilant clusters, and the reflexive -ться/-шся.
fn assimilate(segs: &mut Vec<Seg>) {
    let n = segs.len();
    // softness spreads leftwards from a soft or semi-soft consonant through dentals
    // (пісня, кістці, Дніпро, світ, свято)
    for i in (0..n.saturating_sub(1)).rev() {
        if segs[i].k == K::Cons && is_dental(segs[i].p) && segs[i + 1].k == K::Cons {
            let nx = &segs[i + 1];
            if nx.soft || nx.jglide || nx.semi {
                segs[i].soft = true;
            }
        }
    }
    // voicing before a voiced obstruent (боротьба, вокзал, якби); в does not trigger it
    for i in (0..n.saturating_sub(1)).rev() {
        if segs[i].k == K::Cons && segs[i + 1].k == K::Cons && is_voiced_obstruent(segs[i + 1].p) {
            if let Some(v) = voice(segs[i].p) {
                if segs[i].p == "t" {
                    segs[i].from_t = true;
                }
                segs[i].p = v;
            }
        }
    }
    // г devoices before a voiceless obstruent in легко, вогко, нігті, кігті, дьогтю
    for i in 0..n.saturating_sub(1) {
        if segs[i].p == "h" && segs[i + 1].k == K::Cons && matches!(segs[i + 1].p, "k" | "t") {
            segs[i].p = "x";
        }
    }
    // initial з- before a voiceless obstruent: с (зсунути, зцілити)
    if n >= 2 && segs[0].p == "z" && segs[1].k == K::Cons && is_voiceless(segs[1].p) {
        segs[0].p = "s";
    }
    // sibilant and dental clusters
    let mut i = 0;
    while i + 1 < segs.len() {
        let (a, b) = (segs[i].p, segs[i + 1].p);
        let bsoft = segs[i + 1].soft;
        if segs[i].k != K::Cons || segs[i + 1].k != K::Cons {
            i += 1;
            continue;
        }
        match (a, b) {
            // зш → жш, сш → шш, зж/сж → жж
            ("z", "ʃ") => segs[i].p = "ʒ",
            ("s", "ʃ") => segs[i].p = "ʃ",
            ("z" | "s", "ʒ") => segs[i].p = "ʒ",
            // зч → шч word-initially, жч elsewhere; сч → шч
            ("z", "ʧ") => segs[i].p = if i == 0 { "ʃ" } else { "ʒ" },
            ("s", "ʧ") => segs[i].p = "ʃ",
            // шся → сься, жся → зься, чся → цься
            ("ʃ", "s") if bsoft => {
                segs[i].p = "s";
                segs[i].soft = true;
            }
            ("ʒ", "s") if bsoft => {
                segs[i].p = "z";
                segs[i].soft = true;
            }
            // книжці, дошці, річці
            ("ʒ", "ts") if bsoft => {
                segs[i].p = "z";
                segs[i].soft = true;
            }
            ("ʃ", "ts") if bsoft => {
                segs[i].p = "s";
                segs[i].soft = true;
            }
            ("ʧ", "ts") if bsoft => {
                segs[i].p = "ts";
                segs[i].soft = true;
            }
            // тч → чч, тш → чш
            ("t", "ʧ") => segs[i].p = "ʧ",
            ("t", "ʃ") => segs[i].p = "ʧ",
            // дч → джч, дш → джш
            ("d", "ʧ" | "ʃ") => segs[i].p = "ʤ",
            // тс → ц (-ться, братський), тц → цц; дс → дзс, дц → дзц
            ("t", "s") => {
                segs[i].p = "ts";
                segs[i].long = bsoft && segs[i].soft;
                segs[i].soft = bsoft;
                segs.remove(i + 1);
                continue;
            }
            ("t", "ts") => segs[i].p = "ts",
            ("d", "s" | "ts") => segs[i].p = "ʣ",
            _ => {}
        }
        i += 1;
    }
    // стськ → ськ, стн → сн (шістнадцять), нтств → нств
    let mut i = 0;
    while i + 2 < segs.len() {
        if segs[i].k == K::Cons && segs[i + 1].p == "t" && !segs[i + 1].from_t && segs[i + 2].k == K::Cons {
            let drop = (matches!(segs[i].p, "s" | "n") && matches!(segs[i + 2].p, "s" | "n"))
                || (segs[i].p == "s" && segs[i + 2].p == "ts" && i + 3 < segs.len() && segs[i + 3].k == K::Cons);
            if drop {
                segs.remove(i + 1);
                continue;
            }
        }
        i += 1;
    }
}

fn render(segs: &[Seg]) -> String {
    let mut out = String::with_capacity(segs.len() * 3);
    for s in segs {
        if s.k == K::Vowel {
            if s.stressed {
                out.push('ˈ');
            }
            out.push_str(s.p);
            continue;
        }
        match (s.p, s.soft) {
            ("l", true) => out.push('ʎ'),
            ("n", true) => out.push('ɲ'),
            ("d", false) if s.from_t => out.push_str("d̪"),
            ("t", false) => out.push_str("t̪"),
            (p, true) => {
                out.push_str(p);
                out.push('ʲ');
            }
            (p, false) => out.push_str(p),
        }
        if s.long {
            out.push('ː');
        }
        if s.jglide {
            out.push('j');
        }
    }
    out
}

pub fn join(_outs: &mut [Out]) {}

const ONES: [&str; 20] = [
    "нуль", "один", "два", "три", "чотири", "п'ять", "шість", "сім", "вісім", "дев'ять", "десять", "одинадцять",
    "дванадцять", "тринадцять", "чотирнадцять", "п'ятнадцять", "шістнадцять", "сімнадцять", "вісімнадцять",
    "дев'ятнадцять",
];
const TENS: [&str; 10] = ["", "десять", "двадцять", "тридцять", "сорок", "п'ятдесят", "шістдесят", "сімдесят", "вісімдесят", "дев'яносто"];
const HUNDREDS: [&str; 10] = ["", "сто", "двісті", "триста", "чотириста", "п'ятсот", "шістсот", "сімсот", "вісімсот", "дев'ятсот"];

fn under_1000(n: u128, fem: bool, out: &mut Vec<String>) {
    let h = (n / 100) as usize;
    let r = n % 100;
    if h > 0 {
        out.push(HUNDREDS[h].into());
    }
    if r == 0 {
        return;
    }
    if r < 20 {
        let w = match (r, fem) {
            (1, true) => "одна",
            (2, true) => "дві",
            _ => ONES[r as usize],
        };
        out.push(w.into());
    } else {
        out.push(TENS[(r / 10) as usize].into());
        let u = (r % 10) as usize;
        if u > 0 {
            let w = match (u, fem) {
                (1, true) => "одна",
                (2, true) => "дві",
                _ => ONES[u],
            };
            out.push(w.into());
        }
    }
}

/// Form of a counted noun: 1 → nominative singular, 2–4 → nominative plural, else genitive plural.
fn count_form<'a>(n: u128, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let r100 = n % 100;
    let r10 = n % 10;
    if (11..=14).contains(&r100) {
        many
    } else if r10 == 1 {
        one
    } else if (2..=4).contains(&r10) {
        few
    } else {
        many
    }
}

/// Cardinal number words (nominative, masculine).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["нуль".into()];
    }
    let mut out = Vec::new();
    let scales: [(u128, &str, &str, &str, bool); 4] = [
        (1_000_000_000_000, "трильйон", "трильйони", "трильйонів", false),
        (1_000_000_000, "мільярд", "мільярди", "мільярдів", false),
        (1_000_000, "мільйон", "мільйони", "мільйонів", false),
        (1_000, "тисяча", "тисячі", "тисяч", true),
    ];
    let mut rest = n;
    for (unit, one, few, many, fem) in scales {
        let q = rest / unit;
        if q > 0 {
            under_1000(q, fem, &mut out);
            out.push(count_form(q, one, few, many).into());
            rest %= unit;
        }
    }
    if rest > 0 {
        under_1000(rest, false, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "відсотків",
        '$' => "доларів",
        '€' => "євро",
        '£' => "фунтів",
        '₴' => "гривень",
        '&' => "і",
        '+' => "плюс",
        '@' => "собака",
        '°' => "градусів",
        '=' => "дорівнює",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn letters_and_softening() {
        for (w, p) in [
            ("мама", "mˈama"),
            ("день", "dˈɛɲ"),
            ("сіль", "sʲˈiːʎ"),
            ("земля", "zɛmʎˈa"),
            ("п'ять", "pjˈatʲ"),
            ("м'ясо", "mjˈaso"),
            ("їжа", "jˈiːʒa"),
            ("україна", "ukrajˈiːna"),
            ("сьогодні", "sʲohˈodʲɲiː"),
            ("ґанок", "ɡˈanok"),
            ("щастя", "ʃʧˈasʲtʲa"),
            ("дзвінок", "ʣʲβˈiːnok"),
            ("джерело", "ʤɛrɛlˈo"),
            ("бджола", "bʤˈola"),
            ("життя", "ʒɪtʲtʲˈa"),
            ("знання", "znaɲɲˈa"),
            ("ательє", "at̪ɛʎjˈɛ"),
            ("мільйон", "miːʎjˈon"),
            ("дніпро", "dʲɲiːprˈo"),
            ("пісня", "pˈiːsʲɲa"),
            ("свято", "sʲβjˈat̪o"),
            ("світ", "sʲβˈiːt̪"),
            ("гривня", "hrˈɪβɲa"),
            ("обличчя", "oblˈɪʧʧa"),
            ("кінця", "kiːɲtsʲˈa"),
            ("хлопець", "xlˈopɛtsʲ"),
            ("львів", "ʎβˈiːβ"),
            ("київ", "kˈɪjiːβ"),
            ("їх", "jˈiːx"),
            ("що", "ʃʧˈo"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn assimilation() {
        for (w, p) in [
            ("боротьба", "borodʲbˈa"),
            ("вокзал", "βoɡzˈal"),
            ("якби", "jaɡbˈɪ"),
            ("футбол", "fud̪bˈol"),
            ("легко", "lˈɛxko"),
            ("просьба", "prˈozʲba"),
            ("сміється", "sʲmˈiːjɛtsʲːa"),
            ("дивишся", "dˈɪβɪsʲsʲa"),
            ("книжці", "knˈɪzʲtsʲiː"),
            ("відсотків", "βiːʣsˈotʲkiːβ"),
            ("розчарований", "roʒʧarˈoβanɪj"),
            ("зшити", "ʃʃˈɪt̪ɪ"),
            ("вовк", "βˈoβk"),
            ("ложка", "lˈoʒka"),
            ("шістнадцять", "ʃiːsnˈaʣʲtsʲatʲ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn stress_and_numbers() {
        for (w, p) in [
            ("в", "β"),
            ("на", "nˈa"),
            ("мені", "mɛɲˈiː"),
            ("місто", "mˈiːst̪o"),
            ("стаття", "st̪atʲtʲˈa"),
            ("тисяча", "t̪ˈɪsʲaʧa"),
            ("чотири", "ʧot̪ˈɪrɪ"),
            ("будь-який", "bˈudʲjakˈɪj"),
            ("нью-йорк", "ɲjˈujˈork"),
            ("вісімдесят", "βiːsʲiːmdɛsʲˈat̪"),
            ("дев'яносто", "dɛβjanˈost̪o"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
        assert_eq!(cardinal(21).join(" "), "двадцять один");
        assert_eq!(cardinal(2000).join(" "), "дві тисячі");
        assert_eq!(cardinal(5000).join(" "), "п'ять тисяч");
        assert_eq!(cardinal(1_000_000).join(" "), "один мільйон");
        assert_eq!(cardinal(3_105).join(" "), "три тисячі сто п'ять");
        assert_eq!(cardinal(12_000_000_000).join(" "), "дванадцять мільярдів");
    }
}
