"""Score a trained bag3 / ctx model (state dict) on dev_hard, dev_ewt, a pool held-out file and, only when --sets lists it, other case files.
Saves the logits as an npz (for the policy step).
usage: homograph_eval3.py MODEL.pt --arch bag3 --d 96 --buckets 65536 --hidden 256 --out OUT.npz [--heldout pool_heldout.jsonl] [--cases FILE ...] --data DIR"""
import argparse, json, os, sys
import numpy as np, torch
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import train_homograph3 as S  # noqa: E402
import train_homograph as T  # noqa: E402

ap = argparse.ArgumentParser()
ap.add_argument("model"); ap.add_argument("--arch", default="bag3"); ap.add_argument("--d", type=int, default=64)
ap.add_argument("--buckets", type=int, default=32768); ap.add_argument("--hidden", type=int, default=192); ap.add_argument("--convs", type=int, default=1)
ap.add_argument("--out", required=True); ap.add_argument("--heldout"); ap.add_argument("--cases", nargs="*", default=[])
ap.add_argument("--data", default=None); ap.add_argument("--device", default="cpu")
a = ap.parse_args()
S.NB = a.buckets
table = T.load_table(os.path.join(HERE, "..", "data", "en_homograph.tsv")); R = len(table)
import collections
word_rows = collections.defaultdict(list)
for i, (w, *_x) in enumerate(table): word_rows[w].append(i)
rowmask = torch.zeros(R, R, dtype=torch.bool)
for rs in word_rows.values():
    for r in rs: rowmask[r, rs] = True
model = S.Bag(R, a.d, a.hidden, a.buckets) if a.arch == "bag3" else S.Ctx(R, a.d, a.buckets, a.convs)
model.load_state_dict(torch.load(a.model)); model.eval()
res = {}
def run(name, e):
    e.x = S.featurize(a.arch, a.buckets, list(zip(e.units, e.t)))
    with torch.no_grad():
        lg = model(torch.from_numpy(e.x.astype(np.int64))).masked_fill(~rowmask[torch.tensor(e.row)], -1e4).numpy()
    res[name] = lg; res[name + "_row"] = np.array(e.row); res[name + "_t"] = np.array(e.t)
    acc = (lg.argmax(1) == np.array(e.row)).mean()
    print(f"{name}: {acc:.4f} (n {len(e.row)})", flush=True)
    return lg
e, _ = S.cases(os.path.join(HERE, "..", "tests", "en_homograph_dev.tsv"), table, a.data); run("dev_hard", e)
e, _ = S.cases("/mnt/data30/g2p_eval_1128/r2/sets/dev_ewt.tsv", table, a.data); run("dev_ewt", e)
for c in a.cases:
    e, _ = S.cases(c, table, a.data); run(os.path.basename(c).split(".")[0], e)
if a.heldout:
    po = [json.loads(l) for l in open(a.heldout, encoding="utf8")]
    e = S.build_ex([(o["s"], o["a"], o["b"], o["rows"][max(range(len(o["rows"])), key=lambda i: o["p"][i])]) for o in po], a.data, inits=[o["init"] for o in po])
    run("heldout", e)
np.savez(a.out, **res)
