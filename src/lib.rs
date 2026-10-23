//! polyg2p: fast multilingual grapheme-to-phoneme conversion for TTS.
//!
//! Output follows espeak-ng's IPA conventions (as normalized by BlueTTS) so it is
//! a drop-in replacement for models trained on espeak phonemes, while fixing the
//! pronunciations espeak gets wrong. Apache-2.0; contains no espeak code or data.

pub mod de;
pub mod en;
pub mod es;
pub mod fold;
pub mod he;
pub mod it;
pub mod lexicon;
pub mod nn;
pub mod num;
pub mod ru;
pub mod spell;
pub mod text;
pub mod fr;
pub mod pt;
pub mod pl;
pub mod uk;
pub mod tr;
pub mod nl;
pub mod cs;
pub mod el;
pub mod ro;
pub mod hu;
pub mod fi;
pub mod sv;
pub mod id;
pub mod ca;
pub mod ar;


use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use text::Tok;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Es,
    It,
    De,
    Ru,
    He,
    Fr,
    Pt,
    Pl,
    Uk,
    Tr,
    Nl,
    Cs,
    El,
    Ro,
    Hu,
    Fi,
    Sv,
    Id,
    Ca,
    Ar,
}

impl Lang {
    pub fn parse(s: &str) -> Option<Lang> {
        match s.to_ascii_lowercase().as_str() {
            "en" | "en-us" | "en_us" | "eng" => Some(Lang::En),
            "es" | "es-es" | "spa" => Some(Lang::Es),
            "it" | "it-it" | "ita" => Some(Lang::It),
            "de" | "ge" | "de-de" | "deu" | "ger" => Some(Lang::De),
            "ru" | "ru-ru" | "rus" => Some(Lang::Ru),
            "he" | "heb" | "iw" | "he-il" => Some(Lang::He),
            "fr" | "fr-fr" | "fra" | "fre" => Some(Lang::Fr),
            "pt" | "pt-br" | "por" => Some(Lang::Pt),
            "pl" | "pol" => Some(Lang::Pl),
            "uk" | "ukr" => Some(Lang::Uk),
            "tr" | "tur" => Some(Lang::Tr),
            "nl" | "nld" | "dut" => Some(Lang::Nl),
            "cs" | "ces" | "cze" => Some(Lang::Cs),
            "el" | "ell" | "gre" => Some(Lang::El),
            "ro" | "ron" | "rum" => Some(Lang::Ro),
            "hu" | "hun" => Some(Lang::Hu),
            "fi" | "fin" => Some(Lang::Fi),
            "sv" | "swe" => Some(Lang::Sv),
            "id" | "ind" => Some(Lang::Id),
            "ca" | "cat" => Some(Lang::Ca),
            "ar" | "ara" => Some(Lang::Ar),
            _ => None,
        }
    }
}

impl Lang {
    /// Every language code the crate knows, in a stable order.
    pub const ALL: &[Lang] = &[Lang::En, Lang::Es, Lang::It, Lang::De, Lang::Ru, Lang::He, Lang::Fr, Lang::Pt, Lang::Pl, Lang::Uk, Lang::Tr, Lang::Nl, Lang::Cs, Lang::El, Lang::Ro, Lang::Hu, Lang::Fi, Lang::Sv, Lang::Id, Lang::Ca, Lang::Ar];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
            Lang::It => "it",
            Lang::De => "de",
            Lang::Ru => "ru",
            Lang::He => "he",
            Lang::Fr => "fr",
            Lang::Pt => "pt",
            Lang::Pl => "pl",
            Lang::Uk => "uk",
            Lang::Tr => "tr",
            Lang::Nl => "nl",
            Lang::Cs => "cs",
            Lang::El => "el",
            Lang::Ro => "ro",
            Lang::Hu => "hu",
            Lang::Fi => "fi",
            Lang::Sv => "sv",
            Lang::Id => "id",
            Lang::Ca => "ca",
            Lang::Ar => "ar",
        }
    }

    /// Does this build have a working engine for the language (stubs return false)?
    pub fn implemented(self) -> bool {
        match self {
            Lang::En | Lang::Es | Lang::It | Lang::De | Lang::Ru => true,
            Lang::He => he::IMPLEMENTED,
            Lang::Fr => fr::IMPLEMENTED,
            Lang::Pt => pt::IMPLEMENTED,
            Lang::Pl => pl::IMPLEMENTED,
            Lang::Uk => uk::IMPLEMENTED,
            Lang::Tr => tr::IMPLEMENTED,
            Lang::Nl => nl::IMPLEMENTED,
            Lang::Cs => cs::IMPLEMENTED,
            Lang::El => el::IMPLEMENTED,
            Lang::Ro => ro::IMPLEMENTED,
            Lang::Hu => hu::IMPLEMENTED,
            Lang::Fi => fi::IMPLEMENTED,
            Lang::Sv => sv::IMPLEMENTED,
            Lang::Id => id::IMPLEMENTED,
            Lang::Ca => ca::IMPLEMENTED,
            Lang::Ar => ar::IMPLEMENTED,
        }
    }
}

