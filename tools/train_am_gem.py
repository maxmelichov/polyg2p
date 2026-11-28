"""Amharic gemination tagger (PLAN 2.3, 2.4, step 3). Character-level pre-LN transformer encoder, 0.64 M parameters, the Tagger layout of
polyg2p tools/train_ru.py / train_he.py (same block, same G2PT file). The `stress` head (2 logits) is the gemination head: long iff
logit[1] > logit[0]; `cons` and `vow` are dummies. Embedding = E_char + E_row + E_order, exported as one summed `emb.weight`.
usage: train_am_gem.py TAG [--hm 1|0] [--device cuda:1] [--epochs 30]      TAG -> model/TAG.pt (best by held-out CE) + TAG.log
Input strings: work/sent_labels*.jsonl (sentences, held-out/gold types masked), every lex_train word alone, packs of 3-8 words."""
import sys, os, json, math, random, time, argparse, struct
import numpy as np, torch, torch.nn as nn, torch.nn.functional as F
D = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, D)
TOOLS = os.environ.get("POLYG2P_TOOLS", os.path.dirname(os.path.abspath(__file__))); sys.path.insert(0, TOOLS)
from train_he import MHA, Block   # noqa: E402  (the encoder block of the Hebrew / Russian taggers)
import am_gem_ref as G
PAD, UNK = 0, 1
CFG = dict(d=128, heads=4, layers=4, ff=256, max_len=512)
MARKS = list(" .,?!:")
def vocab_list():
    """the vocabulary: pad, unknown, space and marks, every folded fidel _fidel reads (all of them, not only those seen)"""
    fid = sorted({G.fold(chr(o)) for o in range(0x1200, 0x1380) if G.A._fidel(chr(o))})
    return ["<pad>", "<unk>"] + MARKS + fid
VOC = vocab_list(); V2I = {c: i for i, c in enumerate(VOC)}
def row_order():
    rows = sorted({(ord(c) & ~7) for c in VOC if len(c) == 1 and 0x1200 <= ord(c) < 0x1380}); rid = {r: i + 1 for i, r in enumerate(rows)}
    ri = [0] * len(VOC); oi = [0] * len(VOC)
    for i, c in enumerate(VOC):
        if len(c) == 1 and 0x1200 <= ord(c) < 0x1380: ri[i] = rid[ord(c) & ~7]; oi[i] = (ord(c) & 7) + 1
    return len(rid) + 1, ri, oi
N_ROWS, RI, OI = row_order()
class Tagger(nn.Module):
    def __init__(self, drop=0.1):
        super().__init__(); d = CFG["d"]
        self.ec = nn.Embedding(len(VOC), d); self.er = nn.Embedding(N_ROWS, d); self.eo = nn.Embedding(9, d)
        self.register_buffer("ri", torch.tensor(RI)); self.register_buffer("oi", torch.tensor(OI))
        self.pos = nn.Embedding(CFG["max_len"], d)
        self.enc = nn.ModuleList([Block(d, CFG["heads"], CFG["ff"], drop) for _ in range(CFG["layers"])])
        self.norm = nn.LayerNorm(d); self.cons = nn.Linear(d, 1); self.vow = nn.Linear(d, 1); self.stress = nn.Linear(d, 2); self.drop = nn.Dropout(drop)
    def emb_table(self): return self.ec.weight + self.er(self.ri) + self.eo(self.oi)
    def forward(self, ids):
        x = self.drop(F.embedding(ids, self.emb_table()) + self.pos(torch.arange(ids.size(1), device=ids.device)))
        pad = ids == PAD
        for b in self.enc: x = b(x, pad)
        return self.stress(self.norm(x))
    def export_state(self):
        sd = {k: v.detach().float().cpu() for k, v in self.state_dict().items() if not k.startswith(("ec.", "er.", "eo.", "ri", "oi"))}
        out = {"emb.weight": self.emb_table().detach().float().cpu().contiguous()}; out.update(sd); return out
