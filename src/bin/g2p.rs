//! CLI: `g2p <lang> [--data DIR] [--words]` reads lines on stdin, writes phonemes.
//! With --words each input line is a single word and is phonemized in isolation.
use std::io::{self, BufRead, BufWriter, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lang = args.get(1).and_then(|l| polyg2p::Lang::parse(l)).unwrap_or_else(|| {
        eprintln!("usage: g2p <en|es|it|de|ru|he|...> [--data DIR] [--words]");
        std::process::exit(2)
    });
    let data = args.iter().position(|a| a == "--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let g = polyg2p::G2p::new(data.as_deref());
    let stdin = io::stdin();
    let mut out = BufWriter::new(io::stdout().lock());
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        writeln!(out, "{}", g.phonemize(&line, lang)).unwrap();
    }
}
