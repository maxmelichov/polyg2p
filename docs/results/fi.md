# Finnish (`fi`)

Rules only (`src/fi.rs`, no lexicon, no model). Notation follows espeak-ng's
`fi` voice so TTS models trained on espeak phonemes can consume the output;
pronunciations follow the standard language as transcribed by Wiktionary and
go beyond espeak on compounds, loanwords and sandhi.

## Accuracy vs Wiktionary (WikiPron `fin_latn_broad.tsv`)

`tools/eval_acc.py fi` with `tools/collapse_fi.py` (strips stress, the
non-syllabic mark on diphthong offglides, boundary-gemination `ˣ`, espeak's
allophonic `ɪ`, the `ɑ/a` and `ʋ/v` symbol choice, the predictable nasal
place before k/p; keeps vowel and consonant length).

| set | polyg2p word acc | polyg2p PER | espeak word acc | espeak PER |
|---|---|---|---|---|
| top 20k frequent words in the gold, frequency-weighted (14 638 words) | **99.5%** | **0.2%** | 99.5% | 0.2% |
| same list, lower-case words only, uniform weights (13 210 words) | **99.4%** | **0.1%** | 99.2% | 0.2% |

Finnish spelling is phonemic, so both systems are near the ceiling. The
frequency-weighted set is dominated by tokens the gold reads as bare sounds
(`t` -> `t`, `i` -> `i`, `ä` -> `æ`), lower-case initialisms (`oy`, `wlan`)
and English words; we read letters by their names (`teː`, `iː`) and expand
abbreviations (`esim.` -> esimerkiksi), which the gold counts as errors. On
ordinary words the remaining differences are a handful of loanwords.

Exact-string agreement with espeak on the same words is 96.8%; nearly all of
the rest is secondary stress in compounds (see below).

## What the module does

- **Segments**: `a e i o u y ä ö` -> `a e i o u y æ ø`, doubled vowel ->
  `ː`, the 18 diphthongs (`ai ei oi ui yi äi öi au eu iu ou ey iy äy öy ie uo
  yö`) as vowel pairs, everything else hiatus; `aii` -> `a|iː`, `aai` ->
  `aː|i` (as espeak). Long stops `pː tː kː` (also `bː dː ɡː`), long sonorants
  and fricatives doubled (`ss ll nn mm rr`), `ng` -> `ŋŋ` (before a
  consonant `ŋ`: englanti), `nk` -> `ŋk`, `n` before p/b -> `m` and before
  k/g -> `ŋ` also across compound seams (`eteːmpæin`, `suomeŋkielinen`).
  Loan letters: `c` -> `s` before e/i/y else `k`, `ck` -> `k`, `q(u)` ->
  `k(v)`, `w` -> `v`, `x` -> `ks`, `z` -> `ts`, `š ž` -> `ʃ ʒ`, initial
  `sh`/`ch` -> `ʃ`/`tʃ`, `å` -> `o`. Apostrophes are syllable boundaries
  (`vaa'an` -> `vaːan`).
- **Unstressed i**: espeak writes a short or long `i` nucleus without stress
  as `ɪ`/`ɪː` (`hˈelsɪŋkɪ`, `nˈækemɪːn`); diphthongs keep `i`. We do the same.
- **Stress**: `ˈ` before the first vowel. Secondary stress follows espeak's
  weight-sensitive alternation, reverse-engineered from 14k words: it falls
  on the third syllable, moves to the fourth when the third is light and the
  fourth heavy (`ˈeuroːpːalˌainen`, `tˈodennækˌøisestɪ` in espeak), skips
  two light syllables before a non-final heavy one (`ˈitalɪalˌainen`), then
  repeats every second syllable and never lands on the last (`vˈuosɪtuhat`
  gets none in espeak). Heavy = long vowel/diphthong or closed syllable.
  espeak's explicit rule that the vowel before word-final `-lla` (a/e/i +
  lla, e + llä) is unstressable is replicated (`kˈaverɪlla` vs
  `kˈaverˌissa`).
- **Compounds** (where we deliberately differ from espeak): the stress
  pattern restarts with `ˌ` at a compound boundary, found by (a) numeral
  morphemes (`kˈaksɪkˌymmentævˌiːsɪ`), (b) hyphens, (c) a vowel-harmony
  violation, which proves a boundary between the last front/back vowel and
  the first vowel of the other class — the candidate onsets in that stretch
  are scored (both parts at least two syllables, a legal onset, plausible
  final letter, hiatus before a full vowel, ties to the left), so
  `yhteis|kunta`, `lämpö|tila`, `käsi|kirja`, `ammatti|yhdistys`,
  `varain|ministeriö`, but no split in `olympia`, `fysiikka`, `marttyyri`;
  (d) a list of ~170 frequent first elements audited against the 20k most
  frequent words (`maailman`, `yhteis`, `epä`, `yli`, `ulko`, `työ`,
  `talous`, ...). The remainder after a listed stem must look like a word
  (four letters, a vowel, legal onset) and not like a suffix (`-inen`,
  `-isesti`, `-lainen`, `-mainen`, `-stää`); one-syllable stems (`pää maa
  jää puu kuu tie yö`) only split before a known second element.
