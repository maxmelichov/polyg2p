# English accents

Plain `en` is General American and is unchanged. Seven accent codes run the same engine (lexicon,
homographs, weak forms, numbers, word joining) and then rewrite each finished word (`src/en_accent.rs`).

| code | accent | built from | reference to score against |
|---|---|---|---|
| `en-gb` | Southern British (RP / SSBE) | rules + `data/en_gb_lex.tsv` (about 1,000 words from Britfone, MIT) + `data/en_brit_words.tsv` | WikiPron UK (below) |
| `en-au` | Australian | `en-gb` + rules | none with a usable licence: rule-derived |
| `en-nz` | New Zealand | `en-gb` + rules | none: rule-derived |
| `en-in` | Indian | `en-gb` + rules | none: rule-derived |
| `en-sc` | Scottish | American form + `data/en_brit_words.tsv` + rules | none: rule-derived |
| `en-ie` | Irish | American form + `data/en_brit_words.tsv` + rules | none: rule-derived |
| `en-ca` | Canadian | American form + rules | none: rule-derived |

Aliases: `en-uk`, `en_GB` and the like (case and `_` / `-` do not matter), `en-gb-scotland` for `en-sc`.
An unknown code such as `en-xx` is refused. South African English has no code: at this level of
transcription it has the phonemes of `en-gb`, so use `en-gb` for it.

Unisyn covers most of these accents but is licensed for non-commercial use only, so it was not used.

The variants write which phoneme a word has, not fine vowel quality: Australian FACE is still `eɪ`.
A TTS model learns the quality from recordings of the accent; the phonemes tell it what cannot be heard
from a tag alone (r or no r, BATH, LOT, yod, mergers).

## What each one changes

- `en-gb`: no r unless a vowel follows, with linking r across words and inside hyphenated compounds
  (`fˈɑːɹ əwˈeɪ`, `fˈɑːɹ ˈɒf`, but `fˈɑː.` and `fˈɑː, əwˈeɪ`); LOT `ɒ`; CLOTH joins LOT (`dˈɒɡ`);
  GOAT `əʊ`; BATH `ɑː` (word list); NEAR `ɪə`, SQUARE `eə`, CURE `ʊə`; hurry / sorry / marry /
  squirrel keep short vowels while occurring, blurry, furry keep NURSE; moral, forest, quarrel,
  historic with `ɒ` but story, glorious, memorial with `ɔː`; `t` and `d` instead of the flap, and
  `tən` for the American glottal stop + syllabic n (kitten, written); yod after t d n (`njˈuː`,
  `djˈʊəɹɪŋ`); -ile, -ary / -ory, -ization endings; unstressed vowels written `ə` / `ɪ` (never
  `ɐ` / `ᵻ`).
  Weak forms of, was, from, been are `əv`, `wəz`, `fɹəm`, `biːn`; before a pause or at the end of
  the sentence they are strong (`wɒt ɪz ɪt mˈeɪd ɒv?`). A stranded preposition in the middle of a
  clause ("the man I heard of was there") is not detected and stays weak.
  Words the rules cannot reach from the American form (BATH words, tomato, laboratory) come from
  the word list; inflected forms of a listed word reuse it (passes, dancing, aunts). Words the
  engine reads by context (record, progress, produce, use) have each British reading in the list,
  and the one whose stress and final voicing match the engine's choice is taken. A few words come
  from the hand-written `data/en_brit_words.tsv` (schedule with `ʃ`, vase, yoghurt, lieutenant).
- `en-au`: TRAP before nasal clusters (dance, plants, branch, example) while grass, path, can't
  stay broad; unstressed `ɪ` before t d s z becomes `ə` (rabbit, wanted, roses); tune / dune /
  during with `ʧ` / `ʤ` (not across a word seam: courtyard); sure, poor, tour with `ɔː`, the
  vowel kept after `j`, `ʧ`, `ʤ` (pure, mature, endure).
