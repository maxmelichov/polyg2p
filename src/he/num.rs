//! Hebrew number front end: a port of `hebrew-num2words` 0.1 (the package
//! RenikudPlus runs before its network, since digits never occur in its
//! training text). Same rules, same outputs: counting gender agrees with the
//! following noun, `2` before a noun takes the construct form, bare numerals
//! take the feminine series (1 -> אחד), hh:mm is 12-hour feminine, N% is
//! masculine + אחוז, D.M.YYYY is day + ב+month + feminine year, identifiers are
//! read digit by digit.
//!
//! `normalize(text)` is a no-op on digit-free text; otherwise it re-joins the
//! whitespace tokens with single spaces, exactly like the Python driver.

const MASC_UNITS: [&str; 11] = ["אפס", "אחד", "שניים", "שלושה", "ארבעה", "חמישה", "שישה", "שבעה", "שמונה", "תשעה", "עשרה"];
const FEM_UNITS: [&str; 11] = ["אפס", "אחת", "שתיים", "שלוש", "ארבע", "חמש", "שש", "שבע", "שמונה", "תשע", "עשר"];
const MASC_TEENS: [&str; 9] =
    ["אחד עשר", "שנים עשר", "שלושה עשר", "ארבעה עשר", "חמישה עשר", "שישה עשר", "שבעה עשר", "שמונה עשר", "תשעה עשר"];
const FEM_TEENS: [&str; 9] =
    ["אחת עשרה", "שתים עשרה", "שלוש עשרה", "ארבע עשרה", "חמש עשרה", "שש עשרה", "שבע עשרה", "שמונה עשרה", "תשע עשרה"];
const TENS: [&str; 10] = ["", "", "עשרים", "שלושים", "ארבעים", "חמישים", "שישים", "שבעים", "שמונים", "תשעים"];
const HUNDREDS: [&str; 10] =
    ["", "מאה", "מאתיים", "שלוש מאות", "ארבע מאות", "חמש מאות", "שש מאות", "שבע מאות", "שמונה מאות", "תשע מאות"];
const ORD_MASC: [&str; 11] = ["", "ראשון", "שני", "שלישי", "רביעי", "חמישי", "שישי", "שביעי", "שמיני", "תשיעי", "עשירי"];
const ORD_FEM: [&str; 11] = ["", "ראשונה", "שנייה", "שלישית", "רביעית", "חמישית", "שישית", "שביעית", "שמינית", "תשיעית", "עשירית"];
const MONTHS: [&str; 13] =
    ["", "ינואר", "פברואר", "מרץ", "אפריל", "מאי", "יוני", "יולי", "אוגוסט", "ספטמבר", "אוקטובר", "נובמבר", "דצמבר"];
const FRACTIONS: &[(u64, u64, &str)] = &[
    (1, 2, "חצי"), (1, 3, "שליש"), (1, 4, "רבע"), (2, 3, "שני שלישים"), (3, 4, "שלושת רבעי"), (1, 5, "חמישית"),
    (2, 5, "שתי חמישיות"), (3, 5, "שלוש חמישיות"), (4, 5, "ארבע חמישיות"), (1, 6, "שישית"), (1, 8, "שמינית"),
    (3, 8, "שלוש שמיניות"), (1, 10, "עשירית"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gender {
    M,
    F,
}
use Gender::{F, M};

fn masc_construct(n: u64) -> Option<&'static str> {
    Some(match n {
        2 => "שני", 3 => "שלושת", 4 => "ארבעת", 5 => "חמשת", 6 => "ששת", 7 => "שבעת", 8 => "שמונת", 9 => "תשעת", 10 => "עשרת",
        _ => return None,
    })
}

fn units(n: u64, g: Gender) -> &'static str {
    if g == M { MASC_UNITS[n as usize] } else { FEM_UNITS[n as usize] }
}

