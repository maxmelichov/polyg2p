//! Tool: `en_homograph [--data DIR]` reads `sentence <TAB> start <TAB> end` lines (byte
//! offsets of one word in the sentence) and prints the phonemes that word gets in the
//! sentence, or an empty line when the word cannot be located in the output.
//! Used by tools/en_homograph_bench.py.
use std::io::{self, BufRead, BufWriter, Write};

use polyg2p::text::{Tok, tokenize};

fn parts(w: &str) -> impl Iterator<Item = &str> {
    w.split(|c| c == '-' || c == '\'').filter(|p| !p.is_empty())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = args.iter().position(|a| a == "--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let g = polyg2p::G2p::new(data.as_deref());
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut f = line.split('\t');
        let (Some(sent), Some(a), Some(b)) = (f.next(), f.next().and_then(|x| x.parse::<usize>().ok()), f.next().and_then(|x| x.parse::<usize>().ok())) else {
            writeln!(out).unwrap();
            continue;
        };
        if !(a < b && b <= sent.len() && sent.is_char_boundary(a) && sent.is_char_boundary(b)) {
            writeln!(out).unwrap();
            continue;
        }
        let target = sent[a..b].to_lowercase();
        // which occurrence of the word this is, counting the parts of hyphenated words
        let nth = tokenize(&sent[..a])
            .iter()
            .map(|t| match t {
                Tok::Word(w) => parts(&w.to_lowercase()).filter(|p| *p == target).count(),
                _ => 0,
            })
            .sum::<usize>();
        let mut seen = 0;
        let mut ph = String::new();
        'find: for (src, p) in g.phonemize_words(sent, polyg2p::Lang::En) {
            let n = parts(&src).count();
            for (i, part) in parts(&src).enumerate() {
                if part != target {
                    continue;
                }
                if seen == nth {
                    let pieces: Vec<&str> = p.split(' ').collect();
                    // a hyphenated word is phonemized part by part
                    ph = if src.contains('-') && pieces.len() == n { pieces[i].to_string() } else { p.clone() };
                    break 'find;
                }
                seen += 1;
            }
        }
        writeln!(out, "{ph}").unwrap();
    }
}
