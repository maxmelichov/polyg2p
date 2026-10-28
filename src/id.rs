//! Indonesian grapheme-to-phoneme, espeak-style notation.
//!
//! Indonesian spelling is nearly phonemic; the one real ambiguity is the
//! letter `e`, which is either schwa (pepet, `ə`) or a front vowel (taling,
//! written `ɛ` as espeak-ng's `id` voice does). This module decides it from
//! morphology plus a small table:
//!
//! * derivational prefixes (`me-`/`mem-`/`men-`/`meng-`/`meny-`, `pe-`/`pen-`…,
//!   `ber-`, `ter-`, `per-`, `ke-`, `se-`, `di-`) always have schwa and are
//!   peeled off (`memperoleh` -> `məmpərˈolɛh`);
//! * inside the stem, `e` in a non-final syllable is schwa unless the word
//!   looks foreign (consonant clusters, `ks`, f/v/z, hiatus) or `e` stands
//!   before `w`, `h`+vowel or a vowel; a final closed syllable has `ɛ`
//!   (`-en -et -ek -es -el -eh -ep`) except `-er`/`-em` (`sumbər`, `sistəm`);
//! * ~450 frequent stems whose reading the rules get wrong (`mereka`,
//!   `dekat`, `memang`, `keren`, `meski`…) are in `E_TABLE`; derived forms
//!   reach the table through prefix/suffix stripping (`kesehatan` -> `sehat`).
//!   The table was derived from aggregate statistics over WikiPron.
//!
//! Notation follows espeak's `id` voice: `ɛ` for taling e, `ɔ` for o before
//! r, `aɪ aʊ` for the diphthongs ai/au (one syllable), `ŋ ɲ ç x` for ng ny sy
//! kh (espeak writes sy as `ç`), `tʃ dʒ` for c j (folded to ligatures at the
//! end), final k as `k`, stress `ˈ` right before the vowel of the penultimate
//! syllable, or the last one when the penult has schwa (`təlˈah`); secondary
//! `ˌ` every second non-schwa syllable before it (`mˌanusˈia`).

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "koma";
/// Letter names a..z (phonemes). espeak reads the letter name vowel as plain `e`.
pub const LETTERS: Option<&[&str; 26]> = Some(&[
    "a", "be", "tʃe", "de", "e", "ɛf", "ɡe", "ha", "i", "dʒe", "ka", "ɛl", "ɛm", "ɛn", "o", "pe", "ki", "ɛr", "ɛs",
    "te", "u", "fe", "we", "ɛks", "je", "zɛt",
]);

/// Function words espeak reads without any stress mark.
const UNSTRESSED: &[&str] = &["yang", "dan", "di", "ke", "si", "sang", "per", "pun", "lah", "kah", "nya"];
/// Function words espeak reads with a secondary stress only (on the penultimate syllable).
const SECONDARY: &[&str] = &["dari", "untuk", "adalah", "pada", "atau", "sebagai", "bahwa", "suatu", "daripada", "ialah"];

/// Common abbreviations read as their full words.
const ABBREV: &[(&str, &str)] = &[
    ("yg", "yang"), ("dgn", "dengan"), ("dll", "dan lain-lain"), ("dsb", "dan sebagainya"), ("tsb", "tersebut"),
    ("tdk", "tidak"), ("krn", "karena"), ("spt", "seperti"), ("sdh", "sudah"), ("blm", "belum"), ("jl", "jalan"),
    ("jln", "jalan"), ("rp", "rupiah"), ("km", "kilometer"), ("kg", "kilogram"), ("cm", "sentimeter"), ("mm", "milimeter"),
    ("bpk", "bapak"), ("sdr", "saudara"), ("thn", "tahun"), ("bln", "bulan"), ("hrs", "harus"), ("utk", "untuk"),
    ("pd", "pada"), ("tgl", "tanggal"), ("dr", "dari"), ("dlm", "dalam"), ("sy", "saya"), ("ttg", "tentang"),
    ("bhw", "bahwa"), ("kpd", "kepada"), ("jg", "juga"), ("sm", "sama"), ("bs", "bisa"), ("dg", "dengan"),
    ("no", "nomor"), ("tp", "tapi"), ("org", "orang"), ("bnr", "benar"), ("sbg", "sebagai"), ("msh", "masih"),
    ("hr", "hari"), ("mnt", "menit"), ("dtk", "detik"), ("kec", "kecamatan"), ("kab", "kabupaten"), ("prov", "provinsi"),
];

