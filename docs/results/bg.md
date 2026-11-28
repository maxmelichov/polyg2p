# Bulgarian (`bg`), work in progress: not merged

Rules (`src/bg.rs`) plus an optional data pack: `bg.lex` (word to stressed-vowel index, built with
`build_lex`) and `bg.sfx` (ending statistics for stress). Both are derived from Wiktionary forms
(CC BY-SA) and live in the data pack, not in `src/`. Without the pack the stress is penultimate.

## Accuracy vs Wiktionary (WikiPron `bul_cyrl_narrow.tsv`)

Held-out = forms of 10% of the lemmas (md5 of the lemma), kept out of the lexicon and the ending
table; 1 203 of them are in the gold list. Through `tools/collapse_bg.py` (keeps vowel reduction and
palatalisation; merges o/ɔ, e/ɛ, l/ɫ).

| setting | word acc | PER | stress acc |
|---|---|---|---|
| given the lexicon stress of the held-out forms (G1) | 90.1% | 1.29% | |
| end to end: unknown words get the ending-statistics stress | 78.5% | 3.29% | 76.2% on the 1 203; 81.9% on all held-out multi-syllable forms by a combined prefix/ending vote (experiment, not in the module) |

The plan's stress gate (G2, at least 90%) is **not met**. The remaining work is the character-level
stress tagger; until then the language stays on letters.
