"""Hebrew student tagger: a character-level transformer ENCODER distilled from
RenikudPlus labels (tools/he_distill.py output).

Per character it predicts three labels in the teacher's own label space:
consonant class (25), vowel class (6) and stress (2). The loss is masked to
Hebrew letters. The block is the encoder block of train_g2p.py (pre-LN, learned
positions, ReLU FFN) so src/tagger.rs can mirror it exactly.

usage: train_he.py build-data/he_labels.jsonl outdir [--d 256] [--layers 6] ...
  the FIRST --heldout rows of the jsonl are the dev set (never trained on);
  the exported model goes to outdir/he.g2p (G2PT format, see export()).
"""
from __future__ import annotations

import argparse
import json
import math
import os
import random
import struct
import sys
import time

import torch
import torch.nn as nn
import torch.nn.functional as F

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from he_distill import CONS_VOCAB, VOW_VOCAB, is_heb, render  # noqa: E402

PAD, UNK = 0, 1
N_CONS, N_VOW = 25, 6
# letter -> legal consonant classes (teacher metadata `letter_consonant_constraints`)
LETTER_CONS = {
    "א": [19, 0], "ב": [1, 2], "ג": [20, 24], "ד": [3], "ה": [4, 0], "ו": [2, 18, 0],
    "ז": [5, 23], "ח": [6], "ט": [7], "י": [8, 0], "כ": [9, 6], "ך": [9, 6], "ל": [10, 0],
    "מ": [11], "ם": [11], "נ": [12], "ן": [12], "ס": [13], "ע": [19, 0], "פ": [15, 14],
    "ף": [15, 14], "צ": [16, 17], "ץ": [16, 17], "ק": [9], "ר": [21], "ש": [22, 13, 0], "ת": [7],
}


