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

## Stress tagger attempt (step T)

`bg.rs` can load an optional character-level stress tagger (`bg_stress.g2p` in the data pack, built by a
trainer that is not in the repo; 0.76 MB int8; order: lexicon, tagger, ending table). Three runs on forms of the
training lemmas, scored on forms of held-out lemmas (multi-syllable):

| run | model | held-out stress accuracy |
|---|---|---|
| 1 | d=128, 4 layers, 5 epochs | 75.6, 76.5, 78.5, 79.6, 79.3% (best 79.6%) |
| 2 | same, continued, 7 more epochs, dropout 0.2 | 80.9 to 81.25% (best 81.25%) |
| 3 | d=192, 4 layers, dropout 0.3 | 74.4, 77.2, 79.0, 79.5, 78.6% (about 80%) |

The curves plateau near 80-81% while the training loss keeps falling: the stressed vowel of an
unseen lemma is not predictable from its letters to the 90% the gate asks for. Through the CLI on the 1 203
gold-listed held-out forms the tagger gives 76.9% stress accuracy (word accuracy 80.0%, PER 3.05%), no better than the
ending table. With a lexicon token hit rate of 55% on running text the estimated token stress error is
(1 - 0.55) x (1 - 0.81), about 9% to 11%, against the 3% limit. Bulgarian stays on letters; this PR is a draft.
