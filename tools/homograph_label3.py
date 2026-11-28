"""Round-3 pool labelling: probability vectors of one fine-tuned teacher over the pooled candidates.

step 1 (once): keep candidates whose target the engine's unit finder locates -> CAND_OK.jsonl (record gains "row_any").
step 2 (per teacher): writes PROBS.npy, float16 [n, n_rows] (softmax over the word's rows, zeros elsewhere), same order as CAND_OK.

usage: homograph_label3.py CAND.jsonl CAND_OK.jsonl --data DATA  [--model bert-base-cased --weights W.pt --probs PROBS.npy --device cuda:0]
"""
import argparse, collections, json, os, sys, time
import numpy as np
import torch

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import train_homograph as T  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cand"); ap.add_argument("cand_ok")
    ap.add_argument("--data", default=None)
    ap.add_argument("--model"); ap.add_argument("--weights"); ap.add_argument("--probs")
    ap.add_argument("--device", default="cuda:0")
    ap.add_argument("--bs", type=int, default=256)
    a = ap.parse_args()
    table = T.load_table(os.path.join(HERE, "..", "data", "en_homograph.tsv")); R = len(table)
    word_rows = collections.defaultdict(list)
    for i, (w, *_x) in enumerate(table):
        word_rows[w].append(i)
    if not os.path.exists(a.cand_ok):
        recs = [json.loads(l) for l in open(a.cand, encoding="utf8")]
        t0 = time.time()
        units = T.rust_units([(o["s"], o["a"], o["b"]) for o in recs], a.data)
        n = 0
        with open(a.cand_ok, "w", encoding="utf8") as f:
            for o, u in zip(recs, units):
                if u is None or o["word"] not in word_rows:
                    continue
                o["row_any"] = word_rows[o["word"]][0]; f.write(json.dumps(o, ensure_ascii=False) + "\n"); n += 1
        print(f"units: {n}/{len(recs)} kept [{time.time()-t0:.0f}s]", flush=True)
    if not a.model:
        return
    from homograph_teacher import Teacher, encode, batchify  # noqa: E402
    from transformers import AutoTokenizer
    recs = [json.loads(l) for l in open(a.cand_ok, encoding="utf8")]
    dev = torch.device(a.device)
    if dev.type == "cuda":
        torch.cuda.set_per_process_memory_fraction(min(1.0, 4.0 / (torch.cuda.get_device_properties(dev).total_memory / 2**30)), dev)
    tok = AutoTokenizer.from_pretrained(a.model)
    model = Teacher(a.model, R).to(dev); model.load_state_dict(torch.load(a.weights)); model.eval()
    out = np.zeros((len(recs), R), np.float16)
    order = sorted(range(len(recs)), key=lambda i: len(recs[i]["s"]))
    t0 = time.time()
    for k in range(0, len(order), 20000):
        part = order[k:k + 20000]
        enc = encode(tok, [(recs[i]["s"], recs[i]["a"], recs[i]["b"]) for i in part])
        with torch.no_grad(), torch.autocast("cuda", dtype=torch.float16, enabled=dev.type == "cuda"):
            for j in range(0, len(part), a.bs):
                idx = list(range(j, min(len(part), j + a.bs)))
                ids, att, pos = batchify(enc, idx, dev)
                lg = model(ids, att, pos).float()
                m = torch.zeros(len(idx), R, dtype=torch.bool)
                for q, ii in enumerate(idx):
                    m[q, word_rows[recs[part[ii]]["word"]]] = True
                p = lg.masked_fill(~m.to(dev), -1e4).softmax(1).cpu().numpy()
                out[[part[ii] for ii in idx]] = p.astype(np.float16)
        print(f"{k + len(part)}/{len(order)} [{time.time()-t0:.0f}s]", flush=True)
    np.save(a.probs, out)


if __name__ == "__main__":
    main()
