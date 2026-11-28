# Polyphone readings (`readings`)

Context readings for characters that have more than one reading: `G2p::readings(text, lang)` (Rust),
`Phonemizer.readings(text, lang)` (Python) and `g2p <zh|yue|ja>-readings` (CLI, one sentence per line in,
`index:reading` pairs out). `index` counts characters, not bytes. Readings are pinyin with tone digit for `zh`
(`zhong4`, neutral tone `5`, ü written `v`), Jyutping for `yue`, katakana for `ja`.

The module only chooses among a character's listed readings. Mapping a reading to phonemes (pinyin to IPA, tone
sandhi, word spacing) stays with the caller, so a front end can keep its own segmentation and sandhi and swap only
the reading step.

## How it works

`src/tagger.rs` now exposes its encoder body (character embedding, learned positions, pre-LayerNorm blocks, ReLU FFN,
final LayerNorm, int8 kernels) as a shared type. The Hebrew / Russian tagger keeps its three heads on top
(`tests/tagger_parity.rs` records a digest of the committed `ru-ctx.g2p` outputs, identical before and after the
refactor, and `tests/he_render.rs` passes unchanged). `src/polyphone.rs` puts one linear head over the reading
classes on the same encoder; the logits are masked to the character's own candidate readings (`char_classes` in the
header), so a character can never receive a reading it does not have. Texts longer than the 128-character window are
read in windows centred on each target.

Weight file: G2PT container, header `"kind": "polyphone"`, plus `classes` and `char_classes`; a header without `kind`
is the Hebrew tagger and loads as before. Files are looked up in the data directory as `zh-poly.g2p`,
`yue-poly.g2p`, `ja-poly.g2p`; without the file `readings` returns an empty list. Train with
`tools/train_polyphone.py` (tsv of `sentence<TAB>char_index<TAB>reading`, candidate-reading JSON).

`tests/data/zh_poly_fixture.g2p` is a 64-dimensional fixture trained only on the hand-written sentences of
`tests/zh_poly_cases.tsv`; it exercises the whole path (container, int8 weights, masked head, windows), it is not a
usable model.

## Results (round 3)

All numbers are on held-out sets that no model, selection or threshold touched. The Mandarin hold-out is a hash split
of the benchmark's training file (9,941 targets); the earlier test file had been scored in earlier rounds and is not
reused. The Cantonese hold-out is 8 files chosen by hash from the files that were not the earlier test files (the
earlier test files went into training). The read-speech set (evaluation only) was re-split by hash into a policy-tuning
half and a test half; it was not used for training.

### Mandarin (optional pack `zh-poly.g2p`, 12.7 MB int8, **not in the wheel, not in `data/`**)

Teacher: `bert-base-chinese` (Apache-2.0) fine-tuned on the benchmark's labelled sentences (99.0 % on its dev set).
Student: 6.2 M parameters (d 256, 6 layers, ff 768), trained on the teacher's readings (probability >= 0.9) of
CC0 / CC BY 2.0 FR sentences plus the labelled benchmark sentences. The header carries a deployment policy (`allow`,
13 characters, `cut` 0.6) chosen on the tuning half; with it, `readings` returns only those characters.

| test | dictionary reading (jieba + pypinyin) | this model |
|---|---|---|
| benchmark hold-out, 9,941 polyphonic targets, model on every target | 88.0 % | **97.7 %** (bar 97.0) |
| read speech, human pinyin, 30,082 sentences, 81,768 polyphonic syllables, with the policy | 93.73 % | **94.34 %** (+0.62, 95 % lower bound +0.54) |
| read speech, all syllables | 95.91 % | 96.06 % |

int8 against float: same arg-max on 100.00 % of targets, accuracy identical; Rust against the Python int8 loader:
identical on every target. Reading changes on a sample of 6,000 sentences of mixed corpora: 8.7 % of sentences.

Licence: the benchmark's sentences are Chinese Wikipedia text (CC BY-SA), so the weights are treated as a share-alike
derivative and are distributed apart from the wheel, like the other CC BY-SA packs.

### Cantonese (`data/yue-poly.g2p`, 5.0 MB int8, in the repository)

Teacher: `hon9kon9ize/bert-base-cantonese` (Apache-2.0 per its card) fine-tuned on the HKCanCor corpus (CC BY 4.0):
97.4 % on the dev files. Student: about 5 M parameters (d 192, 4 layers), trained on the HKCanCor gold readings and on
the teacher's readings (probability >= 0.9) of CC0 Common Voice Cantonese sentences and CC0 Cantonese storytelling
transcripts.

| test (8 held-out files, 3,470 polyphonic tokens) | ToJyutping | this model |
|---|---|---|
| reading accuracy | 85.3 % | **93.2 %** (bar 92.5; int8 93.2 %, same arg-max on 99.98 %) |

Errors that remain: 166 tone-only and 70 segmental; most frequent 哦, 到, 成, 話, 來. Rust against the Python int8
loader: identical on every test token. Reading changes on a sample of 4,000 to 5,000 sentences of each of two
Cantonese corpora: 38 % of sentences (12 % of polyphonic tokens).

### Japanese: no model

A Japanese teacher (`sbintuitions/modernbert-ja-130m`, MIT) fine-tuned on the furigana corpora of the National Diet
Library (Aozora Bunko and bibliographic titles) reads the JSUT heteronym tokens worse than the dictionary route
(18.9 % against 14.9 % error on the tuning half): the corpora are literary and bibliographic, and the readings they
teach (後 ノチ, 何 ナニ, 家 ウチ, 日本 ニホン, 中 チュー) differ from modern ones. Free modern text with human
readings was not found (the candidates are share-alike or restricted), so no Japanese model is shipped.

## Licences of what the Mandarin model was trained on

- g2pM repository and pip package: Apache-2.0 (LICENSE file). Its benchmark sentences are split from Chinese
  Wikipedia (Section 3 of the g2pM paper, arXiv 2004.03136), so the text is CC BY-SA 4.0 / 3.0 material that the
  repository redistributes under Apache-2.0. Nothing of the benchmark is copied here, only weights trained on it;
  whether such weights may be used commercially is an open question for the owner of this repository, which is why
  none are shipped.
- g2pW (YC-Chen/g2pW model card: Apache-2.0; bert-base-chinese: Apache-2.0) was tried as a teacher and not used in
  the final model.
- Common Voice sentences: CC0 (dataset README).
- pypinyin, jieba, OpenCC: used for evaluation and data preparation only.
