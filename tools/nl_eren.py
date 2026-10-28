"""Derive data/nl_eren.tsv: for lower-case words ending in -eren/-ere/-erend/-ering, how often the
e before r is a schwa (ver-ANDER-en) vs a stressed eː (stud-EREN), keyed by the two letters before it.

    POLYG2P_SRC=sources python tools/nl_eren.py > data/nl_eren.tsv

Only aggregate counts are stored (no pronunciations), so the table is not a derivative lexicon.
"""
import collections, os, re, sys

SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
cnt = collections.defaultdict(lambda: [0, 0])
for line in open(os.path.join(SRC, "nld_latn_broad.tsv"), encoding="utf8"):
    w, p = line.rstrip("\n").split("\t")
    if not w.isalpha() or not w.islower():
        continue
    m = re.search(r"([a-z]{2})er(en|e|end|ende|ing|ingen)$", w)
    if not m or len(re.findall(r"[aeiouy]+", w)) < 3:
        continue
    key = m.group(1)
    if key[-1] in "aeiouy":
        continue
    p = p.replace(" ", "")
    # phones of the e before the suffix r
    tail = re.search(r"(ə|eː|ɛ)r(ə|ɪ)", p)
    if not tail:
        continue
    cnt[key][0 if tail.group(1) == "ə" else 1] += 1
print("# key\tschwa\tstressed  (letters before -eren: count of ə vs eː)")
for k in sorted(cnt):
    s, e = cnt[k]
    if s + e >= 2:
        print(f"{k}\t{s}\t{e}")
