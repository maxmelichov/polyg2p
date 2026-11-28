//! English (General American), espeak-style notation.
//!
//! Pronunciations come from CMUdict (BSD) converted to espeak's symbol
//! conventions (`en.lex`), unstressed function-word forms, context rules for
//! homographs, and a small neural model (`en.g2p`) for out-of-vocabulary words.

use std::collections::HashMap;
use std::path::Path;

use crate::homograph;
use crate::lexicon::Lexicon;
use crate::nn::Model;
use crate::Out;

const FUNC_TSV: &str = include_str!("../data/en_func.tsv");
const MERGE_TSV: &str = include_str!("../data/en_merge.tsv");
const HOMOGRAPH_TSV: &str = include_str!("../data/en_homograph.tsv");
/// A sentence needs this many words before the heteronym model reads it; shorter inputs (word lists, `--words`) keep the context rules.
const HG_MIN_WORDS: usize = 3;

const LETTERS: [&str; 26] = [
    "eɪ", "biː", "siː", "diː", "iː", "ɛf", "dʒiː", "eɪtʃ", "aɪ", "dʒeɪ", "keɪ", "ɛl", "ɛm", "ɛn", "oʊ", "piː",
    "kjuː", "ɑːɹ", "ɛs", "tiː", "juː", "viː", "dˈʌbəljuː", "ɛks", "waɪ", "ziː",
];

const DET: &[&str] = &[
    "the", "a", "an", "this", "that", "these", "those", "my", "your", "his", "her", "our", "their", "its",
    "every", "each", "no", "some", "any", "one", "first", "last", "new", "big", "of", "for", "in", "on",
    "with", "without", "good", "bad", "great", "small", "whose", "such", "another", "other",
];
const VERBAL: &[&str] = &[
    "to", "will", "would", "can", "could", "shall", "should", "may", "might", "must", "do", "does", "did",
    "don't", "doesn't", "didn't", "won't", "can't", "cannot", "wouldn't", "couldn't", "shouldn't", "i", "you",
    "we", "they", "he", "she", "please", "not", "never", "also", "always", "often", "usually", "let's",
    "i'll", "we'll", "you'll", "they'll", "who", "and",
];
const PERFECT: &[&str] = &[
    "have", "has", "had", "having", "was", "were", "been", "being", "is", "are", "be", "i've", "you've",
    "we've", "they've", "already", "just", "i'd", "we'd", "they'd", "you'd", "he'd", "she'd", "hasn't",
    "haven't", "hadn't", "wasn't", "weren't",
];

// "read": base form /ɹiːd/ or past /ɹɛd/, decided inside the word's clause (between pauses and
// and / but / or / so). 1. the nearest auxiliary just before it (have / was -> past, will / did / to -> base);
// 2. a bare third-person subject ("he read" is past); 3. the nearest time word in the clause.
const READ_BASE: &[&str] = &[
    "will", "would", "can", "could", "shall", "should", "may", "might", "must", "do", "does", "did", "don't",
    "doesn't", "didn't", "won't", "can't", "cannot", "wouldn't", "couldn't", "shouldn't", "to", "i'll", "we'll",
    "you'll", "they'll", "he'll", "she'll", "it'll", "let's", "please", "gonna", "wanna", "better", "rather",
];
const READ_PERFECT: &[&str] = &[
    "have", "has", "had", "having", "was", "were", "been", "being", "is", "are", "be", "am", "i've", "you've",
    "we've", "they've", "already", "just", "hasn't", "haven't", "hadn't", "wasn't", "weren't", "isn't", "aren't",
    "i'd", "we'd", "you'd", "they'd", "he'd", "she'd",
];
const READ_NOUN_DET: &[&str] = &["a", "an", "the", "good", "great", "quick", "easy", "light", "long", "fun", "interesting", "another"];
const READ_SUBJ: &[&str] = &["i", "you", "we", "they"];
const READ_WH: &[&str] = &["why", "what", "when", "where", "how", "which", "who", "whom", "whose", "not"];
const READ_THIRD: &[&str] = &["he", "she", "it", "who", "everyone", "everybody", "someone", "somebody", "nobody", "one"];
const READ_STOP: &[&str] = &["and", "but", "or", "so", "while", "because", "then"];
const READ_PAST: &[&str] = &[
    "yesterday", "ago", "earlier", "previously", "recently", "last", "once", "afterwards", "later", "before",
    "was", "were", "had", "did", "finally", "eventually", "formerly",
];
const READ_PRESENT: &[&str] = &[
    "now", "every", "always", "usually", "often", "sometimes", "daily", "tomorrow", "tonight", "next",
    "currently", "nowadays", "generally", "normally", "rarely", "seldom", "never", "weekly", "regularly",
];
const READ_THIS_PAST: &[&str] = &[
    "morning", "afternoon", "evening", "week", "weekend", "month", "year", "summer", "spring", "autumn", "fall", "winter",
];

