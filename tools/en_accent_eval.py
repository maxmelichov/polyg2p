"""Score en-gb against a pronunciation list it was not built from.

Reference: WikiPron's UK list (Wiktionary, CC BY-SA; not shipped). It is used twice: here for scoring, and in
tools/en_gb_build.py as the check that decides which Britfone entries enter data/en_gb_lex.tsv (no Wiktionary text
is copied; an entry is kept or dropped). So only the groups outside the list are independent results. Three groups:
  list      words in data/en_gb_lex.tsv. Their entries were kept because this same reference confirms them
            (tools/en_gb_build.py), so the full variant is right on them by construction; the "rules only" row is
            the informative one there.
  britfone  other words Britfone has (the rules already agreed with Britfone, or the entry was unconfirmed)
  other     words Britfone does not have (mostly rare words and names)
Rows: plain `en` scored as British, en-gb with the word list switched off, en-gb. Each is printed strict and
lenient (lenient: unstressed ɪ = ə, CURE = THOUGHT).
The other accents have no open reference; they are rule-derived and covered by tests only.

usage: en_accent_eval.py WIKIPRON_UK.tsv BRITFONE.csv [--data DIR] [--misses FILE]
"""
import argparse, collections, os, re, subprocess, sys, unicodedata

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")


def collapse(s, lenient=False):
    s = unicodedata.normalize("NFD", s.replace(" ", ""))
    for a, b in [("t͡ʃ", "ʧ"), ("d͡ʒ", "ʤ"), ("tʃ", "ʧ"), ("dʒ", "ʤ"), ("ʦ", "ts"), ("ʣ", "dz"), ("͡", ""), ("ˈ", ""), ("ˌ", ""), (".", ""),
                 ("g", "ɡ"), ("r", "ɹ"), ("ɫ", "l"), ("ɾ", "t"), ("l̩", "əl"), ("n̩", "ən"), ("m̩", "əm"), ("̯", ""), ("̩", ""), ("̆", ""),
                 ("əw", "əʊ"), ("ɛː", "eə"), ("ɛə", "eə"), ("əː", "ɜː"), ("ɝ", "ɜː"), ("ɚ", "ə"), ("ɐ", "ə"), ("ᵻ", "ɪ")]:
        s = s.replace(a, b)
    # e / ɛ and a / æ are the same phonemes written two ways (outside the diphthongs)
    s = re.sub(r"e(?![ɪə])", "ɛ", s)
    s = re.sub(r"a(?![ɪʊ])", "æ", s)
    s = s.replace("iː", "I").replace("uː", "U").replace("i", "ɪ").replace("u", "ʊ").replace("I", "iː").replace("U", "uː")
    s = s.replace("ɜːː", "ɜː")
    if lenient:
        s = s.replace("ɪə", "N").replace("ɪ", "ə").replace("N", "ɪə").replace("ʊə", "ɔː")
    return s


def run(words, mode, data):
    """mode: en (American form), rules (en-gb without the word list), en-gb."""
    cmd = [os.path.join(ROOT, "target", "release", "en_accent"), "en-gb"] + (["--rules"] if mode == "rules" else []) + (["--data", data] if data else [])
    out = subprocess.run(cmd, input="\n".join(words) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
    return [o.split("\t")[1 if mode == "en" else 2] for o in out[: len(words)]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("gold")
    ap.add_argument("britfone")
    ap.add_argument("--data", default=os.environ.get("POLYG2P_DATA"))
    ap.add_argument("--misses")
    a = ap.parse_args()
    gold = collections.defaultdict(set)
    for line in open(a.gold, encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        if re.fullmatch(r"[a-z']+", w):
            gold[w].add(p)
    britfone = set()
    for line in open(a.britfone, encoding="utf8"):
        britfone.add(re.sub(r"\(\d+\)$", "", line.split(",", 1)[0].strip()).lower())
    in_list = {l.split("\t")[0] for l in open(os.path.join(ROOT, "data", "en_gb_lex.tsv"), encoding="utf8") if not l.startswith("#")}
    group = lambda w: "list" if w in in_list else "britfone" if w in britfone else "other"
    words = sorted(gold)
    miss = []
    for mode, name in (("en", "plain en"), ("rules", "en-gb, rules only"), ("en-gb", "en-gb")):
        outs = run(words, mode, a.data)
        for lenient in (False, True):
            st = collections.Counter()
            for w, o in zip(words, outs):
                k = group(w)
                hit = collapse(o, lenient) in {collapse(g, lenient) for g in gold[w]}
                for kk in (k, "all"):
                    st[kk, "n"] += 1
                    st[kk, "ok"] += hit
                if not hit and mode == "en-gb" and not lenient:
                    miss.append((k, w, o, " | ".join(sorted(gold[w]))))
            print(f"{name:18s} {'lenient' if lenient else 'strict ':8s} " + "  ".join(
                f"{k}: {st[k, 'ok']}/{st[k, 'n']} = {100 * st[k, 'ok'] / max(1, st[k, 'n']):.1f}%" for k in ("list", "britfone", "other", "all")))
    if a.misses:
        with open(a.misses, "w", encoding="utf8") as f:
            for m in miss:
                f.write("\t".join(m) + "\n")


if __name__ == "__main__":
    main()
