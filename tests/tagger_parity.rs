//! The shared-encoder refactor of tagger.rs must not change any logit: checks a digest of the
//! committed Russian context model's outputs (compare the same test on the commit before the refactor).
use polyg2p::tagger::Tagger;

#[test]
fn digest() {
    let m = Tagger::open(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ru-ctx.g2p"))).unwrap();
    let mut acc = 0u64;
    for s in ["на столе стоит замок и ёлка", "все дела идут хорошо", "она берёт всё что есть на полке"] {
        let ids = m.encode(s);
        let lg = m.forward(&ids);
        for v in lg.cons.iter().chain(&lg.vow).chain(&lg.stress) {
            acc = acc.wrapping_mul(1099511628211).wrapping_add(v.to_bits() as u64);
        }
    }
    println!("DIGEST {acc}");
    // value recorded on x86_64 from the commit before the encoder was factored out of the tagger
    #[cfg(not(target_arch = "aarch64"))]
    assert_eq!(acc, 9220334301654860735);
}
