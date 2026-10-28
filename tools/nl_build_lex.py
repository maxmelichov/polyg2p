"""Build the optional Dutch lexicon (nl.lex, CC BY-SA) from WikiPron's nld_latn_broad.tsv.

    POLYG2P_SRC=sources python tools/nl_build_lex.py WORK/nl_lex.tsv && target/release/build_lex WORK/nl_lex.tsv data-out/nl.lex

WikiPron has no stress marks and uses ɦ ʏ ɛi̯ œy̯ ɑu̯; the entries are rewritten to the espeak-ng
`nl` notation polyg2p's rules produce (h ɵ ɛɪ œy ʌʊ, ˈ before the stressed vowel). Where the rule
engine agrees with Wiktionary on the broad phones, its output (which carries stress) is stored; otherwise
the Wiktionary phones get the stress on the vowel group the rules stressed, or the first one.
"""
import os, re, subprocess, sys, unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
SRC = os.environ.get("POLYG2P_SRC", os.path.join(ROOT, "sources"))
sys.path.insert(0, HERE)
from collapse_nl import collapse

VOWEL = re.compile(r"(aː|eː|oː|øː|ɛɪ|œy|ʌʊ|ɛː|[aeiouyəɑɛɔɵɪœ]ː?)")


def to_espeak(p):
    p = unicodedata.normalize("NFC", p.replace(" ", ""))
    for a, b in [("ɦ", "h"), ("ʏ", "ɵ"), ("ɛi̯", "ɛɪ"), ("œy̯", "œy"), ("ɑu̯", "ʌʊ"), ("ɔu̯", "ʌʊ"), ("i̯", "i"), ("u̯", "w"), ("y̯", "y"),
                 ("iː", "i"), ("yː", "y"), ("uː", "u"), ("ɡ", "ɡ"), ("ʃ", "ʃ")]:
        p = p.replace(a, b)
    return p


def stress_index(ours):
    k = 0
    for m in re.finditer(r"ˈ|" + VOWEL.pattern, ours):
        if m.group(0) == "ˈ":
            return k
        k += 1
    return 0


def add_stress(p, k):
    parts = list(VOWEL.finditer(p))
    if not parts:
        return p
    k = min(k, len(parts) - 1)
    s = parts[k].start()
    return p[:s] + "ˈ" + p[s:]


def main():
    out = sys.argv[1]
    words, prons = [], {}
    for line in open(os.path.join(SRC, "nld_latn_broad.tsv"), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        w = unicodedata.normalize("NFC", w)
        if " " in w or not w.isalpha():
            continue
        if w not in prons:
            words.append(w)
            prons[w] = []
        prons[w].append(p)
    lw = [w.lower() for w in words]
    res = subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), "nl"], input="\n".join(lw) + "\n",
                         capture_output=True, text=True, check=True).stdout.split("\n")
    n_rules = 0
    with open(out, "w", encoding="utf8") as f:
        for w, ours in zip(words, res):
            ours = ours.rstrip(".")
            gold = [to_espeak(p) for p in prons[w]]
            cg = {collapse(g) for g in gold}
            if collapse(ours) in cg:
                f.write(f"{w.lower()}\t{ours}\n")
                n_rules += 1
            else:
                k = stress_index(ours)
                f.write(f"{w.lower()}\t" + "\t".join(dict.fromkeys(add_stress(g, k) for g in gold)) + "\n")
    print(f"{len(words)} entries, {n_rules} from the rules", file=sys.stderr)


if __name__ == "__main__":
    main()
