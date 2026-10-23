# Hungarian (`hu`)

Rules only (`src/hu.rs`), no lexicon or model. Evaluated against WikiPron
`hun_latn_narrow.tsv` (the only Hungarian gold in `sources/`) on the 20 000
most frequent words (wordfreq), both outputs collapsed with
`tools/collapse_hu.py` so only pronunciation differences count (stress, ɒ/ɑ,
h/ɦ/x, nasal place allophones ŋ/ɱ/ɲ, the hiatus glide of the narrow gold,
`CC` = `Cː`, and the merged t+s affricates are normalized).

`tools/eval_acc.py` still maps `hu` to `hun_latn_broad.tsv`, which does not
exist; `tools/hu_err.py` points it at the narrow file and prints the same
numbers plus the error table:

```
POLYG2P_SRC=sources .venv/bin/python tools/hu_err.py 20000 ours
  polyg2p : word acc 99.9%   PER 0.1%   unweighted 19919/20000 = 99.6%
POLYG2P_SRC=sources .venv/bin/python tools/hu_err.py 20000 espeak
  espeak  : word acc 95.4%   PER 0.9%   unweighted 18743/20000 = 93.7%
```

Notation agreement with espeak-ng (`hu` voice) on the 6 000 most frequent
words: 67.8% identical strings, 93.0% ignoring stress marks. The gap is
espeak's word list: about a quarter of all words (*meg, csak, van, volt,
vissza, köszönöm, tudom…*) are flagged `$unstressend` in its dictionary and
come out with `ˌ` instead of `ˈ`. That list is GPL and not reproducible from
rules, so we always write the primary mark on the first syllable.
Throughput: ~500 k words/s single core (186 k words in 0.37 s through the CLI).

## What espeak gets wrong (and we don't)

Sorted by frequency-weighted impact on the 20k words (counts are words):

* No voicing assimilation except in a few listed words: *legtöbb, meghalt,
  megfelelő* (g before a voiceless consonant, 256), *években, napokban* (k
  before b, 137), *kapcsolatban, esetben* (t before b, 82), *néztem, házhoz*
  (z before t/h, 73), *városban, lásd* (s before b/d, 71), *mindkét,
  rendkívül* (46), *képzés* (44), *könyvtár* (v before t, 42), *egyébként,
  többször* (33); gy before s/sz stays voiced (*egyszer* → `ɛɟsɛr`, 104).
* *egy* and its forms get a short `ɟ` (*egy, egyik, egyet, huszonegy*);
  *arról, erről* lose the long `rr`; *szívesen* gets a short `i`.
* `lj` is written `ʎj` (*teljes, célja*); the standard pronunciation is a
  long `jː`.
* Foreign `ch` as `tsh` (*Chris, Michael, Richard*).

## Rules implemented

* Vowels: `a ɑ`, `á aː`, `e ɛ`, `é eː`, `i iː`, `o oː`, `ö ø`, `ő øː`,
  `u uː`, `ü y`, `ű yː`; no glide is inserted in hiatus (espeak: *fiú* `fiuː`).
* Consonants: `sz s zs cs c dz dzs` → `s ʃ ʒ tʃ ts dz dʒ`; `gy ty ny ly` →
  `ɟ c ɲ j`; `x` → `ks`, `w` → `v`, `qu` → `kv`, `y` → `i` (`j` word-initially
  before a vowel), `ch` → `h` (`tʃ` word-initially and at the end of names:
  *chip, Madách*), `cz` → `ts`.
* Multigraph parsing is greedy with the doubled spellings (`ssz ggy nny tty
  lly ccs zzs ddz`), and three boundary rules: `z + -ság/-ség` (*igazság,
  község*), `z + sz` (*közszolgálat, százszor*) and `c + sz` (*nyolcszor*)
  are not digraphs; `leg-`, `ország-` and `meg-` + `gy` are `ɡ + ɟ`
  (*leggyakoribb, országgyűlés, meggyőz*; the fruit *meggy* keeps `ɟː`).
* Length: doubled consonants are long; espeak's mix of notations is kept —
  `ː` on stops, affricates and palatals (`tː ɟː tsː`), doubled fricatives and
  sonorants (`ss ll ɲɲ`). `dz`/`dzs` are long between vowels (*edző, bodza,
  maharadzsa*). A long consonant next to another consonant is shortened
  (*hallgat, jobbra, mondd*); identical consonants merge (*tudtam* → `tutːɑm`,
  *kezdte* → `kɛstɛ`).
