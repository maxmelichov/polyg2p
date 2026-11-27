# Amharic (`am`)

Rules only (`src/am.rs`), no lexicon or model. Input is Ge'ez script (fidel);
output is plain IPA.

## Accuracy vs Wiktionary (WikiPron `amh_ethi_broad.tsv`, 478 entries)

`tools/am_eval.py` (the gold file is CC BY-SA and is not in this repository;
it is used for evaluation only). Fidel does not write gemination and the
engine does not predict it, so the headline removes the length mark from the
gold. The gold lists many vowel-initial words twice, with and without an
initial glottal stop; the engine writes none, so the second row leaves `ʔ`
unscored.

| scoring | PER without gemination | PER with gemination | word accuracy without gemination |
|---|---|---|---|
| as written | 7.0% | 14.5% | 71.3% |
| glottal stop unscored | 3.5% | 11.3% | 87.4% |

7.5% of the gold phones carry a length mark (188 of 2 522), which is the whole
gap between the two PER columns. The gold is small and holds citation forms
only, so these numbers say little about inflected words in running text.

There is no espeak-ng column: espeak-ng was not run for this language.

What is left with gemination and `ʔ` set aside is almost all `ɨ`: 22 gold `ɨ`
the rules do not write and 7 they write where the gold has none. Most of the
missing ones sit before a long consonant (`ምሽት` gold `mɨʃːɨt`, ours `mɨʃt`),
which the spelling does not show.

## Notation

* Vowels: `ə u i a e ɨ o` (the seven orders). No stress marks, no length.
* Ejectives carry the modifier apostrophe `ʼ` (U+02BC): `pʼ tʼ kʼ sʼ ʧʼ`.
* Labialized consonants carry `ʷ`: `kʷ ɡʷ kʼʷ hʷ`, and the eighth form of any
  row (`ሏ` → `lʷa`).
* Affricates are folded to `ʧ ʤ` like every other language (`fold.rs`); `t` +
  `s` at the end of a word becomes `ʦ`.
* `ሀ ሐ ኀ ኸ` → `h`; `ሰ ሠ` → `s`; `ጸ ፀ` → `sʼ`; `አ ዐ` are vowel carriers.

## What the module does

1. **Letters.** A fidel is consonant row x vowel order, read from its code
   point. The first order of the gutturals `ሀ ሐ ኀ አ ዐ` is `a`, not `ə`.
   `tests/am_fidel.tsv` lists the reading of all 326 letters.
2. **Vowel carriers.** `አ`/`ዐ` give their vowel with no glottal stop at the
   start of a word; inside a word after a vowel the hiatus is written `ʔ`
   (`ሰዓት` → `səʔat`).
3. **Epenthetic `ɨ`.** A sixth-order letter is a bare consonant. Syllables are
   built left to right as (C)V(C), with CVCC only at the end of the word. A
   bare consonant after a vowel is a coda. After a consonant, or at the start
   of the word, it opens a syllable and takes `ɨ` (`ትምህርት` → `tɨmhɨrt`). The
   last consonant of the word joins the coda when sonority does not rise
   (`ልብስ` → `lɨbs`, `አንድ` → `and`); otherwise `ɨ` goes between the two
   (`ክፍል` → `kɨfɨl`, `እግር` → `ɨɡɨr`). A word-final pair that can stand as a
   cluster is kept, and the consonant before it takes the vowel (`ስምንት` →
   `sɨmɨnt`). A bare consonant before `ɲ` + vowel takes `ɨ` (`አማርኛ` →
   `amarɨɲa`).
4. **Text.** Ethiopic punctuation maps to the kept marks: `።` → `.`,
   `፣ ፤ ፥` → `,`, `፦` → `:`, `፧` → `?`, `፡` → space. A `.`, `,` or `:` right
   after another mark is dropped, so the full stop typed as `::` comes out as
   one `:`. Digits are read as Amharic numbers (`1,000` → `አንድ ሺህ`), `%` as
   `በመቶ`.

The rule for `ɨ` and the cluster table are engineering choices tuned on the
gold above, not taken from a published description.

## Known gaps

* **Gemination is not predicted.** It is lexical and unwritten; the output
  never contains `ː`.
* **`ɨ` is decided on the written word.** A one-letter prefix is syllabified
  with its stem (`በትምህርት` → `bətmɨhɨrt`, where careful speech keeps the
  stem's `tɨm-`), and a final pair of obstruents is always a cluster (`ነጥብ` →
  `nətʼb`).
* Latin-script words and other scripts inside Amharic text are skipped, not
  read. Ethiopic numerals (`፩ ፪ …`) are skipped; write digits.
* Apostrophes and hyphens are passed through as written.