fn read_is_past_form(w: &str) -> bool {
    READ_IRREGULAR_PAST.contains(&w) || (w.len() > 4 && w.ends_with("ed") && !w.ends_with("eed"))
}

fn read_form(words: &[(&str, bool)], k: usize) -> &'static str {
    let n = words.len();
    let start = (0..k).rev().find(|&j| words[j].1).map_or(0, |j| j + 1);
    let end = (k..n).find(|&j| words[j].1).map_or(n, |j| j + 1);
    // the clause: stop at a conjunction on either side
    let lo = (start..k).rev().find(|&j| READ_STOP.contains(&words[j].0)).map_or(start, |j| j + 1);
    let hi = (k + 1..end).find(|&j| READ_STOP.contains(&words[j].0)).unwrap_or(end);
    let prev = if k > lo { words[k - 1].0 } else { "" };
    let clause_final = words[k].1 || k + 1 >= end || ["for", "of"].contains(&words[k + 1].0);
    // the noun: "a read", "a good read", "a disappointing read."
    if READ_NOUN_DET.contains(&prev)
        || (k >= lo + 2 && ["a", "an", "the", "another"].contains(&words[k - 2].0) && !READ_SUBJ.contains(&prev) && clause_final)
    {
        return "ɹˈiːd";
    }
    // "who read" / "he read": the subject right before it, narrative past
    if READ_THIRD.contains(&prev) {
        return "ɹˈɛd";
    }
    // "the parent read his paper": a singular noun subject; the present would be "reads"
    if k >= lo + 2
        && ["the", "a", "an", "this", "that", "my", "your", "his", "her", "our", "their", "its"].contains(&words[k - 2].0)
        && !READ_SUBJ.contains(&prev)
        && !prev.ends_with('s')
        && !READ_NOUN_DET.contains(&prev)
        && !READ_BASE.contains(&prev)
        && !READ_PERFECT.contains(&prev)
        && !(k >= lo + 3 && READ_BASE.contains(&words[k - 3].0))
        && !["children", "people", "men", "women", "police", "folk", "staff", "team", "family", "better", "never", "often", "also", "only", "still"].contains(&prev)
    {
        return "ɹˈɛd";
    }
    // the auxiliary between subject and verb decides (have / was -> past, will / did / to -> base);
    // after "I / you / we / they" only an inverted auxiliary at the clause start counts ("did you read")
    let scan: Vec<usize> = if READ_SUBJ.contains(&prev) {
        if k >= lo + 2 && (k - 2 == lo || READ_WH.contains(&words[k - 3].0)) { vec![k - 2] } else { vec![] }
    } else {
        (lo..k).rev().take(4).collect()
    };
    for j in scan {
        let w = words[j].0;
        if READ_PERFECT.contains(&w) {
            return "ɹˈɛd";
        }
        if READ_BASE.contains(&w) {
            return "ɹˈiːd";
        }
    }
    // the nearest time word in the clause
    let mut best: Option<(usize, bool)> = None;
    for j in lo..hi {
        if j == k {
            continue;
        }
        let w = words[j].0;
        let next = if j + 1 < hi { words[j + 1].0 } else { "" };
        let cue = if READ_PAST.contains(&w) || (w == "this" && READ_THIS_PAST.contains(&next)) {
            Some(true)
        } else if READ_PRESENT.contains(&w) {
            Some(false)
        } else {
            None
        };
        if let Some(c) = cue {
            let d = j.abs_diff(k);
            if best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, c));
            }
        }
    }
    if let Some((_, c)) = best {
        return if c { "ɹˈɛd" } else { "ɹˈiːd" };
    }
    // "procured it and read it": a bare predicate after a conjunction takes the earlier verb's tense
    if k == lo && lo > start {
        if (start..lo - 1).rev().any(|j| read_is_past_form(words[j].0)) {
            return "ɹˈɛd";
        }
    }
    // "read it and liked it": a coordinated verb later in the sentence
    for j in k + 1..end {
        if !["and", "then"].contains(&words[j].0) {
            continue;
        }
        let mut v = j + 1;
        if v < end && ["i", "you", "we", "they", "he", "she", "then"].contains(&words[v].0) {
            v += 1;
        }
        if v < end && read_is_past_form(words[v].0) {
            return "ɹˈɛd";
        }
    }
    "ɹˈiːd"
}
const READ_IRREGULAR_PAST: &[&str] = &[
    "went", "came", "saw", "said", "took", "got", "made", "found", "gave", "told", "thought", "felt", "left",
    "knew", "wrote", "sat", "stood", "heard", "began", "kept", "brought", "bought", "fell", "ran", "spoke",
    "understood", "forgot", "slept", "ate", "drank", "laughed", "cried",
];

