# English results

Measured with `tools/eval_acc.py en --data data-out` (20k most frequent words,
frequency-weighted, against WikiPron/Wiktionary US broad transcriptions, both
systems collapsed to a shared broad phone set) and `tools/homographs.py`.

| | polyg2p | espeak-ng 1.52 |
|---|---|---|
| word accuracy, strict | **97.7%** | 96.7% |
| phoneme error rate | **2.1%** | 2.8% |
| word accuracy, unstressed ɪ = ə | **98.6%** | 97.6% |
| homographs in context (40 sentences) | **40/40** | 32/40 |

What changed on 2026-10-23 (`tools/en_convert.py`): CMUdict writes most
unstressed vowels as `AH0`; espeak and Wiktionary distinguish `ɪ`/`ᵻ` from `ə`
by spelling. The converter now aligns the vowel phones with the word's vowel
letters and picks the symbol espeak uses for that spelling (written *i* → `ɪ`,
*-age* → `ɪʤ`, *-et/-est* → `ɪ`, *-ed/-es* → `ᵻ`, pretonic *re-/de-/be-* → `ᵻ`,
initial unstressed *a-* → `ɐ`, *un-* → `ʌn`, *-ual* → `uːə`, single primary
stress per word). The rules were chosen from the table `tools/en_reduced.py`
prints; nothing from espeak is stored. Also new: `Dr. Mr. Mrs. St. Jr. Prof.
vs. etc.` expand without a pause, `5th / 21st` read as ordinals, `$5.50` reads
"five dollars and fifty cents".

Remaining disagreements with Wiktionary (`tools/en_diff.py` → `build-data/en_diff.tsv`):
0.7% of frequency mass where espeak matches the gold and we don't (mostly
medial *-i-* before *-ty/-ster/-lar*, where Wiktionary writes `ə`), 1.5% where
neither does (largely Wiktionary notation: `feəmli`, `θeɪŋk`, `bɪdnɪs`), 1.7%
where espeak is wrong and we are right.
