"""Ending-based open/closed statistics for stressed e/o in Brazilian Portuguese.

Reads the WikiPron gold file (por_latn_bz_broad.tsv) and writes data/pt_open.tsv:
for every word ending (from the stressed vowel letter to the end of the word,
optionally with one or two letters of left context) the frequency-weighted share
of words (wordfreq zipf) that have an open vowel (ɛ/ɔ) and the number of words. Only counts are stored, no
pronunciations, so the table is a derived statistic, not a copy of the lexicon.

usage: pt_open.py [--src DIR] [--out data/pt_open.tsv] [--exclude WORDFILE] [--min N]
"""
import argparse, collections, os, unicodedata

try:
    from wordfreq import zipf_frequency
except ImportError:  # the table can be rebuilt without wordfreq, unweighted
    def zipf_frequency(w, lang):
        return 0.0

ACUTE = "áéíóúâêô"
TILDE = "ãõ"
VOW = "aeiouáéíóúâêôãõàü"


def base(c):
    return {"á": "a", "â": "a", "ã": "a", "à": "a", "é": "e", "ê": "e", "í": "i", "ó": "o", "ô": "o", "õ": "o", "ú": "u", "ü": "u"}.get(c, c)


def nuclei(w):
    """Orthographic nuclei: list of (start, letters). Mirrors the Rust rules."""
    n = len(w)
    out = []
    i = 0
    while i < n:
        c = w[i]
        if c not in VOW:
            i += 1
            continue
        # u after q/g before a vowel is a glide (or silent)
        if c in "uü" and i > 0 and w[i - 1] in "qg" and i + 1 < n and w[i + 1] in VOW:
            i += 1
            continue
        j = i + 1
        letters = c
        if j < n:
            d = w[j]
            e = w[j + 1] if j + 1 < n else "#"
            # falling diphthong V + i/u (unaccented)
            if d in "iu" and base(c) != d:
                # the i/u is itself the start of a final diphthong iu/ui: a-iu, u-iu
                if e in "iu" and e != d and (j + 2 == n or w[j + 2] not in VOW):
                    pass
                elif e in "rzl" and j + 2 == n:
                    pass  # sair, raiz, raul
                elif e in "mn" and (j + 2 == n or w[j + 2] == "s" and j + 3 == n or w[j + 2] not in VOW):
                    pass  # ruim, ainda, rainha
                elif e == "n" and w[j + 2 : j + 3] == "h":
                    pass
                elif c == "e" and i > 0 and w[i - 1] == "r" and d == "u" and (i == 1) and e in "nt":
                    pass  # reunir, reutilizar
                else:
                    letters += d
                    j += 1
            elif c in TILDE and d in "eo" and (c, d) in (("ã", "e"), ("ã", "o"), ("õ", "e")):
                letters += d
                j += 1
        out.append((i, letters))
        i = j
    return out


def stress(w, nuc):
    if not nuc:
        return None
    for k, (p, l) in enumerate(nuc):
        if any(c in ACUTE for c in l):
            return k
    for k, (p, l) in enumerate(nuc):
        if any(c in TILDE for c in l) and not w[p + len(l) :].startswith("zinh"):
            return k
    if len(nuc) == 1:
        return 0
    tail = w[nuc[-1][0] + len(nuc[-1][1]) :]
    last = nuc[-1][1]
    if tail == "" and last[-1] in "aeo":
        return len(nuc) - 2
    if tail == "s" and last[-1] in "aeo":
        return len(nuc) - 2
    if tail in ("m", "ns") and last in ("a", "e"):
        return len(nuc) - 2
    return len(nuc) - 1


GOLD_V = set("aeiouɐɛɔɪʊæ")


def gold_vowels(p):
    """Vowel phones of a gold transcription (NFD stripped), glides excluded."""
    toks = p.split()
    out = []
    for t in toks:
        b = unicodedata.normalize("NFD", t)
        if "̃" in b and b[0] in "jw":
            continue  # nasal glide
        c = b[0]
        if c in GOLD_V:
            out.append(c)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default=os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(__file__), "..", "sources")))
    ap.add_argument("--out", default=os.path.join(os.path.dirname(__file__), "..", "data", "pt_open.tsv"))
    ap.add_argument("--exclude", default=None, help="file of words to leave out (held-out evaluation)")
    ap.add_argument("--min", type=int, default=2, help="minimum number of distinct words behind a key")
    a = ap.parse_args()
    excl = set()
    if a.exclude:
        excl = {l.strip().lower() for l in open(a.exclude, encoding="utf8")}
    stats = collections.defaultdict(collections.Counter)
    per_word = collections.defaultdict(collections.Counter)
    for line in open(os.path.join(a.src, "por_latn_bz_broad.tsv"), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        w = unicodedata.normalize("NFC", w).lower()
        if not w.isalpha() or w in excl or any(c not in "abcdefghijklmnopqrstuvwxyzçáéíóúâêôãõàü" for c in w):
            continue
        nuc = nuclei(w)
        st = stress(w, nuc)
        if st is None:
            continue
        pos, letters = nuc[st]
        if letters[0] not in "eo":
            continue  # accented vowels are decided by the accent; only plain e/o here
        gv = gold_vowels(p)
        if len(gv) != len(nuc):
            continue
        ph = gv[st]
        if ph in "eɛ" and letters[0] == "e":
            per_word[w][ph == "ɛ"] += 1
        elif ph in "oɔ" and letters[0] == "o":
            per_word[w][ph == "ɔ"] += 1
    for w, c in per_word.items():
        nuc = nuclei(w)
        st = stress(w, nuc)
        pos = nuc[st][0]
        tail = w[pos:]
        prev = w[pos - 1] if pos > 0 else "#"
        prev2 = w[max(pos - 2, 0) : pos] if pos > 1 else "#" + prev
        # weight each word by its (zipf) frequency, split between its variants
        tot = c[True] + c[False]
        wt = 1.0 + 10 ** max(zipf_frequency(w, "pt") - 3.0, 0.0)
        o, cl = wt * c[True] / tot, wt * c[False] / tot
        keys = [tail[0] + "*"]
        for k in range(2, len(tail) + 1):
            key = tail[:k] if k < len(tail) else tail + "#"
            keys += [key, prev + "|" + key, prev2 + "|" + key]
        if len(tail) == 1:
            keys += [tail + "#", prev + "|" + tail + "#", prev2 + "|" + tail + "#"]
        for key in keys:
            stats[key]["o"] += o
            stats[key]["c"] += cl
            stats[key]["n"] += 1
    with open(a.out, "w", encoding="utf8") as f:
        for key, c in sorted(stats.items()):
            tot = c["o"] + c["c"]
            if c["n"] >= a.min:
                f.write(f"{key}\t{c['o'] / tot:.2f}\t{c['n']}\n")


if __name__ == "__main__":
    main()
