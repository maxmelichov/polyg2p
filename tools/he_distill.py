"""Teacher labelling for the Hebrew student tagger: run RenikudPlus over a large
unvocalized sentence corpus and store its per-letter labels.

Two sub-commands:

  he_distill.py prep  CORPUS... --out build-data/he_sents.txt
      Strip niqqud / morpheme bars, dedupe, filter by length, shuffle (seed 0).

  he_distill.py label build-data/he_sents.txt build-data/he_labels.jsonl
      [--start 0] [--end N] [--procs 8] [--verify 500]
      Label rows [start, end) with the teacher (one ONNX session per process,
      one intra-op thread each: the exported graph only takes batch size 1).
      Appends to the output (resumable: --start defaults to the rows already
      written). Throughput is logged to stderr.

Each output line is JSON: {"t": text, "c": cons, "v": vow, "s": stress, "ipa": ipa}
  t    : the exact string the teacher's network saw (after number expansion,
         grapheme normalization, NFD, niqqud strip) -- what the student sees too
  c    : one char per char of `t`: consonant class id as base-36 digit, or '-'
         for non-Hebrew positions (ids are the teacher's `consonant_vocab`)
  v    : vowel class id 0-5 per char ('-' on non-Hebrew)
  s    : '1' where the teacher put the word stress, '0' elsewhere ('-' non-Hebrew)
  ipa  : the teacher's IPA for `t`, byte-identical to G2P.phonemize(t)
         (verified on the first --verify rows)

The labels are the exact-MAP cascade decode (RenikudPlus default). The geresh
rule (ג' -> dʒ etc.) is NOT folded into `c`: it is a rendering rule applied
from the text, identically in `render()` below and in the Rust renderer.
"""
from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys
import time
import unicodedata

NIQQUD_RE = re.compile("[֑-ׇֽֿׁׂׅׄ]")
HEB = re.compile("[א-ת]")
B36 = "0123456789abcdefghijklmnopqrstuvwxyz"

# ---------------------------------------------------------------- rendering
CONS_VOCAB = {0: "∅", 1: "b", 2: "v", 3: "d", 4: "h", 5: "z", 6: "χ", 7: "t", 8: "j",
              9: "k", 10: "l", 11: "m", 12: "n", 13: "s", 14: "f", 15: "p", 16: "ts",
              17: "tʃ", 18: "w", 19: "ʔ", 20: "ɡ", 21: "ʁ", 22: "ʃ", 23: "ʒ", 24: "dʒ"}
VOW_VOCAB = {0: "∅", 1: "a", 2: "e", 3: "i", 4: "o", 5: "u"}
GERESH_MAP = {"ץ": "tʃ", "צ": "tʃ", "ג": "dʒ", "ז": "ʒ"}


def is_heb(ch: str) -> bool:
    return "א" <= ch <= "ת"


def render(text: str, cons: str, vow: str, stress: str) -> str:
    """Mirror of RenikudPlus `_phonemize_window` from stored labels."""
    out = []
    n = len(text)
    for i, ch in enumerate(text):
        if not is_heb(ch):
            if ch in ("'", '"'):
                continue
            out.append(ch)
            continue
        c = CONS_VOCAB[int(cons[i], 36)]
        v = VOW_VOCAB[int(vow[i])]
        st = stress[i] == "1"
        if ch in GERESH_MAP and i + 1 < n and text[i + 1] == "'":
            c = GERESH_MAP[ch]
        word_final = i + 1 >= n or not text[i + 1].isalpha()
        if ch == "ח" and word_final and v == "a":
            out.append(("ˈ" if st else "") + "aχ")
        else:
            s = ""
            if c != "∅":
                s += c
            if st and v != "∅":
                s += "ˈ"
            if v != "∅":
                s += v
            out.append(s)
    return "".join(out)


# ---------------------------------------------------------------- prep
def prep(paths: list[str], out: str, min_len: int, max_len: int, seed: int) -> None:
    seen: set[str] = set()
    rows: list[str] = []
    t0 = time.time()
    for p in paths:
        with open(p, encoding="utf8") as f:
            for line in f:
                s = NIQQUD_RE.sub("", unicodedata.normalize("NFD", line.strip()))
                s = s.replace("|", "")
                s = re.sub(r"\s+", " ", s).strip()
                if not (min_len <= len(s) <= max_len) or not HEB.search(s):
                    continue
                if s in seen:
                    continue
                seen.add(s)
                rows.append(s)
        print(f"{p}: {len(rows)} unique rows so far ({time.time()-t0:.0f}s)", file=sys.stderr)
    random.Random(seed).shuffle(rows)
    with open(out, "w", encoding="utf8") as f:
        for s in rows:
            f.write(s + "\n")
    print(f"wrote {len(rows)} rows to {out}", file=sys.stderr)