def export(state, path, th=None, int8=False, extra=None):
    """G2PT file, layout of train_he.export / train_ru.export. The threshold is baked into the stress bias: bias[1] -= log(th / (1 - th))."""
    state = {k: v.clone() for k, v in state.items()}
    if th is not None: state["stress.bias"][1] -= math.log(th / (1 - th))
    cfg = dict(CFG, n_vocab=len(VOC), n_cons=1, n_vow=1)
    header = {"magic": "G2PT", "config": cfg, "vocab": {c: i for i, c in enumerate(VOC)}, "cons_vocab": ["-"], "vow_vocab": ["-"], "letter_cons": {}, "tensors": [],
              "fold": True, "gem_threshold": th}
    if extra: header.update(extra)
    off = 0; blobs = []
    for k, v in state.items():
        v = v.contiguous()
        if int8 and k.endswith(".weight") and v.dim() == 2 and not k.startswith(("emb", "pos")):
            scale = v.abs().amax(dim=1).clamp(min=1e-8) / 127.0; q = torch.round(v / scale[:, None]).clamp(-127, 127).to(torch.int8)
            header["tensors"].append({"name": k, "shape": list(v.shape), "offset": off, "dtype": "i8"}); b = q.numpy().tobytes(); blobs.append(b); off += len(b)
            header["tensors"].append({"name": k + ".scale", "shape": [v.shape[0]], "offset": off}); b = scale.numpy().astype("<f4").tobytes(); blobs.append(b); off += len(b); continue
        header["tensors"].append({"name": k, "shape": list(v.shape), "offset": off}); b = v.numpy().astype("<f4").tobytes(); blobs.append(b); off += len(b)
    hj = json.dumps(header, ensure_ascii=False).encode("utf8")
    with open(path, "wb") as f:
        f.write(b"G2PT"); f.write(struct.pack("<I", len(hj))); f.write(hj); [f.write(b) for b in blobs]
    return os.path.getsize(path)
def enc(t): return [V2I.get(c, UNK) for c in t]
def windows(t, y, mx=CFG["max_len"]):
    """cut greedily at spaces into windows of at most mx characters"""
    if len(t) <= mx: return [(t, y)]
    out = []; s = 0
    while s < len(t):
        e = min(len(t), s + mx)
        if e < len(t):
            k = t.rfind(" ", s, e)
            if k > s: e = k
        out.append((t[s:e], y[s:e])); s = e + 1 if e < len(t) and t[e] == " " else e
    return out
def load(hm):
    sfx = "" if hm else "_nohm"
    rd = lambda p: [json.loads(l) for l in open(f"{D}/work/{p}{sfx}.jsonl", encoding="utf-8")]
    return [], rd("lex_train"), rd("lex_dev"), rd("gen")
def epoch_rows(sents, lex, rng, gen=(), gfrac=0.2):
    """round 2: single words and packs of 3-8 random words, no sentence rows; generated verb forms (positives only) are subsampled each epoch"""
    pool = list(lex) + [g for g in gen if rng.random() < gfrac]; rows = []
    for r in pool: rows.append((r["w"], r["y"]))
    idx = list(range(len(pool))); rng.shuffle(idx); k = 0
    while k < len(idx):
        n = rng.randint(3, 8); ws = [pool[i] for i in idx[k:k + n]]; k += n
        rows.append((" ".join(w["w"] for w in ws), sum(([*w["y"], -1] for w in ws), [])[:-1]))
    return rows
def batches(rows, tokens, rng=None):
    idx = list(range(len(rows)))
    if rng: rng.shuffle(idx)
    idx.sort(key=lambda i: len(rows[i][0]) + (rng.random() * 8 if rng else 0))
    chunks = []; cur = []; cm = 0
    for i in idx:
        L = len(rows[i][0])
        if cur and max(cm, L) * (len(cur) + 1) > tokens: chunks.append(cur); cur = []; cm = 0
        cur.append(i); cm = max(cm, L)
    if cur: chunks.append(cur)
    if rng: rng.shuffle(chunks)
    for ch in chunks:
        L = max(len(rows[i][0]) for i in ch); ids = torch.zeros(len(ch), L, dtype=torch.long); ys = torch.full((len(ch), L), -100, dtype=torch.long)
        for j, i in enumerate(ch):
            t, y = rows[i]; ids[j, :len(t)] = torch.tensor(enc(t)); ys[j, :len(y)] = torch.tensor([v if v >= 0 else -100 for v in y])
        yield ch, ids, ys
