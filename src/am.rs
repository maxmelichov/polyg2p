//! Amharic grapheme-to-phoneme: Ge'ez script (fidel) to IPA. Rules only.
//!
//! Fidel is a syllabary whose code points are laid out as consonant row x vowel
//! order, so the letter-to-sound part is a table lookup:
//!
//! * A fidel is `consonant + vowel`. The seven orders are `ə u i a e (none) o`;
//!   the eighth form of a row is the labialized `Cʷa` (`ሏ` → `lʷa`).
//! * The first order of the gutturals `ሀ ሐ ኀ አ ዐ` is `a`, not `ə`.
//! * `ሀ ሐ ኀ ኸ` are all `h`; `ሰ ሠ` are `s`; `ጸ ፀ` are `sʼ`. Ejectives are written
//!   with the modifier apostrophe: `pʼ tʼ kʼ sʼ tʃʼ`.
//! * `አ` and `ዐ` carry a vowel. No glottal stop is written at the start of a word;
//!   inside a word after a vowel it is `ʔ` (`ሰዓት` → `səʔat`).
//! * The sixth order is a bare consonant. The epenthetic vowel `ɨ` comes from a
//!   left-to-right syllabification, syllable = (C)V(C), with CVCC only at the end
//!   of the word: a bare consonant after a vowel is a coda; after a consonant or
//!   at the start of the word it opens a syllable and takes `ɨ`. The last
//!   consonant of the word joins the coda when sonority does not rise
//!   (`ልብስ` → `lɨbs`, `አንድ` → `and`); otherwise `ɨ` goes between the two
//!   (`ክፍል` → `kɨfɨl`, `እግር` → `ɨɡɨr`). `h` never opens a cluster (`እኅት` →
//!   `ɨhɨt`).
//! * `n m ɲ h ʃ` never close a cluster: at the end of a word they are nearly
//!   always the suffixes `-ን -ም -ኝ -ህ -ሽ`, which take `ɨ` after a consonant
//!   (`ያለውን` → `jaləwɨn`, `ወይም` → `wəjɨm`, `ቤትህ` → `betɨh`, `ይስጥልኝ` →
//!   `jɨstʼɨlɨɲ`). The words whose stem ends in such a cluster are listed in
//!   `CLUSTER` (`ዓይን` → `ajn`, `ፊልም` → `film`, loans such as `ኦንላይን`).
//! * The subject endings of the perfective are not those suffixes: 1pl `-n`,
//!   2fsg `-ʃ` and 2msg `-h` are bare consonants (`ሰበርን` *säbbärn*, `ሰበርሽ`
//!   *säbbärš*). A verbal prefix + a stem of `PERF` + `ን ሽ ህ` is read by
//!   sonority alone: `ነበርን` → `nəbərn`, `ካልን` → `kaln`, `ነበርሽ` → `nəbərʃ`.
//! * A bare `t` before `ʃ`, a bare `d` before `ʒ` or `z`, and a bare `t` before
//!   a word-final `s` takes `ɨ` (`ቤትሽ` → `betɨʃ`). Written next to each other
//!   the two would be folded into the affricate (`fold.rs`), and `ቤትሽ` would
//!   come out the same as `ቤች`.
//! * `STEMS`: a few stems whose `ɨ` stands next to a long consonant, which
//!   the script does not write, keep their shape before a suffix (`አምስት` →
//!   `amɨst`, so `አምስተኛ` → `amɨstəɲa`).
//!
//! Gemination: fidel does not write it. It is lexical for nouns and is added only by the optional packs
//! (word table `am-gem.tsv`, verb rules `am-verbs.tsv`, see [`Gem`]); without them none is predicted.
//! The `ɨ` rule works on the written word, so a one-letter prefix is syllabified
//! together with its stem.
//!
//! Text goes through the shared tokenizer after [`marks`] has mapped Ethiopic
//! punctuation (`። ፣ ፤ ፥ ፦ ፧ ፡`) to the kept marks and turned everything the
//! module does not read into a space. [`join`] then makes sure no mark is left
//! standing alone.

use crate::Out;

pub const IMPLEMENTED: bool = true;
pub const DECIMAL: &str = "ነጥብ";
pub const LETTERS: Option<&[&str; 26]> = None;

/// Consonant of each regular row (first code point of the row).
fn row(r: u32) -> Option<&'static str> {
    Some(match r {
        0x1200 | 0x1210 | 0x1280 | 0x12B8 => "h",
        0x1208 => "l",
        0x1218 => "m",
        0x1220 | 0x1230 => "s",
        0x1228 => "r",
        0x1238 => "ʃ",
        0x1240 | 0x1250 => "kʼ",
        0x1260 => "b",
        0x1268 => "v",
        0x1270 => "t",
        0x1278 => "tʃ",
        0x1290 => "n",
        0x1298 => "ɲ",
        0x12A0 | 0x12D0 => "ʔ",
        0x12A8 => "k",
        0x12C8 => "w",
        0x12D8 => "z",
        0x12E0 => "ʒ",
        0x12E8 => "j",
        0x12F0 | 0x12F8 => "d",
        0x1300 => "dʒ",
        0x1308 => "ɡ",
        0x1318 => "ŋ",
        0x1320 => "tʼ",
        0x1328 => "tʃʼ",
        0x1330 => "pʼ",
        0x1338 | 0x1340 => "sʼ",
        0x1348 => "f",
        0x1350 => "p",
        _ => return None,
    })
}

/// Labialized series `ቈ ኈ ኰ ጐ` (and the Tigrinya `ቘ ዀ`): `ʷə . ʷi ʷa ʷe ʷɨ`.
fn lab_row(r: u32) -> Option<&'static str> {
    Some(match r {
        0x1248 | 0x1258 => "kʼʷ",
        0x1288 | 0x12C0 => "hʷ",
        0x12B0 => "kʷ",
        0x1310 => "ɡʷ",
        _ => return None,
    })
}

const ORDERS: [&str; 7] = ["ə", "u", "i", "a", "e", "", "o"];

/// One fidel as (consonant, labialized, vowel). An empty vowel is the sixth
/// order (bare consonant); `labialized` is the eighth form `Cʷa`. `None` for
/// anything that is not an Amharic syllable letter.
pub fn fidel(ch: char) -> Option<(&'static str, bool, &'static str)> {
    match ch {
        'ኧ' => return Some(("ʔ", false, "ə")),
        'ፘ' => return Some(("rj", false, "a")),
        'ፙ' => return Some(("mj", false, "a")),
        'ፚ' => return Some(("fj", false, "a")),
        'ሇ' | 'ኇ' => return Some(("h", false, "o")),
        'ቇ' => return Some(("kʼ", false, "o")),
        'ኯ' => return Some(("k", false, "o")),
        'ዏ' => return Some(("w", false, "o")),
        'ዯ' => return Some(("j", false, "o")),
        'ጏ' => return Some(("ɡ", false, "o")),
        'ፇ' => return Some(("sʼ", false, "o")),
        _ => {}
    }
    let o = ch as u32;
    let (r, k) = (o & !7, (o & 7) as usize);
    if let Some(c) = lab_row(r) {
        let v = match k {
            0 => "ə",
            2 => "i",
            3 => "a",
            4 => "e",
            5 => "",
            _ => return None,
        };
        return Some((c, false, v));
    }
    // unassigned code points inside the rows below
    if matches!(o, 0x1257 | 0x12BF | 0x12D7) {
        return None;
    }
    let c = row(r)?;
    if k == 7 {
        return Some((c, true, "a"));
    }
    let guttural = matches!(r, 0x1200 | 0x1210 | 0x1280 | 0x12A0 | 0x12D0);
    Some((c, false, if guttural && k == 0 { "a" } else { ORDERS[k] }))
}

fn sonority(c: &str) -> u8 {
    match c.trim_end_matches('ʷ') {
        "w" | "j" => 4,
        "l" | "r" => 3,
        "m" | "n" | "ɲ" | "ŋ" => 2,
        _ => 1,
    }
}

/// Would `fold.rs` write `c1 c2` as one affricate letter wherever they meet?
fn folds(c1: &str, c2: &str) -> bool {
    matches!((c1, c2), ("t", "ʃ") | ("d", "ʒ") | ("d", "z"))
}

/// The consonants of the suffixes `-ን -ም -ኝ -ህ -ሽ`. They take `ɨ` after a
/// consonant, so none of them is the second consonant of a word-final cluster.
fn no_coda(c: &str) -> bool {
    matches!(c, "n" | "m" | "ɲ" | "h" | "ʃ")
}

