"""Sentence-level comparison of polyg2p and espeak-ng on a sentence corpus.

Runs both on whole sentences (so context rules, elision and assimilation count),
aligns them word by word and lists the most frequent disagreements, each marked
with what the WikiPron gold says about that word (ours ok / espeak ok / both / none).

usage: sentence_cmp.py <lang> <sentences.tsv|.txt> [N] [--data DIR]
  Tatoeba per-language exports work directly:
  https://downloads.tatoeba.org/exports/per_language/<iso3>/<iso3>_sentences.tsv.bz2
  (a .tsv takes the third column; a .txt takes one sentence per line)
  SHOW=<n> controls how many disagreements are listed (default 80).
"""
import sys, os, re, random, subprocess, collections, unicodedata, json
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import eval_acc as e
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator

lang, path = sys.argv[1], sys.argv[2]
n = int(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3].isdigit() else 10**9
data = sys.argv[sys.argv.index("--data") + 1] if "--data" in sys.argv else None
random.seed(7)
lines = [(l.split("\t", 2)[2] if path.endswith(".tsv") else l).strip() for l in open(path, encoding="utf8")]
lines = [l for l in lines if 10 <= len(l) <= 160]
random.shuffle(lines)
sents = lines[:n]
# polyg2p on whole sentences
cmd = [e.CLI, lang] + (["--data", data] if data else [])
import time
t = time.time()
po = subprocess.run(cmd, input="\n".join(sents) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")[:n]
tp = time.time() - t
b = EspeakBackend(e.ESPEAK[lang], with_stress=True, language_switch="remove-flags", preserve_punctuation=False)
t = time.time()
pe = b.phonemize(sents, separator=Separator(phone="", word=" "), strip=True, njobs=4)
te = time.time() - t
strip = lambda s: re.sub(r"[.,!?;:…«»\"“”„()\-–—]", " ", s)
toks = lambda s: [x for x in strip(s).split() if x]
gold = e.load_gold(lang) if lang in e.GOLD else {}
e.ESPEAK.setdefault(lang, lang)
pairs = collections.Counter(); ex = {}
aligned = total = same = 0
for s, a, bb in zip(sents, po, pe):
    ws, ta, tb = toks(s), toks(a), toks(bb)
    total += 1
    if not (len(ws) == len(ta) == len(tb)):
        continue
    aligned += 1
    if all(e.collapse(x, lang) == e.collapse(y, lang) for x, y in zip(ta, tb)):
        same += 1
    for w, x, y in zip(ws, ta, tb):
        if e.collapse(x, lang) != e.collapse(y, lang):
            k = (w.lower(), x, y)
            pairs[k] += 1
            ex.setdefault(k, s)
print(f"[{lang}] {total} sentences; aligned {aligned}; identical (collapsed) {same/aligned:.1%}; polyg2p {tp*1000/total:.3f} ms/sent, espeak {te*1000/total:.3f} ms/sent")
tot_tok = sum(len(toks(s)) for s in sents)
print(f"  word-level disagreements: {sum(pairs.values())} of {tot_tok} tokens ({sum(pairs.values())/tot_tok:.2%}); distinct {len(pairs)}")
def gstat(w, x, y):
    cands = {e.collapse(g, lang) for g in set(gold.get(w, set())) | set(gold.get(w.capitalize(), set()))}
    if not cands: return "   -   "
    ox, oy = e.collapse(x, lang) in cands, e.collapse(y, lang) in cands
    return "ours✓ " if ox and not oy else "esp✓  " if oy and not ox else "both✓ " if ox else "none  "
sc = collections.Counter()
for (w, x, y), c in pairs.items():
    sc[gstat(w, x, y).strip()] += c
print("  gold adjudication (token-weighted):", dict(sc))
for (w, x, y), c in pairs.most_common(int(os.environ.get("SHOW", "80"))):
    print(f"  {c:5d} {gstat(w,x,y)} {w:<18} ours={x:<22} espeak={y:<22} | {ex[(w,x,y)][:70]}")
