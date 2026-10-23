//! he_check he.g2p < sentences — prints `text<TAB>c<TAB>v<TAB>s<TAB>ipa` per line
//! (the tagger's per-character labels in tools/he_distill.py encoding), for
//! Python<->Rust parity tests. With --time it prints per-sentence timing to stderr.
use std::io::{self, BufRead};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let p = args.get(1).expect("model path");
    let timing = args.iter().any(|a| a == "--time");
    let m = polyg2p::tagger::Tagger::open(std::path::Path::new(p)).expect("load");
    let h = polyg2p::he::Hebrew::from_model(m);
    let mut total = 0f64;
    let mut n = 0usize;
    let mut chars = 0usize;
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let t0 = Instant::now();
        let (t, c, v, s) = h.label_codes(&line).unwrap();
        let ipa = h.phonemize(&line);
        let dt = t0.elapsed().as_secs_f64();
        total += dt / 2.0;
        n += 1;
        chars += t.chars().count();
        println!("{t}\t{c}\t{v}\t{s}\t{ipa}");
    }
    if timing {
        eprintln!("{n} sentences, {chars} chars: {:.2} ms/sentence, {:.1} us/char", total / n as f64 * 1e3, total / chars as f64 * 1e6);
    }
}
