#!/usr/bin/env python3
"""Parity check: Russian-G2P (Python) vs polyg2p src/ru.rs (Rust, via the ru_check binary).

    python3 ru_parity.py [--words N] [--sentences N] [--no-build] [--show 30]

A. Word level, word_to_ipa(word) vs ru::word_to_ipa(word, None):
   1. tests/gold.tsv: every `word` row with the stress each gold variant implies
      ('+' before the vowel its ˈ marks; unmarked monosyllables also without '+'),
      plus the words of every `sentence` row aligned with their gold slots.
   2. synthetic: every entry of rus_cyrl_narrow.tsv (466k Wiktionary words, case
      kept) with '+' before a random vowel (seeded), plus edge cases: '+' at a
      random position (before consonants, at the end, doubled), no '+', and
      combining acute accents.
B. Sentence level, phonemize_russian(text) vs Russian::phonemize on data/ud/ru.txt.
   Python refuses to run without ru_lexicon/accents.tsv.gz, and the real RUAccent
   tables cannot be downloaded here, so both sides run on the same stand-ins:
   * "empty":     a RU_LEXICON_DIR holding the seven tables, all empty; Rust gets
                  no .lex files at all (Russian::load(None)). This is the degraded
                  configuration: every polysyllable takes the ood-suffix tier
                  (penultimate stress), ё only where the text writes it, все/всё
                  rule off (все is not a yo-homograph without the table).
   * "synthetic": deterministic fake tables built from the corpus vocabulary and
                  tests/gold.tsv so that EVERY tier fires (dict, yo, yo-homograph
                  + все/всё rule, homograph prior, homograph ctx prior, train,
                  suffix, out-of-range indices); converted to .lex with
                  ru_build_lex.py, so the conversion is tested too.
   In both, no ONNX models exist (Python: om_ok/yo_ok False), the legacy engine
   is unavailable (ruaccent not installed), and Latin words are kept verbatim
   (Python's behaviour when eSpeak is missing; Rust: identity `latin` callback,
   ru_check --latin-keep). A third run checks Latin dropping (Python
   _latin_to_ipa -> [] vs Rust without a callback) on the empty config.
C. Speed: Python word_to_ipa and phonemize_russian vs ru_check --bench.
"""

from __future__ import annotations

import argparse
import collections
import gzip
import hashlib
import io
import os
import random
import re
import subprocess
import sys
import tempfile
import time

REPO = os.environ.get("RUSSIAN_G2P_REPO", "../Russian-G2P")
CRATE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
RU_CHECK = os.path.join(CRATE, "target/release/ru_check")
# parity is against Python WITHOUT its ONNX context models: keep the built-in student off
os.environ["POLYG2P_RU_CTX"] = "0"
WIKT = os.path.join(os.environ.get("POLYG2P_SRC", "sources"), "rus_cyrl_narrow.tsv")
UD = os.path.join(os.environ.get("POLYG2P_SRC", "sources"), "ud", "ru.txt")
HERE = os.path.dirname(os.path.abspath(__file__))
VOWELS = "аеёиоуыэюя"

sys.path.insert(0, REPO)
_stdout = sys.stdout
sys.stdout = io.StringIO()  # text_vocab prints a banner on import
from data import russian_g2p_rules as rules  # noqa: E402
from data import russian_phonemizer as rp  # noqa: E402
sys.stdout = _stdout


def h(s: str) -> int:
    return int.from_bytes(hashlib.md5(s.encode()).digest()[:8], "little")


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------
def rust(args: list[str], lines: list[str]) -> tuple[list[str], float]:
    data = "".join(l + "\n" for l in lines).encode()
    t = time.perf_counter()
    r = subprocess.run([RU_CHECK, *args], input=data, capture_output=True, check=True)
    dt = time.perf_counter() - t
    out = r.stdout.decode().split("\n")
    assert out[-1] == "", "missing final newline"
    out = out[:-1]
    assert len(out) == len(lines), (len(out), len(lines))
    return out, dt


def rust_bench(args: list[str], lines: list[str], passes: int) -> str:
    data = "".join(l + "\n" for l in lines).encode()
    r = subprocess.run([RU_CHECK, *args, "--bench", str(passes)], input=data, capture_output=True, check=True)
    return r.stderr.decode().strip()


def safe_line(s: str) -> bool:
    # one input per line on both sides: no line separators of either language
    return not any(c in s for c in "\n\r\x0b\x0c\x1c\x1d\x1e\x85  ")


