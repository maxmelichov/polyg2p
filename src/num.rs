//! Number and symbol expansion to words, per language.
//!
//! German number words come back with `+` between their parts
//! (`ein+und+zwanzig`) so the German module can stress them as one compound.

use crate::Lang;

/// Parse a digit string with separators into (integer part, fractional digits).
/// en uses `,` for thousands and `.` for decimals; es/it/de the reverse. A single
/// separator followed by exactly three digits is read as a thousands separator.
fn parse(s: &str, lang: Lang) -> Option<(u128, Option<String>)> {
    let (thou, dec) = match lang {
        Lang::En => (',', '.'),
        _ => ('.', ','),
    };
    let mut int_part = String::new();
    let mut frac: Option<String> = None;
    let groups: Vec<&str> = s.split(|c| c == thou || c == '\u{202F}' || c == '\u{00A0}' || c == '\'').collect();
    let thousands_ok = groups.len() > 1 && groups[1..].iter().all(|g| {
        let g = g.split(dec).next().unwrap_or("");
        g.len() == 3
    });
    if groups.len() > 1 && !thousands_ok {
        // e.g. "3.5" in German text is a decimal written English-style
        let (a, b) = s.split_once(thou)?;
        if b.contains(thou) {
            return None;
        }
        return Some((a.parse().ok()?, Some(b.to_string())));
    }
    for (k, g) in groups.iter().enumerate() {
        if k + 1 == groups.len() {
            if let Some((a, b)) = g.split_once(dec) {
                int_part.push_str(a);
                frac = Some(b.chars().filter(|c| c.is_ascii_digit()).collect());
            } else {
                int_part.push_str(g);
            }
        } else {
            int_part.push_str(g);
        }
    }
    if int_part.len() > 30 {
        return None;
    }
    Some((int_part.parse().ok()?, frac))
}

/// Expand a numeric token into words, appended to `out`.
pub fn expand(tok: &str, lang: Lang, _prev: &str, next: &str, out: &mut Vec<String>) {
    if let Some((h, m)) = tok.split_once(':') {
        if let (Ok(h), Ok(m)) = (h.parse::<u128>(), m.parse::<u128>()) {
            out.extend(cardinal(h, lang));
            if m == 0 {
                if lang == Lang::En && next.is_empty() {
                    out.push("o'clock".into());
                } else if lang == Lang::De {
                    out.push("Uhr".into());
                }
            } else {
                match lang {
                    Lang::En if m < 10 => out.push("oh".into()),
                    Lang::Es => out.push("y".into()),
                    Lang::It => out.push("e".into()),
                    Lang::De => out.push("Uhr".into()),
                    _ => {}
                }
                out.extend(cardinal(m, lang));
            }
            return;
        }
    }
    let Some((n, frac)) = parse(tok, lang) else {
        for d in tok.chars().filter(|c| c.is_ascii_digit()) {
            out.extend(cardinal(d as u128 - '0' as u128, lang));
        }
        return;
    };
    let plain = !tok.contains(|c: char| !c.is_ascii_digit());
    if plain && tok.len() == 4 && frac.is_none() {
        if let Some(y) = year(n, lang) {
            out.extend(y);
            return;
        }
    }
    if plain && tok.len() > 1 && tok.starts_with('0') {
        // codes / phone numbers: digit by digit
        for d in tok.chars() {
            out.extend(cardinal(d as u128 - '0' as u128, lang));
        }
        return;
    }
    let mut words = cardinal(n, lang);
    if lang == Lang::Es && frac.is_none() && !next.is_empty() {
        // apocope before a noun: veintiún niños, un libro
        if let Some(last) = words.last_mut() {
            match last.as_str() {
                "uno" => *last = "un".into(),
                "veintiuno" => *last = "veintiún".into(),
                _ => {}
            }
        }
    }
    out.extend(words);
    if let Some(f) = frac {
        out.push(match lang {
            Lang::En => "point",
            Lang::Es => "coma",
            Lang::It => "virgola",
            Lang::De => "Komma",
            Lang::Ru => "запятая",
        }
        .to_string());
        if f.len() <= 2 && !f.starts_with('0') && lang != Lang::En {
            out.extend(cardinal(f.parse().unwrap_or(0), lang));
        } else {
            for d in f.chars() {
                out.extend(cardinal(d as u128 - '0' as u128, lang));
            }
        }
    }
}

