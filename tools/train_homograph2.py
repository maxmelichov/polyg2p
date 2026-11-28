"""Round-2 students for the English heteronym model: `bag` (fast, no attention) and `trunk` (small transformer).

Features `v2` (mirror in src/homograph.rs, `bag_features`): for the target unit t and its neighbours, as UTF-8 strings hashed with
FNV-1a 64 modulo n_buckets (`PAD` units are `<s>` / `</s>`; x(j) is the lower-case unit text):
  `{o}|w={x}` for o in -4..4; `{o}|s2={..}` `{o}|s3={..}` for 1 <= |o| <= 2 (when the word is longer than the suffix); `{o}|cap`
  `b1|x(-2)|x(-1)`, `b2|x(-1)|x(+1)`, `b3|x(+1)|x(+2)`; `first`, `last`, `endq`, `endx`
  `L|x(j)` for the word units j in t-12..t-1, `R|x(j)` for t+1..t+12
  `t={x(0)}|{o}|x(o)` for o in -2,-1,1,2; `t={x(0)}|first`, `t={x(0)}|last`; `t={x(0)}|L|x(j)`, `t={x(0)}|R|x(j)`
Model bag: sum of embeddings (d) -> Linear(d, h) -> ReLU -> Linear(h, rows), masked to the target word's rows.
Model trunk: features v1 as in train_homograph.py, d/layers/ff set by flags, window max_len.

usage: train_homograph2.py CORPUS/data --out DIR --arch bag|trunk [--silver S] [--seeds S] [--pool P] [--gold-share .4 --pool-share .4 --silver-share .2]
          [--mode dev|fold|full] [--folds 5] [--steps 6000] [--probe NAME]
modes: dev = gold fit/dev split of round 1 (reports gold dev, dev_hard, dev_ewt); fold = K-fold over the whole gold train split
       (writes OOF logits); full = train on the whole gold train split, export the file.
"""
from __future__ import annotations
import argparse, collections, json, math, multiprocessing as mp, os, pickle, random, struct, subprocess, sys, time
import numpy as np
import torch, torch.nn as nn

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
sys.path.insert(0, HERE)
import train_homograph as T  # noqa: E402
import en_homograph_bench as bench  # noqa: E402

NB, DB, HB, NF = 32768, 96, 192, 112


def fnv(b):
    return T.fnv1a(b)


def bag_strings(units, t):
    n = len(units)
    low = lambda j: "<s>" if j < 0 else "</s>" if j >= n else units[j].lstrip("^") if units[j] != "^" else units[j]
    isw = lambda j: 0 <= j < n and (units[j][0].isalpha() or units[j][0] == "^" and len(units[j]) > 1 and units[j][1].isalpha())
    cap = lambda j: 0 <= j < n and units[j].startswith("^") and len(units[j]) > 1
    f = []
    for o in range(-4, 5):
        j, x = t + o, low(t + o)
        f.append(f"{o}|w={x}")
        if 1 <= abs(o) <= 2 and 0 <= j < n:
            for k in (2, 3):
                if len(x) > k:
                    f.append(f"{o}|s{k}={x[-k:]}")
        if cap(j):
            f.append(f"{o}|cap")
    f.append(f"b1|{low(t - 2)}|{low(t - 1)}")
    f.append(f"b2|{low(t - 1)}|{low(t + 1)}")
    f.append(f"b3|{low(t + 1)}|{low(t + 2)}")
    first = t == 0
    last = not any(isw(j) for j in range(t + 1, n))
    if first:
        f.append("first")
    if last:
        f.append("last")
    if units and units[-1] == "?":
        f.append("endq")
    if units and units[-1] == "!":
        f.append("endx")
    tw = low(t)
    L = [low(j) for j in range(max(0, t - 12), t) if isw(j)]
    Rr = [low(j) for j in range(t + 1, min(n, t + 13)) if isw(j)]
    f += [f"L|{x}" for x in L] + [f"R|{x}" for x in Rr]
    for o in (-2, -1, 1, 2):
        f.append(f"t={tw}|{o}|{low(t + o)}")
    if first:
        f.append(f"t={tw}|first")
    if last:
        f.append(f"t={tw}|last")
    f += [f"t={tw}|L|{x}" for x in L] + [f"t={tw}|R|{x}" for x in Rr]
    return f


