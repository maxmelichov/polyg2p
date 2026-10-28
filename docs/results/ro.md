# Romanian (`ro`)

Rules only (`src/ro.rs`), no lexicon or model. Notation follows espeak-ng's `ro`
voice; pronunciations follow the orthography and standard Romanian (DOOM), with
hand-written morphological stress rules and a short exception list.

## Accuracy vs Wiktionary (WikiPron `ron_latn_broad.tsv`, 9.3k entries, no stress)

`tools/eval_acc.py ro` with `tools/collapse_ro.py` (both outputs and the gold
collapsed to a broad phone set: diphthong spellings `e̯a/ea`, `o̯a/ɔa`, `aj/aɪ`,
`aw/aʊ/au̯`; palatalisation and the whispered final i `Cʲ Cʲʲ iʲ iː` → `i`;
glide vs hiatus for i/u (`j → i`, `w → u`; the gold is inconsistent here); tap vs
trill; `ŋ`; `ɡz/ks` for `x` and the e̯/i glides after `tʃ dʒ`, where the gold has
both spellings for the same words; espeak's `y` for â/î).

| set | polyg2p word acc | PER | espeak word acc | PER |
|---|---|---|---|---|
| most frequent words found in the gold (4 102 of the top 60k, frequency-weighted) | 98.2% | 0.3% | 97.9% | 0.5% |
| whole gold lexicon, lower-case entries (8 605 words, uniform) | 98.9% | 0.2% | 97.6% | 0.4% |

What remains on the frequent set is almost entirely outside the rules' reach:
single letters, which the gold transcribes as the sound and we as the letter name
(`s` → `es`, `x` → `iks`), and English loans (`site`, `weekend`, `mall`, `pizza`,
`laptop`, `whisky`). The gold also has two real oddities we don't follow: `aer` →
`ajer` (j-epenthesis), `oarecare`/`vigoare` with `wa` for `oa`. On the full
lexicon the same classes remain plus a few foreign spellings (`joule`, `foehn`,
`country`).

What espeak gets wrong and we don't (frequent words): `această`-type words where
it keeps unstressed vowels as written (`exemplare`), `niciun` (`nˈitʃun`), `ii`
in hiatus (`fiind` → `fiˈind` fine, but `viitor` → `viɪtˈor`), and a large
number of stress placements (see below).

## Stress (not measured by the gold; compared with espeak)

`tools/ro_cmp_espeak.py` on the 8 000 most frequent words (`tools/ro_espeak_top.py`):
70% exact agreement, 80% same segments, 74% same segments and primary stress.
The segment differences are notation choices (see below); of the stress
differences, spot checks show espeak wrong more often than us: it stresses the
article of `-ele` plurals (`numˈele`, `zilˈele`, `cuvintˈele`), the `-ică` of
nouns (`muzˈikə`, `publˈikə`), `kˈopiɪ`, `deoaretʃˈe`, `lˈukra`, `eɡzˌistə`;
we are wrong on genuinely lexical items not in the exception list (proper names,
`-er`/`-or` nouns such as `lider`, `tenis`, rarer `-ie` nouns).

Rules: penultimate for words ending in a syllabic vowel, final for words ending
in a consonant or a glide (`lukrˈaʊ`, `femˈeɪ`). Overrides, in order: the
exception list (`STRESS`, ~500 forms, stressed vowel upper-cased), article and
plural suffixes that keep the stem's stress (`-ul -ului -lui -lor -ilor -elor
-ele -ile -uri -urile -ii -iei`: `ˈɔamenˌilor`, `lˈukruɾˌile`, `muzˈeul`),
`-sprezece`, `-î`, 4th-conjugation infinitives (`vorbˈi`, list), `-ic -bil -ctor
-col` (penult), `-ică -ice -ica -cole -gine` (antepenult), `-ifica` (final),
3rd-conjugation `-ere/-erea/-eri` nouns (`kondˈutʃere`), `-em/-eți` of 2nd/3rd
conjugation verbs (`fˈatʃem`), `-ei` genitives (penult), hiatus `-ie -ia -io
-iu` (before the i: `famˈilie`, `mˈedju`; on the i after `-ăr-`, Greek suffixes
`-logie -grafie -nomie …`, and after most stops: `prostˈie`, `biolodʒˈie`),
`-ea` (verbs on the ending, articulated nouns before it), final-syllable `ă` is
unstressed (`nˈumər`, `pˈasəɾe`, `sˈɨmbətə`; `ă + final -a` is an infinitive:
`ɨnvətsˈa`), the `u` of a hiatus `au/ou` is unstressed (`kˈauzə`).

`join`: after the infinitive marker `a`, a word in `-a` or consonant + `-i`
gets final stress (`a lukrˈa`, `a vorbˈi`); otherwise `lucra` is read as the
articulated noun (`lˈukra`) and `vorbi` as the plural/2sg (`vˈorbʲʲ`).

## Notation (espeak `ro` conventions, verified on the 8k most frequent words)

* Vowels `a e i o u ə ɨ`; stress `ˈ` right before the stressed vowel (`kˈasə`),
  secondary `ˌ` on every second syllable from the word start before the primary
  and on the syllable two after it (`ˌɨntʃepˈut`, `asˈemenˌea`, `polˈitikˌə`).
* Diphthongs `ea` (`avˈea`), `ɔa` (`pˈɔate`), `aɪ eɪ uɪ əɪ ɨɪ` (`maɪ`, `lˈuɪ`,
  `kˈɨɪne`), `oj` (`dˈoj`), word-final `aʊ eʊ oʊ əʊ iʊ` (`stˈaʊ`, `nˈoʊ`,
  `tɨrzˈiʊ`), `eaw` (`merdʒˈeaw`); final `-ii` → `iɪ` (`ˈaniɪ`), `-iii` → `iji`.
  `u` between vowels is `w` (`zˈiwa`, `dˈowə`), `i` between vowels `j`.
  Before a consonant `au eu ou` are in hiatus (`ˈaur`, `nˈeutru`); `ăi ui` + C
  too (`trəˈit`, `kˌonstruˈit`), `ai oi ei âi` + C are glides (`hˈaɪnə`).
* Hiatus vs glide for `i` before a vowel after a consonant follows the gold
  (and DOOM): hiatus by default (`natsionˈal`, `itaˈlian`, `famˈilie`,
  `sotʃiˈal`), glide after labials (`vjˈatsə`, `pjˈatrə`, `fjˈer`), after `ch
  gh` (`kjˈar`) and `gi` (`dʒjV`); `ci + o/u` drops the i (`tʃobˈan`,
  `tʃudˈat`); `-iu` after a consonant is `ju` when unstressed (`mˈedju`,
  `servˈitʃju`) and `iʊ` when stressed (`tɨrzˈiʊ`, `aurˈiʊ`). espeak writes a
  glide in most of these (`nˌatsjonˈal`, `famˈilja`); this is the main
  divergence (650 of 8 000 words), neutral after collapsing.
* Final `-i` after a consonant: `ʲʲ` after a plain consonant (`ˈanʲʲ`,
  `tˈotsʲʲ`), `ʲ` after `tʃ dʒ k ɡ` (`nˈitʃʲ`, `ˈokʲ`) as espeak prints it; a
  full `i` after consonant + liquid (`mˈembri`), in monosyllables (`ʃi`) and in
  the listed infinitives.
* `ce ci ge gi` → `ʧ ʤ` (ligatured by `fold.rs`), `che chi ghe ghi` → `k ɡ`,
  `ș ț j h` → `ʃ ts ʒ h`, `x` → `ks` / `ɡz` in `ex-` + vowel, `n` → `ŋ` before
  `g`, `r` → `ɾ` between vowels (`kˈaɾe`; espeak keeps `r` in its dictionary
  stems: `fˈərə`, `nˈumərul` — we tap consistently), doubled consonant letters
  are one phone. Cedilla `ş ţ` are accepted.
* `e-` → `je-` in the forms of *a fi* and the pronouns (`jˈeste`, `jˈel`, `jˈa`,
  `jeɾˈa`); clitic groups with a hyphen are fused and unstressed (`s-a` → `sa`,
  `într-o` → `ˈɨntro`, `și-a` → `ʃja`); other hyphenated compounds are read in
  parts.
* Numbers (`cardinal`, masculine: `douăzeci și unu`, `o sută`, `două mii`,
  `douăzeci și cinci de mii`, `un milion`, `trei milioane`), decimal `virgulă`,
  symbols (`%` → `la sută`, `€` → `euro`, `=` → `egal`), letter names for
  acronyms (`CFR` → `ʧˌeˌefˈer`) and for vowel-less tokens.

Speed: 20k words in 0.04 s (CLI, single core).

## Known gaps

* Stress is lexical beyond the rules: `-a`-final infinitives vs articulated
  nouns (`lucra`), `-i`-final infinitives vs plurals (`vorbi`) are decided by
  the `a` marker in `join` only; `-ea` verbs/nouns, `-er`/`-or`/`-ol` nouns and
  proper names rely on the exception list.
* Hiatus vs glide for `ia ie io iu` inside words is decided by the preceding
  consonant class, not lexically (`diavol`, `ziar`, `societate` get a hiatus,
  as the gold wants; labial-initial `pia- fie- via-` are always glided, so
  `viabil`, `pian` come out as `vja-`, `pjan`).
* English loans are read with Romanian letter values.
* The `-ii` genitive/plural of `-ie` nouns with stressed `i` (`bucătării`) is
  only right for the suffixes in `ie_suffix_stressed` and the `-ării` ending.
