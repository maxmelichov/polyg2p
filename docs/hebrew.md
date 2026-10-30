# Hebrew

Hebrew is written without vowels, so a Hebrew G2P cannot be a per-word
lexicon + rules engine: the same skeleton (`ספר`) is `sˈefeʁ`, `sˈapaʁ`,
`safˈaʁ` or `sipˈeʁ` depending on the sentence. The native engine is therefore a
**sentence-level character tagger** distilled from
[RenikudPlus](https://github.com/maxmelichov/RenikudPlus) (a ~310 MB BERT-style
ONNX model, 86 % word accuracy on its benchmark), and it emits **exactly
RenikudPlus's IPA notation**, which the BlueTTS Hebrew models are trained on.

```
text ──► numbers ──► normalize ──► windows ──► tagger ──► constrained decode ──► render ──► IPA
        (he/num.rs)  (geresh, NFD, (≤300 chars,  (src/tagger.rs)  (legal consonant per letter,   (RenikudPlus rules)
                      strip niqqud) sentence/clause/word)         one stress per word on a vowel)
```

## Label space (the teacher's own)

Per Hebrew letter the model predicts three labels:

| head      | classes | meaning |
|-----------|---------|---------|
| consonant | 25      | `∅ b v d h z χ t j k l m n s f p ts tʃ w ʔ ɡ ʁ ʃ ʒ dʒ` — dagesh/rafe (ב→b/v, כ→k/χ, פ→p/f), shin/sin (ש→ʃ/s/∅), silent matres (ו/י/א/ה/ע→∅), foreign (ג׳→dʒ …) |
| vowel     | 6       | `∅ a e i o u` — the vowel *after* the letter (a mater lectionis is `∅/∅`, its vowel sits on the previous letter) |
| stress    | 2       | the one stressed letter of the word |

Rendering (`he::render`) is a line-by-line port of RenikudPlus's
`_phonemize_window`: per letter `[consonant][ˈ][vowel]`, furtive patah on
word-final `ח` (`ˈaχ`), the geresh map (`ג' ז' צ' ץ'` → `dʒ ʒ tʃ tʃ`), `'` and
`"` dropped, every other character passed through. `tests/he_render.rs` checks
that the renderer reproduces the teacher's IPA byte-for-byte from the teacher's
own labels on 300 sentences (`data/he_fixture.tsv`).

The decode is the greedy+constraints variant of the teacher's exact-MAP cascade:
consonant = argmax over the letter's legal classes, vowel = argmax, and per
whitespace word the stress goes to the vowel-bearing letter with the largest
stress margin. Because the student is trained on the teacher's *decoded*
labels, the cascade conditioning weights are not needed.

## Data

* **Corpus**: `knesset_nikud_v6` (5.3 M vocalized Knesset sentences). Niqqud,
  cantillation and morpheme bars are stripped, sentences deduplicated, 8–300
  characters kept, shuffled (seed 0) → 2.67 M unvocalized sentences
  (`build-data/he_sents.txt`).
* **Teacher labels**: `tools/he_distill.py label` runs RenikudPlus
  (`model.onnx`, exact-MAP decode, number front end on) over the first N
  sentences with 4 worker processes (the exported graph only takes batch 1;
  ~14 sentences/s on an M5 shared with other jobs). Each row stores the exact
  string the teacher saw, the three label strings and the IPA.
* **Held-out**: the first 2 000 rows (never trained on).
* **Gold**: `heb-tts-benchmark/saspeech_100_gold_gt.csv` (100 hand-annotated
  SASPEECH sentences), used only for the final evaluation.

## Student

`tools/train_he.py`: character embedding + learned positions, 6 pre-LN
transformer encoder blocks (d = 256, 4 heads, ReLU FFN 1024 — the encoder block
of `train_g2p.py`), final LayerNorm, three linear heads. Loss = sum of the three
cross-entropies (label smoothing 0.05) masked to Hebrew letters; AdamW +
OneCycle on `mps`. 25 % of the training windows are 2–3 sentences packed
together so multi-sentence windows are in-distribution. 4.9 M parameters.

Export (`G2PT`): `b"G2PT" | u32 header_len | JSON header | tensors`. The
header carries the config, the character vocabulary, the class names and the
per-letter legal consonant sets; tensors are f32 (19.6 MB) or, with `--int8`,
int8 weights with per-output-row f32 scales (5.4 MB). On aarch64 with the
dot-product extension `tagger.rs` keeps them int8 and runs the linear layers
through an int8 `sdot` kernel over per-token quantized activations (3× faster,
held-out agreement unchanged: 97.44 % vs 97.43 %); elsewhere, or with
`POLYG2P_INT8=0`, they are dequantized at load. The engine loads
`he-int8.g2p` when the data directory has one, else `he.g2p`.

## Runtime

`src/tagger.rs` is a dependency-free encoder: every f32 matrix product (the
linears of f32 models *and* `Q·Kᵀ` / `P·V`) goes through one cache-blocked
`A·Bᵀ` kernel (4×4 register tile, NEON intrinsics on aarch64, portable
fallback elsewhere); int8 linears use the `sdot` kernel described above.
Inference is single-threaded and deterministic; `G2p::phonemize_batch`
parallelizes across sentences.

Non-Hebrew spans: digits are expanded by `src/he/num.rs`, a port of
`hebrew-num2words` (integers, years, ordinals `ה-N`, percent, decimals, clock
times, dates, phone numbers, thousands separators, gender agreement with the
following noun); Latin-script words pass through unchanged like the teacher, or
go to a callback (`Hebrew::with_latin`, like `ru.rs`). Punctuation passes through.

A smaller variant (`he-small.g2p`: d = 192, 4 layers, 1.9 M params, 2.5× faster,
~2 points lower agreement) is trained with
`--d 192 --layers 4 --ff 768`.

Model file: `he.g2p` in the data directory (`G2p::new(Some(dir))`,
`$POLYG2P_DATA` in Python, or `$POLYG2P_HE_MODEL` to point at a file). Without
it `phonemize` returns an empty string and warns once.

## Results

See [results/he.md](results/he.md).

## Retraining

```
uv venv .venv-he -p 3.12
uv pip install -p .venv-he/bin/python -e /path/to/RenikudPlus onnxruntime torch numpy
.venv-he/bin/python tools/he_distill.py prep knesset_nikud_v6.txt --out build-data/he_sents.txt
.venv-he/bin/python tools/he_distill.py label build-data/he_sents.txt build-data/he_labels.jsonl --end 300000 --procs 4
.venv-he/bin/python tools/train_he.py build-data/he_labels.jsonl build-data/run1 --epochs 10 --int8
cp build-data/run1/he.g2p data-out/he.g2p
.venv-he/bin/python tools/he_parity.py build-data/run1/he.pt build-data/run1/vocab.json data-out/he.g2p sentences.txt
.venv-he/bin/python tools/he_eval.py --data data-out --teacher
```
