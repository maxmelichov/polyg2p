"""Rust <-> PyTorch parity of a trained heteronym model (acceptance A4) and of the feature hashing.

usage: homograph_parity.py CORPUS/data MODEL_DIR [--data data-out] [--n 1000]
  1. bucket ids: Python `feature_ids` vs `en_homograph --units --ids N` on --n gold sentences
  2. f32 file: Rust masked logits vs PyTorch on the dev split (max abs difference)
  3. int8 file: Rust argmax vs PyTorch f32 argmax on the dev split
MODEL_DIR is a train_homograph.py output folder (best.pt, en_homograph_f32.g2p, en_homograph.g2p).
"""
import os, subprocess, sys, tempfile

import numpy as np
import torch

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import train_homograph as T  # noqa: E402

ROOT = T.ROOT
EXE = os.path.join(ROOT, "target", "release", "en_homograph")


def run(args, items, data):
    inp = "".join(f"{s}\t{a}\t{b}\n" for s, a, b in items)
    r = subprocess.run([EXE] + args + ["--data", data], input=inp, capture_output=True, text=True, check=True)
    return r.stdout.split("\n")[:len(items)]


def data_dir(base, model_file, tmp):
    d = tempfile.mkdtemp(dir=tmp)
    for f in ("en.lex", "en.g2p"):
        if os.path.exists(os.path.join(base, f)):
            os.symlink(os.path.abspath(os.path.join(base, f)), os.path.join(d, f))
    os.symlink(os.path.abspath(model_file), os.path.join(d, "en_homograph.g2p"))
    return d


def main():
    corpus, mdir = sys.argv[1:3]
    base = sys.argv[sys.argv.index("--data") + 1] if "--data" in sys.argv else os.path.join(ROOT, "data-out")
    n = int(sys.argv[sys.argv.index("--n") + 1]) if "--n" in sys.argv else 1000
    table = T.load_table(os.path.join(ROOT, "data", "en_homograph.tsv"))
    gold = T.load_gold(corpus, table, "train")
    fit, dev = T.split_dev(gold, table)
    # 1. bucket ids
    items = [(s, a, b) for s, a, b, _ in gold[:: max(1, len(gold) // n)][:n]]
    units = run(["--units"], items, base)
    ids = run(["--units", "--ids", str(T.N_BUCKETS)], items, base)
    bad = tot = 0
    for u, i in zip(units, ids):
        if not u:
            continue
        f_u, f_i = u.split("\t"), i.split("\t")
        t = int(f_u[0])
        for k, (unit, rid) in enumerate(zip(f_u[1:], f_i[1:])):
            tot += 1
            bad += T.feature_ids(unit, k == t) != [int(x) for x in rid.split(",")]
    print(f"bucket ids: {tot} units of {len(items)} sentences, {bad} differ")
    # 2. + 3. logits
    model = T.HomographModel(len(table))
    model.load_state_dict(torch.load(os.path.join(mdir, "best.pt")))
    model.eval()
    dev = dev[:2000]
    with tempfile.TemporaryDirectory() as tmp:
        d32 = data_dir(base, os.path.join(mdir, "en_homograph_f32.g2p"), tmp)
        d8 = data_dir(base, os.path.join(mdir, "en_homograph.g2p"), tmp)
        its = [(s, a, b) for s, a, b, _ in dev]
        r32, r8 = run(["--logits"], its, d32), run(["--logits"], its, d8)
        ex_units = T.rust_units(its, base)
        maxdiff, agree8, agree32, cnt = 0.0, 0, 0, 0
        word_rows = {}
        for i, (w, *_r) in enumerate(table):
            word_rows.setdefault(w, []).append(i)
        ipa_row = {(w, ipa): i for i, (w, r, c, ipa) in enumerate(table)}
        for (s, a, b, row), u, l32, l8 in zip(dev, ex_units, r32, r8):
            if u is None or not l32:
                continue
            arr, t = T.encode(u[1], u[0])
            ids_, ln, tg = T.collate([arr], [t])
            with torch.no_grad():
                lg = model(ids_, ln, tg)[0]
            w = table[row][0]
            rows = word_rows[w]
            py = {i: lg[i].item() for i in rows}
            parse = lambda line: {ipa_row[(w, kv.rsplit(":", 1)[0])]: float(kv.rsplit(":", 1)[1]) for kv in line.split(" ")}
            p32, p8 = parse(l32), parse(l8)
            maxdiff = max(maxdiff, max(abs(py[i] - p32[i]) for i in rows))
            best = max(rows, key=lambda i: py[i])
            agree32 += max(rows, key=lambda i: p32[i]) == best
            agree8 += max(rows, key=lambda i: p8[i]) == best
            cnt += 1
    print(f"f32: max |Rust - PyTorch| logit difference {maxdiff:.2e} over {cnt} dev sentences; argmax agrees on {agree32}/{cnt}")
    print(f"int8: argmax agrees with PyTorch f32 on {agree8}/{cnt} = {agree8 / cnt:.2%}")


if __name__ == "__main__":
    main()
