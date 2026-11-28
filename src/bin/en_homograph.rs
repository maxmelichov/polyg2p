//! Tool: `en_homograph [--data DIR] [--units [--ids N] | --logits]` reads
//! `sentence <TAB> start <TAB> end` lines (byte offsets of one word in the sentence).
//!
//! * default: the phonemes that word gets in the sentence, or an empty line when the word
//!   cannot be located in the output (tools/en_homograph_bench.py).
//! * `--units`: `target_index <TAB> unit <TAB> unit ...`, the input of the heteronym model
//!   (`^` in front of a capitalised word); `--ids N` prints each unit's feature bucket ids
//!   (n_buckets = N) as `id,id,...` instead of its text. Used by tools/train_homograph.py.
//! * `--bench FILE [--passes N]`: one sentence per line; times `phonemize_words` over the file N times
//!   (single thread; pin it with `taskset -c`) and prints `occurrences<TAB>us_per_pass<TAB>us_model_only`,
//!   where the last column times the heteronym model alone on every covered word (0 without a model).
//! * `--logits`: `pronunciation:logit ...` of the loaded heteronym model for that word
//!   (parity check against PyTorch); empty when the word is not covered.
use std::io::{self, BufRead, BufWriter, Write};

use polyg2p::text::{Tok, tokenize};

fn parts(w: &str) -> impl Iterator<Item = &str> {
    w.split(|c| c == '-' || c == '\'').filter(|p| !p.is_empty())
}

fn bench(g: &polyg2p::G2p, file: &str, passes: usize) {
    let sents: Vec<String> = std::fs::read_to_string(file).unwrap().lines().map(|l| l.to_string()).collect();
    let t0 = std::time::Instant::now();
    g.english(); // load (and, for a model file, open) outside the timed passes
    let load_us = t0.elapsed().as_secs_f64() * 1e6;
    let mut times = Vec::new();
    for _ in 0..passes {
        let t = std::time::Instant::now();
        let mut n = 0usize;
        for s in &sents {
            n += g.phonemize_words(s, polyg2p::Lang::En).len();
        }
        std::hint::black_box(n);
        times.push(t.elapsed().as_secs_f64() * 1e6);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // the model alone, on every covered word of the file
    let inputs: Vec<(Vec<polyg2p::homograph::Unit>, Vec<usize>)> = sents
        .iter()
        .map(|s| {
            let (u, words) = g.heteronym_units(s);
            let t: Vec<usize> = words.iter().filter(|(w, at)| *at != usize::MAX && g.english().heteronym_covers(w)).map(|(_, at)| *at).collect();
            (u, t)
        })
        .collect();
    let occ: usize = inputs.iter().map(|(_, t)| t.len()).sum();
    let mut model_us = 0.0;
    if occ > 0 {
        let mut ts = Vec::new();
        for _ in 0..passes {
            let t = std::time::Instant::now();
            for (u, tg) in &inputs {
                for &k in tg {
                    std::hint::black_box(g.english().heteronym_scores(u, k));
                }
            }
            ts.push(t.elapsed().as_secs_f64() * 1e6);
        }
        ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        model_us = ts[ts.len() / 2];
    }
    println!("sentences\t{}\toccurrences\t{}\tload_us\t{:.0}\tpass_us\t{:.0}\tmodel_us_per_pass\t{:.0}\tmodel_us_per_occ\t{:.2}", sents.len(), occ, load_us, times[times.len() / 2], model_us, if occ > 0 { model_us / occ as f64 } else { 0.0 });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |f: &str| args.iter().position(|a| a == f);
    let data = flag("--data").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);
    if let Some(i) = flag("--bench") {
        let passes = flag("--passes").and_then(|i| args.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(20);
        let g = polyg2p::G2p::new(data.as_deref());
        bench(&g, &args[i + 1], passes);
        return;
    }
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
                        // a hyphenated word has one unit per part, `-` between them (homograph::units)
                        found = if *src == target {
                            Some(*at)
                        } else {
                            polyg2p::homograph::hyphen_parts(src).position(|p| p == target).map(|i| at + 2 * i)
                        };
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
