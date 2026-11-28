//! Locked-test hygiene (the repo has no CI; `cargo test` is the check).
//! `tests/locked/MANIFEST.tsv`: `sha256 path role frozen note`, append-only, the last row of a path wins.
//! Fails when: a listed file's sha256 differs (an edited test); a tests/*.tsv is not listed; a manifest line that
//! `origin/main` has is missing or changed; a `test` file has two or more rows in `LOOKS.tsv` (a second score
//! makes it dev: append a new row with role dev). A path that starts with `/` is external data: hashed only when present.
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut m = data.to_vec();
    m.push(0x80);
    while m.len() % 64 != 56 {
        m.push(0);
    }
    m.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for blk in m.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([blk[4 * i], blk[4 * i + 1], blk[4 * i + 2], blk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut a = h;
        for i in 0..64 {
            let s1 = a[4].rotate_right(6) ^ a[4].rotate_right(11) ^ a[4].rotate_right(25);
            let ch = (a[4] & a[5]) ^ (!a[4] & a[6]);
            let t1 = a[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a[0].rotate_right(2) ^ a[0].rotate_right(13) ^ a[0].rotate_right(22);
            let maj = (a[0] & a[1]) ^ (a[0] & a[2]) ^ (a[1] & a[2]);
            let t2 = s0.wrapping_add(maj);
            a = [t1.wrapping_add(t2), a[0], a[1], a[2], a[3].wrapping_add(t1), a[4], a[5], a[6]];
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(a[i]);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

fn rows(txt: &str) -> Vec<Vec<String>> {
    txt.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| l.split('\t').map(String::from).collect()).collect()
}

#[test]
fn sha256_known_answers() {
    assert_eq!(sha256(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn locked_files_unchanged() {
    let man = std::fs::read_to_string(format!("{ROOT}/tests/locked/MANIFEST.tsv")).unwrap();
    let mut last: HashMap<String, (String, String)> = HashMap::new(); // path -> (sha, role)
    for r in rows(&man) {
        assert_eq!(r.len(), 5, "manifest row needs 5 tab-separated fields: {r:?}");
        assert!(["test", "dev", "unit"].contains(&r[2].as_str()), "unknown role in {r:?}");
        last.insert(r[1].clone(), (r[0].clone(), r[2].clone()));
    }
    // every listed file still has the recorded hash (a superseded older row of the same path may differ)
    for (p, (sha, _)) in &last {
        let full = if p.starts_with('/') { p.clone() } else { format!("{ROOT}/{p}") };
        match std::fs::read(&full) {
            Ok(b) => assert_eq!(&sha256(&b), sha, "{p} was edited after it was locked; add a new file or a new manifest row"),
            Err(_) if p.starts_with('/') => eprintln!("external file {p} not present, not checked"),
            Err(e) => panic!("{p}: {e}"),
        }
    }
    // every tests/*.tsv is listed
    for e in std::fs::read_dir(format!("{ROOT}/tests")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "tsv") {
            let rel = format!("tests/{}", p.file_name().unwrap().to_string_lossy());
            assert!(last.contains_key(&rel), "{rel} is not in tests/locked/MANIFEST.tsv");
        }
    }
    // a test file scored twice is dev
    let looks = std::fs::read_to_string(format!("{ROOT}/tests/locked/LOOKS.tsv")).unwrap();
    let mut n: HashMap<String, usize> = HashMap::new();
    for r in rows(&looks) {
        *n.entry(r[1].clone()).or_default() += 1;
    }
    for (p, c) in n {
        if c >= 2 {
            assert_ne!(last.get(&p).map(|x| x.1.as_str()), Some("test"), "{p} has {c} looks: re-list it as dev");
        }
    }
}

#[test]
fn manifest_is_append_only() {
    // against origin/main when git and the ref exist (a fresh tarball or a shallow clone skips this)
    let Ok(o) = Command::new("git").current_dir(ROOT).args(["show", "origin/main:tests/locked/MANIFEST.tsv"]).output() else { return };
    if !o.status.success() || !Path::new(&format!("{ROOT}/.git")).exists() {
        return;
    }
    let old = rows(&String::from_utf8_lossy(&o.stdout));
    let new = rows(&std::fs::read_to_string(format!("{ROOT}/tests/locked/MANIFEST.tsv")).unwrap());
    assert!(new.len() >= old.len() && new[..old.len()] == old[..], "tests/locked/MANIFEST.tsv: lines already on origin/main were removed or changed");
}
