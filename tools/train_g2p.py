"""Tiny char-level transformer G2P (word -> phoneme string), trained on CPU.

Architecture is deliberately plain so the Rust inference (polyg2p/src/nn.rs) can
mirror it exactly: learned positions, pre-LayerNorm, ReLU FFN, greedy decoding.

usage: train_g2p.py data.tsv outdir [--epochs N] [--d 192] [--layers 3]
data.tsv: word<TAB>phonemes   (rows whose word is in the held-out split are never trained on)
"""
import argparse, json, math, os, random, struct, time

import torch
import torch.nn as nn
import torch.nn.functional as F

PAD, BOS, EOS = 0, 1, 2


class G2P(nn.Module):
    def __init__(self, n_in, n_out, d=192, heads=4, layers=3, ff=512, max_in=40, max_out=48):
        super().__init__()
        self.d, self.heads = d, heads
        self.src_emb = nn.Embedding(n_in, d)
        self.tgt_emb = nn.Embedding(n_out, d)
        self.src_pos = nn.Embedding(max_in, d)
        self.tgt_pos = nn.Embedding(max_out, d)
        self.enc = nn.ModuleList([Block(d, heads, ff, cross=False) for _ in range(layers)])
        self.dec = nn.ModuleList([Block(d, heads, ff, cross=True) for _ in range(layers)])
        self.enc_norm = nn.LayerNorm(d)
        self.dec_norm = nn.LayerNorm(d)
        self.out = nn.Linear(d, n_out)

    def encode(self, src):
        pos = torch.arange(src.size(1), device=src.device)
        x = self.src_emb(src) + self.src_pos(pos)
        mask = src == PAD
        for b in self.enc:
            x = b(x, self_mask=mask)
        return self.enc_norm(x), mask

    def decode(self, tgt, mem, mem_mask):
        T = tgt.size(1)
        pos = torch.arange(T, device=tgt.device)
        y = self.tgt_emb(tgt) + self.tgt_pos(pos)
        causal = torch.triu(torch.ones(T, T, dtype=torch.bool, device=tgt.device), 1)
        for b in self.dec:
            y = b(y, self_mask=None, causal=causal, mem=mem, mem_mask=mem_mask)
        return self.out(self.dec_norm(y))

    def forward(self, src, tgt_in):
        mem, mask = self.encode(src)
        return self.decode(tgt_in, mem, mask)


