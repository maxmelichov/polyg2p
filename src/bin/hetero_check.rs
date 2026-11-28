//! Evaluation and data dump for the heteronym context models (src/hetero.rs).
//!
//!   hetero_check <lang> [--data DIR] [--raw]  stdin: sentence TAB word TAB nth   -> rule_k TAB model_k TAB phonemes of every class
//!                                             (model_k = the class that ships: the rule's for words with mode=rule; --raw = the model's own)
//!   hetero_check <lang> --dump [--data DIR]   stdin: sentence                    -> one JSON line per sentence
//!
//! `nth` is the 0-based occurrence of `word` among the engine's own words of the
//! sentence. `--dump` prints the encoder input, the engine's words with their
//! character offsets, and the table words with the class the rules produced.
use std::io::{self, BufRead, BufWriter, Write};

fn esc(s: &str) -> String {
    serde_json::to_string(s).unwrap()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lang = args.get(1).and_then(|l| polyg2p::Lang::parse(l)).unwrap_or_else(|| {
        eprintln!("usage: hetero_check <lang> [--dump] [--data DIR]");
        std::process::exit(2)
    });
    let data = args.iter().position(|a| a == "--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let dump = args.iter().any(|a| a == "--dump");
    let raw = args.iter().any(|a| a == "--raw");
    let g = polyg2p::G2p::new(data.as_deref());
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        if dump {
            polyg2p::hetero::capture_start(false);
            let _ = g.phonemize(&line, lang);
            let c = polyg2p::hetero::capture_take();
            let words: Vec<String> = c.words.iter().map(|(o, w, p)| format!("[{o},{},{}]", esc(w), esc(p))).collect();
            let occ: Vec<String> = c
                .decisions
                .iter()
                .map(|d| format!("[{},{},{}]", d.word_index, esc(&d.word), d.rule_k.map(|k| k.to_string()).unwrap_or("null".into())))
                .collect();
            writeln!(out, "{{\"t\":{},\"w\":[{}],\"occ\":[{}]}}", esc(&c.text), words.join(","), occ.join(",")).unwrap();
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            writeln!(out, "-\t-\t").unwrap();
            continue;
        }
        let nth: usize = f[2].parse().unwrap_or(0);
        let ds = g.hetero_decisions(f[0], lang);
        let d = ds.iter().filter(|d| d.word == f[1]).nth(nth);
        let fmt = |k: Option<u8>| k.map(|k| k.to_string()).unwrap_or("-".into());
        match d {
            Some(d) => {
                let mut phs = d.phs.clone();
                phs.sort();
                let phs: Vec<String> = phs.iter().map(|(k, p)| format!("{k}={p}")).collect();
                let shipped = if d.mode_rule && !raw && d.model_k.is_some() { d.rule_k } else { d.model_k };
                writeln!(out, "{}\t{}\t{}", fmt(d.rule_k), fmt(shipped), phs.join("|")).unwrap()
            }
            None => writeln!(out, "-\t-\t").unwrap(),
        }
    }
}
