# Swedish (`sv`)

Rules plus a built-in table of ~1 400 frequent irregular words, loans and compound parts
(`src/sv.rs`, no model). Notation follows espeak-ng's `sv` voice so TTS
models trained on espeak phonemes can consume the output; pronunciations
follow Central Standard Swedish as transcribed by Wiktionary, and beat
espeak where the two disagree. An optional lexicon `sv.lex` (WikiPron,
CC BY-SA, built by `tools/sv_lex.py`) is loaded lazily from `$POLYG2P_DATA`.

## Accuracy vs Wiktionary (WikiPron `swe_latn_broad.tsv`, 5 856 entries)

`tools/eval_acc.py sv` with `tools/collapse_sv.py` (strips tone accents
`¹ ²`, consonant length, doubled consonants and narrow detail: ə/e/ɛ, æ/ɛ,
ʏ/y, ʉː/ʉ, œː/øː, retroflex spelling ʈ ɖ ɳ ɭ ʂ vs r + t d n l s, ɧ/ʃ/sx,
ŋk/nk; keeps vowel length and the long/short qualities, which are phonemic).

| set | polyg2p word acc | polyg2p PER | espeak word acc | espeak PER |
|---|---|---|---|---|
| gold words among the top 60k by frequency, frequency-weighted (3 442 words) | **97.9%** | **6.4%** | 95.6% | 11.7% |
| all 5 082 unique gold words, uniform weights | **70.0%** | **8.9%** | 58.5% | 12.5% |
| all gold words with the optional `sv.lex` built from the gold (sanity check only) | 98.3% | 0.4% | | |

The uniform set is dominated by proper names (Kazakstan, Katrineholm,
Blomqvist), bird-name compounds (medelhavstrut, kustsnäppa) and French/
English spellings; the gold itself is noisy there (`dig` → `t i eː`,
`Europa` → `ɛruːpa`, `värsta` → `vɛrsrta`, several entries with tone
diacritics on the vowels).

## What the module does

- **Stress**: `ˈ` right before the stressed vowel, as espeak writes it
  (`bˈɑːra`, `stasxˈuːn`); `ˌ` on the other parts of a compound
  (`ˈansvˌɑːr`, `sxˈøːnheːt`). Default initial; stressed loan suffixes with
  the stressed vowel marked in the table (`tiOn siOn iOn Era Ik Ist Ism Ell
  Ent Ant Ans Ur Yr Ör Ös tEt erI I In It Iv Age Al Är Ess Ett Att É ...`,
  `-On/-Ong/-Ina/-stAn` only in words of three or more syllables),
  antepenultimate stress before `-isk(a) -ium -ien -iker -ia`; unstressed
  prefixes `be-` / `för-` (with the stressed nouns `första, företag, fördel,
  förnamn, ...` in the table); stressed particle prefixes `an- av- in- ut-
  upp- om- på- till- över- under- efter- genom- o- ...` with secondary
  stress on the rest and a stop list (`andra, annan, inte, under, utan,
  ...`); compound tails (`-het -dom -skap -bar -lös -full -sam -dag -tid
  -hus -land -stad -man -bok ...` and place-name tails `-ås -sjö -ö -by
  -hult -borg -berg -holm -torp -vik -sund -näs -ström -kvist -gren`) keep
  their own length under secondary stress. Loans in a + C + V with three or
  more syllables (abonnemang, akademi) are stressed after the first syllable.
- **Vowel length and quality**: stressed vowel long before at most one
  consonant (`tˈɑːla`, `bˈuːk`), short before clusters, doubles, `ŋ`, `j`,
  `x`; long before `rd rn rl` (`bˈuːrd`, `bˈɑːrn`), short before `rt rs`
  except after å/ö/ä/u (`hˈoːrt`, `fˈœʂt` vs `lˈɛːrt`); final single `m`
  short (`rɵm`, `hɛm`), final `n` long except a list (`man kan han hon min
  din sin den en in vän ...`); inflected forms keep the stem's length
  (`sjukt`, `djupt`, `grovt`, `högt`); stop + liquid/nasal/j and `sn`
  clusters from syncopated stems keep it too (`jˈɛːvla`, `fˈoːɡlar`,
  `vˈɑːkna`, `trˈeːdjə`, `vˈiːsnɪŋ`). Long `ɑː eː iː uː oː yː ɛː øː`, long u
  `ʉ` (espeak writes no length mark), short `a ɛ ɪ ɔ ɵ y ɛ œ`; unstressed e
  `ə` after the stress, `ɛ` before it; unstressed o `ʊ` in open syllables and
  in the plural `-or`, `ɔ` otherwise; unstressed u `ɵ`; final `-ö -å -y`
  long even unstressed (Värmdö, Alingsås, Ryssby). `ʊ`/`oː` words that
  escape the rules are in the table (`hon, ost, bott, son, sova, kol, tom`).