/// One output word: its phonemes plus the punctuation that follows it.
#[derive(Debug, Clone, Default)]
pub struct Out {
    pub ph: String,
    pub punct: String,
    /// a pause (punctuation) follows: blocks cross-word assimilation
    pub pause_after: bool,
    /// source word (lower-case), for context rules
    pub src: String,
}

/// The G2P engine. Language resources load lazily on first use.
pub struct G2p {
    data_dir: Option<PathBuf>,
    en: OnceLock<en::English>,
    de: OnceLock<de::German>,
    it: OnceLock<it::Italian>,
    ru: OnceLock<ru::Russian>,
    he: OnceLock<he::Hebrew>,
}

impl G2p {
    /// `data_dir` holds optional lexicon/model files (`en.lex`, `en.g2p`, `de.lex`, ...).
    /// Without it, each language falls back to its built-in resources and rules.
    pub fn new(data_dir: Option<&Path>) -> Self {
        G2p { data_dir: data_dir.map(|p| p.to_path_buf()), en: OnceLock::new(), de: OnceLock::new(), it: OnceLock::new(), ru: OnceLock::new(), he: OnceLock::new() }
    }

    fn file(&self, name: &str) -> Option<PathBuf> {
        let p = self.data_dir.as_ref()?.join(name);
        p.exists().then_some(p)
    }

    pub fn english(&self) -> &en::English {
        self.en.get_or_init(|| {
            // POLYG2P_EN_MODEL=large picks the bigger, slower, more accurate OOV model
            let large = std::env::var("POLYG2P_EN_MODEL").is_ok_and(|v| v == "large");
            let model = if large { self.file("en-large.g2p").or_else(|| self.file("en.g2p")) } else { self.file("en.g2p") };
            en::English::load(self.file("en.lex").as_deref(), model.as_deref())
        })
    }
    pub fn german(&self) -> &de::German {
        self.de.get_or_init(|| de::German::load(self.file("de.lex").as_deref(), self.file("de.g2p").as_deref()))
    }
    pub fn italian(&self) -> &it::Italian {
        self.it.get_or_init(|| it::Italian::load(self.file("it.lex").as_deref()))
    }

    pub fn russian(&self) -> &ru::Russian {
        self.ru.get_or_init(|| {
            // Latin-script words inside Russian text go to the English engine (not espeak)
            let en = std::sync::Arc::new(en::English::load(self.file("en.lex").as_deref(), self.file("en.g2p").as_deref()));
            ru::Russian::load(self.data_dir.as_deref()).with_latin(move |w| {
                let mut outs = Vec::new();
                let p = en.word(w, &outs);
                outs.clear();
                p
            })
        })
    }

    pub fn hebrew(&self) -> &he::Hebrew {
        self.he.get_or_init(|| he::Hebrew::load(self.data_dir.as_deref()))
    }

