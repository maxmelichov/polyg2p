# Slovenian (`sl`), draft: stress gate not met

Rules (`src/sl.rs`) plus an optional data pack built from Sloleks 3.0 (CLARIN.SI 11356/1745, CC BY-SA 4.0,
manually checked part only): `sl.lex` (form -> finished IPA with stress, length and vowel quality), `sl.sfx`
(ending -> stressed vowel from the end) and `sl.vq` (context -> vowel symbol). All three live in the data
pack, not in `src/`. Without the pack the stress is on the penultimate vowel, every `e o` is open and nothing is
long. The pitch accent is not written.

## Accuracy vs Wiktionary (WikiPron `slv_latn_broad.tsv`)

Through `tools/collapse_sl.py`: drops accent marks, length and stress and merges `e ɛ ə`, `o ɔ`; the glide
`u̯` and a word-final / pre-consonantal `ʋ` are one phone.

| set | word acc | PER |
|---|---|---|
| held-out 10% of the gold words (md5 of the word, 493 words, rules only) | 96.2% | 0.65% |
| all 5 177 gold words, uniform, rules only | 95.9% | 0.7% |
| espeak-ng on the same list | 88.9% | 2.6% |

Expert passage (Wikipedia "Slovene phonology", first sentence of the North Wind text, 96 phones, after
removing the glottal stop and the `ɪ` notation that the passage writes and this module does not): PER 4.2%
(pass is 10%). The misses are `kateri` (`ɪ` for `i`), final devoicing of `od` before `nj`, and the `v topel` word
boundary.

## Stress (the open gate)

Held-out = forms of 10% of the lemmas (md5 of the lemma), kept out of the lexicon, the ending table and the
vowel table (88 480 forms). Lexicon off, ending statistics only: **75.4%** (limit 88%): not met. A prefix-stripping
variant and a ending+syllable-count variant tried offline reach 79.9% at most; stress in Slovene is lexical and
the ending does not determine it.

Running text (3 589 Tatoeba sentences, 17 508 tokens): lexicon hit rate 97.2% with the shipped pack, 89.0% when the
10% held-out lemmas are removed. Estimated token stress error = (1 - hit) x (1 - 0.754) = 0.7% shipped, 2.7%
with the held-out pack (limit 3%): met. The language stays on letters because the word-level stress gate
(G2) is not met; the segment rules are good and can ship once a stress source lifts G2.

Native-accented running text (G3b): not run, no free accented running text was found.
