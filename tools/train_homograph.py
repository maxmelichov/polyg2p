"""Sentence-level heteronym model: which reading of "lead", "read", "wind", ... a word has in its sentence.

One design for every language (src/homograph.rs documents the input, the features and the file):
a fastText-style hashed bag of features per unit, the encoder block of tools/train_he.py, and one
linear head read at the target word, masked to the target word's rows of the reading table. Only the
reading table, the unit extractor (`en_homograph --units` for English), the silver recipes and the
benchmark are per language.

usage:
  train_homograph.py CORPUS/data --table data/en_homograph.tsv --out OUTDIR [--silver silver.jsonl]
        [--gold-only | --silver-only] [--steps 6000] [--threads 8] [--data data-out] [--tag NAME]
  CORPUS/data  the Wikipedia Homograph Data `data` folder (gold labels, train split only; never the eval split)
  --silver     output of tools/homograph_silver.py
  The gold train split is split 90/10 per homograph (seed 0) into fit and dev; model selection uses dev.
  With --silver-only no Wikipedia text is touched and 5% of the silver examples are the selection set.
  Writes OUTDIR/en_homograph.g2p (int8), en_homograph_f32.g2p, train.log, LICENCE_NOTES.txt.
"""
from __future__ import annotations

import argparse, collections, json, math, os, random, subprocess, sys, time

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
sys.path.insert(0, HERE)
from train_he import Block, write_g2pt  # noqa: E402  (the encoder block src/tagger.rs mirrors)
import en_homograph_bench as bench  # noqa: E402

N_BUCKETS, D, HEADS, LAYERS, FF, MAX_LEN = 16384, 160, 4, 4, 640, 128
N_FEAT = 9  # w, p2, p3, s1..s4, cap, T