/// May the word end in the cluster `c1 c2` by sonority alone? It must not rise,
/// and two sonorants need a fall. `h` does not open a cluster, and neither
/// consonant may be part of a pair the affricate fold would merge (`ts` is
/// folded at the end of a word).
fn son_ok(c1: &str, c2: &str) -> bool {
    if c1 == c2 || c1 == "h" || folds(c1, c2) || (c1, c2) == ("t", "s") {
        return false;
    }
    let (s1, s2) = (sonority(c1), sonority(c2));
    if s2 > 1 { s1 > s2 } else { s1 >= s2 }
}

/// May the word end in the cluster `c1 c2` without `ɨ`? [`son_ok`], and
/// `n m ɲ h ʃ` do not close one: [`CLUSTER`] lists the words that do end in
/// one, [`perfective`] finds the verb forms that do.
fn coda_ok(c1: &str, c2: &str) -> bool {
    !no_coda(c2) && son_ok(c1, c2)
}

/// One-letter proclitics. A [`CLUSTER`] word or a [`STEMS`] entry may follow one.
const PROCLITICS: [(char, &str); 4] = [('የ', "jə"), ('በ', "bə"), ('ለ', "lə"), ('ከ', "kə")];

/// Words that end in a lexical cluster with `n` or `m`, where the suffix rule
/// would write `ɨ`. Sources: `ዓይን` `a j n`, `ሕልም` `h ɨ l m`, `ፊልም` `f i l m` in
/// Wiktionary's pronunciations (WikiPron `amh_ethi_broad`); `ወይን` `wäyn` in
/// Wiktionary's entry; `ዲዛይን`, `ኦንላይን`, `ዳውን`, `ታይም`, `ፎርም`, `ሪፎርም`,
/// `ፕላትፎርም` are the English *design*, *online*, *down*, *time*, *form*,
/// *reform*, *platform*; `የሚገርም` is the type A imperfect of `ገረመ` (`yɨgərm`).
///
/// Second group: more forms of type A verbs whose last radical is `m` (the
/// imperfect and jussive stem ends in the last two radicals, as in `ይሰብር`
/// *yəsäbr*): `ገረመ`, `አረመ`, `ፈረመ`. `እውን` is the stem of `እውነት` (`ɨwnət`);
/// `ማርሽ` is the Italian *marcia*.
///
/// Third group: loans and names seen in running text, all with a final
/// cluster in the source language: *design*, *online* (three spellings),
/// *headline*, *decline*, *Caroline*, *Valentine*, *Byzantine* (two
/// spellings), *-stein*, *define*, *join*, *cocaine*, *bitcoin*, *examine*,
/// *explain*, *Ukraine*, *pain*, *wine*, *mine*, *Rhine*, *town*, *horn*,
/// *learn*, *pattern*, *Melbourne*, *concern*, *sovereign*, *this time*,
/// *last time*, *crime*, *I'm*, *Stockholm* (two spellings), *confirm*,
/// *inform*, *term*, *brush*.
const CLUSTER: &[&str] = &[
    "ዓይን", "አይን", "ዐይን", "ሕልም", "ህልም", "ፊልም", "ወይን", "ዲዛይን", "ኦንላይን", "ዳውን", "ታይም", "ፎርም", "ሪፎርም", "ፕላትፎርም", "የሚገርም", "በሚገርም", "ሲገርም",
    "ይገርም", "አይገርም", "ወደሚገርም", "እንደሚገርም", "እናርም", "እንዳትፈርም", "እውን", "ዕውን", "ማርሽ",
    "ድዛይን", "አንላይን", "ኦላይን", "ሄድላይን", "ዲክላይን", "ካሮላይን", "ቫላንታይን", "ባይዛንታይን", "ባዛንታይን", "ሽታይን", "ዲፋይን", "ጆይን", "ኮካይን", "ቢትኮይን",
    "ኤግዛማይን", "ኤክስፕሌይን", "ዩክሬይን", "ፔይን", "ዋይን", "ማይን", "ራይን", "ታውን", "ሆርን", "ለርን", "ፓተርን", "ሜልበርን", "ኮንሰርን", "ሶቬርን",
    "ዚስታይም", "ላስታይም", "ክራይም", "አይም", "ስቶክሆልም", "ስቶኮልም", "ኮንፈርም", "ኢንፎርም", "ተርም", "ቡርሽ",
];

/// Perfective stems. The subject endings of the perfective are consonants:
/// 1pl `-n`, 2fsg `-ʃ`, 2msg `-h` (`ሰበርን` *säbbärn*, `ሰበርሽ` *säbbärš*), unlike
/// the accusative `-(ə)n`, the possessives `-(ə)h` `-(ə)š` and the object
/// suffixes, which take the vowel after a consonant. The script does not tell
/// a perfective stem from a noun (`ነገርን` is "the thing" and "we told"), so the
/// stems are listed: verbs met with these endings in running text, none of
/// them also a noun. A verb that is not listed is read by the suffix rule.
///
/// * in `r`: *was*, *begin*, *live*, *try*, *change*, *add*, *speak*, *talk
///   together*, *learn*, *learn from each other*, *teach*, *cross*, *carry
///   out*, *celebrate*, *combine*, *be created*;
/// * in `l`: *say*, *can*, *be called*, *pay*, *share*, *continue*, *follow*,
///   *receive*, *ring*, *notice*, *ripen*;
/// * others, for `-ʃ`: *do*, *know*, *think*, *present*, *like*, *explain*.
const PERF: [&str; 33] = [
    "ነበር", "ጀመር", "ኖር", "ሞከር", "ቀየር", "ጨመር", "ተናገር", "ተነጋገር", "ተማር", "ተማማር", "አስተማር", "ተሻገር", "ተገበር", "አከበር", "አጣመር", "ተፈጠር",
    "አል", "ቻል", "ተባል", "ከፈል", "ተካፈል", "ቀጠል", "ተከተል", "ተቀበል", "ደወል", "አስተዋል", "በሰል", "አደረግ", "አወቅ", "አሰብ", "አቀረብ", "ወደድ", "ገለጽ",
];

/// What may stand before a [`PERF`] stem: nothing, the relative `የ`, the
/// conjunctions `ከ እየ ስለ እንደ እስከ`, and the negative `አል` alone or after one
/// of them.
const HEADS: [&str; 13] = ["", "የ", "ከ", "እየ", "ስለ", "እንደ", "እስከ", "አል", "ያል", "ካል", "ባል", "ስላል", "እንዳል"];
/// The same before a stem that starts with `አ`, which merges with them
/// (`እየ` + `አስተማር` → `እያስተማር`).
const HEADS_A: [&str; 12] = ["አ", "ያ", "ካ", "እያ", "ስላ", "እንዳ", "እስካ", "አላ", "ያላ", "ካላ", "ስላላ", "እንዳላ"];

const T_ROW: &str = "ትተቱቲታቴቶቷ";

/// Stems read as listed when one of the letters in `.1` follows: (letters, next
/// letter, phonemes, also after a proclitic). The `ɨ` of these stands next to a
/// long consonant, which the script does not write, so the syllabifier cannot
/// place it once a suffix follows.
///
/// * The numerals `አምስት` `ስድስት` `ስምንት` (`a mː ɨ s t`, `s ɨ dː ɨ s t`,
///   `s ɨ mː ɨ n t` in Wiktionary's pronunciations; `ስድስተኛ` is `səddəstäñña`
///   in its entry): `አምስተኛ` → `amɨstəɲa`, `ስድስቱ` → `sɨdɨstu`.
/// * `ትልቅ` (`təlləḳ` in Wiktionary's entry): `tɨlɨkʼ`, `ትልቁ` → `tɨlɨkʼu`.
/// * `ክርስት-` at the start of a word (`ክርስቲያን` is `krəstiyan` in Wiktionary's
///   entry, `ክርስትና` `k r ɨ s t ɨ nː a` in its pronunciations): `ክርስቶስ` →
///   `kɨrɨstos`, like `ቤተክርስቲያን` → `betəkrɨstijan`. After a proclitic the
///   syllabifier already gives `-krɨs-`.
const STEMS: [(&str, &str, &[&str], bool); 5] = [
    ("አምስ", T_ROW, &["a", "m", "ɨ", "s"], true),
    ("ስድስ", T_ROW, &["s", "ɨ", "d", "ɨ", "s"], true),
    ("ስምን", T_ROW, &["s", "ɨ", "m", "ɨ", "n"], true),
    ("ትል", "ቅቁቋ", &["t", "ɨ", "l", "ɨ"], true),
    ("ክርስ", T_ROW, &["k", "ɨ", "r", "ɨ", "s"], false),
];

fn is_vowel(p: &str) -> bool {
    matches!(p, "ə" | "u" | "i" | "a" | "e" | "ɨ" | "o")
}

