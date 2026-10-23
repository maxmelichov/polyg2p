//! Hebrew: context-based niqqud/stress prediction + rules to IPA.
//!
//! STUB: see docs/hebrew.md for the design (a small sentence-level tagger
//! distilled from RenikudPlus, run natively, then rule-based rendering).

use std::path::Path;

pub const IMPLEMENTED: bool = false;

pub struct Hebrew {}

impl Hebrew {
    pub fn load(_data_dir: Option<&Path>) -> Self {
        Hebrew {}
    }

    pub fn phonemize(&self, _text: &str) -> String {
        String::new()
    }
}
