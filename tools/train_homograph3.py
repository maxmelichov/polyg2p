"""Round-3 students for the English heteronym model: `bag3` (word-class features, integer hashing) and `ctx` (small context model).

Features v3 (mirror in src/homograph.rs when it is ported). Units are the engine's (`en_homograph --units`); x(j) = lower-case unit
text (`<s>` / `</s>` outside), h(x) = FNV-1a 64 of its UTF-8 bytes, computed once per unit; a feature id = mix(template, offset, h[, h2])
% n_buckets, mix = FNV-style fold over 64-bit words (no strings). c(j) = class of unit j: closed-class table data/en_classes.tsv
(DET POSS PRON_SUBJ PRON_OBJ BE HAVE DO MODAL TO PREP CONJ NEG WH DEG), else capitalised (not at j=0) / digit / punct / -ing -ed -ly
-tion|ment|ness -s / other; BOS / EOS outside.
  w(o), o in -4..4 | suffix 2,3 of w(+-1) | cap(+-1) | bigrams (-2,-1) (-1,+1) (+1,+2) | first last endq endx
  w(o)|t for o = +-1, +-2; first|t last|t | L bag (word units t-8..t-1), R bag (t+1..t+8), each also with t for +-5
  class: c(o) o in -3..3 (o != 0), c(-2)c(-1), c(+1)c(+2), c(-1)c(+1), first&c(+1), first&c(+1)c(+2), last&c(-1); each also with t.
Model bag3: sum of embeddings -> Linear(d,h) -> ReLU -> Linear(h, rows), masked to the target word's rows.
Model ctx: window +-8 units (17 positions); per position word-hash emb + suffix-3-hash emb + class emb + offset emb (d), 1-D conv k=3
(+ReLU) x convs, ONE attention read at the target (target state as query, single head) + residual, MLP d->2d->rows.

usage: train_homograph3.py CORPUS/data --out DIR --arch bag3|ctx [--silver S] [--seeds S] [--pool P] [--mode dev|fold|full] ...
 pool P = jsonl {"s","a","b","rows":[..],"p":[..],"init":bool}; mix per batch: gold-share / pool-share / rest silver+seeds(+templates).
"""
from __future__ import annotations
import argparse, collections, hashlib, json, math, multiprocessing as mp, os, pickle, random, sys, time
import numpy as np
import torch, torch.nn as nn

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
sys.path.insert(0, HERE)
import train_homograph as T  # noqa: E402
import en_homograph_bench as bench  # noqa: E402

NB, NF = 32768, 80
M64 = 0xFFFFFFFFFFFFFFFF
K = 3  # max rows of a word


def fnv(b):
    return T.fnv1a(b)


def mix(*xs):
    h = 0xcbf29ce484222325
    for x in xs:
        h = ((h ^ (x & M64)) * 0x100000001b3) & M64
    return h ^ (h >> 32)


CLASSES = ["DET", "POSS", "PRON_SUBJ", "PRON_OBJ", "BE", "HAVE", "DO", "MODAL", "TO", "PREP", "CONJ", "NEG", "WH", "DEG"]
SHAPES = ["cap", "digit", "punct", "ing", "ed", "ly", "nom", "s", "other"]
CID = {c: i + 1 for i, c in enumerate(CLASSES + SHAPES)}
BOS, EOS = len(CID) + 1, len(CID) + 2
NCLS = len(CID) + 3
_ctab = {}
for l in open(os.path.join(ROOT, "data", "en_classes.tsv"), encoding="utf8"):
    if l.strip() and not l.startswith("#"):
        w, c = l.rstrip("\n").split("\t")
        _ctab[w] = CID[c]


