//! build_lex in.tsv out.lex — TSV of `word<TAB>value[<TAB>value...]`; values are
//! stored tab-joined (the first is the default pronunciation).
use std::io::{BufRead, BufReader, BufWriter};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 3 {
        eprintln!("usage: build_lex in.tsv out.lex");
        std::process::exit(2);
    }
    let f = BufReader::new(std::fs::File::open(&a[1]).expect("open input"));
    let mut map: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for line in f.lines() {
        let line = line.unwrap();
        let mut it = line.splitn(2, '\t');
        let (Some(w), Some(v)) = (it.next(), it.next()) else { continue };
        let e = map.entry(w.to_string()).or_default();
        for x in v.split('\t') {
            if !e.iter().any(|y| y == x) {
                e.push(x.to_string());
            }
        }
    }
    let entries: Vec<(String, String)> = map.into_iter().map(|(k, v)| (k, v.join("\t"))).collect();
    let n = entries.len();
    let out = BufWriter::new(std::fs::File::create(&a[2]).expect("create output"));
    polyg2p::lexicon::build(entries, out).expect("build");
    eprintln!("{n} entries -> {}", a[2]);
}
