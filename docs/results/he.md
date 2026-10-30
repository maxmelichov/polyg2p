# Hebrew results

Teacher: RenikudPlus 0.6 (`model.onnx`, exact-MAP decode, default settings).
Student: `data-out/he.g2p` (d = 256, 6 layers, 4 heads, FFN 1024; 4.89 M
parameters), trained on 998 000 Knesset sentences labelled by the teacher
(20 epochs on the first 50 k, then 4 epochs on all 1 M, resumed). Evaluation:
`tools/he_eval.py` (word accuracy = IPA words equal after stripping edge
punctuation, difflib-aligned when the word counts differ).

## Held-out teacher agreement (2 000 sentences, 24 779 words, never trained on)

| model | word agreement | sentence byte-identical | per-letter cons / vowel / stress |
|---|---:|---:|---|
| student f32 `he.g2p` (19.6 MB) | **97.43 %** | 75.5 % | 99.88 / 99.35 / 99.82 % |
| student int8 `he-int8.g2p` (5.4 MB) | 97.4 % (99.1 % of sentences identical to f32) | | |
| `he-int8.g2p` through the int8 `sdot` kernel (aarch64 default, 3× faster) | 97.44 % | 75.7 % | gold 86.70 % (same as f32) |
| 50 k-sentence checkpoint (20 epochs) | 94.08 % | 53.6 % | 99.72 / 98.45 / 99.60 % |

Training curve on the 1 M run (held-out word agreement): 94.5 → 95.6 (ep 1)
→ 96.75 (ep 2) → 97.24 (ep 3) → 97.43 (ep 4); still rising slowly.

## Gold benchmark (heb-tts-benchmark `saspeech_100_gold_gt.csv`, 100 sentences, 1 346 words, hand-annotated IPA)

| system | word accuracy vs human | sentence exact |
|---|---:|---:|
| RenikudPlus (teacher) | **91.68 %** | 40 % |
| polyg2p student `he.g2p` | 86.63 % | 17 % |
| student vs teacher on this set | 91.90 % agreement | 34 % |

The gold set is podcast speech (SASPEECH); the student is distilled from
Knesset transcripts only, so its agreement with the teacher drops from 97.4 %
in-domain to 91.9 % here, and it loses 5 points of gold accuracy to the
teacher. Typical misses: foreign names (`ג'ורג'` → `dʒʁdʒˈa`, teacher
`dʒˈoʁdʒ`), rare vowel patterns (`beʔikavˈot` for `beʔikvˈot`), colloquial
forms.

## Python ↔ Rust parity

`tools/he_parity.py` on 1 000 unseen sentences (54 194 Hebrew letters): Rust
labels identical to the torch model's constrained decode on **1 000 / 1 000
sentences, 54 194 / 54 194 letters**; the Rust CLI's held-out score equals the
Python evaluator's to four digits.

## Speed (Apple M5, single thread, release build, quiet machine)

| | per sentence (avg 68 chars) | per character |
|---|---:|---:|
| `he.g2p` f32 | 8.3 ms | 122 µs |
| `he-int8.g2p` (dequantized at load, same compute) | 8.2 ms | 121 µs |
| RenikudPlus (onnxruntime, 10 threads) | ~80 ms | |

The kernel runs at ~75 GFLOP/s; the model costs 9.4 MFLOP per character, so a
68-character sentence is ~0.64 GFLOP. `G2p::phonemize_batch` parallelizes
across sentences. The "≪ 10 ms per sentence" target is met only for short
sentences; see the small model below and *Open issues*.

## Sizes

| file | bytes |
|---|---:|
| `he.g2p` (f32) | 19 557 420 |
| `he-int8.g2p` | 5 435 024 |
| RenikudPlus `model.onnx` | ~310 MB |

## Small model `he-small.g2p` (d = 192, 4 layers, FFN 768, 1.89 M params)

3 epochs on the same 1 M rows from scratch. Select it with
`POLYG2P_HE_MODEL=/path/to/he-small.g2p`.

| | small | default |
|---|---:|---:|
| held-out word agreement with the teacher | 95.59 % | 97.43 % |
| held-out sentence byte-identical | 63.4 % | 75.5 % |
| gold word accuracy vs human | 84.77 % | 86.63 % |
| per sentence (68 chars) / per char | **3.3 ms / 49 µs** | 8.3 ms / 122 µs |
| f32 / int8 file | 7.6 MB / 2.3 MB | 19.6 MB / 5.4 MB |

## Open issues

* Domain: labels come from Knesset only. Labelling conversational / news /
  web text (the teacher is the only cost: ~18 sentences/s) should close most of
  the gold gap; names and loanwords are the weakest class.
* Latency: 8 ms for a 68-char sentence single-threaded. Options: the small
  model, an int8 dot-product kernel (`sdot`) for ~3× on the matmuls, or a
  persistent thread pool (a scoped-thread version was measured: spawn overhead
  cancelled the gain, so it was not kept).
* Decode is greedy+constraints, not the teacher's exact-MAP cascade; since the
  student is trained on decoded labels this costs nothing measurable, but
  `niqqud="use"` (pointed input as constraints) is not supported — points are
  stripped like the teacher's default.
* Numbers: `\d` in the Python front end matches non-ASCII digits; the Rust port
  only expands ASCII digits.