/// Stems whose `e` readings the rules get wrong: one letter per written `e`
/// of the key, `ə` or `e`. Sorted for binary search. Keys are what is left
/// after stripping `-nya`, one suffix (`-kan -an -i`) and the derivational
/// prefixes, so `kesehatan` finds `sehat`; keys starting with `=` match the
/// whole word only (`=beri` is `bɛri`, but `berikan` is `bərikan`).
const E_TABLE: &[(&str, &str)] = &[
    ("=belerang", "əe"), ("=beri", "e"), ("=desak", "e"), ("=dicoret", "ə"), ("=diseret", "ee"), ("=empati", "e"),
    ("=esai", "ə"), ("=gerai", "e"), ("=ke", "ə"), ("=kelewat", "əə"), ("=kemoterapi", "ee"), ("=kepri", "ə"),
    ("=lebaran", "ə"), ("=lempar", "e"), ("=letak", "e"), ("=menetes", "eee"), ("=meneteskan", "eee"),
    ("=menstruasi", "e"), ("=penembak", "əe"), ("=periode", "eə"), ("=perkemahan", "ee"), ("=se", "ə"),
    ("=sedimen", "ee"), ("=seismik", "e"), ("=sekertaris", "eə"), ("=sekretariat", "eə"), ("=sekretaris", "eə"),
    ("=selo", "e"), ("=senator", "e"), ("=senior", "e"), ("=sensasional", "e"), ("=sensitivitas", "e"),
    ("=sentimen", "ee"), ("=sentimental", "ee"), ("=serang", "e"), ("=serap", "e"), ("=seret", "əə"),
    ("=sertifikat", "e"), ("=tembak", "e"), ("=tempel", "ee"), ("=tempelkan", "əə"), ("=terapi", "e"),
    ("=terapis", "e"), ("=teras", "e"), ("=terminator", "e"), ("=terminologi", "e"), ("=terorisme", "eə"),
    ("=tersier", "ee"), ("abetes", "eə"), ("ace", "ə"), ("agens", "e"), ("akadem", "e"), ("akreditas", "ə"),
    ("akte", "ə"), ("amelia", "e"), ("amerika", "e"), ("aneka", "e"), ("anekaragam", "e"), ("anime", "e"),
    ("apartemen", "əe"), ("apel", "ə"), ("apes", "ə"), ("are", "ə"), ("arek", "ə"), ("argentina", "e"),
    ("artefak", "ə"), ("atmosfer", "e"), ("babel", "ə"), ("bahtera", "ə"), ("bandel", "ə"), ("bandeng", "ə"),
    ("banget", "ə"), ("barel", "ə"), ("bareng", "ə"), ("barometer", "eə"), ("bebas", "e"), ("bebek", "ee"),
    ("beber", "ee"), ("becak", "e"), ("becek", "ee"), ("beda", "e"), ("begal", "e"), ("bela", "e"), ("belanda", "ə"),
    ("belang", "e"), ("belas", "ə"), ("belet", "əə"), ("belgia", "e"), ("belok", "e"), ("bendera", "əe"),
    ("bengkel", "ee"), ("bengkulu", "ə"), ("benteng", "ee"), ("berfirm", "ə"), ("berken", "əə"), ("besok", "e"),
    ("bet", "ə"), ("beta", "e"), ("bimbel", "ə"), ("biner", "e"), ("boneka", "e"), ("brebes", "əə"),
    ("brengsek", "əə"), ("brunei", "e"), ("ceko", "e"), ("cendol", "e"), ("ceta", "e"), ("cetak", "e"),
    ("chelsea", "e"), ("cilegon", "e"), ("cirebon", "ə"), ("dapet", "ə"), ("dateng", "ə"), ("debat", "e"),
    ("dekan", "e"), ("dekat", "e"), ("dekoratif", "ə"), ("delegas", "eə"), ("demensia", "ee"), ("demo", "e"),
    ("demokrat", "e"), ("demonstr", "e"), ("denah", "e"), ("denpasar", "e"), ("densus", "e"), ("depok", "e"),
    ("depot", "e"), ("deput", "e"), ("deret", "ee"), ("desa", "e"), ("desainer", "eə"), ("desember", "eeə"),
    ("desentralisas", "ee"), ("deta", "e"), ("dewi", "e"), ("dimens", "e"), ("dipes", "ə"), ("dokumentas", "e"),
    ("dokumenter", "ee"), ("ebook", "e"), ("ecer", "ee"), ("edar", "e"), ("eden", "eə"), ("edukas", "ə"),
    ("ekor", "e"), ("ekrut", "e"), ("eksekus", "eə"), ("eksekutif", "eə"), ("eksekutor", "eə"), ("eksperimen", "eee"),
    ("eksperimental", "eəe"), ("ekspres", "ee"), ("ekstens", "ee"), ("ekstrakurikuler", "ee"), ("ekstrem", "ee"),
    ("elepon", "ee"), ("elpon", "e"), ("email", "e"), ("emang", "e"), ("ember", "ee"), ("embrio", "e"),
    ("emirat", "e"), ("emoj", "e"), ("empel", "əə"), ("empiris", "e"), ("enak", "e"), ("encer", "ee"),
    ("endemik", "eə"), ("energ", "ee"), ("energik", "ee"), ("enkrips", "e"), ("ensiklopedia", "ee"), ("ente", "ee"),
    ("entri", "e"), ("episode", "ee"), ("erat", "ə"), ("erjemah", "əə"), ("eropa", "e"), ("esel", "əə"),
    ("esensial", "ee"), ("estate", "ee"), ("eta", "e"), ("fabel", "ə"), ("fase", "ə"), ("federal", "eə"),
    ("fesyen", "eə"), ("fleksibel", "eə"), ("frase", "ə"), ("galer", "e"), ("gamel", "ə"), ("ganteng", "ə"),
    ("gebet", "ee"), ("geger", "ee"), ("geledah", "əe"), ("gema", "e"), ("gender", "əe"), ("generas", "eə"),
    ("generator", "eə"), ("genesis", "ee"), ("genosida", "e"), ("genus", "e"), ("gepeng", "ee"), ("gereja", "əe"),
    ("gese", "ee"), ("gesek", "ee"), ("geser", "ee"), ("globe", "ə"), ("gresik", "ə"), ("halte", "ə"),
    ("helikopter", "eə"), ("hendra", "e"), ("herbal", "e"), ("heroik", "e"), ("herpes", "ee"), ("hostel", "ə"),
    ("identik", "e"), ("imperial", "e"), ("implementas", "əe"), ("impotens", "e"), ("independen", "əeə"),
    ("indonesia", "e"), ("infanter", "ə"), ("instrumental", "e"), ("integral", "ə"), ("integras", "ə"),
    ("internet", "eə"), ("intolerans", "e"), ("jahtera", "ə"), ("jateng", "ə"), ("jeli", "e"), ("jember", "əə"),
    ("jenazah", "ə"), ("jendela", "əe"), ("jenderal", "eə"), ("jenewa", "ee"), ("jengkel", "ee"), ("jerman", "e"),
    ("kalender", "eə"), ("kalteng", "ə"), ("kangen", "ə"), ("karaoke", "e"), ("karate", "e"), ("katedral", "ə"),
    ("kecap", "e"), ("kece", "ee"), ("kediri", "ə"), ("keju", "e"), ("kelok", "e"), ("kendari", "ə"), ("kerang", "e"),
    ("keren", "əə"), ("kernel", "eə"), ("kesah", "e"), ("kolega", "e"), ("komedi", "e"), ("komite", "e"),
    ("kompatibel", "ə"), ("kompetens", "əe"), ("kompetis", "e"), ("kompetitif", "ə"), ("kompresor", "e"),
    ("konsentras", "e"), ("koroner", "e"), ("kristen", "ə"), ("kriteria", "e"), ("kudeta", "e"), ("kuel", "ə"),
    ("kunder", "e"), ("langgeng", "ə"), ("lebak", "e"), ("lebam", "e"), ("lebar", "e"), ("lebaran", "e"),
    ("leceh", "əə"), ("legal", "e"), ("legalitas", "e"), ("legenda", "əe"), ("legendaris", "əe"), ("legiun", "e"),
    ("leher", "ee"), ("lelang", "e"), ("lele", "ee"), ("leleh", "ee"), ("lendang", "e"), ("lengah", "e"),
    ("lengket", "ee"), ("lensa", "e"), ("lentera", "əe"), ("lereng", "ee"), ("les", "ə"), ("lesi", "e"),
    ("lestar", "ə"), ("lestari", "ə"), ("level", "eə"), ("linen", "ə"), ("linier", "e"), ("mabes", "ə"),
    ("macet", "ə"), ("majalengka", "e"), ("manchester", "ee"), ("mantel", "ə"), ("mapel", "ə"), ("maret", "ə"),
    ("marsekal", "ə"), ("mater", "e"), ("material", "e"), ("mebel", "eə"), ("medan", "e"), ("medias", "e"),
    ("mediator", "e"), ("medina", "e"), ("mega", "e"), ("megawati", "e"), ("meja", "e"), ("mekah", "e"),
    ("mekkah", "e"), ("meksiko", "e"), ("memang", "e"), ("memberes", "əee"), ("meme", "ee"), ("mene", "əə"),
    ("menel", "əə"), ("mengaktif", "ə"), ("mengedar", "əe"), ("mengejek", "əəə"), ("mengemb", "əə"),
    ("menghafal", "ə"), ("menghela", "əe"), ("menghemat", "əe"), ("mengklarifikas", "ə"), ("menkes", "əə"),
    ("mental", "e"), ("mentok", "e"), ("mentor", "e"), ("menu", "e"), ("menyeret", "əee"), ("merauke", "ee"),
    ("merek", "eə"), ("merkur", "e"), ("mesir", "ə"), ("mesjid", "ə"), ("meski", "ə"), ("meskipun", "ə"),
    ("mesra", "ə"), ("mesta", "ə"), ("met", "ə"), ("metabolisme", "eə"), ("metafora", "ə"), ("metana", "e"),
    ("meter", "eə"), ("minimarket", "ə"), ("mode", "ə"), ("momen", "ə"), ("momentum", "e"), ("negosias", "e"),
    ("nempel", "ee"), ("nenek", "ee"), ("nepal", "e"), ("netra", "ə"), ("nikel", "ə"), ("nirkabel", "ə"),
    ("november", "eə"), ("obesitas", "e"), ("orde", "ə"), ("otoriter", "e"), ("oven", "ə"), ("overdosis", "ə"),
    ("palembang", "e"), ("pamer", "e"), ("pandem", "e"), ("panger", "e"), ("paramedis", "e"), ("parameter", "eə"),
    ("parlemen", "əe"), ("pecel", "əə"), ("peci", "e"), ("peda", "e"), ("pedal", "e"), ("pede", "ee"),
    ("pedesa", "əe"), ("peka", "e"), ("pelat", "e"), ("pelet", "ee"), ("pena", "e"), ("penalt", "e"),
    ("pencet", "əə"), ("pende", "ee"), ("pendek", "ee"), ("pengedar", "əe"), ("pengen", "ee"), ("penggeledah", "əəe"),
    ("pengin", "e"), ("penguin", "e"), ("penis", "e"), ("pensil", "e"), ("perak", "e"), ("perancis", "ə"),
    ("perang", "e"), ("peron", "e"), ("persik", "e"), ("peru", "e"), ("pesimis", "e"), ("peta", "e"), ("petak", "e"),
    ("petrokimia", "ə"), ("petrus", "ə"), ("peziarah", "ə"), ("pike", "ə"), ("piksel", "ə"), ("polimer", "e"),
    ("ponpes", "ə"), ("portabel", "ə"), ("pose", "ə"), ("preferens", "eəe"), ("prem", "ə"), ("premier", "ee"),
    ("presentas", "ee"), ("primer", "e"), ("problem", "e"), ("proses", "e"), ("proyektil", "ə"), ("puskesmas", "ə"),
    ("ransel", "ə"), ("real", "e"), ("reda", "e"), ("region", "e"), ("regional", "e"), ("register", "eə"),
    ("regulator", "e"), ("reguler", "ee"), ("reka", "e"), ("rekening", "eə"), ("rekomendas", "ee"),
    ("rekonsilias", "e"), ("rekonstruks", "e"), ("rekrutmen", "eə"), ("reksa", "ə"), ("rela", "e"), ("relaksas", "e"),
    ("relas", "e"), ("relaw", "e"), ("relev", "eə"), ("reli", "e"), ("relokas", "e"), ("remeh", "ee"), ("remis", "e"),
    ("rentet", "əə"), ("reparas", "e"), ("republik", "e"), ("reputas", "e"), ("reseptor", "ee"), ("reservas", "əə"),
    ("reserve", "əeə"), ("reses", "ee"), ("residen", "ee"), ("residu", "e"), ("resimen", "ee"), ("resistens", "ee"),
    ("resmi", "ə"), ("resume", "ee"), ("reta", "e"), ("retail", "e"), ("retina", "e"), ("retorika", "e"),
    ("revolusioner", "ee"), ("rezek", "əə"), ("rezeki", "əə"), ("ribet", "ə"), ("sampel", "ə"), ("sebelas", "əə"),
    ("segel", "eə"), ("sekjen", "eə"), ("sekte", "eə"), ("sela", "e"), ("selandia", "e"), ("semak", "e"),
    ("semarang", "ə"), ("semest", "əə"), ("sempet", "əə"), ("sendok", "e"), ("senegal", "ee"), ("sensas", "e"),
    ("sense", "ee"), ("sensei", "ee"), ("sensus", "e"), ("senter", "eə"), ("sentra", "e"), ("sentral", "e"),
    ("september", "eeə"), ("sera", "e"), ("serah", "e"), ("serak", "e"), ("serbia", "e"), ("seri", "e"),
    ("sero", "e"), ("serum", "e"), ("sesi", "e"), ("setan", "e"), ("simpel", "ə"), ("sinerg", "e"), ("singel", "ə"),
    ("sintesis", "e"), ("spektakuler", "ee"), ("stempel", "eə"), ("strateg", "e"), ("subyek", "ə"), ("sukarela", "e"),
    ("sukarelaw", "e"), ("sulsel", "ə"), ("sumsel", "ə"), ("supervis", "ə"), ("suplemen", "əe"), ("suvenir", "ə"),
    ("swedia", "e"), ("tabel", "ə"), ("talenta", "e"), ("tangsel", "ə"), ("tante", "ə"), ("tarekat", "e"),
    ("tega", "e"), ("tegal", "e"), ("tekad", "e"), ("teko", "e"), ("telan", "e"), ("telegram", "ee"),
    ("telekomunikas", "ee"), ("telepat", "ee"), ("telepon", "ee"), ("teleskop", "ee"), ("telkom", "e"),
    ("telpon", "e"), ("tembok", "e"), ("temperamen", "eəe"), ("temperatur", "eə"), ("tempo", "e"), ("tenda", "e"),
    ("tender", "eə"), ("tengger", "ee"), ("tengok", "e"), ("tenis", "e"), ("tensi", "e"), ("tentakel", "eə"),
    ("terkes", "əə"), ("termal", "e"), ("terong", "e"), ("tesis", "e"), ("tetes", "ee"), ("titel", "ə"),
    ("token", "ə"), ("traged", "e"), ("tumpeng", "ə"), ("universal", "ə"), ("universitas", "ə"), ("venezuela", "ee"),
    ("vertikal", "ə"), ("vietnam", "e"), ("visioner", "e"), ("volume", "e"), ("wastafel", "ə"), ("wedang", "e"),
    ("wet", "ə"), ("wewenang", "əə"), ("wikipedia", "e"), ("wortel", "ə"), ("yerusalem", "ee"), ("yesus", "e"),
    ("yohanes", "ə"),
];