/// Does `w` start with `p`, counted in letters?
fn starts(w: &[char], p: &str) -> bool {
    let mut n = 0;
    p.chars().all(|c| {
        n += 1;
        w.get(n - 1) == Some(&c)
    })
}

/// Is `w` one of the [`CLUSTER`] words, with or without a proclitic?
fn cluster_word(w: &[char]) -> bool {
    let listed = |w: &[char]| CLUSTER.iter().any(|x| x.chars().count() == w.len() && starts(w, x));
    listed(w) || (w.first().is_some_and(|c| PROCLITICS.iter().any(|(p, _)| p == c)) && listed(&w[1..]))
}

/// Is `w` a verbal prefix ([`HEADS`]) + a [`PERF`] stem + one of the subject
/// endings `ን ሽ ህ`?
fn perfective(w: &[char]) -> bool {
    let Some((end, body)) = w.split_last() else { return false };
    if !matches!(end, 'ን' | 'ሽ' | 'ህ') {
        return false;
    }
    PERF.iter().any(|s| {
        let merged = s.starts_with('አ');
        let tail: Vec<char> = s.chars().skip(usize::from(merged)).collect();
        let Some(k) = body.len().checked_sub(tail.len()) else { return false };
        let heads: &[&str] = if merged { &HEADS_A } else { &HEADS };
        body[k..] == tail[..] && heads.iter().any(|h| h.chars().count() == k && starts(body, h))
    })
}

/// The [`STEMS`] entry the word starts with, with or without a proclitic:
/// (letters consumed, their phonemes). `(0, [])` when there is none.
fn stem(w: &[char]) -> (usize, Vec<&'static str>) {
    let pre = PROCLITICS.iter().find(|(c, _)| w.first() == Some(c)).map(|&(_, ph)| ph);
    // at the start of the word first, then after its proclitic
    for skip in 0..=usize::from(pre.is_some()) {
        for (s, next, ph, proclitic) in STEMS {
            let k = skip + s.chars().count();
            if (proclitic || skip == 0) && starts(&w[skip..], s) && w.get(k).is_some_and(|c| next.contains(*c)) {
                let mut out = Vec::with_capacity(ph.len() + 2);
                if let (1, Some(pre)) = (skip, pre) {
                    out.push(pre);
                    // hiatus after the proclitic: በአምስት → bəʔamɨst
                    if is_vowel(ph[0]) {
                        out.push("ʔ");
                    }
                }
                out.extend_from_slice(ph);
                return (k, out);
            }
        }
    }
    (0, Vec::new())
}

/// Letters of a written word: Ethiopic, and the Ethiopic Supplement, Extended
/// and Extended-A blocks. Only the first is read ([`fidel`]); a letter of the
/// other three is skipped without splitting the word.
fn in_run(c: char) -> bool {
    matches!(c as u32, 0x1200..=0x135A | 0x1380..=0x138F | 0x2D80..=0x2DDF | 0xAB01..=0xAB2E) && c.is_alphabetic()
}

/// Phonemes of one run of fidel (one written word). `gem[i]` marks the i-th
/// letter long (gemination layer, see [`Gem`]); no flags = the plain reading.
fn run(w: &str, gem: &[bool], out: &mut String) {
    let letters: Vec<char> = w.chars().filter(|&c| fidel(c).is_some()).collect();
    let seg: Vec<(&str, bool, &str)> = letters.iter().filter_map(|&c| fidel(c)).collect();
    let n = seg.len();
    let bare = |i: usize| seg[i].0 != "ʔ" && seg[i].2.is_empty();
    let ok = guards(&letters);
    // a flag stands only where the guards allow a long consonant
    let g: Vec<bool> = (0..n).map(|i| gem.get(i).copied().unwrap_or(false) && ok[i]).collect();
    let (mut k0, mut ph) = stem(&letters);
    // the stem table is skipped when a flag falls on a stem letter or on the letter right after it
    if (0..=k0.min(n.saturating_sub(1))).any(|i| g.get(i).copied().unwrap_or(false)) {
        k0 = 0;
        ph.clear();
    }
    ph.reserve(3 * n);
    // consonants since the last vowel
    let mut coda = u8::from(ph.last().is_some_and(|p| !is_vowel(p)));
    // may the last two consonants end the word as a cluster?
    let fin = n >= 2 && bare(n - 2) && bare(n - 1) && {
        let (c1, c2) = (seg[n - 2].0, seg[n - 1].0);
        // a perfective form ends in a cluster wherever sonority allows one
        // (ነበርን → nəbərn); with -h only after l and r (ነበርህ → nəbərh)
        coda_ok(c1, c2) || cluster_word(&letters) || (perfective(&letters) && son_ok(c1, c2) && (c2 != "h" || sonority(c1) == 3))
    };
    for i in k0..n {
        let (c, lab, v) = seg[i];
        let last = i + 1 == n;
        let m = if g[i] { "ː" } else { "" };
        let gn = !last && g[i + 1];
        if c == "ʔ" && !lab {
            // vowel carrier (sixth order እ = ɨ); ʔ only in hiatus inside the word
            if !ph.is_empty() && coda == 0 {
                ph.push("ʔ");
            }
            // ረቡዕ → rəbuʔ: no ɨ after a word-final carrier that follows a vowel
            if !(last && v.is_empty() && !ph.is_empty() && coda == 0) {
                ph.push(if v.is_empty() { "ɨ" } else { v });
            }
            coda = 0;
        } else if !v.is_empty() {
            ph.push(c);
            if lab {
                ph.push("ʷ");
            }
            if g[i] {
                ph.push(m);
            }
            ph.push(v);
            coda = 0;
        } else if !last && (ph.is_empty() || coda > 0) {
            // opens a syllable
            ph.push(c);
            if g[i] {
                ph.push(m);
            }
            ph.push("ɨ");
            coda = 0;
        } else if !last
            && ((i + 3 == n && fin)
                || (seg[i + 1].0 == "ɲ" && !seg[i + 1].1 && !seg[i + 1].2.is_empty())
                || folds(c, seg[i + 1].0)
                // R2: a long bare consonant opens a syllable too (ምሽት → mɨʃːɨt), except before a glide (ኢትዮጵያ → itjopʼːja)
                || (g[i] && !(!seg[i + 1].1 && !seg[i + 1].2.is_empty() && matches!(seg[i + 1].0, "j" | "w")))
                // R1: a bare consonant before a long one takes ɨ (ሀብታምነት → habtamɨnːət)
                || gn)
        {
            // after a vowel, but it still opens a syllable: the final cluster comes
            // first (ስምንት → sɨmɨnt), ɲ is long before a vowel (ማንኛው → manɨɲaw), and
            // t + ʃ must not read as the affricate (ቤትሻ → betɨʃa)
            ph.push(c);
            if g[i] {
                ph.push(m);
            }
            ph.push("ɨ");
            coda = 0;
        } else if !last || coda == 0 {
            // coda after a vowel (a word that is one bare consonant stays bare)
            ph.push(c);
            if g[i] {
                ph.push(m);
            }
            coda = 1;
        } else if coda == 1 && fin {
            ph.push(c);
            coda = 2;
        } else {
            ph.push("ɨ");
            ph.push(c);
            coda = 1;
        }
    }
    for p in ph {
        out.push_str(p);
    }
}

/// Where a length mark may stand: not word-initial, not on a vowel carrier,
/// not on `h` (any row of it, labialized too), not on a word-final bare letter.
pub fn guards(letters: &[char]) -> Vec<bool> {
    let n = letters.len();
    letters
        .iter()
        .enumerate()
        .map(|(i, &ch)| {
            let (c, _lab, v) = fidel(ch).unwrap_or(("ʔ", false, ""));
            i > 0 && c != "ʔ" && c.trim_end_matches('ʷ') != "h" && !(i + 1 == n && v.is_empty())
        })
        .collect()
}

