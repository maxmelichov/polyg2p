#!/usr/bin/env python3
"""Copy the permissively licensed data files into the Python wheel.

The wheel bundles ``bindings/python/python/polyg2p/data/`` (see pyproject.toml),
so ``Phonemizer()`` works with no arguments. Only Apache/BSD-compatible files
go in; the CC BY-SA Wiktionary lexicons (de, it, uk, nl, sv, pt) stay out and
can be added at run time with ``POLYG2P_DATA`` or ``Phonemizer(data_dir=...)``.
The heteronym context models (``<lang>-ctx.g2p``, docs/heteronyms.md) have CC BY-SA labels in their chain
and stay out for the same reason; ``ru-ctx.g2p`` is compiled into the crate.

usage: tools/bundle_wheel_data.py [DATA_DIR]   (default: data-out)
"""
from __future__ import annotations

import os
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEST = os.path.join(ROOT, "bindings", "python", "python", "polyg2p", "data")

# file -> licence of its source
ALLOW = {
    "en.lex": "CMUdict (BSD-style)",
    "en.g2p": "trained on CMUdict (BSD-style)",
    "ru_accents.lex": "RUAccent dictionaries (Apache-2.0)",
    "ru_homographs.lex": "RUAccent dictionaries (Apache-2.0)",
    "ru_homograph_ctx.lex": "RUAccent dictionaries (Apache-2.0)",
    "ru_suffix.lex": "RUAccent dictionaries (Apache-2.0)",
    "ru_train.lex": "RUAccent dictionaries (Apache-2.0)",
    "ru_yo.lex": "RUAccent dictionaries (Apache-2.0)",
    "ru_yo_homographs.lex": "RUAccent dictionaries (Apache-2.0)",
    "he-int8.g2p": "distilled from RenikudPlus (Apache-2.0)",
}


def main() -> None:
    src = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "data-out")
    missing = [f for f in ALLOW if not os.path.isfile(os.path.join(src, f))]
    if missing:
        sys.exit(f"missing in {src}: {', '.join(missing)} (run tools/build_data.py first)")
    if os.path.isdir(DEST):
        shutil.rmtree(DEST)
    os.makedirs(DEST)
    total = 0
    for f, lic in ALLOW.items():
        shutil.copy2(os.path.join(src, f), os.path.join(DEST, f))
        size = os.path.getsize(os.path.join(DEST, f))
        total += size
        print(f"{f:24s} {size / 1e6:6.1f} MB  {lic}")
    # the licence texts travel with the data: LICENSE (Apache-2.0) and NOTICE (CMUdict, RUAccent, Britfone, ... notices)
    for f in ("LICENSE", "NOTICE"):
        shutil.copy2(os.path.join(ROOT, f), os.path.join(DEST, f))
        print(f"{f:24s} licence text")
    print(f"{'total':24s} {total / 1e6:6.1f} MB -> {DEST}")


if __name__ == "__main__":
    main()
