"""Error analysis for Polish: unweighted accuracy and the mismatches of ours / espeak vs gold.

usage: pl_err.py [N] [ours|espeak]
"""
import collections, sys
import eval_acc as E
from wordfreq import top_n_list

n = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
which = sys.argv[2] if len(sys.argv) > 2 else "ours"
gold = E.load_gold("pl")
words = [w for w in top_n_list("pl", n * 3) if w in gold][:n]
preds = E.ours(words, "pl", None) if which == "ours" else E.espeak(words, "pl")
ok = 0
diffs = collections.Counter()
ex = collections.defaultdict(list)
import difflib
for w, p in zip(words, preds):
    cands = {E.collapse(x, "pl") for x in gold[w]}
    cp = E.collapse(p, "pl")
    if cp in cands:
        ok += 1
        continue
    best = min(cands, key=lambda c: E.edit(cp, c))
    for tag, a1, a2, b1, b2 in difflib.SequenceMatcher(a=cp, b=best, autojunk=False).get_opcodes():
        if tag != "equal":
            k = f"{cp[a1:a2]}>{best[b1:b2]}"
            diffs[k] += 1
            if len(ex[k]) < 4:
                ex[k].append(f"{w}:{cp}|{best}")
print(f"{which}: unweighted word acc {ok}/{len(words)} = {ok/len(words):.1%}")
for k, v in diffs.most_common(60):
    print(v, k, ex[k])
