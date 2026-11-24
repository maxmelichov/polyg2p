//! Tool: `en_accent <code> [--data DIR]` reads one word per line and prints
//! `word <TAB> American form <TAB> accent form`. Used by tools/en_gb_build.py
//! and tools/en_accent_eval.py.
use std::io::{self, BufRead, BufWriter, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let acc = args.get(1).and_then(|l| polyg2p::Lang::parse(l)).and_then(|l| l.accent()).unwrap_or_else(|| {
        eprintln!("usage: en_accent <en-gb|en-au|en-nz|en-za|en-in|en-sc|en-ie|en-ca> [--data DIR]");
        std::process::exit(2)
    });
    let data = args.iter().position(|a| a == "--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let g = polyg2p::G2p::new(data.as_deref());
    let en = g.english();
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let w = line.unwrap();
        let lw = w.trim().to_lowercase();
        let ga = en.citation(&lw);
        let (ph, _) = polyg2p::en_accent::word(en, &ga, &lw, acc);
        writeln!(out, "{w}\t{ga}\t{ph}").unwrap();
    }
}
