# Catalan, Central (`ca`)

Rules only (`src/ca.rs`, no lexicon, no model) plus one small statistics table
(`data/ca_open.tsv`: open/closed quality of unaccented stressed e/o by word
ending, aggregate counts derived from Wiktionary). Notation follows espeak-ng's
`ca` voice so TTS models trained on espeak phonemes can consume the output;
pronunciations follow the Central standard as transcribed by Wiktionary, and
beat espeak where the two disagree.

## Gold data

The shipped `sources/cat_latn_broad.tsv` has 176 lines (~70 frequent words) and
is mostly Algherese/colloquial entries (`part -> palt`, `mirall -> mirai`), so
it cannot measure anything: both systems score 82% on it. WikiPron's
`cat_latn_narrow.tsv` (98 709 entries, 42 559 words) is the real resource, but it
lists several unlabeled dialects per word (Central, Valencian, Balearic).
`tools/ca_gold.py` picks the Central candidates (drops `v ɫ s̺` forms; among
`ə`-forms keeps those with the fewest `ə`, since Balearic also stresses `ə`;
prefers reduced `o -> u`) and writes a 47k-entry gold under the name
`eval_acc.py` expects, so:

```
.venv/bin/python tools/ca_gold.py --fetch <scratch>/cat_latn_narrow.tsv <gold_dir>
POLYG2P_SRC=<gold_dir> .venv/bin/python tools/eval_acc.py ca --show 60
```

`tools/collapse_ca.py` maps both systems and the gold to one broad set: keeps
`ə ɛ ɔ` (phonemic), removes spirant/stop (β/b), tap/trill, `ɫ`, `ɱ`, glide vs
vowel spelling of i/u, geminates (`l·l`, `bbl`, `nn`), `ɲʃ/ɲs`, initial `dʒ/ʒ`.

## Accuracy vs Wiktionary (Central candidates)

| set | polyg2p word acc | polyg2p PER | espeak word acc | espeak PER |
|---|---|---|---|---|
| top 20k frequent words in the gold, frequency-weighted | **93.9%** | **0.7%** | 93.2% | 0.9% |
| 20k most frequent lower-case words, uniform weights | **95.9%** | **0.6%** | 94.7% | 0.9% |
| shipped `cat_latn_broad.tsv` (71 words, dialectal) | 82.2% | 16.9% | 82.1% | 18.1% |

The open/closed table alone predicts the quality of an unaccented stressed e/o
in 89.0% of held-out words (5-fold CV over 9 213 words); the remaining errors
are lexical (Catalan orthography does not mark the distinction), and a
hand-written list of ~400 frequent words (`OPEN_WORDS` / `CLOSED_WORDS`)
covers the common exceptions to the ending statistics.

## What the module does

- **Stress**: written accent; else penultimate for words ending in a vowel,
  vowel + s, -en, -in (a final i/u off-glide counts as a consonant: `palau`,
  `estiu`), final otherwise. `ˈ` immediately before the stressed vowel, as
  espeak writes it (`kˈazə`, `dəspɾˈes`, `əkˈɛstə`). Clitics and articles are
  unstressed (`də lə əl əls pəɾ əm u i`). Adverbs in -ment get two primaries
  (`rˈapiðəmˈen`); nouns in -ment one (`mumˈen`, `əʒuntəmˈen`), decided by the
  stem (accent, consonant stem, adjective endings, noun list).
- **Vowels**: stressed `a e ɛ i o ɔ u`; unstressed a/e -> `ə`, o -> `u`; an
  unstressed e before a stays `e` (`teˈatɾə`, `iðeˈal`). Open/closed from
  `è é ò ó`, else `data/ca_open.tsv` (longest ending key with count >= 3,
  with and without the preceding letter) after the exception lists. Final
  stressed -eu/-ei are open (`muzˈɛw`, `rəmˈɛj`) unless listed (`neu`, `rei`).
