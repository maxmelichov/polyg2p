#!/usr/bin/env python3
"""Train the heteronym context encoder for one language (src/hetero.rs).

Same encoder block and file format as the Russian student (tools/train_ru.py,
tools/train_he.py, src/tagger.rs); only the heads are used differently:
  vow  (4)  reading class of a table word, read at the word's FIRST character
  cons (18) universal POS of every word, read at its first character (auxiliary
            task: teaches syntax from every word of every sentence; ignored at run time)
  stress(2) unused

usage: train_ctx.py silver/fr.jsonl outdir [--d 192 --layers 4 --ff 512 --heads 4 --max-len 320 --epochs 10]
rows: {"t": text, "p": [[offset, upos_id]], "h": [[offset, k, word]], "c": [[start, end]]?}
  "c" = cue spans (silver rows): with p=0.5 the cue is deleted from the text so the model cannot learn the cue only.
Class balance: each (word, class) capped at --cap rows, rows of rarer classes repeated up to --floor.
Writes outdir/<lang>-ctx.g2p (f32), <lang>-ctx-int8.g2p, vocab.json, train.log.
"""
from __future__ import annotations

import argparse
import collections
import json
import os
import random
import struct
import sys
import time

import torch
import torch.nn as nn
import torch.nn.functional as F

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from train_he import Block  # noqa: E402
from hetero_common import UPOS  # noqa: E402

PAD, UNK = 0, 1
N_CONS, N_VOW = 18, 4


class Student(nn.Module):
    def __init__(self, n_vocab, d=192, heads=4, layers=4, ff=512, max_len=320, drop=0.1):
        super().__init__()
        self.emb = nn.Embedding(n_vocab, d)
        self.pos = nn.Embedding(max_len, d)
        self.enc = nn.ModuleList([Block(d, heads, ff, drop) for _ in range(layers)])
        self.norm = nn.LayerNorm(d)
        self.cons = nn.Linear(d, N_CONS)
        self.vow = nn.Linear(d, N_VOW)
        self.stress = nn.Linear(d, 2)
        self.drop = nn.Dropout(drop)

    def forward(self, ids):
        pos = torch.arange(ids.size(1), device=ids.device)
        x = self.drop(self.emb(ids) + self.pos(pos))
        pad = ids == PAD
        for b in self.enc:
            x = b(x, pad)
        x = self.norm(x)
        return self.cons(x), self.vow(x)


def drop_cues(r):
    """delete the cue spans from text and shift offsets (labels inside a cue are dropped)"""
    t, cues = r["t"], sorted(r["c"])
    keep, shift = [], []
    out, pos, cut = [], 0, 0
    for a, b in cues:
        out.append(t[pos:a])
        shift.append((a, b - a))
        pos = b
    out.append(t[pos:])
    text = "".join(out)

    def mv(o):
        s = 0
        for a, ln in shift:
            if o >= a + ln:
                s += ln
            elif o >= a:
                return None
        return o - s
    p = [[mv(o), u] for o, u in r["p"] if mv(o) is not None]
    h = [[mv(o), k, w] for o, k, w in r["h"] if mv(o) is not None]
    return {"t": text, "p": p, "h": h}


