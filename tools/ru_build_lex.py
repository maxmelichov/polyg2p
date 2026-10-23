#!/usr/bin/env python3
"""Convert Russian-G2P's ru_lexicon/*.tsv.gz into polyg2p .lex files for src/ru.rs.

    python3 ru_build_lex.py RU_LEXICON_DIR OUT_DIR [--build-lex PATH]

RU_LEXICON_DIR is what scripts/build_ru_lexicon.py writes (accents.tsv.gz, yo.tsv.gz,
...). Each table becomes OUT_DIR/<name>.lex via polyg2p's build_lex binary
(default: <repo>/target/release/build_lex):

    accents.tsv.gz        -> ru_accents.lex
    yo.tsv.gz             -> ru_yo.lex
    yo_homographs.tsv.gz  -> ru_yo_homographs.lex
    homographs.tsv.gz     -> ru_homographs.lex
    homograph_ctx.tsv.gz  -> ru_homograph_ctx.lex
    suffix_stress.tsv.gz  -> ru_suffix.lex
    train_words.tsv.gz    -> ru_train.lex

The Python loader keeps the LAST value of a duplicated key (dict assignment);
build_lex keeps all values tab-joined and Rust reads the whole value, so
duplicates are resolved here (last one wins) and values containing a tab are
cut at it (Python's int()/split would have failed or kept them; build_ru_lexicon.py
never writes either). Empty tables still produce a valid (empty) .lex file.
"""

from __future__ import annotations

import argparse
import gzip
import os
import subprocess
import sys
import tempfile

TABLES = {
    "accents.tsv.gz": "ru_accents.lex",
    "yo.tsv.gz": "ru_yo.lex",
    "yo_homographs.tsv.gz": "ru_yo_homographs.lex",
    "homographs.tsv.gz": "ru_homographs.lex",
    "homograph_ctx.tsv.gz": "ru_homograph_ctx.lex",
    "suffix_stress.tsv.gz": "ru_suffix.lex",
    "train_words.tsv.gz": "ru_train.lex",
}


def read_tsv(path: str) -> dict[str, str]:
    """Exactly russian_phonemizer._read_tsv."""
    out: dict[str, str] = {}
    with gzip.open(path, "rt", encoding="utf-8") as f:
        for line in f:
            k, _, v = line.rstrip("\n").partition("\t")
            out[k] = v
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("src")
    ap.add_argument("out")
    ap.add_argument("--build-lex", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target", "release", "build_lex"))
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    for name, lex in TABLES.items():
        path = os.path.join(a.src, name)
        if not os.path.isfile(path):
            print(f"[skip] {name}: not found", file=sys.stderr)
            continue
        d = read_tsv(path)
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".tsv", delete=False) as t:
            n = 0
            for k, v in d.items():
                if not k or "\n" in k:
                    continue
                # build_lex skips lines without a tab; an empty value is kept as ""
                t.write(f"{k}\t{v.split(chr(9))[0]}\n")
                n += 1
            tmp = t.name
        try:
            subprocess.run([a.build_lex, tmp, os.path.join(a.out, lex)], check=True)
        finally:
            os.unlink(tmp)
        print(f"{name}: {n} entries -> {os.path.join(a.out, lex)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