# ---------------------------------------------------------------- features (mirror of src/homograph.rs)
def fnv1a(b: bytes) -> int:
    h = 0xcbf29ce484222325
    for x in b:
        h = ((h ^ x) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return h


_unit_cache: dict = {}


def feature_ids(unit: str, target: bool, n_buckets: int = N_BUCKETS) -> list[int]:
    """Bucket ids of the features of one unit string as `en_homograph --units` prints it (`^` = capitalised)."""
    key = (unit, target, n_buckets)
    r = _unit_cache.get(key)
    if r is not None:
        return r
    cap = unit.startswith("^") and len(unit) > 1
    w = unit[1:] if cap else unit
    n = len(w)
    f = [f"w={w}"]
    f += [f"p{k}={w[:k]}" for k in (2, 3) if n > k]
    f += [f"s{k}={w[n - k:]}" for k in (1, 2, 3, 4) if n > k]
    if cap:
        f.append("cap")
    if target:
        f.append("T")
    r = [fnv1a(s.encode("utf8")) % n_buckets for s in f]
    if len(_unit_cache) < 2_000_000:
        _unit_cache[key] = r
    return r


def window(units: list[str], target: int, max_len: int = MAX_LEN):
    if len(units) <= max_len:
        return units, target
    start = min(max(target - max_len // 2, 0), len(units) - max_len)
    return units[start:start + max_len], target - start


def encode(units: list[str], target: int, n_buckets: int = N_BUCKETS):
    """(feature array [len, N_FEAT] of the window, target index inside the window)."""
    u, t = window(units, target)
    a = np.full((len(u), N_FEAT), n_buckets, dtype=np.int32)  # n_buckets = the zero (padding) row
    for i, x in enumerate(u):
        ids = feature_ids(x, i == t, n_buckets)
        a[i, :len(ids)] = ids
    return a, t


# ---------------------------------------------------------------- units from the engine
def rust_units(items, data, exe=None):
    """items: [(sentence, byte start, byte end)] -> [(target index, [unit strings]) or None], in chunks."""
    exe = exe or os.path.join(ROOT, "target", "release", "en_homograph")
    out = []
    for i in range(0, len(items), 20000):
        chunk = items[i:i + 20000]
        inp = "".join(f"{s}\t{a}\t{b}\n" for s, a, b in chunk)
        res = subprocess.run([exe, "--units"] + (["--data", data] if data else []), input=inp, capture_output=True,
                             text=True, check=True).stdout.split("\n")
        for line in res[:len(chunk)]:
            f = line.split("\t")
            out.append((int(f[0]), f[1:]) if len(f) > 1 else None)
    return out


# ---------------------------------------------------------------- model
class HomographModel(nn.Module):
    def __init__(self, n_rows, n_buckets=N_BUCKETS, d=D, heads=HEADS, layers=LAYERS, ff=FF, max_len=MAX_LEN, drop=0.1):
        super().__init__()
        self.n_buckets = n_buckets
        self.emb = nn.Embedding(n_buckets + 1, d, padding_idx=n_buckets)
        self.pos = nn.Embedding(max_len, d)
        self.enc = nn.ModuleList([Block(d, heads, ff, drop) for _ in range(layers)])
        self.norm = nn.LayerNorm(d)
        self.out = nn.Linear(d, n_rows)
        self.drop = nn.Dropout(drop)

    def forward(self, ids, lengths, target):
        """ids [B, L, N_FEAT] (padding bucket = n_buckets), lengths [B], target [B] -> logits [B, n_rows]."""
        B, L, _ = ids.shape
        x = self.emb(ids).sum(2) + self.pos(torch.arange(L, device=ids.device))
        x = self.drop(x)
        pad = torch.arange(L, device=ids.device)[None, :] >= lengths[:, None]
        for b in self.enc:
            x = b(x, pad)
        x = self.norm(x)
        return self.out(x[torch.arange(B, device=ids.device), target])


def collate(arrs, targets, id_drop=0.0, rng=None, n_buckets=N_BUCKETS):
    L = max(len(a) for a in arrs)
    ids = np.full((len(arrs), L, N_FEAT), n_buckets, dtype=np.int64)
    for i, a in enumerate(arrs):
        ids[i, :len(a)] = a
        if id_drop:
            drop = rng.random(len(a)) < id_drop
            drop[targets[i]] = False
            ids[i, :len(a)][drop, 0] = n_buckets  # the identity feature of a context unit
    return torch.from_numpy(ids), torch.tensor([len(a) for a in arrs]), torch.tensor(targets)


# ---------------------------------------------------------------- data
def load_table(path):
    rows = []
    for l in open(path, encoding="utf8"):
        if l.startswith("#") or not l.strip():
            continue
        w, r, c, ipa = l.rstrip("\n").split("\t")
        rows.append((w, r, c, ipa))
    return rows


def load_gold(corpus, table, split="train"):
    """[(sentence, start, end, row)] of the corpus split; label -> table row through the benchmark's matcher."""
    cands = bench.load_wordids(corpus)
    row_of = {}
    for i, (w, r, c, ipa) in enumerate(table):
        wid = bench.match(ipa, cands[w])
        assert wid is not None and (w, wid) not in row_of, f"table row {w} {r} does not map to one corpus label"
        row_of[(w, wid)] = i
    items = []
    for h in sorted(cands):
        for r in bench.read_tsv(os.path.join(corpus, split, h + ".tsv")):
            items.append((r[2], int(r[3]), int(r[4]), row_of[(h, r[1])]))
    return items


def split_dev(items, table, frac=0.1, seed=0):
    """Per word: a random `frac` of its sentences go to dev."""
    by = collections.defaultdict(list)
    for i, it in enumerate(items):
        by[table[it[3]][0]].append(i)
    rnd = random.Random(seed)
    dev = set()
    for w in sorted(by):
        idx = by[w][:]
        rnd.shuffle(idx)
        dev.update(idx[:max(1, round(len(idx) * frac))])
    return [it for i, it in enumerate(items) if i not in dev], [it for i, it in enumerate(items) if i in dev]


def build(items, data, n_buckets=N_BUCKETS):
    """Examples (array, target in window, row) for the items whose target word the engine finds."""
    units = rust_units([(s, a, b) for s, a, b, _ in items], data)
    ex = []
    for (s, a, b, row), u in zip(items, units):
        if u is not None:
            arr, t = encode(u[1], u[0], n_buckets)
            ex.append((arr, t, row))
    return ex, len(items) - len(ex)


def evaluate(model, ex, table, word_rows, bs=256):
    model.eval()
    dev = next(model.parameters()).device
    per = collections.defaultdict(lambda: [0, 0])
    with torch.no_grad():
        for i in range(0, len(ex), bs):
            chunk = ex[i:i + bs]
            ids, ln, tg = (t.to(dev) for t in collate([c[0] for c in chunk], [c[1] for c in chunk]))
            logits = model(ids, ln, tg)
            mask = torch.full_like(logits, float("-inf"))
            for j, c in enumerate(chunk):
                mask[j, word_rows[table[c[2]][0]]] = 0
            pred = (logits + mask).argmax(1)
            for j, c in enumerate(chunk):
                w = table[c[2]][0]
                per[w][0] += int(pred[j] == c[2])
                per[w][1] += 1
    model.train()
    micro = sum(v[0] for v in per.values()) / max(1, sum(v[1] for v in per.values()))
    macro = sum(v[0] / v[1] for v in per.values()) / max(1, len(per))
    return micro, macro, per


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus")
    ap.add_argument("--table", default=os.path.join(ROOT, "data", "en_homograph.tsv"))
    ap.add_argument("--out", required=True)
    ap.add_argument("--silver", default=None)
    ap.add_argument("--gold-only", action="store_true")
    ap.add_argument("--silver-only", action="store_true")
    ap.add_argument("--steps", type=int, default=6000)
    ap.add_argument("--batch", type=int, default=64)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--warmup", type=int, default=300)
    ap.add_argument("--smooth", type=float, default=0.05)
    ap.add_argument("--id-drop", type=float, default=0.1)
    ap.add_argument("--gold-share", type=float, default=0.5, help="share of each batch that is gold when both are used")
    ap.add_argument("--layers", type=int, default=LAYERS)
    ap.add_argument("--buckets", type=int, default=N_BUCKETS)
    ap.add_argument("--eval-every", type=int, default=500)
    ap.add_argument("--threads", type=int, default=8)
    ap.add_argument("--device", default="cpu", help="cpu, or cuda:N when a GPU is free (the model needs about 1 GB)")
    ap.add_argument("--data", default=os.path.join(ROOT, "data-out"))
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--tag", default="")
    a = ap.parse_args()
    assert not (a.gold_only and a.silver_only)
    torch.set_num_threads(min(a.threads, 8))
    torch.manual_seed(a.seed)
    rnd, nrng = random.Random(a.seed), np.random.default_rng(a.seed)
    os.makedirs(a.out, exist_ok=True)
    logf = open(os.path.join(a.out, "train.log"), "a", encoding="utf8")

    def log(*m):
        s = " ".join(str(x) for x in m)
        print(s, flush=True)
        logf.write(s + "\n")
        logf.flush()

    table = load_table(a.table)
    word_rows = collections.defaultdict(list)
    for i, (w, *_r) in enumerate(table):
        word_rows[w].append(i)
    R = len(table)
    log(f"== {a.tag or os.path.basename(a.out)}: {' '.join(sys.argv[1:])}")
    gold_fit, gold_dev, silver_fit, silver_dev = [], [], [], []
    if not a.silver_only:
        g = load_gold(a.corpus, table, "train")
        fit, dev = split_dev(g, table)
        gold_fit, miss1 = build(fit, a.data, a.buckets)
        gold_dev, miss2 = build(dev, a.data, a.buckets)
        log(f"gold: fit {len(gold_fit)} dev {len(gold_dev)} (target not found: {miss1 + miss2})")
    if a.silver and not a.gold_only:
        sil = [json.loads(l) for l in open(a.silver, encoding="utf8")]
        items = [(o["s"], o["a"], o["b"], o["row"]) for o in sil]
        ex, miss = build(items, a.data, a.buckets)
        rnd.shuffle(ex)
        if a.silver_only:
            k = len(ex) // 20
            silver_dev, silver_fit = ex[:k], ex[k:]
        else:
            silver_fit = ex
        log(f"silver: fit {len(silver_fit)} dev {len(silver_dev)} (target not found: {miss})")
    assert gold_fit or silver_fit
    dev_set = gold_dev if gold_dev else silver_dev
    silver_by_row = collections.defaultdict(list)
    for e in silver_fit:
        silver_by_row[e[2]].append(e)
    silver_rows = sorted(silver_by_row)
    log(f"silver readings covered: {len(silver_rows)} of {R}")

    dev = torch.device(a.device)
    if dev.type == "cuda":
        torch.cuda.set_per_process_memory_fraction(min(1.0, 3.0 / (torch.cuda.get_device_properties(dev).total_memory / 2**30)), dev)
    model = HomographModel(R, a.buckets, layers=a.layers).to(dev)
    log(f"parameters {sum(p.numel() for p in model.parameters()) / 1e6:.2f} M")
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01, betas=(0.9, 0.98))
    sched = lambda s: min(1.0, (s + 1) / a.warmup) * (0.5 * (1 + math.cos(math.pi * min(1.0, s / a.steps))) * 0.99 + 0.01)
    best, best_state = -1.0, None
    model.train()
    t0 = time.time()
    for step in range(a.steps):
        for g in opt.param_groups:
            g["lr"] = a.lr * sched(step)
        n_gold = 0 if not gold_fit else (a.batch if not silver_fit else int(a.batch * a.gold_share))
        batch = [gold_fit[rnd.randrange(len(gold_fit))] for _ in range(n_gold)]
        batch += [silver_by_row[r][rnd.randrange(len(silver_by_row[r]))] for r in (rnd.choice(silver_rows) for _ in range(a.batch - n_gold))] if silver_fit else []
        ids, ln, tg = (t.to(dev) for t in collate([b[0] for b in batch], [b[1] for b in batch], a.id_drop, nrng, a.buckets))
        logits = model(ids, ln, tg)
        mask = torch.zeros_like(logits, dtype=torch.bool)
        for j, b in enumerate(batch):
            mask[j, word_rows[table[b[2]][0]]] = True
        logits = logits.masked_fill(~mask, -1e4)
        logp = logits.log_softmax(1)
        y = torch.tensor([b[2] for b in batch], device=dev)
        nll = -logp[torch.arange(len(batch)), y]
        smooth = -(logp * mask).sum(1) / mask.sum(1)
        loss = ((1 - a.smooth) * nll + a.smooth * smooth).mean()
        opt.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        opt.step()
        if step == 199:
            eta = (time.time() - t0) / 200 * (a.steps - 200) / 60
            log(f"200 steps in {time.time() - t0:.0f}s: ETA {eta:.1f} min for the rest")
        if (step + 1) % a.eval_every == 0 or step + 1 == a.steps:
            micro, macro, _ = evaluate(model, dev_set, table, word_rows)
            log(f"step {step + 1} loss {loss.item():.3f} dev micro {micro:.4f} macro {macro:.4f}  [{(time.time() - t0) / 60:.1f} min]")
            if micro >= best:
                best = micro
                best_state = {k: v.detach().clone() for k, v in model.state_dict().items()}
                best_step = step + 1
    model.load_state_dict(best_state)
    micro, macro, per = evaluate(model, dev_set, table, word_rows)
    log(f"best: step {best_step} dev micro {micro:.4f} macro {macro:.4f}")
    worst = sorted(per.items(), key=lambda kv: kv[1][0] / kv[1][1])[:10]
    log("ten worst words on dev: " + ", ".join(f"{w} {c}/{n}" for w, (c, n) in worst))
    export(model, table, a, os.path.join(a.out, "en_homograph_f32.g2p"), False)
    export(model, table, a, os.path.join(a.out, "en_homograph.g2p"), True)
    torch.save({k: v.cpu() for k, v in best_state.items()}, os.path.join(a.out, "best.pt"))
    notes(a, os.path.join(a.out, "LICENCE_NOTES.txt"))
    log("exported", os.path.join(a.out, "en_homograph.g2p"))


def export(model, table, a, path, int8):
    sd = {k: v.cpu() for k, v in model.state_dict().items()}
    sd["emb.weight"] = sd["emb.weight"][:model.n_buckets]  # drop the padding row
    cfg = {"d": D, "heads": HEADS, "layers": a.layers, "ff": FF, "max_len": MAX_LEN, "n_buckets": model.n_buckets, "n_out": len(table)}
    header = {"magic": "G2PT", "kind": "homograph", "lang": "en", "features": "v1", "config": cfg,
              "readings": [f"{w}\t{r}" for w, r, *_ in table], "tensors": []}
    # the sentence encoder trunk uses the names of train_he.Tagger: pos.weight, enc.{l}.*, norm.*
    size = write_g2pt(sd, header, path, int8)
    print(f"{path}: {size / 1e6:.1f} MB")


def commit_of_repo():
    try:
        return subprocess.run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    except Exception:
        return "unknown"


def notes(a, path):
    gold = not a.silver_only
    lines = ["English heteronym model (en_homograph.g2p): training sources and licences", "",
             "Trainer: tools/train_homograph.py, polyg2p commit " + commit_of_repo(), ""]
    if gold:
        lines += ["- Wikipedia Homograph Data, train split (Gorman, Mazovetskiy & Nikolaev 2018): repository licence Apache-2.0;",
                  "  its sentences are English Wikipedia text, CC BY-SA 3.0. Whether weights trained on them are a",
                  "  share-alike derivative is unsettled, so this file is not part of the crate or the wheel."]
    else:
        lines += ["- No Wikipedia text and no corpus labels were used (silver-only model)."]
    if a.silver and not a.gold_only:
        lines += ["- Tatoeba English sentences: CC BY 2.0 FR (part CC0), raw text for silver examples (tools/homograph_silver.py).",
                  "- WordNet 3.1 database files: WordNet licence (permissive, attribution), part-of-speech inventory and irregular forms.",
                  "- The reading table data/en_homograph.tsv and the adversarial sentences are author-written."]
    open(path, "w", encoding="utf8").write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
