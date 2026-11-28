"""Polyphone reader trainer: char-level transformer encoder (the Hebrew student's block, tools/train_he.py) + one masked
classification head. Writes the G2PT file src/polyphone.rs loads (header kind=polyphone).

usage: train_polyphone.py --train a.tsv b.tsv:2 --dev dev.tsv --test test.tsv --cands cands.json --out dir [--d 256 --layers 6 --ff 768]

Data: tsv lines  sentence<TAB>char_index<TAB>reading.  Each file is given as  path[:weight]  (weight = repeats).
Candidates per character come from a JSON  {char: [readings]}  (--cands); every listed character is served (one candidate = a lookup). Logits are masked to the character's own candidates (conditional softmax, as g2pW).
Export: G2PT container, header kind=polyphone (see export()).
"""
import argparse, json, math, os, random, struct, sys, time, collections
import torch, torch.nn as nn, torch.nn.functional as F
PAD, UNK = 0, 1

class MHA(nn.Module):
    def __init__(s, d, h):
        super().__init__(); s.h = h; s.q = nn.Linear(d, d); s.k = nn.Linear(d, d); s.v = nn.Linear(d, d); s.o = nn.Linear(d, d)
    def forward(s, x, kp):
        B, T, D = x.shape; h = s.h
        q = s.q(x).view(B, T, h, D // h).transpose(1, 2); k = s.k(x).view(B, T, h, D // h).transpose(1, 2); v = s.v(x).view(B, T, h, D // h).transpose(1, 2)
        att = (q @ k.transpose(-1, -2)) / math.sqrt(D // h)
        att = att.masked_fill(kp[:, None, None, :], float("-inf"))
        return s.o((att.softmax(-1) @ v).transpose(1, 2).reshape(B, T, D))
class Block(nn.Module):
    def __init__(s, d, h, ff, drop):
        super().__init__(); s.ln1 = nn.LayerNorm(d); s.sa = MHA(d, h); s.ln3 = nn.LayerNorm(d); s.ff1 = nn.Linear(d, ff); s.ff2 = nn.Linear(ff, d); s.drop = nn.Dropout(drop)
    def forward(s, x, kp):
        x = x + s.drop(s.sa(s.ln1(x), kp)); return x + s.drop(s.ff2(F.relu(s.ff1(s.ln3(x)))))
class Poly(nn.Module):
    def __init__(s, nv, ncls, d=192, heads=4, layers=4, ff=512, max_len=128, drop=0.1):
        super().__init__()
        s.emb = nn.Embedding(nv, d); s.pos = nn.Embedding(max_len, d)
        s.enc = nn.ModuleList([Block(d, heads, ff, drop) for _ in range(layers)]); s.norm = nn.LayerNorm(d)
        s.head = nn.Linear(d, ncls); s.drop = nn.Dropout(drop)
    def forward(s, ids):
        x = s.drop(s.emb(ids) + s.pos(torch.arange(ids.size(1), device=ids.device)))
        kp = ids == PAD
        for b in s.enc: x = b(x, kp)
        return s.head(s.norm(x))

def window(n, i, L):
    if n <= L: return 0, n
    st = max(0, min(i - L // 2, n - L)); return st, st + L

def load_tsv(path, rep=1):
    rows = []
    for l in open(path, encoding="utf-8"):
        p = l.rstrip("\n").split("\t")
        if len(p) == 3: rows += [(p[0], int(p[1]), p[2])] * rep
    return rows

def group(rows):
    g = collections.OrderedDict()
    for s, i, r in rows: g.setdefault(s, []).append((i, r))
    return list(g.items())

def make_batch(items, vocab, cls_id, cand_mask, L, unk=0.0):
    B = len(items); ids = torch.zeros(B, L, dtype=torch.long); tpos = []; tcls = []; tm = []
    for b, (s, labs) in enumerate(items):
        i0 = labs[0][0]; st, en = window(len(s), i0, L); seg = s[st:en]; tgtset = {i - st for i, _ in labs}
        e = [vocab.get(c, UNK) if (unk == 0 or random.random() > unk or k in tgtset) else UNK for k, c in enumerate(seg)]
        ids[b, :len(e)] = torch.tensor(e)
        for i, r in labs:
            if st <= i < en and r in cls_id:
                tpos.append((b, i - st)); tcls.append(cls_id[r]); tm.append(cand_mask[s[i]])
    return ids, torch.tensor(tpos).reshape(-1, 2), torch.tensor(tcls), (torch.stack(tm) if tm else torch.zeros(0, len(cls_id), dtype=torch.bool))

@torch.no_grad()
def evaluate(model, grp, vocab, cls_id, cand_mask, L, dev, bs=256):
    model.eval(); ok = n = 0
    per = collections.defaultdict(lambda: [0, 0])
    for k in range(0, len(grp), bs):
        items = grp[k:k + bs]
        ids, tp, tc, tm = make_batch(items, vocab, cls_id, cand_mask, L)
        if len(tp) == 0: continue
        lg = model(ids.to(dev))[tp[:, 0], tp[:, 1]].float().cpu().masked_fill(~tm, -1e9)
        pr = lg.argmax(-1); ok += int((pr == tc).sum()); n += len(tc)
    model.train(); return ok / max(n, 1), n

def export(model, vocab, classes, char_classes, path, cfg, int8=True):
    sd = {k: v.detach().float().cpu().contiguous() for k, v in model.state_dict().items()}
    hd = {"magic": "G2PT", "kind": "polyphone", "config": cfg, "vocab": vocab, "classes": classes, "char_classes": char_classes, "tensors": []}
    off = 0; blobs = []
    for k, v in sd.items():
        if int8 and k.endswith(".weight") and v.dim() == 2 and not k.startswith(("emb", "pos")):
            sc = v.abs().amax(dim=1).clamp(min=1e-8) / 127.0
            q = torch.round(v / sc[:, None]).clamp(-127, 127).to(torch.int8)
            hd["tensors"].append({"name": k, "shape": list(v.shape), "offset": off, "dtype": "i8"}); b = q.numpy().tobytes(); blobs.append(b); off += len(b)
            hd["tensors"].append({"name": k + ".scale", "shape": [v.shape[0]], "offset": off}); b = sc.numpy().astype("<f4").tobytes(); blobs.append(b); off += len(b); continue
        hd["tensors"].append({"name": k, "shape": list(v.shape), "offset": off}); b = v.numpy().astype("<f4").tobytes(); blobs.append(b); off += len(b)
    hj = json.dumps(hd, ensure_ascii=False).encode("utf8")
    with open(path, "wb") as f:
        f.write(b"G2PT"); f.write(struct.pack("<I", len(hj))); f.write(hj)
        for b in blobs: f.write(b)
    return os.path.getsize(path)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--train", nargs="+", required=True); ap.add_argument("--dev", required=True); ap.add_argument("--test", default=None)
    ap.add_argument("--cands", required=True); ap.add_argument("--out", required=True)
    ap.add_argument("--d", type=int, default=192); ap.add_argument("--layers", type=int, default=4); ap.add_argument("--ff", type=int, default=512)
    ap.add_argument("--heads", type=int, default=4); ap.add_argument("--max-len", type=int, default=128)
    ap.add_argument("--epochs", type=int, default=8); ap.add_argument("--bs", type=int, default=256); ap.add_argument("--lr", type=float, default=3e-4)
    ap.add_argument("--dev-id", default="cuda" if torch.cuda.is_available() else "cpu"); ap.add_argument("--mem", type=float, default=0.12)
    ap.add_argument("--min-count", type=int, default=3); ap.add_argument("--drop", type=float, default=0.1); ap.add_argument("--unk", type=float, default=0.0); ap.add_argument("--wd", type=float, default=0.01)
    a = ap.parse_args(); random.seed(0); torch.manual_seed(0); os.makedirs(a.out, exist_ok=True)
    dev = a.dev_id
    if dev.startswith("cuda"): torch.cuda.set_per_process_memory_fraction(a.mem, torch.device(dev))
    cands = {c: v for c, v in json.load(open(a.cands, encoding="utf-8")).items() if len(v) >= 1}
    classes = sorted({r for v in cands.values() for r in v}); cls_id = {r: i for i, r in enumerate(classes)}
    cand_mask = {}
    for c, v in cands.items():
        m = torch.zeros(len(classes), dtype=torch.bool); m[[cls_id[r] for r in v]] = True; cand_mask[c] = m
    rows = []
    for spec in a.train:
        p, _, w = spec.partition(":"); rows += load_tsv(p, int(w or 1))
    rows = [r for r in rows if r[2] in cls_id and r[0][r[1]] in cand_mask and r[2] in cands[r[0][r[1]]]]
    cnt = collections.Counter(c for s, _, _ in rows for c in s)
    vocab = {c: i + 2 for i, c in enumerate(sorted(c for c, n in cnt.items() if n >= a.min_count))}
    tr = group(rows)
    dv = group([r for r in load_tsv(a.dev) if r[2] in cls_id and r[0][r[1]] in cand_mask])
    cfg = dict(d=a.d, heads=a.heads, layers=a.layers, ff=a.ff, max_len=a.max_len, n_vocab=len(vocab) + 2, n_classes=len(classes))
    model = Poly(cfg["n_vocab"], len(classes), a.d, a.heads, a.layers, a.ff, a.max_len, a.drop).to(dev)
    npar = sum(p.numel() for p in model.parameters())
    print(f"train sents {len(tr)} targets {len(rows)} dev {len(dv)} vocab {len(vocab)} classes {len(classes)} polyphonic chars {len(cands)} params {npar/1e6:.2f}M", flush=True)
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=a.wd, betas=(0.9, 0.98))
    total = (len(tr) // a.bs + 1) * a.epochs
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, max_lr=a.lr, total_steps=total + 5, pct_start=0.05, anneal_strategy="cos", div_factor=20, final_div_factor=100)
    best = -1; step = 0; t0 = time.time()
    for ep in range(a.epochs):
        random.shuffle(tr); tl = tn = 0
        for k in range(0, len(tr), a.bs):
            ids, tp, tc, tm = make_batch(tr[k:k + a.bs], vocab, cls_id, cand_mask, a.max_len, a.unk)
            if len(tp) == 0: continue
            lg = model(ids.to(dev))[tp[:, 0].to(dev), tp[:, 1].to(dev)].float().masked_fill(~tm.to(dev), -1e9)
            loss = F.cross_entropy(lg, tc.to(dev))
            opt.zero_grad(set_to_none=True); loss.backward(); nn.utils.clip_grad_norm_(model.parameters(), 1.0); opt.step()
            if step < total: sched.step()
            step += 1; tl += loss.item(); tn += 1
        acc, n = evaluate(model, dv, vocab, cls_id, cand_mask, a.max_len, dev)
        print(f"ep {ep+1} loss {tl/max(tn,1):.4f} dev_acc {acc:.4f} (n={n}) {time.time()-t0:.0f}s", flush=True)
        if acc > best:
            best = acc; torch.save(model.state_dict(), a.out + "/poly.pt")
    model.load_state_dict(torch.load(a.out + "/poly.pt")); 
    cc = {c: v for c, v in cands.items()}
    json.dump({"vocab": vocab, "classes": classes, "cfg": cfg}, open(a.out + "/meta.json", "w", encoding="utf-8"), ensure_ascii=False)
    size = export(model, vocab, classes, cc, a.out + "/model-int8.g2p", cfg, int8=True)
    export(model, vocab, classes, cc, a.out + "/model-f32.g2p", cfg, int8=False)
    print("best dev", best, "int8 size", size, flush=True)
    if a.test:
        te = group([r for r in load_tsv(a.test) if r[2] in cls_id and r[0][r[1]] in cand_mask])
        print("test_acc", evaluate(model, te, vocab, cls_id, cand_mask, a.max_len, dev), flush=True)

if __name__ == "__main__":
    main()
