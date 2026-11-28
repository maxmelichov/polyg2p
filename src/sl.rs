//! Sl grapheme-to-phoneme: scaffold stub (`IMPLEMENTED = false`), see `docs/adding-a-language.md`.
//! The engine lands in a later change; until then the language is listed but not served.

use crate::Out;

pub const IMPLEMENTED: bool = false;
pub const DECIMAL: &str = "";
pub const LETTERS: Option<&[&str; 26]> = None;

pub fn word(_lw: &str) -> String {
    String::new()
}

pub fn join(_outs: &mut [Out]) {}

/// Stub: the number is kept as digits (no panic for any size).
pub fn cardinal(n: u128) -> Vec<String> {
    vec![n.to_string()]
}

pub fn symbol(_c: char) -> Option<&'static str> {
    None
}
