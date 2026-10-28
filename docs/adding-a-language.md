# Adding a language

Every language lives in one file, `src/<code>.rs`, and nothing else needs to
change: `lib.rs`, `num.rs` and `spell.rs` already dispatch to it. A stub for
each planned language is checked in with `IMPLEMENTED = false`.

## The module contract

```rust
pub const IMPLEMENTED: bool;                 // flip to true when word() works
pub const DECIMAL: &str;                     // word read for the decimal separator ("virgule")
pub const LETTERS: Option<&[&str; 26]>;      // letter names a..z for spelling acronyms
pub fn word(lw: &str) -> String;             // context-free phonemes of one lower-case word
pub fn join(outs: &mut [Out]);               // sentence pass: liaison, sandhi, homographs
pub fn cardinal(n: u128) -> Vec<String>;     // number words, one orthographic word per item
pub fn symbol(c: char) -> Option<&'static str>; // % $ € £ & + @ ° =
```

`word` receives one token already lower-cased. It may contain apostrophes and
hyphens (`l'homme`, `peut-être`). Return `""` for tokens with no letters. If the
language needs a lexicon or a model, follow `it.rs` / `de.rs`: a struct with
`load(Option<&Path>)` held in a `OnceLock` on `G2p` (see `lib.rs`), and files in
the data directory named `<code>.lex` / `<code>.g2p`.

`cardinal` returns orthographic words; `lib.rs` feeds each one back through
`word`, so number words must be pronounceable by the rules. Grammatical
agreement (gender, case) is handled by the language's `join` if needed.

## Notation: match espeak-ng exactly

The output is consumed by TTS models trained on espeak-ng phonemes (after
BlueTTS `normalize_text`: NFD, `ʧ ʤ ʦ ʣ` ligatures — `fold.rs` does that at the
end, don't do it yourself). The *symbol conventions* must be espeak's; the
*pronunciations* must be right even where espeak's aren't. Workflow:

1. `tools/espeak_words.py <lang> <n>` phonemizes frequent words with espeak
   (`.venv/bin/python`, needs `phonemizer` + `espeakng-loader`; both installed).
   Study its symbol inventory: which vowels, stress marks (`ˈ ˌ`), length `ː`,
   how it writes palatals, affricates, nasal vowels, r-sounds, syllabic
   consonants. Write those symbols.
2. Implement `word` from the orthography rules, then diff against espeak on
   several thousand words (`tools/cmp_ours.py` / `tools/diffstat.py`) and sort
   the differences into "notation" (fix ours) vs "espeak is wrong" (keep ours).
3. Measure against Wiktionary with `tools/eval_acc.py <lang> --show 40`. The
   gold files live in `sources/` (WikiPron; for Catalan run `tools/ca_gold.py` to
   derive `cat_latn_central.tsv` from the narrow file). Add `tools/collapse_<lang>.py`
   with a `collapse(s)` that maps both outputs to a shared broad phone set so
   only real pronunciation differences count. The target: beat espeak's
   word accuracy on the 20k most frequent words, and never be far below it.
4. `cargo test` must pass; add unit tests with ~20 words covering every rule.
5. Update the language table and results in `README.md`.

Keep stress conventions: primary `ˈ` before the stressed *syllable's onset
consonant* the way espeak places it for that language (check its output; in
most languages espeak puts `ˈ` before the vowel's onset, e.g. `kˈasa`).

## Speed and dependencies

Rules only, no regex crate, no allocations per character where avoidable.
Everything compiled into the crate must be Apache-2.0 compatible; Wiktionary-
derived lexicons (CC BY-SA) go to the optional data pack, never into `src/`.
