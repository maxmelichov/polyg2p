# Brazilian Portuguese (`pt`)

Rules-only engine in `src/pt.rs`, espeak-ng `pt-br` notation. Measured with
`tools/eval_acc.py pt` against WikiPron's Brazilian gold
(`por_latn_bz_broad.tsv`, 188k entries) on the 20 000 most frequent words
(wordfreq) that have a gold entry. Both outputs and the gold go through
`tools/collapse_pt.py` first, so r-sounds, coda s/ʃ, final-vowel reduction,
glide spelling and t/d palatalization don't count; open/closed e/o (ɛ/e, ɔ/o)
does.

## Numbers

| 20k most frequent words              | polyg2p word acc | PER  | espeak word acc | PER  |
|--------------------------------------|------------------|------|-----------------|------|
| frequency-weighted                   | **99.6 %**       | 0.8 %| 96.9 %          | 1.0 %|
| uniform weights                      | **96.3 %**       | 0.8 %| 95.1 %          | 1.0 %|
| held-out statistics, freq-weighted ¹ | 98.4 %           | 1.0 %| 96.9 %          | 1.0 %|
| held-out statistics, uniform ¹       | 95.1 %           | 1.0 %| 95.1 %          | 1.0 %|

¹ `data/pt_open.tsv` rebuilt with `tools/pt_open.py --exclude <the 20k eval
words>`, i.e. the open/closed statistics never saw the words they are scored
on. The shipped table uses the whole gold file.

Notation agreement with espeak on the same words (after the ligature fold
`tʃ dʒ → ʧ ʤ` that `fold.rs` applies): 92.7 % identical strings, 93.4 % when
stress marks are ignored. The remaining differences are the places where we
disagree with espeak on purpose (next section).

## What the engine does

* **Stress**: written accent (acute/circumflex, else tilde unless followed by
  `-zinh-`); otherwise penultimate for words ending in a/e/o(+s), -am, -em,
  -ens, final for everything else (r, l, z, i, u, ão, im, um, om, x, ...).
  Orthographic nuclei follow the spelling rules: i/u + vowel is a hiatus unless
  post-tonic (`história` → `ˌistˈɔɾjæ`), falling diphthongs ai/ei/oi/ui/au/eu/
  iu/ou are one nucleus except before a word-final r/z/l/m/n or `nh`
  (`sair`, `raiz`, `ruim`, `rainha`, `ainda`).
* **Open/closed e, o**: written accent when present, closed before a nasal,
  otherwise `data/pt_open.tsv`: the frequency-weighted share of open vowels per
  word ending (from the stressed vowel to the end, with 0/1/2 letters of left
  context; longest key wins), computed by `tools/pt_open.py` from the gold
  file. Two small lists in `pt.rs` (`OPEN`, `CLOSED`) pin frequent words whose
  ending is ambiguous (`agora`, `maior`, `hoje`, `senhor`, `fosse`, future
  subjunctives in `-er`, ...). `-mente` adverbs keep the adjective's vowel
  (`novamente` → `nˌɔvæmˈeɪŋtʃy`; `provavelmente`, `seriamente` restore the
  dropped accent of the adjective).
* **Nasals**: `ɐ̃ eɪ i o ũ` + `ŋ`/`m` in the coda, `ão`=`ɐ̃ʊ̃`, `-am`=`ɐ̃ʊ̃`,
  `ãe`=`ɐ̃y`, `õe`=`õj`, `-em`=`eɪŋ`; stressed a/u before m/n/nh → `ɐ̃`/`ũ`.
* **Consonants**: `t/d` + i (or final unstressed e, or a j-glide) → `tʃ dʒ`;
  coda `l` → `ʊ` (`w` after o/u) everywhere, including after ɔ where espeak
  keeps `l` (`sol` → `sˈɔw`); `r` → `x` initial/doubled/after n-l-s, `ɾə` in a
  medial coda, `r` final, `ɾ` elsewhere; `lh`=`lj`, `nh`=`ɲ`; s/z voicing by
  position; `x` by context and a stem list (`ʃ` default, `ks` in `fixo sexo
  táxi tóxico ...`, `s` in `próximo máximo auxílio trouxe`, `z` in initial
  `ex-` + vowel, `s` before a consonant); qu/gu before e/i silent-u with the
  usual exception list (`tranquilo`, `linguiça`, `aguentar`, ...).