/// Words for a spoken symbol.
pub fn symbol(c: char, lang: Lang) -> Option<&'static str> {
    Some(match (lang, c) {
        (Lang::En, '%') => "percent",
        (Lang::En, '&') => "and",
        (Lang::En, '+') => "plus",
        (Lang::En, '@') => "at",
        (Lang::En, '°') => "degrees",
        (Lang::En, '=') => "equals",
        (Lang::En, '$') => "dollars",
        (Lang::En, '€') => "euros",
        (Lang::En, '£') => "pounds",
        (Lang::En, '₪') => "shekels",
        (Lang::Es, '%') => "por ciento",
        (Lang::Es, '&') => "y",
        (Lang::Es, '+') => "más",
        (Lang::Es, '@') => "arroba",
        (Lang::Es, '°') => "grados",
        (Lang::Es, '=') => "igual",
        (Lang::Es, '$') => "dólares",
        (Lang::Es, '€') => "euros",
        (Lang::Es, '£') => "libras",
        (Lang::It, '%') => "per cento",
        (Lang::It, '&') => "e",
        (Lang::It, '+') => "più",
        (Lang::It, '@') => "chiocciola",
        (Lang::It, '°') => "gradi",
        (Lang::It, '=') => "uguale",
        (Lang::It, '$') => "dollari",
        (Lang::It, '€') => "euro",
        (Lang::It, '£') => "sterline",
        (Lang::De, '%') => "Prozent",
        (Lang::De, '&') => "und",
        (Lang::De, '+') => "plus",
        (Lang::De, '@') => "at",
        (Lang::De, '°') => "Grad",
        (Lang::De, '=') => "gleich",
        (Lang::De, '$') => "Dollar",
        (Lang::De, '€') => "Euro",
        (Lang::De, '£') => "Pfund",
        _ => return None,
    })
}

pub fn is_currency(c: char) -> bool {
    matches!(c, '$' | '€' | '£' | '₪' | '¥' | '₽')
}

fn year(n: u128, lang: Lang) -> Option<Vec<String>> {
    match lang {
        Lang::En if (1100..=1999).contains(&n) || (2010..=2099).contains(&n) => {
            let hi = n / 100;
            let lo = n % 100;
            let mut v = cardinal(hi, lang);
            if lo == 0 {
                v.push("hundred".into());
            } else if lo < 10 {
                v.push("oh".into());
                v.extend(cardinal(lo, lang));
            } else {
                v.extend(cardinal(lo, lang));
            }
            Some(v)
        }
        Lang::De if (1100..=1999).contains(&n) => {
            let hi = n / 100;
            let lo = n % 100;
            let mut s = de_below_1000(hi);
            s.push_str("+hundert");
            if lo > 0 {
                s.push('+');
                s.push_str(&de_below_1000(lo));
            }
            Some(vec![s])
        }
        _ => None,
    }
}

pub fn cardinal(n: u128, lang: Lang) -> Vec<String> {
    match lang {
        Lang::En => en_card(n),
        Lang::Es => es_card(n, true),
        Lang::It => it_card(n),
        Lang::De => de_card(n),
        // Russian text never reaches this module (ru.rs normalizes its own numbers)
        Lang::Ru => en_card(n),
    }
}

// ---------------------------------------------------------------- English
const EN_ONES: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
];
const EN_TENS: [&str; 10] = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];
const SCALES_EN: [&str; 5] = ["", "thousand", "million", "billion", "trillion"];

fn en_card(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["zero".into()];
    }
    let mut groups = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push((m % 1000) as u32);
        m /= 1000;
    }
    if groups.len() > SCALES_EN.len() {
        return n.to_string().chars().map(|d| EN_ONES[d as usize - '0' as usize].to_string()).collect();
    }
    let mut out = Vec::new();
    for (k, &g) in groups.iter().enumerate().rev() {
        if g == 0 {
            continue;
        }
        en_below_1000(g, &mut out);
        if k > 0 {
            out.push(SCALES_EN[k].into());
        }
    }
    out
}

fn en_below_1000(g: u32, out: &mut Vec<String>) {
    let h = g / 100;
    let r = g % 100;
    if h > 0 {
        out.push(EN_ONES[h as usize].into());
        out.push("hundred".into());
    }
    if r >= 20 {
        out.push(EN_TENS[(r / 10) as usize].into());
        if r % 10 > 0 {
            out.push(EN_ONES[(r % 10) as usize].into());
        }
    } else if r > 0 {
        out.push(EN_ONES[r as usize].into());
    }
}