def unit_class(u, j):
    cap = u.startswith("^") and len(u) > 1
    w = u[1:] if cap else u
    wl = w.lower()
    if wl in _ctab:
        return _ctab[wl]
    if cap and j > 0:
        return CID["cap"]
    if wl and wl[0].isdigit():
        return CID["digit"]
    if not wl or not wl[0].isalpha():
        return CID["punct"]
    if wl.endswith("ing") and len(wl) > 4:
        return CID["ing"]
    if wl.endswith("ed") and len(wl) > 3:
        return CID["ed"]
    if wl.endswith("ly") and len(wl) > 3:
        return CID["ly"]
    if wl.endswith(("tion", "ment", "ness")):
        return CID["nom"]
    if wl.endswith("s") and len(wl) > 3:
        return CID["s"]
    return CID["other"]


_hc = {}
def H(x):
    h = _hc.get(x)
    if h is None:
        h = fnv(x.encode("utf8"))
        if len(_hc) < 2_000_000:
            _hc[x] = h
    return h


def v3_ids(units, t, nb=NB):
    n = len(units)
    low = lambda j: "<s>" if j < 0 else "</s>" if j >= n else (units[j][1:] if units[j].startswith("^") and len(units[j]) > 1 else units[j])
    isw = lambda j: 0 <= j < n and low(j)[:1].isalpha()
    cap = lambda j: 0 <= j < n and units[j].startswith("^") and len(units[j]) > 1
    cls = lambda j: BOS if j < 0 else EOS if j >= n else unit_class(units[j], j)
    f = []
    ht = H(low(t))
    # template ids: small ints
    for o in range(-4, 5):
        f.append(mix(1, o + 8, H(low(t + o))))
    for o in (-2, -1, 1, 2) if False else (-1, 1):
        x = low(t + o)
        if 0 <= t + o < n:
            for k in (2, 3):
                if len(x) > k:
                    f.append(mix(2, (o + 8) * 4 + k, H(x[-k:])))
    for o in (-1, 1):
        if cap(t + o):
            f.append(mix(3, o + 8))
    f.append(mix(4, H(low(t - 2)), H(low(t - 1))))
    f.append(mix(5, H(low(t - 1)), H(low(t + 1))))
    f.append(mix(6, H(low(t + 1)), H(low(t + 2))))
    first = t == 0
    last = not any(isw(j) for j in range(t + 1, n))
    if first: f.append(mix(7, 0))
    if last: f.append(mix(7, 1))
    if units and units[-1] == "?": f.append(mix(7, 2))
    if units and units[-1] == "!": f.append(mix(7, 3))
    for o in (-2, -1, 1, 2):
        f.append(mix(8, o + 8, ht, H(low(t + o))))
    if first: f.append(mix(9, 0, ht))
    if last: f.append(mix(9, 1, ht))
    for j in range(max(0, t - 8), t):
        if isw(j):
            f.append(mix(10, H(low(j))))
            if t - j <= 5: f.append(mix(11, ht, H(low(j))))
    for j in range(t + 1, min(n, t + 9)):
        if isw(j):
            f.append(mix(12, H(low(j))))
            if j - t <= 5: f.append(mix(13, ht, H(low(j))))
    c = {o: cls(t + o) for o in range(-3, 4)}
    cf = [(14 + o + 3, (c[o],)) for o in (-3, -2, -1, 1, 2, 3)]
    cf.append((21, (c[-2], c[-1]))); cf.append((22, (c[1], c[2]))); cf.append((23, (c[-1], c[1])))
    if first:
        cf.append((24, (c[1],))); cf.append((25, (c[1], c[2])))
    if last:
        cf.append((26, (c[-1],)))
    for tid, args in cf:
        f.append(mix(100 + tid, *args))
        f.append(mix(200 + tid, ht, *args))
    out = [x % nb for x in f]
    return out[:NF]


CW, CS = 8, 17  # context half-window, positions


def ctx_arrays(units, t, nbw):
    n = len(units)
    low = lambda j: units[j][1:] if units[j].startswith("^") and len(units[j]) > 1 else units[j]
    w = np.zeros(CS, np.int32); s = np.zeros(CS, np.int32); c = np.zeros(CS, np.int32)
    for k, o in enumerate(range(-CW, CW + 1)):
        j = t + o
        if j < 0:
            x, cc = "<s>", BOS
        elif j >= n:
            x, cc = "</s>", EOS
        else:
            x, cc = low(j), unit_class(units[j], j)
        w[k] = 1 + H(x) % (nbw - 1)
        s[k] = 1 + H("s3|" + x[-3:]) % 4095
        c[k] = cc
    return np.concatenate([w, s, c])


