# English accents

Plain `en` is General American and is unchanged. Eight accent codes run the same engine (lexicon,
homographs, weak forms, numbers, word joining) and then rewrite each finished word (`src/en_accent.rs`).

| code | accent | built from | reference to score against |
|---|---|---|---|
| `en-gb` | Southern British (RP / SSBE) | rules + `data/en_gb_lex.tsv` (1,535 words from Britfone, MIT) | WikiPron UK (below) |
| `en-au` | Australian | `en-gb` + rules | none with a usable licence: rule-derived |
| `en-nz` | New Zealand | `en-gb` + rules | none: rule-derived |
| `en-za` | South African | same phonemes as `en-gb` | none: rule-derived |
| `en-in` | Indian | `en-gb` + rules | none: rule-derived |
| `en-sc` | Scottish | American form + rules | none: rule-derived |
| `en-ie` | Irish | American form + rules | none: rule-derived |
| `en-ca` | Canadian | American form + rules | none: rule-derived |

Unisyn covers most of these accents but is licensed for non-commercial use only, so it was not used.

The variants write which phoneme a word has, not fine vowel quality: Australian FACE is still `eɪ`.
A TTS model learns the quality from recordings of the accent; the phonemes tell it what cannot be heard
from a tag alone (r or no r, BATH, LOT, yod, mergers).

## What each one changes

- `en-gb`: weak forms of, was, from as `əv`, `wəz`, `fɹəm`; no r unless a vowel follows, with linking r across words (`fˈɑːɹ əwˈeɪ`, `fˈɑː.`); LOT `ɒ`;
  CLOTH joins LOT (`dˈɒɡ`); GOAT `əʊ`; BATH `ɑː` (word list); NEAR `ɪə`, SQUARE `eə`, CURE `ʊə`;
  hurry / sorry / marry keep short vowels; `t` and `d` instead of the flap; yod after t d n (`njˈuː`);
  -ile, -ary / -ory, -ization endings; unstressed vowels written `ə` / `ɪ` (never `ɐ` / `ᵻ`).
  Words the rules cannot reach from the American form (tomato, schedule, laboratory, BATH words) come
  from the word list; inflected forms of a listed word reuse it (passes, dancing).
- `en-au`: TRAP before nasal clusters (dance, plant, example) while grass, path, can't stay broad;
  unstressed `ɪ` before t d s z becomes `ə` (rabbit, wanted, roses); tune / dune with `ʧ` / `ʤ`;
  sure, poor, tour with `ɔː`.
- `en-nz`: as `en-au` but BATH stays broad everywhere, and SQUARE merges into NEAR (`skwˈɪə`).
- `en-za`: the `en-gb` phonemes.
- `en-in`: the `en-gb` phonemes with FACE `eː` and GOAT `oː`.
- `en-sc`: rhotic; LOT = THOUGHT `ɔ`; TRAP = PALM `a`; FOOT = GOOSE `u`; FACE `eː`, GOAT `oː`;
  no flap; yod kept; wh- as `ʍ`.
- `en-ie`: rhotic; FACE `eː`, GOAT `oː`; no flap; yod kept; wh- as `ʍ`.
- `en-ca`: American with THOUGHT merged into LOT / PALM (`kˈɑːt` for caught).

Not done: letter names (Z stays "zee"), British number wording ("one hundred and five"), and
context-dependent homographs keep the American reading run through the rules.

## Accuracy of `en-gb`

Word accuracy against WikiPron's UK list (Wiktionary; any listed variant counts; stress, length of
unstressed vowels and e/ɛ, a/æ spelling differences ignored). `tools/en_accent_eval.py`.

| | words WikiPron and Britfone share (11,837) | other WikiPron words (53,859) |
|---|---|---|
| plain `en` scored as British | 66.3% | 41.9% |
| `en-gb`, rules only | 87.9% | 55.6% |
| `en-gb` | 92.2% | 55.7% |

The second column is mostly rare words and names, where the out-of-vocabulary model does the work
(plain `en` scores 62% on the same kind of list against American pronunciations).

Rules against Britfone itself: 89.9% of 15,155 words agree up to secondary stress and the spelling of
unstressed vowels; with one tenth of Britfone held out of the word list, 90.7% of those held-out words.

Plain `en` output is byte-identical to the previous build on 5,000 sentences.

## Rebuilding the word list

```
printf '# word\tSouthern British form; built by tools/en_gb_build.py from Britfone (MIT)\n' > data/en_gb_lex.tsv
cargo build --release --bin en_accent
python tools/en_gb_build.py britfone.main.3.0.1.csv --data <dir with en.lex>
cargo build --release
```
