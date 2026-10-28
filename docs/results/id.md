# Indonesian (`id`)

Rules only (`src/id.rs`, no lexicon, no model) plus a ~560-entry table of
stems for the one real ambiguity of the spelling, the letter `e` (schwa vs
front vowel). Notation follows espeak-ng's `id` voice; pronunciations follow
Wiktionary (WikiPron) where the two disagree.

## Accuracy vs Wiktionary (WikiPron `ind_latn_broad.tsv`)

`tools/eval_acc.py id` with `tools/collapse_id.py` (folds lax vowel
allophones ɪ ʊ ɛ ɔ, diphthong spellings aɪ / a i̯, the glottal stop WikiPron
writes for final k and before initial vowels, unreleased stops, espeak's ç for
sy, hiatus glides ija/uwa, n before palatal affricates, stress).

| set | polyg2p word acc | polyg2p PER | espeak word acc | espeak PER |
|---|---|---|---|---|
| top 20k frequent words found in the gold, frequency-weighted (8 250 words) | **99.6%** | **0.6%** | 88.4% | 2.8% |
| all lower-case gold words, uniform weights (16 506 words) | **90.9%** | **1.5%** | 80.0% | 3.1% |

The e-table was derived from the same gold (aggregate statistics: for each
written `e` of each word, schwa or not, grouped by stem), so the numbers
above are in-sample for the ~560 stems it holds. Held-out estimate (2-fold
cross-validation over the gold words containing `e`, table built from one
half, scored on the other): e-words correct 92.2% (rules only) -> 94.9%
(rules + the half-table), frequency-weighted; uniform 77.1% -> 78.3%. A
frequent word is almost always in the table; a rare unseen word with `e`
is right about three times out of four.

## What the module does

- **Consonants**: `c j ny ng ngg sy kh` -> `tʃ dʒ ɲ ŋ ŋɡ ç x` (espeak's
  symbols; `ç` is what espeak writes for sy), `y` -> `j`, `q` -> `k`,
  `x` -> `ks`, `v f z` kept (`vˈidɛo`, `zˈaman`), `sh ch th dh bh gh ph`
  -> `ç tʃ t d b ɡ f` (loans and old spelling: Yudhoyono, Thamrin, Soekarno
  with `oe` -> `u`), doubled consonants -> one (`ˈalah`), `nny` keeps
  the n (`laɪnɲa`). Final k stays `k` (espeak); the gold's `ʔ` is folded.
- **Vowels**: `ai au` -> `aɪ aʊ` (one syllable, as espeak counts them;
  `oi` is two), other sequences are hiatus (`dˈia`, `kəluˈarɡa`, `sˈaat`),
  `o` -> `ɔ` before r (`ˈɔraŋ`, `nˈomɔr`), taling e -> `ɛ`, pepet -> `ə`.
- **e / ə** (the hard part), decided per written `e`:
  1. the whole word, then the word without `-nya`, `-kan/-an/-i`, then with
     up to three derivational prefixes peeled (`memper- diper- ber- ter-
     per- me(m/n/ng/ny)- pe(m/n/ng/ny/l)- ke- se- di-`), is looked up in
     `E_TABLE` (stems with their e-pattern; `=`-keys match whole words only,
     so `=beri` is `bɛri` but `berikan` is `bərikan`);
  2. otherwise prefix e's are schwa and the stem follows rules:
     non-final `e` is schwa unless the stem looks foreign (onset cluster
     `pr tr kr st sp…`, obstruent cluster after e `kn kt ks pt`, letters
     f/v/z/q) or stands in hiatus, before `w`, or before `h`+vowel
     (`daɛrah`, `lɛwat`, `sɛhat`); word-initial `e` + nasal in a short
     word is schwa (`əmpat`, `ənam`), otherwise `ɛ` (`ɛkonomi`); a final
     closed syllable has `ɛ` (`-en -et -ek -es -el -eh`) except `-er`/`-em`
     (`sˈumbər`, `sˈistəm`); a final open `e` is `ɛ` in two-syllable
     stems (`sˈorɛ`, `kˈuɛ`) and schwa in longer ones (`pɛriˈodə`,
     `-isme`); monosyllables have `ɛ`.
- **Stress**: `ˈ` right before the vowel of the penultimate syllable,
  or of the last one when the penult has schwa (`təlˈah`, `bənˈar`,
  `səmbˈilan`); `ˌ` on every second non-schwa syllable before it
  (`mˌanusˈia`, `ˌunivərsˈitas`). Function words follow espeak: `yang dan
  di ke si` unstressed, `dari untuk pada atau bahwa adalah sebagai` with
  `ˌ` only. Hyphenated reduplications are two stressed words
  (`ˈɔraŋ ˈɔraŋ`).
- **Numbers**: `nol satu … sembilan`, `sepuluh sebelas`, `X belas`, `X
  puluh`, `seratus`, `seribu`, `satu juta / miliar / triliun`; decimal
  separator `koma` (3,5 -> `tiga koma lima`; `1.500` is a thousands
  group). Symbols: % persen, $ dolar, € euro, £ paun sterling, & dan,
  + plus, @ at, ° derajat, = sama dengan.
- **Letters and abbreviations**: letter names (`a be tʃe de e ɛf ɡe ha i
  dʒe ka ɛl ɛm ɛn o pe ki ɛr ɛs te u fe we ɛks je zɛt`) for acronyms (DPR,
  PBB) and vowel-less tokens (`tv`); ~45 written abbreviations expanded
  (yg, dgn, dll, dsb, tsb, tdk, krn, spt, jl, rp, km, kg, cm, bpk, no, …).

## Where espeak is wrong and we differ on purpose

- espeak guesses `e` with its own heuristics and gets the frequent words
  wrong in both directions: `mərˈɛka` is right but `tərsˈɛbut`,
  `pˈɛrnah`, `kˈɛtʃil`, `tʃˈɛpat`, `lˈɛbih`, `bˈɛsar`, `sˈɛdaŋ`,
  `tˈɛmpat`, `pˈɛrlu` should have schwa, and `ˈoləh`, `kˈakək`,
  `ˈidə`, `kˈodə`, `ˈokə`, `prəsˈidən` should have `ɛ`. Its word
  accuracy on frequent words is held down almost entirely by this.
- espeak reads `oi` (`ambˈoi`), `ei` and loans like `quran` (`kwrˈan`)
  oddly, writes `səɹˈibu` for seribu (an English r), glues number
  words into one token (`dˈuapˌuluhsˈatu`) and stresses `ˈəmpat` on a
  schwa; we give separate words and `əmpˈat`.
- espeak spells `yg`, `dll`, `tv` as letters; we expand the written
  abbreviations.

## Known gaps

- The e-table is lexical: an unseen stem with taling e in a non-final
  syllable (`sepeda`-type words) gets schwa unless a loan signal fires.
  Non-final `e` before `r` is the least predictable context (`seperti`
  vs `materi`).
- Final voiced stops are kept (`klub`, `sebab`); WikiPron devoices a
  few (`klup`, `ɡaip`) but keeps most, and espeak keeps all.
- Schwa deletion in fast speech (`karna`, `məntri`, `filəm` epenthesis)
  is not modelled; the gold lists both forms for some words.
- English loanwords (`real`, `break`, `sound`, `dealer`) are read with
  Indonesian letter values, like espeak.
- `Rp 5.000` is read `rupiah lima ribu` (the symbol first): making `Rp`
  a currency prefix needs a change in `num.rs`/`lib.rs`.
- Stress on `-nya`/`-lah` forms follows the whole word (`rumˈahɲa`), as
  espeak does; some speakers keep the stem stress.
