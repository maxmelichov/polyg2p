//! Hungarian grapheme-to-phoneme, espeak-style notation.
//!
//! STUB: not implemented yet. The contract every rule-based language module
//! fulfils (see `es.rs` for a complete example):
//!   word(lw)      context-free phonemes of one lower-case word
//!   join(outs)    sentence pass (cross-word effects, homographs); may be empty
//!   cardinal(n)   number words for n (one token per word)
//!   DECIMAL       the word read for the decimal separator
//!   symbol(c)     spoken form of % $ € £ & + @ ° =
//!   LETTERS       letter names a..z for spelling acronyms (None = spell nothing)

use crate::Out;

pub const IMPLEMENTED: bool = false;
pub const DECIMAL: &str = "";
pub const LETTERS: Option<&[&str; 26]> = None;

pub fn word(_lw: &str) -> String {
    String::new()
}

pub fn join(_outs: &mut [Out]) {}

pub fn cardinal(n: u128) -> Vec<String> {
    vec![n.to_string()]
}

pub fn symbol(_c: char) -> Option<&'static str> {
    None
}
