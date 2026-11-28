# Heteronyms: context models for the European languages

A heteronym is a word spelled one way and read two ways. A keyword rule ("the word after `je` is a verb") is
fragile, so for the languages below a small character-level encoder reads the whole sentence and picks the
reading of each listed word. The rules stay as the fallback: **without the model file nothing changes, and the
output is byte-identical to the rules.** English has its own work and Russian has `ru-ctx.g2p`
([russian.md](russian.md)); Hebrew and Yiddish are not covered here.

## How it works

- `data/hetero_<lang>.tsv` (compiled in, hand-written): the words and their readings.
  Columns `word, k, reading, tag, note, mode`. `k` is the class (0 to 3). `reading` is `rule` (keep what the
  rules produced), `respell:<spelling>` (the engine's own word function on another spelling),
  `ph:<phonemes>` or `sub:<from>><to>` (the rules' phonemes with the last `<from>` replaced).
  A word `*suffix` stands for an open class (French `*tions`). `tag` is the part of speech or lemma a tagger
  gives for that reading (used to label training sentences). `mode = rule` keeps the rules for that word.
- `<lang>-ctx.g2p` (optional pack, in the data directory): the encoder, the same `G2PT` file as the Hebrew and
  Russian taggers, 1.46 M parameters, 1.7 MB as int8. The class of a word is the `vow` head read at its first
  character. The `cons` head is trained to predict the part of speech of every word (an auxiliary task that
  teaches syntax); it is ignored at run time.
- `src/hetero.rs` runs after the per-word pass and before the language's sentence pass, only on sentences
  that contain a listed word. `POLYG2P_CTX=0` disables every model of this module.
- `hetero_check <lang> [--data DIR] [--raw]` reads `sentence TAB word TAB nth` and prints the class the rules
  produced, the class the model picks and the phonemes of every class. `--dump` prints the encoder input and
  the table words of each sentence (the Rust engine is the only place the sentence is normalised).

## Results

Accuracy = the phonemes of the predicted class equal the phonemes of the gold class (classes that sound
alike are not errors). All numbers go through the Rust path with the int8 file. "Gold" is human annotation
(Universal Dependencies parts of speech and lemmas, mapped to the table's classes). "Silver" is held-out
Tatoeba sentences labelled by cue rules written down in `tools/hetero_silver.py` (the cue word is deleted
from half of the training sentences). "Hand" is sentences labelled by the person running the experiment.
The sets are reported apart and never mixed.

| lang | words (active on the model) | table words per 1,000 Tatoeba sentences | gold (n) rules -> model | silver (n) rules -> model | hand (n) rules -> model | verdict |
|---|---|---|---|---|---|---|
| fr | 27 + `*tions` listed (5 had been active: sens, vis, lis, fier, os) | 17.6 | UD dev/test (196) 97.5 -> 98.0; UD train split (1,282) 98.3 -> 98.7 | (984) 79.7 -> 96.4 | (71) 59.2 -> 91.5; the 31 cases of `tests/hetero_cases.tsv` 51.6 -> 71.0 | rules kept: measured and dropped by the gate (+0.5 point on the human gold, gate +5); `fr-ctx.g2p` is not loaded |
| pt (Brazil) | 28 pairs listed (8 active) | 18.7 | UD dev/test (260) 83.9 -> 92.7; UD train split (993) 87.4 -> 94.3 | (519) 39.9 -> 85.4 | (18, `tests/hetero_cases.tsv`) 50.0 -> 77.8 | model kept, optional pack |
| it | 24 pairs listed (6 active) | 17.6 (74% is `ancora`, kept on the rules) | UD dev/test (116) 74.1 -> 87.9; UD train split (737) 74.0 -> 86.3 | (231) 84.4 -> 92.2 | (15, `tests/hetero_cases.tsv`) 46.7 -> 73.3 | model kept, optional pack (eval sets non-commercial: not redistributed) |
| nl | `een`, regent, voornaam | 144.7 (`een`) | UD (1,667) 97.0 -> 89.6 | | | rules kept: the model is below the rules |
| de | 6 words | 0.6 | UD (58 decisions) 60 -> 60 on dev/test | | | rules kept: too rare, too little gold |
| uk | 25 pairs | 9.3 | none (no permissive tagger) | (32) 81.3 | (56) 64.3 (rules only) | rules kept: no clean teacher |
| ro | `copii` | 2.0 | 37 decisions | | | rules kept: too rare |

Notes on the verdicts (gate in the plan: keep a model only if it is clearly above the rules and no listed word
with 50 or more gold decisions is below its rule accuracy):

- fr (dropped), pt, it: the human gold is dominated by words the rules already read correctly (French newswire is full of
  `président`, `fils`, `-tions` nouns), so the gain there is a few points; the gain is concentrated in the
  classes where the rules are wrong (verb readings: `je sens`, `je vis`, `eu gosto`, `ho seguito`), which the
  silver and hand sets measure. French missed the +5 point gate on the human gold (+0.5 point dev/test, +0.4
  train split; the gold holds almost none of the decisions the rules get wrong). A model that misses a pre-set
  gate does not ship, so French is dropped: it stays on the rules, `src/hetero.rs` does not load or recognise
  `fr-ctx.g2p` (`model_languages`: pt and it only), and the table, labeller and trainer stay generic. The
  silver and hand numbers above are kept as a record, not as a reason to ship.
- Which words stay on the rules (`mode = rule`, lists in `tools/hetero_rule_<lang>.txt`): the gate (a word with
  50 or more gold decisions where the model scored below the rules: pt `governo`, `jogo`; it `subito`,
  `ancora`) and an evidence floor (a word needs 30 training decisions in each class, or 15 or more gold
  decisions with the model 10 points above the rules). The floor removed every French `-ent` word (the tagger
  labels are noisy there: a river is a `affluent`, tagged as a verb) and the `*tions` class (on running text
  the model read plural nouns such as `supplications` as the verb form).
- int8 against f32: the same class on 99.96% (fr, not shipped), 100% (pt, it) of decisions.
- The Italian gold treebanks (ISDT, VIT, ParTUT) are non-commercial: they were used to measure only.
- The Ukrainian pairs and their readings are in the table; with no permissive tagger the only labels are
  cue rules, which give 265 training decisions: not enough for a model. The rule baseline is in the table.
- Not built, by the frequency gate: Swedish (candidate words are 1.3% of Tatoeba sentences, but `men`, `japan`,
  `kort` have one dominant reading; the genuinely two-way words are about 0.3%; the engine writes no tonal
  accent, so the tonal pairs are out of scope), Greek (the pairs differ in the written accent already),
  Turkish and Catalan (not measured further: Catalan `dona` is a noun in 98 of 99 treebank tokens, which is a
  lexicon fix, not a context decision).
- Spanish, Polish, Czech, Hungarian, Finnish: no context-dependent class found (written accents, fixed stress
  or word-level lexicon matters), so there is no table.

## Cost bound and confidence

- One encoder pass per window, shared by every listed word inside it (the model's window is `max_len` = 320
  characters; a hit needs a quarter window of context after it, so a pass advances about 160 characters), and
  at most `MAX_PASSES` = 32 passes per `phonemize` call; hits beyond the cap keep the rules. Before this, every
  listed word of a text longer than one window paid its own pass (`"gosto " * 3000` took over 40 s). A sentence
  that fits one window still costs one pass. The tests assert pass counts, not wall time.
- A truncated or corrupt `<lang>-ctx.g2p` is an error at load (every read of the file is bounds-checked and the
  tensor sizes are checked against the config), and the engine keeps the rules.
- Confidence threshold (keep the rules when the model's probability of its pick is below t): measured and
  not kept. t was chosen on the UD train split only (the model never saw UD), then read on the held-out sets.
  Accuracy of the shipped class, rules -> t = 0 (no threshold) / 0.7 / 0.8 / 0.9:

  | lang | set | n | rules | 0 | 0.7 | 0.8 | 0.9 |
  |---|---|---|---|---|---|---|---|
  | pt | UD train split (choice) | 993 | 87.4 | 94.3 | 94.3 | 94.3 | 94.3 |
  | pt | UD dev/test (held out) | 260 | 83.8 | 92.7 | 92.7 | 91.9 | 91.5 |
  | pt | silver | 519 | 39.9 | 85.4 | 85.4 | 85.4 | 85.2 |
  | it | UD train split (choice) | 737 | 73.9 | 86.3 | 85.2 | 84.7 | 83.4 |
  | it | UD dev/test (held out) | 116 | 74.1 | 88.8 | 87.9 | 87.1 | 87.1 |
  | it | silver | 231 | 84.4 | 92.2 | 91.3 | 91.3 | 90.0 |

  The choice set never improves with a threshold, held-out accuracy falls, so none is applied
  (`Decision::model_conf` is still recorded). Of the six reported wrong readings, `questão de gosto` (0.73),
  `tinha gosto de madeira` (0.54) and `Se solo avessi seguito il tuo consiglio` (0.57) would be fixed at
  t >= 0.8; `Puoi scrivere in qualsiasi lingua desideri` is wrong at 0.9999 and no threshold helps; `Olho por olho`
  and `pintadas com gosto` are read correctly by the current pt model.

## Russian (measured, no change)

`ru-ctx.g2p` was only ever scored as agreement with its teacher. Against sentences with the reading chosen by a
person (191 Tatoeba sentences over 22 homograph words, labelled by the experimenter; 679 sentences for `ё`
taken from writers who typed `ё`, with the `ё` removed for the test):

| set | n | first dictionary candidate | `ru-ctx.g2p` | RUAccent (the teacher) |
|---|---|---|---|---|
| stress homographs, all | 191 | 55.0 | 86.4 | 92.7 |
| lexical pairs (замок, мука, орган ...) | 83 | 49.4 | 83.1 | 96.4 |
| grammatical-form pairs (руки, города, окна ...) | 108 | 59.3 | 88.9 | 89.8 |
| `ё` restoration, writers' spelling | 679 | 43.3 | 81.9 | about 89 (285 sentences) |

The student is 6 points below its teacher on stress and clearly worse on `небо` (the sky is restored as
`нёбо` the palate in 88% of the writers' sentences; the teacher is also weak there, 47% right). A retrain with
more data is a separate proposal.

## Licences

Every treebank and tool was checked against its own licence text (its README) before use. In short: Tatoeba sentences are
CC BY 2.0 FR; the label chain for fr, pt, nl uses Stanza (Apache-2.0) models trained on CC BY-SA treebanks and
for it on a CC BY treebank, with word vectors and character models that were not audited. Because of that chain
the model files are an optional pack like the CC BY-SA lexicons, not part of the wheel. A chain with a
non-commercial link (UD Ukrainian-IU, UD Italian ISDT/VIT/ParTUT, Greek GDT, Turkish IMST, Swedish LinES,
Russian SynTagRus) is evaluation only.

## Rebuilding

```bash
tools/hetero_label.py gold   --lang fr --ud fr_gsd-ud-dev.conllu ... --out gold/fr.tsv     # UD gold
tools/hetero_label.py silver --lang fr --sentences tatoeba_fr.txt --out silver/fr_cue.jsonl --out-eval gold/fr_silver.tsv
tools/hetero_label.py label  --lang fr --sentences tatoeba_fr.txt --out silver/fr.jsonl   # Stanza teacher labels
tools/train_ctx.py silver/fr.jsonl,silver/fr_cue.jsonl models/fr --lang fr
tools/hetero_eval.py --lang fr --gold gold/fr.tsv --data models/fr_int8_dir                # rules vs model
tools/hetero_diff.py --main main/g2p --branch target/release/g2p --text-dir tatoeba --langs fr de ...
```

Gold sets are not committed. UD treebanks come from github.com/UniversalDependencies; Tatoeba from
downloads.tatoeba.org/exports/per_language. `tools/hetero_table_{pt,uk,it}.py` regenerate those tables.

## Not covered: languages that still read spelling

| lang | what a G2P needs | open resource to check (licence to be read first) |
|---|---|---|
| bg | stress dictionary (free, unmarked stress); vowel reduction (а/ъ, о/у); final devoicing and voicing assimilation; stress homographs need a context model later | Wiktionary/WikiPron bg |
| da | a pronunciation lexicon plus a small unknown-word model (as `en.g2p`); stød, soft d, r-vocalisation, schwa assimilation, vigesimal numbers, compounds | NST Danish pronunciation lexicon, WikiPron da |
| et | the third quantity degree and palatalisation are not written: lexicon or morphological analyser; first-syllable stress with loanword exceptions | Vabamorf, EKI dictionaries |
| hr | segments by rule; stress place, length and tone are not written: accented lexicon | hrLex (CLARIN.SI) |
| lt | free stress and tone are not written: accentuation dictionary; palatalisation and vowel length by rule | Liepa project tools, Wiktionary lt |
| lv | initial stress with exceptions; open/closed e and three readings of o are not written: lexicon plus rules | Tezaurs.lv, WikiPron lv |
| sk | closest to done: port `cs.rs`, add de/te/ne/le softening with a loanword list, ä, ô, v-vocalisation | WikiPron sk |
| sl | free stress, open/closed e/o, schwa, l/v -> w: accented lexicon plus an unknown-word model | Sloleks 3.0 (CLARIN.SI) |

Common to all eight: number and abbreviation expansion (`num.rs`), symbols inside the 256-token vocabulary, a
top-20k Wiktionary score with `tools/eval_acc.py`.
