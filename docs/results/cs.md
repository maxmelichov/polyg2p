# Czech (`cs`)

Rules only (`src/cs.rs`, no lexicon, no model). Notation follows espeak-ng's
`cs` voice so TTS models trained on espeak phonemes can consume the output;
pronunciations follow the standard (Bohemian) norm as transcribed by
Wiktionary, and beat espeak where the two disagree.

## Accuracy vs Wiktionary (WikiPron `ces_latn_narrow.tsv`)

`tools/eval_acc.py cs` with `tools/collapse_cs.py` (strips narrow detail:
ɪ/i, ɛ/e, ɦ/h, u̯/ʊ, schwa epenthesis, voicing diacritics, syllabic marks,
geminates; keeps vowel length and the palatal series).

| set | polyg2p word acc | polyg2p PER | espeak word acc | espeak PER |
|---|---|---|---|---|
| top 20k frequent words in the gold, frequency-weighted (18 879 words) | **99.9%** | **0.1%** | 96.4% | 0.8% |
| same list, lower-case words only, uniform weights (8 778 words) | **99.8%** | **0.1%** | 96.5% | 0.6% |
| all gold words among the top 80k by frequency (22 382) | 198 errors (0.9%) | | | |

Exact-string agreement with espeak on the top 20k words is 85.7%; almost all
of the remainder is deliberate (see below).

## What the module does

- **Stress**: `ˈ` before the first nucleus (after the onset, espeak style),
  `ˌ` on odd non-final syllables from the third on (`nˈeɪpravɟˌepodˌobɲejʃiː`).
  Clitics and prepositions are unstressed (`se`, `jsem`, `na`, ...), a few
  function words get only `ˌ` (`nˌebo`, `bˌude`), as espeak does.
