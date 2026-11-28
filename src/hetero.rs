//! Context models for heteronyms (words spelled alike, read differently), for the
//! European languages. English and Russian have their own paths; this module
//! covers the rest.
//!
//! Per language a hand-written table (`data/hetero_<lang>.tsv`, compiled in) lists
//! the words and their readings. A small character-level encoder
//! (`<lang>-ctx.g2p` in the data directory, trained by `tools/train_ctx.py`, run
//! by `tagger::Tagger`) reads the whole sentence and picks the reading class of
//! each table word. No model file: nothing changes, output is byte-identical to
//! the rules. `POLYG2P_CTX=0` disables every model of this module.
//!
//! No confidence threshold: `Decision::model_conf` (softmax over the word's own classes) is recorded for
//! evaluation, but keeping the rules below a threshold was measured and rejected (docs/heteronyms.md).
//! Cost bound: one encoder pass per window shared by all hits in it, at most `MAX_PASSES` per call;
//! hits beyond the cap keep the rules.
//!
//! Table columns: `word <TAB> k <TAB> reading <TAB> tag <TAB> note [<TAB> mode]`
//!   k        class index 0..3 (the `vow` head of the encoder predicts it)
//!   reading  `rule` (keep what the rules produced), `respell:<spelling>` (run the
//!            language's word function on another spelling), `ph:<phonemes>` or
//!            `sub:<from>><to>` (the rules' phonemes with the last <from> replaced)
//!   word     a word, or `*suffix` for an open class (exact words win)
//!   tag      teacher tag used by tools/hetero_label.py (`UPOS[:Feat=Val...]`, `-` = none)
//!   mode     `rule` = the model is not trusted for this word, keep the rules
//!
//! The encoder input is the sentence as the engine sees it: the lower-cased source
//! words, each followed by its punctuation, joined by single spaces.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use crate::tagger::{Logits, Tagger};
use crate::{G2p, Lang, Out};

struct Row {
    k: u8,
    reading: String,
    mode_rule: bool,
}

/// Exact words, plus suffix rules (`*tions`) for open classes.
#[derive(Default)]
struct Table {
    exact: HashMap<String, Vec<Row>>,
    suffix: Vec<(String, Vec<Row>)>,
}

impl Table {
    fn get(&self, w: &str) -> Option<&Vec<Row>> {
        self.exact.get(w).or_else(|| {
            self.suffix.iter().find(|(s, _)| w.chars().count() > s.chars().count() + 2 && w.ends_with(s.as_str())).map(|(_, r)| r)
        })
    }
}

fn parse_table(src: &str) -> Table {
    let mut t = Table::default();
    for line in src.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let Ok(k) = f[1].parse::<u8>() else { continue };
        let row = Row { k, reading: f[2].to_string(), mode_rule: f.get(5).is_some_and(|m| *m == "rule") };
        if let Some(suf) = f[0].strip_prefix('*') {
            match t.suffix.iter_mut().find(|(s, _)| s == suf) {
                Some((_, rows)) => rows.push(row),
                None => t.suffix.push((suf.to_string(), vec![row])),
            }
        } else {
            t.exact.entry(f[0].to_string()).or_default().push(row);
        }
    }
    t
}

fn table(lang: Lang) -> Option<&'static Table> {
    macro_rules! tab {
        ($f:literal) => {{
            static T: OnceLock<Table> = OnceLock::new();
            Some(T.get_or_init(|| parse_table(include_str!(concat!("../data/", $f)))))
        }};
    }
    match lang {
        Lang::Fr => tab!("hetero_fr.tsv"),
        Lang::De => tab!("hetero_de.tsv"),
        Lang::Pt => tab!("hetero_pt.tsv"),
        Lang::Uk => tab!("hetero_uk.tsv"),
        Lang::It => tab!("hetero_it.tsv"),
        Lang::Nl => tab!("hetero_nl.tsv"),
        Lang::Ro => tab!("hetero_ro.tsv"),
        _ => None,
    }
}

fn disabled() -> bool {
    static D: OnceLock<bool> = OnceLock::new();
    *D.get_or_init(|| std::env::var("POLYG2P_CTX").is_ok_and(|v| matches!(v.as_str(), "0" | "off" | "false")))
}

/// Languages whose `<lang>-ctx.g2p` is recognised at run time (those that passed the gate).
fn model_languages(lang: Lang) -> bool {
    matches!(lang, Lang::Pt | Lang::It)
}

fn model(g: &G2p, lang: Lang) -> Option<Arc<Tagger>> {
    // keyed by the model file's path, so engines with different data directories do not share a cache
    static M: OnceLock<Mutex<HashMap<PathBuf, Option<Arc<Tagger>>>>> = OnceLock::new();
    let path = g.file(&format!("{}-ctx.g2p", lang.code()))?;
    let m = M.get_or_init(|| Mutex::new(HashMap::new()));
    let mut m = m.lock().unwrap();
    m.entry(path.clone()).or_insert_with(|| Tagger::open(&path).ok().map(Arc::new)).clone()
}