const AUX_RESTRESS: &[&str] = &[
    "could", "would", "should", "can", "will", "do", "does", "did", "have", "has", "had", "is", "are", "was",
    "were", "am", "be", "been",
];

#[derive(Clone, Copy)]
enum Homograph {
    /// noun/adjective stresses earlier than the verb (REcord / reCORD)
    StressShift,
    /// noun ends in s, verb in z (use, house, excuse)
    Voicing,
    /// adjective/noun -ət vs verb -eɪt (separate, estimate)
    Ate,
    Read,
    Live,
    Wind,
    Tear,
    Lead,
    Close,
}

/// Is this word resolved by the sentence pass (accent layer: its form depends on context)?
pub(crate) fn is_homograph(w: &str) -> bool {
    homograph(w).is_some()
}

fn homograph(w: &str) -> Option<Homograph> {
    use Homograph::*;
    Some(match w {
        "record" | "records" | "present" | "presents" | "object" | "objects" | "project" | "projects"
        | "content" | "contract" | "contracts" | "permit" | "permits" | "desert" | "deserts" | "refuse"
        | "produce" | "conduct" | "convict" | "convicts" | "increase" | "increases" | "decrease" | "decreases"
        | "export" | "exports" | "import" | "imports" | "insult" | "insults" | "rebel" | "rebels" | "subject"
        | "subjects" | "progress" | "protest" | "protests" | "suspect" | "suspects" | "survey" | "surveys"
        | "transfer" | "transfers" | "transport" | "combat" | "conflict" | "conflicts" | "contest" | "contests"
        | "digest" | "extract" | "extracts" | "perfect" | "compound" | "compounds" | "discount" | "escort"
        | "impact" | "insert" | "inserts" | "invalid" | "reject" | "rejects" | "upset" | "attribute"
        | "attributes" | "console" | "contrast" | "convert" | "converts" | "defect" | "defects" | "entrance"
        | "incense" | "intern" | "interns" | "invite" | "invites" | "refund" | "refunds" | "segment"
        | "segments" | "torment" | "address" | "addresses" | "conduct's" | "update" | "updates" | "upgrade"
        | "upgrades" | "rewrite" | "rewrites" | "recall" | "recalls" | "reset" | "resets" => StressShift,
        "use" | "uses" | "excuse" | "excuses" | "abuse" | "abuses" | "house" | "houses" | "misuse" => Voicing,
        "separate" | "estimate" | "estimates" | "graduate" | "graduates" | "moderate" | "alternate"
        | "associate" | "associates" | "advocate" | "advocates" | "appropriate" | "delegate" | "delegates"
        | "duplicate" | "duplicates" | "approximate" | "elaborate" | "articulate" | "deliberate"
        | "degenerate" | "intimate" | "subordinate" | "coordinate" | "coordinates" | "designate" | "aggregate"
        | "certificate" | "initiate" | "syndicate" | "affiliate" | "affiliates" | "animate" => Ate,
        "read" => Read,
        "live" => Live,
        "wind" | "winds" => Wind,
        "tear" | "tears" => Tear,
        "lead" => Lead,
        "close" | "closes" => Close,
        _ => return None,
    })
}

