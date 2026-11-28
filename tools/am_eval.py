"""Amharic: phone error rate of polyg2p against Wiktionary (WikiPron) pronunciations.

WikiPron's amh_ethi_broad.tsv is small (478 entries) and espeak-ng has no usable
Amharic lexicon to compare with, so this reports PER and word accuracy of
polyg2p alone instead of going through eval_acc.py. The gold file is CC BY-SA
and is not part of this repository: put it in sources/ (or POLYG2P_SRC).

Fidel does not write gemination and the engine does not predict it, so the
headline number removes the length mark from the gold. The gold lists many
vowel-initial words twice, with and without an initial glottal stop; the engine
writes none, so a second line leaves ʔ unscored.

usage: am_eval.py [--show N]
"""
import argparse, collections, os, subprocess, sys, unicodedata

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
SRC = os.environ.get("POLYG2P_SRC", os.path.join(ROOT, "sources"))
CLI = os.environ.get("G2P_CLI", os.path.join(ROOT, "target", "release", "g2p"))


def phones(s):
    """IPA string -> phones: ejective, length and labialization marks stay on their consonant; affricates are one phone."""
    s = unicodedata.normalize("NFD", s).replace("͡", "").replace("ʧ", "tʃ").replace("ʤ", "dʒ").replace("ʦ", "ts").replace("ʣ", "dz")
    out = []
    for ch in s:
        if ch.isspace():
            continue
        if ch in "ʼːʷ" and out:
            out[-1] += ch
        else:
            out.append(ch)
    merged, i = [], 0
    while i < len(out):
        if i + 1 < len(out) and (out[i], out[i + 1][0]) in (("t", "ʃ"), ("t", "s"), ("d", "ʒ")):
            merged.append(out[i] + out[i + 1]); i += 2
        else:
            merged.append(out[i]); i += 1
    # notation: tap = r, script g, the gold's two spellings of the first-order vowel
    return [p.replace("ɾ", "r").replace("g", "ɡ").replace("ä", "ə").replace("ɛ", "ə") for p in merged]


def edits(a, b):
    """Levenshtein distance between two phone lists."""
    prev = list(range(len(b) + 1))
    for i, x in enumerate(a, 1):
        cur = [i]
        for j, y in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (x != y)))
        prev = cur
    return prev[-1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--show", type=int, default=0, help="print the first N words that differ")
    a = ap.parse_args()
    gold = [l.rstrip("\n").split("\t") for l in open(os.path.join(SRC, "amh_ethi_broad.tsv"), encoding="utf-8")]
    hyp = subprocess.run([CLI, "am"], input="\n".join(w for w, _ in gold) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")[:-1]
    assert len(hyp) == len(gold)
    nogem = lambda ph: [p.replace("ː", "") for p in ph]
    for name, drop in (("as written", ()), ("glottal stop unscored", ("ʔ",))):
        t = collections.Counter(); shown = []
        for (w, g), h in zip(gold, hyp):
            r = [p for p in phones(g) if p not in drop]; x = [p for p in phones(h) if p not in drop]
            t["n"] += len(r); t["full"] += edits(r, x); t["nogem"] += edits(nogem(r), nogem(x)); t["w"] += 1; t["ok"] += nogem(r) == nogem(x)
            if nogem(r) != nogem(x): shown.append((w, "".join(r), "".join(x)))
        print(f"{name}: words {t['w']}, gold phones {t['n']} | PER without gemination {100 * t['nogem'] / t['n']:.1f}% "
              f"| PER with gemination {100 * t['full'] / t['n']:.1f}% | word accuracy without gemination {100 * t['ok'] / t['w']:.1f}%")
        if drop:
            for s in shown[:a.show]: print("  ", *s)
    gem = sum("ː" in p for _, g in gold for p in phones(g)); tot = sum(len(phones(g)) for _, g in gold)
    print(f"gold phones with a length mark: {gem} of {tot} ({100 * gem / tot:.1f}%)")


if __name__ == "__main__":
    main()