// ---------------------------------------------------------------- Spanish
const ES_ONES: [&str; 30] = [
    "cero", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once", "doce",
    "trece", "catorce", "quince", "dieciséis", "diecisiete", "dieciocho", "diecinueve", "veinte", "veintiuno",
    "veintidós", "veintitrés", "veinticuatro", "veinticinco", "veintiséis", "veintisiete", "veintiocho",
    "veintinueve",
];
const ES_TENS: [&str; 10] =
    ["", "", "", "treinta", "cuarenta", "cincuenta", "sesenta", "setenta", "ochenta", "noventa"];
const ES_HUND: [&str; 10] = [
    "", "ciento", "doscientos", "trescientos", "cuatrocientos", "quinientos", "seiscientos", "setecientos",
    "ochocientos", "novecientos",
];

fn es_below_1000(g: u32, last: bool, out: &mut Vec<String>) {
    let h = g / 100;
    let r = g % 100;
    if g == 100 {
        out.push("cien".into());
        return;
    }
    if h > 0 {
        out.push(ES_HUND[h as usize].into());
    }
    if r == 0 {
        return;
    }
    let one = |s: &str| -> String {
        if !last {
            match s {
                "uno" => "un".into(),
                "veintiuno" => "veintiún".into(),
                _ => s.into(),
            }
        } else {
            s.into()
        }
    };
    if r < 30 {
        out.push(one(ES_ONES[r as usize]));
    } else {
        out.push(ES_TENS[(r / 10) as usize].into());
        if r % 10 > 0 {
            out.push("y".into());
            out.push(one(ES_ONES[(r % 10) as usize]));
        }
    }
}

fn es_card(n: u128, last: bool) -> Vec<String> {
    let mut out = Vec::new();
    if n == 0 {
        return vec!["cero".into()];
    }
    if n >= 1_000_000_000_000_000_000 {
        return n.to_string().chars().map(|d| ES_ONES[d as usize - '0' as usize].to_string()).collect();
    }
    let millions = n / 1_000_000;
    let rest = (n % 1_000_000) as u32;
    if millions > 0 {
        if millions == 1 {
            out.push("un".into());
            out.push("millón".into());
        } else {
            out.extend(es_card(millions, false));
            out.push("millones".into());
        }
    }
    let th = rest / 1000;
    let r = rest % 1000;
    if th > 0 {
        if th > 1 {
            es_below_1000(th, false, &mut out);
        }
        out.push("mil".into());
    }
    if r > 0 {
        es_below_1000(r, last, &mut out);
    }
    out
}

// ---------------------------------------------------------------- Italian
const IT_ONES: [&str; 20] = [
    "zero", "uno", "due", "tre", "quattro", "cinque", "sei", "sette", "otto", "nove", "dieci", "undici",
    "dodici", "tredici", "quattordici", "quindici", "sedici", "diciassette", "diciotto", "diciannove",
];
const IT_TENS: [&str; 10] =
    ["", "", "venti", "trenta", "quaranta", "cinquanta", "sessanta", "settanta", "ottanta", "novanta"];

fn it_below_100(r: u32) -> String {
    if r < 20 {
        return IT_ONES[r as usize].into();
    }
    let t = IT_TENS[(r / 10) as usize];
    let u = r % 10;
    match u {
        0 => t.into(),
        1 | 8 => format!("{}{}", &t[..t.len() - 1], IT_ONES[u as usize]),
        3 => format!("{t}tré"),
        _ => format!("{t}{}", IT_ONES[u as usize]),
    }
}

fn it_below_1000(g: u32) -> String {
    let h = g / 100;
    let r = g % 100;
    let mut s = String::new();
    if h == 1 {
        s.push_str("cento");
    } else if h > 1 {
        s.push_str(IT_ONES[h as usize]);
        s.push_str("cento");
    }
    if r > 0 {
        let tail = it_below_100(r);
        if h > 0 && (r / 10 == 8) {
            s.pop(); // cent'ottanta
        }
        s.push_str(&tail);
    }
    s
}