pub struct English {
    lex: Option<Lexicon>,
    model: Option<Model>,
    cache: std::sync::Mutex<HashMap<String, String>>,
    func: HashMap<&'static str, &'static str>,
    merge: HashMap<&'static str, &'static str>,
    /// optional sentence-level model for the words of data/en_homograph.tsv (`en_homograph.g2p`)
    hg: Option<homograph::Model>,
}

fn table(s: &'static str) -> HashMap<&'static str, &'static str> {
    s.lines().filter_map(|l| l.split_once('\t')).collect()
}

const VOWEL_START: &str = "aeiouæɑɔəɛɜɪʊʌɚᵻɐ";

fn starts_with_vowel(ph: &str) -> bool {
    ph.chars().find(|&c| c != 'ˈ' && c != 'ˌ').map_or(false, |c| VOWEL_START.contains(c))
}

/// Index (in chars) of the primary stress mark relative to the vowels before it.
fn stress_rank(p: &str) -> usize {
    let mut v = 0;
    let mut prev_v = false;
    for c in p.chars() {
        if c == 'ˈ' {
            return v;
        }
        let isv = VOWEL_START.contains(c);
        if isv && !prev_v {
            v += 1;
        }
        prev_v = isv;
    }
    usize::MAX
}

impl English {
    pub fn load(lex: Option<&Path>, model: Option<&Path>) -> Self {
        English {
            lex: lex.and_then(|p| Lexicon::open(p).ok()),
            model: model.and_then(|p| Model::open(p).ok()),
            cache: Default::default(),
            func: table(FUNC_TSV),
            merge: table(MERGE_TSV),
            hg: None,
        }
    }

    /// Masked logits of the heteronym model for the word at unit `target` (tools): (pronunciation, logit) in table order.
    pub fn heteronym_scores(&self, units: &[homograph::Unit], target: usize) -> Option<Vec<(&str, f32)>> {
        self.hg.as_ref()?.scores(units, target)
    }

    /// Load the heteronym model (`en_homograph.g2p`); a file that does not open or does not fit the table is ignored.
    pub fn load_homograph(&mut self, path: &Path) {
        self.hg = homograph::Model::open(path, &homograph::Table::parse(HOMOGRAPH_TSV)).ok();
    }

    pub fn has_lexicon(&self) -> bool {
        self.lex.is_some()
    }

    /// The lexicon's default pronunciation of a lower-case word (accent layer).
    pub(crate) fn lexicon_form(&self, lw: &str) -> Option<&str> {
        self.lookup(lw)
    }

    /// The full (stressed) form of a lower-case word: its lexicon entry when it has
    /// one, else what `word` gives (accent tools).
    pub fn citation(&self, lw: &str) -> String {
        match self.lookup(lw) {
            Some(p) => p.to_string(),
            None => self.word(lw, &[]),
        }
    }

    /// Is this word one whose reading the sentence pass chooses (accent tools)?
    pub fn homograph_word(&self, lw: &str) -> bool {
        is_homograph(lw) || self.hg.as_ref().is_some_and(|m| m.covers(lw))
    }

    /// Is `ph` this word read letter by letter (accent layer: ROI, CRO)?
    pub(crate) fn is_spelled(&self, lw: &str, ph: &str) -> bool {
        lw.len() <= 6 && lw.chars().all(|c| c.is_ascii_alphabetic()) && self.lookup(lw) != Some(ph) && self.spell(lw) == ph
    }

    /// Is `ph` the pronunciation this word gets with no context: its lexicon
    /// entry, or what `word` returns for it (accent layer: a weak form, a
    /// homograph reading or a merged pair is not).
    pub(crate) fn is_default_form(&self, lw: &str, ph: &str) -> bool {
        if self.lookup(lw) == Some(ph) {
            return true;
        }
        !self.func.contains_key(lw) && self.word(lw, &[]) == ph
    }

