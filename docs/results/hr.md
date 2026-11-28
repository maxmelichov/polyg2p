# Croatian (`hr`), work in progress: not merged

Rules (`src/hr.rs`, Latin script) plus an optional data pack: `hr.lex` (word to stressed-vowel index
and per-vowel length, built with `build_lex`) and `hr.sfx` (ending statistics for stress). Both come
from Wiktionary forms (CC BY-SA) and live in the data pack, not in `src/`. Without the pack: first-syllable
stress, no length. Pitch accent is not written.

## Accuracy vs Wiktionary (WikiPron `hbs_latn_broad.tsv`)

Through `tools/collapse_hr.py` (drops pitch-accent marks, vowel length and syllabic marks; keeps segments).

| set | word acc | segment PER |
|---|---|---|
| held-out 10% of the gold words (md5 of the word, 2 686 words, rules only) | 98.0% | 0.33% |
| the other 90% (23 800 words) | 97.9% | 0.30% |

Stress: held-out = forms of 10% of the lemmas, kept out of the lexicon and ending table (3 192 forms).
Lexicon off, ending statistics only: 76.1% (limit 85%): **not met**. Lexicon token hit rate on running text
is 52%, so the estimated token stress error is about 11% (limit 3%). The language stays on letters until a
stress tagger lifts this.