def compare(name: str, inputs: list[str], py: list[str], rs: list[str], show: int) -> int:
    bad = [(i, p, r) for i, (p, r) in enumerate(zip(py, rs)) if p != r]
    n = len(inputs)
    print(f"  {name}: {n - len(bad)}/{n} exact ({100.0 * (n - len(bad)) / max(n, 1):.4f}%)")
    for i, p, r in bad[:show]:
        print(f"    MISMATCH {inputs[i]!r}\n      py {p!r}\n      rs {r!r}")
    return len(bad)


# ---------------------------------------------------------------------------
# A. words
# ---------------------------------------------------------------------------
def load_gold():
    rows = []
    with open(os.path.join(REPO, "tests/gold.tsv"), encoding="utf-8") as f:
        for line in f:
            if not line.strip() or line.startswith("#"):
                continue
            kind, text, gold, *_ = line.rstrip("\n").split("\t")
            slots = [s.split("|") for s in gold.split(" ")] if kind == "sentence" else [gold.split("|")]
            rows.append((kind, text, slots))
    return rows


def stress_inputs(word: str, variants: list[str]) -> list[str]:
    """The '+'-forms of `word` implied by its gold variants."""
    low = word.lower()
    vs = [i for i, c in enumerate(low) if c in VOWELS]
    out = []
    for g in variants:
        k = rules.stress_vowel_number(g)
        if k is not None and k < len(vs):
            out.append(word[: vs[k]] + "+" + word[vs[k]:])
        elif len(vs) <= 1:
            out.append(word)  # monosyllable / vowelless: label convention, no mark
            if vs:
                out.append(word[: vs[0]] + "+" + word[vs[0]:])
    if not out:
        out.append(word)
    return list(dict.fromkeys(out))


def gold_words() -> tuple[list[str], int, int]:
    """-> inputs, and (gold-match count, gold rows) for the Python rules as a sanity check."""
    inputs: list[str] = []
    hit = tot = 0
    for kind, text, slots in load_gold():
        if kind == "word":
            pairs = [(text, slots[0])]
        else:
            toks = re.findall(r"[а-яёА-ЯЁ]+", text)
            if len(toks) != len(slots):
                pairs = [(t, []) for t in toks]
            else:
                pairs = list(zip(toks, slots))
        for w, variants in pairs:
            ins = stress_inputs(w, variants)
            inputs.extend(ins)
            if variants:
                tot += 1
                hit += any(rules.word_to_ipa(x) in variants for x in ins)
    return list(dict.fromkeys(inputs)), hit, tot


def synthetic_words(n_limit: int | None) -> list[str]:
    rnd = random.Random(20261023)
    words = []
    with open(WIKT, encoding="utf-8") as f:
        for line in f:
            w = line.rstrip("\n").split("\t")[0]
            if w and safe_line(w):
                words.append(w)
    if n_limit:
        words = words[:n_limit]
    out = []
    for w in words:
        vs = [i for i, c in enumerate(w) if c.lower() in VOWELS]
        if vs:
            i = rnd.choice(vs)
            out.append(w[:i] + "+" + w[i:])
        else:
            out.append(w)
    # edge cases on a sample
    sample = rnd.sample(words, min(30000, len(words)))
    for w in sample[:10000]:  # '+' anywhere, including before consonants / at the end
        i = rnd.randint(0, len(w))
        out.append(w[:i] + "+" + w[i:])
    for w in sample[10000:15000]:  # two '+': the last one wins
        i, j = sorted(rnd.randint(0, len(w)) for _ in range(2))
        out.append(w[:i] + "+" + w[i:j] + "+" + w[j:])
    out.extend(sample[15000:25000])  # no stress mark
    for w in sample[25000:30000]:  # combining acute after the vowel (dropped), upper-cased
        vs = [i for i, c in enumerate(w) if c.lower() in VOWELS]
        if vs:
            i = rnd.choice(vs)
            out.append((w[: i + 1] + "́" + w[i + 1:]).upper())
    # fuzz: random strings over Cyrillic (both cases), '+', ь/ъ-heavy clusters, Latin capitals that look
    # like the rules' internal placeholders (T D N ... must be dropped, not treated as hard consonants)
    alpha = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя" * 3 + "АЕЁИОУЫЭЮЯТДСЖ" + "++" + "TDNSZRLMFPBVÇŚŜƬǱƐЭ\u0301- '"
    for _ in range(50000):
        out.append("".join(rnd.choice(alpha) for _ in range(rnd.randint(1, 14))))
    # every prefix/stem/exception/clitic with every stress position, plus inflected forms
    special = list(rules.EXCEPTIONS) + list(rules.HARD_STEMS) + list(rules.CLITICS) + list(rules.MONO_STRESSED)
    special += list(rules._OGO_KEEP_G) + list(rules._OE_FULL) + ["его", "кого", "того", "чего", "сего", "всё", "ж"]
    special += [p + "с" + x for p in rules._GEM_PREFIXES for x in ("сора", "тать", "делать", "звать", "шить")]
    for w in list(special):
        special += [w + e for e in ("а", "е", "ого", "ться", "тся", "ский", "ство", "ие", "ое")]
    for w in special:
        vs = [i for i, c in enumerate(w) if c in VOWELS]
        out.append(w)
        out.extend(w[:i] + "+" + w[i:] for i in vs)
    return out


