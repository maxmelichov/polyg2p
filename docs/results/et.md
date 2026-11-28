# Estonian (`et`)

Rules only (`src/et.rs`, no lexicon, no model). Stress on the first syllable; a doubled vowel
or consonant is `ː`; `õ ä ö ü` = `ɤ æ ø y`; `ng` and `nk` = `ŋk` / `ŋ`.

## Accuracy vs Wiktionary (WikiPron `est_latn_broad.tsv`)

Through `tools/collapse_et.py`. WikiPron marks quantity degrees only inconsistently (`aabits`
and `Eesti` carry no length, `maa` and `kapp` do), so the collapse drops **all length**, palatalisation
and the lenis marks; the scores say nothing about quantity. Quantity degree 3 and palatalisation
are not in the spelling and are not marked by the module.

| set | word acc | PER |
|---|---|---|
| held-out 10% of the gold words (md5 of the word, 280 words, never used for a rule) | 99.3% | 0.11% |
| the other 90% (2 493 words) | 97.8% | 0.56% |
| all 2 773 gold words, uniform | 97.9% | 0.5% |
| espeak-ng on the same list (it marks length and palatalisation, which the collapse removes) | 24.1% | 19.7% |

The gold list has no lemma column; the split is by word. There is no stress mark in the gold
and stress is fixed (first syllable), so stress accuracy is 100% by construction. The Wikipedia
page for Estonian phonology has no sample passage, so the passage check was not run.

Not done: palatalisation, overlong degree, stress of the later parts of compounds, secondary stress.
