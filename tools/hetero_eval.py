#!/usr/bin/env python3
"""Rules vs context model on a gold set, through the Rust path (target/release/hetero_check).

  hetero_eval.py --lang fr --gold gold/fr.tsv --data models/fr [--data-f32 models/fr_f32]

gold rows (tab separated): sentence, word, nth, k, source
  sentence  raw text; word = the table word (lower case); nth = 0-based occurrence among the engine's words
  k         gold class of the table (data/hetero_<lang>.tsv); a decision counts as right when the PHONEMES of
            the predicted class equal the phonemes of the gold class (classes that sound the same are not errors)
  source    a UD file name for human gold, `silver:<rule>` for cue-rule labels, `model:<who>` for model-labelled rows;
            sources are reported apart and never mixed in one number.
Gold sets are not committed; build them with tools/hetero_label.py gold (UD treebanks, licences in docs/heteronyms.md).
`--data` holds <lang>-ctx.g2p (the int8 file renamed, or a folder with only that file); no file = rules only.
Prints per source: n, rule accuracy, model accuracy, error cut; per word (n >= --min-word); and the words where
the model is below the rules (`--rule-list` writes them for the table's mode column).
"""
from __future__ import annotations

import argparse
import collections
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hetero_common import BIN  # noqa: E402


def run(lang, rows, data, shards=6, raw=False):
    """one hetero_check process per shard (nice 19), order preserved"""
    import concurrent.futures as cf
    cmd = ["nice", "-n", "19", BIN, lang] + (["--data", data] if data else []) + (["--raw"] if raw else [])
    parts = [rows[i::shards] for i in range(shards)]

    def one(part):
        inp = "".join(f"{r[0]}\t{r[1]}\t{r[2]}\n" for r in part)
        return subprocess.run(cmd, input=inp, capture_output=True, text=True, check=True).stdout

    with cf.ThreadPoolExecutor(shards) as ex:
        outs = list(ex.map(one, parts))
    res = [None] * len(rows)
    for si, o in enumerate(outs):
        for n, line in enumerate(o.split("\n")[:len(parts[si])]):
            c = line.split("\t")
            phs = dict(x.split("=", 1) for x in c[2].split("|")) if len(c) > 2 and c[2] else {}
            res[si + n * shards] = (c[0], c[1], phs)
    return res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lang", required=True)
    ap.add_argument("--gold", required=True)
    ap.add_argument("--data", default=None)
    ap.add_argument("--data-f32", default=None, help="a folder with the f32 file: reports int8 vs f32 agreement")
    ap.add_argument("--min-word", type=int, default=50)
    ap.add_argument("--raw", action="store_true", help="the model's own class even for words with mode=rule (gate 4 uses it)")
    ap.add_argument("--rule-list", default=None)
    a = ap.parse_args()
    rows = [l.rstrip("\n").split("\t") for l in open(a.gold, encoding="utf8") if l.strip()]
    empty = os.path.join(os.path.dirname(os.path.abspath(a.gold)), "_nomodel")
    os.makedirs(empty, exist_ok=True)
    base = run(a.lang, rows, empty)
    mod = run(a.lang, rows, a.data, raw=a.raw)
    f32 = run(a.lang, rows, a.data_f32, raw=a.raw) if a.data_f32 else None
    st = collections.defaultdict(lambda: [0, 0, 0])
    wd = collections.defaultdict(lambda: [0, 0, 0])
    agree = [0, 0]
    for r, b, m in zip(rows, base, mod):
        gk, src, w = r[3], r[4], r[1]
        if b[0] == "-":
            continue
        nomodel = m[1] == "-"
        gold_ph = m[2].get(gk)
        okr = b[2].get(b[0]) == gold_ph if b[0] != "-" else False
        okm = (not nomodel) and m[2].get(m[1]) == gold_ph
        kind = "silver" if src.startswith("silver") else ("model-labelled" if src.startswith("model") else "gold:" + ("train" if "train" in src else "dev/test"))
        targets = [st[kind], wd[(kind, w)]]
        if kind.startswith("gold"):
            targets.append(wd[("gold:all", w)])
        for d in targets:
            d[0] += 1
            d[1] += okr
            d[2] += okm
    if f32:
        for a_, b_ in zip(mod, f32):
            agree[0] += 1
            agree[1] += a_[1] == b_[1]
    if all(m[1] == "-" for m in mod):
        print("(no model file: rules only)")
    print(f"{'set':<18}{'n':>7}{'rules':>9}{'model':>9}{'err cut':>9}")
    for k, (n, r, m) in sorted(st.items()):
        er, em = 1 - r / n, 1 - m / n
        print(f"{k:<18}{n:>7}{r/n:>9.4f}{m/n:>9.4f}{(1 - em/er if er > 0 else 0):>9.2f}")
    if f32:
        print(f"int8 vs f32 class agreement {agree[1]/max(agree[0],1):.4f} (n {agree[0]})")
    bad = []
    print("\nper word (n >= %d):" % a.min_word)
    for (k, w), (n, r, m) in sorted(wd.items(), key=lambda x: -x[1][0]):
        if n >= a.min_word:
            flag = " <- below rules" if m < r else ""
            print(f"  {k:<18}{w:<16}{n:>6}  rules {r/n:.3f}  model {m/n:.3f}{flag}")
            if m < r and k == "gold:all":
                bad.append(w)
    if a.rule_list:
        open(a.rule_list, "w").write("\n".join(sorted(set(bad))) + "\n")


if __name__ == "__main__":
    main()