/// One decision, for evaluation (`bin/hetero_check`).
#[derive(Debug, Clone)]
pub struct Decision {
    pub word_index: usize,
    pub word: String,
    /// the table class whose reading equals what the rules produced
    pub rule_k: Option<u8>,
    /// the class the encoder picks (None without a model)
    pub model_k: Option<u8>,
    /// the phonemes of every class, in table order
    pub phs: Vec<(u8, String)>,
    /// the table keeps the rules for this word (`mode` = rule): `model_k` is then only the model's opinion
    pub mode_rule: bool,
    /// the model's probability of `model_k` (softmax over the word's own classes); None without a model
    pub model_conf: Option<f32>,
}

impl Decision {
    /// The class that ships: the model's, unless the word is `mode = rule`.
    pub fn shipped_k(&self) -> Option<u8> {
        match self.model_k {
            Some(_) if self.mode_rule => self.rule_k,
            k => k,
        }
    }
}

/// What `apply` saw, for the training-data dump.
#[derive(Debug, Clone, Default)]
pub struct Capture {
    pub text: String,
    /// (char offset of the word in `text`, word, has punctuation after it)
    pub words: Vec<(usize, String, String)>,
    pub decisions: Vec<Decision>,
    /// encoder forward passes this call ran
    pub passes: usize,
}

thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
    static NO_REPLACE: RefCell<bool> = const { RefCell::new(false) };
}

/// Capture of what `apply` sees on this thread, scoped to the guard: dropping it (also on a panic
/// or an early return) clears the capture and the no-replace flag, so evaluation-only behaviour
/// (languages whose model is not shipped, output left untouched) can never leak into a later
/// ordinary `phonemize`. `replace` = false leaves the output untouched.
pub struct CaptureGuard(());

pub fn capture(replace: bool) -> CaptureGuard {
    CAPTURE.with(|c| *c.borrow_mut() = Some(Capture::default()));
    NO_REPLACE.with(|c| *c.borrow_mut() = !replace);
    CaptureGuard(())
}

impl CaptureGuard {
    /// What the last `phonemize` under this guard saw (the capture is then empty until the next one).
    pub fn take(&self) -> Capture {
        CAPTURE.with(|c| c.borrow_mut().replace(Capture::default())).unwrap_or_default()
    }
}

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        CAPTURE.with(|c| *c.borrow_mut() = None);
        NO_REPLACE.with(|c| *c.borrow_mut() = false);
    }
}

/// Hard cap on encoder passes per `phonemize` call; hits beyond it keep the rules.
pub const MAX_PASSES: usize = 32;

fn word_ph(g: &G2p, lang: Lang, w: &str) -> String {
    match lang {
        Lang::Fr => crate::fr::word(w),
        Lang::Pt => crate::pt::word(w),
        Lang::Nl => crate::nl::word(w),
        Lang::Uk => crate::uk::word(w),
        Lang::Ro => crate::ro::word(w),
        Lang::De => g.german().word(w),
        Lang::It => g.italian().word(w),
        _ => String::new(),
    }
}

fn reading_ph(g: &G2p, lang: Lang, row: &Row, rule_ph: &str) -> String {
    if let Some(s) = row.reading.strip_prefix("respell:") {
        word_ph(g, lang, s)
    } else if let Some(p) = row.reading.strip_prefix("ph:") {
        p.to_string()
    } else if let Some(sub) = row.reading.strip_prefix("sub:") {
        match sub.split_once('>') {
            Some((a, b)) => match rule_ph.rfind(a) {
                Some(i) => format!("{}{}{}", &rule_ph[..i], b, &rule_ph[i + a.len()..]),
                None => rule_ph.to_string(),
            },
            None => rule_ph.to_string(),
        }
    } else {
        rule_ph.to_string()
    }
}

/// Start of a window of at most `max` characters with `focus` at most `lead` characters from its
/// left edge, snapped back to a word start.
fn window_start(sc: &[char], focus: usize, max: usize, lead: usize) -> usize {
    if sc.len() <= max {
        return 0;
    }
    let last = sc.len() - max;
    let mut start = focus.saturating_sub(lead).min(last);
    if start == last {
        // the window ends the text: snap forward so it still does (the end is what the next hits rely on)
        while start > 0 && start < focus && sc[start - 1] != ' ' {
            start += 1;
        }
    } else {
        while start > 0 && sc[start - 1] != ' ' && focus - start < max - 1 {
            start -= 1;
        }
    }
    start
}