fn it_card(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["zero".into()];
    }
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| IT_ONES[d as usize - '0' as usize].to_string()).collect();
    }
    let mut out = Vec::new();
    let bil = (n / 1_000_000_000) as u32;
    let mil = ((n / 1_000_000) % 1000) as u32;
    let th = ((n / 1000) % 1000) as u32;
    let r = (n % 1000) as u32;
    if bil > 0 {
        if bil == 1 {
            out.extend(["un".into(), "miliardo".into()]);
        } else {
            out.push(it_below_1000(bil));
            out.push("miliardi".into());
        }
    }
    if mil > 0 {
        if mil == 1 {
            out.extend(["un".into(), "milione".into()]);
        } else {
            out.push(it_below_1000(mil));
            out.push("milioni".into());
        }
    }
    if th > 0 {
        if th == 1 {
            out.push("mille".into());
        } else {
            out.push(format!("{}mila", it_below_1000(th)));
        }
    }
    if r > 0 {
        out.push(it_below_1000(r));
    }
    out
}

// ---------------------------------------------------------------- German
const DE_ONES: [&str; 20] = [
    "null", "eins", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn", "elf", "zwölf",
    "dreizehn", "vierzehn", "fünfzehn", "sechzehn", "siebzehn", "achtzehn", "neunzehn",
];
const DE_TENS: [&str; 10] =
    ["", "", "zwanzig", "dreißig", "vierzig", "fünfzig", "sechzig", "siebzig", "achtzig", "neunzig"];

fn de_below_1000(g: u128) -> String {
    let g = g as u32;
    let h = g / 100;
    let r = g % 100;
    let mut parts: Vec<String> = Vec::new();
    if h > 0 {
        parts.push(if h == 1 { "ein".into() } else { DE_ONES[h as usize].into() });
        parts.push("hundert".into());
    }
    if r > 0 {
        if r < 20 {
            parts.push(if r == 1 && h > 0 { "eins".into() } else { DE_ONES[r as usize].into() });
        } else {
            let u = r % 10;
            if u > 0 {
                parts.push(if u == 1 { "ein".into() } else { DE_ONES[u as usize].into() });
                parts.push("und".into());
            }
            parts.push(DE_TENS[(r / 10) as usize].into());
        }
    }
    parts.join("+")
}

fn de_card(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["null".into()];
    }
    if n >= 1_000_000_000_000 {
        return n.to_string().chars().map(|d| DE_ONES[d as usize - '0' as usize].to_string()).collect();
    }
    let mut out = Vec::new();
    let bil = n / 1_000_000_000;
    let mil = (n / 1_000_000) % 1000;
    let th = (n / 1000) % 1000;
    let r = n % 1000;
    if bil > 0 {
        if bil == 1 {
            out.extend(["eine".into(), "Milliarde".into()]);
        } else {
            out.push(de_below_1000(bil));
            out.push("Milliarden".into());
        }
    }
    if mil > 0 {
        if mil == 1 {
            out.extend(["eine".into(), "Million".into()]);
        } else {
            out.push(de_below_1000(mil));
            out.push("Millionen".into());
        }
    }
    let mut tail = String::new();
    if th > 0 {
        let t = if th == 1 { "ein".to_string() } else { de_below_1000(th) };
        tail.push_str(&t);
        tail.push_str("+tausend");
    }
    if r > 0 {
        if !tail.is_empty() {
            tail.push('+');
        }
        tail.push_str(&de_below_1000(r));
    }
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn e(s: &str, l: Lang) -> String {
        let mut v = Vec::new();
        expand(s, l, "", "", &mut v);
        v.join(" ")
    }
    #[test]
    fn numbers() {
        assert_eq!(e("1945", Lang::En), "nineteen forty five");
        assert_eq!(e("2026", Lang::En), "twenty twenty six");
        assert_eq!(e("2005", Lang::En), "two thousand five");
        assert_eq!(e("1,234.5", Lang::En), "one thousand two hundred thirty four point five");
        assert_eq!(e("21", Lang::Es), "veintiuno");
        assert_eq!(e("1.000.000", Lang::Es), "un millón");
        assert_eq!(e("31", Lang::Es), "treinta y uno");
        assert_eq!(e("23", Lang::It), "ventitré");
        assert_eq!(e("188", Lang::It), "centottantotto");
        assert_eq!(e("2026", Lang::It), "duemila ventisei");
        assert_eq!(e("21", Lang::De), "ein+und+zwanzig");
        assert_eq!(e("1945", Lang::De), "neunzehn+hundert+fünf+und+vierzig");
        assert_eq!(e("3,5", Lang::De), "drei Komma fünf");
        assert_eq!(e("3:45", Lang::En), "three forty five");
        assert_eq!(e("14:05", Lang::De), "vierzehn Uhr fünf");
    }
}