* **Epenthesis**: word-final stops get the Brazilian `i` (`internet` →
  `ˌiŋteɾənˈɛtʃi`, `blog` → `blˈɔɡi`), and `d`/`t` before another consonant do
  too (`advogado` → `ˌadʒivoɡˈadʊ`, `ritmo` → `xˈitʃimʊ`); the gold only lists
  the epenthetic forms for the `ad-` words.
* **Function words**: a fixed table (`de` → `dʒy`, `o` → `ʊ`, `que` → `ky`,
  `e` → `i`, `com` → `koŋ`, `muito` → `mwˈiŋtʊ` as espeak writes it, ...), a
  list of secondary-stress words (`para`, `como`, `está`, `onde`, ...),
  frequent English loanwords (`site`, `show`, `web`, `facebook`, ...) and chat
  abbreviations (`vc`, `pq`, `tb`).
* **Sentence pass** (`join`): word-final `s` → `z` before a vowel or voiced
  consonant (`os amigos` → `ʊz ˌæmˈiɡʊs`, `mais ou menos` → `mˈaɪz ow`), as
  espeak does.
* **Numbers**: masculine cardinals with the Portuguese `e` rules (`mil
  duzentos e trinta e quatro`, `dois milhões e quinhentos mil`); `DECIMAL` =
  `vírgula`; symbols `% $ € £ & + @ ° =`; letter names for acronyms (`FBI` →
  `ˌɛfybˌeˈi`); single consonants and vowel-less tokens are spelled.

## Where we differ from espeak on purpose

* espeak keeps `l` after a stressed ɔ (`sol` → `sˈɔl`, `volta` → `vˈɔltæ`);
  Brazilian Portuguese vocalizes every coda l.
* espeak has no open/closed statistics: `hoje` → `ˈɔʒy`, `novo` → `nˈɔvʊ`,
  `agora` → `ˌaɡˈɔɾæ` but `mulher` → `muljˈer`, `teto` → `tˈɛtʊ`, `pega` →
  `pˈɛɡæ`; about 400 of the 20k words differ in e/ɛ or o/ɔ.
* espeak writes `e` for the conjunction `e` (`ˈɛ`) and `ˈɔ` for the article
  `o` in isolation; we write `i` and `ʊ`.
* epenthetic `i` (above); post-tonic `e` before a vowel as a glide (`área` →
  `ˈaɾjæ`, espeak `ˈaɾeæ`; the gold has `aɾia`/`aɾja`).
* `-mente` adverbs: espeak reads the adjective with its default (closed) vowel.

## Remaining errors (top 20k, uniform weights: 3.7 %)

Roughly in order of frequency:

1. **English names and loanwords** (`Jason`, `Vanessa`, `Lincoln`, `brother`,
   `bacon`, `record`, `single`, ...): read with Portuguese rules. The ~100
   most frequent ones are in the fixed table; the rest would need an English
   fallback.
2. **Open/closed e/o** in words whose ending is genuinely ambiguous
   (`credo`, `grega`, `vermelhos`, `aconteceram`, `cheias`): no spelling cue,
   the statistics pick the majority for the ending. Homographs with both
   readings (`seca`, `olho`, `força`, `gosto`) get the more frequent one.
3. **Pre-tonic vowel quality**: the gold sometimes has an open pre-tonic vowel
   (`aeroporto` → `aɛroportu`, `protocolo` → `prɔtokɔlu`, `historicamente`),
   or raised `o`/`e` (`preocupado` → `preukupadu`); we always write the closed
   vowel there, like espeak.
4. **Odd gold entries**: `quanto` → `katu`, `áreas` → `areas` only (while
   `área` lists `aɾja`), `mexicano` → `meksikanu`, `h` → `r`.

## Known gaps

* No lexicon: homographs that differ only in e/o quality or stress (`pôr/por`
  handled by the accent, but `sede`, `colher`, `gosto`, `jogo` always get one
  reading).
* The epenthetic `i` after `d`/`t` + consonant is applied inside every word;
  words where the cluster is a real onset in careful speech (`atmosfera`)
  still get it (the gold accepts both).
* `R$` reaches `symbol` as the letter `R` + `$` and is read `ˈɛxy dˈɔlar`;
  `10:30` has no `e` between hours and minutes (both in `num.rs` /
  `text.rs`, outside this module).
* European Portuguese is not supported (the engine is Brazilian only).

## Files

* `src/pt.rs` — the engine (rules, tables, unit tests).
* `data/pt_open.tsv` — ending statistics for stressed e/o (derived counts, no
  pronunciations), built by `tools/pt_open.py`.
* `tools/collapse_pt.py` — broad-phone collapse used by `eval_acc.py`.