- **Diphthongs/hiatus**: falling `aj ej oj uj aw ew iw ow` (`ˈajɣwə`, `bˈɛwɾə`,
  `siwtˈat`, `kˈujnə`); rising glides only word-initially, between vowels or
  after g/q (`jugˈurt`, `nˈɔjə`, `dˈiwən`, `kwˈatɾə`, `ʎˈeŋɡwə`); i/u after a
  consonant before a vowel is a hiatus (`siˈɛnsiə`, `fəmˈiliə`, `əksiˈo`);
  `ï ü` always nuclei (`bəˈinə`, `rəˈim`), `gü/qü` -> `ɡw kw`; infinitives
  and gerunds in vowel + ir/int are hiatus (`kunstɾuˈi`, `uβəˈin`).
- **Consonants**: b/v merged (`bˈa`, `kəβˈaʎ`); `β ð ɣ` after a vowel, glide,
  liquid or fricative, stops word-initially, after nasals (and l for d),
  after stops and before consonants; `bl/gl` after a stressed vowel geminate
  (`pˈɔbblə`, `rˈeɡɡlə`, also pobl-/publ-). `c/ç/qu/gu`, `ll` -> `ʎ`,
  `l·l`/`ŀl` -> `ll`, `ny` -> `ɲ`, `tx tj tg dj` -> `tʃ dʒ`, `tz` -> `dz`,
  `j`/`g+e,i` -> `ʒ`, `gg+e` -> `dʒ`, `ig` final after a stressed i or a
  vowel -> `tʃ` (`bˈatʃ`, `mˈitʃ`, `pˈutʃ`; `kˈastik`), `x`: `ʃ` initially,
  after i (the i is silent after a vowel: `kˈaʃə`, `bˈaʃ`) and after a
  consonant (`ʃˈarʃə`), `ks` elsewhere (`tˈaksi`, `tˈekst`, `fˈiks`), `ɡz` in
  ex-/inex-/hex- + vowel (`əɡzˈamən`, `əɡzˈemplə`, `əɡzibisiˈo`); `s` -> `z`
  between vowels, before voiced consonants, in trans- + vowel (`tɾənzisiˈo`);
  `r`: `ɾ` between vowels, `r` initially, after n/l/s, doubled and in coda
  (`pˈart`, `mˈar`); `n` -> `ŋ` before velars, `m` before labials, `ɱ` before
  f, `ɲ` before `ʒ ʎ ʃ` (`məɲʒˈa`, `əɲʎˈa`); `c/g` before n -> `ŋ`
  (`tˈɛŋnik`); `tl tll` -> `ll ʎ`, `tm` -> `mm` in setmana, `mpt` -> `mt`
  (`kˈomtə`), `mb` + consonant -> `m` (`əmdˈos`), `xc+e` -> `ks`, `sc+e` -> `s`,
  `sh` is s + silent h (`ələzˈɔɾəs`), `ps-` keeps p (`psikuluʒˈiə`).
- **Word end**: devoicing (`sˈut`, `fɾˈɛt`, `bˈɛrt`, `klˈup`), `-nt -lt -nd`
  lose the stop (`sˈan`, `mˈol`, also before s: `pˈuns`), `-mp -mb` -> `m`
  (`kˈam`, `əm`), `-nc -ng` -> `ŋ` (`sˈiŋ`, `sˈaŋ`), `-sts` -> `ts`
  (`əkˈɛts`), `-rt -st` keep the stop. Final **-r** is silent in infinitives
  and in polysyllables in -ar/-er/-or/-ir (`kəntˈa`, `pɾimˈe`, `səɲˈo`,
  `kullˈo`; plural `kərˈes`), pronounced in monosyllables (`mˈar`, `kˈɔr`;
  silent in clar, dur, flor, por, ser, fer, dir, plor), in -ur (`futˈur`;
  silent in segur, madur), when the stress is not final (`dˈɔlər`, `kˈansər`,
  but -er infinitives: `kˈorə`, `kunˈɛʃə`) and in a list of ~130 nouns and
  adjectives (favor, motor, amor, popular, militar, radar, emir, ...);
  -rs stems (univers, divers, concurs) keep it.
