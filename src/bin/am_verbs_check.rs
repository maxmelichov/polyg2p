//! am_verbs_check AM-VERBS.tsv < words — prints `word<TAB>comma list of long letter indices` per line
//! (verb rules only, no word table), for the parity test against tools/am_verbs_pack.py.
use std::io::{self, BufRead};

fn main() {
    let pack = std::fs::read_to_string(std::env::args().nth(1).expect("am-verbs.tsv")).expect("pack");
    let gem = polyg2p::am::Gem::from_packs(None, Some(&pack));
    for line in io::stdin().lock().lines() {
        let w = line.unwrap();
        let f = gem.flags(&[w.as_str()]).remove(0);
        let idx: Vec<String> = f.iter().enumerate().filter(|&(_, &x)| x).map(|(i, _)| i.to_string()).collect();
        println!("{w}\t{}", idx.join(","));
    }
}