* j-assimilation: `tj dj gyj tyj nyj nj lj llj` → `cː ɟː ɟː cː ɲɲ ɲɲ jj jj`
  (*látja, tudja, hagyj, anyja, bánja, teljes, hallja*).
* Sibilant assimilation: `t/d + s, sz, c, cs` → long `tʃː`/`tsː`
  (*tanítson, szabadság, kétszer*); a sibilant fricative before another takes
  its place (*egészség* → `ʃʃ`, *igazság, község*); `l + r` → `rr`
  (*balra*), `n + m` → `mm` (*önmaga*), `n + ny` → `ɲɲ` (*negyvennyolc*).
* Regressive voicing assimilation across obstruent clusters, with `v` and
  `h` undergoing but not triggering it (*azt, népdal, rakd, szívtelen,
  hatvan, adhat*; *gy* → `c` before s: *egyszer, nagyság*).
* *egy*: long `ɟː` as a word, in numbers (*tizenegy, huszonegy*), with
  inflections and in *egyik, együtt, egyes, egyetlen, egyezik, egyértelmű,
  mindegyik, megegyezik*; short in *egyedül, egyetem, egyenlő, egyéb, egyéni,
  egyél*. *kisebb* has `ʃʃ`; *millió* a short `l`.
* Silent final `h`: *méh, cseh, juh, düh, pléh, rüh, céh, oláh* (pronounced
  in suffixed forms).
* Stress: `ˈ` right before the first vowel (`nˈɛm`, `ˈɑz`, `brˈinɡɑ`), on
  every word. Hyphenated parts are stressed separately; a case suffix after a
  hyphen (*BKV-nál, 25-től, 1848-ban*) is glued unstressed to the preceding
  number word or acronym (`join`), like espeak's `nɛɟvɛnɲoltsbˌɑn`.
* Numbers: one word up to 2000 (*ezernyolcszáznegyvennyolc*), hyphen-separated
  thousand groups above (*kétezer-huszonnégy*), *két-* in compounds and
  *kettő* standalone (*tizenkettő* / *tizenkétezer*), *ezer* without *egy*,
  *egymillió*; decimal *egész* (3,5 → *három egész öt*); symbols % *százalék*,
  $ *dollár*, € *euró*, £ *font*, & *és*, + *plusz*, @ *kukac*, ° *fok*,
  = *egyenlő*; letter names as espeak reads them (*ɛff, ɛll, ɛʃʃ, ipsilon,
  duplɑveː*), including the digraphs (*cs* → `tʃeː`, *sz* → `ɛss`, *ny* →
  `ɛɲɲ`); a consonant-only token is spelled (*bkv* → `bˌeːkˌaːvˈeː`).
* Tables: ~40 abbreviations (*db, kb, pl, stb, vmi, vki, km, kg, dkg, ft, dr,
  kft, zrt…*), ~25 historical name spellings (*Kossuth, Batthyány, Thököly,
  Dessewffy, Weöres, Eötvös, Horváth, Tóth, Németh, Rácz, Széchenyi,
  Madách…*) and ~50 English loans (*online, design, show, facebook, klub,
  pizza, rock, chip…*).

## Remaining errors (top 20k)

81 misses: vowel shortening the gold applies in compounds and derivatives
but the spelling does not show (*vízilabda* `vizi-`, *tízes, húszas,
ítéletet, színházban*); gold-only loan geminates or degeminations (*helló*,
*intelligencia*, *tinédzser*, *vákuum*, *efféle*, *klubok*); compound
boundaries where the gold does not assimilate (*hadjárat, védjegy,
tengeralattjáró, titkosszolgálat*) or does more than we do (*éhség* →
`eːʃːeːɡ`, *ajánljuk* → `ɒjaːjːuk`, *fennmaradó* → `fɛmːɒrɒdoː`,
*mindnyájan*); devoiced final *j* written `h` (*lépj, rakj, kapj*);
English words not in the table (*chat, spray, hacker, jackpot, ketchup*);
and single letters/abbreviations with several readings (*q, mp, tsz, ui*).

## Known gaps

* No lexicon: unpredictable loanword pronunciation beyond the small table;
  *ch* in loans is `h` (*technika*) unless word-initial or final.
* Compound boundaries are not detected beyond the listed prefixes, so
  *házsor* is read with `ʒ` and *kisszoba* with `sː`.
* No cross-word voicing assimilation in `join` (espeak does none either).
* Vowel shortening in derived stems (*víz* → *vizet*) only when spelled.
* Secondary stress (`ˌ`) is never written; espeak's `$unstressend` word list
  is not reproduced (always `ˈ` on the first syllable).
