//! Python bindings for polyg2p (the Rust core: en, es, it, de).
use std::path::PathBuf;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(frozen, module = "polyg2p._polyg2p")]
struct Engine {
    inner: polyg2p::G2p,
}

fn lang(l: &str) -> PyResult<polyg2p::Lang> {
    polyg2p::Lang::parse(l).ok_or_else(|| PyValueError::new_err(format!("unsupported language for the Rust core: {l}")))
}

#[pymethods]
impl Engine {
    /// Engine(data_dir=None): data_dir holds optional en.lex / en.g2p / de.lex / it.lex ... files.
    #[new]
    #[pyo3(signature = (data_dir=None))]
    fn new(data_dir: Option<PathBuf>) -> Self {
        Engine { inner: polyg2p::G2p::new(data_dir.as_deref()) }
    }

    /// Phonemize one text. The GIL is released while Rust works.
    fn phonemize(&self, py: Python<'_>, text: &str, lang_code: &str) -> PyResult<String> {
        let l = lang(lang_code)?;
        let text = text.to_owned();
        Ok(py.allow_threads(|| self.inner.phonemize(&text, l)))
    }

    /// Phonemize many texts in one call (cheaper per item than repeated calls).
    fn phonemize_batch(&self, py: Python<'_>, texts: Vec<String>, lang_code: &str) -> PyResult<Vec<String>> {
        let l = lang(lang_code)?;
        Ok(py.allow_threads(|| self.inner.phonemize_batch(&texts, l)))
    }
}

#[pymodule]
fn _polyg2p(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Engine>()?;
    m.add("CORE_LANGS", vec!["en", "es", "it", "de", "ru"])?;
    Ok(())
}