class MHA(nn.Module):
    def __init__(self, d, heads):
        super().__init__()
        self.h = heads
        self.q = nn.Linear(d, d)
        self.k = nn.Linear(d, d)
        self.v = nn.Linear(d, d)
        self.o = nn.Linear(d, d)

    def forward(self, x, kv, key_pad=None, causal=None):
        B, T, D = x.shape
        S = kv.size(1)
        h = self.h
        q = self.q(x).view(B, T, h, D // h).transpose(1, 2)
        k = self.k(kv).view(B, S, h, D // h).transpose(1, 2)
        v = self.v(kv).view(B, S, h, D // h).transpose(1, 2)
        att = (q @ k.transpose(-1, -2)) / math.sqrt(D // h)
        if key_pad is not None:
            att = att.masked_fill(key_pad[:, None, None, :], float("-inf"))
        if causal is not None:
            att = att.masked_fill(causal[None, None], float("-inf"))
        y = att.softmax(-1) @ v
        return self.o(y.transpose(1, 2).reshape(B, T, D))


class Block(nn.Module):
    def __init__(self, d, heads, ff, cross):
        super().__init__()
        self.ln1 = nn.LayerNorm(d)
        self.sa = MHA(d, heads)
        self.cross = cross
        if cross:
            self.ln2 = nn.LayerNorm(d)
            self.ca = MHA(d, heads)
        self.ln3 = nn.LayerNorm(d)
        self.ff1 = nn.Linear(d, ff)
        self.ff2 = nn.Linear(ff, d)
        self.drop = nn.Dropout(0.1)

    def forward(self, x, self_mask=None, causal=None, mem=None, mem_mask=None):
        h = self.ln1(x)
        x = x + self.drop(self.sa(h, h, key_pad=self_mask, causal=causal))
        if self.cross:
            x = x + self.drop(self.ca(self.ln2(x), mem, key_pad=mem_mask))
        x = x + self.drop(self.ff2(F.relu(self.ff1(self.ln3(x)))))
        return x


def load(path):
    rows = []
    for line in open(path, encoding="utf8"):
        p = line.rstrip("\n").split("\t")
        if len(p) == 2 and p[0] and p[1]:
            rows.append((p[0], p[1]))
    return rows


def split(rows, test_frac, seed=0):
    words = sorted({w for w, _ in rows})
    rnd = random.Random(seed)
    rnd.shuffle(words)
    test = set(words[: int(len(words) * test_frac)])
    return [r for r in rows if r[0] not in test], [r for r in rows if r[0] in test]


def batches(rows, si, ti, bs, shuffle=True):
    idx = list(range(len(rows)))
    if shuffle:
        random.shuffle(idx)
    # bucket by length for speed
    idx.sort(key=lambda i: len(rows[i][0]) + random.random() * 2)
    chunks = [idx[i:i + bs] for i in range(0, len(idx), bs)]
    if shuffle:
        random.shuffle(chunks)
    for ch in chunks:
        ws = [rows[i][0] for i in ch]
        ps = [rows[i][1] for i in ch]
        S = max(len(w) for w in ws)
        T = max(len(p) for p in ps) + 2
        src = torch.zeros(len(ch), S, dtype=torch.long)
        tgt = torch.zeros(len(ch), T, dtype=torch.long)
        for j, (w, p) in enumerate(zip(ws, ps)):
            src[j, : len(w)] = torch.tensor([si.get(c, 3) for c in w])
            seq = [BOS] + [ti[c] for c in p] + [EOS]
            tgt[j, : len(seq)] = torch.tensor(seq)
        yield src, tgt


@torch.no_grad()
def greedy(model, src, max_len=None):
    max_len = max_len or model.tgt_pos.num_embeddings
    mem, mask = model.encode(src)
    B = src.size(0)
    ys = torch.full((B, 1), BOS, dtype=torch.long)
    done = torch.zeros(B, dtype=torch.bool)
    for _ in range(max_len - 1):
        logits = model.decode(ys, mem, mask)[:, -1]
        nxt = logits.argmax(-1)
        nxt[done] = PAD
        ys = torch.cat([ys, nxt[:, None]], 1)
        done |= nxt == EOS
        if done.all():
            break
    return ys[:, 1:]


def evaluate(model, rows, si, ti, it, n=3000):
    model.eval()
    byword = {}
    for w, p in rows:
        byword.setdefault(w, set()).add(p)
    words = sorted(byword)[:n]
    correct = 0
    for i in range(0, len(words), 256):
        chunk = words[i:i + 256]
        S = max(len(w) for w in chunk)
        src = torch.zeros(len(chunk), S, dtype=torch.long)
        for j, w in enumerate(chunk):
            src[j, : len(w)] = torch.tensor([si.get(c, 3) for c in w])
        out = greedy(model, src)
        for j, w in enumerate(chunk):
            s = ""
            for t in out[j].tolist():
                if t in (EOS, PAD):
                    break
                s += it[t]
            correct += s in byword[w]
    model.train()
    return correct / len(words)


def export(model, si, to, path, cfg):
    """Flat little-endian f32 blob + JSON header (read by nn.rs)."""
    sd = {k: v.detach().float().contiguous() for k, v in model.state_dict().items()}
    header = {"config": cfg, "src_vocab": si, "tgt_vocab": to, "tensors": []}
    off = 0
    blobs = []
    for k, v in sd.items():
        header["tensors"].append({"name": k, "shape": list(v.shape), "offset": off})
        b = v.numpy().astype("<f4").tobytes()
        blobs.append(b)
        off += len(b)
    hj = json.dumps(header, ensure_ascii=False).encode("utf8")
    with open(path, "wb") as f:
        f.write(b"G2PW")
        f.write(struct.pack("<I", len(hj)))
        f.write(hj)
        for b in blobs:
            f.write(b)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("data")
    ap.add_argument("outdir")
    ap.add_argument("--epochs", type=int, default=30)
    ap.add_argument("--d", type=int, default=192)
    ap.add_argument("--layers", type=int, default=3)
    ap.add_argument("--ff", type=int, default=512)
    ap.add_argument("--bs", type=int, default=256)
    ap.add_argument("--test", type=float, default=0.05)
    ap.add_argument("--hours", type=float, default=99)
    a = ap.parse_args()
    torch.set_num_threads(2)
    random.seed(0)
    torch.manual_seed(0)
    os.makedirs(a.outdir, exist_ok=True)
    rows = load(a.data)
    train, test = split(rows, a.test)
    chars = sorted({c for w, _ in rows for c in w})
    phones = sorted({c for _, p in rows for c in p})
    si = {c: i + 4 for i, c in enumerate(chars)}  # 0 pad, 1 bos, 2 eos, 3 unk
    to = ["<pad>", "<bos>", "<eos>"] + phones
    ti = {c: i for i, c in enumerate(to)}
    max_in = max(len(w) for w, _ in rows) + 1
    max_out = max(len(p) for _, p in rows) + 3
    cfg = dict(d=a.d, heads=4, layers=a.layers, ff=a.ff, max_in=max_in, max_out=max_out,
               n_in=len(si) + 4, n_out=len(to))
    model = G2P(len(si) + 4, len(to), a.d, 4, a.layers, a.ff, max_in, max_out)
    nparam = sum(p.numel() for p in model.parameters())
    print(f"train {len(train)} test {len(test)} params {nparam/1e6:.2f}M cfg {cfg}", flush=True)
    opt = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=0.01)
    steps_per = math.ceil(len(train) / a.bs)
    total = steps_per * a.epochs
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, max_lr=1e-3, total_steps=total, pct_start=0.05)
    t0 = time.time()
    best = -1
    for ep in range(a.epochs):
        tot, n = 0.0, 0
        for src, tgt in batches(train, si, ti, a.bs):
            logits = model(src, tgt[:, :-1])
            loss = F.cross_entropy(logits.reshape(-1, logits.size(-1)), tgt[:, 1:].reshape(-1),
                                   ignore_index=PAD, label_smoothing=0.05)
            opt.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            sched.step()
            tot += loss.item()
            n += 1
        acc = evaluate(model, test, si, ti, to)
        el = (time.time() - t0) / 3600
        print(f"ep {ep+1} loss {tot/n:.4f} heldout_word_acc {acc:.4f} elapsed {el:.2f}h", flush=True)
        if acc > best:
            best = acc
            export(model, si, to, os.path.join(a.outdir, "g2p.bin"), cfg)
            torch.save(model.state_dict(), os.path.join(a.outdir, "g2p.pt"))
        if el > a.hours:
            break
    with open(os.path.join(a.outdir, "test.tsv"), "w", encoding="utf8") as f:
        for w, p in test:
            f.write(f"{w}\t{p}\n")
    print("best", best)


if __name__ == "__main__":
    main()