/// The one-to-one homophone fold the gemination table is keyed by
/// (ሠ → ሰ, ጸ → ፀ, ዐ ዓ ኣ → አ, the h rows → ሀ): positions are kept.
pub fn fold(c: char) -> char {
    match c {
        'ሠ' => 'ሰ', 'ሡ' => 'ሱ', 'ሢ' => 'ሲ', 'ሣ' => 'ሳ', 'ሤ' => 'ሴ', 'ሥ' => 'ስ', 'ሦ' => 'ሶ',
        'ጸ' => 'ፀ', 'ጹ' => 'ፁ', 'ጺ' => 'ፂ', 'ጻ' => 'ፃ', 'ጼ' => 'ፄ', 'ጽ' => 'ፅ', 'ጾ' => 'ፆ',
        'ዐ' => 'አ', 'ዑ' => 'ኡ', 'ዒ' => 'ኢ', 'ዓ' => 'አ', 'ዔ' => 'ኤ', 'ዕ' => 'እ', 'ዖ' => 'ኦ', 'ኣ' => 'አ',
        'ሃ' => 'ሀ', 'ሐ' => 'ሀ', 'ሑ' => 'ሁ', 'ሒ' => 'ሂ', 'ሓ' => 'ሀ', 'ሔ' => 'ሄ', 'ሕ' => 'ህ', 'ሖ' => 'ሆ',
        'ኀ' => 'ሀ', 'ኁ' => 'ሁ', 'ኂ' => 'ሂ', 'ኃ' => 'ሀ', 'ኄ' => 'ሄ', 'ኅ' => 'ህ', 'ኆ' => 'ሆ',
        'ኸ' => 'ሀ', 'ኹ' => 'ሁ', 'ኺ' => 'ሂ', 'ኻ' => 'ሀ', 'ኼ' => 'ሄ', 'ኽ' => 'ህ', 'ኾ' => 'ሆ',
        _ => c,
    }
}

/// Optional Amharic gemination layer: an unwritten long consonant is a property of
/// the word, not of the script, so it needs data. One optional file in the data
/// directory: `am-gem.tsv`, `word<TAB>word with U+135F after each long letter`.
/// A word found in the table gets its marks; every other word keeps the plain
/// reading. `$POLYG2P_AM_GEM=0` switches the layer off.
///
/// Without the table nothing changes: [`Gem::apply`] returns at once.
pub struct Gem {
    table: Option<std::collections::HashMap<String, Vec<bool>>>,
    verbs: Option<Verbs>,
}

impl Gem {
    pub fn load(data_dir: Option<&std::path::Path>) -> Self {
        if std::env::var_os("POLYG2P_AM_GEM").is_some_and(|v| v == "0") {
            return Gem { table: None, verbs: None };
        }
        let table = data_dir.and_then(|d| std::fs::read_to_string(d.join("am-gem.tsv")).ok()).map(|t| Self::parse_table(&t));
        let verbs = data_dir.and_then(|d| std::fs::read_to_string(d.join("am-verbs.tsv")).ok()).map(|t| Verbs::parse(&t));
        Gem { table, verbs }
    }

    pub fn from_table(table: Option<&str>) -> Self {
        Gem { table: table.map(Self::parse_table), verbs: None }
    }

    /// The word table and the verb rules ([`Verbs`], `am-verbs.tsv`) given as text.
    pub fn from_packs(table: Option<&str>, verbs: Option<&str>) -> Self {
        Gem { table: table.map(Self::parse_table), verbs: verbs.map(Verbs::parse) }
    }

    fn parse_table(t: &str) -> std::collections::HashMap<String, Vec<bool>> {
        let mut m = std::collections::HashMap::new();
        for l in t.lines() {
            let Some((_, marked)) = l.split_once('\t') else { continue };
            let (mut word, mut long) = (String::new(), Vec::new());
            for c in marked.chars() {
                if c == '\u{135F}' {
                    if let Some(x) = long.last_mut() {
                        *x = true;
                    }
                } else if fidel(c).is_some() {
                    word.push(fold(c));
                    long.push(false);
                }
            }
            m.entry(word).or_insert(long);
        }
        m
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_none() && self.verbs.is_none()
    }

    /// Flags (one per fidel letter) for every word; a word the table does not know has none.
    pub fn flags(&self, words: &[&str]) -> Vec<Vec<bool>> {
        words
            .iter()
            .map(|w| {
                let l: Vec<char> = w.chars().filter(|&c| fidel(c).is_some()).collect();
                let key: String = l.iter().map(|&c| fold(c)).collect();
                // a word the table lists keeps the table's marks (even none); the verb rules only see the others
                let mut f = match self.table.as_ref().and_then(|t| t.get(&key)) {
                    Some(f) => f.clone(),
                    None => self.verbs.as_ref().map(|v| v.flags(&key)).unwrap_or_else(|| vec![false; l.len()]),
                };
                let ok = guards(&l);
                for (i, x) in f.iter_mut().enumerate() {
                    *x = *x && ok[i];
                }
                f
            })
            .collect()
    }

    /// Rewrites `ph` of every word that has a long letter. Runs after [`join`].
    pub fn apply(&self, outs: &mut [Out]) {
        if self.is_empty() || outs.is_empty() {
            return;
        }
        let words: Vec<&str> = outs.iter().map(|o| o.src.as_str()).collect();
        let flags = self.flags(&words);
        for (o, f) in outs.iter_mut().zip(flags) {
            if f.iter().any(|&x| x) {
                o.ph = word_gem(&o.src, &f);
            }
        }
    }
}

/// One slot of a verb template: a literal letter of the affix, or the `k`-th radical of the root
/// with the vowel order (empty = sixth) and labialization it is written with.
#[derive(Clone, PartialEq)]
enum Slot {
    X(char),
    R(usize, String, bool),
}

struct Key {
    cell: u32,
    slots: Vec<Slot>,
    long: Vec<usize>,
}

/// Verb rules (optional pack `am-verbs.tsv`, derived from the Armbruster verb tables): unwritten long
/// consonants of the forms of a verb whose root is in the pack's root list, for roots the word table
/// does not know. A template is a cell of the verb table (perfect, imperfect, jussive, ...) with the root
/// radicals as variables; it marks a letter only where at least 90% of the verbs that share the template
/// agree (type A and type B verbs differ in the imperfect, so there nothing is marked). A word is read
/// after stripping a prefix chain (`የ በ ለ ከ ስለ እንደ እስከ ወደ እንዲ` + `አል እየ`, or the person-replacing
/// `ሲ ሚ አይ እንዲ እንዳይ`) and a suffix chain (object and clitic); the person endings of the last radical
/// come from [`ENDINGS`]. When two readings disagree nothing is marked.
pub struct Verbs {
    roots: std::collections::HashSet<Vec<String>>,
    keys: Vec<Key>,
    pre: Vec<(Vec<char>, bool)>,
    suf: std::collections::HashSet<String>,
}

const PREP: [&str; 11] = ["", "የ", "በ", "ለ", "ከ", "ስለ", "እንደ", "እስከ", "ወደ", "እንዲ", "ስለሚ"];
const AUXP: [&str; 3] = ["", "አል", "እየ"];
const PREP_R0: [&str; 9] = ["", "የ", "በ", "ለ", "ከ", "ስለ", "እንደ", "እስከ", "ወደ"];
const AUXR: [&str; 5] = ["ሲ", "ሚ", "አይ", "እንዲ", "እንዳይ"];
const OBJ: [&str; 19] = ["", "ው", "ዋ", "ሽ", "ህ", "ኝ", "ት", "ን", "ሁ", "ቸው", "ችሁ", "ችን", "ሽው", "ዎ", "ት", "ልኝ", "ላት", "ለት", "ልን"];
const CLI: [&str; 5] = ["", "ም", "ና", "ስ", "ን"];
/// (vowel of the last radical, letters after it) in a template -> the endings the running text may carry instead.
const ENDINGS: [(&str, &str, &[(&str, &str)]); 3] = [
    ("ə", "", &[("ə", ""), ("ə", "ች"), ("", "ኩ"), ("", "ክ"), ("", "ህ"), ("", "ሽ"), ("", "ን"), ("", "ኩት"), ("", "ኩህ"), ("u", ""), ("", "ኩሽ")]),
    ("a", "ል", &[("a", "ል"), ("a", "ለች"), ("a", "ለሁ"), ("a", "ለን"), ("a", "ሉ"), ("a", "ሎች")]),
    ("", "", &[("", ""), ("u", ""), ("i", ""), ("", "ኡ")]),
];

fn person(c: char) -> &'static str {
    match c {
        'ይ' => "ይትእን",
        'ያ' => "ያታአና",
        _ => "",
    }
}