def run_words(inputs: list[str], name: str, show: int) -> tuple[int, float, float]:
    t = time.perf_counter()
    py = [rules.word_to_ipa(w) for w in inputs]
    dpy = time.perf_counter() - t
    rs, drs = rust([], inputs)
    nbad = compare(name, inputs, py, rs, show)
    return nbad, dpy, drs


# ---------------------------------------------------------------------------
# B. sentences: lexicon configurations
# ---------------------------------------------------------------------------
TABLES = ["accents", "yo", "homographs", "suffix_stress", "train_words", "homograph_ctx", "yo_homographs"]


def write_tables(d: str, tables: dict[str, list[tuple[str, object]]]) -> None:
    os.makedirs(d, exist_ok=True)
    for t in TABLES:
        with gzip.open(os.path.join(d, f"{t}.tsv.gz"), "wt", encoding="utf-8") as f:
            for k, v in tables.get(t, []):
                f.write(f"{k}\t{v}\n")


def synthetic_tables(sentences: list[str]) -> dict[str, list]:
    """Fake but deterministic tables shaped like build_ru_lexicon.py's output."""
    vocab = collections.Counter()
    bigrams = []
    for s in sentences:
        toks = [t.lower() for t in rp._TOKEN.findall(rp._expand_numbers(s)) if t[0] not in rp._KEEP_PUNCT]
        vocab.update(toks)
        bigrams.extend(zip(["<s>"] + toks, toks + ["</s>"]))
    for kind, text, slots in load_gold():
        for t in re.findall(r"[а-яёА-ЯЁ]+", text):
            vocab[t.lower()] += 1
    cyr = sorted(w for w in vocab if re.fullmatch(r"[а-яё]+", w))
    nv = {w: sum(c in VOWELS for c in w) for w in cyr}
    acc, train, hom, yo, yo_h, ctx = {}, {}, {}, {}, {}, {}
    # yo: е-spellings of corpus words that contain ё
    for w in cyr:
        if "ё" in w:
            yo[w.replace("ё", "е")] = w
    for e, y in [("все", "всё"), ("сел", "сёл"), ("лет", "лёт"), ("чем", "чём"), ("небо", "нёбо"),
                 ("далеко", "далёко"), ("узнает", "узнаёт"), ("берет", "берёт"), ("осел", "осёл")]:
        yo_h[e] = y
        yo.pop(e, None)  # build_ru_lexicon.py: yo excludes the yo-homographs
    for w in cyr:
        x = h(w)
        if nv[w] == 0:
            continue
        if x % 10 < 7:
            # stress from the end depends on the ending, so the suffix table fills up
            from_end = h(w[-3:]) % 3 if x % 13 else h(w[-4:]) % 2
            k = max(0, nv[w] - 1 - from_end)
            if x % 50 == 0:
                k = nv[w] + 1  # out-of-range index: Python clamps with min()
            acc[w] = k
        elif x % 10 == 7:
            train[w] = x % (nv[w] + 1)
        if nv[w] >= 2 and x % 17 == 0:
            c = list(range(nv[w])) + [nv[w] + 2]
            random.Random(x).shuffle(c)
            hom[w] = ",".join(map(str, c[: 2 + x % 2]))
    for w, c in [("замок", "1,0"), ("дела", "1,0"), ("году", "1,0"), ("стоит", "1,0"), ("белки", "0,1")]:
        hom[w] = c
    for a, b in bigrams:
        if b in hom and h(a + "|" + b) % 3 == 0:
            ctx[f"p|{a}|{b}"] = h(a + b) % (nv.get(b, 2) + 1)
        if a in hom and h(a + "#" + b) % 3 == 0:
            ctx[f"n|{a}|{b}"] = h(b + a) % (nv.get(a, 2) + 1)
    # suffix table exactly as build_ru_lexicon.py derives it from the accents rows
    rows = [(w, k) for w, k in acc.items()]
    suf = collections.defaultdict(collections.Counter)
    for w, k in rows:
        n = sum(c in VOWELS for c in w)
        if n < 2:
            continue
        for L in (3, 4, 5, 6):
            if len(w) > L:
                suf[w[-L:]][n - 1 - k] += 1
    suf_rows = [(s, c.most_common(1)[0][0]) for s, c in suf.items()
                if sum(c.values()) >= 5 and c.most_common(1)[0][1] / sum(c.values()) >= 0.6]
    return {
        "accents": sorted(acc.items()), "yo": sorted(yo.items()), "yo_homographs": sorted(yo_h.items()),
        "homographs": sorted(hom.items()), "homograph_ctx": sorted(ctx.items()),
        "suffix_stress": sorted(suf_rows), "train_words": sorted(train.items()),
    }