class MHA(nn.Module):
    def __init__(self, d, heads):
        super().__init__()
        self.h = heads
        self.q = nn.Linear(d, d)
        self.k = nn.Linear(d, d)
        self.v = nn.Linear(d, d)
        self.o = nn.Linear(d, d)

    def forward(self, x, key_pad):
        B, T, D = x.shape
        h = self.h
        q = self.q(x).view(B, T, h, D // h).transpose(1, 2)
        k = self.k(x).view(B, T, h, D // h).transpose(1, 2)
        v = self.v(x).view(B, T, h, D // h).transpose(1, 2)
        att = (q @ k.transpose(-1, -2)) / math.sqrt(D // h)
        att = att.masked_fill(key_pad[:, None, None, :], float("-inf"))
        y = att.softmax(-1) @ v
        return self.o(y.transpose(1, 2).reshape(B, T, D))


class Block(nn.Module):
    def __init__(self, d, heads, ff, drop):
        super().__init__()
        self.ln1 = nn.LayerNorm(d)
        self.sa = MHA(d, heads)
        self.ln3 = nn.LayerNorm(d)
        self.ff1 = nn.Linear(d, ff)
        self.ff2 = nn.Linear(ff, d)
        self.drop = nn.Dropout(drop)

    def forward(self, x, key_pad):
        x = x + self.drop(self.sa(self.ln1(x), key_pad))
        x = x + self.drop(self.ff2(F.relu(self.ff1(self.ln3(x)))))
        return x


class Tagger(nn.Module):
    def __init__(self, n_vocab, d=256, heads=4, layers=6, ff=1024, max_len=512, drop=0.1):
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
        return self.cons(x), self.vow(x), self.stress(x)


# ---------------------------------------------------------------- data
def load_rows(path, limit=None):
    rows = []
    with open(path, encoding="utf8") as f:
        for line in f:
            r = json.loads(line)
            rows.append((r["t"], r["c"], r["v"], r["s"], r["ipa"]))
            if limit and len(rows) >= limit:
                break
    return rows


def build_vocab(rows, min_count=3):
    cnt = {}
    for t, *_ in rows:
        for ch in t:
            cnt[ch] = cnt.get(ch, 0) + 1
    chars = sorted(c for c, n in cnt.items() if n >= min_count)
    return {c: i + 2 for i, c in enumerate(chars)}  # 0 pad, 1 unk


def encode(text, vocab, max_len):
    return [vocab.get(c, UNK) for c in text[:max_len]]


def batches(rows, vocab, max_len, tokens_per_batch, shuffle=True):
    idx = list(range(len(rows)))
    if shuffle:
        random.shuffle(idx)
    idx.sort(key=lambda i: len(rows[i][0]) + (random.random() * 8 if shuffle else 0))
    chunks, cur, cur_max = [], [], 0
    for i in idx:
        L = min(len(rows[i][0]), max_len)
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
        L = min(max(len(rows[i][0]) for i in ch), max_len)
        ids = torch.zeros(len(ch), L, dtype=torch.long)
        yc = torch.full((len(ch), L), -100, dtype=torch.long)
        yv = torch.full((len(ch), L), -100, dtype=torch.long)
        ys = torch.full((len(ch), L), -100, dtype=torch.long)
        for j, i in enumerate(ch):
            t, c, v, s, _ = rows[i]
            e = encode(t, vocab, max_len)
            ids[j, : len(e)] = torch.tensor(e)
            for p in range(len(e)):
                if c[p] != "-":
                    yc[j, p] = int(c[p], 36)
                    yv[j, p] = int(v[p])
                    ys[j, p] = int(s[p])
        yield ch, ids, yc, yv, ys


# ---------------------------------------------------------------- decode (mirrors src/he.rs)
def decode_labels(text, lc, lv, ls):
    """Constrained greedy decode from logits (numpy/tensors [L, *]) -> (c, v, s) label strings."""
    n = len(text)
    cons = ["-"] * n
    vow = ["-"] * n
    st = ["-"] * n
    heb = [i for i in range(n) if is_heb(text[i])]
    for i in heb:
        allowed = LETTER_CONS[text[i]]
        best = max(allowed, key=lambda k: float(lc[i, k]))
        cons[i] = "0123456789abcdefghijklmnopqrstuvwxyz"[best]
        vow[i] = str(int(lv[i].argmax()))
        st[i] = "0"
    # one stress per whitespace word, only on a letter with a vowel, by margin
    i = 0
    while i < n:
        if text[i].isspace():
            i += 1
            continue
        j = i
        while j < n and not text[j].isspace():
            j += 1
        cands = [p for p in range(i, j) if is_heb(text[p]) and vow[p] != "0"]
        if cands:
            b = max(cands, key=lambda p: float(ls[p, 1] - ls[p, 0]))
            st[b] = "1"
        i = j
    return "".join(cons), "".join(vow), "".join(st)


@torch.no_grad()
def evaluate(model, rows, vocab, max_len, dev, tokens_per_batch=16000, dump=None):
    model.eval()
    tot = {"c": 0, "v": 0, "s": 0, "n": 0}
    words_ok = words = sents_ok = 0
    out_rows = {}
    for ch, ids, yc, yv, ys in batches(rows, vocab, max_len, tokens_per_batch, shuffle=False):
        lc, lv, ls = model(ids.to(dev))
        lc, lv, ls = lc.float().cpu(), lv.float().cpu(), ls.float().cpu()
        m = yc != -100
        tot["n"] += int(m.sum())
        tot["c"] += int(((lc.argmax(-1) == yc) & m).sum())
        tot["v"] += int(((lv.argmax(-1) == yv) & m).sum())
        for j, i in enumerate(ch):
            t, c, v, s, ipa = rows[i]
            L = min(len(t), max_len)
            pc, pv, ps = decode_labels(t[:L], lc[j, :L], lv[j, :L], ls[j, :L])
            tot["s"] += sum(1 for a, b in zip(ps, s) if a == b and a != "-")
            got = render(t[:L], pc, pv, ps)
            gw, tw = got.split(), ipa.split()
            words += len(tw)
            words_ok += sum(1 for a, b in zip(gw, tw) if a == b) if len(gw) == len(tw) else 0
            sents_ok += got == ipa
            if dump is not None:
                out_rows[i] = (t, pc, pv, ps, got)
    model.train()
    n = max(tot["n"], 1)
    res = {"cons_acc": tot["c"] / n, "vow_acc": tot["v"] / n, "stress_acc": tot["s"] / n,
           "word_acc": words_ok / max(words, 1), "sent_acc": sents_ok / max(len(rows), 1)}
    if dump is not None:
        with open(dump, "w", encoding="utf8") as f:
            for i in range(len(rows)):
                t, pc, pv, ps, got = out_rows[i]
                f.write(json.dumps({"t": t, "c": pc, "v": pv, "s": ps, "ipa": got}, ensure_ascii=False) + "\n")
    return res


# ---------------------------------------------------------------- export
def export(model, vocab, path, cfg, int8=False):
    """G2PT file: b"G2PT" | u32 header_len | JSON header | tensors (f32 LE, or int8 + f32 row scales).

    Linear weights are stored [out, in] (PyTorch layout). With int8=True each Linear
    weight is quantized symmetrically per output row: w ~= q * scale[row]; the
    header entry carries "dtype": "i8" and the scales follow as `<name>.scale`.
    """
    sd = {k: v.detach().float().cpu().contiguous() for k, v in model.state_dict().items()}
    header = {"magic": "G2PT", "config": cfg, "vocab": vocab,
              "cons_vocab": [CONS_VOCAB[i] for i in range(N_CONS)],
              "vow_vocab": [VOW_VOCAB[i] for i in range(N_VOW)],
              "letter_cons": LETTER_CONS, "tensors": []}
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
    ap.add_argument("--epochs", type=int, default=10)
    ap.add_argument("--d", type=int, default=256)
    ap.add_argument("--heads", type=int, default=4)
    ap.add_argument("--layers", type=int, default=6)
    ap.add_argument("--ff", type=int, default=1024)
    ap.add_argument("--max-len", type=int, default=512)
    ap.add_argument("--tokens", type=int, default=16384, help="characters per batch")
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--heldout", type=int, default=2000)
    ap.add_argument("--limit", type=int, default=None, help="use only the first N rows")
    ap.add_argument("--hours", type=float, default=99)
    ap.add_argument("--eval-every", type=int, default=2000, help="steps")
    ap.add_argument("--resume", default=None, help="state dict to continue from")
    ap.add_argument("--int8", action="store_true", help="also write he-int8.g2p")
    a = ap.parse_args()
    random.seed(0)
    torch.manual_seed(0)
    os.makedirs(a.outdir, exist_ok=True)
    dev = "mps" if torch.backends.mps.is_available() else ("cuda" if torch.cuda.is_available() else "cpu")
    rows = load_rows(a.data, a.limit)
    test, train = rows[: a.heldout], rows[a.heldout:]
    vocab_path = os.path.join(a.outdir, "vocab.json")
    if a.resume and os.path.exists(vocab_path):
        vocab = json.load(open(vocab_path, encoding="utf8"))
    else:
        vocab = build_vocab(train)
        json.dump(vocab, open(vocab_path, "w", encoding="utf8"), ensure_ascii=False)
    cfg = dict(d=a.d, heads=a.heads, layers=a.layers, ff=a.ff, max_len=a.max_len,
               n_vocab=len(vocab) + 2, n_cons=N_CONS, n_vow=N_VOW)
    json.dump(cfg, open(os.path.join(a.outdir, "config.json"), "w"))
    model = Tagger(cfg["n_vocab"], a.d, a.heads, a.layers, a.ff, a.max_len).to(dev)
    if a.resume:
        model.load_state_dict(torch.load(a.resume, map_location=dev))
    nparam = sum(p.numel() for p in model.parameters())
    print(f"train {len(train)} test {len(test)} vocab {len(vocab)} params {nparam/1e6:.2f}M "
          f"({nparam*4/1e6:.1f} MB f32) dev {dev} cfg {cfg}", flush=True)
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
        el = (time.time() - t0) / 3600
        msg = f"{tag} step {step} " + " ".join(f"{k} {v:.4f}" for k, v in res.items()) + f" elapsed {el:.2f}h"
        print(msg, flush=True)
        log.write(msg + "\n")
        log.flush()
        if res["word_acc"] > best:
            best = res["word_acc"]
            torch.save(model.state_dict(), os.path.join(a.outdir, "he.pt"))
            size = export(model, vocab, os.path.join(a.outdir, "he.g2p"), cfg)
            if a.int8:
                export(model, vocab, os.path.join(a.outdir, "he-int8.g2p"), cfg, int8=True)
            print(f"  saved he.g2p ({size/1e6:.1f} MB) word_acc {best:.4f}", flush=True)
        return res

    for ep in range(a.epochs):
        tot, n = 0.0, 0
        for _ch, ids, yc, yv, ys in batches(train, vocab, a.max_len, a.tokens):
            ids, yc, yv, ys = ids.to(dev), yc.to(dev), yv.to(dev), ys.to(dev)
            lc, lv, ls = model(ids)
            loss = (F.cross_entropy(lc.reshape(-1, N_CONS), yc.reshape(-1), ignore_index=-100, label_smoothing=0.05)
                    + F.cross_entropy(lv.reshape(-1, N_VOW), yv.reshape(-1), ignore_index=-100, label_smoothing=0.05)
                    + F.cross_entropy(ls.reshape(-1, 2), ys.reshape(-1), ignore_index=-100, label_smoothing=0.05))
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
    print("best word_acc", best)


if __name__ == "__main__":
    main()
