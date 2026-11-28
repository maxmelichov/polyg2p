//! am_gem_check [TABLE.tsv] < sentences — prints `sentence<TAB>long marks<TAB>ipa` per line, for
//! parity tests of the Amharic gemination layer (tools/am_gem_parity.py).
//! TABLE is an am-gem.tsv; without it the output is the plain reading.
use std::io::{self, BufRead};

fn main() {
    let table = std::env::args().nth(1).map(|p| std::fs::read_to_string(p).expect("table"));
    let gem = polyg2p::am::Gem::from_table(table.as_deref());
    let g = polyg2p::G2p::new(None).with_amharic(gem);
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let ipa = g.phonemize(&line, polyg2p::Lang::Am);
        println!("{line}\t{}\t{ipa}", ipa.matches('ː').count());
    }
}
