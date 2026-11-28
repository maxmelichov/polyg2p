"""Two-teacher agreement filter for the round-3 pool (4.2 of the plan).

step `agree`: CAND_OK + two PROBS.npy -> AGREED.jsonl (records with rows, p = mean of the two distributions, init) for items where both
  teachers pick the same reading with probability >= 0.90 (>= 0.95 when the target is sentence-initial; --cut overrides the 0.90).
step `select`: AGREED + POS tags (TSV, same order, '-' = skip) -> POOL_TRAIN.jsonl / POOL_HELDOUT.jsonl + counts.tsv:
  POS check for part-of-speech pairs only (words whose readings have pairwise disjoint classes, all in N A V PAST NAME), not at sentence-initial
  tokens; at most --cap kept per word (all sentence-initial items kept first), a floor of 15% for each reading with >= 30 kept items;
  10% of the kept items per word go to the held-out file.
usage: homograph_agree.py agree CAND_OK P1.npy P2.npy OUT [--cut .9]
       homograph_agree.py select AGREED POS.tsv OUT_PREFIX [--cap 6000]
"""
import collections, json, os, random, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
table = [l.rstrip("\n").split("\t") for l in open(os.path.join(HERE, "..", "data", "en_homograph.tsv"), encoding="utf8") if l.strip() and not l.startswith("#")]
word_rows = collections.defaultdict(list)
for i, (w, *_x) in enumerate(table):
    word_rows[w].append(i)
opt = lambda k, d: type(d)(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d

TAGCLS = {"NN": {"N"}, "NNS": {"N"}, "NNP": {"N", "NAME"}, "NNPS": {"N", "NAME"}, "JJ": {"A"}, "JJR": {"A"}, "JJS": {"A"},
          "VB": {"V"}, "VBP": {"V"}, "VBZ": {"V"}, "VBD": {"V", "PAST"}, "VBN": {"V", "PAST", "A"}, "VBG": {"V", "N", "A"}}


def pos_pair(w):
    cl = [set(table[r][2].split(",")) for r in word_rows[w]]
    if not all(c <= {"N", "A", "V", "PAST", "NAME"} for c in cl):
        return False
    return all(not (cl[i] & cl[j]) for i in range(len(cl)) for j in range(i + 1, len(cl)))


def main():
    if sys.argv[1] == "agree":
        recs = [json.loads(l) for l in open(sys.argv[2], encoding="utf8")]
        p1, p2 = np.load(sys.argv[3]).astype(np.float32), np.load(sys.argv[4]).astype(np.float32)
        cut = opt("--cut", 0.90); n = 0
        a1, a2 = p1.argmax(1), p2.argmax(1)
        with open(sys.argv[5], "w", encoding="utf8") as f:
            for i, o in enumerate(recs):
                need = max(cut, 0.95) if o["init"] else cut
                if a1[i] != a2[i] or p1[i, a1[i]] < need or p2[i, a2[i]] < need:
                    continue
                rs = word_rows[o["word"]]; p = (p1[i, rs] + p2[i, rs]) / 2
                o["rows"] = rs; o["p"] = [round(float(x), 4) for x in p]; o["row_any"] = int(a1[i])
                f.write(json.dumps(o, ensure_ascii=False) + "\n"); n += 1
        print(f"agreed {n}/{len(recs)} (cut {cut})")
        return
    recs = [json.loads(l) for l in open(sys.argv[2], encoding="utf8")]
    tags = [l.strip() for l in open(sys.argv[3])]
    assert len(tags) == len(recs)
    cap = opt("--cap", 6000); rnd = random.Random(0)
    pp = {w: pos_pair(w) for w in word_rows}
    kept = collections.defaultdict(list); contra = 0
    for o, tg in zip(recs, tags):
        top = int(o["row_any"])
        if pp[o["word"]] and not o["init"] and tg in TAGCLS:
            if not (TAGCLS[tg] & set(table[top][2].split(","))):
                contra += 1; continue
        kept[o["word"]].append(o)
    tr, ho = [], []; cnt = []
    for w, items in sorted(kept.items()):
        ini = [o for o in items if o["init"]]; rest = [o for o in items if not o["init"]]
        by = collections.defaultdict(list)
        for o in rest:
            by[o["row_any"]].append(o)
        room = max(0, cap - len(ini)); chosen = []
        big = [r for r in by if len(by[r]) + sum(1 for o in ini if o["row_any"] == r) >= 30]
        for r in big:
            fl = min(len(by[r]), int(0.15 * cap)); rnd.shuffle(by[r]); chosen += by[r][:fl]; by[r] = by[r][fl:]
        left = [o for r in by for o in by[r]]; rnd.shuffle(left)
        chosen += left[:max(0, room - len(chosen))]
        sel = ini + chosen; rnd.shuffle(sel)
        nh = max(1, len(sel) // 10) if len(sel) >= 10 else 0
        ho += sel[:nh]; tr += sel[nh:]
        c = collections.Counter(table[o["row_any"]][1] for o in sel)
        cnt.append((w, len(sel), len(ini), dict(c)))
    for name, rows in (("_train.jsonl", tr), ("_heldout.jsonl", ho)):
        with open(sys.argv[4] + name, "w", encoding="utf8") as f:
            for o in rows:
                f.write(json.dumps({k: o[k] for k in ("s", "a", "b", "word", "init", "rows", "p", "src")}, ensure_ascii=False) + "\n")
    with open(sys.argv[4] + "_counts.tsv", "w") as f:
        f.write("word\tkept\tinitial\treadings\n")
        for w, n, i, c in cnt:
            f.write(f"{w}\t{n}\t{i}\t{c}\n")
    print(f"POS contradictions dropped {contra}; train {len(tr)} heldout {len(ho)}; initial in train {sum(o['init'] for o in tr)}")


main()
