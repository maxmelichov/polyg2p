"""Export a bag3 checkpoint of tools/train_homograph3.py as a G2PT `homograph` v3 file (int8 embedding rows + f32 scale, policy bias
folded into out.bias) and check it against the Rust engine: logits of PyTorch (dequantised embedding, same bias) vs `en_homograph --logits`.

usage: homograph_bag3_export.py CKPT.pt OUT.g2p --d 96 --hidden 256 --buckets 65536 [--policy policy_final.json] [--parity DATA_DIR [--extra jsonl N]]
policy_final.json: {"bias_rows": {"word\\treading": bias, ...}, ...} (added to out.bias; the header keeps the policy for the record)
"""
import argparse, json, os, struct, subprocess, sys
import numpy as np, torch

HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.join(HERE, "..")
sys.path.insert(0, HERE)
import train_homograph3 as S  # noqa: E402
import train_homograph as T  # noqa: E402
import en_homograph_bench as bench  # noqa: E402


def export(sd, table, path, d, h, nb, bias_rows, policy):
    emb = sd["emb.weight"][:nb].float()
    scale = emb.abs().amax(1).clamp(min=1e-8) / 127.0
    q = torch.round(emb / scale[:, None]).clamp(-127, 127).to(torch.int8)
    ob = sd["out.bias"].float().clone()
    for i, (w, r, *_x) in enumerate(table):
        ob[i] += bias_rows.get(f"{w}\t{r}", 0.0)
    tensors, blobs, off = [], [], 0
    def add(name, arr, dtype=None):
        nonlocal off
        b = arr.tobytes(); m = {"name": name, "shape": list(arr.shape), "offset": off}
        if dtype: m["dtype"] = dtype
        tensors.append(m); blobs.append(b); off += len(b)
    add("emb.weight", q.numpy(), "i8"); add("emb.weight.scale", scale.numpy().astype("<f4"))
    add("emb.pad", sd["emb.weight"][nb].float().numpy().astype("<f4"))  # the trainer's padding row is not zero; the engine adds it once per missing feature slot
    add("l1.weight", sd["l1.weight"].float().numpy().astype("<f4")); add("l1.bias", sd["l1.bias"].float().numpy().astype("<f4"))
    add("out.weight", sd["out.weight"].float().numpy().astype("<f4")); add("out.bias", ob.numpy().astype("<f4"))
    header = {"magic": "G2PT", "kind": "homograph", "lang": "en", "features": "v3", "config": {"d": d, "h": h, "n_buckets": nb, "n_out": len(table)},
              "readings": [f"{w}\t{r}" for w, r, *_ in table], "tensors": tensors}
    if policy:
        header["policy"] = policy
    hj = json.dumps(header, ensure_ascii=False).encode("utf8")
    with open(path, "wb") as f:
        f.write(b"G2PT"); f.write(struct.pack("<I", len(hj))); f.write(hj)
        for b in blobs: f.write(b)
    return q.float() * scale[:, None], ob


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("ckpt"); ap.add_argument("out")
    ap.add_argument("--hidden", type=int, default=256); ap.add_argument("--buckets", type=int, default=65536); ap.add_argument("--d", type=int, default=96)
    ap.add_argument("--policy", default=None); ap.add_argument("--parity", default=None); ap.add_argument("--extra", nargs=2, default=None)
    a = ap.parse_args()
    S.NB = a.buckets
    table = T.load_table(os.path.join(ROOT, "data", "en_homograph.tsv"))
    sd = torch.load(a.ckpt)
    pol = json.load(open(a.policy)) if a.policy else {}
    deq, ob = export(sd, table, a.out, a.d, a.hidden, a.buckets, pol.get("bias_rows", {}), {k: v for k, v in pol.items() if k != "share"} or None)
    print("wrote", a.out, os.path.getsize(a.out) / 1e6, "MB")
    if not a.parity:
        return
    sd2 = dict(sd); sd2["emb.weight"] = torch.cat([deq, sd["emb.weight"][a.buckets:a.buckets + 1]]); sd2["out.bias"] = ob
    model = S.Bag(len(table), a.d, a.hidden, a.buckets); model.load_state_dict(sd2); model.eval()
    its = [(s, a_, b) for w, r, s, a_, b, m in bench.read_cases(os.path.join(ROOT, "tests", "en_homograph_dev.tsv"))]
    words = [w for w, *_ in bench.read_cases(os.path.join(ROOT, "tests", "en_homograph_dev.tsv"))]
    if a.extra:
        for l in list(open(a.extra[0], encoding="utf8"))[:int(a.extra[1])]:
            o = json.loads(l); its.append((o["s"], o["a"], o["b"])); words.append(o["word"])
    units = T.rust_units(its, a.parity)
    inp = "".join(f"{s}\t{a_}\t{b}\n" for s, a_, b in its)
    exe = os.path.join(ROOT, "target", "release", "en_homograph")
    rust = subprocess.run([exe, "--logits", "--data", a.parity], input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
    maxd, agree, n, skipped = 0.0, 0, 0, 0
    for w, u, line in zip(words, units, rust):
        if u is None or not line:
            skipped += 1; continue
        ids = np.full((1, S.NF), a.buckets, dtype=np.int64)
        f = S.v3_ids(u[1], u[0], a.buckets); ids[0, :len(f)] = f
        with torch.no_grad():
            lg = model(torch.from_numpy(ids))[0]
        rows = [i for i, t in enumerate(table) if t[0] == w]
        rl = [float(x.rsplit(":", 1)[1]) for x in line.split(" ")]
        pl = [float(lg[i]) for i in rows]
        maxd = max(maxd, max(abs(x - y) for x, y in zip(rl, pl)))
        agree += int(np.argmax(rl) == np.argmax(pl)); n += 1
    print(f"parity: {n} sentences ({skipped} skipped), max |logit diff| {maxd:.2e}, argmax agreement {agree}/{n}")


if __name__ == "__main__":
    main()