    /// Phonemize running text.
    pub fn phonemize(&self, text: &str, lang: Lang) -> String {
        if lang == Lang::Ru {
            return self.russian().phonemize(text);
        }
        if lang == Lang::He {
            return self.hebrew().phonemize(text);
        }
        let toks = text::tokenize(text);
        let mut outs: Vec<Out> = Vec::with_capacity(toks.len());
        let mut words: Vec<String> = Vec::new();
        let mut skip = usize::MAX;
        for (ti, t) in toks.iter().enumerate() {
            if ti == skip {
                continue;
            }
            match t {
                Tok::Word(w) => self.push_word(&mut outs, w, lang),
                Tok::Sym(c) if num::is_currency(*c) && matches!(toks.get(ti + 1), Some(Tok::Num(_))) => {
                    let Some(Tok::Num(n)) = toks.get(ti + 1) else { unreachable!() };
                    words.clear();
                    num::currency(*c, n, lang, &mut words);
                    for w in words.drain(..) {
                        self.push_word(&mut outs, &w, lang);
                    }
                    skip = ti + 1;
                }
                Tok::Num(n) => {
                    // look at the next word for context (years, units, currency suffixes)
                    let next = toks[ti + 1..].iter().find_map(|t| match t {
                        Tok::Word(w) => Some(w.as_str()),
                        Tok::Sym(_) | Tok::Punct(_) => Some(""),
                        _ => None,
                    });
                    let prev = outs.last().map(|o| o.src.as_str()).unwrap_or("");
                    let prev = prev.to_string();
                    words.clear();
                    num::expand(n, lang, &prev, next.unwrap_or(""), &mut words);
                    for w in words.drain(..) {
                        self.push_word(&mut outs, &w, lang);
                    }
                }
                Tok::Sym(c) => {
                    if let Some(ws) = num::symbol(*c, lang) {
                        for w in ws.split(' ') {
                            self.push_word(&mut outs, w, lang);
                        }
                    }
                }
                Tok::Punct(p) => {
                    if let Some(o) = outs.last_mut() {
                        if !o.punct.is_empty() && *p == '.' && o.punct.ends_with('.') {
                            continue;
                        }
                        o.punct.push(*p);
                        o.pause_after = true;
                    }
                }
                Tok::Sep => {}
            }
        }
        match lang {
            Lang::Es => es::join(&mut outs),
            Lang::En => self.english().join(&mut outs),
            Lang::It => it::join(&mut outs),
            Lang::De | Lang::Ru | Lang::He => {}
            Lang::Fr => fr::join(&mut outs),
            Lang::Pt => pt::join(&mut outs),
            Lang::Pl => pl::join(&mut outs),
            Lang::Uk => uk::join(&mut outs),
            Lang::Tr => tr::join(&mut outs),
            Lang::Nl => nl::join(&mut outs),
            Lang::Cs => cs::join(&mut outs),
            Lang::El => el::join(&mut outs),
            Lang::Ro => ro::join(&mut outs),
            Lang::Hu => hu::join(&mut outs),
            Lang::Fi => fi::join(&mut outs),
            Lang::Sv => sv::join(&mut outs),
            Lang::Id => id::join(&mut outs),
            Lang::Ca => ca::join(&mut outs),
            Lang::Ar => ar::join(&mut outs),
        }
        let mut s = String::with_capacity(text.len() * 2);
        for o in &outs {
            if o.ph.is_empty() {
                if !o.punct.is_empty() {
                    s.push_str(&o.punct);
                }
                continue;
            }
            if !s.is_empty() && !o.src.is_empty() && !o.ph.starts_with('\u{0}') {
                s.push(' ');
            }
            s.push_str(o.ph.trim_start_matches('\u{0}'));
            s.push_str(&o.punct);
        }
        fold::fold(s.trim())
    }

    /// Phonemize many texts using all available cores (order preserved).
    pub fn phonemize_batch(&self, texts: &[String], lang: Lang) -> Vec<String> {
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        if threads <= 1 || texts.len() < 64 {
            return texts.iter().map(|t| self.phonemize(t, lang)).collect();
        }
        // warm the language resources once before fanning out
        let _ = self.phonemize("a", lang);
        let chunk = texts.len().div_ceil(threads);
        let mut out: Vec<Vec<String>> = Vec::new();
        std::thread::scope(|s| {
            let handles: Vec<_> = texts
                .chunks(chunk)
                .map(|c| s.spawn(move || c.iter().map(|t| self.phonemize(t, lang)).collect::<Vec<_>>()))
                .collect();
            out = handles.into_iter().map(|h| h.join().expect("worker panicked")).collect();
        });
        out.into_iter().flatten().collect()
    }

    fn push_word(&self, outs: &mut Vec<Out>, w: &str, lang: Lang) {
        let lw = w.to_lowercase();
        if lang != Lang::En && spell::is_acronym(w) {
            if let Some(ph) = spell::spell(w, lang) {
                outs.push(Out { ph, punct: String::new(), pause_after: false, src: lw });
                return;
            }
        }
        let ph = match lang {
            Lang::Es => es::word(&lw),
            Lang::En => self.english().word(w, outs),
            Lang::It => self.italian().word(&lw),
            Lang::De => self.german().word(w),
            Lang::Ru => self.russian().phonemize(w),
            Lang::He => self.hebrew().phonemize(w),
            Lang::Fr => fr::word(&lw),
            Lang::Pt => pt::word(&lw),
            Lang::Pl => pl::word(&lw),
            Lang::Uk => uk::word(&lw),
            Lang::Tr => tr::word(&lw),
            Lang::Nl => nl::word(&lw),
            Lang::Cs => cs::word(&lw),
            Lang::El => el::word(&lw),
            Lang::Ro => ro::word(&lw),
            Lang::Hu => hu::word(&lw),
            Lang::Fi => fi::word(&lw),
            Lang::Sv => sv::word(&lw),
            Lang::Id => id::word(&lw),
            Lang::Ca => ca::word(&lw),
            Lang::Ar => ar::word(&lw),
        };
        outs.push(Out { ph, punct: String::new(), pause_after: false, src: lw });
    }
}

/// Process-wide default engine (no external data directory).
pub fn default_engine() -> &'static G2p {
    static E: OnceLock<G2p> = OnceLock::new();
    E.get_or_init(|| G2p::new(std::env::var_os("POLYG2P_DATA").as_deref().map(Path::new)))
}