- **Function words**: `ja on ei se hän tai en et ne he vai me te` carry no
  stress; pronouns `minä sinä` and the forms of `olla`/`lienee` only `ˌ`,
  as in espeak's dictionary.
- **Sentence pass (`join`)**: boundary gemination (*loppukahdennus*) in
  espeak's own notation — the next word's initial stop gets `ː`
  (`tˈule tːˈænne`), another consonant is appended to the previous word
  (`ˌoleh hˈyvæ`, `kˈolmem mˈiestæ`). Triggers: nominative `-e` nominals
  (kone, perhe, lause; not pronouns or `-mme/-tte` forms), allative `-lle`,
  adverbs in `-sti` (3+ syllables, not posti/viesti) and `-nne`, prolative
  `-tse`, `kolme`, `itse`, and ~150 common imperatives / first infinitives
  (`anna`, `tule`, `mennä`, `syödä`, `ostaa`). espeak only knows `-lle`,
  `-nne`, `-sti`, `-tse` and a 60-word list. Word-final `n` -> `ŋ`/`m`
  before k/p (`mˈennæːŋ kˈotɪːn`), as espeak. `-lle` + enclitic geminates
  the enclitic (`kˈenellˌekːæːn`, `sˈillekːɪn`).
- **Numbers**: Finnish cardinals as one orthographic word below a million
  (`kaksisataakolmekymmentäviisi`, `tuhatyhdeksänsataayhdeksänkymmentäviisi`),
  `miljoona/miljoonaa`, `miljardi/miljardia`, `biljoona`, `triljoona`;
  decimal `pilkku`. The numeral parser in `word` gives every morpheme its
  own stress. Symbols: % prosenttia, $ dollaria, € euroa, £ puntaa, & ja,
  + plus, @ ät, ° astetta, = on yhtä kuin.
- **Letters, abbreviations, loans**: letter names (`aː beː seː ... kaksoisveː
  æks yː tset`) for spelling, vowel-less tokens and single letters; ~25
  lower-case initialisms spelled (`oy`, `atk`, `tv`); ~30 abbreviations
  expanded (klo, esim., jne., ym., ns., tms., vrt., kpl, puh., nro, hra,
  rva, tri, prof., ap., ip.); ~110 English/French/German words and names
  respelled (facebook -> feisbuk, show -> šou, design -> disain,
  washington -> vašinkton, peugeot -> pöžoo).

## Where espeak is wrong and we differ on purpose

- espeak's secondary stress ignores compound structure: `ˈylɪopˌisto`
  (we: `ˈylɪˌopɪsto`), `pˈæːmɪnˌisterɪ` (we: `pˈæːmˌinɪstˌerɪ`),
  `ˈaustralɪan` (we: `ˈaustralɪan`, espeak lengthens the a); where its
  weight rule happens to hit the seam (`lˈentokˌone`, `ˈepæˌonnɪstˌumɪnen`)
  the two agree; number words get espeak's weight pattern
  (`kˈaksɪsatˌaːkolmekˌymmentævˌiːsɪ`) and `25` is read `kˌaksikymmentævˈiːsi`.
- espeak reads `ch` as `kh` (`khˈikaɡo`), `q` as a literal `q`, English words
  letter by letter (`fˈaseboːk`, `ʃˈov`, `dˈesɪɡn`), doubles `m` in
  `sydämen`, and reads `ruoan` as `ruːan`.
- espeak's boundary gemination covers only four suffixes and a short list;
  `kolme miestä`, `anna kissalle`, `ole paikalla` do get it from us.
- Numbers with spaces (`2 300 000`) are tokenised as three numbers by the
  shared tokenizer, not by this module.

## Known gaps

- Compound detection is heuristic: a harmony violation is sufficient but not
  necessary, so all-back or all-front compounds not in the stem list keep
  espeak's weight pattern (`kirjasto`, `vuosikymmen` is caught by `vuosi`,
  `pankkitili` is not). False splits are rare but possible after short
  stems (`maa|ilma` is right, `työ|läinen` is blocked by the suffix list).
- `-e` nominals vs other `-e` words for gemination are decided by
  orthography only; foreign names in `-e` not in `NO_DOUBLE` trigger it.
  First infinitives in `-a/-ä` are only recognised from a list.
- `ie uo yö` are diphthongs in every syllable; `iu/eu/ou/ey/iy/äy/öy` in
  non-initial syllables are hiatus for some speakers.
- Wiktionary is inconsistent about `-lle` + enclitic gemination
  (`sillekin` with, `jollekin` without); we always geminate.
- English words not in the respelling table are read with Finnish letter
  values; single vowel letters are read as names (`aː`) where the gold has
  the bare sound.
- No ordinals (`12.` is read as a cardinal), no Roman numerals.
