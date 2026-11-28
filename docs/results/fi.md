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

Misses on the lists: 99 -> 96 (top list), 1,210 -> 1,167 (all 168,814 words)
after the letter rules below. Gains: `ng`+r is a short `ŋ` (kongressi,
gangreeni, 20 of 20 gold words), `th`/`gh`/`ph` in loans, `WC` read with
`veː`. Losses: `ph` is always `f`, which the gold does not do in four rare
words (hiphop, apheli, Ghana, ghanalainen).

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
  (`ˌoleh hˈyvæ`). Which word triggers it depends on context, following the
  Wiktionary appendix on Finnish pronunciation (final gemination: nominals in
  `-e`, allative, first infinitives, connegatives, 2nd-person imperatives,
  adverbs in `-itse/-nne/-sti/-lle`):
  - always: `-lle`, `-tse`, `-nne` (not the possessive: *talossanne*), `-e`
    nominals (kone, perhe, lause; not the diphthong `-ie`, not loans with a
    foreign letter near the end, not nukke/saame compounds), adverbs in
    `-sti` by shape (`sti_adverb`: nopeasti, yleisesti, normaalisti; not
    turisti, eversti, past tenses varasti, pelasti), `-da/-dä` after a long
    vowel, diphthong or `h` and `-ta/-tä` after consonant + short vowel in
    three or more syllables (saada, tehdä, haluta), ~45 unambiguous
    imperatives and ~360 first infinitives that cannot be a finite verb
    (`src/fi_verbs.rs`);
  - only as an infinitive (`Trigger::Infinitive`): the 606 verbs whose
    infinitive is spelled like the 3rd person singular (antaa, pitää,
    tietää, auttaa, puhua, ...). They geminate only when a governing verb
    (voida, saada, täytyä, pitää, haluta, osata, aikoa, yrittää ...) stands
    earlier in the clause, at most six words back, with no pause or clause
    conjunction between: `Voisitko antaa hänelle` geminates, `Hän antaa
    meille` does not;
  - only as an imperative or connegative (`Trigger::Imperative`): käy, juo,
    syö, vie, luo, tee, muista, avaa, saa, jää, voi, osaa ...: after a negative
    verb within four words (`en voi tehdä`, `ei saa tulla`, `Se ei käy
    päinsä`), clause-initially, or after `ja/tai/sekä` when the clause opens
    with such an imperative; not otherwise (`Tomi käy kirkossa`).
  `kolme`, `itse` (alone) and `nukke` do not geminate (VISK § 34 — cited from
  the planning notes, the page returned 403 and was **not read**; unverified).
  The `-e`, `-sti` and `-da/-ta` rules are the part that would need a real
  morphological analysis: they are decided by spelling. The rule tables are
  generated from the gold by `tools/fi_boundary.py build` (words whose every
  gold entry carries `ˣ`, wordfreq Zipf >= 3.0) and measured with
  `tools/fi_boundary.py eval`.
  Word-final `n` -> `ŋ`/`m` before k/p (`mˈennæːŋ kˈotɪːn`), as espeak. `-lle`
  + enclitic geminates the enclitic (`kˈenellˌekːæːn`, `sˈillekːɪn`);
  imperative + enclitic inside a word (`otapa` -> `ˈotapːa`, `tulehan` ->
  `tˈulehhan`).
