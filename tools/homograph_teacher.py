"""Teacher for the English heteronym model (training time only, never shipped).

A pretrained cased encoder fine-tuned as a reading classifier: the hidden state at the target's first sub-word -> Linear to
the rows of the reading table, masked to the word's rows. Fine-tunes on gold fit + silver v2 + seeds, scores gold dev,
dev_hard and dev_ewt (G3), then labels a pool of Tatoeba sentences with probability vectors.

usage: homograph_teacher.py CORPUS/data --silver silver_v2.jsonl --seeds seeds.jsonl --pool pool_items.jsonl --out DIR
         [--model bert-base-cased] [--epochs 3] [--device cuda:1] [--max-pool-per-word 2000]
"""
import argparse, collections, json, os, random, re, sys, time
import numpy as np
import torch, torch.nn as nn

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import train_homograph as T  # noqa: E402
import en_homograph_bench as bench  # noqa: E402
from transformers import AutoModel, AutoTokenizer  # noqa: E402


def cases_items(path, table):
    row_of = {(w, r): i for i, (w, r, *_x) in enumerate(table)}
    out = []
    for word, reading, sent, a, b, _m in bench.read_cases(path):
        out.append((sent, a, b, row_of[(word, reading)]))
    return out


class Teacher(nn.Module):
    def __init__(self, name, n_rows):
        super().__init__()
        self.enc = AutoModel.from_pretrained(name)
        self.drop = nn.Dropout(0.1)
        self.head = nn.Linear(self.enc.config.hidden_size, n_rows)

    def forward(self, ids, att, pos):
        h = self.enc(input_ids=ids, attention_mask=att).last_hidden_state
        return self.head(self.drop(h[torch.arange(len(pos)), pos]))


