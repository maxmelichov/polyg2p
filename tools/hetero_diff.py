#!/usr/bin/env python3
"""Regression check for src/hetero.rs (gate 6, speed = gate 7).

  hetero_diff.py --main BIN_MAIN --branch BIN_BRANCH --text-dir DIR --langs fr de ... [--n 20000] [--models DIR]

DIR holds one sentence per line files `tat_<iso3>.txt` (Tatoeba exports, see docs/heteronyms.md).
1. no model: the branch binary's output must be byte-identical to the main binary's on N sentences per language
   (all languages, including ones this module does not touch). Exit code 1 otherwise.
2. with --models DIR/<lang>/<lang>-ctx.g2p: every sentence whose output changes must contain a table word
   (or a word ending in a `*suffix` entry of the table); the number of changed sentences is printed.
3. sentences per second, main vs branch, no model and with model.
"""
import argparse
import os
import random
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hetero_common import load_table, lookup  # noqa: E402

ISO3 = {"fr": "fra", "de": "deu", "pt": "por", "uk": "ukr", "it": "ita", "nl": "nld", "ro": "ron", "ru": "rus", "sv": "swe",
        "el": "ell", "tr": "tur", "ca": "cat", "es": "spa", "pl": "pol", "cs": "ces", "hu": "hun", "fi": "fin", "en": "eng"}


def run(binary, lang, sents, data=None):
    cmd = [binary, lang] + (["--data", data] if data else [])
    t = time.time()
    out = subprocess.run(cmd, input="\n".join(sents) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")[:len(sents)]
    return out, time.time() - t


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--main", required=True)
    ap.add_argument("--branch", required=True)
    ap.add_argument("--text-dir", required=True)
    ap.add_argument("--langs", nargs="+", required=True)
    ap.add_argument("--n", type=int, default=20000)
    ap.add_argument("--models", default=None)
    a = ap.parse_args()
    bad = 0
    print(f"{'lang':<5}{'identical':>10}{'changed':>9}{'bad':>5}{'main s/s':>10}{'br s/s':>9}{'br+model s/s':>14}")
    for lang in a.langs:
        path = os.path.join(a.text_dir, f"tat_{ISO3[lang]}.txt")
        sents = [l.strip() for l in open(path, encoding="utf8") if 3 <= len(l.strip()) <= 300]
        random.Random(0).shuffle(sents)
        sents = sents[:a.n]
        o_main, t_main = run(a.main, lang, sents)
        o_br, t_br = run(a.branch, lang, sents)
        same = sum(x == y for x, y in zip(o_main, o_br))
        if same != len(sents):
            bad += 1
        changed = badc = 0
        t_model = float("nan")
        if a.models and os.path.exists(os.path.join(a.models, lang, f"{lang}-ctx.g2p")):
            o_m, t_model = run(a.branch, lang, sents, os.path.join(a.models, lang))
            try:
                table = load_table(lang)
            except FileNotFoundError:
                table = {}
            import re
            for s, x, y in zip(sents, o_main, o_m):
                if x != y:
                    changed += 1
                    ws = [w.lower() for w in re.findall(r"[^\W\d_]+(?:['’\-][^\W\d_]+)*", s)]
                    if not any(lookup(table, w) for w in ws):
                        badc += 1
        if badc:
            bad += 1
        print(f"{lang:<5}{same:>5}/{len(sents):<5}{changed:>9}{badc:>5}{len(sents)/t_main:>10.0f}{len(sents)/t_br:>9.0f}{len(sents)/t_model:>14.0f}")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
