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

## Mandarin: measured (held-out sets, one model)

Model: 6.4 M parameters (d 256, 6 layers, ff 768, 128-character window), 13 MB int8. Trained on the labelled
sentences of the g2pM benchmark (Apache-2.0), their Traditional-script copies (OpenCC `s2t`), and the readings the
g2pM model (Apache-2.0) assigns to every polyphonic character of those sentences and of CC0 Common Voice sentences.

| test | current dictionary reading (jieba + pypinyin 0.55) | g2pM | this model (int8, Rust) |
|---|---|---|---|
| g2pM test, 10,254 polyphonic targets (class-balanced, Wikipedia sentences) | 87.9 % | 97.3 % | 97.2 % |
| human-annotated pinyin of read speech, 4,000 sentences, characters of the polyphone list (evaluation only) | 93.9 % | | 91.4 % (all characters), 93.5 % (only outside dictionary words), 93.8 % (outside dictionary words, p >= 0.99) |

int8 against PyTorch float: the same arg-max on 99.98 % of the targets, test accuracy equal to 0.01 point; Rust
against the Python int8 loader: identical on every target.

The class-balanced benchmark over-represents rare characters and written text. On read speech the pypinyin
dictionary-plus-default reading is better than this model for every combination of policy and threshold that was
tried, so **the model is not enabled anywhere and no weights are shipped**. The module is the mechanism: a model
trained on spoken-domain data would plug in without code changes.

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