def encode(tok, items, max_chars=300):
    """-> list of (ids, target token index)."""
    out = []
    for s, a, b, *_ in items:
        ca = len(s.encode("utf8")[:a].decode("utf8"))
        lo = max(0, ca - max_chars // 2)
        txt = s[lo:lo + max_chars]
        enc = tok(txt, return_offsets_mapping=True, truncation=True, max_length=96)
        ca -= lo
        pos = 1
        for k, (x, y) in enumerate(enc["offset_mapping"]):
            if y > x and x <= ca < y:
                pos = k
                break
        out.append((enc["input_ids"], pos))
    return out


def batchify(enc, idx, dev):
    L = max(len(enc[i][0]) for i in idx)
    ids = torch.zeros(len(idx), L, dtype=torch.long)
    att = torch.zeros(len(idx), L, dtype=torch.long)
    for j, i in enumerate(idx):
        ids[j, :len(enc[i][0])] = torch.tensor(enc[i][0])
        att[j, :len(enc[i][0])] = 1
    return ids.to(dev), att.to(dev), torch.tensor([enc[i][1] for i in idx]).to(dev)


def predict(model, enc, rows_mask, dev, bs=128):
    """softmax over each item's word rows -> [n, R] numpy."""
    model.eval()
    out = []
    with torch.no_grad(), torch.autocast("cuda", dtype=torch.float16, enabled=dev.type == "cuda"):
        for i in range(0, len(enc), bs):
            idx = list(range(i, min(len(enc), i + bs)))
            ids, att, pos = batchify(enc, idx, dev)
            lg = model(ids, att, pos).float()
            lg = lg.masked_fill(~rows_mask[idx].to(dev), -1e4)
            out.append(lg.softmax(1).cpu().numpy())
    model.train()
    return np.concatenate(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus")
    ap.add_argument("--table", default=os.path.join(HERE, "..", "data", "en_homograph.tsv"))
    ap.add_argument("--silver"), ap.add_argument("--seeds"), ap.add_argument("--pool"), ap.add_argument("--out", required=True)
    ap.add_argument("--model", default="bert-base-cased")
    ap.add_argument("--epochs", type=float, default=3)
    ap.add_argument("--lr", type=float, default=3e-5)
    ap.add_argument("--batch", type=int, default=32)
    ap.add_argument("--device", default="cuda:1")
    ap.add_argument("--max-pool-per-word", type=int, default=2000)
    ap.add_argument("--silver-cap", type=int, default=40000)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--load", default=None, help="skip training: load these weights")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    logf = open(os.path.join(a.out, "teacher.log"), "a")
    def log(*m):
        s = " ".join(str(x) for x in m); print(s, flush=True); logf.write(s + "\n"); logf.flush()
    rnd = random.Random(a.seed); torch.manual_seed(a.seed)
    dev = torch.device(a.device)
    if dev.type == "cuda":
        torch.cuda.set_per_process_memory_fraction(min(1.0, 4.0 / (torch.cuda.get_device_properties(dev).total_memory / 2**30)), dev)
    table = T.load_table(a.table)
    R = len(table)
    word_rows = collections.defaultdict(list)
    for i, (w, *_x) in enumerate(table):
        word_rows[w].append(i)
    def mask_of(items):
        m = torch.zeros(len(items), R, dtype=torch.bool)
        for j, it in enumerate(items):
            m[j, word_rows[table[it[3]][0]]] = True
        return m
    gold = T.load_gold(a.corpus, table, "train")
    fit, devg = T.split_dev(gold, table)
    extra = []
    for p in (a.silver, a.seeds):
        if p:
            extra += [(o["s"], o["a"], o["b"], o["row"]) for o in map(json.loads, open(p, encoding="utf8"))]
    rnd.shuffle(extra)
    train = fit + extra[:a.silver_cap]
    rnd.shuffle(train)
    log(f"teacher {a.model}: train {len(train)} (gold fit {len(fit)}, silver+seeds {min(len(extra), a.silver_cap)}), dev {len(devg)}")
    tok = AutoTokenizer.from_pretrained(a.model)
    enc_train = encode(tok, train)
    sets = {"gold_dev": devg, "dev_hard": cases_items(os.path.join(ROOT := os.path.join(HERE, ".."), "tests", "en_homograph_dev.tsv"), table)}
    ewt = "/mnt/data30/g2p_eval_1128/r2/sets/dev_ewt.tsv"
    if os.path.exists(ewt):
        sets["dev_ewt"] = cases_items(ewt, table)
    enc_sets = {k: (v, encode(tok, v), mask_of(v)) for k, v in sets.items()}
    model = Teacher(a.model, R).to(dev)
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=0.01)
    steps = 0 if a.load else int(a.epochs * len(train) / a.batch)
    if a.load:
        model.load_state_dict(torch.load(a.load))
    sched = torch.optim.lr_scheduler.LambdaLR(opt, lambda s: min((s + 1) / (0.06 * max(steps, 1)), max(0.0, (steps - s) / (max(steps, 1) * 0.94))))
    scaler = torch.cuda.amp.GradScaler(enabled=dev.type == "cuda")
    mtrain = mask_of(train)
    ytrain = torch.tensor([t[3] for t in train])
    order, k = [], 0
    t0 = time.time()
    def evaluate():
        res = {}
        for name, (items, enc, m) in enc_sets.items():
            p = predict(model, enc, m, dev)
            res[name] = float(np.mean([p[i].argmax() == it[3] for i, it in enumerate(items)]))
        return res
    for step in range(steps):
        if k + a.batch > len(order):
            order = list(range(len(train))); rnd.shuffle(order); k = 0
        idx = order[k:k + a.batch]; k += a.batch
        ids, att, pos = batchify(enc_train, idx, dev)
        with torch.autocast("cuda", dtype=torch.float16, enabled=dev.type == "cuda"):
            lg = model(ids, att, pos)
        lg = lg.float().masked_fill(~mtrain[idx].to(dev), -1e4)
        loss = nn.functional.cross_entropy(lg, ytrain[idx].to(dev), label_smoothing=0.05)
        opt.zero_grad(); scaler.scale(loss).backward(); scaler.unscale_(opt)
        nn.utils.clip_grad_norm_(model.parameters(), 1.0); scaler.step(opt); scaler.update(); sched.step()
        if step == 99:
            log(f"100 steps in {time.time() - t0:.0f}s: ETA {(time.time() - t0) / 100 * (steps - 100) / 60:.1f} min")
        if (step + 1) % 1000 == 0 or step + 1 == steps:
            log(f"step {step + 1}/{steps} loss {loss.item():.3f} {evaluate()} [{(time.time() - t0) / 60:.1f} min]")
    res = evaluate()
    log("FINAL", json.dumps(res))
    json.dump(res, open(os.path.join(a.out, "teacher_eval.json"), "w"))
    torch.save({k: v.cpu() for k, v in model.state_dict().items()}, os.path.join(a.out, "teacher.pt"))
    if a.pool:
        pool = [json.loads(l) for l in open(a.pool, encoding="utf8")]
        items = [(o["s"], o["a"], o["b"], o["row_any"]) for o in pool]
        # pool rows: every table row of the word is allowed
        m = torch.zeros(len(items), R, dtype=torch.bool)
        for j, o in enumerate(pool):
            m[j, word_rows[o["word"]]] = True
        p = predict(model, encode(tok, items), m, dev)
        kept = collections.defaultdict(list)
        stats = collections.defaultdict(lambda: [0, 0, collections.Counter()])
        for j, o in enumerate(pool):
            top = int(p[j].argmax()); st = stats[o["word"]]; st[0] += 1
            if p[j][top] >= 0.9:
                kept[o["word"]].append((j, top)); st[1] += 1; st[2][table[top][1]] += 1
        with open(os.path.join(a.out, "..", "..", "pool", "pool_labels.jsonl"), "w", encoding="utf8") as f, open(os.path.join(a.out, "..", "..", "pool", "pool_counts.tsv"), "w") as fc:
            fc.write("word\tseen\tkept\tdropped\treading_counts\n")
            for w in sorted(stats):
                ks = kept[w]; rnd.shuffle(ks); ks = ks[:a.max_pool_per_word]
                for j, top in ks:
                    o = pool[j]
                    f.write(json.dumps({"s": o["s"], "a": o["a"], "b": o["b"], "row": top, "p": {str(r): round(float(p[j][r]), 4) for r in word_rows[w]}}) + "\n")
                fc.write(f"{w}\t{stats[w][0]}\t{len(ks)}\t{stats[w][0] - len(ks)}\t{dict(stats[w][2])}\n")
        log("pool labelled")


if __name__ == "__main__":
    main()