impl Verbs {
    pub fn parse(t: &str) -> Self {
        let (mut roots, mut keys) = (std::collections::HashSet::new(), Vec::new());
        for l in t.lines() {
            if l.starts_with('#') || l.trim().is_empty() {
                continue;
            }
            let a: Vec<&str> = l.split('\t').collect();
            match a[0] {
                "R" if a.len() >= 2 => {
                    roots.insert(a[1].split(' ').map(str::to_string).collect::<Vec<_>>());
                }
                "K" if a.len() >= 4 => {
                    let slots: Vec<Slot> = a[2]
                        .split(' ')
                        .filter_map(|t| {
                            let mut ch = t.chars();
                            match ch.next()? {
                                'X' => Some(Slot::X(ch.next()?)),
                                'R' => {
                                    let f: Vec<&str> = ch.as_str().split(':').collect();
                                    Some(Slot::R(f.first()?.parse().ok()?, f.get(1)?.to_string(), *f.get(2)? == "1"))
                                }
                                _ => None,
                            }
                        })
                        .collect();
                    let long = a[3].split(',').filter_map(|x| x.parse().ok()).collect();
                    keys.push(Key { cell: a[1].parse().unwrap_or(0), slots, long });
                }
                _ => {}
            }
        }
        let mut pre = std::collections::BTreeSet::new();
        for p in PREP {
            for q in AUXP {
                pre.insert((format!("{p}{q}"), false));
            }
        }
        for p in PREP_R0 {
            for q in AUXR {
                pre.insert((format!("{p}{q}"), true));
            }
        }
        let pre = pre.into_iter().map(|(s, r)| (s.chars().collect(), r)).collect();
        let mut suf = std::collections::HashSet::new();
        for o in OBJ {
            for c in CLI {
                suf.insert(format!("{o}{c}"));
            }
        }
        Verbs { roots, keys, pre, suf }
    }

    /// Long letters (one flag per fidel letter) of a folded word, none when no verb reading or when the readings disagree.
    pub fn flags(&self, key: &str) -> Vec<bool> {
        let l: Vec<char> = key.chars().collect();
        let mut flags = vec![false; l.len()];
        let marks = self.marks(&l);
        if let Some(m) = marks {
            for i in m {
                if i < flags.len() {
                    flags[i] = true;
                }
            }
        }
        flags
    }

    fn marks(&self, l: &[char]) -> Option<std::collections::BTreeSet<usize>> {
        let seg: Vec<(&str, bool, &str)> = l.iter().filter_map(|&c| fidel(c)).collect();
        if seg.len() != l.len() {
            return None;
        }
        // (base consonant, labialized, vowel) as the templates were learnt
        let lt: Vec<(&str, bool, &str)> = seg.iter().map(|&(c, lab, v)| (c.trim_end_matches('ʷ'), lab || c.ends_with('ʷ'), v)).collect();
        let mut readings: Vec<std::collections::BTreeSet<usize>> = Vec::new();
        for (p, repl) in &self.pre {
            if l.len() < p.len() || l[..p.len()] != p[..] {
                continue;
            }
            let o = p.len();
            for k in &self.keys {
                let mut slots: &[Slot] = &k.slots;
                if *repl {
                    if !matches!(k.cell, 2..=4) || slots.first() != Some(&Slot::X('ይ')) {
                        continue;
                    }
                    slots = &slots[1..];
                }
                let Some(lr) = slots.iter().rposition(|s| matches!(s, Slot::R(..))) else { continue };
                let head = &slots[..lr];
                if l.len() - o < head.len() + 1 {
                    continue;
                }
                let mut got: Vec<Option<&str>> = vec![None; slots.len()];
                let mut ok = true;
                for (i, s) in head.iter().enumerate() {
                    let (ch, c) = (l[o + i], lt[o + i]);
                    match s {
                        Slot::X(x) => {
                            if !(ch == *x || (i == 0 && person(*x).contains(ch))) {
                                ok = false;
                                break;
                            }
                        }
                        Slot::R(r, v, lab) => {
                            if c.2 != v || c.1 != *lab {
                                ok = false;
                                break;
                            }
                            got[*r] = Some(c.0);
                        }
                    }
                }
                if !ok {
                    continue;
                }
                let Slot::R(fr, fv, flab) = &slots[lr] else { continue };
                let c = lt[o + head.len()];
                got[*fr] = Some(c.0);
                if c.1 != *flab {
                    continue;
                }
                let tail: Option<String> = slots[lr + 1..].iter().map(|s| if let Slot::X(x) = s { Some(*x) } else { None }).collect();
                let Some(tail) = tail else { continue };
                let rs: String = l[o + head.len() + 1..].iter().collect();
                let own = [(fv.as_str(), tail.as_str())];
                let ends: &[(&str, &str)] = ENDINGS.iter().find(|e| e.0 == fv && e.1 == tail).map(|e| e.2).unwrap_or(&own);
                let good = ends.iter().any(|&(v, t)| {
                    c.2 == v && (rs == t || (rs.starts_with(t) && self.suf.contains(&rs[t.len()..])))
                });
                if !good {
                    continue;
                }
                let nr = slots.iter().filter(|s| matches!(s, Slot::R(..))).count();
                let root: Option<Vec<String>> = (0..nr).map(|i| got[i].map(str::to_string)).collect();
                let Some(root) = root else { continue };
                if !self.roots.contains(&root) {
                    continue;
                }
                let sh = usize::from(*repl);
                readings.push(k.long.iter().filter(|&&i| o + i >= sh).map(|&i| o + i - sh).collect());
            }
        }
        let first = readings.first()?;
        if readings.iter().any(|r| r != first) {
            return Some(Default::default());
        }
        Some(first.clone())
    }
}

/// Context-free phonemes of one word: its fidel letters as one run (anything
/// else in the token is skipped).
pub fn word(lw: &str) -> String {
    word_gem(lw, &[])
}

/// [`word`] with the letters flagged in `gem` read as long consonants (`ː`).
pub fn word_gem(lw: &str, gem: &[bool]) -> String {
    let mut out = String::with_capacity(lw.len());
    run(lw, gem, &mut out);
    out
}

/// Ethiopic and typographic punctuation → the mark the output keeps, followed
/// by a space so that it never reads as a digit separator (`3፣4` is two
/// numbers). `Some(" ")` is a word boundary; `Some("")` deletes the character,
/// so the letters around it join: quotation marks, apostrophes, and the
/// combining marks (the Ethiopic gemination and vowel-length marks
/// U+135D..U+135F, and U+0300..U+036F).
fn punct(c: char) -> Option<&'static str> {
    Some(match c {
        '።' | '፨' | '…' => ". ",
        '፣' | '፤' | '፥' | '(' | ')' | '–' | '—' | ';' => ", ",
        '፦' => ": ",
        '፧' => "? ",
        '፡' | '፠' => " ",
        '«' | '»' | '“' | '”' | '‘' | '’' | '"' | '\'' | '\u{135D}'..='\u{135F}' | '\u{0300}'..='\u{036F}' => "",
        _ => return None,
    })
}

const KEPT: &[char] = &['.', ',', '?', '!', ':'];
/// Symbols the shared tokenizer hands to [`symbol`].
const SPOKEN: &[char] = &['%', '$', '€', '£', '₪', '¥', '₽', '&', '+', '@', '°', '='];

/// The punctuation pass that runs before the shared tokenizer.
///
/// * Ethiopic punctuation becomes `. , : ?`; the word space `፡` becomes a space.
/// * `፡፡` and `::`, the usual ways to type `።`, are a full stop; `፡-` is `፦`.
/// * Quotation marks, apostrophes and combining marks are deleted.
/// * Fidel, digits, `. , ? ! :`, white space and the symbols that are read
///   (`%`, currency signs, `&`, `°`) pass through.
/// * Anything else is a space: hyphens and dashes, slashes, Latin letters, other
///   scripts, Ethiopic numerals.
pub fn marks(text: &str) -> String {
    let src: Vec<char> = text.chars().collect();
    let mut s = String::with_capacity(text.len() + 16);
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        i += 1;
        if matches!(c, '፡' | ':') && src.get(i) == Some(&c) {
            while src.get(i) == Some(&c) {
                i += 1;
            }
            s.push_str(". ");
        } else if c == '፡' && src.get(i) == Some(&'-') {
            i += 1;
            s.push_str(": ");
        } else if let Some(p) = punct(c) {
            s.push_str(p);
        } else if in_run(c) || c.is_ascii_digit() || c.is_whitespace() || KEPT.contains(&c) || SPOKEN.contains(&c) {
            s.push(c);
        } else {
            s.push(' ');
        }
    }
    s
}

/// Is `c` Ethiopic punctuation? (`lib.rs` keeps it with an inline tag: `[laughs]።`.)
pub fn is_mark(c: char) -> bool {
    matches!(c, '።' | '፨' | '፣' | '፤' | '፥' | '፦' | '፧' | '፡' | '፠')
}

/// `.`, `,`, `:` right after another mark are dropped (`?.` → `?`).
fn dedupe(punct: &mut String) {
    let mut first = true;
    punct.retain(|c| {
        let keep = first || !matches!(c, '.' | ',' | ':');
        first = false;
        keep
    });
}

/// The kept marks of a run of punctuation that follows an inline tag.
pub fn tag_marks(run: &str) -> String {
    let mut p: String = marks(run).chars().filter(|c| KEPT.contains(c)).collect();
    dedupe(&mut p);
    p
}

