"""Python<->Rust parity for the Hebrew tagger: the Rust inference (he_check)
must reproduce the torch model's constrained-decoded labels.

usage: he_parity.py outdir/he.pt outdir/vocab.json outdir/he.g2p sentences.txt [--n 1000]
  outdir/he.pt + vocab.json : the torch checkpoint (train_he.py)
  he.g2p                    : the exported model the Rust side loads
  sentences.txt             : one (unvocalized) sentence per line
Builds target/release/he_check if needed.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys

import torch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from train_he import Tagger, decode_labels, encode  # noqa: E402
from he_distill import render  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pt")
    ap.add_argument("vocab")
    ap.add_argument("g2p")
    ap.add_argument("sentences")
    ap.add_argument("--n", type=int, default=1000)
    a = ap.parse_args()
    vocab = json.load(open(a.vocab, encoding="utf8"))
    sd = torch.load(a.pt, map_location="cpu")
    d = sd["emb.weight"].shape[1]
    layers = max(int(k.split(".")[1]) for k in sd if k.startswith("enc.")) + 1
    ff = sd["enc.0.ff1.weight"].shape[0]
    max_len = sd["pos.weight"].shape[0]
    heads = None
    cfg_path = os.path.join(os.path.dirname(a.pt), "config.json")
    if os.path.exists(cfg_path):
        heads = json.load(open(cfg_path))["heads"]
    heads = heads or 4
    model = Tagger(sd["emb.weight"].shape[0], d, heads, layers, ff, max_len)
    model.load_state_dict(sd)
    model.eval()
    sents = [l.rstrip("\n") for l in open(a.sentences, encoding="utf8")][: a.n]
    # the Rust side normalizes (numbers, graphemes, NFD, niqqud) itself; feed it raw text
    exe = os.path.join(ROOT, "target", "release", "he_check")
    subprocess.run(["cargo", "build", "--release", "--bin", "he_check"], cwd=ROOT, check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    res = subprocess.run([exe, a.g2p, "--time"], input="\n".join(sents) + "\n", capture_output=True,
                         text=True, check=True)
    print(res.stderr.strip())
    rust = [l.split("\t") for l in res.stdout.rstrip("\n").split("\n")]
    assert len(rust) == len(sents), (len(rust), len(sents))
    same = same_ipa = 0
    pos_total = pos_same = 0
    shown = 0
    with torch.no_grad():
        for (t, c, v, s, ipa), raw in zip(rust, sents):
            ids = torch.tensor([encode(t, vocab, max_len)])
            lc, lv, ls = model(ids)
            pc, pv, ps = decode_labels(t[: max_len], lc[0], lv[0], ls[0])
            # the Rust side may window long inputs; compare on the common prefix
            L = min(len(pc), len(c))
            for i in range(L):
                if c[i] != "-":
                    pos_total += 1
                    pos_same += (c[i], v[i], s[i]) == (pc[i], pv[i], ps[i])
            ok = (pc, pv, ps) == (c[:L], v[:L], s[:L])
            same += ok
            same_ipa += render(t[:L], pc, pv, ps) == ipa[: len(render(t[:L], pc, pv, ps))] if L == len(t) else ok
            if not ok and shown < 3:
                shown += 1
                print("MISMATCH", t)
                print("  py  ", pc, pv, ps)
                print("  rust", c, v, s)
    print(f"sentences identical: {same}/{len(sents)} ({same/len(sents):.4f}); "
          f"letters identical: {pos_same}/{pos_total} ({pos_same/max(pos_total,1):.5f})")


if __name__ == "__main__":
    main()
