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
| as written | 6.9% | 14.4% | 71.5% |
| glottal stop unscored | 3.5% | 11.2% | 87.9% |

7.5% of the gold phones carry a length mark (188 of 2 522), which is the whole
gap between the two PER columns. The gold is small and holds citation forms
only, so these numbers say little about inflected words in running text.

There is no espeak-ng column: espeak-ng was not run for this language.

What is left with gemination and `ʔ` set aside is almost all `ɨ`: 20 gold `ɨ`
the rules do not write and 7 they write where the gold has none. Most of the
missing ones sit before a long consonant (`ምሽት` gold `mɨʃːɨt`, ours `mɨʃt`),
which the spelling does not show.

## Notation

* Vowels: `ə u i a e ɨ o` (the seven orders). No stress marks, no length.
* Ejectives carry the modifier apostrophe `ʼ` (U+02BC): `pʼ tʼ kʼ sʼ ʧʼ`.
* Labialized consonants carry `ʷ`: `kʷ ɡʷ kʼʷ hʷ`, and the eighth form of any
  row (`ሏ` → `lʷa`).
* Affricates are folded to `ʧ ʤ` like every other language (`fold.rs`). Only
  the affricate letters `ች ጅ ጭ` give them: a plain `t` or `d` is never left
  next to `ʃ`, `ʒ`, `z` or a word-final `s`, so `ʦ` and `ʣ` do not occur.
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
   `amarɨɲa`). `h` does not close a cluster (`እኅት` → `ɨhɨt`); this took the
   headline from 7.0% to 6.9%.
4. **No false affricates.** A bare `t` or `d` directly before `ʃ`, `ʒ` or `z`,
   and `t` before a word-final `s`, takes `ɨ`: `ቤትሽ` → `betɨʃ`, `ቤትሻ` →
   `betɨʃa`, `ቤትስ` → `betɨs`. Left next to each other the two would be folded
   into one affricate and `ቤትሽ` would read like `ቤች` (`beʧ`).
5. **Punctuation.** Ethiopic punctuation maps to the kept marks: `።` → `.`,
   `፣ ፤ ፥` → `,`, `፦` → `:`, `፧` → `?`, `፡` → space. The typed substitutes
   are read the same way: `፡፡` and `::` are a full stop, `፡-` and `:-` are
   `፦`. Parentheses, dashes and `;` become a comma. A `.`, `,` or `:` right
   after another mark is dropped (`?.` → `?`). A mark never stands alone:
   marks before the first word are dropped, and a mark after something that is
   not read goes to the word before it.
6. **Inside a word.** Quotation marks and apostrophes are deleted. The
   combining marks U+135D–U+135F (gemination and vowel length) and
   U+0300–U+036F are deleted too, so they do not split the word; the
   gemination they mark is not written out. A hyphen is a word boundary
   (`ሥነ-ሥርዓት` → `sɨnə sɨrat`).
7. **Numbers and symbols.** Digits are read as Amharic numbers through the
   shared number code (`1,000` → `አንድ ሺህ`, `2.5` → `ሁለት ነጥብ አምስት` with the
   fraction digit by digit, `3:30` → `ሦስት ሠላሳ`). `%` is `በመቶ`, `$ € £` are
   `ዶላር ዩሮ ፓውንድ` after the amount, `&` is `እና`, `°` is `ዲግሪ`. Digits written
   against letters are one word, read like the spelled-out form: `በ5` →
   `በአምስት`, `5ኛ` → `አምስተኛ`, `9ኛ` → `ዘጠነኛ`, `2ቱ` → `ሁለቱ`, `10ሩ` → `አሥሩ`.
8. **Inline tags.** Ethiopic punctuation right after a tag stays with it:
   `ሰላም [laughs]። ዓለም` → `səlam [laughs]. aləm`.

The rule for `ɨ` and the cluster table are engineering choices tuned on the
gold above, not taken from a published description.

## Known gaps

* **Gemination is not predicted.** It is lexical and unwritten; the output
  never contains `ː`.
* **`ɨ` is decided on the written word.** A one-letter prefix is syllabified
  with its stem (`በትምህርት` → `bətmɨhɨrt`, where careful speech keeps the
  stem's `tɨm-`), and a final pair of obstruents is always a cluster (`ነጥብ` →
  `nətʼb`).
* **A final pair of sonorants with falling sonority is a cluster**
  (`ይስጥልኝ` → `jɨstʼɨlɲ`, spoken `-lɨɲ`). Writing `ɨ` there was tried: the gold
  has no word that ends in `ɲ`, and the general rule made it worse (`ሕልም`
  `hɨlm`, `ፊልም` `film`, `ዓይን` `ajn` are clusters in the gold), so it stays.
* **Abbreviations are read letter by letter as written.** The period and the
  slash are not expanded: `ዓ.ም.` → `a. m.`, `ዶ/ር` → `do r`. Write the words
  out (`ዓመተ ምሕረት`, `ዶክተር`).
* **Only the Ethiopic block is read** (U+1200–U+135A). Letters of the Ethiopic
  Supplement, Extended and Extended-A blocks (Sebat Bet, Me'en, Bilen and
  other languages) are skipped without splitting the word.
* Latin-script words and other scripts inside Amharic text are skipped, not
  read. Ethiopic numerals (`፩ ፪ …`) are skipped; write digits.
* A minus sign is not read (`-5` and `−5` → `አምስት`), like in the other
  languages; a hyphen between numbers is a boundary (`1-2` → `አንድ ሁለት`).
  Symbols without a word here (`+ = @`, other currency signs) are skipped.
  Dates are not recognized: `12/03/2021` is read as three numbers.
