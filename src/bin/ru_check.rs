//! Parity driver for the Russian G2P (`src/ru.rs`).
//!
//!     ru_check                     # stdin: one '+'-stressed word per line -> word_to_ipa(word, None)
//!     ru_check --sent [--lex DIR]  # stdin: one sentence per line -> Russian::phonemize
//!     ru_check --report [--lex DIR]  # as --sent, then "word<TAB>tier<TAB>ipa" lines and a blank line
//!     ru_check --sent --latin-keep  # Latin words kept verbatim (= Python without eSpeak)
//!     ru_check --bench N           # time N passes of word mode over stdin (stderr: words/s)
//!
//! Output is one line per input line (newlines inside results cannot occur).
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

#[allow(unused_imports)]
use polyg2p::lexicon;

#[allow(dead_code)]
#[path = "../ru.rs"]
mod ru;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut mode = "word";
    let mut lex: Option<PathBuf> = None;
    let mut bench = 0usize;
    let mut latin_keep = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--sent" => mode = "sent",
            "--report" => mode = "report",
            "--latin-keep" => latin_keep = true,
            "--lex" => {
                i += 1;
                lex = args.get(i).map(PathBuf::from);
            }
            "--bench" => {
                i += 1;
                bench = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1);
            }
            a => {
                eprintln!("unknown argument {a}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let lines: Vec<String> = std::io::stdin().lock().lines().map(|l| l.expect("utf-8 input")).collect();
    let load = || {
        let r = ru::Russian::load(lex.as_deref());
        if latin_keep { r.with_latin(|s| s.to_string()) } else { r }
    };
    let out = std::io::stdout();
    let mut out = BufWriter::new(out.lock());
    if bench > 0 {
        let ru = (mode != "word").then(load);
        let t = Instant::now();
        let mut n = 0usize;
        for _ in 0..bench {
            for l in &lines {
                let r = match &ru {
                    None => ru::word_to_ipa(l, None),
                    Some(r) => r.phonemize(l),
                };
                n += r.len();
            }
        }
        let dt = t.elapsed().as_secs_f64();
        let items = (lines.len() * bench) as f64;
        eprintln!("{} items in {:.3}s: {:.0} items/s, {:.2} us/item (checksum {n})", items, dt, items / dt, dt * 1e6 / items);
        return;
    }
    match mode {
        "word" => {
            for l in &lines {
                writeln!(out, "{}", ru::word_to_ipa(l, None)).unwrap();
            }
        }
        _ => {
            let r = load();
            for l in &lines {
                if mode == "sent" {
                    writeln!(out, "{}", r.phonemize(l)).unwrap();
                } else {
                    let (s, rep) = r.phonemize_report(l, None);
                    writeln!(out, "{s}").unwrap();
                    for (w, t, p) in rep {
                        writeln!(out, "{w}\t{t}\t{p}").unwrap();
                    }
                    writeln!(out).unwrap();
                }
            }
        }
    }
}
