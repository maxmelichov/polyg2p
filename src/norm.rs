//! Text normalization shared by the Latin-script languages: abbreviations,
//! dates, decades and German ordinal dates.

use crate::Lang;

/// Abbreviations read in full when written with a period (`Dr.`, `z.B.`).
/// Keys are lower case; a two-part key holds its inner period (`z.b`).
pub fn abbrev(lang: Lang, key: &str) -> Option<&'static str> {
    let table: &[(&str, &str)] = match lang {
        Lang::De => &[
            ("dr", "Doktor"), ("prof", "Professor"), ("hr", "Herr"), ("hrn", "Herrn"), ("fr", "Frau"), ("frl", "Fräulein"),
            ("nr", "Nummer"), ("str", "Straße"), ("st", "Sankt"), ("ca", "circa"), ("bzw", "beziehungsweise"),
            ("usw", "und so weiter"), ("etc", "et cetera"), ("evtl", "eventuell"), ("ggf", "gegebenenfalls"),
            ("vgl", "vergleiche"), ("inkl", "inklusive"), ("exkl", "exklusive"), ("zzgl", "zuzüglich"),
            ("mio", "Millionen"), ("mrd", "Milliarden"), ("tsd", "Tausend"), ("jh", "Jahrhundert"), ("jhd", "Jahrhundert"),
            ("tel", "Telefon"), ("abs", "Absatz"), ("abb", "Abbildung"), ("bd", "Band"), ("bsp", "Beispiel"),
            ("geb", "geboren"), ("gest", "gestorben"), ("max", "maximal"), ("min", "mindestens"), ("mind", "mindestens"),
            ("sog", "sogenannt"), ("sogn", "sogenannt"), ("urspr", "ursprünglich"), ("ehem", "ehemalig"),
            ("allg", "allgemein"), ("bes", "besonders"), ("bzgl", "bezüglich"), ("insb", "insbesondere"),
            ("z.b", "zum Beispiel"), ("d.h", "das heißt"), ("u.a", "unter anderem"), ("o.ä", "oder ähnliches"),
            ("u.ä", "und ähnliches"), ("i.d.r", "in der Regel"), ("u.u", "unter Umständen"), ("z.t", "zum Teil"),
            ("z.zt", "zur Zeit"), ("s.o", "siehe oben"), ("s.u", "siehe unten"), ("v.a", "vor allem"),
            ("m.e", "meines Erachtens"), ("i.a", "im Allgemeinen"), ("a.d", "an der"), ("n.chr", "nach Christus"),
            ("v.chr", "vor Christus"), ("st.", "Sankt"), ("hbf", "Hauptbahnhof"), ("gmbh", "GmbH"),
        ],
        Lang::Es => &[
            ("sr", "señor"), ("sra", "señora"), ("srta", "señorita"), ("sres", "señores"), ("dr", "doctor"),
            ("dra", "doctora"), ("ud", "usted"), ("uds", "ustedes"), ("vd", "usted"), ("vds", "ustedes"),
            ("d", "don"), ("dña", "doña"), ("prof", "profesor"), ("profa", "profesora"), ("lic", "licenciado"),
            ("ing", "ingeniero"), ("av", "avenida"), ("avda", "avenida"), ("pág", "página"), ("págs", "páginas"),
            ("pag", "página"), ("núm", "número"), ("num", "número"), ("tel", "teléfono"), ("etc", "etcétera"),
            ("ej", "ejemplo"), ("p.ej", "por ejemplo"), ("aprox", "aproximadamente"), ("cía", "compañía"),
            ("sta", "santa"), ("sto", "santo"), ("gral", "general"), ("cap", "capítulo"), ("art", "artículo"),
            ("ee.uu", "Estados Unidos"), ("a.c", "antes de Cristo"), ("d.c", "después de Cristo"),
            ("vol", "volumen"), ("máx", "máximo"), ("mín", "mínimo"), ("kg", "kilogramos"), ("km", "kilómetros"),
        ],
        Lang::It => &[
            ("dott", "dottor"), ("dr", "dottor"), ("dott.ssa", "dottoressa"), ("dr.ssa", "dottoressa"),
            ("sig", "signor"), ("sigg", "signori"), ("sig.ra", "signora"), ("sig.na", "signorina"),
            ("prof", "professor"), ("prof.ssa", "professoressa"), ("ing", "ingegner"), ("avv", "avvocato"),
            ("arch", "architetto"), ("geom", "geometra"), ("rag", "ragionier"), ("on", "onorevole"),
            ("ecc", "eccetera"), ("etc", "eccetera"), ("es", "esempio"), ("p.es", "per esempio"), ("ad es", "ad esempio"),
            ("n", "numero"), ("pag", "pagina"), ("pagg", "pagine"), ("p", "pagina"), ("pp", "pagine"), ("sec", "secolo"),
            ("ca", "circa"), ("cfr", "confronta"), ("v", "vedi"), ("vol", "volume"), ("art", "articolo"),
            ("tel", "telefono"), ("s", "san"), ("sant", "sant'"), ("cap", "capitolo"), ("a.c", "avanti Cristo"),
            ("d.c", "dopo Cristo"), ("c.a", "circa"), ("spa", "S p A"), ("srl", "S r l"), ("ass", "associazione"),
            ("max", "massimo"), ("min", "minimo"), ("km", "chilometri"), ("kg", "chilogrammi"),
        ],
        Lang::Pt => &[
            ("sr", "senhor"), ("sra", "senhora"), ("srta", "senhorita"), ("dr", "doutor"), ("dra", "doutora"),
            ("prof", "professor"), ("profa", "professora"), ("av", "avenida"), ("pág", "página"), ("etc", "etcétera"),
            ("tel", "telefone"), ("ex", "exemplo"), ("p.ex", "por exemplo"), ("n", "número"), ("nº", "número"),
        ],
        Lang::Fr => &[
            ("m", "monsieur"), ("mm", "messieurs"), ("mme", "madame"), ("mmes", "mesdames"), ("mlle", "mademoiselle"),
            ("dr", "docteur"), ("pr", "professeur"), ("av", "avenue"), ("bd", "boulevard"), ("bvd", "boulevard"),
            ("etc", "et cetera"), ("ex", "exemple"), ("p.ex", "par exemple"), ("tél", "téléphone"), ("env", "environ"),
            ("st", "saint"), ("ste", "sainte"), ("n", "numéro"), ("nº", "numéro"), ("c.-à-d", "c'est-à-dire"),
            ("cf", "confer"), ("vol", "volume"), ("chap", "chapitre"), ("art", "article"), ("réf", "référence"),
            ("j.-c", "Jésus-Christ"),
        ],
        _ => return None,
    };
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// Abbreviations a sentence can end with (etc.).
pub fn sentence_final(key: &str) -> bool {
    matches!(key, "etc" | "usw" | "ecc" | "u.ä" | "o.ä" | "u.v.m")
}