fn thousands_words(k: u64) -> Vec<String> {
    if k == 1 {
        return vec!["אלף".into()];
    }
    if k == 2 {
        return vec!["אלפיים".into()];
    }
    if let Some(c) = masc_construct(k) {
        return vec![c.into(), "אלפים".into()];
    }
    let mut v: Vec<String> = cardinal(k, M, false).split(' ').map(String::from).collect();
    v.push("אלף".into());
    v
}

fn sub_thousand_groups(mut n: u64, g: Gender) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    if n >= 100 {
        groups.push(HUNDREDS[(n / 100) as usize].split(' ').map(String::from).collect());
        n %= 100;
    }
    if n >= 20 {
        let mut tail = vec![TENS[(n / 10) as usize].to_string()];
        if n % 10 != 0 {
            tail.push(format!("ו{}", units(n % 10, g)));
        }
        groups.push(tail);
    } else if n >= 11 {
        let t = if g == M { MASC_TEENS[(n - 11) as usize] } else { FEM_TEENS[(n - 11) as usize] };
        groups.push(t.split(' ').map(String::from).collect());
    } else if n >= 1 {
        groups.push(vec![units(n, g).to_string()]);
    }
    groups
}

/// Hebrew cardinal for 0 .. 999,999 (`construct`: שני/שתי and שלושת … עשרת).
pub fn cardinal(n: u64, g: Gender, construct: bool) -> String {
    if n == 0 {
        return "אפס".into();
    }
    if construct {
        if g == M {
            if let Some(c) = masc_construct(n) {
                return c.into();
            }
        } else if n == 2 {
            return "שתי".into();
        }
    }
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut n = n;
    if n >= 1000 {
        groups.push(thousands_words(n / 1000));
        n %= 1000;
    }
    groups.extend(sub_thousand_groups(n, g));
    if groups.len() > 1 && !groups.last().unwrap().iter().any(|w| w.starts_with('ו')) {
        let last = groups.last_mut().unwrap();
        last[0] = format!("ו{}", last[0]);
    }
    groups.iter().flatten().cloned().collect::<Vec<_>>().join(" ")
}

pub fn ordinal(n: u64, g: Gender, definite: bool) -> String {
    let word = if (1..=10).contains(&n) {
        (if g == M { ORD_MASC } else { ORD_FEM })[n as usize].to_string()
    } else {
        cardinal(n, g, false)
    };
    if definite { format!("ה{word}") } else { word }
}