fn is_vowel(c: u8) -> bool {
    matches!(c, b'a' | b'e' | b'i' | b'o' | b'u')
}
fn nv(s: &[u8]) -> usize {
    s.iter().filter(|&&c| is_vowel(c)).count()
}

/// Fold the token to ASCII letters (diacritics dropped, apostrophes and digits removed).
fn ascii_letters(w: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(w.len());
    for c in w.chars() {
        let m = match c {
            'a'..='z' => c as u8,
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => b'a',
            'è' | 'é' | 'ê' | 'ë' => b'e',
            'ì' | 'í' | 'î' | 'ï' => b'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => b'o',
            'ù' | 'ú' | 'û' | 'ü' => b'u',
            'ç' => b'c',
            'ñ' => b'n',
            'ß' => b's',
            _ => continue,
        };
        out.push(m);
    }
    out
}

/// Phonemize one lower-case word (no context).
pub fn word(w: &str) -> String {
    // hyphenated compounds and reduplications: each part on its own
    if w.contains('-') && w.chars().any(|c| c.is_alphabetic()) {
        let parts: Vec<String> = w.split('-').map(word).filter(|p| !p.is_empty()).collect();
        return parts.join(" ");
    }
    let lw = ascii_letters(w);
    if lw.is_empty() {
        return String::new();
    }
    let s = std::str::from_utf8(&lw).unwrap_or("");
    if let Some(&(_, full)) = ABBREV.iter().find(|&&(a, _)| a == s) {
        if full != s {
            return full.split(' ').map(word).collect::<Vec<_>>().join(" ");
        }
    }
    if nv(&lw) == 0 {
        return spell_consonants(&lw);
    }
    let schwa = e_pattern(&lw);
    let stress = if UNSTRESSED.contains(&s) {
        Stress::None
    } else if SECONDARY.contains(&s) {
        Stress::Secondary
    } else {
        Stress::Primary
    };
    realize(&lw, &schwa, stress)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stress {
    Primary,
    Secondary,
    None,
}

// ---------------------------------------------------------------- e / ə ----

/// Enclitic stripped before suffix stripping (`-ku`/`-mu` are too often stem-final: temu, beku).
const CLITICS: &[&str] = &["nya"];
/// One derivational suffix.
const SUFFIXES: &[&str] = &["kan", "an", "i"];
/// Derivational prefixes, longest first.
const PREFIXES: &[&str] = &[
    "memper", "diper", "member", "pember", "keber", "keter", "seper", "keper", "sebel", "dipel", "pel", "pem", "pen",
    "peng", "peny", "per", "mem", "men", "meng", "meny", "me", "pe", "ber", "bel", "ter", "ke", "se", "di",
];

/// Length of the word without `-nya` and without one of `-kan`/`-an` (the rules run on this).
fn rule_base(w: &[u8]) -> usize {
    let mut n = w.len();
    for c in CLITICS {
        if w.ends_with(c.as_bytes()) && nv(&w[..n - c.len()]) >= 2 {
            n -= c.len();
            break;
        }
    }
    for s in ["kan", "an"] {
        if w[..n].ends_with(s.as_bytes()) && nv(&w[..n - s.len()]) >= 2 {
            n -= s.len();
            break;
        }
    }
    n
}

/// The word, then with a clitic stripped, then with one suffix stripped (byte lengths).
fn suffix_forms(w: &[u8]) -> Vec<usize> {
    let mut out = vec![w.len()];
    let mut n = w.len();
    for c in CLITICS {
        if w[..n].ends_with(c.as_bytes()) && nv(&w[..n - c.len()]) >= 2 {
            n -= c.len();
            out.push(n);
            break;
        }
    }
    for s in SUFFIXES {
        if w[..n].ends_with(s.as_bytes()) && nv(&w[..n - s.len()]) >= 2 {
            // -i after a vowel is part of the stem (sampai, mulai)
            if *s == "i" && is_vowel(w[n - 2]) {
                continue;
            }
            out.push(n - s.len());
            break;
        }
    }
    out
}

/// Length of one derivational prefix at the start of `w`, if any.
fn peel_prefix(w: &[u8]) -> Option<usize> {
    for p in PREFIXES {
        let pb = p.as_bytes();
        if w.len() <= pb.len() || !w.starts_with(pb) {
            continue;
        }
        let rest = &w[pb.len()..];
        let r0 = rest[0];
        let r1 = rest.get(1).copied().unwrap_or(b'#');
        let ok = match *p {
            // me-/pe- only as mel- mer- mew- mey- (melihat, merasa, mewah? no: 1 vowel)
            "me" | "pe" => matches!(r0, b'l' | b'r' | b'w' | b'y') && is_vowel(r1),
            "pem" | "mem" => matches!(r0, b'b' | b'p' | b'f') || is_vowel(r0),
            "pen" | "men" => matches!(r0, b'd' | b't' | b'c' | b'j' | b's' | b'z') || is_vowel(r0),
            "peng" | "meng" => matches!(r0, b'g' | b'k' | b'h') || is_vowel(r0),
            "peny" | "meny" => is_vowel(r0),
            _ => true,
        };
        if !ok {
            return None;
        }
        if nv(rest) < 2 {
            return None;
        }
        return Some(pb.len());
    }
    None
}

/// Offsets of the stem after peeling 0..3 prefixes (first element is 0).
fn peel_all(w: &[u8]) -> Vec<usize> {
    let mut out = vec![0];
    let mut off = 0;
    for _ in 0..3 {
        match peel_prefix(&w[off..]) {
            Some(n) => {
                off += n;
                out.push(off);
            }
            None => break,
        }
    }
    out
}

fn has_loan_letter(s: &[u8]) -> bool {
    s.iter().any(|&c| matches!(c, b'f' | b'v' | b'z' | b'q'))
}

/// A two-consonant string that is one Indonesian sound or a native cluster (not a loan onset).
fn native_pair(a: u8, b: u8) -> bool {
    matches!(
        (a, b),
        (b'n', b'g') | (b'n', b'y') | (b's', b'y') | (b'k', b'h') | (b'm', b'b') | (b'n', b'd') | (b'n', b't') | (b'n', b'j')
            | (b'n', b'c') | (b'm', b'p') | (b'n', b's') | (b'n', b'k') | (b'g', b'g')
    )
}

/// Reading of the `e` at byte index `i` of `stem` (`prefixed`: a prefix was peeled before it).
fn stem_rule(stem: &[u8], i: usize, prefixed: bool) -> u8 {
    const SCHWA: u8 = b'@';
    const E: u8 = b'e';
    let total = nv(stem);
    let after = nv(&stem[i + 1..]);
    let rest = &stem[i + 1..];
    let pre = &stem[..i];
    let cons_len = rest.iter().take_while(|&&c| !is_vowel(c)).count();
    let cons = &rest[..cons_len];
    if total == 1 {
        return E;
    }
    if after == 0 {
        // final syllable
        if cons.is_empty() {
            if stem.ends_with(b"isme") {
                return SCHWA;
            }
            return if total == 2 { E } else { SCHWA };
        }
        if cons == b"r" || cons == b"m" {
            return SCHWA;
        }
        return E;
    }
    if i == 0 {
        if prefixed {
            return SCHWA;
        }
        // native em-/en-/eng- words (empat, enam, engkau); loans keep e (energi, ember)
        if matches!(rest[0], b'm' | b'n') && !has_loan_letter(stem) && total <= 3 {
            return SCHWA;
        }
        return E;
    }
    let r0 = rest[0];
    let r1 = rest.get(1).copied().unwrap_or(b'#');
    let p1 = pre[pre.len() - 1];
    if (is_vowel(r0) && r0 != b'e') || (is_vowel(p1) && p1 != b'e') {
        return E; // hiatus: daerah, teori, kue
    }
    if r0 == b'w' {
        return E; // lewat, dewan, hewan
    }
    if r0 == b'k' && r1 == b's' {
        return E; // eksekutif, teks
    }
    if r0 == b'h' && is_vowel(r1) && r1 != b'e' {
        return E; // sehat, leher
    }
    if has_loan_letter(stem) {
        return E;
    }
    // onset cluster before e (presiden, tragedi, spesial) -- not across a native pair
    if pre.len() >= 2 && !is_vowel(p1) && !is_vowel(pre[pre.len() - 2]) && (pre.len() == 2 || is_vowel(pre[pre.len() - 3]))
        && !native_pair(pre[pre.len() - 2], p1)
    {
        return E;
    }
    // obstruent cluster after e (teknik, sektor, elektronik)
    if matches!(r0, b'b' | b'd' | b'g' | b'k' | b'p' | b't' | b's' | b'f' | b'z')
        && matches!(r1, b'b' | b'c' | b'd' | b'f' | b'g' | b'j' | b'k' | b'l' | b'm' | b'n' | b'p' | b'r' | b's' | b't' | b'v' | b'z')
        && !native_pair(r0, r1)
    {
        return E;
    }
    SCHWA
}

/// For each byte of `w`: true where a written `e` is schwa (other bytes false).
fn e_pattern(w: &[u8]) -> Vec<bool> {
    let mut schwa = vec![false; w.len()];
    let es: Vec<usize> = (0..w.len()).filter(|&i| w[i] == b'e').collect();
    if es.is_empty() {
        return schwa;
    }
    // table lookup: the whole word (exact keys), then suffix forms x prefix levels
    let mut exact = Vec::with_capacity(w.len() + 1);
    exact.push(b'=');
    exact.extend_from_slice(w);
    if let Ok(k) = E_TABLE.binary_search_by(|&(key, _)| key.as_bytes().cmp(&exact[..])) {
        let pat: Vec<bool> = E_TABLE[k].1.chars().map(|c| c == 'ə').collect();
        if pat.len() == es.len() {
            for (&i, &s) in es.iter().zip(&pat) {
                schwa[i] = s;
            }
            return schwa;
        }
    }
    for &end in &suffix_forms(w) {
        for &off in &peel_all(&w[..end]) {
            let stem = &w[off..end];
            let Ok(k) = E_TABLE.binary_search_by(|&(key, _)| key.as_bytes().cmp(stem)) else { continue };
            let stem_es: Vec<usize> = es.iter().copied().filter(|&i| i >= off && i < end).collect();
            let pat_chars: Vec<bool> = E_TABLE[k].1.chars().map(|c| c == 'ə').collect();
            if pat_chars.len() != stem_es.len() {
                continue;
            }
            for &i in &es {
                schwa[i] = true;
            }
            for (&i, &s) in stem_es.iter().zip(&pat_chars) {
                schwa[i] = s;
            }
            return schwa;
        }
    }
    let base = &w[..rule_base(w)];
    let off = *peel_all(base).last().unwrap_or(&0);
    let stem = &base[off..];
    for &i in &es {
        schwa[i] = i < off || (i < base.len() && stem_rule(stem, i - off, off > 0) == b'@');
    }
    schwa
}

// ------------------------------------------------------------ realization ----

/// Letter names for a vowel-less token (consonant abbreviations).
fn spell_consonants(w: &[u8]) -> String {
    let table = LETTERS.unwrap();
    let mut out = String::new();
    let n = w.len();
    for (k, &c) in w.iter().enumerate() {
        let name = table[(c - b'a') as usize];
        let mark = if k + 1 == n { "ˈ" } else { "ˌ" };
        let pos = name.find(|c: char| "aeiouɛ".contains(c)).unwrap_or(0);
        out.push_str(&name[..pos]);
        out.push_str(mark);
        out.push_str(&name[pos..]);
    }
    out
}

#[derive(Clone, Copy)]
struct Ph {
    s: &'static str,
    vowel: bool,
    schwa: bool,
}
const fn c(s: &'static str) -> Ph {
    Ph { s, vowel: false, schwa: false }
}
const fn v(s: &'static str) -> Ph {
    Ph { s, vowel: true, schwa: false }
}

/// Letters -> phones with espeak's symbols and stress marks.
fn realize(w: &[u8], schwa: &[bool], stress: Stress) -> String {
    let n = w.len();
    let at = |i: usize| -> u8 { if i < n { w[i] } else { b'#' } };
    let mut ph: Vec<Ph> = Vec::with_capacity(n + 2);
    let mut i = 0;
    while i < n {
        let ch = w[i];
        let nx = at(i + 1);
        // doubled consonants: one sound (allah, massa); nng/nny keep the n (lainnya)
        if !is_vowel(ch) && nx == ch && !(ch == b'n' && matches!(at(i + 2), b'g' | b'y')) {
            i += 1;
            continue;
        }
        match ch {
            b'a' => {
                if nx == b'i' {
                    ph.push(v("aɪ"));
                    i += 1;
                } else if nx == b'u' {
                    ph.push(v("aʊ"));
                    i += 1;
                } else {
                    ph.push(v("a"));
                }
            }
            b'e' => {
                if schwa[i] {
                    ph.push(Ph { s: "ə", vowel: true, schwa: true });
                } else {
                    ph.push(v("ɛ"));
                }
            }
            b'i' => ph.push(v("i")),
            b'o' => {
                if nx == b'e' && !(i == 1 && w[0] == b'k') {
                    // old spelling oe = u (Soekarno, Soeharto); ko-e- is a prefix (koefisien)
                    ph.push(v("u"));
                    i += 1;
                } else if nx == b'r' {
                    ph.push(v("ɔ"));
                } else {
                    ph.push(v("o"));
                }
            }
            b'u' => ph.push(v("u")),
            b'b' => {
                if nx == b'h' {
                    i += 1; // bhakti, bhayangkara
                }
                ph.push(c("b"));
            }
            b'c' => {
                if nx == b'h' {
                    i += 1; // ch: English loans (chat) and old spelling
                }
                ph.push(c("tʃ"));
            }
            b'd' => {
                if nx == b'h' {
                    i += 1; // dharma, Yudhoyono
                }
                ph.push(c("d"));
            }
            b'f' => ph.push(c("f")),
            b'g' => {
                if nx == b'h' {
                    i += 1; // ghaib
                }
                ph.push(c("ɡ"));
            }
            b'h' => ph.push(c("h")),
            b'j' => ph.push(c("dʒ")),
            b'k' => {
                if nx == b'h' {
                    ph.push(c("x"));
                    i += 1;
                } else {
                    ph.push(c("k"));
                }
            }
            b'l' => ph.push(c("l")),
            b'm' => ph.push(c("m")),
            b'n' => {
                if nx == b'g' {
                    ph.push(c("ŋ"));
                    i += 1;
                    if at(i + 1) == b'g' {
                        ph.push(c("ɡ"));
                        i += 1;
                    }
                } else if nx == b'y' {
                    ph.push(c("ɲ"));
                    i += 1;
                } else {
                    ph.push(c("n"));
                }
            }
            b'p' => {
                if nx == b'h' {
                    ph.push(c("f"));
                    i += 1;
                } else {
                    ph.push(c("p"));
                }
            }
            b'q' => ph.push(c("k")),
            b'r' => ph.push(c("r")),
            b's' => {
                if nx == b'y' || nx == b'h' {
                    ph.push(c("ç"));
                    i += 1;
                } else {
                    ph.push(c("s"));
                }
            }
            b't' => {
                if nx == b'h' {
                    i += 1; // Thamrin, Athena
                }
                ph.push(c("t"));
            }
            b'v' => ph.push(c("v")),
            b'w' => ph.push(c("w")),
            b'x' => {
                ph.push(c("k"));
                ph.push(c("s"));
            }
            b'y' => ph.push(c("j")),
            b'z' => ph.push(c("z")),
            _ => {}
        }
        i += 1;
    }
    // stress
    let vowels: Vec<usize> = ph.iter().enumerate().filter(|(_, p)| p.vowel).map(|(k, _)| k).collect();
    let nsyl = vowels.len();
    let mut primary = if nsyl >= 2 { nsyl - 2 } else { 0 };
    if nsyl >= 2 && ph[vowels[primary]].schwa && !ph[vowels[nsyl - 1]].schwa {
        primary = nsyl - 1;
    }
    let mut marks: Vec<&'static str> = vec![""; nsyl];
    match stress {
        Stress::None => {}
        Stress::Secondary => marks[primary] = "ˌ",
        Stress::Primary => {
            marks[primary] = "ˈ";
            // secondary stress on every second non-schwa syllable before the primary
            let mut count = 0;
            for k in (0..primary).rev() {
                if ph[vowels[k]].schwa {
                    continue;
                }
                count += 1;
                if count % 2 == 0 {
                    marks[k] = "ˌ";
                }
            }
        }
    }
    let mut out = String::with_capacity(n * 2 + 4);
    let mut k = 0;
    for p in &ph {
        if p.vowel {
            out.push_str(marks[k]);
            k += 1;
        }
        out.push_str(p.s);
    }
    out
}

/// Sentence pass: nothing cross-word in Indonesian.
pub fn join(_outs: &mut [Out]) {}

// ---------------------------------------------------------------- numbers ----

const ONES: [&str; 10] = ["nol", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh", "delapan", "sembilan"];

fn below_1000(n: u128, out: &mut Vec<String>) {
    let h = n / 100;
    let r = n % 100;
    if h == 1 {
        out.push("seratus".into());
    } else if h > 1 {
        out.push(ONES[h as usize].into());
        out.push("ratus".into());
    }
    if r == 0 {
        if n == 0 {
            out.push("nol".into());
        }
    } else if r < 10 {
        out.push(ONES[r as usize].into());
    } else if r == 10 {
        out.push("sepuluh".into());
    } else if r == 11 {
        out.push("sebelas".into());
    } else if r < 20 {
        out.push(ONES[(r - 10) as usize].into());
        out.push("belas".into());
    } else {
        out.push(ONES[(r / 10) as usize].into());
        out.push("puluh".into());
        if r % 10 > 0 {
            out.push(ONES[(r % 10) as usize].into());
        }
    }
}

/// Cardinal number words (seratus, seribu, dua puluh satu, satu juta).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec!["nol".into()];
    }
    let mut out = Vec::new();
    const SCALES: [(u128, &str); 5] = [
        (1_000_000_000_000_000, "kuadriliun"),
        (1_000_000_000_000, "triliun"),
        (1_000_000_000, "miliar"),
        (1_000_000, "juta"),
        (1_000, "ribu"),
    ];
    let mut rest = n;
    for (val, name) in SCALES {
        let q = rest / val;
        if q > 0 {
            if q == 1 && val == 1_000 {
                out.push("seribu".into());
            } else {
                below_1000(q, &mut out);
                out.push(name.into());
            }
            rest %= val;
        }
    }
    if rest > 0 {
        below_1000(rest, &mut out);
    }
    out
}

