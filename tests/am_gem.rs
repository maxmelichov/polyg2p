//! Amharic gemination layer: loading, windowing, the table override. A tiny model with fixed weights is built here,
//! so no data directory is needed. The real model is checked only when `POLYG2P_AM_GEM` points at one.
use polyg2p::{G2p, Lang, am, tagger::Tagger};

/// A G2PT file whose encoder is the identity-free zero network: every tensor 0 except the `stress` bias, so every
/// letter gets logit[1] - logit[0] = `margin`. 2 layers, d 16, max_len `max_len`.
fn tiny(margin: f32, max_len: usize) -> Tagger {
    let (d, ff, layers, nv) = (16usize, 16usize, 2usize, 8usize);
    let mut names: Vec<(String, Vec<usize>, Vec<f32>)> = Vec::new();
    let mut t = |n: &str, s: &[usize], v: Option<Vec<f32>>| names.push((n.to_string(), s.to_vec(), v.unwrap_or_else(|| vec![0.0; s.iter().product()])));
    t("emb.weight", &[nv, d], None);
    t("pos.weight", &[max_len, d], None);
    for l in 0..layers {
        for ln in ["ln1", "ln3"] {
            t(&format!("enc.{l}.{ln}.weight"), &[d], Some(vec![1.0; d]));
            t(&format!("enc.{l}.{ln}.bias"), &[d], None);
        }
        for p in ["sa.q", "sa.k", "sa.v", "sa.o"] {
            t(&format!("enc.{l}.{p}.weight"), &[d, d], None);
            t(&format!("enc.{l}.{p}.bias"), &[d], None);
        }
        t(&format!("enc.{l}.ff1.weight"), &[ff, d], None);
        t(&format!("enc.{l}.ff1.bias"), &[ff], None);
        t(&format!("enc.{l}.ff2.weight"), &[d, ff], None);
        t(&format!("enc.{l}.ff2.bias"), &[d], None);
    }
    t("norm.weight", &[d], Some(vec![1.0; d]));
    t("norm.bias", &[d], None);
    t("cons.weight", &[1, d], None);
    t("cons.bias", &[1], None);
    t("vow.weight", &[1, d], None);
    t("vow.bias", &[1], None);
    t("stress.weight", &[2, d], None);
    t("stress.bias", &[2], Some(vec![0.0, margin]));
    let mut off = 0usize;
    let mut meta = Vec::new();
    let mut blob = Vec::new();
    for (n, s, v) in &names {
        meta.push(format!(r#"{{"name":"{n}","shape":{s:?},"offset":{off}}}"#));
        for x in v {
            blob.extend_from_slice(&x.to_le_bytes());
        }
        off += v.len() * 4;
    }
    let vocab: Vec<String> = ["<pad>", "<unk>", " ", ".", "ም", "ሽ", "ት", "አ"].iter().enumerate().map(|(i, c)| format!(r#""{c}":{i}"#)).collect();
    let header = format!(
        r#"{{"magic":"G2PT","config":{{"d":{d},"heads":2,"layers":{layers},"ff":{ff},"max_len":{max_len},"n_cons":1,"n_vow":1,"n_vocab":{nv}}},"vocab":{{{}}},"cons_vocab":["-"],"vow_vocab":["-"],"letter_cons":{{}},"tensors":[{}]}}"#,
        vocab.join(","),
        meta.join(",")
    );
    let mut f = b"G2PT".to_vec();
    f.extend_from_slice(&(header.len() as u32).to_le_bytes());
    f.extend_from_slice(header.as_bytes());
    f.extend_from_slice(&blob);
    Tagger::from_bytes(&f).expect("tiny model loads")
}

#[test]
fn no_model_no_table_is_the_plain_reading() {
    let plain = G2p::new(None);
    let empty = G2p::new(None).with_amharic(am::Gem::from_parts(None, None));
    for s in ["ምሽት አምስት ሀብታምነት።", "ሰላም ነው! እንዴት ነህ?"] {
        assert_eq!(plain.phonemize(s, Lang::Am), empty.phonemize(s, Lang::Am));
        assert!(!plain.phonemize(s, Lang::Am).contains('ː'));
    }
}

#[test]
fn model_marks_every_allowed_letter_and_windows_cover_long_input() {
    // margin +5: every letter the guards allow is long
    let g = G2p::new(None).with_amharic(am::Gem::from_parts(Some(tiny(5.0, 512)), None));
    assert_eq!(g.phonemize("ምሽት", Lang::Am), "mɨʃːɨt");
    // margin -5: nothing is long
    let g0 = G2p::new(None).with_amharic(am::Gem::from_parts(Some(tiny(-5.0, 512)), None));
    assert_eq!(g0.phonemize("ምሽት", Lang::Am), G2p::new(None).phonemize("ምሽት", Lang::Am));
    // a window of 8 characters: the tagger reads one word at a time, every word is still flagged
    let gw = G2p::new(None).with_amharic(am::Gem::from_parts(Some(tiny(5.0, 8)), None));
    let s = "ምሽት ምሽት ምሽት ምሽት ምሽት ምሽት";
    let out = gw.phonemize(s, Lang::Am);
    assert_eq!(out.matches('ː').count(), 6, "{out}");
    // a word longer than the window is read in pieces and every allowed letter is still flagged
    let g4 = G2p::new(None).with_amharic(am::Gem::from_parts(Some(tiny(5.0, 4)), None));
    let w = g4.phonemize("ምሽትምሽትምሽት", Lang::Am);
    assert_eq!(w.matches('ː').count(), 7, "{w}"); // 9 letters: the first and the final bare one are guarded
}

#[test]
fn table_replaces_the_tagger_for_its_word() {
    // the table says: ምሽት has no long letter, አምስት has ም long (U+135F after the letter)
    let table = "ምሽት\tምሽት\nአምስት\tአም\u{135F}ስት\n";
    let g = G2p::new(None).with_amharic(am::Gem::from_parts(Some(tiny(5.0, 512)), Some(table)));
    assert_eq!(g.phonemize("ምሽት", Lang::Am), G2p::new(None).phonemize("ምሽት", Lang::Am));
    assert_eq!(g.phonemize("አምስት", Lang::Am), "amːɨst");
    // a table alone works without a model
    let gt = G2p::new(None).with_amharic(am::Gem::from_parts(None, Some(table)));
    assert_eq!(gt.phonemize("አምስት", Lang::Am), "amːɨst");
}

#[test]
fn real_model_when_given() {
    let Some(p) = std::env::var_os("POLYG2P_AM_GEM").filter(|p| p != "0") else { return };
    let m = Tagger::open(std::path::Path::new(&p)).expect("load");
    let g = G2p::new(None).with_amharic(am::Gem::from_parts(Some(m), None));
    let plain = G2p::new(None).phonemize("ሰላም ነው", Lang::Am);
    assert!(!g.phonemize("ሰላም ነው", Lang::Am).is_empty() && !plain.is_empty());
}