pub fn spell_digits(digits: &str) -> String {
    digits.chars().filter_map(|c| c.to_digit(10)).map(|d| FEM_UNITS[d as usize]).collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------- noun gender
const FEM_IM_PLURALS: &[&str] = &[
    "שנים", "נשים", "מילים", "ביצים", "דבורים", "חיטים", "תאנים", "פעמים", "אבנים", "עצים", "יונים", "צפרניים", "שעורים",
];
const MASC_OT_PLURALS: &[&str] = &[
    "שולחנות", "כיסאות", "מקומות", "לילות", "קולות", "חלומות", "רחובות", "אבות", "שמות", "לוחות", "קירות", "מטבעות",
    "שבועות", "רעיונות", "חלונות", "זנבות", "מזלגות", "סכינים", "אריות", "נכסים",
];
const NOUN_GENDER: &[(&str, Gender)] = &[
    ("שעה", F), ("שעות", F), ("דקה", F), ("דקות", F), ("שניה", F), ("שניות", F), ("שנה", F), ("שנות", F), ("מנה", F),
    ("מנות", F), ("כוס", F), ("כוסות", F), ("קומה", F), ("קומות", F), ("פעם", F), ("פעמים", F), ("בת", F), ("בנות", F),
    ("מעלה", F), ("מעלות", F), ("טיסה", F), ("טיסות", F), ("אישה", F), ("נשים", F), ("דירה", F), ("דירות", F),
    ("עונה", F), ("עונות", F), ("שורה", F), ("שורות", F), ("חנות", F), ("חנויות", F), ("מסעדה", F), ("מסעדות", F),
    ("פגישה", F), ("פגישות", F), ("יחידה", F), ("יחידות", F), ("משימה", F), ("משימות", F), ("מחברת", F), ("מחברות", F),
    ("מעטפה", F), ("מעטפות", F), ("חולצה", F), ("חולצות", F), ("מיטה", F), ("מיטות", F), ("דלת", F), ("דלתות", F),
    ("רגל", F), ("רגליים", F), ("יד", F), ("ידיים", F), ("עין", F), ("עיניים", F), ("אוזן", F), ("אוזניים", F),
    ("עיר", F), ("ערים", F), ("ארץ", F), ("דרך", F), ("נפש", F), ("כיתה", F), ("כיתות", F),
    ("שקל", M), ("שקלים", M), ("ש\"ח", M), ("אגורה", F), ("אגורות", F), ("דולר", M), ("דולרים", M), ("יורו", M),
    ("אירו", M), ("יום", M), ("ימים", M), ("ימי", M), ("חודש", M), ("חודשים", M), ("חודשי", M), ("שבוע", M),
    ("שבועות", M), ("איש", M), ("אנשים", M), ("ילד", M), ("ילדים", M), ("בן", M), ("בנים", M), ("חבר", M), ("חברים", M),
    ("עובד", M), ("עובדים", M), ("ספר", M), ("ספרים", M), ("כלב", M), ("כלבים", M), ("חתול", M), ("חתולים", M),
    ("תפוח", M), ("תפוחים", M), ("כרטיס", M), ("כרטיסים", M), ("מטר", M), ("מטרים", M), ("קילומטר", M), ("קילו", M),
    ("ליטר", M), ("מיליון", M), ("מיליארד", M), ("אלף", M), ("אחוז", M), ("אחוזים", M), ("מוצר", M), ("מוצרים", M),
    ("משתתף", M), ("משתתפים", M), ("בניין", M), ("בניינים", M), ("משרד", M), ("משרדים", M), ("סטודנט", M),
    ("סטודנטים", M), ("דייר", M), ("דיירים", M), ("עט", M), ("עטים", M), ("מכתב", M), ("מכתבים", M), ("כדור", M),
    ("כדורים", M), ("נרשם", M), ("נרשמים", M), ("פועל", M), ("פועלים", M), ("ק\"ג", M), ("ק\"מ", M), ("מ\"ר", M),
    ("קמ\"ש", M), ("מ\"מ", M), ("ס\"מ", M),
];

fn noun_lex(w: &str) -> Option<Gender> {
    NOUN_GENDER.iter().find(|(k, _)| *k == w).map(|(_, g)| *g)
}

fn nchars(s: &str) -> usize {
    s.chars().count()
}

/// `^(?:[בכלמשו]?ה|[בכלמשוה])` stripped once, when the word is long enough.
fn strip_prefix(word: &str) -> &str {
    if nchars(word) > 3 {
        let cs: Vec<(usize, char)> = word.char_indices().collect();
        let cut = if cs.len() >= 2 && "בכלמשו".contains(cs[0].1) && cs[1].1 == 'ה' {
            Some(cs[2].0)
        } else if cs[0].1 == 'ה' || "בכלמשוה".contains(cs[0].1) {
            Some(cs[1].0)
        } else {
            None
        };
        if let Some(c) = cut {
            let s = &word[c..];
            if nchars(s) >= 2 {
                return s;
            }
        }
    }
    word
}

const EDGE: &[char] = &['.', ',', '!', '?', ';', ':', '"', '\'', '(', ')', '[', ']', '—', '–', '-'];

pub fn noun_gender(word: &str) -> Option<Gender> {
    let word = word.trim_matches(EDGE);
    if word.is_empty() {
        return None;
    }
    for cand in [word, strip_prefix(word)] {
        if let Some(g) = noun_lex(cand) {
            return Some(g);
        }
        if FEM_IM_PLURALS.contains(&cand) {
            return Some(F);
        }
        if MASC_OT_PLURALS.contains(&cand) {
            return Some(M);
        }
    }
    let stem = strip_prefix(word);
    if nchars(stem) < 3 {
        return None;
    }
    if stem.ends_with("יות") || stem.ends_with("ות") {
        return Some(F);
    }
    if stem.ends_with("ים") || stem.ends_with("יים") {
        return Some(M);
    }
    None
}

const CODE_NOUNS: &[&str] = &[
    "קוד", "מספר", "מיקוד", "סיסמה", "סיסמא", "סיומת", "ברקוד", "טלפון", "שלוחה", "סניף", "מזהה", "כרטיס", "אשראי",
    "פוליסה", "הזמנה", "משלוח", "זהות", "ת.ז", "ת\"ז", "רישיון", "אימות", "פנימי", "סודי", "סידורי", "חשאי", "אישי",
    "בנק", "גישה", "ז", "מנעול",
];
const ORDINAL_NOUNS: &[&str] =
    &["קומה", "פעם", "מקום", "פרק", "חלק", "עונה", "טסט", "שורה", "שלב", "כיתה", "רבעון", "סיבוב", "מחזור"];
const HOUR_CUES: &[&str] = &["שעה", "השעה", "בשעה", "לשעה", "שעון", "השעון"];
const DAY_OF_MONTH_CUES: &[&str] = &["לחודש", "בחודש", "החודש"];
const YEAR_CUES: &[&str] = &["שנת", "בשנת", "משנת", "לשנת", "שנה", "השנה"];
const PUNCT_TAIL: &[char] = &['.', ',', '!', '?', ';', ':', '"', '\'', ')', ']', '…'];
const PUNCT_TAIL_DASH: &[char] = &['.', ',', '!', '?', ';', ':', '"', '\'', ')', ']', '…', '-'];

fn hour_word(h: u64) -> String {
    let mut h = h % 24;
    if h == 0 {
        h = 12;
    }
    if h > 12 {
        h -= 12;
    }
    cardinal(h, F, false)
}

pub fn expand_time(hh: u64, mm: u64) -> String {
    let hour = hour_word(hh);
    match mm {
        0 => hour,
        15 => format!("{hour} ורבע"),
        30 => format!("{hour} וחצי"),
        45 => format!("רבע ל{}", hour_word(hh + 1)),
        m if m < 30 => format!("{hour} ו{}", cardinal(m, M, false)),
        m => format!("{} ל{}", cardinal(60 - m, M, false), hour_word(hh + 1)),
    }
}

pub fn expand_date(day: u64, month: u64, year: Option<u64>) -> String {
    let mut out = format!("{} ב{}", cardinal(day, M, false), MONTHS[month as usize]);
    if let Some(y) = year {
        out.push(' ');
        out.push_str(&cardinal(y, F, false));
    }
    out
}

fn decimal_words(ip_s: &str, fp: &str, g: Gender) -> String {
    let ip: u64 = ip_s.parse().unwrap_or(0);
    if fp == "5" {
        if ip == 0 {
            return "חצי".into();
        }
        return format!("{} וחצי", cardinal(ip, g, false));
    }
    if nchars(fp) == 2 {
        let f: u64 = fp.parse().unwrap_or(0);
        return format!("{} {}", cardinal(ip, if ip != 1 { F } else { M }, false), cardinal(f, F, false));
    }
    let head = cardinal(ip, if ip == 1 { M } else { F }, false);
    format!("{head} נקודה {}", spell_digits(fp))
}

fn grouped_digits(groups: &[&str]) -> String {
    if !groups.is_empty() && (groups[0] == "1" || groups[0] == "*") && groups.len() > 1 && nchars(groups[1]) == 3 {
        let mut out = vec![if groups[0] == "1" { MASC_UNITS[1].to_string() } else { spell_digits(groups[0]) }];
        for g in &groups[1..] {
            out.push(cardinal(g.parse().unwrap_or(0), F, false));
        }
        return out.join(" ");
    }
    groups.iter().map(|g| spell_digits(g)).collect::<Vec<_>>().join(" ")
}

fn in_set(w: &str, set: &[&str]) -> bool {
    set.contains(&w) || set.contains(&strip_prefix(w))
}

fn prev_base<'a>(prev: &[&'a str], k: usize) -> &'a str {
    if prev.len() < k { "" } else { prev[prev.len() - k].trim_matches(PUNCT_TAIL_DASH) }
}