def _enc_chunk(args):
    arch, nb, chunk = args
    if arch == "bag3":
        out = np.full((len(chunk), NF), nb, dtype=np.int32)
        for i, (u, t) in enumerate(chunk):
            ids = v3_ids(u, t, nb)
            out[i, :len(ids)] = ids
        return out
    return np.stack([ctx_arrays(u, t, nb) for u, t in chunk]) if chunk else np.zeros((0, 3 * CS), np.int32)


def featurize(arch, nb, pairs, procs=6):
    chunks = [(arch, nb, pairs[i:i + 4000]) for i in range(0, len(pairs), 4000)]
    if not chunks:
        return np.zeros((0, NF if arch == "bag3" else 3 * CS), np.int32)
    with mp.Pool(procs) as p:
        return np.concatenate(p.map(_enc_chunk, chunks))


class Bag(nn.Module):
    def __init__(self, R, d, h, nb):
        super().__init__()
        self.emb = nn.Embedding(nb + 1, d, padding_idx=nb)
        nn.init.normal_(self.emb.weight, std=0.05)
        self.l1 = nn.Linear(d, h); self.out = nn.Linear(h, R); self.drop = nn.Dropout(0.1)

    def forward(self, ids):
        return self.out(torch.relu(self.l1(self.drop(self.emb(ids).sum(1)))))


class Ctx(nn.Module):
    def __init__(self, R, d, nbw, convs=1):
        super().__init__()
        self.ew = nn.Embedding(nbw, d, padding_idx=0); self.es = nn.Embedding(4096, d, padding_idx=0)
        self.ec = nn.Embedding(NCLS, d); self.eo = nn.Embedding(CS, d)
        for e in (self.ew, self.es):
            nn.init.normal_(e.weight, std=0.05)
        self.convs = nn.ModuleList([nn.Conv1d(d, d, 3, padding=1) for _ in range(convs)])
        self.q = nn.Linear(d, d); self.k = nn.Linear(d, d); self.v = nn.Linear(d, d)
        self.mlp = nn.Sequential(nn.Linear(d, 2 * d), nn.ReLU(), nn.Linear(2 * d, R))
        self.drop = nn.Dropout(0.1); self.d = d

    def forward(self, x):
        w, s, c = x[:, :CS], x[:, CS:2 * CS], x[:, 2 * CS:]
        h = self.ew(w) + self.es(s) + self.ec(c) + self.eo.weight[None]
        h = self.drop(h)
        for cv in self.convs:
            h = torch.relu(cv(h.transpose(1, 2))).transpose(1, 2) + h
        qt = self.q(h[:, CW])
        att = torch.softmax((self.k(h) * qt[:, None]).sum(-1) / math.sqrt(self.d), 1)
        z = h[:, CW] + (att[..., None] * self.v(h)).sum(1)
        return self.mlp(self.drop(z))


class Ex:
    def __init__(self):
        self.units, self.t, self.row, self.init, self.x = [], [], [], [], None
        self.rows = None; self.p = None  # [n,K] targets (soft)


def build_ex(items, data, inits=None):
    units = T.rust_units([(s, a, b) for s, a, b, _ in items], data)
    e = Ex()
    for i, ((s, a, b, row), u) in enumerate(zip(items, units)):
        if u is None:
            continue
        e.units.append(u[1]); e.t.append(u[0]); e.row.append(row)
        e.init.append(inits[i] if inits is not None else (a == 0))
    return e


def cases(path, table, data):
    row_of = {(w, r): i for i, (w, r, *_x) in enumerate(table)}
    its = [(sent, a, b, row_of[(w, r)]) for w, r, sent, a, b, _m in bench.read_cases(path)]
    return build_ex(its, data), its


