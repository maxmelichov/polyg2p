# Danish (`da`), draft: word-accuracy gate missed by 1.4 points

Lexicon first (`da.lex`: 198 982 words, built from the NST Danish pronunciation lexicon, CC0 per the Sprakbanken
record, SAMPA converted to IPA; stress before the vowel, stød `ˀ` after its carrier), then a char-level
transformer (`da.g2p`, 1.27 M parameters, `tools/train_g2p.py`, greedy decoding) for words outside it, then plain
letter rules without a pack. Both files live in the data pack, not in `src/`.

Stød `ˀ` (id 224) has no training occurrences yet: its row starts as a copy of `ʔ` (listed in `row_init.json`).

## Held-out words (10% by md5 of the word, 19 935 words, kept out of the lexicon and the model's training data)

Model only (lexicon off), against the NST transcription, stress and stød marks inside the score:

| metric | limit | measured |
|---|---|---|
| word accuracy | 75% | **73.6% (not met)** |
| PER (all marks) | 6.0% | 5.2% |
| segment PER (no stress, stød, length) | | 2.7% |
| stød F1 (position match) | 0.85 | 0.87 |

Three training runs were used (2 + 8 + 7 epochs, the last continued from the second at a lower rate); held-out
word accuracy went 57.4% -> 69.9% -> 73.6%, still rising. A rule-based compound splitter (lexicon head + linking
`s`/`e`) was tried in front of the model and lowered accuracy (50% vs 56% on a 5 000-word sample), so it is
not in the lookup order.

## Running text

30 000 Tatoeba sentences, 192 452 tokens: lexicon hit rate 98.7% with the shipped pack, so the model decides
only 1.3% of tokens. 0 panics, 0 characters outside the vocabulary (the stød mark `ˀ` is the untrained row).
Expert passage (Wikipedia "Danish phonology", first sentence): not comparable, the published reading is an
indistinct, reduced one (`og` -> `ʌ`, `af` -> `æ`, syllabic consonants); collapsed PER 63%, listed as not met.
Function-word weak forms in running speech are not modelled.

Not done: weak-form table for function words, a larger or longer-trained model.