    fn lookup(&self, w: &str) -> Option<&str> {
        self.lex.as_ref()?.get(w)
    }

    fn variants(&self, w: &str) -> Vec<&str> {
        self.lex.as_ref().and_then(|l| l.get_all(w)).map(|s| s.split('\t').collect()).unwrap_or_default()
    }

    fn spell(&self, w: &str) -> String {
        let letters: Vec<usize> = w
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| (c.to_ascii_lowercase() as u8 - b'a') as usize)
            .collect();
        let mut out = String::new();
        for (k, &l) in letters.iter().enumerate() {
            let name = LETTERS[l].to_string();
            if l == 22 {
                // "double-u" already carries its own stress
                out.push_str(&name.replace('ˈ', if k + 1 == letters.len() { "ˈ" } else { "ˌ" }));
                continue;
            }
            let mark = if k + 1 == letters.len() { "ˈ" } else { "ˌ" };
            // stress goes right before the letter name's vowel
            let pos = name.find(|c: char| VOWEL_START.contains(c)).unwrap_or(0);
            out.push_str(&name[..pos]);
            out.push_str(mark);
            out.push_str(&name[pos..]);
        }
        out
    }

    fn oov(&self, lw: &str) -> String {
        if let Some(p) = self.cache.lock().unwrap().get(lw) {
            return p.clone();
        }
        let p = self.decompose(lw).unwrap_or_else(|| match &self.model {
            Some(m) => m.predict(lw),
            None => self.spell(lw),
        });
        let mut c = self.cache.lock().unwrap();
        if c.len() > 50_000 {
            c.clear();
        }
        c.insert(lw.to_string(), p.clone());
        p
    }

    /// Known word + productive suffix / prefix, or two known words (cheap, and
    /// more reliable than the model when it applies).
    fn decompose(&self, lw: &str) -> Option<String> {
        let lex = self.lex.as_ref()?;
        let n = lw.len();
        if n < 5 || !lw.is_ascii() {
            return None;
        }
        // suffixes with their pronunciation; `voice` picks s/z/ᵻz style allomorphs
        const SUF: &[(&str, &str)] = &[
            ("ness", "nəs"), ("less", "ləs"), ("ful", "fəl"), ("ment", "mənt"), ("able", "əbəl"), ("ly", "li"),
            ("ish", "ɪʃ"), ("like", "lˌaɪk"), ("ism", "ˌɪzəm"), ("ist", "ɪst"), ("ers", "ɚz"), ("er", "ɚ"),
            ("ings", "ɪŋz"), ("ing", "ɪŋ"), ("ed", "d"), ("es", "z"), ("s", "z"),
        ];
        for (suf, sp) in SUF {
            if let Some(stem) = lw.strip_suffix(suf) {
                if stem.len() < 3 {
                    continue;
                }
                let mut cands = vec![stem.to_string()];
                if matches!(*suf, "ed" | "er" | "ers" | "ing" | "ings" | "able" | "es") {
                    cands.push(format!("{stem}e"));
                    let b = stem.as_bytes();
                    if b.len() >= 2 && b[b.len() - 1] == b[b.len() - 2] {
                        cands.push(stem[..stem.len() - 1].to_string()); // stopped -> stop
                    }
                }
                if *suf == "ly" || *suf == "ness" || *suf == "less" || *suf == "ful" {
                    if let Some(s2) = stem.strip_suffix('i') {
                        cands.push(format!("{s2}y")); // happily -> happy
                    }
                }
                for c in cands {
                    if let Some(p) = lex.get(&c) {
                        let last = p.chars().last().unwrap_or(' ');
                        let tail = match *suf {
                            "s" | "es" => match last {
                                's' | 'z' | 'ʃ' | 'ʒ' | 'ʧ' | 'ʤ' => "ᵻz",
                                'p' | 't' | 'k' | 'f' | 'θ' => "s",
                                _ => "z",
                            }
                            .to_string(),
                            "ed" => match last {
                                't' | 'd' => "ᵻd",
                                'p' | 'k' | 'f' | 'θ' | 's' | 'ʃ' | 'ʧ' => "t",
                                _ => "d",
                            }
                            .to_string(),
                            _ => sp.to_string(),
                        };
                        return Some(format!("{p}{tail}"));
                    }
                }
            }
        }
        const PRE: &[(&str, &str)] = &[
            ("un", "ʌn"), ("re", "ɹˌiː"), ("non", "nˌɑːn"), ("pre", "pɹˌiː"), ("over", "ˌoʊvɚ"), ("under", "ˌʌndɚ"),
            ("out", "ˌaʊt"), ("mis", "mˌɪs"), ("dis", "dˌɪs"), ("anti", "ˌæntaɪ"), ("super", "sˌuːpɚ"),
            ("sub", "sˌʌb"), ("inter", "ˌɪntɚ"), ("counter", "kˌaʊntɚ"), ("multi", "mˌʌlti"), ("self", "sˌɛlf"),
        ];
        for (pre, pp) in PRE {
            if let Some(rest) = lw.strip_prefix(pre) {
                let rest = rest.trim_start_matches('-');
                if rest.len() >= 4 {
                    if let Some(p) = lex.get(rest) {
                        return Some(format!("{pp}{p}"));
                    }
                }
            }
        }
        // compound of two known words (both >= 3 letters), head stress on the first
        for i in 3..=n.saturating_sub(3) {
            let (a, b) = lw.split_at(i);
            if let (Some(pa), Some(pb)) = (lex.get(a), lex.get(b)) {
                if a.len() >= 3 && b.len() >= 3 && !self.func.contains_key(a) && !self.func.contains_key(b) {
                    return Some(format!("{pa}{}", pb.replace('ˈ', "ˌ")));
                }
            }
        }
        None
    }

    /// Context-free pronunciation of one word (homographs get their default here
    /// and are revisited in `join`, which sees both neighbours).
    pub fn word(&self, w: &str, _prev: &[Out]) -> String {
        let lw = w.to_lowercase();
        if let Some(f) = self.func.get(lw.as_str()) {
            return f.to_string();
        }
        let caps = w.chars().all(|c| !c.is_lowercase()) && w.chars().filter(|c| c.is_alphabetic()).count() >= 2;
        if caps {
            let n = w.chars().filter(|c| c.is_alphabetic()).count();
            let known = self.lookup(&lw).is_some();
            let has_vowel = lw.chars().any(|c| "aeiouy".contains(c));
            if n <= 3 || !has_vowel || (!known && n <= 5) {
                return self.spell(w);
            }
        }
        if let Some(p) = self.lookup(&lw) {
            return p.to_string();
        }
        // hyphenated compounds: each part on its own
        if lw.contains('-') {
            let parts: Vec<String> = lw.split('-').filter(|p| !p.is_empty()).map(|p| self.word(p, &[])).collect();
            return parts.join(" ");
        }
        // possessive / plural 's of a known word
        if let Some(base) = lw.strip_suffix("'s").or_else(|| lw.strip_suffix("s'")) {
            if let Some(p) = self.lookup(base) {
                let last = p.chars().last().unwrap_or(' ');
                let suf = match last {
                    's' | 'z' | 'ʃ' | 'ʒ' | 'ʧ' | 'ʤ' => "ᵻz",
                    'p' | 't' | 'k' | 'f' | 'θ' => "s",
                    _ => "z",
                };
                return format!("{p}{suf}");
            }
        }
        if !lw.chars().any(|c| c.is_ascii_alphabetic()) {
            return String::new();
        }
        self.oov(&lw)
    }

    fn pick_homograph(&self, kind: Homograph, w: &str, prev: &str, next: &str) -> Option<String> {
        use Homograph::*;
        let vs = self.variants(w);
        if vs.len() < 2 && !matches!(kind, Read) {
            return None;
        }
        let det = DET.contains(&prev);
        let verbal = VERBAL.contains(&prev) || next == "the" || next == "a" || next == "an" || next == "it" || next == "them";
        let pick = |f: &dyn Fn(&str) -> bool| vs.iter().find(|v| f(v)).map(|s| s.to_string());
        match kind {
            StressShift | Voicing | Ate | Close => {
                let want_verb = match kind {
                    Close => {
                        if ["very", "so", "too", "quite", "pretty", "is", "are", "was", "were", "be", "a", "the"].contains(&prev)
                            || ["to", "by", "friend", "friends", "call", "up", "relationship"].contains(&next)
                        {
                            false
                        } else {
                            verbal
                        }
                    }
                    _ => !det && verbal,
                };
                match kind {
                    StressShift => {
                        let mut sorted: Vec<&&str> = vs.iter().collect();
                        sorted.sort_by_key(|v| stress_rank(v));
                        let v = if want_verb { sorted.last() } else { sorted.first() };
                        v.map(|s| s.to_string())
                    }
                    Voicing | Close => {
                        let end = if want_verb { 'z' } else { 's' };
                        let strip = |v: &str| v.trim_end_matches(|c| c == 'ᵻ' || c == 'z' && w.ends_with("es")).to_string();
                        let _ = strip;
                        pick(&|v: &str| {
                            let core = if w.ends_with("es") && v.ends_with("ᵻz") { &v[..v.len() - "ᵻz".len()] } else { v };
                            core.ends_with(end)
                        })
                    }
                    _ => pick(&|v: &str| if want_verb { v.contains("eɪt") } else { !v.contains("eɪt") }),
                }
            }
            Read => {
                let past = PERFECT.contains(&prev);
                Some(if past { "ɹˈɛd".into() } else { "ɹˈiːd".into() })
            }
            Live => {
                let adj = ["music", "show", "stream", "streaming", "performance", "concert", "broadcast", "event",
                    "audience", "album", "coverage", "version", "wire", "animals", "bait", "ammunition", "action",
                    "tv", "television", "recording", "update", "updates", "feed", "chat", "demo", "band"]
                    .contains(&next)
                    || ["a", "the", "go", "goes", "went", "is", "are", "was", "were"].contains(&prev);
                pick(&|v: &str| if adj { v.contains("aɪ") } else { !v.contains("aɪ") })
            }
            Wind => {
                let verb = VERBAL.contains(&prev) || ["up", "down", "back"].contains(&next);
                pick(&|v: &str| if verb { v.contains("aɪ") } else { !v.contains("aɪ") })
            }
            Tear => {
                let verb = (VERBAL.contains(&prev) && !det)
                    || ["up", "down", "apart", "off", "it", "the", "them", "through", "into"].contains(&next);
                let noun = det || ["in", "into", "of"].contains(&prev) || ["gas", "drop", "drops", "duct", "ducts"].contains(&next);
                let verb = verb && !noun;
                pick(&|v: &str| if verb { v.contains("ɛɹ") } else { v.contains("ɪɹ") })
            }
            Lead => {
                let metal = ["pipe", "pipes", "paint", "poisoning", "pencil", "pencils", "weight", "weights",
                    "balloon", "levels", "exposure", "free", "shot", "crystal"]
                    .contains(&next)
                    || prev == "of";
                pick(&|v: &str| if metal { v.contains('ɛ') } else { v.contains("iː") })
            }
        }
    }

    /// Title / unit abbreviations ("Dr. Smith" reads "doctor smith" without a pause): the first step of `join`.
    pub(crate) fn expand_titles(&self, outs: &mut [Out]) {
        let n = outs.len();
        // title / unit abbreviations: "Dr. Smith" reads "doctor smith" without a pause
        for k in 0..n {
            let src = outs[k].src.as_str();
            let has_next = k + 1 < n && !outs[k + 1].src.is_empty();
            let expanded = match src {
                "dr" => "doctor",
                "mr" => "mister",
                "mrs" => "missus",
                "ms" => "miz",
                "jr" => "junior",
                "sr" => "senior",
                "prof" => "professor",
                "mt" => "mount",
                "rev" => "reverend",
                "capt" => "captain",
                "lt" => "lieutenant",
                "sgt" => "sergeant",
                "gov" => "governor",
                "sen" => "senator",
                "vs" => "versus",
                "etc" => "etcetera",
                "inc" => "incorporated",
                "ltd" => "limited",
                "st" if outs[k].punct == "." || has_next => {
                    if has_next && outs[k].punct.is_empty() || outs[k].punct == "." && has_next { "saint" } else { "street" }
                }
                _ => continue,
            };
            // a plain lowercase word in running text ("the st") is more likely not an abbreviation
            let abbreviated = outs[k].punct.starts_with('.') || matches!(src, "mr" | "mrs" | "dr" | "ms" | "vs" | "etc" | "jr" | "sr");
            if !abbreviated {
                continue;
            }
            let ph = match expanded {
                "miz" => Some("mˈɪz".to_string()),
                w => self.lookup(w).map(|p| p.to_string()),
            };
            if let Some(p) = ph {
                outs[k].ph = p;
                if outs[k].punct == "." && has_next && expanded != "etcetera" {
                    outs[k].punct.clear();
                    outs[k].pause_after = false;
                }
            }
        }
    }

    /// Sentence pass: homographs, function-word linking and espeak's word joining.
    pub fn join(&self, outs: &mut [Out]) {
        let n = outs.len();
        self.expand_titles(outs);
        // the heteronym model, when there is one, reads whole sentences
        let hg = self.hg.as_ref().and_then(|m| {
            let (u, at) = homograph::units(outs);
            (u.iter().filter(|x| x.word).count() >= HG_MIN_WORDS).then_some((m, u, at))
        });
        for k in 0..n {
            let src = outs[k].src.clone();
            if let Some((m, u, at)) = &hg {
                if m.covers(&src) {
                    if let Some(p) = m.pick(u, at[k]) {
                        outs[k].ph = p.to_string();
                        continue;
                    }
                }
            }
            let prev = if k > 0 && !outs[k - 1].pause_after { outs[k - 1].src.clone() } else { String::new() };
            let next = if k + 1 < n && !outs[k].pause_after { outs[k + 1].src.clone() } else { String::new() };
            if let Some(kind) = homograph(&src) {
                if let Some(mut p) = self.pick_homograph(kind, &src, &prev, &next) {
                    if matches!(kind, Homograph::Read) {
                        p = read_form(&outs.iter().map(|o| (o.src.as_str(), o.pause_after)).collect::<Vec<_>>(), k).into();
                    }
                    outs[k].ph = p;
                }
            }
        }
        // an auxiliary at the end of a phrase is stressed ("if you could.")
        for k in 0..n {
            let at_end = outs[k].pause_after || k + 1 == n;
            if at_end && AUX_RESTRESS.contains(&outs[k].src.as_str()) {
                if let Some(p) = self.lookup(&outs[k].src) {
                    outs[k].ph = p.to_string();
                }
            }
        }
        for k in 0..n.saturating_sub(1) {
            if outs[k].pause_after || outs[k + 1].ph.is_empty() {
                continue;
            }
            let next_vowel = starts_with_vowel(&outs[k + 1].ph);
            match outs[k].src.as_str() {
                "the" if next_vowel => outs[k].ph = "ðɪ".into(),
                "to" if next_vowel => outs[k].ph = "tʊ".into(),
                _ => {}
            }
        }
        let mut k = 0;
        while k + 1 < n {
            if !outs[k].pause_after && !outs[k + 1].src.is_empty() {
                let key = format!("{} {}", outs[k].src, outs[k + 1].src);
                if let Some(m) = self.merge.get(key.as_str()) {
                    outs[k].ph = m.to_string();
                    outs[k].punct = std::mem::take(&mut outs[k + 1].punct);
                    outs[k].pause_after = outs[k + 1].pause_after;
                    outs[k + 1].ph.clear();
                    outs[k + 1].src.clear();
                    k += 2;
                    continue;
                }
            }
            k += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spelling() {
        let e = English { lex: None, model: None, cache: Default::default(), func: table(FUNC_TSV), merge: table(MERGE_TSV), hg: None };
        assert_eq!(e.spell("FBI"), "ˌɛfbˌiːˈaɪ");
        assert_eq!(e.spell("UK"), "jˌuːkˈeɪ");
    }
}
