//! nn_check model.bin < words  — prints model predictions (for parity tests).
use std::io::{self, BufRead};
fn main() {
    let p = std::env::args().nth(1).expect("model path");
    let m = polyg2p::nn::Model::open(std::path::Path::new(&p)).expect("load");
    for line in io::stdin().lock().lines() {
        let w = line.unwrap();
        println!("{}", m.predict(w.trim()));
    }
}