_memo = {}


def bag_ids(units, t):
    ids = []
    for s in bag_strings(units, t):
        h = _memo.get(s)
        if h is None:
            h = fnv(s.encode("utf8")) % NB
            if len(_memo) < 3_000_000:
                _memo[s] = h
        ids.append(h)
    return ids[:NF]


def _enc_chunk(chunk):
    out = np.full((len(chunk), NF), NB, dtype=np.int32)
    for i, (u, t) in enumerate(chunk):
        ids = bag_ids(u, t)
        out[i, :len(ids)] = ids
    return out


def featurize(pairs, procs=6):
    chunks = [pairs[i:i + 2000] for i in range(0, len(pairs), 2000)]
    with mp.Pool(procs) as p:
        return np.concatenate(p.map(_enc_chunk, chunks)) if chunks else np.zeros((0, NF), np.int32)


class Bag(nn.Module):
    def __init__(self, R, d=DB, h=HB, nb=NB):
        super().__init__()
        self.emb = nn.Embedding(nb + 1, d, padding_idx=nb)
        nn.init.normal_(self.emb.weight, std=0.05)
        self.l1 = nn.Linear(d, h)
        self.out = nn.Linear(h, R)
        self.drop = nn.Dropout(0.1)

    def forward(self, ids):
        x = self.drop(self.emb(ids).sum(1))
        return self.out(torch.relu(self.l1(x)))


class Ex:
    """examples as parallel arrays"""
    def __init__(self):
        self.units, self.t, self.row, self.soft, self.kind, self.init = [], [], [], [], [], []
        self.x = None


def build_ex(items, data, kinds=None, soft=None, inits=None):
    """items [(s, a, b, row)] -> Ex (items whose target the engine cannot find are dropped)."""
    units = T.rust_units([(s, a, b) for s, a, b, _ in items], data)
    e = Ex()
    for i, ((s, a, b, row), u) in enumerate(zip(items, units)):
        if u is None:
            continue
        e.units.append(u[1]); e.t.append(u[0]); e.row.append(row)
        e.soft.append(soft[i] if soft else None)
        e.kind.append(kinds[i] if kinds else "")
        e.init.append(inits[i] if inits else (a == 0))
    return e


def cases(path, table, data):
    row_of = {(w, r): i for i, (w, r, *_x) in enumerate(table)}
    its = [(sent, a, b, row_of[(w, r)]) for w, r, sent, a, b, _m in bench.read_cases(path)]
    return build_ex(its, data), its


def load_jsonl(path, cap=None):
    return [json.loads(l) for l in open(path, encoding="utf8")]


def evaluate(pred_rows, ex):
    ok = np.array([p == r for p, r in zip(pred_rows, ex.row)])
    return float(ok.mean()) if len(ok) else float("nan")


