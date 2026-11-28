"""Per-word bias and fixed-majority policy from out-of-fold logits (plan 5.6), plus the A2 pre-check without the eval split.
usage: homograph_policy.py OOF_DIR OUT_POLICY.json [--table T]
Steps 1 (bias b in {0,.5,1,1.5,2,3} on the gold-majority row) and 3 (fixed reading when still under the majority count);
step 2 (rules) is not run here: the rules' train-split accuracy is not part of this tool."""
import collections, json, os, random, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
BIAS = [0, 0.5, 1, 1.5, 2, 3]


def main():
    oof_dir, out = sys.argv[1:3]
    table = [l.rstrip("\n").split("\t") for l in open(os.path.join(HERE, "..", "data", "en_homograph.tsv"), encoding="utf8") if not l.startswith("#") and l.strip()]
    lg = np.load(os.path.join(oof_dir, "oof_logits.npy"))
    rows = np.array(json.load(open(os.path.join(oof_dir, "oof_rows.json")))["row"])
    word = [table[r][0] for r in rows]
    by = collections.defaultdict(list)
    for i, w in enumerate(word):
        by[w].append(i)
    maj = {w: collections.Counter(rows[ix]).most_common(1)[0][0] for w, ix in by.items()}
    def acc(ix, b, w):
        l = lg[ix].copy(); l[:, maj[w]] += b
        return (l.argmax(1) == rows[ix])
    bias, fixed = {}, {}
    for w, ix in by.items():
        scores = [(acc(ix, b, w).sum(), -b) for b in BIAS]
        bias[w] = -max(scores)[1]
        n_maj = int((rows[ix] == maj[w]).sum())
        if acc(ix, bias[w], w).sum() < n_maj:
            fixed[w] = int(maj[w])
    def totals(bias_, fixed_, ixs=None):
        c = n = 0; mac = []
        for w, ix in by.items():
            ix = ix if ixs is None else [i for i in ix if i in ixs]
            if not ix:
                continue
            ok = (rows[ix] == fixed_[w]) if w in fixed_ else acc(ix, bias_[w], w)
            c += int(ok.sum()); n += len(ix); mac.append(ok.mean())
        return c / n, float(np.mean(mac))
    zero = {w: 0 for w in by}
    print("OOF no policy   micro %.4f macro %.4f" % totals(zero, {}))
    print("OOF bias only   micro %.4f macro %.4f (fit and scored on the same OOF: optimistic)" % totals(bias, {}))
    print("OOF bias+fixed  micro %.4f macro %.4f (optimistic)" % totals(bias, fixed))
    # A2 pre-check: fit on a random half of each word's OOF sentences, test on the other half
    rnd = random.Random(0)
    below, split_acc = [], []
    for rep in range(20):
        fit_ix, test_ix = {}, {}
        for w, ix in by.items():
            ix = ix[:]; rnd.shuffle(ix); h = len(ix) // 2
            fit_ix[w], test_ix[w] = ix[:h], ix[h:]
        b_ = {}; f_ = {}
        for w in by:
            sc = [(acc(fit_ix[w], b, w).sum(), -b) for b in BIAS]
            b_[w] = -max(sc)[1]
            if acc(fit_ix[w], b_[w], w).sum() < int((rows[fit_ix[w]] == maj[w]).sum()):
                f_[w] = int(maj[w])
        okw = {w: ((rows[test_ix[w]] == f_[w]) if w in f_ else acc(test_ix[w], b_[w], w)) for w in by}
        split_acc.append(np.mean(np.concatenate(list(okw.values()))))
        for _ in range(10):
            n_below = 0
            for w in by:
                ix = [rnd.randrange(len(test_ix[w])) for _ in range(10)]
                got = int(okw[w][ix].sum()); mj = int(sum(rows[test_ix[w]][i] == maj[w] for i in ix))
                n_below += got < mj
            below.append(n_below)
    print("held-out-half micro (mean of 20 splits) %.4f" % np.mean(split_acc))
    print("A2 pre-check: words below majority per 10-sentence draw: mean %.1f p90 %.1f (n draws %d)" % (np.mean(below), np.percentile(below, 90), len(below)))
    json.dump({"rules": [], "fixed": {table[r][0]: table[r][1] for r in fixed.values()}, "bias": {w: b for w, b in bias.items() if b}}, open(out, "w"), indent=1)
    print(len(fixed), "fixed words:", sorted(fixed), "; words with bias > 0:", sum(1 for b in bias.values() if b))


main()
