# Latvian (`lv`)

Rules only (`src/lv.rs`, no lexicon, no model). Stress on the first syllable (fixed); the
pitch accent (tone) is not marked.

## Accuracy vs Wiktionary (WikiPron `lav_latn_narrow.tsv`)

Through `tools/collapse_lv.py`. The gold is narrow and carries tone marks on the vowels, so the
collapse drops tone and all length, and merges `e ɛ æ`, `o ɔ uo` and `a ɑ`.

| set | word acc | PER |
|---|---|---|
| held-out 10% of the gold words (md5 of the word, 163 words, never used for a rule) | 92.0% | 1.82% |
| the other 90% (1 494 words) | 94.4% | 1.10% |
| all 1 657 gold words, uniform | 94.2% | 1.2% |
| espeak-ng on the same list | 73.9% | 4.9% |

The gold list has no lemma column; the split is by word. Stress is fixed and unmarked in the gold:
100% by construction. The Wikipedia page for Latvian phonology has no sample passage, so the
passage check was not run.

Open vs closed `e/ē` (`æ` vs `ɛ`): the module uses a stand-in rule (open before a back vowel in
the next syllable, closed otherwise; the quality is lexical in Latvian). Against the gold
words with exactly one `e`: 82% on 587 words, 77% on the 48 held-out ones. This is below the 92%
the plan wanted reported as a target; it is not part of the G1 PER because the collapse merges the two.
Native `o` is written `uo`, loan-shaped words keep `o`; this choice is not scored either.

Not done: tone, lexicon for `e`/`o` quality, secondary stress.