def main():
    global NB, DB, HB
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus")
    ap.add_argument("--table", default=os.path.join(ROOT, "data", "en_homograph.tsv"))
    ap.add_argument("--out", required=True)
    ap.add_argument("--arch", default="bag")
    ap.add_argument("--silver"), ap.add_argument("--seeds"), ap.add_argument("--pool")
    ap.add_argument("--gold-share", type=float, default=None), ap.add_argument("--pool-share", type=float, default=None)
    ap.add_argument("--silver-share", type=float, default=None)
    ap.add_argument("--mode", default="dev")
    ap.add_argument("--folds", type=int, default=5)
    ap.add_argument("--steps", type=int, default=6000)
    ap.add_argument("--batch", type=int, default=64)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--warmup", type=int, default=300)
    ap.add_argument("--smooth", type=float, default=0.05)
    ap.add_argument("--hidden", type=int, default=HB)
    ap.add_argument("--buckets", type=int, default=NB)
    ap.add_argument("--d", type=int, default=DB)
    ap.add_argument("--layers", type=int, default=2)
    ap.add_argument("--ff", type=int, default=384)
    ap.add_argument("--max-len", type=int, default=25)
    ap.add_argument("--silver-only-gold-dev", action="store_true")
    ap.add_argument("--threads", type=int, default=8)
    ap.add_argument("--device", default="cpu")
    ap.add_argument("--data", default=os.path.join(ROOT, "data-out"))
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--eval-every", type=int, default=1000)
    ap.add_argument("--seed-share", type=float, default=0.15)
    ap.add_argument("--no-gold-fit", action="store_true", help="probe: leave gold out of training (silver-only G1 probe)")
    ap.add_argument("--cache", default=None)
    a = ap.parse_args()
    NB, DB, HB = a.buckets, a.d, a.hidden
    torch.set_num_threads(min(a.threads, 8)); torch.manual_seed(a.seed)
    rnd = random.Random(a.seed)
    os.makedirs(a.out, exist_ok=True)
    logf = open(os.path.join(a.out, "train.log"), "a")
    def log(*m):
        s = " ".join(str(x) for x in m); print(s, flush=True); logf.write(s + "\n"); logf.flush()
    log("== " + " ".join(sys.argv[1:]))
    table = T.load_table(a.table); R = len(table)
    word_rows = collections.defaultdict(list)
    for i, (w, *_x) in enumerate(table):
        word_rows[w].append(i)
    rowmask = torch.zeros(R, R, dtype=torch.bool)   # rowmask[r] = rows of r's word
    for rs in word_rows.values():
        for r in rs:
            rowmask[r, rs] = True
    gold = T.load_gold(a.corpus, table, "train")
    if a.mode == "dev":
        fit_items, dev_items = T.split_dev(gold, table)
    else:
        fit_items, dev_items = gold, []
    # extra data
    sil = load_jsonl(a.silver) if a.silver else []
    seeds = load_jsonl(a.seeds) if a.seeds else []
    pool = load_jsonl(a.pool) if a.pool else []
    t0 = time.time()
    ex_gold = build_ex(fit_items, a.data)
    ex_dev = build_ex(dev_items, a.data) if dev_items else None
    ex_sil = build_ex([(o["s"], o["a"], o["b"], o["row"]) for o in sil], a.data, [o["kind"] for o in sil])
    ex_seed = build_ex([(o["s"], o["a"], o["b"], o["row"]) for o in seeds], a.data)
    softs = [[(int(r), p) for r, p in o["p"].items()] for o in pool]
    ex_pool = build_ex([(o["s"], o["a"], o["b"], o["row"]) for o in pool], a.data, soft=softs)
    ex_hard, hard_items = cases(os.path.join(ROOT, "tests", "en_homograph_dev.tsv"), table, a.data)
    ewt = "/mnt/data30/g2p_eval_1128/r2/sets/dev_ewt.tsv"
    ex_ewt, _ = cases(ewt, table, a.data)
    log(f"examples: gold {len(ex_gold.row)} dev {len(ex_dev.row) if ex_dev else 0} silver {len(ex_sil.row)} seeds {len(ex_seed.row)} pool {len(ex_pool.row)} hard {len(ex_hard.row)} ewt {len(ex_ewt.row)} [{time.time()-t0:.0f}s]")
    dev_ = torch.device(a.device)
    # features
    def prep(e):
        if a.arch == "bag":
            e.x = featurize(list(zip(e.units, e.t)))
        else:
            def enc1(u, t):
                uu, tt = T.window(u, t, a.max_len)
                arr = np.full((len(uu), T.N_FEAT), NB, dtype=np.int32)
                for i, x in enumerate(uu):
                    ids = T.feature_ids(x, i == tt, NB)
                    arr[i, :len(ids)] = ids
                return arr, tt
            e.x = [enc1(u, t) for u, t in zip(e.units, e.t)]
            e.x_arr = None
    for e in (ex_gold, ex_dev, ex_sil, ex_seed, ex_pool, ex_hard, ex_ewt):
        if e is not None and len(e.row):
            prep(e)
    log(f"features done [{time.time()-t0:.0f}s]")

    def make_model():
        if a.arch == "bag":
            return Bag(R, a.d, a.hidden, a.buckets).to(dev_)
        T.D, T.FF, T.MAX_LEN = a.d, a.ff, a.max_len
        return T.HomographModel(R, a.buckets, d=a.d, layers=a.layers, ff=a.ff, max_len=a.max_len).to(dev_)

    def logits_of(model, e, idx):
        if a.arch == "bag":
            ids = torch.from_numpy(e.x[idx].astype(np.int64)).to(dev_)
            return model(ids)
        arrs = [e.x[i][0] for i in idx]
        tg = [e.x[i][1] for i in idx]
        ids, ln, tgt = (z.to(dev_) for z in T.collate(arrs, tg, 0.0, None, NB))
        return model(ids, ln, tgt)

    def predict(model, e, bs=512):
        model.eval(); out = []
        with torch.no_grad():
            for i in range(0, len(e.row), bs):
                idx = list(range(i, min(len(e.row), i + bs)))
                lg = logits_of(model, e, idx).cpu()
                rm = rowmask[torch.tensor([e.row[j] for j in idx])]
                out.append(lg.masked_fill(~rm, -1e4).numpy())
        model.train()
        return np.concatenate(out) if out else np.zeros((0, R))

    def score(model, e):
        lg = predict(model, e)
        return evaluate(lg.argmax(1), e), lg

    def hard_split(lg):
        res = {}
        ini_imp = [i for i, (s, a_, b, r) in enumerate(hard_items) if a_ == 0 and "V" in table[r][2].split(",") and table[r][1] not in ("adj", "adjnoun")]
        ini_noun = [i for i, (s, a_, b, r) in enumerate(hard_items) if a_ == 0 and i not in set(ini_imp)]
        # hard_items and ex_hard share order only when every target was found; map through row/units lengths
        return res
    def train(fit_ex, tag, steps):
        model = make_model()
        opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01, betas=(0.9, 0.98))
        sched = lambda s: min(1.0, (s + 1) / a.warmup) * (0.5 * (1 + math.cos(math.pi * min(1.0, s / steps))) * 0.99 + 0.01)
        has_pool = len(ex_pool.row) > 0
        gs = a.gold_share if a.gold_share is not None else (0.4 if has_pool else 0.6)
        ps = a.pool_share if a.pool_share is not None else (0.4 if has_pool else 0.0)
        ss = a.silver_share if a.silver_share is not None else (1.0 - gs - ps)
        if a.no_gold_fit:
            gs = 0.0
        tot = gs + ps + ss
        gs, ps, ss = gs / tot, ps / tot, ss / tot
        # pool: word -> reading -> [index]; word picked uniformly, reading by the teacher's mass with a 15% floor
        pw = collections.defaultdict(lambda: collections.defaultdict(list))
        for i, r in enumerate(ex_pool.row):
            pw[table[r][0]][r].append(i)
        pw_words = sorted(pw)
        def draw_pool():
            w = rnd.choice(pw_words)
            rs = pw[w]
            big = [r for r in rs if len(rs[r]) >= 20] or list(rs)
            tot_n = sum(len(rs[r]) for r in big)
            q = [max(0.15, len(rs[r]) / tot_n) for r in big]
            r = rnd.choices(big, q)[0]
            return rnd.choice(rs[r])
        sil_n, seed_n = len(ex_sil.row), len(ex_seed.row)
        model.train(); t1 = time.time()
        for step in range(steps):
            for g in opt.param_groups:
                g["lr"] = a.lr * sched(step)
            nb = a.batch
            n_g, n_p = int(round(nb * gs)), int(round(nb * ps)) if has_pool else 0
            n_s = nb - n_g - n_p
            parts = []
            if n_g:
                parts.append((fit_ex, [rnd.randrange(len(fit_ex.row)) for _ in range(n_g)], False))
            if n_p:
                parts.append((ex_pool, [draw_pool() for _ in range(n_p)], True))
            if n_s:
                n_seed = int(round(n_s * a.seed_share)) if seed_n else 0
                if sil_n and n_s - n_seed:
                    parts.append((ex_sil, [rnd.randrange(sil_n) for _ in range(n_s - n_seed)], False))
                if n_seed:
                    parts.append((ex_seed, [rnd.randrange(seed_n) for _ in range(n_seed)], False))
            lgs, tgts, masks = [], [], []
            for e, idx, soft in parts:
                lg = logits_of(model, e, idx)
                tg = torch.zeros(len(idx), R)
                for j, i in enumerate(idx):
                    if soft:
                        for r, p in e.soft[i]:
                            tg[j, r] = p
                    else:
                        tg[j, e.row[i]] = 1.0
                lgs.append(lg); tgts.append(tg)
                masks.append(rowmask[torch.tensor([e.row[i] for i in idx])])
            lg = torch.cat(lgs); tg = torch.cat(tgts).to(dev_); mk = torch.cat(masks).to(dev_)
            lg = lg.masked_fill(~mk, -1e4)
            logp = lg.log_softmax(1)
            tg = tg / tg.sum(1, keepdim=True).clamp(min=1e-9)
            smooth = (logp * mk).sum(1) / mk.sum(1)
            loss = (-(1 - a.smooth) * (tg * logp).sum(1) - a.smooth * smooth).mean()
            opt.zero_grad(); loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0); opt.step()
            if step == 199:
                log(f"[{tag}] 200 steps in {time.time()-t1:.0f}s: ETA {(time.time()-t1)/200*(steps-200)/60:.1f} min")
            if ex_dev is not None and ((step + 1) % a.eval_every == 0 or step + 1 == steps):
                msg = {"gold_dev": score(model, ex_dev)[0], "dev_hard": score(model, ex_hard)[0], "dev_ewt": score(model, ex_ewt)[0]}
                log(f"[{tag}] step {step + 1} loss {loss.item():.3f} " + " ".join(f"{k} {v:.4f}" for k, v in msg.items()) + f" [{(time.time()-t1)/60:.1f} min]")
        return model

    def breakdown(model):
        lg = predict(model, ex_hard)
        pred = lg.argmax(1)
        # sentence-initial imperatives vs nouns/names: recompute from the units (target index 0)
        imp = [i for i in range(len(ex_hard.row)) if ex_hard.t[i] == 0 and table[ex_hard.row[i]][1] in ("verb", "present", "wax", "plant", "turn", "rip", "bend") ]
        non = [i for i in range(len(ex_hard.row)) if ex_hard.t[i] == 0 and i not in set(imp)]
        f = lambda ix: float(np.mean([pred[i] == ex_hard.row[i] for i in ix])) if ix else float("nan")
        return f(imp), len(imp), f(non), len(non)

    if a.mode in ("dev", "full"):
        model = train(ex_gold, a.mode, a.steps)
        if ex_dev is not None:
            gd = score(model, ex_dev)[0]
            log(f"FINAL gold_dev {gd:.4f} dev_hard {score(model, ex_hard)[0]:.4f} dev_ewt {score(model, ex_ewt)[0]:.4f} initial(imperative acc, n, noun/name acc, n) {breakdown(model)}")
        torch.save({k: v.cpu() for k, v in model.state_dict().items()}, os.path.join(a.out, f"{a.arch}_{a.mode}.pt"))
        pickle.dump({"args": vars(a)}, open(os.path.join(a.out, "args.pkl"), "wb"))
    else:
        # K-fold over the whole gold train split; OOF logits written for the policy step
        by = collections.defaultdict(list)
        for i, r in enumerate(ex_gold.row):
            by[table[r][0]].append(i)
        fold_of = {}
        for w in sorted(by):
            ix = by[w][:]; rnd.shuffle(ix)
            for k, i in enumerate(ix):
                fold_of[i] = k % a.folds
        oof = np.zeros((len(ex_gold.row), R), np.float32)
        for k in range(a.folds):
            tr = Ex(); te = Ex()
            for e_, sel in ((tr, [i for i in range(len(ex_gold.row)) if fold_of[i] != k]), (te, [i for i in range(len(ex_gold.row)) if fold_of[i] == k])):
                e_.units = [ex_gold.units[i] for i in sel]; e_.t = [ex_gold.t[i] for i in sel]; e_.row = [ex_gold.row[i] for i in sel]
                e_.soft = [None] * len(sel); e_.x = ex_gold.x[sel] if a.arch == "bag" else [ex_gold.x[i] for i in sel]
            model = train(tr, f"fold{k}", a.steps)
            oof[[i for i in range(len(ex_gold.row)) if fold_of[i] == k]] = predict(model, te)
            log(f"fold {k}: held-out acc {evaluate(oof[[i for i in range(len(ex_gold.row)) if fold_of[i] == k]].argmax(1), te):.4f}")
        np.save(os.path.join(a.out, "oof_logits.npy"), oof)
        json.dump({"row": ex_gold.row}, open(os.path.join(a.out, "oof_rows.json"), "w"))
        log(f"OOF micro {evaluate(oof.argmax(1), ex_gold):.4f}")


if __name__ == "__main__":
    main()
