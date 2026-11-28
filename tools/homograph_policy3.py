"""Round-3 policy: four global numbers instead of 162 per-word biases (plan 4.5), fitted on out-of-fold logits.

  b_w = min(3, lam * ln(p_w / (1 - p_w)))  added to word w's majority row (p_w = gold-train majority share, clipped to [.5, .99]);
  three prior bands (p >= .97 / .85-.97 / < .85) each with one margin threshold t_band in {0,.5,1,1.5}: below it the answer is the
  majority reading (the rules fallback of the plan is not used: no rules predictions on the gold train split exist; stated in the report).
  Setting = (lam in {0,.25,.5,.75,1}, t_hi, t_mid, t_lo): 320 settings. Choice = fewest expected words below majority in 10-sentence draws
  among settings whose OOF micro is within 0.2 point of the best setting; ties -> smaller values. The honest A2 pre-check: 20 random halves
  per word, setting chosen on one half, 10-sentence draws on the other (10 draws x 20 splits, repeated 20x for a mean), as in round 2.
usage: homograph_policy3.py OOF_DIR OUT.json [--dev-npz X/eval.npz]  (dev_hard logits of a dev-mode model to report with / without the policy)
"""
import collections, itertools, json, math, os, random, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
LAMS = [0, .25, .5, .75, 1.0]; THR = [0, .5, 1.0, 1.5]
SETTINGS = [(l, a, b, c) for l in LAMS for a in THR for b in THR for c in THR]


def band(p):
    return 0 if p >= .97 else 1 if p >= .85 else 2


def prep(table, lg, rows, idx_by_word):
    """per word: logits restricted to its rows, labels as local index, majority local index"""
    W = {}
    for w, ix in idx_by_word.items():
        rs = sorted({i for i, r in enumerate(table) if table[i][0] == w})
        L = lg[np.ix_(ix, rs)]
        y = np.array([rs.index(rows[i]) for i in ix])
        W[w] = (L, y, collections.Counter(y).most_common(1)[0][0], len(rs), rs)
    return W


def predict(L, m, p, setting):
    lam, t = setting[0], setting[1:]
    L = L.copy()
    pc = min(max(p, .5), .99)
    L[:, m] += min(3.0, lam * math.log(pc / (1 - pc)))
    s = np.sort(L, 1)
    marg = s[:, -1] - s[:, -2] if L.shape[1] > 1 else np.full(len(L), 9.0)
    pred = L.argmax(1)
    return np.where(marg < t[band(p)], m, pred)


def evaluate(W, share, setting, sub=None):
    """-> (micro, macro, per-word correctness arrays)"""
    ok = {}
    for w, (L, y, m, k, rs) in W.items():
        ix = np.arange(len(y)) if sub is None else sub[w]
        if len(ix) == 0:
            continue
        pr = predict(L[ix], m, share[w], setting)
        ok[w] = pr == y[ix]
    n = sum(len(v) for v in ok.values())
    return sum(int(v.sum()) for v in ok.values()) / n, float(np.mean([v.mean() for v in ok.values()])), ok


def expected_below(W, ok, sub, rng, draws=100):
    tot = 0.0
    for w, v in ok.items():
        L, y, m, k, rs = W[w]
        ix = sub[w] if sub is not None else np.arange(len(y))
        if len(ix) < 10:
            continue
        isj = (y[ix] == m)
        b = 0
        for _ in range(draws):
            d = rng.choice(len(ix), 10, replace=False)
            b += v[d].sum() < isj[d].sum()
        tot += b / draws
    return tot


def choose(W, share, sub, rng):
    res = []
    for s in SETTINGS:
        mi, ma, ok = evaluate(W, share, s, sub)
        res.append((s, mi, ma, ok))
    best = max(r[1] for r in res)
    cand = [r for r in res if r[1] >= best - 0.002]
    scored = [(expected_below(W, r[3], sub, rng), sum(r[0]), r) for r in cand]
    scored.sort(key=lambda x: (round(x[0], 3), x[1]))
    return scored[0][2][0], scored[0][0], res


def main():
    oof_dir, out = sys.argv[1:3]
    table = [l.rstrip("\n").split("\t") for l in open(os.path.join(HERE, "..", "data", "en_homograph.tsv"), encoding="utf8") if not l.startswith("#") and l.strip()]
    lg = np.load(os.path.join(oof_dir, "oof_logits.npy")); rows = np.array(json.load(open(os.path.join(oof_dir, "oof_rows.json")))["row"])
    by = collections.defaultdict(list)
    for i, r in enumerate(rows):
        by[table[r][0]].append(i)
    W = prep(table, lg, rows, by)
    share = {w: collections.Counter(W[w][1])[W[w][2]] / len(W[w][1]) for w in W}
    rng = np.random.default_rng(0)
    zero = (0, 0, 0, 0)
    mi, ma, ok = evaluate(W, share, zero)
    print("OOF no policy: micro %.4f macro %.4f; expected words below majority %.2f" % (mi, ma, expected_below(W, ok, None, rng)))
    best, eb, res = choose(W, share, None, rng)
    mi, ma, ok = evaluate(W, share, best)
    print("chosen setting (lam, t_hi, t_mid, t_lo) =", best, "OOF micro %.4f macro %.4f expected below %.2f (in-sample)" % (mi, ma, eb))
    # honest pre-check: choose on one random half, test on the other
    rnd = random.Random(0); below = []; accs = []
    for rep in range(20):
        fit, test = {}, {}
        for w in W:
            ix = list(range(len(W[w][1]))); rnd.shuffle(ix); h = len(ix) // 2
            fit[w], test[w] = np.array(ix[:h]), np.array(ix[h:])
        share_f = {w: collections.Counter(W[w][1][fit[w]]).get(W[w][2], 0) / max(1, len(fit[w])) for w in W}
        s_, _, _ = choose(W, share_f, fit, np.random.default_rng(rep))
        mi_t, ma_t, ok_t = evaluate(W, share_f, s_, test)
        accs.append((mi_t, ma_t))
        for _ in range(10):
            nb = 0
            for w, v in ok_t.items():
                if len(test[w]) < 10: continue
                d = rnd.sample(range(len(test[w])), 10)
                isj = (W[w][1][test[w]] == W[w][2])
                nb += int(v[d].sum() < isj[d].sum())
            below.append(nb)
        print("split", rep, s_, "half micro %.4f A2-draw mean so far %.2f" % (mi_t, np.mean(below)), flush=True)
    print("held-out-half micro %.4f macro %.4f; A2 pre-check mean %.2f p90 %.1f (n %d)" % (np.mean([a[0] for a in accs]), np.mean([a[1] for a in accs]), np.mean(below), np.percentile(below, 90), len(below)))
    json.dump({"lam": best[0], "thresholds": {"hi": best[1], "mid": best[2], "lo": best[3]}, "bands": {"hi": ">=0.97", "mid": "0.85-0.97", "lo": "<0.85"},
               "majority": {w: table[W[w][4][W[w][2]]][1] for w in W}, "share": {w: round(float(share[w]), 4) for w in W}, "oof_micro": mi, "oof_macro": ma,
               "precheck_mean": float(np.mean(below)), "precheck_p90": float(np.percentile(below, 90))}, open(out, "w"), indent=1)


main()
