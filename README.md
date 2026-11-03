# polyg2p

Grapheme-to-phoneme conversion for TTS in 20 languages, written in Rust.
It writes espeak-ng's IPA notation, so models trained on espeak phonemes can
switch without retraining. It matches or beats espeak-ng's accuracy in every
language measured and is 35 to 65 times faster for most of them. Apache-2.0,
with no espeak code or data.

```python
from polyg2p import Phonemizer

g2p = Phonemizer()
g2p("I have read the record, and I will record it again.", "en")
# 'aɪ hæv ɹˈɛd ðə ɹˈɛkɚd, ænd aɪ wɪl ɹᵻkˈɔːɹd ɪt ɐɡˈɛn.'
g2p("Die Bundesregierung hat am 3. Oktober entschieden.", "de")
# 'diː bʊndɛsreːɡˈiːrʊŋ hat am dɾˈɪtən ɔktˈoːbɜ ɛnʧˈiːdən.'
g2p("Всё хорошо, дверь закрыта.", "ru")
# 'fsʲˈɵ xərɐʂˈo, dvʲerʲ zɐkrˈɨtə.'
```

## Why

- **Accurate.** On each language's 20,000 most frequent Wiktionary words,
  polyg2p beats espeak-ng in 17 languages and ties it in Turkish and Finnish.
  Italian goes from 87.1% to 99.7%, Greek from 81.0% to 99.3%, Russian from
  62.4% to 98.9%.
- **Context-aware.** Homographs are read from the sentence: *read*, *record*,
  *live*, Russian замо́к / за́мок and все / всё, Hebrew without vowel points.
  Sandhi, elision and liaison apply across words.
- **Fast.** 400k to 550k sentences per second on a laptop for most languages,
  against about 10k for espeak-ng. The CLI and the batch API use all cores.
- **Drop-in.** Same symbols as espeak-ng: NFD, `ʧ ʤ ʦ ʣ` ligatures, and every
  symbol inside the usual 256-token espeak phoneme vocabulary.
- **Small.** 12 MB of memory for English and 45 MB for Russian with its
  dictionaries and context model. Lexicons are memory-mapped.

## Install

polyg2p is not on PyPI yet. Build the wheel from this repository. It bundles
the English, Russian and Hebrew data, so `Phonemizer()` works with no
arguments.

```console
python tools/build_data.py --out data-out        # English lexicon from sources/cmudict.dict
python tools/bundle_wheel_data.py                # copy the Apache/BSD data into the wheel
cd bindings/python && maturin build --release -o ../../dist
pip install ../../dist/polyg2p-*.whl
```

