"""Audit the reading table against CMUdict: every row should match a listed CMU variant (broad phones, stress ignored).
usage: homograph_cmu_audit.py CMUDICT OUT.tsv"""
import os, re, sys, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import en_homograph_bench as bench

ARPA = {"AA": "ɑ", "AE": "æ", "AH": "ə", "AO": "ɑ", "AW": "aʊ", "AY": "aɪ", "B": "b", "CH": "tʃ", "D": "d", "DH": "ð", "EH": "ɛ",
        "ER": "ɚ", "EY": "eɪ", "F": "f", "G": "g", "HH": "h", "IH": "ɪ", "IY": "i", "JH": "dʒ", "K": "k", "L": "l", "M": "m", "N": "n",
        "NG": "ŋ", "OW": "oʊ", "OY": "ɔɪ", "P": "p", "R": "ɹ", "S": "s", "SH": "ʃ", "T": "t", "TH": "θ", "UH": "ʊ", "UW": "u", "V": "v",
        "W": "w", "Y": "j", "Z": "z", "ZH": "ʒ"}


def conv(ph):
    return "".join(ARPA[re.sub(r"\d", "", p)] for p in ph.split())


def norm(p):
    p = bench.collapse(p)[0]
    return p.replace("ʌ", "ə").replace("ɪ", "ə").replace("ɚ", "ɚ")


def main():
    cmu = collections.defaultdict(list)
    for l in open(sys.argv[1], encoding="utf8"):
        w, _, ph = l.partition(" ")
        w = re.sub(r"\(\d+\)$", "", w)
        cmu[w].append(norm(conv(ph.split("#")[0].strip())))
    n = bad = 0
    with open(sys.argv[2], "w", encoding="utf8") as f:
        for w, rows in bench.read_table().items():
            for r, ipa in rows:
                n += 1
                if w not in cmu:
                    f.write(f"{w}\t{r}\t{ipa}\tNOT IN CMUDICT\n")
                elif norm(ipa) not in cmu[w]:
                    bad += 1
                    f.write(f"{w}\t{r}\t{ipa}\tno match; cmu: {' | '.join(sorted(set(cmu[w])))}\n")
    print(n, "rows,", bad, "mismatches")


main()
