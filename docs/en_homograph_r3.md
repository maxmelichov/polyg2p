# English heteronym model, round 3

`en_homograph.g2p` (optional, never shipped with the crate) decides the reading of the 162 table words in `data/en_homograph.tsv` from the
whole sentence. Round 3 replaces the hashed-bag students of the earlier rounds by `bag3`: integer-hashed features (no strings), word-class
features from the closed-class table `data/en_classes.tsv`, one hidden layer, int8 embedding rows. The file format is `features: "v3"`
(`src/homograph.rs`); older v1 / v2 files still load.

Training: the gold train split, author-written silver and seed sentences, and a pool of public text labelled by two fine-tuned encoders that
had to agree (see `docs/en_homograph_licence_notes.txt` and `tools/homograph_*.py`). A global prior-shaped bias on each word's majority
reading (lambda 0.75, 0.375 for words in the selection set) is folded into `out.bias`.

Frozen file: sha256 `aef6e122117e8029e04744dae4d9a57d2d7e3dad042a147874522fbe9a936164` (int8, 7.0 MB).

| | rules only | round-1 model | round-3 file | encoder teacher |
|---|---|---|---|---|
| Wikipedia Homograph eval, micro | 71.3 | 92.5 | 96.1 | 98.6 |
| macro | 71.3 | 92.5 | 96.2 | 98.6 |
| words below the majority baseline (of 162) | | | 3 | |
| `tests/en_homograph_cases.tsv` | 72.4 | 86.2 | 92.3 (4/4 must) | 96.1 |
| `tests/en_homograph_cases2.tsv` | 70.3 | 81.1 | 91.4 | 96.4 |

Cost: about 7 microseconds per covered occurrence on one core; 20,000 Tatoeba sentences through the CLI, wall x1.04 and CPU x1.08 of the
run without the file. Sentences without a table word build no units. Without the file the output is byte-identical to before; the other
languages are byte-identical.
