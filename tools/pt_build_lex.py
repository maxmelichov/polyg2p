#!/usr/bin/env python3
"""Build the optional Brazilian Portuguese lexicon pt.lex (CC BY-SA) from WikiPron's
por_latn_bz_broad.tsv.

The rules already get the segments right for almost every word; what Wiktionary
adds is the open/closed quality of e and o (ɛ/e, ɔ/o), which no rule predicts
reliably. So the lexicon stores the rule output with the mid vowels corrected to
the gold where the two differ, and nothing for words the rules already get right.

    POLYG2P_SRC=sources python tools/pt_build_lex.py WORK/pt_lex.tsv && target/release/build_lex WORK/pt_lex.tsv data-out/pt.lex
"""
import os, re, subprocess, sys, unicodedata, collections

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
SRC = os.environ.get("POLYG2P_SRC", os.path.join(ROOT, "sources"))
sys.path.insert(0, HERE)
from eval_acc import collapse as _collapse, edit


def collapse(s):
    return _collapse(s, "pt")

VOW = "aeɛioɔuʊæɐyɪ"


def nuclei(s, glides):
    """Indices of vowel nuclei: a vowel char not glued to a preceding vowel char (diphthong)."""
    out = []
    prev_v = False
    for i, ch in enumerate(s):
        v = ch in VOW and ch not in glides
        if v and not (prev_v and ch in "ɪʊ"):  # ɪ/ʊ after a vowel is the off-glide of a diphthong
            out.append(i)
        prev_v = v
    return out


def main():
    out_path = sys.argv[1]
    gold = collections.defaultdict(list)
    for line in open(os.path.join(SRC, "por_latn_bz_broad.tsv"), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        gold[unicodedata.normalize("NFC", w)].append(p.replace(" ", ""))
    words = sorted(gold)
    exe = os.path.join(ROOT, "target", "release", "g2p")
    data = os.environ.get("POLYG2P_DATA", os.path.join(ROOT, "data-out"))
    env = dict(os.environ, POLYG2P_PT_NOLEX="1")
    ours = subprocess.run([exe, "pt", "--data", data], input="\n".join(words) + "\n", capture_output=True, text=True, check=True, env=env).stdout.split("\n")
    n_fix = n_same = n_skip = 0
    with open(out_path, "w", encoding="utf8") as f:
        for w, o in zip(words, ours):
            o = o.rstrip(".")
            co = collapse(o.replace("ˈ", "").replace("ˌ", ""))
            variants = [(edit(co, collapse(g)), g) for g in gold[w]]
            d, g = min(variants)
            if d == 0:
                n_same += 1
                continue
            # align vowel nuclei and copy the gold's e/ɛ, o/ɔ quality
            no = nuclei(o, glides="")
            ng = nuclei(unicodedata.normalize("NFD", g).replace("̃", "").replace("̯", ""), glides="jw")
            gg = unicodedata.normalize("NFD", g).replace("̃", "").replace("̯", "")
            if len(no) != len(ng):
                n_skip += 1
                continue
            chars = list(o)
            changed = False
            for i, j in zip(no, ng):
                a, b = chars[i], gg[j]
                if (a, b) in {("e", "ɛ"), ("ɛ", "e"), ("o", "ɔ"), ("ɔ", "o")}:
                    chars[i] = b
                    changed = True
            if not changed:
                n_skip += 1
                continue
            n_fix += 1
            f.write(f"{w}\t{''.join(chars)}\n")
    print(f"{n_same} words already right, {n_fix} mid-vowel corrections stored, {n_skip} other differences left to the rules", file=sys.stderr)


if __name__ == "__main__":
    main()