pub fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        '%' => "persen",
        '$' => "dolar",
        '€' => "euro",
        '£' => "paun sterling",
        '&' => "dan",
        '+' => "plus",
        '@' => "at",
        '°' => "derajat",
        '=' => "sama dengan",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{cardinal, word};

    #[test]
    fn consonants_and_digraphs() {
        for (w, p) in [
            ("cara", "tʃˈara"),
            ("juga", "dʒˈuɡa"),
            ("banyak", "bˈaɲak"),
            ("orang", "ˈɔraŋ"),
            ("menggunakan", "məŋɡunˈakan"),
            ("bangun", "bˈaŋun"),
            ("syarat", "çˈarat"),
            ("khusus", "xˈusus"),
            ("saya", "sˈaja"),
            ("tidak", "tˈidak"),
            ("allah", "ˈalah"),
            ("taksi", "tˈaksi"),
            ("zaman", "zˈaman"),
            ("video", "vidˈɛo"),
            ("film", "fˈilm"),
            ("quran", "kˈuran"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn vowels_and_diphthongs() {
        for (w, p) in [
            ("pulau", "pˈulaʊ"),
            ("sampai", "sˈampaɪ"),
            ("amboi", "ambˈoi"),
            ("koboi", "kobˈoi"),
            ("dia", "dˈia"),
            ("dunia", "dunˈia"),
            ("keluarga", "kəluˈarɡa"),
            ("saat", "sˈaat"),
            ("kota", "kˈota"),
            ("nomor", "nˈomɔr"),
            ("korban", "kˈɔrban"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn schwa_vs_e() {
        for (w, p) in [
            ("mereka", "mərˈɛka"),
            ("memang", "mˈɛmaŋ"),
            ("dekat", "dˈɛkat"),
            ("kesehatan", "kəsɛhˈatan"),
            ("sehingga", "səhˈiŋɡa"),
            ("memperoleh", "məmpərˈolɛh"),
            ("tersebut", "tərsəbˈut"),
            ("sumber", "sˈumbər"),
            ("presiden", "prɛsˈidɛn"),
            ("ekonomi", "ˌɛkonˈomi"),
            ("empat", "əmpˈat"),
            ("enam", "ənˈam"),
            ("lewat", "lˈɛwat"),
            ("keren", "kˈərən"),
            ("teknologi", "tˌɛknolˈoɡi"),
            ("sepeda", "səpˈɛda"),
            ("proses", "prˈosɛs"),
            ("oleh", "ˈolɛh"),
            ("kakek", "kˈakɛk"),
            ("sistem", "sˈistəm"),
            ("mengerti", "məŋərtˈi"),
            ("penelitian", "pənəlitˈian"),
            ("ke", "kə"),
            ("perempuan", "pərəmpˈuan"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn stress() {
        for (w, p) in [
            ("telah", "təlˈah"),
            ("benar", "bənˈar"),
            ("beberapa", "bəbərˈapa"),
            ("manusia", "mˌanusˈia"),
            ("universitas", "ˌunivərsˈitas"),
            ("memperhatikan", "məmpərhatˈikan"),
            ("internasional", "intərnˌasiˈonal"),
            ("yang", "jaŋ"),
            ("dari", "dˌari"),
            ("rumahnya", "rumˈahɲa"),
            ("orang-orang", "ˈɔraŋ ˈɔraŋ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn abbreviations() {
        assert_eq!(word("yg"), "jaŋ");
        assert_eq!(word("dll"), "dan lˈaɪn lˈaɪn");
        assert_eq!(word("tv"), "tˌefˈe");
        assert_eq!(word("rp"), "rupˈiah");
    }

    #[test]
    fn table_sorted() {
        for w in super::E_TABLE.windows(2) {
            assert!(w[0].0 < w[1].0, "{} >= {}", w[0].0, w[1].0);
        }
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), vec!["nol"]);
        assert_eq!(cardinal(11), vec!["sebelas"]);
        assert_eq!(cardinal(15).join(" "), "lima belas");
        assert_eq!(cardinal(21).join(" "), "dua puluh satu");
        assert_eq!(cardinal(100), vec!["seratus"]);
        assert_eq!(cardinal(1000), vec!["seribu"]);
        assert_eq!(cardinal(1500).join(" "), "seribu lima ratus");
        assert_eq!(cardinal(2024).join(" "), "dua ribu dua puluh empat");
        assert_eq!(cardinal(100_000).join(" "), "seratus ribu");
        assert_eq!(cardinal(1_000_000).join(" "), "satu juta");
        assert_eq!(cardinal(2_000_000_000).join(" "), "dua miliar");
    }
}