def balance(rows, cap, floor, seed=0):
    rnd = random.Random(seed)
    rnd.shuffle(rows)
    seen = collections.Counter()
    out = []
    for r in rows:
        h = []
        for o, k, w in r["h"]:
            if seen[(w, k)] < cap:
                seen[(w, k)] += 1
                h.append([o, k, w])
        out.append({**r, "h": h})
    rep = {}
    for r in out:
        f = 1
        for o, k, w in r["h"]:
            f = max(f, min(20, -(-floor // max(seen[(w, k)], 1))))
        rep[id(r)] = f
    res = []
    for r in out:
        res.extend([r] * rep[id(r)])
    return res, seen


def build_vocab(rows, min_count=3):
    cnt = collections.Counter()
    for r in rows:
        cnt.update(r["t"])
    return {c: i + 2 for i, c in enumerate(sorted(c for c, n in cnt.items() if n >= min_count))}


def make_batches(rows, vocab, max_len, tokens, shuffle=True, cue_p=0.5):
    idx = list(range(len(rows)))
    if shuffle:
        random.shuffle(idx)
    idx.sort(key=lambda i: len(rows[i]["t"]) + (random.random() * 8 if shuffle else 0))
    chunks, cur, mx = [], [], 0
    for i in idx:
        L = min(len(rows[i]["t"]), max_len)
        if cur and max(mx, L) * (len(cur) + 1) > tokens:
            chunks.append(cur)
            cur, mx = [], 0
        cur.append(i)
        mx = max(mx, L)
    if cur:
        chunks.append(cur)
    if shuffle:
        random.shuffle(chunks)
    for ch in chunks:
        rs = []
        for i in ch:
            r = rows[i]
            if shuffle and r.get("c") and random.random() < cue_p:
                r = drop_cues(r)
            rs.append(r)
        L = min(max(len(r["t"]) for r in rs), max_len)
        ids = torch.zeros(len(rs), L, dtype=torch.long)
        yp = torch.full((len(rs), L), -100, dtype=torch.long)
        yh = torch.full((len(rs), L), -100, dtype=torch.long)
        for j, r in enumerate(rs):
            e = [vocab.get(c, UNK) for c in r["t"][:L]]
            ids[j, : len(e)] = torch.tensor(e)
            for o, u in r["p"]:
                if o < L:
                    yp[j, o] = u
            for o, k, _w in r["h"]:
                if o < L:
                    yh[j, o] = k
        yield ch, rs, ids, yp, yh


@torch.no_grad()
def evaluate(model, rows, vocab, max_len, dev):
    model.eval()
    ok, n = collections.Counter(), collections.Counter()
    for ch, rs, ids, _yp, yh in make_batches(rows, vocab, max_len, 16000, shuffle=False):
        _lc, lv = model(ids.to(dev))
        pred = lv.argmax(-1).cpu()
        for j, r in enumerate(rs):
            for o, k, w in r["h"]:
                if o < ids.size(1):
                    # restrict to the classes this word has
                    n[w] += 1
                    ok[w] += int(pred[j, o]) == k
    model.train()
    tot = sum(n.values())
    return sum(ok.values()) / max(tot, 1), tot


def export(model, vocab, path, cfg, int8=False):
    sd = {k: v.detach().float().cpu().contiguous() for k, v in model.state_dict().items()}
    header = {"magic": "G2PT", "config": cfg, "vocab": vocab, "cons_vocab": UPOS + ["?"],
              "vow_vocab": ["0", "1", "2", "3"], "letter_cons": {}, "tensors": []}
    off, blobs = 0, []
    for k, v in sd.items():
        if int8 and k.endswith(".weight") and v.dim() == 2 and not k.startswith(("emb", "pos")):
            scale = v.abs().amax(dim=1).clamp(min=1e-8) / 127.0
            q = torch.round(v / scale[:, None]).clamp(-127, 127).to(torch.int8)
            header["tensors"].append({"name": k, "shape": list(v.shape), "offset": off, "dtype": "i8"})
            b = q.numpy().tobytes()
            blobs.append(b)
            off += len(b)
            header["tensors"].append({"name": k + ".scale", "shape": [v.shape[0]], "offset": off})
            b = scale.numpy().astype("<f4").tobytes()
            blobs.append(b)
            off += len(b)
            continue
        header["tensors"].append({"name": k, "shape": list(v.shape), "offset": off})
        b = v.numpy().astype("<f4").tobytes()
        blobs.append(b)
        off += len(b)
    hj = json.dumps(header, ensure_ascii=False).encode("utf8")
    with open(path, "wb") as f:
        f.write(b"G2PT")
        f.write(struct.pack("<I", len(hj)))
        f.write(hj)
        for b in blobs:
            f.write(b)
    return os.path.getsize(path)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("data")
    ap.add_argument("outdir")
    ap.add_argument("--lang", default=None)
    ap.add_argument("--epochs", type=int, default=10)
    ap.add_argument("--d", type=int, default=192)
    ap.add_argument("--heads", type=int, default=4)
    ap.add_argument("--layers", type=int, default=4)
    ap.add_argument("--ff", type=int, default=512)
    ap.add_argument("--max-len", type=int, default=320)
    ap.add_argument("--tokens", type=int, default=16384)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--dev-frac", type=float, default=0.04)
    ap.add_argument("--cap", type=int, default=5000)
    ap.add_argument("--floor", type=int, default=200)
    ap.add_argument("--pos-weight", type=float, default=0.3)
    ap.add_argument("--minutes", type=float, default=18.0)
    ap.add_argument("--gpu-frac", type=float, default=0.125)
    a = ap.parse_args()
    lang = a.lang or os.path.basename(a.data.split(",")[0]).split(".")[0].split("_")[0]
    random.seed(0)
    torch.manual_seed(0)
    os.makedirs(a.outdir, exist_ok=True)
    dev = "cuda" if torch.cuda.is_available() else "cpu"
    if dev == "cuda":
        torch.cuda.set_per_process_memory_fraction(a.gpu_frac)
    import zlib
    rows = []
    for path in a.data.split(","):
        rows += [json.loads(l) for l in open(path, encoding="utf8")]
    n0 = len(rows)
    rows = [r for r in rows if zlib.crc32(r["t"].encode("utf8")) % 10 != 0]  # silver-eval sentences are never trained on
    print(f"held-out sentences removed: {n0 - len(rows)}", flush=True)
    with_h = [r for r in rows if r["h"]]
    other = [r for r in rows if not r["h"]]
    random.shuffle(with_h)
    nd = int(len(with_h) * a.dev_frac)
    test, train_h = with_h[:nd], with_h[nd:]
    train_h, seen = balance(train_h, a.cap, a.floor)
    train = train_h + other
    vocab = build_vocab(train)
    json.dump(vocab, open(os.path.join(a.outdir, "vocab.json"), "w", encoding="utf8"), ensure_ascii=False)
    cfg = dict(d=a.d, heads=a.heads, layers=a.layers, ff=a.ff, max_len=a.max_len, n_vocab=len(vocab) + 2,
               n_cons=N_CONS, n_vow=N_VOW)
    model = Student(cfg["n_vocab"], a.d, a.heads, a.layers, a.ff, a.max_len).to(dev)
    nparam = sum(p.numel() for p in model.parameters())
    log = open(os.path.join(a.outdir, "train.log"), "a")

    def say(m):
        print(m, flush=True)
        log.write(m + "\n")
        log.flush()
    say(f"{lang}: train {len(train)} (h rows {len(train_h)}) dev {len(test)} vocab {len(vocab)} params {nparam/1e6:.2f}M dev {dev}")
    say("class counts (after cap): " + ", ".join(f"{w}/{k}:{n}" for (w, k), n in sorted(seen.items(), key=lambda x: -x[1])[:40]))
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01, betas=(0.9, 0.98))
    n_b = sum(1 for _ in make_batches(train, vocab, a.max_len, a.tokens, shuffle=False))
    total = n_b * a.epochs
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, max_lr=a.lr, total_steps=total + 10, pct_start=0.05,
                                                anneal_strategy="cos", div_factor=20, final_div_factor=100)
    t0, step, best = time.time(), 0, -1.0
    stop = False
    for ep in range(a.epochs):
        tot, n = 0.0, 0
        for _ch, _rs, ids, yp, yh in make_batches(train, vocab, a.max_len, a.tokens):
            ids, yp, yh = ids.to(dev), yp.to(dev), yh.to(dev)
            lc, lv = model(ids)
            loss = F.cross_entropy(lv.reshape(-1, N_VOW), yh.reshape(-1), ignore_index=-100) \
                if (yh != -100).any() else lv.sum() * 0
            if (yp != -100).any():
                loss = loss + a.pos_weight * F.cross_entropy(lc.reshape(-1, N_CONS), yp.reshape(-1), ignore_index=-100)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            if step < total:
                sched.step()
            step += 1
            tot += loss.item()
            n += 1
            if (time.time() - t0) / 60 > a.minutes:
                stop = True
                break
        acc, nt = evaluate(model, test, vocab, a.max_len, dev)
        say(f"ep {ep+1} step {step}/{total} loss {tot/max(n,1):.4f} dev_acc {acc:.4f} (n {nt}) {(time.time()-t0)/60:.1f} min")
        if acc > best:
            best = acc
            torch.save(model.state_dict(), os.path.join(a.outdir, f"{lang}-ctx.pt"))
            export(model, vocab, os.path.join(a.outdir, f"{lang}-ctx.g2p"), cfg)
            sz = export(model, vocab, os.path.join(a.outdir, f"{lang}-ctx-int8.g2p"), cfg, int8=True)
            say(f"  saved (int8 {sz/1e6:.2f} MB)")
        if stop:
            say("time box reached")
            break
    say(f"best dev_acc {best:.4f}")


if __name__ == "__main__":
    main()
