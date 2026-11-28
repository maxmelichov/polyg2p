# Slovak (`sk`)

Rules only (`src/sk.rs`, no lexicon, no model). Notation follows `cs.rs`
(`h`, `x`, `ts tʃ`, `aʊ eʊ oʊ`, `r̩ l̩`); stress is fixed on the first syllable.

## Accuracy vs Wiktionary (WikiPron `slk_latn_broad.tsv`)

`tools/eval_acc.py sk` and the split below, both through `tools/collapse_sk.py`
(strips o/ɔ, e/ɛ, ɦ/h, ɫ/l, v/w/ʋ, glide-vs-hiatus spelling of `ia ie iu`, consonant
length; keeps vowel length, the palatal series and the voicing).

| set | word acc | PER |
|---|---|---|
| held-out 10% of the gold words (md5 of the word, 1 520 words, never used to write a rule or a stem) | 91.9% | 1.31% |
| the other 90% (14 374 words; the loan-stem list below was mined from these) | 92.3% | 1.36% |
| top 20k frequent words in the gold, frequency-weighted (6 008 words) | 94.0% | 1.2% |
| espeak-ng on the same frequent list | 70.4% | 7.8% |

The gold list has no lemma column, so the held-out split is by word, not by lemma.
Stress is fixed (first syllable) and the gold has no stress marks, so stress accuracy
is 100% by construction. Published sample sentence (Wikipedia, "Slovak phonology"):
PER 6.8% after collapsing; the differences are `ʎ` for the published `l` in `hádali`, `x`
for `ɣ` before the next word's voiced `j`, `ʃ` for `ʂ`.

## What the module does

- palatals `ď ť ň ľ`; `d t n l` palatalised before `i í e` in native words, hard before
  `é` and in a list of loan stems (`-izmus`, `-atív`, `-ália`, ...), a few dozen
  shapes that cover about 40% of the loan words the rule gets wrong; the rest remain errors;
- `ia ie iu` rising diphthongs, hiatus in loan endings (`-cia -zia -gia -ium`, and `-ria -nia`
  after an accented vowel); `ô` = `u̯o`, `ä` = `æ`, `au eu ou` diphthongs;
- syllabic `r l` (`vlk`, `krv`) and long `ŕ ĺ`; `ch` = `x`; `dz dž`;
- voicing assimilation and final devoicing; `v` is `f` word-initially before a voiceless
  obstruent (`vták`), otherwise `v`;
- a one-syllable preposition and its word form one stress unit; a few clitics are unstressed.

Not done: loan-word palatalisation beyond the stem list, the rhythmic law, secondary stress.