/// Titles and other abbreviations normally followed by a capitalized name.
pub fn is_title(lang: Lang, key: &str) -> bool {
    match lang {
        Lang::Es => matches!(key, "sr" | "sra" | "srta" | "sres" | "dr" | "dra" | "d" | "dña" | "prof" | "profa" | "lic" | "ing" | "av" | "avda" | "sta" | "sto" | "gral" | "cía"),
        Lang::It => matches!(key, "dott" | "dr" | "dott.ssa" | "dr.ssa" | "sig" | "sigg" | "sig.ra" | "sig.na" | "prof" | "prof.ssa" | "ing" | "avv" | "arch" | "geom" | "rag" | "on" | "s" | "sant"),
        Lang::Pt => matches!(key, "sr" | "sra" | "srta" | "dr" | "dra" | "prof" | "profa" | "av"),
        Lang::Fr => matches!(key, "m" | "mm" | "mme" | "mmes" | "mlle" | "dr" | "pr" | "av" | "bd" | "bvd" | "st" | "ste"),
        _ => false,
    }
}

pub fn month_name(lang: Lang, m: u32) -> Option<&'static str> {
    const EN: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    const DE: [&str; 12] = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];
    const ES: [&str; 12] = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];
    const IT: [&str; 12] = ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"];
    const PT: [&str; 12] = ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"];
    const FR: [&str; 12] = ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
    if !(1..=12).contains(&m) {
        return None;
    }
    let t = match lang {
        Lang::En => &EN,
        Lang::De => &DE,
        Lang::Es => &ES,
        Lang::It => &IT,
        Lang::Pt => &PT,
        Lang::Fr => &FR,
        _ => return None,
    };
    Some(t[m as usize - 1])
}

/// Is `w` a month name of `lang` (lower case)?
pub fn is_month(lang: Lang, w: &str) -> bool {
    (1..=12).any(|m| month_name(lang, m).is_some_and(|n| n.eq_ignore_ascii_case(w) || n.to_lowercase() == w))
}

/// German ordinal number word: dritte / dritten (after am, vom, zum, seit, bis …).
pub fn de_ordinal(n: u128, dative: bool) -> String {
    let base = match n {
        1 => "erste".to_string(),
        3 => "dritte".to_string(),
        7 => "siebte".to_string(),
        8 => "achte".to_string(),
        _ => {
            let c = crate::num::cardinal(n, Lang::De).join("");
            if n < 20 { format!("{c}te") } else { format!("{c}ste") }
        }
    };
    if dative { format!("{base}n") } else { base }
}

