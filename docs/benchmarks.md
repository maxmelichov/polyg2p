# Benchmarks

Everything here compares polyg2p with espeak-ng 1.52, called through
`phonemizer`, against Wiktionary pronunciations from
[WikiPron](https://github.com/CUNY-CL/wikipron). The scripts are in `tools/`.
Numbers were measured on 2026-11-03 on an Apple M5 laptop with 10 cores.

## How words are scored

Both systems' output and the gold transcription are collapsed to a shared
broad phone set before comparing. Only real pronunciation differences count,
not notation such as `ɾ` versus `t` or a tie bar. The collapse rules for each
language are in `tools/collapse_<lang>.py` and `tools/eval_acc.py`.

```console
python tools/eval_acc.py de --data data-out            # 20k most frequent words, frequency-weighted
python tools/eval_acc.py de --data data-out --all      # every gold word, uniform weights
python tools/eval_acc.py de --data data-out --show 40  # list the misses
```

## Frequent words

The 20,000 most frequent words in WikiPron, weighted by corpus frequency.
"Rules only" means without the optional CC BY-SA lexicon, which is what the
Python wheel ships.

| Language | polyg2p | rules only | espeak-ng |
|---|---|---|---|
| English | **97.9%** | | 96.7% |
| English, unstressed ɪ = ə ² | **98.6%** | | 97.6% |
| Spanish | **99.5%** | | 99.3% |
| Italian | **99.7%** | **98.6%** | 87.1% |
| German | **98.7%** | **96.5%** | 96.5% |
| French | **99.0%** | | 97.7% |
| Portuguese (Brazil) | **99.7%** | **99.6%** | 96.9% |
| Dutch | **99.9%** ¹ | **99.1%** | 96.8% |
| Swedish (3.4k gold words) | **99.3%** ¹ | **98.7%** | 95.6% |
| Ukrainian | **99.3%** | **99.3%** | 79.5% |
| Polish | **99.8%** | | 91.5% |
| Czech | **99.9%** | | 96.4% |
| Hungarian | **99.8%** | | 95.2% |
| Greek | **99.3%** | | 81.0% |
| Romanian | **98.0%** | | 97.8% |
| Catalan (Central) | **93.9%** | | 93.2% |
| Turkish | 99.7% | | 99.7% |
| Finnish | 99.5% | | 99.5% |
| Indonesian | **99.6%** | | 88.4% |
| Russian ³ | **98.9%** | | 62.4% |

¹ These lexicons come from Wiktionary too, so scoring them on Wiktionary is
optimistic. Built from 90% of WikiPron and scored on the other 10%:

| | rules only | with lexicon |
|---|---|---|
| Dutch | 82.1% | 83.9% |
| Swedish | 68.7% | 71.7% |
| Portuguese | 94.2% | 94.2% |

The Portuguese lexicon stores only the open or closed e and o of words the
rules misjudge.

² CMUdict and Wiktionary disagree mostly on how reduced vowels are written.

³ Stress is scored through vowel reduction. Uses the `ru_*.lex` dictionaries
and the context model; see [russian.md](russian.md).

## Every gold word

All words in WikiPron with uniform weights, so rare words, names and
abbreviations count as much as "the".

| Language | words | polyg2p | rules only | espeak-ng |
|---|---|---|---|---|
| English ⁴ | 80,992 | 62.3% | | 61.8% |
| Spanish | 132,191 | **97.2%** | | 97.1% |
| Italian | 82,280 | **96.8%** | **89.4%** | 89.2% |
| German | 53,011 | **97.6%** ¹ | 65.4% | 72.3% |
| French | 85,516 | **94.0%** | | 89.9% |
| Portuguese (Brazil) | 57,900 | **96.9%** ¹ | **94.3%** | 93.2% |
| Dutch | 45,872 | **99.9%** ¹ | **82.0%** | 67.9% |
| Swedish | 5,082 | **98.3%** ¹ | **70.1%** | 58.5% |
| Ukrainian | 51,646 | **98.9%** | **98.9%** | 61.3% |
| Polish | 148,992 | **99.0%** | | 97.5% |
| Czech | 64,350 | **97.9%** | | 94.2% |
| Hungarian | 64,286 | **98.8%** | | 91.3% |
| Greek | 19,108 | **98.3%** | | 93.1% |
| Romanian | 8,978 | **98.6%** | | 97.3% |
| Catalan | 42,559 | **94.5%** | | 92.5% |
| Turkish | 11,582 | 98.8% | | 98.7% |
| Finnish | 168,814 | 99.3% | | 99.2% |
| Indonesian | 17,967 | **90.6%** | | 78.8% |
| Russian ³ | 403,834 | **94.4%** | | 31.2% |

⁴ Mostly rare words, names and acronyms, where the unknown-word model does the
work.

## Whole sentences

`tools/sentence_cmp.py` runs both engines on every Tatoeba sentence in the
language, 8.8 million sentences in all, so context rules, elision and sandhi
count. Words are aligned one to one and compared after the same collapse.
Where the two disagree, WikiPron decides who is right. Most differences are
words WikiPron cannot settle, such as names, loanwords and notation; the last
two columns count only the words it can. Measured on commit 4269d83; later
fixes reduced the right-hand column further on 200k-sentence samples.

```console
python tools/sentence_cmp.py de tatoeba/deu.tsv --data data-out
SHOW_ESP=40 python tools/sentence_cmp.py de tatoeba/deu.tsv 200000 --data data-out   # list where espeak wins
```

| Language | sentences | words | words that differ | polyg2p right, espeak wrong | espeak right, polyg2p wrong |
|---|---|---|---|---|---|
| English | 2,018,803 | 15,002,702 | 4.35% | 219,339 | 24,973 |
| Spanish | 437,127 | 3,083,117 | 1.24% | 28,195 | 26 |
| Italian | 984,011 | 5,845,851 | 8.24% | 309,296 | 425 |
| German | 773,746 | 5,781,616 | 3.50% | 83,273 | 2,240 |
| French | 721,913 | 5,179,252 | 3.61% | 101,519 | 1,898 |
| Portuguese | 433,944 | 3,048,347 | 1.47% | 21,096 | 634 |
| Polish | 135,463 | 714,336 | 3.94% | 22,597 | 774 |
| Ukrainian | 186,162 | 914,147 | 22.34% | 148,398 | 274 |
| Turkish | 746,747 | 3,712,308 | 0.03% | 54 | 2 |
| Dutch | 199,250 | 1,285,739 | 11.21% | 71,912 | 102 |
| Czech | 89,023 | 457,561 | 2.67% | 9,170 | 1,244 |
| Greek | 42,314 | 243,900 | 4.41% | 5,233 | 635 |
| Romanian | 41,083 | 277,562 | 0.61% | 405 | 21 |
| Hungarian | 437,246 | 2,330,269 | 3.86% | 55,212 | 411 |
| Finnish ⁵ | 152,839 | 749,768 | 2.66% | 1,215 | 11,193 |
| Swedish | 56,357 | 317,717 | 17.94% | 15,350 | 78 |
| Indonesian | 27,927 | 153,974 | 9.41% | 10,917 | 24 |
| Catalan | 10,659 | 76,516 | 6.62% | 1,117 | 211 |

⁵ Finnish writes boundary gemination and nasal assimilation across words
(*tehdä seuraavaksi* → `tˈehdæs sˈeuraavaksi`, *on kaunis* → `oŋ kˈaunis`).
A word-by-word comparison scores these as errors although the sentence is
right.

## English specifics

Words not in the dictionary, on 3,000 CMUdict words held out of model
training, mostly names:

| | exact | phoneme error rate |
|---|---|---|
| polyg2p, default model (`en.g2p`, 0.6M params) | **70.4%** | **7.3%** |
| polyg2p, `POLYG2P_EN_MODEL=large` (1.2M params) | **74.6%** | **6.2%** |
| espeak-ng | 64.2% | 8.7% |

Homographs in context, on 40 sentences covering read, record, live, close,
tear, lead, use, wind, present, object, estimate and separate: polyg2p 40/40,
espeak-ng 32/40. The sentences were written by the authors, so treat this as
indicative.

## Speed

20,000 random Tatoeba sentences per language, piped through the CLI. The time
includes process start; piped input is phonemized in 2,048-line chunks on all
cores, in order. espeak-ng runs in `phonemizer` batch mode with one job.

| Language | polyg2p | espeak-ng |
|---|---|---|
| English | **402k sent/s** | 11.3k sent/s |
| Spanish | **495k sent/s** | 11.8k sent/s |
| Italian | **466k sent/s** | 12.9k sent/s |
| German | **549k sent/s** | 8.2k sent/s |
| French | **394k sent/s** | 11.7k sent/s |
| Portuguese | **477k sent/s** | 10.6k sent/s |
| Russian, with the context model | **12.0k sent/s** | 9.6k sent/s |
| Hebrew (`he-int8.g2p`, a 4.9M-param tagger on every sentence) | **1.9k sent/s** | |

Single-threaded, a Russian sentence with a homograph costs about 0.35 ms in
the context model and a Hebrew sentence about 1.7 ms. An unknown English word
costs about 0.5 ms in the default model and 1.6 ms in the large one; results
are cached, and affix and compound splitting resolves many words before the
model is needed.

Peak memory of the CLI on 5,000 sentences:

| Language | peak memory |
|---|---|
| English | 12 MB |
| German, with `de.lex` | 21 MB |
| Russian, with dictionaries and context model | 45 MB |

The lexicons are memory-mapped, so several processes share them.

## Notation agreement with espeak

On 30k random words, the share of words where polyg2p writes exactly what
espeak writes: Spanish 98.6%, Italian 78%, English 45%, German 42%. The rest
are mostly words where espeak is wrong, as the tables above show, or
secondary-stress placement.