def py_config(lexdir: str, latin: str) -> None:
    rp.LEXICON_DIR = lexdir
    rp._LEX = None
    rp._CTX = None
    rp._ENGINE = False  # legacy RUAccent+RUPhon engine: unavailable (as when ruaccent is not installed)
    if latin == "keep":
        rp._latin_to_ipa = lambda words: list(words)  # what Python does when eSpeak is missing
    else:
        rp._latin_to_ipa = lambda words: []


def run_sentences(name: str, sentences: list[str], lexdir: str, rust_lex: str | None, latin: str,
                  show: int, report: bool = False) -> tuple[int, float, float]:
    py_config(lexdir, latin)
    assert not rp._ctx().om_ok and not rp._ctx().yo_ok
    t = time.perf_counter()
    py, tiers = [], collections.Counter()
    errors = []
    for s in sentences:
        try:
            py.append(rp.phonemize_russian(s))
        except Exception as e:  # e.g. IndexError on numbers >= 10**12 (no scale word)
            py.append(None)
            errors.append((s, f"{type(e).__name__}: {e}"))
            continue
        if report:
            tiers.update(t for _, t, _ in rp.last_report())
    dpy = time.perf_counter() - t
    args = ["--sent"] + (["--lex", rust_lex] if rust_lex else []) + (["--latin-keep"] if latin == "keep" else [])
    rs, drs = rust(args, sentences)
    keep = [i for i, p in enumerate(py) if p is not None]
    if errors:
        print(f"    Python raises on {len(errors)} inputs (excluded from the count), e.g.:")
        for s, e in errors[:2]:
            print(f"      {s!r}: {e}\n        Rust: {rs[sentences.index(s)]!r}")
    nbad = compare(name, [sentences[i] for i in keep], [py[i] for i in keep], [rs[i] for i in keep], show)
    if report:
        print("    python tiers:", dict(tiers.most_common()))
    return nbad, dpy, drs


def compare_reports(sentences: list[str], lexdir: str, rust_lex: str | None, show: int) -> int:
    py_config(lexdir, "keep")
    py = []
    for s in sentences:
        try:
            rp.phonemize_russian(s)
            py.append(rp.last_report())
        except Exception:
            py.append(None)
    data = "".join(l + "\n" for l in sentences).encode()
    args = [RU_CHECK, "--report", "--latin-keep"] + (["--lex", rust_lex] if rust_lex else [])
    out = subprocess.run(args, input=data, capture_output=True, check=True).stdout.decode().split("\n")
    rs, cur, k = [], None, 0
    while k < len(out):
        if cur is None:
            if out[k] == "" and k == len(out) - 1:
                break
            cur = []
            k += 1  # the ipa line
            continue
        if out[k] == "":
            rs.append(cur)
            cur = None
        else:
            w, t, p = out[k].split("\t")
            cur.append((w, t, p))
        k += 1
    bad = 0
    for s, a, b in zip(sentences, py, rs):
        if a is not None and [tuple(x) for x in a] != b:
            bad += 1
            if bad <= show:
                print(f"    REPORT MISMATCH {s!r}\n      py {a}\n      rs {b}")
    print(f"  stress_report: {len(sentences) - bad}/{len(sentences)} sentences identical (word, tier, ipa)")
    return bad


