//! Compact pronunciation lexicon: an FST from word to an offset into a blob of
//! phoneme strings. Memory-mapped, so a multi-million-entry lexicon costs almost
//! no RAM and loads instantly.
//!
//! File layout: b"PGLX" | u32 version | u64 fst_len | fst bytes | blob bytes.
//! Each FST value packs (offset << 16) | len into the blob. A blob entry may hold
//! several pronunciations separated by '\t' (first one is the default).

use std::fs::File;
use std::io::{self, Write};
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

use fst::{Map, MapBuilder};
use memmap2::Mmap;

#[derive(Clone)]
enum Backing {
    Mmap(Arc<Mmap>),
    Static(&'static [u8]),
    Owned(Arc<Vec<u8>>),
}

impl Backing {
    fn bytes(&self) -> &[u8] {
        match self {
            Backing::Mmap(m) => m,
            Backing::Static(s) => s,
            Backing::Owned(v) => v,
        }
    }
}

#[derive(Clone)]
struct Slice {
    b: Backing,
    r: Range<usize>,
}

impl AsRef<[u8]> for Slice {
    fn as_ref(&self) -> &[u8] {
        &self.b.bytes()[self.r.clone()]
    }
}

pub struct Lexicon {
    map: Map<Slice>,
    blob: Slice,
}

const MAGIC: &[u8; 4] = b"PGLX";

impl Lexicon {
    pub fn open(path: &Path) -> io::Result<Lexicon> {
        let f = File::open(path)?;
        // SAFETY: the file is opened read-only and not expected to change while mapped.
        let m = unsafe { Mmap::map(&f)? };
        Self::from_backing(Backing::Mmap(Arc::new(m)))
    }

    pub fn from_static(bytes: &'static [u8]) -> io::Result<Lexicon> {
        Self::from_backing(Backing::Static(bytes))
    }

    pub fn from_vec(bytes: Vec<u8>) -> io::Result<Lexicon> {
        Self::from_backing(Backing::Owned(Arc::new(bytes)))
    }

    fn from_backing(b: Backing) -> io::Result<Lexicon> {
        let bytes = b.bytes();
        if bytes.len() < 16 || &bytes[..4] != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "not a polyg2p lexicon"));
        }
        let fst_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let fst_r = 16..16 + fst_len;
        let blob_r = 16 + fst_len..bytes.len();
        let map = Map::new(Slice { b: b.clone(), r: fst_r }).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Lexicon { map, blob: Slice { b, r: blob_r } })
    }

    /// All pronunciations for `word`, tab-separated (first is the default).
    pub fn get_all(&self, word: &str) -> Option<&str> {
        let v = self.map.get(word.as_bytes())?;
        let off = (v >> 16) as usize;
        let len = (v & 0xFFFF) as usize;
        std::str::from_utf8(&self.blob.as_ref()[off..off + len]).ok()
    }

    /// The default pronunciation for `word`.
    pub fn get(&self, word: &str) -> Option<&str> {
        self.get_all(word).map(|s| s.split('\t').next().unwrap_or(s))
    }

    pub fn contains(&self, word: &str) -> bool {
        self.map.contains_key(word.as_bytes())
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Build a lexicon file from (word, pronunciations) pairs. Pairs need not be sorted.
pub fn build<W: Write>(mut entries: Vec<(String, String)>, out: W) -> io::Result<()> {
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    entries.dedup_by(|a, b| a.0 == b.0);
    let mut blob: Vec<u8> = Vec::new();
    let mut fst_bytes = Vec::new();
    {
        let mut mb = MapBuilder::new(&mut fst_bytes).map_err(io::Error::other)?;
        // share identical pronunciation strings
        let mut seen: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for (w, p) in &entries {
            let v = if let Some(&v) = seen.get(p) {
                v
            } else {
                let off = blob.len() as u64;
                blob.extend_from_slice(p.as_bytes());
                let len = p.len() as u64;
                assert!(len < 0xFFFF, "pronunciation too long for {w}");
                let v = (off << 16) | len;
                seen.insert(p.clone(), v);
                v
            };
            mb.insert(w.as_bytes(), v).map_err(io::Error::other)?;
        }
        mb.finish().map_err(io::Error::other)?;
    }
    let mut out = out;
    out.write_all(MAGIC)?;
    out.write_all(&1u32.to_le_bytes())?;
    out.write_all(&(fst_bytes.len() as u64).to_le_bytes())?;
    out.write_all(&fst_bytes)?;
    out.write_all(&blob)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let mut v = Vec::new();
        build(vec![("b".into(), "bˈiː".into()), ("a".into(), "ɐ\tˈeɪ".into())], &mut v).unwrap();
        let l = Lexicon::from_vec(v).unwrap();
        assert_eq!(l.get("a"), Some("ɐ"));
        assert_eq!(l.get_all("a"), Some("ɐ\tˈeɪ"));
        assert_eq!(l.get("b"), Some("bˈiː"));
        assert_eq!(l.get("c"), None);
    }
}