/// Sentence pass: a mark never stands alone. A token with no phonemes (letters
/// the module does not read) is removed and its marks go to the word before it;
/// marks before the first word are dropped.
pub fn join(outs: &mut Vec<Out>) {
    let mut kept: Vec<Out> = Vec::with_capacity(outs.len());
    for o in outs.drain(..) {
        if !o.ph.is_empty() {
            kept.push(o);
        } else if let Some(prev) = kept.last_mut() {
            prev.pause_after |= !o.punct.is_empty();
            prev.punct.push_str(&o.punct);
        }
    }
    for o in &mut kept {
        dedupe(&mut o.punct);
    }
    *outs = kept;
}

/// Prefixes written against a number that are read as one word with it:
/// `በ5` → `በአምስት`, `እስከ5` → `እስከአምስት`.
pub fn num_prefix(w: &str) -> bool {
    matches!(w, "በ" | "ለ" | "ከ" | "የ" | "እስከ")
}

/// Is `a`, written right after a number whose last word is `last`, a suffix of
/// the number? Anything else is a word of its own (`5ሰዎች` is `አምስት ሰዎች`,
/// `10ሺህ` is `አሥር ሺህ`).
///
/// * the ordinal `ኛ` and its definite and case forms: `5ኛ`, `5ኛው`, `5ኛዋ`,
///   `5ኛውን`;
/// * the definite form, which restates the last consonant of the number word
///   in the second or the eighth order (`2ቱ`, `2ቷ`, `9ኙ`, `10ሩ`), or `ው` / `ዋ`
///   after a vowel (`20ው`);
/// * the plurals `ዎቹ` / `ዎች` (`1990ዎቹ`);
/// * the clitics `ን`, `ም`, `ና` alone or after any of the above (`2ም`, `5ቱን`).
pub fn num_suffix(last: &str, a: &str) -> bool {
    const TAILS: [&str; 6] = ["", "ን", "ም", "ና", "ንም", "ንና"];
    let Some(first) = a.chars().next() else { return false };
    let rest = &a[first.len_utf8()..];
    if TAILS.contains(&a) {
        return true;
    }
    if first == 'ኛ' {
        return TAILS.contains(&rest) || ["ው", "ዋ", "ዎቹ", "ዎች"].iter().any(|d| rest.strip_prefix(d).is_some_and(|t| TAILS.contains(&t)));
    }
    if let Some(t) = a.strip_prefix("ዎቹ").or_else(|| a.strip_prefix("ዎች")) {
        return TAILS.contains(&t);
    }
    if !TAILS.contains(&rest) {
        return false;
    }
    match last.chars().last().and_then(|e| Some((e, fidel(e)?))) {
        // a bare consonant: the same row in the second or eighth order
        Some((e, (c, false, ""))) if c != "ʔ" => {
            let (o, r) = (first as u32, e as u32 & !7);
            o == r + 1 || o == r + 7
        }
        Some(_) => matches!(first, 'ው' | 'ዋ'),
        None => false,
    }
}

/// A prefix ([`num_prefix`]) or a suffix ([`num_suffix`]) written against
/// digits is one word with them. The prefix goes in front of the first number
/// word (`በ5` → `በአምስት`). The suffix restates the end of the last one, as in
/// the written language:
///
/// * `ኛ` makes the ordinal: a final bare consonant takes its first order
///   (`5ኛ` → `አምስተኛ`, `1ኛ` → `አንደኛ`; `9ኛ` → `ዘጠነኛ`);
/// * a suffix that starts with the consonant the number word ends in replaces
///   that letter (`2ቱ` → `ሁለቱ`, `10ሩ` → `አሥሩ`);
/// * anything else is appended (`2ም` → `ሁለትም`).
pub fn glue(before: Option<&str>, words: &mut [String], after: Option<&str>) {
    if let (Some(b), Some(w)) = (before, words.first_mut()) {
        w.insert_str(0, b);
    }
    let (Some(a), Some(w)) = (after, words.last_mut()) else { return };
    let end = w.chars().last().and_then(|e| Some((e, fidel(e)?)));
    let start = a.chars().next().and_then(fidel);
    if let (Some((e, (ec, false, ""))), Some((sc, _, _))) = (end, start) {
        // the number word ends in a bare consonant
        let guttural = matches!(ec, "h" | "ʔ");
        if a.starts_with('ኛ') && !guttural {
            w.pop();
            w.push(if e == 'ኝ' { 'ነ' } else { char::from_u32(e as u32 & !7).unwrap_or(e) });
        } else if sc == ec && !guttural {
            w.pop();
        }
    }
    w.push_str(a);
}

const UNITS: [&str; 10] = ["ዜሮ", "አንድ", "ሁለት", "ሦስት", "አራት", "አምስት", "ስድስት", "ሰባት", "ስምንት", "ዘጠኝ"];
const TENS: [&str; 10] = ["", "አሥር", "ሃያ", "ሠላሳ", "አርባ", "ሃምሳ", "ስልሳ", "ሰባ", "ሰማንያ", "ዘጠና"];
const SCALES: [&str; 5] = ["", "ሺህ", "ሚሊዮን", "ቢሊዮን", "ትሪሊዮን"];

fn below_thousand(n: u128, out: &mut Vec<String>) {
    let (h, r) = (n / 100, n % 100);
    if h > 0 {
        out.push(UNITS[h as usize].to_string());
        out.push("መቶ".to_string());
    }
    let (t, u) = (r / 10, r % 10);
    match (t, u) {
        (0, 0) => {}
        (0, u) => out.push(UNITS[u as usize].to_string()),
        (1, 0) => out.push(TENS[1].to_string()),
        (1, u) => {
            // 11..19: አሥራ + unit
            out.push("አሥራ".to_string());
            out.push(UNITS[u as usize].to_string());
        }
        (t, 0) => out.push(TENS[t as usize].to_string()),
        (t, u) => {
            out.push(TENS[t as usize].to_string());
            out.push(UNITS[u as usize].to_string());
        }
    }
}

/// Number words, one written word per item (`መቶ` hundred, `ሺህ` thousand).
pub fn cardinal(n: u128) -> Vec<String> {
    if n == 0 {
        return vec![UNITS[0].to_string()];
    }
    if n >= 1_000_000_000_000_000 {
        // beyond the named scales: digit by digit
        return n.to_string().bytes().map(|b| UNITS[(b - b'0') as usize].to_string()).collect();
    }
    let mut groups = Vec::new();
    let mut m = n;
    while m > 0 {
        groups.push(m % 1000);
        m /= 1000;
    }
    let mut out = Vec::new();
    for (k, &g) in groups.iter().enumerate().rev() {
        if g == 0 {
            continue;
        }
        below_thousand(g, &mut out);
        if k > 0 {
            out.push(SCALES[k].to_string());
        }
    }
    out
}