# ---------------------------------------------------------------- label
_G = None


def _init_worker():
    global _G
    import onnxruntime as ort
    from renikud_onnx import G2P
    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    _G = G2P(session_options=so)


def _label_one(text: str) -> dict | None:
    from renikud_onnx import (contains_digit, normalize_numbers, normalize_graphemes,
                              strip_niqqud, split_for_decode)
    g = _G
    try:
        if contains_digit(text):
            text = normalize_numbers(text)
        text = normalize_graphemes(text)
        norm = strip_niqqud(unicodedata.normalize("NFD", text))
        if len(split_for_decode(norm)) != 1:
            return None
        offsets, cp, vp, stressed, _cl = g._forward(norm)
        n = len(norm)
        cons = ["-"] * n
        vow = ["-"] * n
        st = ["-"] * n
        for t, (a, b) in enumerate(offsets):
            if b - a != 1 or not is_heb(norm[a]):
                continue
            cons[a] = B36[int(cp[t])]
            vow[a] = str(int(vp[t]))
            st[a] = "1" if t in stressed else "0"
        cons, vow, st = "".join(cons), "".join(vow), "".join(st)
        return {"t": norm, "c": cons, "v": vow, "s": st, "ipa": render(norm, cons, vow, st)}
    except Exception as e:  # noqa: BLE001
        print(f"skip {text[:40]!r}: {e}", file=sys.stderr)
        return None


def _verify_one(row: dict) -> bool:
    return _G.phonemize(row["t"]) == row["ipa"]


def label(src: str, out: str, start: int | None, end: int | None, procs: int, verify: int) -> None:
    from multiprocessing import Pool
    if start is None:
        start = 0
        if os.path.exists(out):
            with open(out, encoding="utf8") as f:
                start = sum(1 for _ in f)
    todo = []
    with open(src, encoding="utf8") as f:
        for i, line in enumerate(f):
            if end is not None and i >= end:
                break
            if i >= start:
                todo.append(line.rstrip("\n"))
    end = start + len(todo)
    print(f"labelling rows {start}..{end} ({len(todo)}) with {procs} procs", file=sys.stderr)
    t0 = time.time()
    done = 0
    with Pool(procs, initializer=_init_worker) as pool, open(out, "a", encoding="utf8") as f:
        if verify and start == 0:
            rows = [r for r in pool.map(_label_one, todo[:verify], chunksize=4) if r]
            ok = pool.map(_verify_one, rows, chunksize=4)
            bad = [r["t"] for r, o in zip(rows, ok) if not o]
            print(f"verify: {sum(ok)}/{len(rows)} rendered IPA == G2P.phonemize", file=sys.stderr)
            for b in bad[:5]:
                print("  mismatch:", b, file=sys.stderr)
            if bad:
                raise SystemExit("renderer does not reproduce the teacher; fix render()")
        for r in pool.imap(_label_one, todo, chunksize=8):
            done += 1
            if r is not None:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
            if done % 2000 == 0:
                el = time.time() - t0
                print(f"{done}/{len(todo)} {done/el:.1f} sent/s elapsed {el/60:.1f}m", file=sys.stderr)
                f.flush()
    el = time.time() - t0
    print(f"done {done} rows in {el/60:.1f}m ({done/max(el,1e-9):.1f} sent/s)", file=sys.stderr)


def main() -> None:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("prep")
    p.add_argument("corpus", nargs="+")
    p.add_argument("--out", required=True)
    p.add_argument("--min-len", type=int, default=8)
    p.add_argument("--max-len", type=int, default=300)
    p.add_argument("--seed", type=int, default=0)
    l = sub.add_parser("label")
    l.add_argument("src")
    l.add_argument("out")
    l.add_argument("--start", type=int, default=None)
    l.add_argument("--end", type=int, default=None)
    l.add_argument("--procs", type=int, default=8)
    l.add_argument("--verify", type=int, default=500)
    a = ap.parse_args()
    if a.cmd == "prep":
        prep(a.corpus, a.out, a.min_len, a.max_len, a.seed)
    else:
        label(a.src, a.out, a.start, a.end, a.procs, a.verify)


if __name__ == "__main__":
    main()
