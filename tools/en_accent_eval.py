"""Score an English accent variant against an independent pronunciation list.

en-gb: WikiPron's UK list (Wiktionary, CC BY-SA; used for scoring only, not shipped). Reported for
  (a) words Britfone also has (the word list can apply) and (b) words it does not (rules only),
  next to plain `en` scored the same way, so the number shows what the variant changes.
The other accents have no open reference; they are rule-derived and covered by tests only.

usage: en_accent_eval.py WIKIPRON_UK.tsv BRITFONE.csv [--data DIR] [--lang en-gb] [--lenient]
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


def run(words, lang, data):
    if lang == "en":
        cmd = [os.path.join(ROOT, "target", "release", "en_accent"), "en-gb"] + (["--data", data] if data else [])
        out = subprocess.run(cmd, input="\n".join(words) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
        return [o.split("\t")[1] for o in out[: len(words)]]
    cmd = [os.path.join(ROOT, "target", "release", "en_accent"), lang] + (["--data", data] if data else [])
    out = subprocess.run(cmd, input="\n".join(words) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
    return [o.split("\t")[2] for o in out[: len(words)]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("gold")
    ap.add_argument("britfone")
    ap.add_argument("--data", default=os.environ.get("POLYG2P_DATA"))
    ap.add_argument("--lang", default="en-gb")
    ap.add_argument("--misses")
    a = ap.parse_args()
    gold = collections.defaultdict(set)
    for line in open(a.gold, encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        if re.fullmatch(r"[a-z']+", w):
            gold[w].add(p)
    listed = set()
    for line in open(a.britfone, encoding="utf8"):
        listed.add(re.sub(r"\(\d+\)$", "", line.split(",", 1)[0].strip()).lower())
    words = sorted(gold)
    miss = []
    for lang in ("en", a.lang):
        outs = run(words, lang, a.data)
        for lenient in (False, True):
            st = collections.Counter()
            for w, o in zip(words, outs):
                k = "listed" if w in listed else "unlisted"
                hit = collapse(o, lenient) in {collapse(g, lenient) for g in gold[w]}
                st[k, "n"] += 1
                st[k, "ok"] += hit
                if not hit and lang != "en" and not lenient:
                    miss.append((k, w, o, " | ".join(sorted(gold[w]))))
            print(f"{lang:6s} {'lenient (unstressed ɪ = ə, CURE = THOUGHT)' if lenient else 'strict':44s} " + "  ".join(
                f"{k}: {st[k, 'ok']}/{st[k, 'n']} = {100 * st[k, 'ok'] / max(1, st[k, 'n']):.1f}%" for k in ("listed", "unlisted")))
    if a.misses:
        with open(a.misses, "w", encoding="utf8") as f:
            for m in miss:
                f.write("\t".join(m) + "\n")


if __name__ == "__main__":
    main()