- `en-nz`: as `en-au` but BATH stays broad everywhere, and SQUARE merges into NEAR (`skwˈɪə`).
- `en-in`: the `en-gb` phonemes with FACE `eː` and GOAT `oː`.
- `en-sc`: rhotic; LOT = THOUGHT `ɔ`; TRAP = PALM `a`; FOOT = GOOSE `u`; FACE `eː`, GOAT `oː`;
  no flap, `tən` in kitten; yod kept (new, during); wh as `ʍ` (which, somewhere); hurry `ʌ`,
  marry `a` against merry `ɛ`, squirrel `ɪ`; British word forms (herb with h, tomato, vase,
  schedule, lieutenant, -ile).
- `en-ie`: rhotic; FACE `eː`, GOAT `oː`; no flap, `tən` in kitten; yod kept; wh as `ʍ`; hurry,
  marry, squirrel and the British word forms as in `en-sc`.
- `en-ca`: American with THOUGHT merged into LOT / PALM (`kˈɑːt` for caught, `ɑːn` for on).

Not done: letter names (Z stays "zee"), British number wording ("one hundred and five"),
function words of `en-sc` / `en-ie` (was, what stay American), and a homograph whose British
reading the list does not hold (contrast) keeps the American reading run through the rules.

## The `en-gb` word list

`tools/en_gb_build.py` writes `data/en_gb_lex.tsv` from Britfone. Britfone has typing errors and
some American forms among its entries (deflection with r-, geese with -z, glory with the vowel of
lot), so an entry is kept only when it has exactly one main stress and Wiktionary's UK list (via
WikiPron) gives the same pronunciation. Of 1,444 Britfone words that differ from the rule output,
501 are left to the rules for that reason. Wiktionary is only the check; every form in the list
is Britfone's.

Provenance: Britfone is published under the MIT licence. Its author states (README and issue #2
of the Britfone repository) that the transcriptions started from CMUdict, that published
dictionaries (Collins, Oxford, Cambridge, Macmillan) "were consulted rather than included", and
that the only sources included, "after heavy modification", were computer-readable ones (CMUdict
and word-frequency lists). polyg2p relies on that statement; it has not audited Britfone entry by
entry. The list is optional: with a header-only `data/en_gb_lex.tsv` the variant runs on its rules
and `data/en_brit_words.tsv`, at the "rules only" accuracy below.

## Accuracy of `en-gb`

Word accuracy against WikiPron's UK list (Wiktionary, CC BY-SA; not shipped; any listed variant counts; stress, length of unstressed
vowels and e/ɛ, a/æ spelling differences ignored). `tools/en_accent_eval.py`. This is a count over
dictionary words, not the frequency-weighted top-20,000 figure the README gives for other
languages.

| | words in the list (813) | other Britfone words (11,024) | words Britfone lacks (53,859) |
|---|---|---|---|
| plain `en` scored as British | 24.5% | 69.4% | 41.9% |
| `en-gb`, rules only | 33.7% | 92.3% | 55.9% |
| `en-gb` | 95.7% | 92.4% | 56.0% |

The first column is not an independent result for the full variant: those entries were kept
because this reference confirms them. The other two columns are independent of the list. The
last one is mostly rare words and names, where the out-of-vocabulary model does the work (plain
`en` scores 62% on the same kind of list against American pronunciations).

Rules against Britfone itself: 90.5% of 15,147 words agree up to secondary stress and the
spelling of unstressed vowels.

Plain `en` output is byte-identical to the previous build (checked on 5,000 sentences with both
English models).

## Tests

`cargo test` runs the rule tests. The sentence and word cases need the English lexicon:

```
POLYG2P_DATA=<dir with en.lex and en.g2p> cargo test --test en_accent -- --include-ignored
```

## Rebuilding the word list

```
cargo build --release --bin en_accent
python tools/en_gb_build.py britfone.main.3.0.1.csv wikipron_eng_latn_uk_broad.tsv --data <dir with en.lex>
cargo build --release
```
