"""Hungarian eval against the narrow WikiPron gold (hun_latn_narrow.tsv): weighted and
unweighted accuracy of ours vs espeak, plus the most frequent mismatches.

usage: hu_err.py [N] [ours|espeak] [--show K]
"""
import collections, difflib, sys
import eval_acc as E
from wordfreq import top_n_list, word_frequency

E.GOLD["hu"] = "hun_latn_narrow.tsv"
args = [a for a in sys.argv[1:] if not a.startswith("--")]
n = int(args[0]) if args else 20000
which = args[1] if len(args) > 1 else "ours"
show = int(sys.argv[sys.argv.index("--show") + 1]) if "--show" in sys.argv else 0
gold = E.load_gold("hu")
words = [w for w in top_n_list("hu", n * 3) if w in gold][:n]
preds = E.ours(words, "hu", None) if which == "ours" else E.espeak(words, "hu")
weights = {w: word_frequency(w, "hu") for w in words}
acc, per = E.score(words, preds, gold, "hu", weights)
ok = 0
diffs = collections.Counter()
ex = collections.defaultdict(list)
shown = 0
for w, p in zip(words, preds):
    cands = {E.collapse(x, "hu") for x in gold[w]}
    cp = E.collapse(p, "hu")
    if cp in cands:
        ok += 1
        continue
    best = min(cands, key=lambda c: E.edit(cp, c))
    if shown < show:
        print("   ", w, cp, sorted(cands)[:3])
        shown += 1
    for tag, a1, a2, b1, b2 in difflib.SequenceMatcher(a=cp, b=best, autojunk=False).get_opcodes():
        if tag != "equal":
            k = f"{cp[a1:a2]}>{best[b1:b2]}"
            diffs[k] += 1
            if len(ex[k]) < 4:
                ex[k].append(f"{w}:{cp}|{best}")
print(f"{which}: top {len(words)} words: weighted acc {acc:.1%}  PER {per:.1%}  unweighted {ok}/{len(words)} = {ok/len(words):.1%}")
for k, v in diffs.most_common(50):
    print(v, k, ex[k])
