# Lithuanian (`lt`), draft: stress gate not met

Rules (`src/lt.rs`) plus an optional data pack: `lt.lex` (word -> stressed syllable index) and `lt.sfx`
(ending + syllable count -> stressed syllable from the end), both built from accent-marked Wiktionary forms
(CC BY-SA) and kept in the data pack, not in `src/`. Without the pack the stress is on the penultimate
syllable. The pitch accent and the lengthening of stressed short vowels are not written.

## Accuracy vs Wiktionary (WikiPron `lit_latn_narrow.tsv`)

Through `tools/collapse_lt.py`. The gold is narrow (stress digits, half-long, vowel qualities); the collapse
drops stress, length and the a/e/i/u/o quality splits, and keeps the consonants, palatalisation `ʲ` and the
diphthongs. Palatalisation follows the gold: before `e ę ė i į y ie`, before a softening `i`, and spread
left over a consonant cluster except velars; `a` after `j` or a softening `i` is `ɛ`.

| set | word acc | PER |
|---|---|---|
| held-out 10% of the gold words (md5 of the word, 1 538 words, rules only) | 78.4% | 2.1% |
| all 15 302 gold words, uniform | 79.1% | 2.1% |
| espeak-ng on the same list | 23.7% | 17.3% |

The remaining misses are mostly places where the gold is inconsistent between near-identical words
(`ie` as `jɛ` or `iɛ`, `ɒu` for `au`, `ə` for unstressed `e`).

## Stress (the open gate)

Held-out = forms of 10% of the lemmas (md5 of the lemma), kept out of the lexicon and the ending table
(10 984 forms). Lexicon off, ending + syllable count: **82.7%** (limit 88%): not met (offline variants reach 85%).

Running text (30 000 Tatoeba sentences, 144 371 tokens): lexicon hit rate 71.7% with the shipped pack (the
accent-marked forms available are 116 569), 66.1% with the held-out lemmas removed. Estimated token stress error
= (1 - hit) x (1 - 0.827) = 4.9% shipped, 5.9% held-out (limit 5%): borderline / not met. The language stays on
letters until a larger accented source lifts both numbers.

Expert passage (G3a): Wikipedia "Lithuanian phonology" has no sample passage, not run. Native-accented running
text (G3b): not run.