# ---------------------------------------------------------------------------
def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--words", type=int, default=0, help="limit the Wiktionary list (0 = all)")
    ap.add_argument("--sentences", type=int, default=0, help="limit ru.txt (0 = all)")
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument("--show", type=int, default=20)
    a = ap.parse_args()
    if not a.no_build:
        subprocess.run(["cargo", "build", "--release", "--bin", "ru_check", "--bin", "build_lex"], cwd=CRATE, check=True)

    total_bad = 0
    print("== A. word_to_ipa ==")
    gw, hit, tot = gold_words()
    print(f"  (Python rules vs gold.tsv references: {hit}/{tot} gold words reachable with the implied stress)")
    nb, _, _ = run_words(gw, "gold.tsv words", a.show)
    total_bad += nb
    sw = synthetic_words(a.words or None)
    nb, dpy, drs = run_words(sw, "synthetic Wiktionary words", a.show)
    total_bad += nb
    print(f"  speed (incl. process I/O for Rust): python {len(sw) / dpy:,.0f} words/s, rust {len(sw) / drs:,.0f} words/s")
    print("  rust in-process:", rust_bench([], sw[:200000], 3))

    print("== B. phonemize (sentence pipeline, no ONNX models) ==")
    with open(UD, encoding="utf-8") as f:
        sentences = [l for l in f.read().split("\n") if l.strip()]
    sentences = [s for s in sentences if safe_line(s)]
    if a.sentences:
        sentences = sentences[: a.sentences]
    tmp = tempfile.mkdtemp(prefix="ru_parity_")
    empty = os.path.join(tmp, "empty")
    write_tables(empty, {})
    synth = os.path.join(tmp, "synthetic")
    write_tables(synth, synthetic_tables(sentences))
    synth_lex = os.path.join(tmp, "synthetic_lex")
    subprocess.run([sys.executable, os.path.join(HERE, "ru_build_lex.py"), synth, synth_lex], check=True,
                   capture_output=True)
    extra = [
        "В 1945 году закончилась война, и 9 мая стал праздником.",
        "Он повесил замок на дверь. Солнце уже село за старый замок.",
        "Цена выросла на 3,5 % и составила 1 250 000 $ — «рекорд»… (почти)!",
        "Все пришли, и всё было хорошо?! Ну...", "все", "Все они были там; все это знали: 21 книга, 2,125 кг.",
        "OK, это test-drive по-русски: e-mail don't 22.10.2026 №5 & 7+8=15 °C", "", "   ", "…", "123", "١٢٣ ٤",
        "Вот ж ты! В к с.", "ts dz tʃ", "Ёлка, ЁЖИК и щётка", "Он сказал: 1000000000000 раз.",
    ]
    rnd = random.Random(7)
    pieces = ["замок", "все", "сел", "Всё", "году", "мая", "года", "человека", "книга", "ts", "tʃ", "dz", "dʒ", "TS",
              "t͡s", "ſ", "test", "don't", "x'y", "1", "21", "2", "1945", "3,5", "0.25", "1 000", "12\u00a0345", "%",
              "$", "€", "№", "+", "=", "&", "°", "…", "...", "?!", "!?", "—", "«", "»", "(", ")", "\"", "'", "´", "`",
              ",", ".", ":", ";", "-", " ", " ", "\u00a0", "\u2009", "\t", "١٢", "е\u0301", "Ё", "ИТ-", "_", "|", "^",
              "😀", "♥", "©", "привет", "щётка", "солнце", "ж", "в", "к", "с", "2000", "2100", "31", "32", "11", "12"]
    fuzz = ["".join(rnd.choice(pieces) for _ in range(rnd.randint(1, 12))) for _ in range(20000)]
    extra += [f for f in fuzz if safe_line(f)]
    results = {}
    for name, d, rl in [("empty", empty, None), ("synthetic", synth, synth_lex)]:
        nb, dpy, drs = run_sentences(f"ru.txt [{name} lexicon]", sentences + extra, d, rl, "keep", a.show, report=True)
        total_bad += nb
        results[name] = (dpy, drs)
        total_bad += compare_reports(sentences[:3000] + extra, d, rl, a.show)
    nb, _, _ = run_sentences("ru.txt [empty lexicon, Latin dropped]", sentences + extra, empty, None, "drop", a.show)
    total_bad += nb
    n = len(sentences) + len(extra)
    for name, (dpy, drs) in results.items():
        print(f"  speed [{name}] (incl. process I/O for Rust): python {n / dpy:,.1f} sent/s, rust {n / drs:,.1f} sent/s")
    print("  rust in-process [synthetic]:", rust_bench(["--sent", "--lex", synth_lex, "--latin-keep"], sentences, 3))
    print(f"\nTOTAL mismatches: {total_bad}")
    return 1 if total_bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