- **Numbers and normalisation** (`prenormalize`, `expand_token`):
  - cardinals as one orthographic word below a million
    (`kaksisataakolmekymmentäviisi`), `miljoona/miljoonaa`, `miljardi`,
    `biljoona`, `triljoona`; decimal `pilkku`; beyond a language's scale words
    digit by digit instead of a panic (shared `num.rs`);
  - a space as the thousands separator (`2 300 000`, `100 000 euroa`) is one
    number, also after `, ` `. ` `: ` when a word precedes (`auton, 20 000
    euroa`, `Hinta: 1 000 euroa`, `tuli. 2 000 ihmistä`); not when a leading
    zero or more digits follow (phone numbers), after another number and a
    space (`1 2 000`), or in a short list (`3, 2 000`: a digit run directly
    before the comma or colon, or a 1-2 digit run before the period, keeps the
    old digit-by-digit reading); minus sign (`-5` -> miinus);
  - inflected numerals written with a colon: `5:ssä` -> viidessä, `25:n`,
    `3:lla`, `300:aa`, `7:ään`, `2:ksi`, ordinals `5:s`, `3:nnessa`;
  - ordinals before a lower-case word: `5. luokka`, `12. joulukuuta`,
    `5. kerroksessa` (agrees in -ssa/-sta/-lla/-lta/-lle/-ksi, and päivänä,
    kertana, vuonna, and a genitive noun in -an/-än/-en/-in/-on/-ön/-un/-yn:
    `3. luokan` -> kolmannen luokan, `1.–3. luokan` -> ensimmäisen–kolmannen
    luokan; illatives, pronouns and adverbs in -n stay nominative); `Olen 25.` and `3.5.` stay cardinal / decimal;
  - dates `24.12.2023`, `24.12.` (day ordinal, month partitive, year
    cardinal), clock times after `klo`/`kello` (also `Kello on 9.05` and the
    second time of a range, `klo 9.05–10.15`) and `14:30` with a colon
    (`14.05` -> neljätoista nolla viisi); a bare `9.05` without a clock word is
    a decimal; a score `3:2` is read `kolme kaksi`;
  - `25 %:n` -> prosentin, `20 m2`/`5 m3` -> neliömetriä/kuutiometriä
    (also km, cm, mm);
  - units after a number (`5 kg`, `1 km`, `50 km/h`, `25 °C`, `2 h 30 min`)
    and abbreviations with a period (`n. 5 km`, `mm.`, `esim.`, `v. 1995`);
  - abbreviations with a case ending: `EU:n`, `USA:ssa`, `PS:n`, `NATO:n`.
  Rules follow the Kielitoimiston ohjepankki conventions for colon
  inflection, dates and clock times; those pages were **not fetched** while
  writing this, so treat the details (e.g. the `nolla` before minutes below
  ten) as unverified against them.
- **Letters, abbreviations, loans**: letter names (`aː beː seː ... kaksoisveː
  æks yː tset`) for spelling, vowel-less tokens and single letters; ~25
  lower-case initialisms spelled (`oy`, `atk`, `tv`); `W` inside a spelled
  abbreviation is `veː` (`WC`, `BMW`, `www`); ~30 abbreviations expanded
  (klo, esim., jne., ym., ns., tms., vrt., kpl, puh., nro, hra, rva, tri,
  prof., ap., ip.); ~110 English/French/German words and names respelled
  (facebook -> feisbuk, show -> šou, design -> disain, washington ->
  vašinkton, peugeot -> pöžoo), and loans of five or more letters with a case
  ending (`Chicagossa`, `Facebookissa`, `Thaimaassa`).
- **Letter folding**: text is NFC-composed first (`a` + combining diaeresis is
  `ä`); a macron is a long vowel (`Tōkyō`); `ñ` -> nj, `č ć` -> tš, `ő ű`,
  `þ ð ł ı`; any other diacritic falls back to its base letter; `ph` -> f,
  word-initial `th`/`gh` before a vowel (and `thr`) and word-final `th` ->
  t/ɡ, initial `chr`/`chl` -> k; `ng` before r is a short `ŋ` (kongressi,
  sangria).

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
- espeak's boundary gemination covers only four suffixes and a short list
  and ignores context; `anna kissalle` and `ei ole paikalla` get it from us,
  `kolme miestä` does not (kolme has no final gemination).
- espeak writes no gemination after connegatives and finite verbs it cannot
  tell from the infinitive; we geminate `en voi tehdä` and not `Tom pitää
  siitä`.

## Sentence-level measurements

