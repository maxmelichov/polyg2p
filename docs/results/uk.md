# Ukrainian (`uk`)

Rules-only engine with ending/stem stress statistics compiled in, plus an optional
CC BY-SA stress lexicon (`uk.lex`). Output follows espeak-ng's `uk` symbol
conventions; the pronunciations follow standard Ukrainian orthoepy where espeak
is wrong (palatalization, no final/pre-voiceless devoicing, в never `f`).

## Evaluation

Gold: WikiPron `ukr_cyrl_narrow.tsv` (53k entries), collapsed with
`tools/collapse_uk.py`. Word list: the 20 000 most frequent wordfreq words that
have a gold entry, frequency-weighted (the `eval_acc.py` default).

The gold file has no stress marks, but it encodes stress through unstressed-vowel
reduction (а→ɐ, е/и→e, у→ʊ). `eval_acc.py` strips `ˈ` *before* the per-language
collapse runs, so in that harness only segments are scored. `tools/uk_eval.py`
runs the same evaluation with a stress-aware collapse (our/espeak stress marks
are turned into the gold's vowel reduction, so a wrong stress is an error).

| configuration | eval_acc (segments only) | uk_eval (segments + stress) |
|---|---|---|
| espeak-ng `uk` | 79.7 % word acc, PER 6.1 % | 69.1 % word acc, PER 12.4 % |
| polyg2p rules only | **99.4 %**, PER 0.1 % | **97.9 %**, PER 1.4 % |
| polyg2p + `uk.lex` (`POLYG2P_DATA`) | 99.4 %, PER 0.1 % | **98.8 %**, PER 0.5 % |
| rules only, statistics built without the 20k eval words | 99.3 %, PER 0.1 % | 95.0 %, PER 3.0 % |

The last row is the honest generalization figure for the compiled-in tables: the
ending/stem statistics in `data/uk_*.tsv` are derived from the whole gold file
(as `it_suffix.tsv` is for Italian), so the 97.9 % row partly reflects endings
and stems the tables have seen. Unweighted, on polysyllabic eval words with
recoverable stress, the statistics alone place the stress correctly in 80.0 % of
held-out words (ending table alone 76.7 %; espeak 56.5 %, which mostly defaults
to the penult).

Speed: 20k words in 0.06 s user time (the CLI, release build).

## What is implemented (`src/uk.rs`)

Segments
- Vowels `a o u ɛ ɪ iː` (espeak writes і as `iː` in every position); no vowel
  reduction, as espeak.
- я/ю/є: `j`+vowel word-initially, after a vowel, after the apostrophe and after
  ь; after a consonant they soften it (dentals, р) or give labial/velar + `j`
  (свято `sʲβjˈat̪o`, бюро `bjurˈo`). ї is always `jiː`. і softens dentals.
- Soft consonants: `ʎ ɲ` (espeak's symbols) and `tʲ dʲ sʲ zʲ tsʲ ʣʲ rʲ`.
  Softness spreads leftwards through dental clusters (пісня, кістці, Дніпро,
  світ `sʲβˈiːt̪`).
- в is `β` everywhere; г `h`, ґ `ɡ`, щ `ʃʧ`, дз/дж affricates, т `t̪` (`d̪` when
  voiced by assimilation, as espeak writes it), ц `ts` (fold.rs makes it `ʦ`
  word-finally).
- Regressive voicing before voiced obstruents (боротьба `borodʲbˈa`, вокзал,
  якби, футбол `fud̪bˈol`); no devoicing before voiceless consonants or
  word-finally (ложка `lˈoʒka`, дід `dʲˈiːd`) except г → `x` in легко, вогко,
  нігті, кігті, дьогтю and initial з- before a voiceless obstruent.
- Sibilant clusters: зш/сш/зж/сж/зч/сч (зшити `ʃʃˈɪt̪ɪ`, розчарований
  `roʒʧ…`), шся/жся → `sʲsʲ`/`zʲsʲ`, -ться → `tsʲːa`, тч/дч/тш/дш, тс/дс/дц
  (відсотків `βiːʣsˈotʲkiːβ`), книжці/дошці/річці, стськ → ськ, стн → сн
  (шістнадцять), нтств → нств.
- Doubled letters are written twice, both soft (життя `ʒɪtʲtʲˈa`, знання
  `znaɲɲˈa`), as espeak writes `ˈanna`.
- Hyphenated compounds keep one stress per part (будь-який `bˈudʲjakˈɪj`);
  Latin-script words get a rough Cyrillic reading; abbreviations грн, кг, км,
  млн, млрд, тис, вул, обл are expanded.

Stress (vowel index from the end), first hit wins:
1. built-in table of ~900 function words, pronoun paradigms, numerals and
   frequent words with irregular/mobile stress (hand-written);
2. `uk.lex` if present (word → stress index, built from WikiPron);
3. `data/uk_stem.tsv`: longest word start (4–8 letters) all of whose gold words
   stress the same syllable counted from the start (fixed stem stress);
4. `data/uk_suffix.tsv`: longest ending (1–9 letters) with ≥2 words and ≥60 %
   agreement (stress counted from the end); `-ся/-сь` verbs fall back to the
   base verb's ending;
5. penultimate syllable.

Both tables are pruned to entries that change the prediction of the next shorter
key (2.2k endings, 5.1k stems, 150 KB of TSV). They hold aggregate counts only.
The mark goes directly before the stressed vowel (`ukrajˈiːna`), as espeak's.
Monosyllables are stressed except the proclitics в з й ж б у.

Numbers: `cardinal` gives nominative masculine forms with feminine agreement for
тисяча (одна тисяча, дві тисячі, п'ять тисяч) and the мільйон/мільярд/трильйон
plural forms. Decimal separator "кома". Symbols: % відсотків, $ доларів, € євро,
£ фунтів, ₴ гривень, & і, + плюс, @ собака, ° градусів, = дорівнює. Latin
letter names for acronyms (`LETTERS`).

## Error analysis (stress-aware, rules only)

Of the remaining 2.1 % (frequency-weighted):
- ~70 % are stress: words whose stress neither the built-in table nor the
  statistics predict (mobile-stress nouns in oblique cases, perfective verbs,
  names). The lexicon fixes most of them (98.8 %).
- the gold's extra assimilations we do not write: н before сь+к stays hard in the
  gold (українські `…nsʲki`), and the gold does not soften с before тʲрʲ
  (зустріч `zustʲrʲ…` vs our `zusʲtʲrʲ…`); both are narrow-transcription
  choices rather than errors.
- г → `x` in легко/могти (standard orthoepy, the gold keeps `ɦ`).
- gold oddities: ж transcribed `ʒɛ`, пан with `pˠɑ`, abbreviations (грн),
  a few words with a second stress variant the gold does not list.

Segmental differences from espeak that are deliberate (espeak is wrong): espeak
devoices before voiceless consonants and word-finally (все `fsˈɛ`, вовк `βˈofk`,
ложка `lˈoʃka`), voices before в (свято `zβjˈat̪o`), does not palatalize most
soft consonants (життя `ʒˈɪt̪t̪ja`, кінця `kˈiːntsja`), writes шв/хв as `ɬβ`/`ɣβ`,
and reads numbers wrongly (1000000 → "один тисяча").

## Known gaps

- Stress of out-of-table words with mobile stress (oblique cases, perfective
  verb forms, proper names): the ending/stem statistics reach ~80 %; use the
  lexicon pack for the rest.
- Homographs (за́мок/замо́к, ма́ла/мала́) take the more frequent reading; `join`
  does no context disambiguation.
- Secondary stress in compounds is not marked (espeak marks some, e.g. її
  `jˌiːjˈiː`).
- Numbers are read in the nominative only (years, dates, ordinals and case
  agreement after prepositions are not inflected).
- Semi-palatalization of labials/velars before і is not written (espeak does
  not write it either).

## Files

- `src/uk.rs` — the engine, unit tests (56 words).
- `data/uk_suffix.tsv`, `data/uk_stem.tsv` — aggregate stress statistics
  (`tools/uk_stress.py`, Apache-safe).
- `tools/uk_stress.py` — derives `uk_lex.tsv` (for the CC BY-SA pack:
  `build_lex uk_lex.tsv uk.lex`) and the two tables; `--eval WORDS` holds words
  out and reports the statistics' accuracy.
- `tools/collapse_uk.py` — collapse for `eval_acc.py`; `tools/uk_eval.py` — the
  stress-aware run.

The lexicon is read from `$POLYG2P_DATA/uk.lex` inside the module (a `OnceLock`
in `uk.rs`); `G2p::new(data_dir)` / `g2p --data` do not reach it until `lib.rs`
gains a `uk` resource slot like `it.lex`.
