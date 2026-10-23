# Polish (`pl`)

Rules only (`src/pl.rs`), no lexicon or model. Evaluated against WikiPron
`pol_latn_broad.tsv` on the 20 000 most frequent words (wordfreq), both outputs
collapsed with `tools/collapse_pl.py` so only pronunciation differences count
(stress, nasal-vowel notation, ʂ/ʃ, ɘ/ɨ, ʲ/j, geminates are normalized).

```
POLYG2P_SRC=sources .venv/bin/python tools/eval_acc.py pl
[pl] top 20000 frequent words (frequency-weighted)
  polyg2p : word acc 99.9%   PER 0.2%
  espeak  : word acc 91.5%   PER 0.5%

tools/pl_err.py 20000 (unweighted)
  polyg2p : 19935/20000 = 99.7%
  espeak  : 19546/20000 = 97.7%
```

Notation agreement with espeak-ng (`pl` voice) on the 5 000 most frequent
words: 94.1% identical strings (after the ligature fold `lib.rs` applies),
94.4% ignoring stress marks. Throughput: ~650 k words/s single core.

## What espeak gets wrong (and we don't)

Sorted by frequency-weighted impact:

* `dz` is always read as the soft `dʑ`: *bardzo, między, widzę, pieniądze,
  dzwon, rodzaj* (268 of the 20k words). We write `dz`; `dzi`/`dź` stay `dʑ`.
* The prepositions *w* and *z* come out as `vɨ`, `zɨ`; we output `v`/`z` and
  let the sentence pass set their voicing (*w szkole* → `f ʃkˈɔlɛ`,
  *w domu* → `v dˈɔmu`, *z tobą* → `s tˈɔbɔ̃`).
* Word-final `ł` after a consonant is dropped: *znalazł, zmarł, poszedł*.
* `w` is dropped in *pierwszy* and its forms.
* `si`/`zi` before a vowel is sometimes hard: *musimy, prosimy, wyrazić,
  guzik*.
* `-rcia` is read as `-rsija` (*wsparcia, otwarcia*).
* `ch` before `i` + vowel stays `h` (*hierarchia* → `rarhja`).
* Stress: *muzyka* (penult), *czterysta* / *siedemset* ok, but *uniwersytet*,
  *prezydent*, *byłoby*, *gdybyśmy* penult; we apply the antepenult rules
  (-yka/-ika stems, -liśmy/-liście, conditional -by-, a short loanword list).

## Rules implemented

* Digraphs `sz cz rz dz dź dż ch`; `trz`/`drz` written `tʃ`/`dʒ` as espeak does.
* Soft series: `ś ź ć ń dź` and `si zi ci ni dzi` + vowel → `ɕ ʑ tɕ ɲʲ dʑ`;
  before a consonant `si` → `ɕi` etc.
* Palatalization before `i` + vowel: `pʲ bj vʲ fj mʲ/mj kʲ ɡʲ` (espeak's
  mix), `tj dj rj lj xj` elsewhere; `-ii` → `ji`; `gi` → `ɡʲi`, `ki` → `ki`;
  `ch`/`h` + `i` + consonant → `ç`.
* Nasal vowels: `ɛm/ɔm` before p b, `ɛn/ɔn` before t d ts dz tʃ dʒ, `ɛɲ/ɔɲ`
  before tɕ dʑ ɕ ʑ, `ɛŋ/ɔŋ` before k ɡ, oral before l ł, `ɛ̃/ɔ̃` before
  fricatives, word-final `ę` → `ɛ`, `ą` → `ɔ̃`.
* `n` → `ŋ` before k/g and → `ɲ` before tɕ/dʑ.
* Voicing: final devoicing (not for vowelless *w*, *z*); progressive devoicing
  of `w`/`rz` after a voiceless obstruent, also across r/l (*twój, trwać,
  krwi, przez*); regressive assimilation in obstruent clusters (*wszystko,
  jakże, prośba, liczba, wschód*); `x` → `ɣ` before voiced (*Bohdan*).
  `marz`+n/ł, *zamarzać*, *Tarzan*, *erzac* keep `rz`; `rzi` → `rʑ`.
* Diphthongs `au`/`eu` → `aw`/`ɛw` (*auto, Europa*), except *na-u-*, *za-u-*
  (*nauka, zauważyć*), `ieu` and Latin `-eum/-eus(z)` (*muzeum*).
* Stress: penultimate; `ˈ` directly before the stressed vowel (espeak's
  placement); secondary `ˌ` on alternate syllables as espeak writes them.
  Antepenult for -yka/-ika stems (and -yce/-ice), 1st/2nd plural past
  (-liśmy, -łyście), conditional clitics (-bym/-byś/-by/-byśmy/-byście after
  a past form), *czterysta, siedemset, osiemset, dziewięćset, uniwersytet,
  prezydent, opera, okolica, reguła, ryzyko, nauka, maksimum, minimum*.
  Unstressed clitics as in espeak: *w i nie się to z a od dla czy mi mnie ich
  jej mu im we wam ni*.
* Sentence pass (`join`): a word-final obstruent voices before a word starting
  with a voiced obstruent (other than w/rz), like espeak (*jak dobrze* →
  `jˈaɡ dˈɔbʒɛ`, *osiemset dziewięćset* → `…sɛd dʑ…`).
* Numbers: full cardinals with the paucal/genitive plural of *tysiąc, milion,
  miliard, bilion* (2 tysiące, 5 tysięcy, 22 tysiące, 12 tysięcy); decimal
  *przecinek*; symbols % *procent*, $ *dolarów*, € *euro*, £ *funtów*,
  & *i*, + *plus*, @ *małpa*, ° *stopni*, = *równa się*; letter names for
  acronym spelling; a lone consonant letter (*r., s.*) is read by its name.
* A small built-in respelling table for ~200 frequent English loans and
  abbreviations (*weekend, show, online, jazz, nr, dr, zł, np, itd…*), and a
  phoneme table for loans with hard `si`/`ci` (*pepsi, singiel, sinus,
  maksimum, pizza*).

## Remaining errors (top 20k)

65 misses, essentially all English/foreign names and loans not in the
respelling table (*musical, Connecticut, Massachusetts, Missouri, curry,
café, pub*) and single-letter tokens (*y, ż, ś*). Native Polish vocabulary is
covered; the only native-word misses come from gold entries that look wrong
(*nóg* → `nɔk`, *niezwykłe* → `wɛ`).

## Known gaps

* No lexicon: unpredictable loanword pronunciation beyond the small table.
* Homographs with different stress/pronunciation are not disambiguated
  (*Dania* vs *dania* — both `dˈaɲʲa`; *muzyka* is always antepenult).
* Prefix boundaries that block softening are not detected (*zidentyfikować*
  → `ʑi…` instead of `zi…`; *nadzwyczajny* reads `dz` as the affricate,
  which the broad transcription does not distinguish anyway).
* Cross-word voicing before *w*/*rz* (Cracow-Poznań style) is not applied.
