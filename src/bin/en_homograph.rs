//! Tool: `en_homograph [--data DIR] [--units [--ids N] | --logits]` reads
//! `sentence <TAB> start <TAB> end` lines (byte offsets of one word in the sentence).
//!
//! * default: the phonemes that word gets in the sentence, or an empty line when the word
//!   cannot be located in the output (tools/en_homograph_bench.py).
//! * `--units`: `target_index <TAB> unit <TAB> unit ...`, the input of the heteronym model
//!   (`^` in front of a capitalised word); `--ids N` prints each unit's feature bucket ids
//!   (n_buckets = N) as `id,id,...` instead of its text. Used by tools/train_homograph.py.
//! * `--logits`: `pronunciation:logit ...` of the loaded heteronym model for that word
//!   (parity check against PyTorch); empty when the word is not covered.
use std::io::{self, BufRead, BufWriter, Write};

use polyg2p::text::{Tok, tokenize};

fn parts(w: &str) -> impl Iterator<Item = &str> {
    w.split(|c| c == '-' || c == '\'').filter(|p| !p.is_empty())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |f: &str| args.iter().position(|a| a == f);
    let data = flag("--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    let ids = flag("--ids").and_then(|i| args.get(i + 1)).and_then(|n| n.parse::<usize>().ok());
    let (units_mode, logits_mode) = (flag("--units").is_some(), flag("--logits").is_some());
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
        if units_mode || logits_mode {
            let (units, words) = g.heteronym_units(sent);
            let mut seen = 0;
            let mut found = None;
            for (src, at) in &words {
                if parts(src).any(|p| p == target) {
                    if seen == nth {
                        found = (*src == target).then_some(*at);
                        break;
                    }
                    seen += parts(src).filter(|p| *p == target).count();
                }
            }
            let Some(t) = found else {
                writeln!(out).unwrap();
                continue;
            };
            if logits_mode {
                let s = g.english().heteronym_scores(&units, t).unwrap_or_default();
                writeln!(out, "{}", s.iter().map(|(p, l)| format!("{p}:{l:.6}")).collect::<Vec<_>>().join(" ")).unwrap();
            } else {
                let cols: Vec<String> = units
                    .iter()
                    .enumerate()
                    .map(|(i, u)| match ids {
                        Some(n) => polyg2p::homograph::feature_ids(u, i == t, n).iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                        None => u.display(),
                    })
                    .collect();
                writeln!(out, "{t}\t{}", cols.join("\t")).unwrap();
            }
            continue;
        }
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
