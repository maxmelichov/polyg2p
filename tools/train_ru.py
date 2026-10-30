#!/usr/bin/env python3
"""Russian context student: a character-level transformer ENCODER distilled from
RUAccent's homograph classifier and ё tagger (labels from tools/ru_distill.py).

Same block and weight file as the Hebrew tagger (tools/train_he.py, src/tagger.rs),
so src/ru.rs runs it through `tagger::Tagger` unchanged. Heads per character:
  stress (2): is this character the stressed vowel of its word
  vow    (2): is this е really ё                           (the "yo" head)
  cons   (1): unused, kept so the file layout matches
Only the homograph / ё-homograph decisions are consulted at run time; the rest
of the stress supervision (every dictionary word in the sentence) is there so the
model learns Russian stress and context cheaply.

usage: train_ru.py build-data/ru_ctx.jsonl outdir [--d 192 --layers 4 ...]
  the FIRST --heldout rows are the dev set; the best model (by homograph
  accuracy) is exported to outdir/ru-ctx.g2p.
"""
from __future__ import annotations

import argparse
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
from train_he import MHA, Block  # noqa: E402,F401  (same encoder block)

PAD, UNK = 0, 1
N_CONS, N_VOW = 1, 2


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
        return self.vow(x), self.stress(x)


def load_rows(path, limit=None):
    rows = []
    with open(path, encoding="utf8") as f:
        for line in f:
            rows.append(json.loads(line))
            if limit and len(rows) >= limit:
                break
    return rows


def build_vocab(rows, min_count=3):
    cnt = {}
    for r in rows:
        for ch in r["t"]:
            cnt[ch] = cnt.get(ch, 0) + 1
    chars = sorted(c for c, n in cnt.items() if n >= min_count)
    return {c: i + 2 for i, c in enumerate(chars)}


def encode(text, vocab, max_len):
    return [vocab.get(c, UNK) for c in text[:max_len]]


def batches(rows, vocab, max_len, tokens_per_batch, shuffle=True):
    idx = list(range(len(rows)))
    if shuffle:
        random.shuffle(idx)
    idx.sort(key=lambda i: len(rows[i]["t"]) + (random.random() * 8 if shuffle else 0))
    chunks, cur, cur_max = [], [], 0
    for i in idx:
        L = min(len(rows[i]["t"]), max_len)
        if cur and max(cur_max, L) * (len(cur) + 1) > tokens_per_batch:
            chunks.append(cur)
            cur, cur_max = [], 0
        cur.append(i)
        cur_max = max(cur_max, L)
    if cur:
        chunks.append(cur)
    if shuffle:
        random.shuffle(chunks)
    for ch in chunks:
        L = min(max(len(rows[i]["t"]) for i in ch), max_len)
        ids = torch.zeros(len(ch), L, dtype=torch.long)
        ys = torch.full((len(ch), L), -100, dtype=torch.long)
        yy = torch.full((len(ch), L), -100, dtype=torch.long)
        wt = torch.ones(len(ch), L)
        for j, i in enumerate(ch):
            r = rows[i]
            e = encode(r["t"], vocab, max_len)
            ids[j, : len(e)] = torch.tensor(e)
            s, y, w = r["s"], r["y"], r["w"]
            for p in range(len(e)):
                if s[p] != "-":
                    ys[j, p] = int(s[p])
                if y[p] != "-":
                    yy[j, p] = int(y[p])
                wt[j, p] = float(w[p])
        yield ch, ids, ys, yy, wt


def weighted_ce(logits, target, weight, n):
    l = F.cross_entropy(logits.reshape(-1, n), target.reshape(-1), ignore_index=-100, reduction="none", label_smoothing=0.05)
    m = (target.reshape(-1) != -100).float()
    w = weight.reshape(-1) * m
    return (l * w).sum() / w.sum().clamp(min=1.0)


@torch.no_grad()
def evaluate(model, rows, vocab, max_len, dev, tokens_per_batch=16000):
    model.eval()
    hm_ok = hm_n = yh_ok = yh_n = st_ok = st_n = 0
    for ch, ids, ys, yy, _wt in batches(rows, vocab, max_len, tokens_per_batch, shuffle=False):
        lv, ls = model(ids.to(dev))
        lv, ls = lv.float().cpu(), ls.float().cpu()
        for j, i in enumerate(ch):
            r = rows[i]
            L = min(len(r["t"]), max_len)
            margin = ls[j, :L, 1] - ls[j, :L, 0]
            for start, ln, cands, chosen in r["hm"]:
                if start + ln > L:
                    continue
                # candidates and the chosen index are relative to the word start
                best = max(cands, key=lambda p: float(margin[start + p]))
                hm_ok += best == chosen
                hm_n += 1
            pyo = lv[j, :L].softmax(-1)[:, 1]
            for start, ln, lab in r["yh"]:
                if start + ln > L:
                    continue
                es = [p for p in range(start, start + ln) if r["t"][p] == "е"]
                pred = 1 if es and max(float(pyo[p]) for p in es) >= 0.5 else 0
                yh_ok += pred == lab
                yh_n += 1
            # whole-word stress accuracy on every labelled word
            s = r["s"][:L]
            p = 0
            while p < L:
                if s[p] == "-":
                    p += 1
                    continue
                q = p
                while q < L and s[q] != "-":
                    q += 1
                gold = s.index("1", p) if "1" in s[p:q] else -1
                vow = [k for k in range(p, q) if r["t"][k] in "аеёиоуыэюя"]
                if gold >= 0 and vow:
                    st_ok += max(vow, key=lambda k: float(margin[k])) == gold
                    st_n += 1
                p = q
    model.train()
    return {"hom_acc": hm_ok / max(hm_n, 1), "yo_acc": yh_ok / max(yh_n, 1), "stress_acc": st_ok / max(st_n, 1),
            "hom_n": hm_n, "yo_n": yh_n}