/// Words after which a German date ordinal takes the dative/accusative -n.
pub fn de_dative(prev: &str) -> bool {
    matches!(prev, "am" | "vom" | "zum" | "seit" | "bis" | "ab" | "dem" | "den" | "des" | "nach" | "vor" | "im" | "bis zum")
}

/// `12/03/2021` (en: month/day/year, others day/month/year) and `3.10.2021`.
pub fn date(tok: &str, lang: Lang, prev: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = tok.split(['/', '.', '-']).collect();
    if parts.len() != 3 || !parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    if parts[0].len() > 2 || parts[1].len() > 2 || !(parts[2].len() == 4 || parts[2].len() == 2) {
        return None;
    }
    let a: u32 = parts[0].parse().ok()?;
    let b: u32 = parts[1].parse().ok()?;
    let y: u128 = parts[2].parse().ok()?;
    let (day, month) = if lang == Lang::En { (b, a) } else { (a, b) };
    let (day, month) = if month > 12 && day <= 12 { (month, day) } else { (day, month) };
    if !(1..=31).contains(&day) || !(1..=12).contains(&month) {
        return None;
    }
    let mon = month_name(lang, month)?;
    let y = if parts[2].len() == 2 { if y < 50 { 2000 + y } else { 1900 + y } } else { y };
    let mut out = Vec::new();
    let year = |out: &mut Vec<String>| {
        let mut ys = Vec::new();
        crate::num::expand(&y.to_string(), lang, "", "", &mut ys);
        out.extend(ys);
    };
    match lang {
        Lang::En => {
            out.push(mon.to_string());
            let mut d = crate::num::cardinal(day as u128, lang);
            crate::num::en_ordinal(&mut d);
            out.extend(d);
            year(&mut out);
        }
        Lang::De => {
            out.push(de_ordinal(day as u128, de_dative(prev) || prev.is_empty()));
            out.push(mon.to_string());
            year(&mut out);
        }
        Lang::Es => {
            if day == 1 { out.push("primero".into()) } else { out.extend(crate::num::cardinal(day as u128, lang)) }
            out.push("de".into());
            out.push(mon.to_string());
            out.push("de".into());
            year(&mut out);
        }
        Lang::It => {
            if day == 1 { out.push("primo".into()) } else { out.extend(crate::num::cardinal(day as u128, lang)) }
            out.push(mon.to_string());
            year(&mut out);
        }
        Lang::Pt => {
            if day == 1 { out.push("primeiro".into()) } else { out.extend(crate::num::cardinal(day as u128, lang)) }
            out.push("de".into());
            out.push(mon.to_string());
            out.push("de".into());
            year(&mut out);
        }
        Lang::Fr => {
            if day == 1 { out.push("premier".into()) } else { out.extend(crate::num::cardinal(day as u128, lang)) }
            out.push(mon.to_string());
            year(&mut out);
        }
        _ => return None,
    }
    Some(out)
}

/// `1990s` → nineteen nineties (English); other languages read the number.
pub fn decade(tok: &str, lang: Lang) -> Option<Vec<String>> {
    let digits = tok.strip_suffix(['s', 'S'])?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut words = Vec::new();
    crate::num::expand(digits, lang, "", "", &mut words);
    if lang == Lang::En {
        if let Some(last) = words.last_mut() {
            *last = match last.as_str() {
                w if w.ends_with('y') => format!("{}ies", &w[..w.len() - 1]),
                w => format!("{w}s"),
            };
        }
    }
    Some(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_decades() {
        assert_eq!(date("12/03/2021", Lang::En, "").unwrap().join(" "), "december third twenty twenty one");
        assert_eq!(date("12/03/2021", Lang::Es, "").unwrap().join(" "), "doce de marzo de dos mil veintiuno");
        assert_eq!(date("3.10.2021", Lang::De, "am").unwrap().join(" "), "dritten Oktober zwei+tausend+ein+und+zwanzig");
        assert_eq!(date("1/5/21", Lang::It, "").unwrap().join(" "), "primo maggio duemila ventuno");
        assert!(date("1/2", Lang::En, "").is_none());
        assert_eq!(decade("1990s", Lang::En).unwrap().join(" "), "nineteen nineties");
        assert_eq!(decade("2000s", Lang::En).unwrap().join(" "), "two thousands");
        assert_eq!(de_ordinal(1, false), "erste");
        assert_eq!(de_ordinal(21, true), "ein+und+zwanzigsten");
        assert_eq!(abbrev(Lang::De, "z.b"), Some("zum Beispiel"));
        assert_eq!(abbrev(Lang::It, "sig.ra"), Some("signora"));
        assert_eq!(abbrev(Lang::Pl, "dr"), None);
    }
}
