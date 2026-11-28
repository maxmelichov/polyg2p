//! am_gem_check [TABLE.tsv [AM-VERBS.tsv]] < sentences — prints `sentence<TAB>long marks<TAB>ipa` per line, for
//! parity tests of the Amharic gemination layer (tools/am_gem_parity.py).
//! TABLE is an am-gem.tsv, AM-VERBS an am-verbs.tsv (`-` for no table); without them the output is the plain reading.
use std::io::{self, BufRead};

fn main() {
    let read = |n: usize| std::env::args().nth(n).filter(|p| p != "-").map(|p| std::fs::read_to_string(p).expect("pack"));
    let (table, verbs) = (read(1), read(2));
    let gem = polyg2p::am::Gem::from_packs(table.as_deref(), verbs.as_deref());
    let g = polyg2p::G2p::new(None).with_amharic(gem);
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let ipa = g.phonemize(&line, polyg2p::Lang::Am);
        println!("{line}\t{}\t{ipa}", ipa.matches('ː').count());
    }
}
