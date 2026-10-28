# French (`fr`) results

Rules-only engine (`src/fr.rs`) with a built-in exception lexicon of ~900 entries
(irregular words, Latin/English loans, proper names seen in the frequency list).
Notation follows espeak-ng `fr-fr`: `ʁ`, nasal vowels with a combining tilde
(`ɑ̃ ɛ̃ ɔ̃ œ̃`), /ɥ/ written `y`, circumflex length in the final syllable (`aː oː yː`),
`ˈ` before the last vowel of content words, function words unstressed inside a
phrase (`ˌ` on the multi-syllable ones) and stressed only phrase-finally, liaison
consonant attached to the preceding word (`lez ɑ̃fˈɑ̃`).

## Accuracy against Wiktionary (WikiPron `fra_latn_broad.tsv`)

`POLYG2P_SRC=... .venv/bin/python tools/eval_acc.py fr` — 20 000 most frequent
words (wordfreq), frequency-weighted, both systems collapsed with
`tools/collapse_fr.py` (r variants, ɥ/y, optional e-caduc, glides, e/ɛ o/ɔ ø/œ
neutralized outside the final syllable; the harness strips the nasal tilde itself).

| system  | word accuracy | PER  |
|---------|---------------|------|
| polyg2p | **99.1 %**    | 0.6 % |
| espeak-ng fr-fr | 98.2 % | 1.8 % |

Exact-string agreement with espeak on the same 20k words is 79 %; the
systematic differences are notation/quality choices where Wiktionary sides with
us: espeak writes `o` for every non-final `o` (`momɑ̃`), `nj` for `gn`
(`ɡanje`), `ø` in `seulement`/`jeune`, drops the yod in `février`, voices `obtenir`.

### The remaining misses (427 of 20 000 word types, 0.9 % of the frequency mass)

* 245 are capitalized tokens: almost all foreign proper names (`John`, `Washington`,
  `Michael`, `Montréal` with the Québec `mɔ̃ʁeal`).
* ~130 lower-case English/foreign loans with English vowels in the gold
  (`news`, `people`, `baseball`, `skate`, `streamer`, `tutti`, `führer`...).
  The most frequent ones (`week-end`, `football`, `leader`, `job`, `jazz`, `cookie`,
  `smartphone`, ...) are in the lexicon.
* ~40 are gold noise or ambiguity: a 3-pl verb listed only as a noun or the
  reverse (`dirigent`, `livrent`, `citent`, `serrent`, `provient` → `pʁɔvjɑ̃` in gold,
  `convergent`), inconsistent `o/ɔ` before `m` (`atome` o but `atomes` ɔ, `nœuds`
  `nœd`), written geminates (`immortel` `imm`), `-ai` as e/ɛ (`mettrai`, `chais`),
  `-isme` as `izm`, misspellings (`parceque`) and abbreviations (`bd`, `ko`, `tkt`, `nº`).
* ~12 are genuine gaps of the rules (rare words): `arguant` (ɡɥ), `royer`,
  `abbayes`, `tian`, `lesbien` (gold voices `sb`), `coïncident` (verb/adjective).

## What the rules cover

Silent final consonants (`-s -t -d -x -z -p -g -c` with the pronounced exceptions:
`-ct`, `-st`, `-pt`, `-lt`, `-ac/-ic/-oc/-uc/-ec`, `-ing`, Latin `-us/-is/-os`),
`-er` infinitives vs `fer/mer/hiver/...`, the `-ent` verb ending vs nouns/adverbs
(morphological heuristic by the stem's last letter plus two short exception lists:
`parlent` / `moment`, `aiment` / `vêtement`, `-ient` verbs of `-ier`, `vient/tient`
vs `client/patient`), e-caduc (`samedi`, `lentement`, `gouvernement`), e quality
(closed → ɛ, geminate → e, `inter-`), nasal vowels with all the blocking contexts
(`nn/mm`, `mn`, `enn-/emm-`, `-emment` → `am`, `-um` → `ɔm`, `-en` → `ɛn` after a
consonant), `c/ç/g/gu/qu/x/s/ti/ch/gn/ill/y/œ/ay/oy/uy`, glides vs vowels after
obstruent+liquid (`client` klijɑ̃, `cruel`), `eu/œu` (ø/œ), `au/eau/o/ô` (o/ɔ),
`ai/-ai/-rai`, `ien/yen/éen`, `oin/ouin/uin`, `-tion/-tie/-tiel/-tieux` vs `-stion`,
`fais-` → `fəz`, prefix boundaries (`dés-`, `trans-`, `para-`...), apostrophe
clitics (`l'homme`, `qu'il`, `jusqu'à`, `p'tit`), hyphen compounds with internal
liaison (`peut-être` pøtɛtʁ, `va-t-il`, `est-ce`), and hyphenated numbers
(`vingt-et-un`, `dix-huit` dizɥit, `quatre-vingt-dix-neuf`, `deux-cents`).

`join`: phrase-final stress for function words, sandhi of `six/dix/huit/neuf/plus/tous`
(`si mwa` / `siz ɑ̃` / `sis`; `plyz`/`ply`/`plys`; `tu le ʒuʁ`), obligatory liaison
(determiners, clitic pronouns, `est/c'est/ont/sont`, `très/plus/bien/rien`,
`quand/dont/dans/chez/sans/sous/en/on/un/mon`, prenominal adjectives, number words
incl. the last part of a hyphenated number) blocked by h-aspiré and punctuation.

`cardinal`: 1990 hyphenated spelling below a million, `million(s)`/`milliard(s)` as
separate words, `et` in 21/31/.../71, `quatre-vingts`/`cents` plural rule.
Symbols: `%` pour cent, `$` dollars, `€` euros, `£` livres, `&` et, `+` plus,
`@` arobase, `°` degrés, `=` égale. Letters: `a be se de ə ɛf ʒe aʃ i ʒi ka ɛl ɛm ɛn
o pe ky ɛʁ ɛs te y ve dubləve iks iɡʁɛk zɛd`.

## Known gaps

* No part-of-speech: homographs are read with the most frequent reading
  (`fils` fis, `est` ɛ, `as` a (verb), `plus` by position only, `portions` noun,
  `couvent` noun, `président` noun, `excellent` adjective, `ferment` verb).
* Liaison covers the obligatory cases only; optional liaisons after plural nouns,
  `pas`, `-ent` verbs, `trop`, `beaucoup` are not made (espeak makes them all,
  including wrong ones such as `vous êtes` → `ɛtz`).
* English/foreign loans not in the lexicon are read with French rules.
* Proper names: only the ones frequent enough to show up in the top-20k list.
* `aspirated h` list is short (~70 stems).
* The eval harness strips the nasal tilde for every language before the plugin
  runs, so `ɛ̃/ɛ`, `ɑ̃/a`-type errors are invisible to `eval_acc.py`; the unit tests
  and the espeak diff cover them instead.
* `RATP`-style acronyms with a vowel in second position are read as words by
  `spell::is_acronym` (`ʁatp`); that is in `spell.rs`, not in this module.

## Strict nasal scoring

With `collapse_raw` (nasal vowels kept distinct from oral ones, added after the
main evaluation): polyg2p 98.9% / PER 0.7%, espeak-ng 97.7% / PER 1.9%.