- **Consonants**: sj/skj/stj/sk+front vowel/-tion/-sion/-ssion → `sx`
  (espeak's spelling of ɧ), sch/sh/French ge → `ʃ`, tj/kj/k+front → `ɕ`
  (loans `kille kex keps kilo ...` stay k), g+front → `j` (`ge gärna göra`;
  `gel gen genetik ...` stay ɡ, `genre gelé geni gest` → ʃ), stem-initial
  only, so `mage` and `fågel` keep ɡ; dj/gj/hj/lj → `j`; ng → `ŋ`, nk → `ŋk`,
  gn → `ŋn` (lugn, vagn, regn), loan `ng` + vowel → `ŋɡ` (tango, fungera);
  rs → `ʂ` (also across prefixes: `fœʂtˈoː`), rt rd rn rl kept as r + C;
  rg/lg → `rj lj` finally and before e/s (berg, berget, helg, älg; `-rgi`
  loans excepted); b d g → p t k before t and final s (snabbt, lagt, gods);
  x → ks, z → s, c → s before front vowels, cc → ks, w → v, q(u) → k(v),
  tt/ck between vowels → `tː kː`, other doubles single; au → aʊ, eu → ɛʉ,
  ei → ɛj, oo → uː.
- **Lexicon mechanics**: `FUNC` (unstressed function words: `ɔk at deː dɔm
  ɛːr jɑːɡ ...`), `LEX` (irregular and frequent loan words; later entries
  override), lexicon word + native ending (`hotell-et`, `bil-arna`,
  `person-lig`, `-ning -het -skap ...` with the ending's phonemes), and a
  two-part compound splitter over the lexicon (with an optional linking
  `-s-`): `med+arbetare`, `arbets+plats`, `jätte+bra`, `brev+bärare` (the
  last only with `sv.lex`). With `POLYG2P_DATA` set, `sv.lex` (5 053
  WikiPron words converted to espeak notation) is consulted after the
  built-in tables and as a source of compound parts.
- **Numbers**: `noll ett två ... tjugo`, `tjugoett ... nittionio` as one
  word (`ɕˈʉɡʊˌɛt`), `hundra`, `två hundra`, `tusen`, `en miljon`, `två
  miljarder`; decimal separator `komma`. Symbols: % procent, $ dollar,
  € euro, £ pund, & och, + plus, @ snabela, ° grader, = lika med.
- **Letters**: `ɑː beː seː deː eː ɛf ɡeː hoː iː jiː koː ɛl ɛm ɛn uː peː kʉ
  ɛr ɛs teː ʉ veː dɵbəlveː ɛks yː sɛːta` (SVT → `ˌɛsvˌeːtˈeː`).
- **Sentence pass**: none (Swedish has no sandhi espeak models; the
  retroflexion across words, `för sent` → `fœ ˈʂeːnt`, is not applied).

## Where espeak is wrong and we differ on purpose

- espeak drops the r of `rt` (`kort` → `kˈɔt`, `snart` → `snˈɑːt`, `hjärta`
  → `jˈɛta`), reads `de`/`dem` as `deː`/`dɛm` instead of `dɔm`, and `igen`
  as `ˈiːjən`.
- espeak softens g inside words (`fågel` → `fˈoːjəl`, `regn` → `rəŋn`),
  misses the softening in `gift` only by luck, and reads `ange` right but
  `inköp` as `ˈɪŋɕøːp`.
- espeak's stress is often wrong in loans, which also wrecks vowel length:
  `kamera` → `kˈameːrˌa`, `politiker` → `pˈuːliːtˌiːkər`, `natur` →
  `nˈatˈʉr`, `studera` → `stˈʉdeːrˌa`, `förstå` → `fˈœʂtoː`, `universitet`
  → `ˈɵniːvˌɛʂɪtˌeːt`, `antal` → `antˈɑːl`.
- espeak writes long vowels in unstressed open syllables (`radio` →
  `rˈɑːdiːˌuː`, `union` → `ˈɵniːˌuːn`, `skrivit` → `skrˈiːviːt`) and
  short ones in compound parts (`tillbaka` → `tˈɪlbɑːkˌa` is fine, but
  `människor` → `mˈɛnɪsxˌuːr`).
- espeak has no `oo`/`ei`/`cc` handling (`cool`, `Leif`, `acceptera`) and
  reads `rum` as `rˈɵːm`.

## Known gaps

- Compounds without a lexicon part: the second element's long vowel is
  missed when neither part is in the built-in table (`Elfenbenskusten`,
  `alpseglare`); with `sv.lex` many more split correctly. A frequency list
  of compound heads (`data/sv_*.tsv`) would be the next step.
- Short `o`: ʊ vs ɔ (`bott` vs `kom`), and long `o`: uː vs oː (`bok` vs
  `son`) are lexical; the rules default to ɔ/uː with a word list.
- Pretonic vowel length in loans with non-initial stress that no suffix
  rule covers (`semester`, `recept`, `metod`, `teater` are in the table;
  `agat`, `delikat` get initial stress). Final `-i` is always stressed
  (`kemi`, `parti`) which is wrong for `taxi`-type words outside the table.
- `-tion` before a vowel (`internationell`) and `-tial/-tiell` (`potential`,
  `speciell`) are table-driven; `-ie`/`-ia` names (`Jennie`, `Maria`) too.
- English and French spellings outside the table are read with Swedish
  letter values (`case`, `clown`, `croissant`).
- Pitch accent (acute/grave) is not marked — espeak does not mark it either.
- No ordinals, no year reading, no cross-word retroflexion in `join`.
- `eval_acc.py --data` passes the directory to the CLI, but this module
  reads `sv.lex` from the `POLYG2P_DATA` environment variable (the module
  contract has no data-dir parameter for rule-based languages).