/// Windows that cover every hit once. A hit is covered by a window when it lies at least `max / 4`
/// characters before its right edge (or the window ends the text). The first window is centred on the
/// first hit; each later one starts a quarter-window before the first uncovered hit, so a pass
/// advances about `max / 2` characters. Returns the window starts (at most `cap`) and, per hit,
/// the window that covers it (None = beyond the cap: the rules stay).
/// `hits` = (char offset, char length) in the text of `n` characters.
fn plan_windows(sc: &[char], hits: &[(usize, usize)], max: usize, cap: usize) -> (Vec<usize>, Vec<Option<usize>>) {
    let n = sc.len();
    let margin = max / 4;
    let mut starts: Vec<usize> = Vec::new();
    let mut of_hit = Vec::with_capacity(hits.len());
    for &(off, len) in hits {
        let covered = |start: usize| {
            let end = (start + max).min(n);
            off >= start && off + len <= end && (end == n || off + len + margin <= end)
        };
        if starts.last().is_some_and(|&s| covered(s)) {
            of_hit.push(Some(starts.len() - 1));
            continue;
        }
        if starts.len() >= cap {
            of_hit.push(None);
            continue;
        }
        let lead = if starts.is_empty() { max / 2 } else { margin };
        starts.push(window_start(sc, off, max, lead));
        of_hit.push(Some(starts.len() - 1));
    }
    (starts, of_hit)
}

/// Called from the sentence pass, after the per-word loop and before the
/// language's `join`: replaces `outs[i].ph` of table words by the reading the
/// context model picks.
pub fn apply(g: &G2p, lang: Lang, outs: &mut [Out]) {
    let capturing = CAPTURE.with(|c| c.borrow().is_some());
    // fr was measured and dropped by the gate (docs/heteronyms.md); evaluation (capture) may still run it
    if !capturing && !model_languages(lang) {
        return;
    }
    let Some(tab) = table(lang) else { return };
    if disabled() && !capturing {
        return;
    }
    if !outs.iter().any(|o| tab.get(&o.src).is_some()) && !capturing {
        return;
    }
    // encoder input
    let mut text = String::new();
    let mut offs: Vec<usize> = Vec::with_capacity(outs.len());
    let mut idx: Vec<usize> = Vec::with_capacity(outs.len());
    let mut n = 0usize;
    for (i, o) in outs.iter().enumerate() {
        if o.src.is_empty() {
            continue;
        }
        if !text.is_empty() {
            text.push(' ');
            n += 1;
        }
        offs.push(n);
        idx.push(i);
        text.push_str(&o.src);
        text.push_str(&o.punct);
        n += o.src.chars().count() + o.punct.chars().count();
    }
    let hits: Vec<usize> = (0..idx.len()).filter(|&j| tab.get(&outs[idx[j]].src).is_some()).collect();
    let net = if disabled() { None } else { model(g, lang) };
    let sc: Vec<char> = text.chars().collect();
    // one forward pass per window, shared by every hit it covers (not one pass per hit)
    let mut logits: Vec<Logits> = Vec::new();
    let mut starts: Vec<usize> = Vec::new();
    let mut of_hit: Vec<Option<usize>> = vec![None; hits.len()];
    if let Some(net) = &net {
        let spans: Vec<(usize, usize)> =
            hits.iter().map(|&j| (offs[j], outs[idx[j]].src.chars().count() + outs[idx[j]].punct.chars().count())).collect();
        (starts, of_hit) = plan_windows(&sc, &spans, net.cfg.max_len, MAX_PASSES);
        for &s in &starts {
            let win: String = sc[s..(s + net.cfg.max_len).min(sc.len())].iter().collect();
            logits.push(net.forward(&net.encode(&win)));
        }
    }
    let mut decisions = Vec::new();
    let mut changes: Vec<(usize, String)> = Vec::new();
    for (h, &j) in hits.iter().enumerate() {
        let i = idx[j];
        let rows = tab.get(&outs[i].src).unwrap();
        let rule_ph = outs[i].ph.clone();
        let ph_of: Vec<(u8, String)> = rows.iter().map(|r| (r.k, reading_ph(g, lang, r, &rule_ph))).collect();
        let rule_k = ph_of
            .iter()
            .zip(rows)
            .find(|((_, p), r)| *p == rule_ph && !r.reading.starts_with("rule"))
            .or_else(|| ph_of.iter().zip(rows).find(|((_, p), _)| *p == rule_ph))
            .map(|((k, _), _)| *k);
        let mut model_k = None;
        let mut model_conf = None;
        if let (Some(net), Some(w)) = (&net, of_hit[h]) {
            let lg = &logits[w];
            let p = offs[j] - starts[w];
            if p < lg.len {
                let nv = net.cfg.n_vow;
                let mut best = (f32::MIN, 0u8);
                let mut zs = Vec::with_capacity(rows.len());
                for r in rows {
                    let v = lg.vow[p * nv + r.k as usize];
                    zs.push(v);
                    if v > best.0 {
                        best = (v, r.k);
                    }
                }
                let z: f32 = zs.iter().map(|v| (v - best.0).exp()).sum();
                model_conf = Some(1.0 / z);
                model_k = Some(best.1);
            }
        }
        if let Some(k) = model_k {
            if !rows.iter().any(|r| r.mode_rule) {
                if let Some((_, ph)) = ph_of.iter().find(|(kk, _)| *kk == k) {
                    if !ph.is_empty() && *ph != rule_ph {
                        changes.push((i, ph.clone()));
                    }
                }
            }
        }
        decisions.push(Decision { word_index: j, word: outs[i].src.clone(), rule_k, model_k, phs: ph_of.clone(), mode_rule: rows.iter().any(|r| r.mode_rule), model_conf });
    }
    if capturing {
        let words = idx.iter().enumerate().map(|(j, &i)| (offs[j], outs[i].src.clone(), outs[i].punct.clone())).collect();
        CAPTURE.with(|c| *c.borrow_mut() = Some(Capture { text: text.clone(), words, decisions, passes: starts.len() }));
        if NO_REPLACE.with(|c| *c.borrow()) {
            return;
        }
    }
    for (i, ph) in changes {
        outs[i].ph = ph;
    }
}

