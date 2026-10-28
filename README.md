# polyg2p

Fast multilingual grapheme-to-phoneme conversion for TTS, written in Rust,
Apache-2.0. It writes the same IPA symbols espeak-ng does (after BlueTTS's
`normalize_text`), so models trained on espeak phonemes can switch without
retraining. It contains no espeak code or data.

| Language | Engine | What it does |
|---|---|---|
| English (en) | Rust | CMUdict lexicon in espeak notation (spelling-aware reduced vowels), context rules for homographs (read/read, record/record, live/live, …), weak forms and word joining, affix/compound decomposition, title abbreviations, ordinals, currency, tiny transformer for unknown words |
| Spanish (es) | Rust | Rules only: syllabification, orthographic stress, β/ð/ɣ allophones and cross-word effects |
| Italian (it) | Rust | Rules + stress and open/closed-vowel statistics, optional lexicon |
| German (de) | Rust | Lexicon (optional pack) + compound splitting + rules |
| French (fr) | Rust | Rules + ~900-entry exception lexicon: silent finals, e-caduc, nasal vowels, liaison and sandhi in `join` |
| Portuguese, Brazil (pt) | Rust | Rules + open/closed-vowel ending statistics, palatalization, epenthesis, sandhi |
| Catalan, Central (ca) | Rust | Rules + open/closed ending statistics, vowel reduction, sandhi in `join` |
| Romanian (ro) | Rust | Rules: glides vs hiatus, morphological stress overrides, exception list |
| Polish (pl) | Rust | Rules: soft series, nasal vowels by context, voicing assimilation, stress exceptions, loan respellings |
| Czech (cs) | Rust | Rules: palatalization, ř, assimilation, preposition merging, loanword stems |
| Ukrainian (uk) | Rust | Rules + stress statistics (stem/ending tables), optional `uk.lex` stress pack |
| Hungarian (hu) | Rust | Rules: assimilation, long consonants, `egy` rules, compounds |
| Greek (el) | Rust | Rules: palatals, synizesis, final-ν sandhi, σ-voicing |
| Turkish (tr) | Rust | Rules: vowel allophones, ğ, pre-stressing suffixes, lexical stress list |
| Dutch (nl) | Rust | Rules: syllable-openness vowel length, schwa morphology, prefix layering, assimilation; optional `nl.lex` |
| Finnish (fi) | Rust | Rules: weight-sensitive secondary stress, compound splitting, boundary gemination in `join` |
| Indonesian (id) | Rust | Rules + e/ə stem table, prefix peeling, stress |
| Swedish (sv) | Rust | Rules + ~1,400-entry table, loan-suffix stress, compound splitter; optional `sv.lex` |
| Russian (ru) | Rust | Exact port of [Russian-G2P](https://github.com/maxmelichov/Russian-G2P): Moscow-norm rules, stress tiers from mmap'd lexicons; Latin words go to the English engine |
| Hebrew (he) | Rust | Sentence-level character tagger (4.9M-param transformer encoder, `he.g2p`) distilled from [RenikudPlus](https://github.com/maxmelichov/RenikudPlus), same IPA notation; number front end ported from hebrew-num2words. See [docs/hebrew.md](docs/hebrew.md). Falls back to renikud-onnx in Python when `he.g2p` is absent |
| Yiddish (yi) | phonikud-yi | Routed to [phonikud-yi](https://github.com/maxmelichov/phonikud-yi-blue-tts) |

Arabic (ar) is scaffolded in `src/` and pending a context model like Hebrew's; see `docs/adding-a-language.md` and `docs/results/`.

## Results

All numbers on 2 CPU cores, single-threaded, against espeak-ng 1.52 through
`phonemizer` (the way BlueTTS calls it). Scripts are in `tools/`.

**Accuracy vs Wiktionary** (WikiPron; 20k most frequent words, frequency-weighted;
both outputs collapsed to a shared broad phone set so only real pronunciation
differences count, not notation; per-language collapse rules in `tools/collapse_*.py`):

| | polyg2p | espeak-ng |
|---|---|---|
| English, strict | **97.7%** | 96.7% |
| English, unstressed ɪ = ə ² | **98.6%** | 97.6% |
| Spanish | **99.5%** | 99.3% |
| Italian (rules + tables, no lexicon) | **97.0%** | 85.4% |
| German, with `de.lex` ¹ | **98.4%** | 96.4% |
| German, rules only | 92.1% | 96.4% |
| French (nasals scored) | **98.9%** | 97.7% |
| Dutch, rules only | **99.1%** | 96.8% |
| Portuguese (BR) | **99.6%** | 96.9% |
| Polish | **99.9%** | 91.5% |
| Romanian | **98.0%** | 97.8% |
| Catalan (Central gold from `tools/ca_gold.py`) | **93.9%** | 93.2% |
| Czech | **99.9%** | 96.4% |
| Ukrainian (segments / with stress, `uk.lex`) ¹ | **99.3%** / **98.8%** | 79.7% / 69.1% |
| Hungarian | **99.8%** | 95.2% |
| Greek | **99.3%** | 80.8% |
| Turkish | 99.7% | 99.7% |
| Indonesian | **99.6%** | 88.4% |
| Finnish | 99.5% | 99.5% |
| Swedish (3.4k gold words) | **97.9%** | 95.6% |

¹ These lexicons come from Wiktionary too, so the numbers are optimistic.
² CMUdict and Wiktionary disagree mostly on how reduced vowels are written.

**English words not in the dictionary** (3,000 CMUdict words held out of model
training; mostly names):

| | exact | phoneme error rate |
|---|---|---|
| polyg2p, default model (`en.g2p`, 0.6M params) | **70.4%** | **7.3%** |
| polyg2p, `POLYG2P_EN_MODEL=large` (1.2M params) | **74.6%** | **6.2%** |
| espeak-ng | 64.2% | 8.7% |

**English homographs in context** (40 sentences: read, record, live, close,
tear, lead, use, wind, present, object, estimate, separate, …; written by the
authors, so indicative only): polyg2p 40/40, espeak-ng 32/40.

**Russian:** exact port of Russian-G2P. Identical output to the Python
version on 845 gold words, 555k synthetic words and 28.8k sentences. It's about
25× faster per word, and the 3.19M-form stress dictionary is memory-mapped
instead of ~1.5 GB of RAM.

**Speed** (Universal Dependencies test sentences, one pass, including process start):

| | polyg2p | espeak-ng (batch) |
|---|---|---|
| English (web text, many names/URLs) | **11.6k sent/s** | 4.7k sent/s |
| Spanish | **10.9k sent/s** | 2.6k sent/s |
| Italian | **24.3k sent/s** | 4.6k sent/s |
| German | **24.4k sent/s** | 3.9k sent/s |
| Russian (vs the Python Russian-G2P) | **12k sent/s** | ~13 sent/s |

From Python, one English sentence takes about 11 µs (espeak per call: ~240 µs).
`phonemize_batch` uses all cores. Peak memory with lexicons loaded: under 20 MB.
Each unknown English word costs ~0.5 ms in the default model (~1.6 ms large);
results are cached and affix/compound splitting resolves many of them first.

## Use

### Python

```console
pip install polyg2p            # wheel from bindings/python (maturin build --release)
pip install renikud-onnx       # for Hebrew
```

```python
from polyg2p import Phonemizer, unknown_symbols

g2p = Phonemizer()
g2p("I have read the record, and I will record it again.", "en")
# 'aɪ hæv ɹˈɛd ðə ɹˈɛkɚd, ænd aɪ wɪl ɹɪkˈɔːɹd ɪt əɡˈɛn.'
g2p("Hello <es>hola amigo</es> world", "en", tags=True)   # BlueTTS-style <lang> tags
g2p.phonemize_batch(lines, "de")                          # all cores, for corpora
unknown_symbols(ipa)                                      # [] if inside the 256-token vocab
```

### BlueTTS

`bluetts-polyg2p.patch` replaces espeak in inference (`TextProcessor`), dataset
phonemization, `combine_datasets.py`, the training eval phrases, the
dependencies and the Dockerfile:

```console
cd BlueTTS && git apply ../polyg2p/bluetts-polyg2p.patch
```

Or swap only inference: `from polyg2p.bluetts import TextProcessor`.

### Rust

```rust
let g2p = polyg2p::G2p::new(Some(std::path::Path::new("data-out")));
let ipa = g2p.phonemize("Die Bundesregierung hat entschieden.", polyg2p::Lang::De);
```

CLI: `cargo build --release && echo "La città è bella." | target/release/g2p it --data data-out`

## Data files

`G2p::new(data_dir)` loads these lazily if present (`POLYG2P_DATA` in Python):

| File | Source | License | Without it |
|---|---|---|---|
| `en.lex` | CMUdict | BSD-style (notice in NOTICE) | English needs it |
| `en.g2p` | trained on CMUdict (small, default) | BSD-style | unknown words are spelled out |
| `en-large.g2p` | trained on CMUdict (larger) | BSD-style | used with `POLYG2P_EN_MODEL=large` |
| `it.lex` | WikiPron (Wiktionary) | CC BY-SA | Italian uses rules + statistics |
| `de.lex` | ipa-dict / german-ipa-dict (Wiktionary) | CC BY-SA | German uses rules |
| `ru_*.lex` | RUAccent dictionaries (`tools/ru_build_lex.py ru_lexicon/ out/`) | Apache-2.0 | Russian uses its suffix fallback (stress often wrong) |

Rebuild everything from the public sources:

```console
cargo build --release
python tools/build_data.py --out data-out            # English
python tools/build_data.py --out data-out --ccbysa   # + Italian, German
python tools/train_g2p.py build-data/en_train.tsv models/en_small --epochs 24 --d 96 --ff 256 --layers 2
python tools/train_g2p.py build-data/en_train.tsv models/en --epochs 20 --d 128 --ff 384 --layers 3
cp models/en_small/g2p.bin data-out/en.g2p && cp models/en/g2p.bin data-out/en-large.g2p
python tools/ru_build_lex.py /path/to/Russian-G2P/ru_lexicon data-out   # Russian stress tiers
```

The build is deterministic: rerunning it reproduces the lexicons byte for byte.

### Data licensing

The code and the English data are Apache-2.0 / BSD compatible. The German and
Italian lexicons come from Wiktionary, whose share-alike terms do not fit an
Apache package, so they are a separate optional pack. The small Italian tables
compiled into the crate (`data/it_suffix.tsv`, `data/it_open.tsv`) hold only
per-ending counts derived from Wiktionary; aggregate statistics are generally
not copyrightable, but if you want zero Wiktionary-derived material, delete
them and Italian falls back to the hand-written morphology rules.

## How it matches espeak

Notation was matched by phonemizing 30k words per language with espeak and
diffing against the converted lexicons until the remaining differences were
real pronunciation differences, not symbol choices. Examples of conventions
reproduced: English `ɾ` flapping (t only), `ᵻ` in -ed/-es, weak forms (`ðə`,
`ɐ`, `tə`), joined pairs (`ʌvðə`, `hɐvbɪn`), `ðɪ`/`tʊ` before vowels;
Spanish secondary stress on even syllables before the primary, `ŋ` only
before g/x; Italian `Cː` for long stops and affricates but doubled letters for
continuants, `rɾ` for rr, `ɲɲ` after the stress; German `ɾ` after plosives and
before consonants, `ɜ` for -er, long unstressed tense vowels. Output is NFD
with `ʧ ʤ ʦ ʣ` ligatures, exactly what BlueTTS's `normalize_text` produces.

Agreement with espeak's notation on 30k random words: Spanish 98.6% identical,
Italian 78%, German 42%, English 45%. The rest are mostly words where espeak
is wrong (the accuracy table above) or secondary-stress placement.

## Layout

```
src/            Rust core (lib.rs, en/es/it/de/ru.rs, lexicon.rs FST store, nn.rs transformer, num.rs, fold.rs)
bindings/python pyo3 extension + Python router (polyg2p.Phonemizer, polyg2p.bluetts.TextProcessor)
data/           small tables compiled into the crate
tools/          data build, model training, evaluation and benchmark scripts
```

`tools/eval_acc.py`, `tools/speed.py`, `tools/cmp_ours.py` and
`tools/espeak_words.py` call espeak through `phonemizer` (GPL) for comparison.
They are development tools, not part of the package.
