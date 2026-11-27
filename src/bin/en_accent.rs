//! Tool: `en_accent <code> [--data DIR] [--rules]` reads one word per line and prints
//! `word <TAB> American form <TAB> accent form <TAB> H when the word is a homograph`.
//! `--rules` leaves the en-gb word list out (what the rules alone give). Used by tools/en_gb_build.py
//! and tools/en_accent_eval.py.
use std::io::{self, BufRead, BufWriter, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let acc = args.get(1).and_then(|l| polyg2p::Lang::parse(l)).and_then(|l| l.accent()).unwrap_or_else(|| {
        eprintln!("usage: en_accent <en-gb|en-au|en-nz|en-in|en-sc|en-ie|en-ca> [--data DIR] [--rules]");
        std::process::exit(2)
    });
    let data = args.iter().position(|a| a == "--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let g = polyg2p::G2p::new(data.as_deref());
    let en = g.english();
    let use_lex = !args.iter().any(|a| a == "--rules");
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let w = line.unwrap();
        let lw = w.trim().to_lowercase();
        let ga = en.citation(&lw);
        let (ph, _) = polyg2p::en_accent::word_at(en, &ga, &lw, acc, false, use_lex);
        writeln!(out, "{w}\t{ga}\t{ph}\t{}", if en.homograph_word(&lw) { "H" } else { "" }).unwrap();
    }
}
