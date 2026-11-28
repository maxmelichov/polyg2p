//! am_gem_check MODEL|- [TABLE.tsv] < sentences — prints `sentence<TAB>long marks<TAB>ipa` per line, for
//! Python<->Rust parity tests of the Amharic gemination layer (tools/am_gem_parity.py).
//! MODEL is a G2PT tagger (`-` = none); TABLE is an am-gem.tsv. Without both the output is the plain reading.
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = args.get(1).expect("model path or -");
    let m = if model == "-" { None } else { Some(polyg2p::tagger::Tagger::open(std::path::Path::new(model)).expect("load")) };
    let table = args.get(2).map(|p| std::fs::read_to_string(p).expect("table"));
    let gem = polyg2p::am::Gem::from_parts(m, table.as_deref());
    let g = polyg2p::G2p::new(None).with_amharic(gem);
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let ipa = g.phonemize(&line, polyg2p::Lang::Am);
        println!("{line}\t{}\t{ipa}", ipa.matches('ː').count());
    }
}
