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

## Stress tagger attempt (shared trainer, developed on Bulgarian)

A character-level tagger trained on the same kaikki forms reached 79.6%, 81.2% and about 80% held-out stress
accuracy on Bulgarian over three runs (training loss kept falling, held-out accuracy flat: it memorises
paradigms instead of generalising). Croatian was not trained further: the same ceiling is expected, below the
85% limit, and the lexicon covers only about half of running-text tokens. Both gates stay open; this PR is a
draft. The segment rules are good and can ship once a better stress source exists.