impl G2p {
    /// Table words of `text` with the class the rules produced and the class the
    /// context model picks (for evaluation).
    pub fn hetero_decisions(&self, text: &str, lang: Lang) -> Vec<Decision> {
        let cap = capture(false);
        let _ = self.phonemize(text, lang);
        cap.take().decisions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hits_of(text: &str, w: &str) -> (Vec<char>, Vec<(usize, usize)>) {
        let mut offs = Vec::new();
        let mut n = 0;
        for word in text.split(' ') {
            if word == w {
                offs.push((n, word.chars().count()));
            }
            n += word.chars().count() + 1;
        }
        (text.chars().collect(), offs)
    }

    /// Pass counts, not wall time: 3000 hits in one line must cost a bounded number of passes.
    #[test]
    fn windows_are_shared_and_capped() {
        let text = "gosto ".repeat(3000);
        let text = text.trim_end();
        let (sc, hits) = hits_of(text, "gosto");
        assert_eq!(hits.len(), 3000);
        let (starts, of_hit) = plan_windows(&sc, &hits, 320, MAX_PASSES);
        assert!(starts.len() <= MAX_PASSES, "{} passes", starts.len());
        assert_eq!(starts.len(), MAX_PASSES, "the cap is what bounds this text");
        // every hit covered by a window lies inside it, with context on both sides
        for (&(off, len), w) in hits.iter().zip(&of_hit) {
            if let Some(w) = w {
                assert!(off >= starts[*w] && off + len <= starts[*w] + 320);
            }
        }
        // hits beyond the cap are left to the rules, the ones before it are covered
        assert!(of_hit[0].is_some() && of_hit[2999].is_none());
        // with room to spare, windows are reused: ~every 160 characters is one pass
        let text = "gosto ".repeat(300);
        let (sc, hits) = hits_of(text.trim_end(), "gosto");
        let (starts, of_hit) = plan_windows(&sc, &hits, 320, MAX_PASSES);
        assert!(of_hit.iter().all(|w| w.is_some()));
        assert!(starts.len() <= 1800 / 160 + 2, "{} passes for 300 hits", starts.len());
    }

    #[test]
    fn short_text_is_one_window() {
        let (sc, hits) = hits_of("eu gosto de ti e tu gosto", "gosto");
        let (starts, of_hit) = plan_windows(&sc, &hits, 320, MAX_PASSES);
        assert_eq!(starts, vec![0]);
        assert_eq!(of_hit, vec![Some(0), Some(0)]);
    }

    #[test]
    fn capture_is_scoped_to_the_guard() {
        {
            let c = capture(false);
            assert!(CAPTURE.with(|c| c.borrow().is_some()));
            assert!(NO_REPLACE.with(|c| *c.borrow()));
            let _ = c.take();
            assert!(CAPTURE.with(|c| c.borrow().is_some()), "still capturing after take");
        }
        assert!(CAPTURE.with(|c| c.borrow().is_none()));
        assert!(!NO_REPLACE.with(|c| *c.borrow()));
        // a panic inside the scope also resets it
        let r = std::panic::catch_unwind(|| {
            let _c = capture(false);
            panic!("boom");
        });
        assert!(r.is_err());
        assert!(CAPTURE.with(|c| c.borrow().is_none()));
        assert!(!NO_REPLACE.with(|c| *c.borrow()));
    }
}