fn is_code_context(prev: &[&str]) -> bool {
    let mut prev = prev;
    for i in (0..prev.len()).rev() {
        if prev[i].chars().any(|c| c.is_ascii_digit()) {
            prev = &prev[i + 1..];
            break;
        }
    }
    let start = prev.len().saturating_sub(3);
    prev[start..].iter().any(|w| in_set(w.trim_matches(PUNCT_TAIL_DASH), CODE_NOUNS))
}

fn month_follows(next: &[&str]) -> bool {
    let Some(w) = next.first() else { return false };
    let w = w.trim_matches(PUNCT_TAIL);
    if DAY_OF_MONTH_CUES.contains(&w) {
        return true;
    }
    let stripped = w.strip_prefix(['ב', 'ל', 'מ', 'ה']).unwrap_or(w);
    MONTHS[1..].contains(&w) || MONTHS[1..].contains(&stripped)
}

fn next_noun_gender(next: &[&str]) -> Option<Gender> {
    next.first().and_then(|w| noun_gender(w))
}

fn is_year_cue(w: &str) -> bool {
    YEAR_CUES.contains(&w)
}

fn expand_integer(digits: &str, pre: &str, prev: &[&str], next: &[&str]) -> String {
    let n: u64 = digits.parse().unwrap_or(0);
    let nd = nchars(digits);
    if nd >= 5 || (nd >= 3 && is_code_context(prev)) || (nd > 1 && digits.starts_with('0')) {
        return spell_digits(digits);
    }
    if is_year_cue(prev_base(prev, 1)) {
        return cardinal(n, F, false);
    }
    if month_follows(next) && (1..=31).contains(&n) {
        return cardinal(n, M, false);
    }
    let p = prev_base(prev, 1);
    let p_stripped = if ORDINAL_NOUNS.contains(&p) { p } else { strip_prefix(p) };
    if pre.ends_with('ה') && n <= 999 {
        let g = noun_lex(p_stripped).or_else(|| noun_gender(p)).unwrap_or(M);
        return ordinal(n, g, false);
    }
    if ORDINAL_NOUNS.contains(&p_stripped) && n <= 10 {
        return ordinal(n, noun_lex(p_stripped).unwrap_or(M), false);
    }
    if let Some(g) = next_noun_gender(next) {
        if n == 2 {
            return cardinal(2, g, true);
        }
        return cardinal(n, g, false);
    }
    if (pre == "ב" || pre == "מ" || pre == "ל")
        || p == "עד"
        || p == "ועד"
        || HOUR_CUES.contains(&p)
        || HOUR_CUES.contains(&prev_base(prev, 2))
    {
        if n <= 24 && !is_code_context(prev) {
            return hour_word(n);
        }
    }
    cardinal(n, if n == 1 { M } else { F }, false)
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// `\d{1,3}(?:,\d{3})+`
fn is_grouped_thousands(s: &str) -> bool {
    let parts: Vec<&str> = s.split(',').collect();
    parts.len() >= 2
        && (1..=3).contains(&parts[0].len())
        && all_digits(parts[0])
        && parts[1..].iter().all(|p| p.len() == 3 && all_digits(p))
}

fn replace_digit_runs(s: &str) -> String {
    let mut out = String::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i].is_ascii_digit() {
            let st = i;
            while i < cs.len() && cs[i].is_ascii_digit() {
                i += 1;
            }
            let run: String = cs[st..i].iter().collect();
            out.push_str(&cardinal(run.parse().unwrap_or(0), F, false));
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

/// Expand the numeric core of a token. `None` = unsupported (the Python raises
/// and the token passes through unchanged).
fn expand_body(body: &str, pre: &str, prev: &[&str], next: &[&str]) -> Option<String> {
    if let Some(core) = body.strip_suffix('%') {
        let mut core = core.to_string();
        if is_grouped_thousands(&core) {
            core = core.replace(',', "");
        }
        let tail = match next.first().map(|w| w.trim_matches(PUNCT_TAIL)) {
            Some("אחוז") | Some("אחוזים") => "",
            _ => " אחוז",
        };
        if let Some((ip, fp)) = core.split_once('.') {
            return Some(decimal_words(ip, fp, M) + tail);
        }
        return Some(cardinal(core.parse().ok()?, M, false) + tail);
    }
    // clock
    if let Some((h, m)) = body.split_once(':') {
        if (1..=2).contains(&h.len()) && all_digits(h) && m.len() == 2 && all_digits(m) {
            return Some(expand_time(h.parse().ok()?, m.parse().ok()?));
        }
    }
    // fraction / date with slash
    if let Some((a, b)) = body.split_once('/') {
        if (1..=2).contains(&a.len()) && all_digits(a) && (1..=4).contains(&b.len()) && all_digits(b) {
            let (a, b): (u64, u64) = (a.parse().ok()?, b.parse().ok()?);
            if let Some((_, _, w)) = FRACTIONS.iter().find(|(x, y, _)| *x == a && *y == b) {
                return Some(w.to_string());
            }
            if (1..=12).contains(&b) && (1..=31).contains(&a) {
                return Some(expand_date(a, b, None));
            }
            return Some(format!("{} {}", cardinal(a, F, false), cardinal(b, F, false)));
        }
    }
    // phone / serial with hyphens
    if body.contains('-') {
        let groups: Vec<&str> = body.split('-').collect();
        if groups.iter().all(|g| all_digits(g)) && groups.len() >= 2 {
            if groups.len() >= 3 || groups.iter().any(|g| g.len() >= 5) {
                return Some(grouped_digits(&groups));
            }
            if groups.iter().all(|g| g.len() <= 2) {
                return Some(groups.iter().map(|g| cardinal(g.parse().unwrap_or(0), F, false)).collect::<Vec<_>>().join(" "));
            }
            return Some(grouped_digits(&groups));
        }
    }
    // thousands separators
    if is_grouped_thousands(body) {
        let n: u64 = body.replace(',', "").parse().ok()?;
        let g = next_noun_gender(next);
        if n >= 1_000_000 {
            let head = n / 1_000_000;
            let rest = n % 1_000_000;
            let word = if head == 1 { "מיליון".to_string() } else { format!("{} מיליון", cardinal(head, M, true)) };
            return Some(if rest == 0 { word } else { format!("{word} {}", cardinal(rest, g.unwrap_or(M), false)) });
        }
        return Some(cardinal(n, g.unwrap_or(M), false));
    }
    // dates D.M.YYYY and M.YYYY
    let dots: Vec<&str> = body.split('.').collect();
    if dots.len() == 3
        && (1..=2).contains(&dots[0].len())
        && (1..=2).contains(&dots[1].len())
        && dots[2].len() == 4
        && dots.iter().all(|p| all_digits(p))
    {
        let (d, mo, y): (u64, u64, u64) = (dots[0].parse().ok()?, dots[1].parse().ok()?, dots[2].parse().ok()?);
        if (1..=31).contains(&d) && (1..=12).contains(&mo) {
            return Some(expand_date(d, mo, Some(y)));
        }
    }
    if dots.len() == 2 && (1..=2).contains(&dots[0].len()) && dots[1].len() == 4 && dots.iter().all(|p| all_digits(p)) {
        let m: u64 = dots[0].parse().ok()?;
        if (1..=12).contains(&m) {
            return Some(format!("ב{} {}", MONTHS[m as usize], cardinal(dots[1].parse().ok()?, F, false)));
        }
    }
    // one decimal point
    if dots.len() == 2 && all_digits(dots[0]) && all_digits(dots[1]) {
        let (ip_s, fp_s) = (dots[0], dots[1]);
        let (ip, fp): (u64, u64) = (ip_s.parse().ok()?, fp_s.parse().ok()?);
        let clock_cue = HOUR_CUES.contains(&prev_base(prev, 1));
        let unit_follows = next.first().is_some_and(|w| noun_gender(w).is_some());
        if !clock_cue
            && (pre == "ב" || pre == "ל" || pre == "ה")
            && fp_s.len() <= 2
            && (1..=12).contains(&fp)
            && (1..=31).contains(&ip)
            && !unit_follows
        {
            return Some(expand_date(ip, fp, None));
        }
        if clock_cue && fp_s.len() == 2 {
            return Some(expand_time(ip, fp));
        }
        let g = next_noun_gender(next);
        return Some(decimal_words(ip_s, fp_s, g.unwrap_or(F)));
    }
    if all_digits(body) {
        return Some(expand_integer(body, pre, prev, next));
    }
    Some(replace_digit_runs(body))
}

fn is_pre_char(c: char) -> bool {
    ('\u{0590}'..='\u{05EA}').contains(&c) || c == '"' || c == '\''
}

fn is_body_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | ',' | ':' | '/' | '%' | '-')
}

