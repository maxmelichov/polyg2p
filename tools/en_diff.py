"""Where polyg2p and espeak differ from Wiktionary on frequent English words.

Writes build-data/en_diff.tsv: word, ours, espeak, gold variants, status
(status: OURS_WRONG_ESPEAK_RIGHT / BOTH_WRONG / ESPEAK_WRONG) — the first
group is where notation conventions still differ from espeak's.
"""
import os, sys, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import eval_acc as E
from wordfreq import top_n_list, word_frequency

lang = "en"
gold = E.load_gold(lang)
words = []
for w in top_n_list(lang, 60000):
    if w in gold:
        words.append(w)
    if len(words) >= 20000:
        break
po = E.ours(words, lang, "data-out")
pe = E.espeak(words, lang)
os.makedirs("build-data", exist_ok=True)
stat = collections.Counter()
with open("build-data/en_diff.tsv", "w", encoding="utf8") as f:
    for w, o, e in zip(words, po, pe):
        cands = {E.collapse(x, lang) for x in gold[w]}
        co, ce = E.collapse(o, lang), E.collapse(e, lang)
        ok_o, ok_e = co in cands, ce in cands
        if ok_o and ok_e:
            continue
        st = "ESPEAK_WRONG" if ok_o else ("OURS_WRONG_ESPEAK_RIGHT" if ok_e else "BOTH_WRONG")
        stat[st] += word_frequency(w, lang)
        f.write(f"{w}\t{o}\t{e}\t{' | '.join(sorted(gold[w]))}\t{co}\t{ce}\t{st}\n")
tot = sum(word_frequency(w, lang) for w in words)
for k, v in stat.items():
    print(f"{k}: {v/tot:.2%} of frequency mass")