Tatoeba Finnish, 152,906 sentences (`tools/sentence_cmp.py fi`), before ->
after: words that differ from espeak 2.29% -> 3.42%; of those, WikiPron
sides with polyg2p on 1,150 -> 1,214, with espeak on 9,270 -> 13,716,
cannot decide 6,460 -> 10,342. The increase is notation, not error: the gold
holds isolated words, so every sandhi we write (and espeak does not) counts as
"espeak right". Word-level (lexical) differences where espeak is right
dropped 20 -> 8 (before: kellekään 10, kongressi 6, jollekin 3, millekään 1).

Gemination hand-labelled (150 random events per build from the Tatoeba
sample, right = a real boundary gemination in that sentence under the rules
above): origin/main 141 of 150 right (94.0%; the nine wrong are finite verbs
spelled like the infinitive or a past tense: *laulaa loilotti*, *Mies juo
vettä*, *Tom pitää siitä*, *varasti*, *pyydysti*); this branch 150 of 150.
`Trigger::Infinitive`: 99 of 100 right (*tuo kirjailija yrittää sanoa*: finite,
governor six words back). `Trigger::Imperative`: 99 of 100 right (*ei saa lupaa
tehdä*: `lupaa` is the noun). Against the gold `ˣ` mark
(`tools/fi_boundary.py eval`, frequency-weighted): precision 98.2% -> 99.8%
(never-`ˣ` words we geminate after: 1,186 -> 133 types), recall on always-`ˣ`
words 64.4% -> 98.1% in infinitive context (67.7% in finite context, by
design).

Changed output on all 155,032 Tatoeba sentences, origin/main -> this branch:
13,608 sentences (8.8%) and 13,847 of 754,609 aligned words (1.83%) differ.
Gemination added 10,665 words (one event changes two neighbouring words; about
1,700 events are connegative `ei saa / en voi / ei jää / en osaa`, which did not
geminate before), gemination removed 2,934, numbers and abbreviations 670
(567 sentences change token count), letters and notation 145.

## Known gaps

- Compound detection is heuristic: a harmony violation is sufficient but not
  necessary, so all-back or all-front compounds not in the stem list keep
  espeak's weight pattern (`kirjasto`, `vuosikymmen` is caught by `vuosi`,
  `pankkitili` is not). False splits are rare but possible after short
  stems (`maa|ilma` is right, `työ|läinen` is blocked by the suffix list).
- `-e` nominals vs other `-e` words for gemination are decided by
  orthography only; foreign names and loans in `-e` not in `NO_DOUBLE`
  (house, anime, sote) still trigger it. Infinitives are recognised from the
  generated lists plus the `-da/-ta` shape rule.
- Sentence-initial `Anna` is the imperative, never the name (5 of ~340 in
  Tatoeba are the name). `lupaa`, `tapaa` after a negation can be the noun.
  `tulee`, `toivoa` are not governors; after `rakastaa` an infinitive geminates.
- Possessive `-nsA` gemination varies by speaker: only yleensä, tahansa,
  yhteensä, sinänsä, hyvänsä, ympäriinsä are written.
- Length is written as espeak does (`kː pː tː` for stops, doubled letters for
  other consonants); not unified to `ː`.
- `ie uo yö` are diphthongs in every syllable; `iu/eu/ou/ey/iy/äy/öy` in
  non-initial syllables are hiatus for some speakers.
- Wiktionary is inconsistent about `-lle` + enclitic gemination
  (`sillekin` with, `jollekin` without); we always geminate.
- English words not in the respelling table are read with Finnish letter
  values; single vowel letters are read as names (`aː`) where the gold has
  the bare sound.
- Numbers: no Roman numerals, fractions or phone numbers; `1 000 000:n` (a
  spaced number with a colon ending) is not joined; `1 %` reads prosenttia;
  ordinal agreement only for -ssa/-sta/-lla/-lta/-lle/-ksi and
  päivänä/kertana/vuonna; ordinals only below 1,000, case numerals only below
  10^9.
