//! CLI: `g2p <lang> [--data DIR] [--words]` reads lines on stdin, writes phonemes.
//! With --words each input line is a single word and is phonemized in isolation.
//! Piped input is phonemized in chunks on all cores (order preserved); a terminal
//! is answered line by line.
use std::io::{self, BufRead, BufWriter, IsTerminal, Write};

const CHUNK: usize = 2048;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lang = args.get(1).and_then(|l| polyg2p::Lang::parse(l)).unwrap_or_else(|| {
        eprintln!("usage: g2p <en|es|it|de|ru|he|...> [--data DIR] [--words]");
        std::process::exit(2)
    });
    let data = args.iter().position(|a| a == "--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let g = polyg2p::G2p::new(data.as_deref());
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();
    let mut out = BufWriter::new(io::stdout().lock());
    let mut lines = stdin.lock().lines();
    loop {
        let mut chunk: Vec<String> = Vec::with_capacity(if interactive { 1 } else { CHUNK });
        while chunk.len() < chunk.capacity() {
            match lines.next() {
                Some(l) => chunk.push(l.unwrap()),
                None => break,
            }
        }
        if chunk.is_empty() {
            break;
        }
        for ph in g.phonemize_batch(&chunk, lang) {
            writeln!(out, "{ph}").unwrap();
        }
        if interactive {
            out.flush().unwrap();
        }
    }
}
