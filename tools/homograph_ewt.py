"""Build a selection set from UD English-EWT (licence class SA: selection only, never training).

Keeps occurrences of table words whose reading follows from the human tags (UPOS, XPOS, Tense, VerbForm) through the
`class` column of data/en_homograph.tsv; a word whose readings share the class of the occurrence is skipped, and so is
a sentence in which the word occurs twice (the bench scores the first occurrence).

usage: homograph_ewt.py DIR_WITH_CONLLU OUT.tsv
"""
import collections, glob, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))


def table():
    t = collections.OrderedDict()
    for l in open(os.path.join(HERE, "..", "data", "en_homograph.tsv"), encoding="utf8"):
        if l.startswith("#") or not l.strip():
            continue
        w, r, c, _ = l.rstrip("\n").split("\t")
        t.setdefault(w, []).append((r, set(c.split(","))))
    return t


def classes(upos, xpos, feats):
    """Table classes the tags allow."""
    if upos == "NOUN":
        return {"N"}
    if upos == "PROPN":
        return {"NAME"}
    if upos == "ADJ":
        return {"A"}
    if upos in ("VERB", "AUX"):
        if "VerbForm=Ger" in feats and xpos == "VBG":
            return {"N"}          # gerund: "reading" the activity
        if xpos in ("VBD", "VBN"):
            return {"PAST"}
        return {"V"}
    return set()


def main():
    t = table()
    out, seen = [], set()
    for path in sorted(glob.glob(os.path.join(sys.argv[1], "*.conllu"))):
        sent, toks = None, []
        for line in open(path, encoding="utf8"):
            line = line.rstrip("\n")
            if line.startswith("# text = "):
                sent, toks = line[9:], []
            elif line and not line.startswith("#"):
                f = line.split("\t")
                if "-" in f[0] or "." in f[0]:
                    continue
                toks.append(f)
            elif not line and sent is not None:
                words = re.findall(r"[A-Za-z']+", sent.lower())
                if len(words) >= 3 and sent not in seen:
                    for f in toks:
                        w = f[1].lower()
                        if w not in t or words.count(w) != 1 or f[1] != f[1] and False:
                            continue
                        cls = classes(f[3], f[4], f[5])
                        if not cls:
                            continue
                        # the written form must be the table word itself (not a longer token)
                        rows = [r for r, c in t[w] if c & cls]
                        if len(rows) == 1 and re.search(r"(?<![\w'-])" + re.escape(w) + r"(?![\w'-])", sent, re.I):
                            out.append((sent, w, rows[0]))
                            seen.add(sent)
                            break
                sent, toks = None, []
    with open(sys.argv[2], "w", encoding="utf8") as fo:
        fo.write("# UD English-EWT occurrences labelled through the human tags and the table's class column (selection only)\n")
        for s, w, r in out:
            fo.write(f"{s}\t{w}\t{r}\n")
    print(len(out), "sentences", len({w for _, w, _ in out}), "words", collections.Counter(w for _, w, _ in out).most_common(12))


main()
