"""Merge the per-source pools of tools/homograph_pool.py into one candidate list for labelling (round 3).

Per word: non-initial occurrences water-filled across sources up to --cap (equal share per source, leftovers go to the
sources that have more), sentence-initial occurrences likewise up to --init-cap. Cross-source dedupe on the lower-cased sentence.
Writes OUT.jsonl (+ "src") and OUT.counts.tsv (word, source, noninitial, initial) and the thin list (< 300 occurrences over all sources).

usage: homograph_pool_merge.py OUT.jsonl SRC=file.jsonl[,file2.jsonl] ... [--cap 12000] [--init-cap 3000]
"""
import collections, json, random, sys

def waterfill(avail, cap):
    """avail {src: n} -> {src: take}"""
    take = {s: 0 for s in avail}; left = cap; live = [s for s in avail if avail[s] > 0]
    while left > 0 and live:
        share = max(1, left // len(live)); nxt = []
        for s in live:
            t = min(share, avail[s] - take[s], left); take[s] += t; left -= t
            if take[s] < avail[s]:
                nxt.append(s)
        live = nxt
    return take

def main():
    args = [x for x in sys.argv[1:] if not x.startswith("--")]
    opt = lambda k, d: int(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d
    cap, icap = opt("--cap", 12000), opt("--init-cap", 3000)
    cap_args = {sys.argv[sys.argv.index(k) + 1] for k in ("--cap", "--init-cap") if k in sys.argv}
    args = [x for x in args if x not in cap_args]
    out, srcs = args[0], args[1:]
    rnd = random.Random(0)
    by = collections.defaultdict(lambda: collections.defaultdict(list)); seen = set()
    for spec in srcs:
        name, files = spec.split("=")
        for f in files.split(","):
            for l in open(f, encoding="utf8"):
                o = json.loads(l); k = (o["s"].lower(), o["a"])
                if k in seen:
                    continue
                seen.add(k); o["src"] = name
                by[(o["word"], o["init"])][name].append(o)
    cnt = collections.defaultdict(lambda: [0, 0]); tot = collections.Counter(); n = 0
    with open(out, "w", encoding="utf8") as f:
        for (w, init), d in sorted(by.items()):
            take = waterfill({s: len(v) for s, v in d.items()}, icap if init else cap)
            for s, v in d.items():
                rnd.shuffle(v)
                for o in v[:take[s]]:
                    f.write(json.dumps(o, ensure_ascii=False) + "\n"); n += 1
                cnt[(w, s)][int(init)] += take[s]; tot[w] += take[s]
    with open(out + ".counts.tsv", "w") as f:
        f.write("word\tsource\tnoninitial\tinitial\n")
        for (w, s), (a, b) in sorted(cnt.items()):
            f.write(f"{w}\t{s}\t{a}\t{b}\n")
    words = sorted({l.split("\t")[0] for l in open("data/en_homograph.tsv") if l.strip() and not l.startswith("#")})
    ini = sum(b for (a, b) in cnt.values())
    print(f"{n} candidates; words >=1000: {sum(tot[w] >= 1000 for w in words)}, <100: {sum(tot[w] < 100 for w in words)}, initial total {ini}")
    print("thin (<300):", [(w, tot[w]) for w in words if tot[w] < 300])

main()