/// Words for a spoken symbol. Symbols without an entry are skipped.
pub fn symbol(c: char) -> Option<&'static str> {
    match c {
        '%' => Some("በመቶ"),
        '$' => Some("ዶላር"),
        '€' => Some("ዩሮ"),
        '£' => Some("ፓውንድ"),
        '&' => Some("እና"),
        '°' => Some("ዲግሪ"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters() {
        for (w, p) in [
            ("ሰላም", "səlam"),
            ("ሀሎ", "halo"),
            ("ሁለት", "hulət"),
            ("ውሃ", "wɨha"),
            ("ቋንቋ", "kʼʷankʼʷa"),
            ("ጐረቤት", "ɡʷərəbet"),
            ("ጸሐይ", "sʼəhaj"),
            ("ፀሐይ", "sʼəhaj"),
            ("ጨው", "tʃʼəw"),
            ("ጳጳስ", "pʼapʼas"),
            ("ጀመረ", "dʒəmərə"),
            ("ዠ", "ʒə"),
            ("ኧረ", "ərə"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn carriers() {
        for (w, p) in [("አንድ", "and"), ("ኢትዮጵያ", "itjopʼja"), ("እግር", "ɨɡɨr"), ("ሰዓት", "səʔat"), ("ረቡዕ", "rəbuʔ"), ("ዓለም", "aləm")] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn epenthesis() {
        for (w, p) in [
            ("ስም", "sɨm"),
            ("ብር", "bɨr"),
            ("ልብስ", "lɨbs"),
            ("ስልክ", "sɨlk"),
            ("ክፍል", "kɨfɨl"),
            ("ክብር", "kɨbɨr"),
            ("ጥቅም", "tʼɨkʼɨm"),
            ("ድንግል", "dɨnɡɨl"),
            ("ትምህርት", "tɨmhɨrt"),
            ("መንግሥት", "mənɡɨst"),
            ("ስምንት", "sɨmɨnt"),
            ("አንድነት", "andɨnət"),
            ("ምክንያት", "mɨknɨjat"),
            ("ማንኛው", "manɨɲaw"),
            ("ሆስፒታል", "hospital"),
            ("ች", "tʃ"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn clusters() {
        assert!(coda_ok("b", "s") && coda_ok("n", "d") && coda_ok("r", "t") && coda_ok("j", "kʼ") && coda_ok("j", "l"));
        assert!(!coda_ok("f", "l") && !coda_ok("ɡ", "r") && !coda_ok("kʼ", "m") && !coda_ok("m", "n") && !coda_ok("t", "t"));
        // h does not open a cluster
        assert!(!coda_ok("h", "t"));
        assert_eq!(word("እኅት"), "ɨhɨt");
    }

    #[test]
    fn suffix_consonants() {
        // n m ɲ h ʃ do not close a cluster, whatever stands before them
        for c1 in ["l", "r", "j", "w", "t", "b", "m", "dʒ"] {
            for c2 in ["n", "m", "ɲ", "h", "ʃ"] {
                assert!(!coda_ok(c1, c2), "{c1} {c2}");
            }
        }
        for (w, p) in [
            ("ያለውን", "jaləwɨn"),
            ("እግዚአብሔርን", "ɨɡziʔabherɨn"),
            ("ወይም", "wəjɨm"),
            ("አይችልም", "ajtʃɨlɨm"),
            ("ቤትህ", "betɨh"),
            ("ልጅህ", "lɨdʒɨh"),
            ("ፍትሕ", "fɨtɨh"),
            ("ስምሽ", "sɨmɨʃ"),
            ("ልጅሽ", "lɨdʒɨʃ"),
            ("ትንሽ", "tɨnɨʃ"),
            ("ይስጥልኝ", "jɨstʼɨlɨɲ"),
            ("ንገርኝ", "nɨɡərɨɲ"),
            // the consonant before the suffix closes the syllable before it
            ("ትምህርትን", "tɨmhɨrtɨn"),
            ("ስነግርሽ", "sɨnəɡrɨʃ"),
            // a vowel before the suffix: nothing to add
            ("ቤቱን", "betun"),
            ("ነህ", "nəh"),
            // other final clusters are untouched
            ("ኃይል", "hajl"),
            ("ሆንክ", "honk"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn lexical_clusters() {
        for (w, p) in [
            ("ዓይን", "ajn"),
            ("አይን", "ajn"),
            ("የዓይን", "jəʔajn"),
            ("ወይን", "wəjn"),
            ("ከወይን", "kəwəjn"),
            ("ሕልም", "hɨlm"),
            ("ህልም", "hɨlm"),
            ("ፊልም", "film"),
            ("የፊልም", "jəfilm"),
            ("ዲዛይን", "dizajn"),
            ("ኦንላይን", "onlajn"),
            ("ታይም", "tajm"),
            ("ዳውን", "dawn"),
            ("ፕላትፎርም", "pɨlatform"),
            ("የሚገርም", "jəmiɡərm"),
            ("ወደሚገርም", "wədəmiɡərm"),
            ("እናርም", "ɨnarm"),
            ("እውን", "ɨwn"),
            ("ማርሽ", "marʃ"),
            ("ታውን", "tawn"),
            ("ከራይን", "kərajn"),
            ("ቢትኮይን", "bitkojn"),
            ("ኮንፈርም", "konfərm"),
            ("ስቶክሆልም", "sɨtokholm"),
            // not listed: the suffix reading
            ("ላይን", "lajɨn"),
            ("ሰማይን", "səmajɨn"),
            ("ደስታውን", "dəstawɨn"),
            ("አይታይም", "ajtajɨm"),
            // only the listed word: a longer word that ends the same way takes the suffix reading
            ("ያህልም", "jahlɨm"),
            ("ሀገርም", "haɡərɨm"),
            ("ወይንም", "wəjnɨm"),
            // a suffix after the listed word
            ("ዓይንህ", "ajnɨh"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn perfective_endings() {
        for (w, p) in [
            // 1pl -n after a stem in r or l
            ("ነበርን", "nəbərn"),
            ("ጀመርን", "dʒəmərn"),
            ("ተነጋገርን", "tənəɡaɡərn"),
            ("ስለነበርን", "sɨlənəbərn"),
            ("እየኖርን", "ɨjənorn"),
            ("ካልጀመርን", "kaldʒəmərn"),
            ("እያስተማርን", "ɨjastəmarn"),
            ("እንዳከበርን", "ɨndakəbərn"),
            ("ካልን", "kaln"),
            ("እያልን", "ɨjaln"),
            ("አላልን", "alaln"),
            ("ከቻልን", "kətʃaln"),
            ("ስላልቻልን", "sɨlaltʃaln"),
            // 2fsg -ʃ, 2msg -h
            ("ነበርሽ", "nəbərʃ"),
            ("ያልሽ", "jalʃ"),
            ("ካልከፈልሽ", "kalkəfəlʃ"),
            ("አደረግሽ", "adərəɡʃ"),
            ("ወደድሽ", "wədədʃ"),
            ("ነበርህ", "nəbərh"),
            ("አደረግህ", "adərəɡɨh"),
            // sonority still decides: an obstruent before -n takes the vowel
            ("አደረግን", "adərəɡɨn"),
            // not a perfective form: nouns, the infinitive, a noun that looks like a stem
            ("ነገርን", "nəɡərɨn"),
            ("ሀገርን", "haɡərɨn"),
            ("መኖርን", "mənorɨn"),
            ("መማርን", "məmarɨn"),
            ("ባልን", "balɨn"),
            ("ሀሳብሽ", "hasabɨʃ"),
            // the benefactive -ll- + 1pl object keeps the vowel
            ("ይስጥልን", "jɨstʼɨlɨn"),
            ("እያሰፋልን", "ɨjasəfalɨn"),
            ("የላከልን", "jəlakəlɨn"),
            // the negative with -m is syllabified as before
            ("አልነበርንም", "alnəbərnɨm"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
        let chars = |w: &str| w.chars().collect::<Vec<char>>();
        assert!(perfective(&chars("ነበርን")) && perfective(&chars("ያስተማርሽ")) && perfective(&chars("አልህ")));
        assert!(!perfective(&chars("ነበር")) && !perfective(&chars("ን")) && !perfective(&chars("")) && !perfective(&chars("መኖርን")) && !perfective(&chars("ነበርም")));
    }

    #[test]
    fn stems() {
        for (w, p) in [
            ("አምስት", "amɨst"),
            ("አምስተኛ", "amɨstəɲa"),
            ("አምስቱ", "amɨstu"),
            ("አምስትም", "amɨstɨm"),
            ("ስድስት", "sɨdɨst"),
            ("ስድስተኛው", "sɨdɨstəɲaw"),
            ("ስድስቱን", "sɨdɨstun"),
            ("ስምንት", "sɨmɨnt"),
            ("ስምንተኛ", "sɨmɨntəɲa"),
            ("ስምንቱ", "sɨmɨntu"),
            ("በአምስት", "bəʔamɨst"),
            ("የአምስተኛው", "jəʔamɨstəɲaw"),
            ("ከስድስት", "kəsɨdɨst"),
            ("በስምንተኛ", "bəsɨmɨntəɲa"),
            ("ትልቅ", "tɨlɨkʼ"),
            ("ትልቁ", "tɨlɨkʼu"),
            ("በትልቅ", "bətɨlɨkʼ"),
            ("ትልቅነት", "tɨlɨkʼnət"),
            ("ክርስቶስ", "kɨrɨstos"),
            ("ክርስቲያን", "kɨrɨstijan"),
            ("ክርስትና", "kɨrɨstɨna"),
            ("ቤተክርስቲያን", "betəkrɨstijan"),
            ("በክርስቶስ", "bəkrɨstos"),
            // the stem needs its next letter: these are other words
            ("አምስ", "ams"),
            ("ትል", "tɨl"),
            ("ትልም", "tɨlɨm"),
            ("ስምን", "sɨmɨn"),
        ] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn no_false_affricate() {
        // pairs fold.rs would write as ʧ ʦ ʤ ʣ are never left next to each other
        assert!(!coda_ok("t", "ʃ") && !coda_ok("t", "s") && !coda_ok("d", "ʒ") && !coda_ok("d", "z"));
        assert!(coda_ok("tʼ", "s") && coda_ok("d", "s") && coda_ok("t", "sʼ"));
        for (w, p) in [
            ("ቤትሽ", "betɨʃ"),
            ("ኮትሽ", "kotɨʃ"),
            ("ቤትሽን", "betɨʃɨn"),
            ("ቤትስ", "betɨs"),
            ("ቤትሻ", "betɨʃa"),
            ("ከብትሽ", "kəbtɨʃ"),
            ("ውድዥ", "wɨdɨʒ"),
            ("ውድዝ", "wɨdɨz"),
            ("ውድዛ", "wɨdɨza"),
            ("ትሿ", "tɨʃʷa"),
            // the real affricates and other clusters are untouched
            ("ቤች", "betʃ"),
            ("ቤትሳ", "betsa"),
            ("መንግሥት", "mənɡɨst"),
        ] {
            assert_eq!(word(w), p, "{w}");
            let folded = crate::fold::fold(&word(w));
            assert_eq!(folded.contains(['ʧ', 'ʦ', 'ʤ', 'ʣ']), w == "ቤች", "{w} -> {folded}");
        }
    }

    #[test]
    fn punctuation_pass() {
        assert_eq!(marks("ሰላም፣ ዓለም።"), "ሰላም,  ዓለም. ");
        // ፡፡ and :: are a full stop, ፡- is ፦, one ፡ is a space
        assert_eq!(marks("ሀገር፡፡"), "ሀገር. ");
        assert_eq!(marks("ሰላም::"), "ሰላም. ");
        assert_eq!(marks("ለምሳሌ፡- ቡና"), "ለምሳሌ:  ቡና");
        assert_eq!(marks("ሰላም፡ዓለም"), "ሰላም ዓለም");
        // an Ethiopic comma between digits is not a digit separator
        assert_eq!(marks("3፣4"), "3, 4");
        // deleted: quotation marks, apostrophes, combining marks
        assert_eq!(marks("«የ'ሰ\u{135F}ላ\u{0301}ም'»"), "የሰላም");
        // a space: hyphens, the minus sign, slashes, Latin letters, Ethiopic numerals
        assert_eq!(marks("ሥነ-ሥርዓት −5 ዶ/ር COVID-19 ፩"), "ሥነ ሥርዓት  5 ዶ ር       19  ");
        assert_eq!(marks("$5 50% 3:30 1,000.5"), "$5 50% 3:30 1,000.5");
        assert_eq!(tag_marks("።"), ".");
        assert_eq!(tag_marks("?።"), "?");
        assert_eq!(tag_marks("፡፡"), ".");
        assert_eq!(tag_marks(";"), ",");
        assert_eq!(tag_marks("፡"), "");
    }

    #[test]
    fn numbers() {
        assert_eq!(cardinal(0), ["ዜሮ"]);
        assert_eq!(cardinal(11), ["አሥራ", "አንድ"]);
        assert_eq!(cardinal(21), ["ሃያ", "አንድ"]);
        assert_eq!(cardinal(100), ["አንድ", "መቶ"]);
        assert_eq!(cardinal(1234), ["አንድ", "ሺህ", "ሁለት", "መቶ", "ሠላሳ", "አራት"]);
        assert_eq!(cardinal(2_000_000), ["ሁለት", "ሚሊዮን"]);
        assert_eq!(symbol('%'), Some("በመቶ"));
        assert_eq!(symbol('$'), Some("ዶላር"));
        assert_eq!(symbol('+'), None);
    }

    #[test]
    fn digits_against_letters() {
        let glued = |before: Option<&str>, n: u128, after: Option<&str>| {
            let mut w = cardinal(n);
            glue(before, &mut w, after);
            w.join(" ")
        };
        for (n, after, want) in [
            (1, "ኛ", "አንደኛ"),
            (2, "ኛ", "ሁለተኛ"),
            (5, "ኛ", "አምስተኛ"),
            (9, "ኛ", "ዘጠነኛ"),
            (10, "ኛ", "አሥረኛ"),
            (20, "ኛ", "ሃያኛ"),
            (100, "ኛው", "አንድ መቶኛው"),
            (1000, "ኛ", "አንድ ሺህኛ"),
            (2, "ቱ", "ሁለቱ"),
            (5, "ቱ", "አምስቱ"),
            (9, "ኙ", "ዘጠኙ"),
            (10, "ሩ", "አሥሩ"),
            (2, "ም", "ሁለትም"),
            (20, "ዎቹ", "ሃያዎቹ"),
        ] {
            assert_eq!(glued(None, n, Some(after)), want, "{n}{after}");
        }
        assert_eq!(glued(Some("በ"), 5, None), "በአምስት");
        assert_eq!(glued(Some("ከ"), 1000, Some("ኛ")), "ከአንድ ሺህኛ");
        assert_eq!(glued(None, 5, None), "አምስት");
        // the ordinal and the definite form keep the shape of the cardinal
        for (w, p) in [("አምስተኛ", "amɨstəɲa"), ("ስድስተኛ", "sɨdɨstəɲa"), ("ስምንተኛ", "sɨmɨntəɲa"), ("አምስቱ", "amɨstu"), ("ስድስቱ", "sɨdɨstu"), ("ስምንቱ", "sɨmɨntu")] {
            assert_eq!(word(w), p, "{w}");
        }
    }

    #[test]
    fn what_is_glued_to_digits() {
        for (last, a) in [
            ("አምስት", "ኛ"),
            ("አምስት", "ኛው"),
            ("አምስት", "ኛዋ"),
            ("ሦስት", "ኛውን"),
            ("አምስት", "ኛም"),
            ("ሁለት", "ቱ"),
            ("ሁለት", "ቷ"),
            ("አምስት", "ቱን"),
            ("ዘጠኝ", "ኙ"),
            ("አሥር", "ሩ"),
            ("አንድ", "ዱ"),
            ("ሃያ", "ው"),
            ("ዘጠና", "ዎቹ"),
            ("ሃያ", "ዎቹን"),
            ("ሁለት", "ም"),
            ("ሁለት", "ን"),
            ("ሁለት", "ና"),
        ] {
            assert!(num_suffix(last, a), "{last} + {a}");
        }
        // words of their own
        for (last, a) in [("አሥር", "ሺህ"), ("ሦስት", "ሚሊዮን"), ("አምስት", "ሰዎች"), ("አምስት", "ዓ"), ("አምስት", "ብር"), ("ሁለት", "ሩ"), ("ሃያ", "ቱ"), ("አምስት", "ኛክፍል"), ("አምስት", "ቱር")] {
            assert!(!num_suffix(last, a), "{last} + {a}");
        }
        assert!(["በ", "ለ", "ከ", "የ", "እስከ"].iter().all(|w| num_prefix(w)));
        assert!(!num_prefix("ብር") && !num_prefix("ሰዓት") && !num_prefix(""));
    }

    #[test]
    fn long_consonants() {
        let l = |w: &str, g: &[usize]| {
            let n = w.chars().filter(|&c| fidel(c).is_some()).count();
            word_gem(w, &(0..n).map(|i| g.contains(&i)).collect::<Vec<_>>())
        };
        // R2: a long bare consonant that is not word-final takes ɨ
        assert_eq!(l("ምሽት", &[1]), "mɨʃːɨt");
        assert_eq!(l("አምስት", &[1]), "amːɨst");
        assert_eq!(l("ስድስት", &[1]), "sɨdːɨst");
        // R1: a bare consonant before a long one takes ɨ
        assert_eq!(l("ሀብታምነት", &[4]), "habtamɨnːət");
        assert_eq!(l("ማንኛው", &[2]), "manɨɲːaw");
        // guards: a final bare letter, a first letter, h, are never long
        assert_eq!(l("ቤት", &[1]), "bet");
        assert_eq!(l("ሰላም", &[0]), "səlam");
        assert_eq!(l("ሰሀ", &[1]), "səha");
        // a labialized letter and an ejective affricate: ː after the whole consonant
        assert_eq!(l("ሰሏ", &[1]), "səlʷːa");
        assert_eq!(l("ጠጨ", &[1]), "tʼətʃʼːə");
        // R2 does not apply before a glide (ኢትዮጵያ)
        assert_eq!(l("ኢትዮጵያ", &[3]), "itjopʼːja");
        // the stem table is skipped when a flag falls on a stem letter or on the letter after it
        assert_eq!(l("አምስተኛ", &[1]), "amːɨstəɲa");
        assert_eq!(l("አምስተኛ", &[3]), "amsɨtːəɲa");
        assert_eq!(l("አምስተኛ", &[]), "amɨstəɲa");
    }

    #[test]
    fn guards_and_fold() {
        let g = |w: &str| guards(&w.chars().collect::<Vec<_>>());
        assert_eq!(g("ሰላም"), vec![false, true, false]);
        assert_eq!(g("ሰሀ"), vec![false, false]);
        assert_eq!(g("አምስ"), vec![false, true, false]);
        assert_eq!(fold('ሠ'), 'ሰ');
        assert_eq!(fold('ዓ'), 'አ');
        assert_eq!(fold('ኸ'), 'ሀ');
        assert_eq!(fold('ሰ'), 'ሰ');
    }
}