- **Vowels**: length from diacritics (`á é í ó ú ů ý` -> `ː`), `i/y` -> `i`,
  diphthongs `ou au eu` -> `oʊ aʊ eʊ` (espeak's symbols), `ej` -> `eɪ`
  (but `ěj` stays `ej`, matching espeak). Hiatus across prefixes:
  `po-u` (použít, poučit, poukázat), `na-u`, `za-u`, `ne-u` (neuvěřitelně;
  not neutrální/neuro-). `i` + vowel inserts `j` (poezije, raːdijo).
- **Palatals**: `ě` after d/t/n -> `ɟe ce ɲe`, after b/p/v/f -> `Cje`,
  after m -> `mɲe`; `d/t/n + i/í` -> `ɟ c ɲ` except in foreign stems. The
  foreign stems come from two sources: generic patterns (`-tiv-`, `-tik-`,
  `-tick-`, `-tiz-`, `di` + f/g/k/p/r/vowel, `ni` + a/o/u, nouns in `-ie`
  and their inflections) and a list of ~250 stems (`minist`, `techni`,
  `politi`, `uni`, `studi`, ...) with a short native-override list
  (`tito`, `vtip`, `protiv`, `hledisko`, `vznik`, ...). `nn` + ě/i -> one `ɲ`.
- **Consonants**: `ch` -> `x`, `h` -> `h` (espeak's symbol for ɦ),
  `c č š ž ř ď ť ň` -> `ts tʃ ʃ ʒ r̝ ɟ c ɲ`, `n` before k/g -> `ŋ`, `x` ->
  `ks` (`eɡz` in ex- + vowel), `q` -> `kv`, `w` -> `v`, doubled letters ->
  one phone (Anna, činnost; `oddělení` keeps `dɟ`), `d` before c/č and `t`
  before c dropped (dcera, srdce, otce, rozhodčí), `-ismus` -> `izmus`.
- **Voicing**: final devoicing (`lef`, `hrat`, `buːx`), regressive
  assimilation through whole clusters (`fstup`, `spjet`, `zbor`, `ɡdo`,
  `svadba`); `v` and `ř` take voicing but don't spread it (`svatba`, `tvůj`);
  `ř` is `r̝̊` after a voiceless consonant and word-finally (`tr̝̊i`, `ker̝̊`).
  `sh` is read `zh` (Wiktionary's choice; Bohemian `sx` would be the
  alternative).
- **Syllabic consonants**: `r̩ l̩` between consonants or finally after a
  consonant (`kr̩k`, `vl̩k`, `nesl̩`), `m̩` finally (`sedm̩`). Word-initial
  r/l before a consonant is not syllabic (`lze`, `rty`), as in Wiktionary.
- **Sentence pass (`join`)**: a monosyllabic preposition and the next word
  form one stress unit written as one token, like espeak's `nˈastole`;
  non-syllabic `v k s z` are glued on and assimilated (`fprˈaze`, `ɡdˈomu`,
  `zbrˈatrem`); vocalic ones take the stress (`nˈastole`, `bˈespraːtse` —
  the preposition's final obstruent is devoiced before v, standard Czech).
  `se` is a preposition only before s/z/š/ž-initial words. Word-final
  obstruents are voiced before a word beginning with a voiced obstruent
  (`hrˈad bˈil`), not before sonorants or vowels.
- **Numbers**: full Czech cardinals with `dvě stě`, `tři sta`, `pět set`,
  `tisíce/tisíc`, `miliony/milionů`, `miliardy/miliard`; decimal separator
  `celá` (3,5 -> `tři celá pět`). Symbols: % procent, $ dolarů, € eur,
  £ liber, & a, + plus, @ zavináč, ° stupňů, = rovná se.
- **Letters and abbreviations**: letter names for spelling (`LETTERS`),
  plus Czech letters (ČR -> `tʃˌeːˈer`, ČSSD) handled inside `word` for
  vowel-less tokens that are not Czech words (krk, vlk, smrt, ... are read);
  common abbreviations expanded (km, kg, tzv., tj., např., atd., Kč, mld.).
- **Loanwords**: ~130 frequent English/French/German spellings respelled
  into Czech orthography (design -> dyzajn, software -> softvér, Josef ->
  jozef, filosofie -> filozofie, Washington -> vašinkton).

## Where espeak is wrong and we differ on purpose

- espeak reads the prepositions `v k s z` as letter names (`vˈeː`, `ˈes`)
  and never assimilates them; we glue them to the next word.
- espeak does not devoice final `v` (`stˈav`, `lˈev`), keeps `k` in `kdo`,
  `kde`, `když`, devoices before `h` (`rˈoshodɲe`, `nˈaːtherneː`) and
  keeps the stop before an affricate (`sˈr̩ttse`, `ttsˈera`).
- espeak palatalizes many foreign stems (`fˈescival`, `tˈexɲika`,
  `pˈozicivɲiː`) and misses native ones (`tˈitul` is right, but `vznikla`
  is `vzɲˈikla` and espeak gets it, we match).
- espeak writes `dʑ` for `dz` (`pˈodʑim`), spells `vlk` as letters, treats
  `použít`/`neustále` as diphthongs, and skips the `j` glide in `i` + vowel
  hiatus (`sˈotsiˌaːlɲiː` vs Wiktionary `sot͡sɪjaːlɲiː`).
- Numbers: espeak reads `3,5` as `tři čárka pět` and merges number words
  into one token (`tr̝̊ˈistˈa`); we produce separate words with `celá`.

## Known gaps

- Foreign-stem palatalization is a list; Wiktionary itself is inconsistent
  (`Daniel` hard / `Daniela` soft, `tradice` hard / `tradic` soft,
  `manipulace` hard / `manipulovat` soft, `-ismus` -> z but `-ismu` -> s).
  Remaining errors in the top 20k are almost all of this kind.
- English names and brand names not in the respelling table are read with
  Czech letter values.
- `sh` is always `zh`; speakers from Bohemia say `sx` in some words
  (shoda, shodit).
- No ordinal numbers (`3. května`), no Roman numerals (read as acronyms),
  no year-specific number reading.
- Secondary stress inside merged preposition units is not recomputed.
- `eval_acc.py` maps `cs` to `ces_latn_broad.tsv`; the gold shipped in
  `sources/` is `ces_latn_narrow.tsv` (the evaluation above used a symlink
  with the broad name).