/// Match `^(pre){0,4}(-?)(body)(post)$` the way the Python regex does.
fn split_token(tok: &str) -> Option<(String, String, String)> {
    let cs: Vec<char> = tok.chars().collect();
    let mut i = 0;
    while i < cs.len() && i < 4 && is_pre_char(cs[i]) {
        i += 1;
    }
    let pre: String = cs[..i].iter().collect();
    if i < cs.len() && cs[i] == '-' {
        i += 1;
    }
    if i >= cs.len() || !cs[i].is_ascii_digit() {
        return None;
    }
    // body is lazy: the shortest prefix such that the rest is all post chars
    let rest = &cs[i..];
    let mut j = 1;
    while j < rest.len() && !rest[j..].iter().all(|c| PUNCT_TAIL.contains(c)) {
        j += 1;
    }
    if !rest[..j].iter().all(|&c| is_body_char(c)) {
        return None;
    }
    Some((pre, rest[..j].iter().collect(), rest[j..].iter().collect()))
}

pub fn expand_token(tok: &str, prev: &[&str], next: &[&str]) -> String {
    if !tok.chars().any(|c| c.is_ascii_digit()) {
        return tok.to_string();
    }
    let Some((pre, body, post)) = split_token(tok) else {
        return replace_digit_runs(tok);
    };
    match expand_body(&body, &pre, prev, next) {
        Some(words) => format!("{pre}{words}{post}"),
        None => tok.to_string(),
    }
}