The Russian dictionaries and the Hebrew model come from separate builds; see
[Data files](#data-files).

## Usage

### Python

```python
from polyg2p import Phonemizer, unknown_symbols

g2p = Phonemizer()                       # or Phonemizer(data_dir="data-out") for the lexicon pack
g2p("Gracias, [laughs] hasta mañana.", "es-419")
# 'ɡɾˈasjas, [laughs] ˌasta maɲˈana.'
g2p("Hello <es>hola amigo</es> world", "en", tags=True)
# '<en>həlˈoʊ</en><es>ˈola amˈiɣo</es><en>wˈɜːld</en>'
g2p.phonemize_batch(lines, "it")         # all cores, for corpora
unknown_symbols(ipa)                     # [] when every symbol is in the espeak vocabulary
```

`<xx>…</xx>` spans switch language inside a sentence. `POLYG2P_DATA` sets the
data folder for every `Phonemizer`.

### Command line

```console
cargo build --release
echo "La città è bella." | target/release/g2p it --data data-out
target/release/g2p de --data data-out < sentences.txt > phonemes.txt
```

One line in, one line out. Piped input runs in parallel and keeps its order.

### Rust

```rust
let g2p = polyg2p::G2p::new(Some(std::path::Path::new("data-out")));
let ipa = g2p.phonemize("Die Bundesregierung hat entschieden.", polyg2p::Lang::De);
```

## Languages

Word accuracy on the 20,000 most frequent Wiktionary words, weighted by
frequency. The "polyg2p" column is what the wheel gives. "With lexicon pack"
adds the optional CC BY-SA lexicons described under [Licensing](#licensing).

| Language | Code | How it works | polyg2p | with lexicon pack | espeak-ng |
|---|---|---|---|---|---|
| English | `en` | CMUdict, context rules, small transformer for unknown words | **97.9%** | | 96.7% |
| Spanish | `es`, `es-419` | rules; `es-419` writes Latin American seseo and yeísmo | **99.5%** | | 99.3% |
| Italian | `it` | rules, stress and vowel-quality statistics | **98.6%** | **99.7%** | 87.1% |
| German | `de` | rules, compound splitting | 96.5% | **98.7%** | 96.5% |
| French | `fr` | rules, exception lexicon, liaison | **99.0%** | | 97.7% |
| Portuguese (Brazil) | `pt` | rules, vowel-quality statistics | **99.6%** | **99.7%** | 96.9% |
| Dutch | `nl` | rules: vowel length, schwa, prefixes | **99.1%** | **99.9%** ¹ | 96.8% |
| Swedish | `sv` | rules, compound splitter | **98.7%** | **99.3%** ¹ | 95.6% |
| Ukrainian | `uk` | rules, stress statistics | **99.3%** | | 79.5% |
| Polish | `pl` | rules | **99.8%** | | 91.5% |
| Czech | `cs` | rules | **99.9%** | | 96.4% |
| Hungarian | `hu` | rules | **99.8%** | | 95.2% |
| Greek | `el` | rules | **99.3%** | | 81.0% |
| Romanian | `ro` | rules | **98.0%** | | 97.8% |
| Catalan (Central) | `ca` | rules, vowel-quality statistics | **93.9%** | | 93.2% |
| Turkish | `tr` | rules | 99.7% | | 99.7% |
| Finnish | `fi` | rules | 99.5% | | 99.5% |
| Indonesian | `id` | rules, e/ə stem table | **99.6%** | | 88.4% |
| Russian | `ru` | dictionaries, context model ([details](docs/russian.md)) | **98.9%** | | 62.4% |
| Hebrew | `he` | sentence-level tagger ([details](docs/hebrew.md)) | | | |
| Yiddish | `yi` | routed to [phonikud-yi](https://github.com/maxmelichov/phonikud-yi-blue-tts) | | | |

¹ Built from Wiktionary, so scoring on Wiktionary is optimistic. On a
held-out 10% split the Dutch lexicon adds 1.8 points and the Swedish one 3.0.

Russian is an exact port of [Russian-G2P](https://github.com/maxmelichov/Russian-G2P),
with RUAccent's context models distilled into one 1.5M-parameter model.
Hebrew is distilled from [RenikudPlus](https://github.com/maxmelichov/RenikudPlus).
Arabic is scaffolded and waits for a context model like Hebrew's.

[docs/benchmarks.md](docs/benchmarks.md) has every table: all gold words,
8.8 million Tatoeba sentences, unknown English words, memory, and how the
scoring works.

## Speed

20,000 Tatoeba sentences per language piped through the CLI on an Apple M5
laptop, including process start. espeak-ng 1.52 runs through `phonemizer` in
batch mode.

| Language | polyg2p | espeak-ng |
|---|---|---|
| English | **402k sent/s** | 11.3k sent/s |
| German | **549k sent/s** | 8.2k sent/s |
| Spanish | **495k sent/s** | 11.8k sent/s |
| Russian, with the context model | **12.0k sent/s** | 9.6k sent/s |
| Hebrew | **1.9k sent/s** | |

Russian and Hebrew run a neural model over the sentence. On aarch64 CPUs with
the dot-product extension it uses an int8 kernel; `POLYG2P_INT8=0` forces f32.

## Text normalization

Raw text can go straight in:

- **Numbers** in every language: cardinals, decimals, thousands separators,
  currency, percent.
- **Years, decades, times and dates** in English, German, Spanish, Italian,
  Portuguese and French: `1990s` → nineteen nineties, `14:30 Uhr`,
  `12/03/2021`, `am 3. Oktober` → am dritten Oktober.
- **Abbreviations** written with a period: `Dr.`, `z.B.`, `bzw.`, `e.g.`,
  `Sr.`, `dott.`, `sig.ra`, `Mme`.
- **Letters, acronyms and web addresses**: `la h y la ñ`, `FBI`,
  `www.example.com` → double-u double-u double-u dot example dot com.
- **Tags** in square brackets, such as `[laughs]` or `[sighs]`, pass through
  unchanged.

## Data files

`G2p::new(data_dir)` and `Phonemizer(data_dir=...)` load these when present.

| File | Source | License | In the wheel | Without it |
|---|---|---|---|---|
| `en.lex` | CMUdict | BSD-style | yes | English needs it |
| `en.g2p` | trained on CMUdict | BSD-style | yes | unknown words are spelled out |
| `en-large.g2p` | trained on CMUdict | BSD-style | no | `POLYG2P_EN_MODEL=large` uses it |
| `ru_*.lex` | RUAccent dictionaries | Apache-2.0 | yes | stress falls back to suffix rules |
| `ru-ctx.g2p` | distilled from RUAccent | Apache-2.0 | compiled in | |
| `he-int8.g2p` | distilled from RenikudPlus | Apache-2.0 | yes | Hebrew needs it or renikud-onnx |
| `de.lex` | ipa-dict, Wiktionary | CC BY-SA | no | German uses rules |
| `it.lex`, `uk.lex`, `nl.lex`, `sv.lex`, `pt.lex` | WikiPron, Wiktionary | CC BY-SA | no | rules only |

Rebuild from the public sources in `sources/`:

```console
python tools/build_data.py --out data-out            # English
python tools/build_data.py --out data-out --ccbysa   # + Italian, German
python tools/ru_build_lex.py /path/to/Russian-G2P/ru_lexicon data-out
python tools/train_g2p.py build-data/en_train.tsv models/en_small --epochs 24 --d 96 --ff 256 --layers 2
```

The build is deterministic and reproduces the lexicons byte for byte.

## Licensing

The code is Apache-2.0. The bundled data is Apache-2.0 or BSD-compatible; the
CMUdict notice is in `NOTICE`.

The German, Italian, Ukrainian, Dutch, Swedish and Portuguese lexicons come
from Wiktionary. Its share-alike license does not fit an Apache package, so
they are an optional pack that the wheel leaves out. Without them every
language still works from its rules and hand-written tables, at the "polyg2p"
accuracy in the table above.

The small Italian tables compiled into the crate, `data/it_suffix.tsv` and
`data/it_open.tsv`, hold per-ending counts derived from Wiktionary. Aggregate
statistics are generally not copyrightable. Delete them if you want no
Wiktionary-derived material at all, and Italian falls back to its
hand-written rules.

The evaluation scripts call espeak-ng through `phonemizer`, which is GPL. They
are development tools, not part of the package.

## Development

```
src/              Rust core: one file per language, lexicon store, transformer, numbers, normalization
bindings/python/  pyo3 extension and the Python router
data/             small tables compiled into the crate
tools/            data build, training, evaluation and benchmark scripts
docs/             benchmarks, Russian, Hebrew, adding a language
```

To add a language, follow [docs/adding-a-language.md](docs/adding-a-language.md).
Measure every change with `tools/eval_acc.py`, on the frequent list and with
`--all`, before keeping it.
