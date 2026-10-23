# Turkish (`tr`)

Rules only (`src/tr.rs`), no lexicon or model. Notation follows espeak-ng's `tr`
voice; pronunciations follow the orthography and standard Istanbul Turkish.

## Accuracy vs Wiktionary (WikiPron `tur_latn_broad.tsv`, 12.3k entries)

`tools/eval_acc.py tr` with `tools/collapse_tr.py` (both outputs and the gold
collapsed to a broad phone set: lax/tense vowel allophones, dark l, tap/trill,
palatal k/g, aspiration, length and the several spellings of ğ are folded;
ö/ü share one broad symbol because espeak writes lax ü as `ø`).

| set | polyg2p word acc | PER | espeak word acc | PER |
|---|---|---|---|---|
| most frequent words found in the gold (6 400 of the top 60k, frequency-weighted) | 99.5% | 0.3% | 99.7% | 0.3% |
| whole gold lexicon (11 582 words, uniform) | 98.7% | 0.3% | 98.7% | 0.3% |

Turkish spelling is phonemic, so both systems sit at the ceiling of this gold.
The *only* frequent words espeak gets and we don't are `I`, `Irak`, `Ilgaz`,
`Ilgın`, `Işıktan`: capital dotless `I` reaches `tr::word` already lower-cased
by Rust's `to_lowercase` as `i` (see "Known gaps"). The one word we get and
espeak doesn't is `Kocaeli` (espeak reads `ae` as `ɛj`).

Remaining mismatches on the full lexicon are gold artefacts and foreign words:
English loans transcribed as English (`online`, `like`, `cool`, `chat`),
gold vowel fronting (`sosyal` → `sosjel`, `taze` → `teze`), epenthesis in
clusters (`prens` → `pirens`, `trans` → `tɯrans`), final devoicing (`Bob` →
`bop`), h-dropping variants (`müthiş` → `mytiʃ`), and `-eceğim` contracted to
`-icem`.

## Notation (espeak `tr` conventions, verified on the 20k most frequent words)

* Vowels: `a ɯ e i o œ u y`; `i u o e ü` are written lax `ɪ ʊ ɔ ɛ ø` in a closed
  syllable and in the word's last syllable; `e` → `æ` before a coda `m n l`
  (`gel` → `ɟˈæl`, `her` → `hˈɛr`); `ö` is always `œ`.
* `l` → `ɫ` before a back vowel or after one in a coda; `r` → `ɾ` between
  vowels; `g` → `ɟ` next to a front vowel; `k` stays `k` (espeak writes `c`
  only in its `-ken` rule and number words).
* `ğ`: lengthens a preceding back vowel (`dağ` → `dˈaː`), merges `ağa oğo uğu`
  into one long vowel (`ağaç` → `ˈaːʧ`), is `j` after `e`/`i` (`değil` →
  `dˈejɪl`), `ɯɯ` after `ı` (`çığ` → `ʧˈɯɯ`, as espeak prints a long `ɯ`).
* Doubled stops are one phone plus length (`etti` → `etːˈɪ`); `ll nn ss rr` stay
  double, as in espeak.
* Circumflex vowels are long and never laxed (`hikâye` → `hikaːjˈɛ`, `millî` →
  `mɪllˈiː`); espeak drops the length — kept deliberately.
* Stress: `ˈ` right before the stressed vowel (`kitˈap`, `ˈankaɾa`). Final by
  default; pre-stressing suffixes move it left: `-(i)yor` (`ɟelˈijɔr`), negative
  `-me/-ma` with its verbal continuations (`ɟˈælmedɪ`, `ˈɔɫmadan`, `jˈapmamak`),
  `-ken`, instrumental `-(y)le`, copular `-dir`, copular/aorist person endings
  after a vowel (`ijˈijɪm`, `ɟelˈɪrsinɪz`). Clitics `ve ki de da mi …` carry no
  mark, `ile veya yoksa` a secondary one. ~190 words with lexical non-final
  stress (place names by Sezer stress, adverbs, question words, kinship terms,
  loans) are listed in `STRESS`; they keep their stress before case suffixes
  and after an apostrophe (`Ankara'da` → `ˈankaɾada`).
* Agreement with espeak on the 20k most frequent words after collapsing the
  affricate ligatures: ~69% exact; nearly all remaining differences are stress
  placement (espeak treats every final `-ma -me -la -le -ra -re -ce` as an
  unstressed suffix, giving `sɔnrˈa` but `ˈœjle`, `ɟˈedʒɛ`, `dˈeɾedʒɛ`).
* Numbers (`cardinal`: `bin`, never `bir bin`; `bir milyon`), decimal `virgül`,
  symbols (`%` → `yüzde`, `€` → `avro`, `£` → `sterlin`, `=` → `eşittir`),
  letter names for acronyms (`TBMM` → `tˌɛbˌɛmˌɛmˈɛ`).

Speed: 20k words in 0.05 s (CLI, single core).

## Known gaps

* **Capital dotless I.** `lib.rs` lower-cases tokens with Rust's
  `str::to_lowercase`, which maps `I` → `i` (Turkish needs `ı`) and `İ` →
  `i̇` (handled: the combining dot is dropped). All-caps or capitalised
  `I…` words (`Irak`, `ISPARTA`, `Işık`) are therefore read with dotted `i`.
  Fix outside this module: lower-case Turkish tokens with `I` → `ı`, `İ` → `i`
  before calling `tr::word`.
* Stress for negative `-ma/-me` is decided by surface shape (consonant-final
  stem + verbal suffix chain), so a few nouns are pre-stressed (`hurma` and
  friends are in the lexicon; others may slip).
* Foreign names are read letter by letter with Turkish values (`new` → `nɛv`).
* Palatal `k` before front vowels is not written (espeak convention); long
  vowels in Arabic/Persian loans without a circumflex (`mecbur`, `hâkim` written
  `hakim`) are short.