pub fn contains_digit(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_digit())
}

/// Expand every numeric expression; digit-free text is returned unchanged.
pub fn normalize(text: &str) -> String {
    if !contains_digit(text) {
        return text.to_string();
    }
    let toks: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::with_capacity(toks.len());
    for (i, t) in toks.iter().enumerate() {
        if contains_digit(t) {
            out.push(expand_token(t, &toks[..i], &toks[i + 1..]));
        } else {
            out.push(t.to_string());
        }
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_python_reference() {
        let s = "ב-3 במאי 2021, 25% ממצב 3.5 ק\"מ ו-1,250 שקלים בשעה 14:30 ה-11 קומה 3 טלפון 050-1234567 1/2 12.3.2021 2/3 מיליון";
        let want = "בשלושה במאי אלפיים עשרים ואחת, עשרים וחמישה אחוז ממצב שלושה וחצי ק\"מ ואלף מאתיים וחמישים שקלים בשעה שתיים וחצי האחד עשר קומה שלישית טלפון אפס חמש אפס אחת שתיים שלוש ארבע חמש שש שבע חצי שנים עשר במרץ אלפיים עשרים ואחת שני שלישים מיליון";
        assert_eq!(normalize(s), want);
        assert_eq!(normalize("המחיר 1250 שקלים"), "המחיר אלף מאתיים וחמישים שקלים");
        assert_eq!(cardinal(2, F, true), "שתי");
        assert_eq!(cardinal(3000, M, false), "שלושת אלפים");
        assert_eq!(normalize("אין ספרות"), "אין ספרות");
    }
}
