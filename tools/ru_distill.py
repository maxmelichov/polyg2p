#!/usr/bin/env python3
"""Teacher labels for the Russian context student (homograph stress + ё).

Input: JSONL written by running Russian-G2P's pipeline WITH RUAccent's two
ONNX context models, one object per sentence:
    {"t": text, "ipa": ..., "r": [[word, tier, ipa], ...],
     "d": [[word, "+"-accented word, tier, yo_label, marked_sentence], ...]}
("d" is every `_stress_word` decision in order; `marked_sentence` is the
RUAccent-style sentence with the word wrapped in <w></w>, present for
(ё-)homographs.)

Output (one JSON per line, for tools/train_ru.py):
    t   the sentence exactly as the Rust side builds it for the context models
        (lower case, words space-separated, punctuation glued to the word before)
    s   per char: '1' stressed vowel, '0' other char of a word with known stress, '-' ignore
    y   per char: '1' this е is ё, '0' this е stays е, '-' ignore
    w   per char weight digit: '3' on words the teacher *models* decided, '1' elsewhere
    hm  [[start, len, [candidate char idx...], chosen char idx], ...] homograph-model words
    yh  [[start, len, 1/0], ...] ё-homograph words the ё tagger decided

usage: ru_distill.py RU_LEXICON_DIR part*.jsonl --out build-data/ru_ctx.jsonl
"""
from __future__ import annotations

import argparse
import gzip
import json
import os
import random
import sys

VOWELS = "аеёиоуыэюя"


def read_tsv(path):
    out = {}
    with gzip.open(path, "rt", encoding="utf8") as f:
        for line in f:
            k, _, v = line.rstrip("\n").partition("\t")
            out[k] = v
    return out


def vowel_char_positions(w):
    return [i for i, c in enumerate(w) if c in VOWELS]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("lexdir")
    ap.add_argument("parts", nargs="+")
    ap.add_argument("--out", required=True)
    ap.add_argument("--seed", type=int, default=0)
    a = ap.parse_args()
    hom = read_tsv(os.path.join(a.lexdir, "homographs.tsv.gz"))
    yo_hom = read_tsv(os.path.join(a.lexdir, "yo_homographs.tsv.gz"))
    rows = []
    stats = {"rows": 0, "skipped": 0, "hm": 0, "yh": 0, "err": 0}
    for part in a.parts:
        with open(part, encoding="utf8") as f:
            for line in f:
                r = json.loads(line)
                d = r.get("d") or []
                if any(x[2] == "error" for x in r.get("r", [])):
                    stats["err"] += 1
                    continue
                marked = next((x[4] for x in d if x[4]), None)
                if marked is None:
                    stats["skipped"] += 1
                    continue
                sent = marked.replace("<w>", "", 1).replace("</w>", "", 1)
                n = len(sent)
                s = ["-"] * n
                y = ["-"] * n
                wt = ["1"] * n
                hm, yh = [], []
                pos = 0
                ok = True
                for w, acc, tier, yo_label, _m in d:
                    i = sent.find(w, pos)
                    if i < 0:
                        ok = False
                        break
                    pos = i + len(w)
                    clean = acc.replace("+", "")
                    if len(clean) != len(w):
                        continue
                    heavy = tier == "homograph-model" or (w in yo_hom and yo_label is not None)
                    if heavy:
                        for p in range(i, i + len(w)):
                            wt[p] = "3"
                    # stress: only tiers whose stress is known, not guessed
                    if "+" in acc and tier in ("dict", "mono", "homograph-model", "yo", "train", "context"):
                        k = acc.index("+")
                        for p in range(len(w)):
                            s[i + p] = "0"
                        s[i + k] = "1"
                        if tier == "homograph-model" and w in hom:
                            vps = vowel_char_positions(w)
                            cands = [vps[int(c)] for c in hom[w].split(",") if c and int(c) < len(vps)]
                            if k in cands and len(cands) > 1:
                                hm.append([i, len(w), cands, k])
                    # ё: every е of the word is labelled from the teacher's spelling
                    if "е" in w and "ё" not in w:
                        for p, ch in enumerate(w):
                            if ch == "е":
                                y[i + p] = "1" if clean[p] == "ё" else "0"
                        if w in yo_hom and yo_label is not None:
                            yh.append([i, len(w), 1 if "ё" in clean else 0])
                if not ok:
                    stats["skipped"] += 1
                    continue
                stats["rows"] += 1
                stats["hm"] += len(hm)
                stats["yh"] += len(yh)
                rows.append({"t": sent, "s": "".join(s), "y": "".join(y), "w": "".join(wt), "hm": hm, "yh": yh})
    random.Random(a.seed).shuffle(rows)
    with open(a.out, "w", encoding="utf8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(stats, file=sys.stderr)


if __name__ == "__main__":
    main()