def hard_targets(e, rows_pad=True):
    n = len(e.row)
    e.rows = np.full((n, K), -1, np.int64); e.p = np.zeros((n, K), np.float32)
    e.rows[:, 0] = e.row; e.p[:, 0] = 1.0


def main():
    global NB
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus")
    ap.add_argument("--table", default=os.path.join(ROOT, "data", "en_homograph.tsv"))
    ap.add_argument("--out", required=True)
    ap.add_argument("--arch", default="bag3")
    ap.add_argument("--silver"), ap.add_argument("--seeds", nargs="*"), ap.add_argument("--pool")
    ap.add_argument("--gold-share", type=float, default=0.35), ap.add_argument("--pool-share", type=float, default=0.50)
    ap.add_argument("--pool-init-frac", type=float, default=0.2)
    ap.add_argument("--seed-share", type=float, default=0.15, help="share of the non-gold non-pool part that are seeds")
    ap.add_argument("--mode", default="dev")
    ap.add_argument("--folds", type=int, default=5)
    ap.add_argument("--steps", type=int, default=30000)
    ap.add_argument("--batch", type=int, default=256)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--warmup", type=int, default=300)
    ap.add_argument("--smooth", type=float, default=0.05)
    ap.add_argument("--hidden", type=int, default=192)
    ap.add_argument("--buckets", type=int, default=NB)
    ap.add_argument("--d", type=int, default=64)
    ap.add_argument("--convs", type=int, default=1)
    ap.add_argument("--threads", type=int, default=8)
    ap.add_argument("--device", default="cpu")
    ap.add_argument("--data", default=os.path.join(ROOT, "data-out"))
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--eval-every", type=int, default=5000)
    ap.add_argument("--cache", default=None, help="directory for featurized pool/silver arrays")
    ap.add_argument("--max-minutes", type=float, default=0, help="stop training after this wall time (cosine schedule is cut to fit from the 200-step ETA)")
    ap.add_argument("--drop-class", action="store_true")
    a = ap.parse_args()
    NB = a.buckets
    torch.set_num_threads(min(a.threads, 8)); torch.manual_seed(a.seed)
    rnd = random.Random(a.seed); nrnd = np.random.default_rng(a.seed)
    os.makedirs(a.out, exist_ok=True)
    logf = open(os.path.join(a.out, "train.log"), "a")
    def log(*m):
        s = " ".join(str(x) for x in m); print(s, flush=True); logf.write(s + "\n"); logf.flush()
    log("== " + " ".join(sys.argv[1:]))
    table = T.load_table(a.table); R = len(table)
    word_rows = collections.defaultdict(list)
    for i, (w, *_x) in enumerate(table):
        word_rows[w].append(i)
    assert max(len(v) for v in word_rows.values()) <= K
    rowmask = torch.zeros(R, R, dtype=torch.bool)
    for rs in word_rows.values():
        for r in rs:
            rowmask[r, rs] = True
    gold = T.load_gold(a.corpus, table, "train")
    fit_items, dev_items = T.split_dev(gold, table) if a.mode == "dev" else (gold, [])
    t0 = time.time()
    def cached(name, items_fn, soft_fn=None):
        """build Ex + features for a named data file, cached on disk (units + features)"""
        if a.cache:
            os.makedirs(a.cache, exist_ok=True)
            key = hashlib.md5((name + a.arch + str(NB) + str(NF) + str(a.drop_class)).encode()).hexdigest()[:10]
            fp = os.path.join(a.cache, f"{os.path.basename(name)}_{key}.pkl")
            if os.path.exists(fp):
                return pickle.load(open(fp, "rb"))
        e = items_fn()
        if len(e.row):
            e.x = featurize(a.arch, NB, list(zip(e.units, e.t)))
        e.units = None
        if a.cache:
            pickle.dump(e, open(fp, "wb"), protocol=4)
        return e
    def gold_ex(items):
        e = build_ex(items, a.data); e.x = featurize(a.arch, NB, list(zip(e.units, e.t))) if e.row else None; hard_targets(e); return e
    ex_gold = gold_ex(fit_items)
    ex_dev = gold_ex(dev_items) if dev_items else None
    def sil_fn():
        sil = [json.loads(l) for l in open(a.silver, encoding="utf8")]
        e = build_ex([(o["s"], o["a"], o["b"], o["row"]) for o in sil], a.data); hard_targets(e); return e
    ex_sil = cached(a.silver, sil_fn) if a.silver else Ex()
    def seed_fn():
        sd = [json.loads(l) for f in a.seeds for l in open(f, encoding="utf8")]
        e = build_ex([(o["s"], o["a"], o["b"], o["row"]) for o in sd], a.data); hard_targets(e); return e
    ex_seed = cached("seeds:" + ",".join(a.seeds or []) + ":" + ",".join(str(os.path.getmtime(f)) for f in (a.seeds or [])), seed_fn) if a.seeds else Ex()
    def pool_fn():
        po = [json.loads(l) for l in open(a.pool, encoding="utf8")]
        e = build_ex([(o["s"], o["a"], o["b"], o["rows"][0]) for o in po], a.data, inits=[o["init"] for o in po])
        # build_ex drops items the unit finder misses (should be none: the pool is pre-filtered); align by order
        assert len(e.row) == len(po), (len(e.row), len(po))
        e.rows = np.full((len(po), K), -1, np.int64); e.p = np.zeros((len(po), K), np.float32)
        for i, o in enumerate(po):
            for j, (r, p) in enumerate(zip(o["rows"], o["p"])):
                e.rows[i, j] = r; e.p[i, j] = p
        return e
    ex_pool = cached(a.pool + str(os.path.getmtime(a.pool)), pool_fn) if a.pool else Ex()
    ex_hard, hard_items = cases(os.path.join(ROOT, "tests", "en_homograph_dev.tsv"), table, a.data)
    ex_ewt, _ = cases("/mnt/data30/g2p_eval_1128/r2/sets/dev_ewt.tsv", table, a.data)
    for e in (ex_hard, ex_ewt):
        e.x = featurize(a.arch, NB, list(zip(e.units, e.t))); hard_targets(e)
    ex_hard.init_arr = np.array(ex_hard.init)
    log(f"examples: gold {len(ex_gold.row)} dev {len(ex_dev.row) if ex_dev else 0} silver {len(ex_sil.row)} seeds {len(ex_seed.row)} pool {len(ex_pool.row)} hard {len(ex_hard.row)} ewt {len(ex_ewt.row)} [{time.time()-t0:.0f}s]")
    dev_ = torch.device(a.device)
    if dev_.type == "cuda":
        torch.cuda.set_per_process_memory_fraction(min(1.0, 4.0 / (torch.cuda.get_device_properties(dev_).total_memory / 2**30)), dev_)

    def make_model():
        if a.arch == "bag3":
            return Bag(R, a.d, a.hidden, NB).to(dev_)
        return Ctx(R, a.d, NB, a.convs).to(dev_)

    def predict(model, e, bs=2048):
        model.eval(); out = []
        with torch.no_grad():
            for i in range(0, len(e.row), bs):
                x = torch.from_numpy(e.x[i:i + bs].astype(np.int64)).to(dev_)
                lg = model(x).cpu()
                rm = rowmask[torch.tensor(e.row[i:i + bs])]
                out.append(lg.masked_fill(~rm, -1e4).numpy())
        model.train()
        return np.concatenate(out) if out else np.zeros((0, R))

    acc = lambda lg, e: float((lg.argmax(1) == np.array(e.row)).mean()) if len(e.row) else float("nan")
    IMP = ("verb", "present", "wax", "plant", "turn", "rip", "bend")
    def breakdown(lg):
        pred = lg.argmax(1)
        imp = [i for i in range(len(ex_hard.row)) if ex_hard.t[i] == 0 and table[ex_hard.row[i]][1] in IMP]
        non = [i for i in range(len(ex_hard.row)) if ex_hard.t[i] == 0 and i not in set(imp)]
        f = lambda ix: float(np.mean([pred[i] == ex_hard.row[i] for i in ix])) if ix else float("nan")
        return f(imp), len(imp), f(non), len(non)

    def pools_by_word(ex, only_init=False):
        pw = collections.defaultdict(lambda: collections.defaultdict(list))
        for i in range(len(ex.row)):
            if only_init and not ex.init[i]:
                continue
            pw[table[ex.row[i]][0]][int(ex.rows[i, int(np.argmax(ex.p[i]))])].append(i)
        return pw

    def train(fit_ex, tag, steps):
        model = make_model()
        opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01, betas=(0.9, 0.98))
        total = [steps]
        sched = lambda s: min(1.0, (s + 1) / a.warmup) * (0.5 * (1 + math.cos(math.pi * min(1.0, s / total[0]))) * 0.99 + 0.01)
        has_pool = len(ex_pool.row) > 0
        gs, ps = a.gold_share, (a.pool_share if has_pool else 0.0)
        ss = max(0.0, 1.0 - gs - ps); tot = gs + ps + ss; gs, ps, ss = gs / tot, ps / tot, ss / tot
        pw_all = pools_by_word(ex_pool) if has_pool else {}
        pw_init = pools_by_word(ex_pool, True) if has_pool else {}
        def draw_pool(pw, ws):
            w = ws[rnd.randrange(len(ws))]; rs = pw[w]
            big = [r for r in rs if len(rs[r]) >= 20] or list(rs)
            tn = sum(len(rs[r]) for r in big); q = [max(0.15, len(rs[r]) / tn) for r in big]
            r = rnd.choices(big, q)[0]
            return rs[r][rnd.randrange(len(rs[r]))]
        wl_all, wl_init = sorted(pw_all), sorted(pw_init)
        sil_n, seed_n = len(ex_sil.row), len(ex_seed.row)
        model.train(); t1 = time.time()
        step = 0
        while step < total[0]:
            for g in opt.param_groups:
                g["lr"] = a.lr * sched(step)
            nb = a.batch
            n_g, n_p = int(round(nb * gs)), int(round(nb * ps)) if has_pool else 0
            n_s = nb - n_g - n_p
            xs, rws, pp = [], [], []
            def add(e, idx):
                xs.append(e.x[idx]); rws.append(e.rows[idx]); pp.append(e.p[idx])
            if n_g:
                add(fit_ex, np.array([rnd.randrange(len(fit_ex.row)) for _ in range(n_g)]))
            if n_p:
                n_i = int(round(n_p * a.pool_init_frac)) if pw_init else 0
                idx = [draw_pool(pw_init, wl_init) for _ in range(n_i)] + [draw_pool(pw_all, wl_all) for _ in range(n_p - n_i)]
                add(ex_pool, np.array(idx))
            if n_s:
                n_seed = int(round(n_s * a.seed_share)) if seed_n else 0
                if sil_n and n_s - n_seed:
                    add(ex_sil, np.array([rnd.randrange(sil_n) for _ in range(n_s - n_seed)]))
                if n_seed:
                    add(ex_seed, np.array([rnd.randrange(seed_n) for _ in range(n_seed)]))
            x = torch.from_numpy(np.concatenate(xs).astype(np.int64)).to(dev_)
            rows_ = np.concatenate(rws); p_ = np.concatenate(pp)
            tg = torch.zeros(len(rows_), R)
            for j in range(K):
                ok = rows_[:, j] >= 0
                tg[np.where(ok)[0], rows_[ok, j]] = torch.from_numpy(p_[ok, j])
            mk = rowmask[torch.from_numpy(rows_[:, 0])]
            lg = model(x).masked_fill(~mk.to(dev_), -1e4)
            logp = lg.log_softmax(1)
            tg = (tg / tg.sum(1, keepdim=True).clamp(min=1e-9)).to(dev_); mkd = mk.to(dev_)
            smooth = (logp * mkd).sum(1) / mkd.sum(1)
            loss = (-(1 - a.smooth) * (tg * logp).sum(1) - a.smooth * smooth).mean()
            opt.zero_grad(); loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0); opt.step()
            step += 1
            if step == 200:
                eta = (time.time() - t1) / 200 * (total[0] - 200) / 60
                log(f"[{tag}] 200 steps in {time.time()-t1:.0f}s: ETA {eta:.1f} min")
                if a.max_minutes and eta > a.max_minutes:
                    total[0] = int(200 + (a.max_minutes * 60 / ((time.time() - t1) / 200)) - 200)
                    log(f"[{tag}] steps cut to {total[0]} to fit {a.max_minutes} min")
            if ex_dev is not None and (step % a.eval_every == 0 or step == total[0]):
                lgh = predict(model, ex_hard)
                log(f"[{tag}] step {step} loss {loss.item():.3f} gold_dev {acc(predict(model, ex_dev), ex_dev):.4f} dev_hard {acc(lgh, ex_hard):.4f} dev_ewt {acc(predict(model, ex_ewt), ex_ewt):.4f} [{(time.time()-t1)/60:.1f} min]")
        return model

    if a.mode in ("dev", "full"):
        model = train(ex_gold, a.mode, a.steps)
        if ex_dev is not None:
            lgd, lgh, lge = predict(model, ex_dev), predict(model, ex_hard), predict(model, ex_ewt)
            log(f"FINAL gold_dev {acc(lgd, ex_dev):.4f} dev_hard {acc(lgh, ex_hard):.4f} dev_ewt {acc(lge, ex_ewt):.4f} initial(imperative acc, n, noun/name acc, n) {breakdown(lgh)}")
            np.savez(os.path.join(a.out, "dev_logits.npz"), gold_dev=lgd, dev_hard=lgh, dev_ewt=lge, hard_row=np.array(ex_hard.row), dev_row=np.array(ex_dev.row), ewt_row=np.array(ex_ewt.row), hard_t=np.array(ex_hard.t))
        torch.save({k: v.cpu() for k, v in model.state_dict().items()}, os.path.join(a.out, f"{a.arch}_{a.mode}.pt"))
        pickle.dump({"args": vars(a)}, open(os.path.join(a.out, "args.pkl"), "wb"))
    else:
        by = collections.defaultdict(list)
        for i, r in enumerate(ex_gold.row):
            by[table[r][0]].append(i)
        fold_of = {}
        for w in sorted(by):
            ix = by[w][:]; rnd.shuffle(ix)
            for k, i in enumerate(ix):
                fold_of[i] = k % a.folds
        oof = np.zeros((len(ex_gold.row), R), np.float32)
        only = [int(x) for x in os.environ["FOLDS"].split(",")] if os.environ.get("FOLDS") else range(a.folds)
        for k in only:
            tr_i = [i for i in range(len(ex_gold.row)) if fold_of[i] != k]; te_i = [i for i in range(len(ex_gold.row)) if fold_of[i] == k]
            tr = Ex(); te = Ex()
            for e_, sel in ((tr, tr_i), (te, te_i)):
                e_.row = [ex_gold.row[i] for i in sel]; e_.x = ex_gold.x[sel]; e_.rows = ex_gold.rows[sel]; e_.p = ex_gold.p[sel]
            model = train(tr, f"fold{k}", a.steps)
            oof[te_i] = predict(model, te)
            log(f"fold {k}: held-out acc {acc(oof[te_i], te):.4f}")
            np.save(os.path.join(a.out, f"oof_logits_fold{k}.npy"), oof[te_i]); json.dump({"idx": te_i}, open(os.path.join(a.out, f"oof_idx_fold{k}.json"), "w"))
        if len(list(only)) == a.folds:
            np.save(os.path.join(a.out, "oof_logits.npy"), oof)
            json.dump({"row": ex_gold.row}, open(os.path.join(a.out, "oof_rows.json"), "w"))
            log(f"OOF micro {acc(oof, ex_gold):.4f}")


if __name__ == "__main__":
    main()
