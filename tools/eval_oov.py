"""OOV generalization: model vs espeak on words held out from model training.
Gold = the held-out word's CMUdict pronunciation(s) (espeak notation), compared after
the same collapse as eval_acc.py (English rules), strict and lenient."""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import random, subprocess, sys
sys.argv += [] 
import eval_acc as E
from train_g2p import load, split
model = sys.argv[1] if len(sys.argv) > 1 else "models/en/g2p.bin"
rows = load("en_train.tsv")
_, test = split(rows, 0.05)
gold = {}
for w, p in test:
    gold.setdefault(w, set()).add(p)
words = sorted(gold)
random.seed(0)
words = random.sample(words, min(3000, len(words)))
ours = subprocess.run([os.path.join(ROOT, "target", "release", "nn_check"), model], input="\n".join(words) + "\n",
                      capture_output=True, text=True).stdout.split("\n")
esp = E.espeak(words, "en")
for lenient in (False, True):
    E.LENIENT = lenient
    def acc(preds):
        ok = per_e = per_n = 0
        for w, p in zip(words, preds):
            c = {E.collapse(x, "en") for x in gold[w]}
            cp = E.collapse(p, "en")
            if cp in c:
                ok += 1
            else:
                per_e += min(E.edit(cp, x) for x in c)
            per_n += min(len(x) for x in c)
        return ok / len(words), per_e / per_n
    a, pe = acc(ours); b, pb = acc(esp)
    print(f"{'lenient' if lenient else 'strict '}: model word acc {a:.1%} PER {pe:.1%} | espeak word acc {b:.1%} PER {pb:.1%}  (n={len(words)} held-out words)")
