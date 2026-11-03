# Russian

The Russian engine is an exact port of
[Russian-G2P](https://github.com/maxmelichov/Russian-G2P): Moscow-norm rules,
stress from tiered dictionaries, and a context model for the words a
dictionary cannot decide. Latin-script words inside Russian text go to the
English engine.

## Port fidelity

Without context models, the Rust port gives output identical to the Python
original on:

| test set | size |
|---|---|
| gold words | 845 |
| synthetic words | 555k |
| sentences | 28.8k |

It is about 25× faster per word. The 3.19M-form stress dictionary is
memory-mapped instead of taking about 1.5 GB of RAM.

## Homographs and ё

Homograph stress (замо́к / за́мок, цепи́, стены́) and е/ё (все / всё, чем / чём)
need the sentence. Russian-G2P uses RUAccent's two ONNX models for this, a
351 MB BERT and a 14 MB tagger. polyg2p replaces both with one
1.5M-parameter character-level student, `ru-ctx.g2p`, distilled from them.

Training data is 454k sentences that contain a homograph: Tatoeba, plus
Leipzig news, web and Wikipedia sentences chosen for the rarer homographs.
Sentences whose writer spelled всё / чём / нём with ё are also trained in a
copy where every ё is written е, so the student restores ё in text that omits
it. Teacher labels come from `tools/ru_distill.py`; training is
`tools/train_ru.py`.

Agreement with the teacher pipeline on held-out sentences:

| held-out set | homograph decisions | sentences identical |
|---|---|---|
| 5,000 Tatoeba sentences | **96.1%** (dictionary's first candidate: 48.9%) | 95.1% |
| 2,000 news and Wikipedia sentences | 90.5% | |
| the Tatoeba set with every ё rewritten as е | | 89.0% |

Stressed-ё words on the Tatoeba set: 98.4%.

## Cost and switches

The student runs only on sentences that contain a homograph, at about
0.35 ms each with the int8 kernel. `ru-ctx-small.g2p` (0.55M parameters,
92.0% of homograph decisions, 0.3 ms) is the faster option.

- An int8 copy of `ru-ctx.g2p` (1.7 MB, same accuracy) is compiled into the
  crate, so Russian works with no data folder.
- A `ru-ctx.g2p` in the data folder overrides the built-in copy.
- `POLYG2P_RU_CTX=0` turns the student off. Homographs then take the
  dictionary's first candidate and все/всё is decided by rule.

On aarch64 CPUs with the dot-product extension, Apple Silicon included, the
int8 weights run an inline-assembly `sdot` kernel. It is 3× faster than the
f32 math and changes 5 of 20,000 Tatoeba sentences. `POLYG2P_INT8=0` forces
f32. The Hebrew tagger (`he-int8.g2p`) uses the same kernel.

## Data

The stress dictionaries (`ru_*.lex`) are built from RUAccent's dictionaries,
Apache-2.0:

```console
python tools/ru_build_lex.py /path/to/Russian-G2P/ru_lexicon data-out
```

Without them, stress falls back to suffix rules and is often wrong. The
Python wheel bundles them.