def export(model, vocab, path, cfg, int8=False):
    """G2PT file, identical layout to train_he.export (see src/tagger.rs)."""
    sd = {k: v.detach().float().cpu().contiguous() for k, v in model.state_dict().items()}
    header = {"magic": "G2PT", "config": cfg, "vocab": vocab, "cons_vocab": ["-"], "vow_vocab": ["е", "ё"],
              "letter_cons": {}, "tensors": []}
    off = 0
    blobs = []
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
    ap.add_argument("--epochs", type=int, default=8)
    ap.add_argument("--d", type=int, default=192)
    ap.add_argument("--heads", type=int, default=4)
    ap.add_argument("--layers", type=int, default=4)
    ap.add_argument("--ff", type=int, default=512)
    ap.add_argument("--max-len", type=int, default=320)
    ap.add_argument("--tokens", type=int, default=16384)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--heldout", type=int, default=5000)
    ap.add_argument("--limit", type=int, default=None)
    ap.add_argument("--hours", type=float, default=99)
    ap.add_argument("--eval-every", type=int, default=1000)
    ap.add_argument("--int8", action="store_true")
    a = ap.parse_args()
    random.seed(0)
    torch.manual_seed(0)
    os.makedirs(a.outdir, exist_ok=True)
    dev = "mps" if torch.backends.mps.is_available() else ("cuda" if torch.cuda.is_available() else "cpu")
    rows = load_rows(a.data, a.limit)
    test, train = rows[: a.heldout], rows[a.heldout:]
    vocab = build_vocab(train)
    json.dump(vocab, open(os.path.join(a.outdir, "vocab.json"), "w", encoding="utf8"), ensure_ascii=False)
    cfg = dict(d=a.d, heads=a.heads, layers=a.layers, ff=a.ff, max_len=a.max_len,
               n_vocab=len(vocab) + 2, n_cons=N_CONS, n_vow=N_VOW)
    model = Student(cfg["n_vocab"], a.d, a.heads, a.layers, a.ff, a.max_len).to(dev)
    nparam = sum(p.numel() for p in model.parameters())
    print(f"train {len(train)} test {len(test)} vocab {len(vocab)} params {nparam/1e6:.2f}M dev {dev} cfg {cfg}", flush=True)
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01, betas=(0.9, 0.98))
    n_batches = sum(1 for _ in batches(train, vocab, a.max_len, a.tokens, shuffle=False))
    total = n_batches * a.epochs
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, max_lr=a.lr, total_steps=total + 10, pct_start=0.05,
                                                anneal_strategy="cos", div_factor=20, final_div_factor=100)
    t0 = time.time()
    best = -1.0
    step = 0
    log = open(os.path.join(a.outdir, "train.log"), "a")

    def do_eval(tag):
        nonlocal best
        res = evaluate(model, test, vocab, a.max_len, dev)
        msg = f"{tag} step {step} " + " ".join(f"{k} {v:.4f}" if isinstance(v, float) else f"{k} {v}" for k, v in res.items()) \
            + f" elapsed {(time.time()-t0)/60:.1f}min"
        print(msg, flush=True)
        log.write(msg + "\n")
        log.flush()
        score = res["hom_acc"] + 0.5 * res["yo_acc"]
        if score > best:
            best = score
            torch.save(model.state_dict(), os.path.join(a.outdir, "ru-ctx.pt"))
            size = export(model, vocab, os.path.join(a.outdir, "ru-ctx.g2p"), cfg)
            if a.int8:
                export(model, vocab, os.path.join(a.outdir, "ru-ctx-int8.g2p"), cfg, int8=True)
            print(f"  saved ru-ctx.g2p ({size/1e6:.1f} MB)", flush=True)

    for ep in range(a.epochs):
        tot, n = 0.0, 0
        for _ch, ids, ys, yy, wt in batches(train, vocab, a.max_len, a.tokens):
            ids, ys, yy, wt = ids.to(dev), ys.to(dev), yy.to(dev), wt.to(dev)
            lv, ls = model(ids)
            loss = weighted_ce(ls, ys, wt, 2) + weighted_ce(lv, yy, wt, N_VOW)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            if step < total:
                sched.step()
            step += 1
            tot += loss.item()
            n += 1
            if step % 200 == 0:
                print(f"ep {ep+1} step {step}/{total} loss {tot/n:.4f} lr {sched.get_last_lr()[0]:.2e} "
                      f"{step/(time.time()-t0):.1f} step/s", flush=True)
            if step % a.eval_every == 0:
                do_eval(f"ep {ep+1}")
            if (time.time() - t0) / 3600 > a.hours:
                break
        do_eval(f"END ep {ep+1} loss {tot/max(n,1):.4f}")
        if (time.time() - t0) / 3600 > a.hours:
            break
    print("best", best)


if __name__ == "__main__":
    main()