@torch.no_grad()
def probs(model, strings, dev, tokens=16384):
    """P(long) per character for every string (list of float arrays)"""
    model.eval(); rows = [(s, [-1] * len(s)) for s in strings]; out = [None] * len(rows)
    for ch, ids, _ in batches(rows, tokens):
        lg = model(ids.to(dev)).float(); p = (lg[..., 1] - lg[..., 0]).sigmoid().cpu().numpy()
        for j, i in enumerate(ch): out[i] = p[j, :len(rows[i][0])]
    return out
def heldout_ce(model, held, dev):
    ps = probs(model, [r["w"] for r in held], dev); n = 0; tot = 0.0
    for r, p in zip(held, ps):
        for v, q in zip(r["y"], p):
            if v >= 0: tot += -math.log(max(q if v else 1 - q, 1e-9)); n += 1
    return tot / max(n, 1)
def main():
    ap = argparse.ArgumentParser(); ap.add_argument("tag"); ap.add_argument("--hm", type=int, default=1); ap.add_argument("--device", default="cpu")
    ap.add_argument("--epochs", type=int, default=30); ap.add_argument("--tokens", type=int, default=8192); ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--warm", type=int, default=500); ap.add_argument("--patience", type=int, default=5); ap.add_argument("--memfrac", type=float, default=4 / 32.6)
    ap.add_argument("--threads", type=int, default=4); ap.add_argument("--gfrac", type=float, default=0.2, help="share of generated verb forms (positives only) drawn per epoch"); ap.add_argument("--extra", default=None, help="extra sent_labels-style jsonl (heteronym labels) replacing work/sent_labels")
    a = ap.parse_args(); torch.set_num_threads(a.threads); random.seed(0); np.random.seed(0); torch.manual_seed(0); rng = random.Random(0)
    dev = a.device
    if dev.startswith("cuda"): torch.cuda.set_per_process_memory_fraction(a.memfrac, torch.device(dev))
    sents, train, held, gen = load(a.hm)
    model = Tagger().to(dev); n = sum(p.numel() for p in model.parameters()) - sum(p.numel() for p in (model.ec.weight, model.er.weight, model.eo.weight))
    print(f"vocab {len(VOC)} params(without the 3 embedding parts) {n/1e6:.3f}M sentences {len(sents)} lex_train {len(train)} heldout {len(held)} device {dev}", flush=True)
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01, betas=(0.9, 0.98))
    spe = sum(1 for _ in batches(epoch_rows(sents, train, random.Random(1), gen, a.gfrac), a.tokens)); total = spe * a.epochs; step = 0
    lr_at = lambda s: a.lr * (s + 1) / a.warm if s < a.warm else 0.5 * a.lr * (1 + math.cos(math.pi * min(1.0, (s - a.warm) / max(1, total - a.warm))))
    best = 1e9; bad = 0; log = open(f"{D}/model/{a.tag}.log", "w"); t0 = time.time()
    for ep in range(a.epochs):
        model.train(); tot = 0.0; cnt = 0
        for _, ids, ys in batches(epoch_rows(sents, train, rng, gen, a.gfrac), a.tokens, rng):
            for g in opt.param_groups: g["lr"] = lr_at(step)
            ids, ys = ids.to(dev), ys.to(dev)
            loss = F.cross_entropy(model(ids).reshape(-1, 2), ys.reshape(-1), ignore_index=-100)
            opt.zero_grad(set_to_none=True); loss.backward(); nn.utils.clip_grad_norm_(model.parameters(), 1.0); opt.step(); step += 1; tot += loss.item(); cnt += 1
        ce = heldout_ce(model, held, dev); msg = f"ep {ep+1} step {step}/{total} train {tot/cnt:.4f} heldout_ce {ce:.4f} {(time.time()-t0)/60:.1f}min"
        print(msg, flush=True); log.write(msg + "\n"); log.flush()
        if ce < best: best = ce; bad = 0; torch.save(model.state_dict(), f"{D}/model/{a.tag}.pt")
        else:
            bad += 1
            if bad >= a.patience: print("early stop", flush=True); break
    print("best heldout_ce", best, flush=True)
if __name__ == "__main__": main()