- **Compounds and clitics**: hyphen parts keep their stress with sandhi at
  the junction (`bˈintiðˈos`); enclitics are unstressed and a verb keeps its
  final r/t before them (`purtˈarsə`, `fˈeɾu`, `ənˈaɾi`, `dˈonəməl`);
  proclitics `l' d' m' t' s' n'` glue on (`lˈɔmə`, `dˈunə`, `məɣɾˈaðə`),
  `se'n me'n se'ls` are read as clitics (`sən`). Prefixes with unreduced
  vowels: sobre-, ben-, gairebé, qualsevol, portaveu, passaport.
- **Sentence pass (`join`)**: final `s ʃ tʃ ts` voice before a vowel or voiced
  consonant (`əlz əmˈiks`, `ləz ˈɔɾəs`, `bˈadʒ ənˈa`), final `p t k f` only
  before a voiced consonant (`kˈab bəɣˈaðə`), initial `b d ɡ` soften after a
  vowel, glide, liquid or fricative (`mˈol βˈe`, `dˈoz ðˈiəs`, `tɾˈɛz ɣˈats`),
  final n assimilates (`uŋ ɡˈat`, `um bˈɔn`). Pairs split at a middle dot by
  the tokenizer (`col` + `legi`) are re-merged for the known l·l first halves.
- **Numbers**: `vint-i-un`, `trenta-cinc`, `dos-cents`, `mil`, `dos mil`,
  `un milió`, `dos milions`, `mil milions`, `un bilió`; decimal `coma`
  (3,5 -> `tres coma cinc`); symbols % per cent, $ dòlars, € euros, £ lliures,
  & i, + més, @ arrova, ° graus, = igual a.
- **Letters**: `a be se de e efə ʒe ak i ʒɔtə ka elə emə enə o pe ku erə esə te u
  be bedobblə iks iɣɾeɣə zɛtə`; a bare single consonant letter is read by name.

## Where espeak is wrong and we differ on purpose

- espeak drops the t of `aquest` (`əkˈɛt`), `text` (`tˈeks`) and keeps
  `-ig` as `tʃ` in `càstig`; it reads `anys` as `ˈaɲʃ` (Wiktionary `aɲs`).
- espeak does not reduce the first vowel of `reina` (`ɾəˈinə` with a tap),
  misreads `rei` as `rˈei`, reads `puig` with a secondary stress only.
- espeak keeps `o` in several -ment nouns/adverbs inconsistently
  (`rəsəmmˈen` for recentment) and gives the -ir hiatus words a diphthong
  (`kunstɾˈuj`).
- Open/closed: espeak and Wiktionary disagree on many frequent words
  (espeak `tˈems`/`sˈensə` closed where Wiktionary has `tems`/`sɛnsə`,
  espeak `mˈol` for molt); we follow Wiktionary's Central transcription
  where the ending statistics or the exception list say so.

## Known gaps

- **Tokenizer**: `text.rs` treats the middle dot `·` as a separator, so
  `col·legi` arrives as two words. `join` re-merges the common first halves,
  but the real fix is to keep `·` inside a word when it sits between letters
  (one line in `text::tokenize`, outside this module's files).
- Open/closed e/o of unaccented words is ~89% by rule; rare words and proper
  names are guessed from endings.
- Unstressed e/o kept in hiatus (`vídeo`, `aeroport`, `poeta`) and compounds
  with unreduced vowels are partly list-based.
- Final -r in -ar/-er/-or adjectives (popular, militar) is a list; Wiktionary
  itself lists both readings for many.
- Wiktionary quirks we do not imitate: plural `-dors` without the s
  (`ləkto`), `-eu` names (Andreu `e`, Mateu `e` but museu `ɛ`).
- Dialects: Central only (no Valencian/Balearic mode).
- No ordinals, Roman numerals, or year-specific number reading.
