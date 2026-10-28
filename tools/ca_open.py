"""Catalan: ending statistics for open/closed stressed e/o -> data/ca_open.tsv.

For every unaccented word of the Central gold whose stressed vowel is e/o,
record whether Wiktionary writes it open (ɛ ɔ) or closed (e o) under every
prefix of the ending (e.g. for "festa": e, ef, efs, efst, efsta#, plus the
same keys with the preceding letter: fe, fef, ...). Only aggregate counts are
written. `src/ca.rs` picks the longest key with count >= 3.

Also prints a 5-fold cross-validation of the lookup.
usage: ca_open.py gold.tsv data/ca_open.tsv
"""
import sys, collections, re, random
sys.path.insert(0, __import__("os").path.dirname(__import__("os").path.abspath(__file__)))
from ca_gold import stress_index


def samples(gold_path):
    by = collections.defaultdict(list)
    for line in open(gold_path, encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        by[w].append(p)
    out = []
    for w, ps in by.items():
        if not w.islower() or not re.fullmatch(r"[a-zçàèéíòóúïü·]+", w):
            continue
        si = stress_index(w)
        if si is None:
            continue
        sv = w[si]
        if sv not in "eo" or any(ch in "àèéíòóú" for ch in w):
            continue
        votes = []
        for p in ps:
            mids = [ch for ch in p if ch in "eɛoɔ"]
            if len(mids) != 1:
                continue
            m = mids[0]
            if (sv == "e" and m in "eɛ") or (sv == "o" and m in "oɔ"):
                votes.append(m in "ɛɔ")
        if votes:
            out.append((w, sv, w[si + 1:], w[si - 1] if si > 0 else "#", sum(votes) / len(votes)))
    return out


def keys(sv, rest, prev):
    ks = []
    for k in range(len(rest) + 1):
        key = sv + rest[:k] + ("#" if k == len(rest) else "")
        ks.append(key)
        ks.append(prev + key)
    return ks


def train(ss):
    t = collections.defaultdict(lambda: [0.0, 0.0])
    for w, sv, rest, prev, op in ss:
        for key in keys(sv, rest, prev):
            t[key][0] += op
            t[key][1] += 1
    return t


def predict(t, sv, rest, prev, minc=3):
    best = None
    for key in keys(sv, rest, prev):
        if key in t and t[key][1] >= minc:
            best = t[key]
    return 0.5 if best is None else best[0] / best[1]


def main():
    gold, out = sys.argv[1], sys.argv[2]
    ss = samples(gold)
    random.seed(1)
    random.shuffle(ss)
    ok = tot = 0
    for f in range(5):
        t = train([s for i, s in enumerate(ss) if i % 5 != f])
        for w, sv, rest, prev, op in ss[f::5]:
            pred = predict(t, sv, rest, prev) > 0.5
            ok += (pred and op > 0) or (not pred and op < 1)
            tot += 1
    print(f"{len(ss)} samples, held-out accuracy {ok / tot:.1%}")
    t = train(ss)
    with open(out, "w", encoding="utf8") as f:
        f.write("# Catalan: open/closed quality of unaccented stressed e/o by ending (derived from Wiktionary Central transcriptions; aggregate counts only)\n")
        f.write("# key = [preceding letter] stressed vowel + following letters ('#' = word end), open ratio, count\n")
        for k, (o, n) in sorted(t.items()):
            if n >= 3:
                f.write(f"{k}\t{o / n:.2f}\t{int(n)}\n")


if __name__ == "__main__":
    main()
