"""Leak check for the English heteronym model's training inputs.

A training / seed / template / pool sentence is an offender when, lower-cased, it equals a held-out sentence (corpus eval
split, tests/en_homograph_cases*.tsv, tests/en_homograph_dev.tsv) or shares with one a 4-word window that contains a table
word. Prints counts only; with --filter it writes the kept lines to OUT (one record per line, jsonl / tsv / plain text).

usage: homograph_leakcheck.py CORPUS/data IN [--filter OUT] [--kind jsonl|tsv|txt]
"""
import csv, glob, json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
WORD = re.compile(r"[a-z']+")


def table_words():
    return {l.split("\t")[0] for l in open(os.path.join(HERE, "..", "data", "en_homograph.tsv"), encoding="utf8") if not l.startswith("#") and l.strip()}


def grams(sentence, tw):
    t = WORD.findall(sentence.lower())
    return {tuple(t[i:i + 4]) for i in range(len(t) - 3) if any(x in tw for x in t[i:i + 4])}


def held_out(corpus):
    sents = []
    for f in glob.glob(os.path.join(corpus, "eval", "*.tsv")):
        for r in list(csv.reader(open(f, encoding="utf8", newline=""), delimiter="\t", quotechar='"'))[1:]:
            sents.append(r[2])
    for p in ("en_homograph_cases.tsv", "en_homograph_cases2.tsv", "en_homograph_dev.tsv", "en_homograph_dev2.tsv"):
        for l in open(os.path.join(HERE, "..", "tests", p), encoding="utf8"):
            if not l.startswith("#") and l.strip():
                sents.append(l.split("\t")[0])
    return sents


def load_forbidden(corpus):
    tw = table_words()
    full, g = set(), set()
    for s in held_out(corpus):
        full.add(" ".join(WORD.findall(s.lower())))
        g |= grams(s, tw)
    return tw, full, g


def sentence_of(line, kind):
    if kind == "jsonl":
        return json.loads(line)["s"]
    if kind == "tsv":
        return line.split("\t")[0]
    return line


def leaks(s, tw, full, g):
    return " ".join(WORD.findall(s.lower())) in full or bool(grams(s, tw) & g)


def main():
    corpus, inp = sys.argv[1:3]
    out = sys.argv[sys.argv.index("--filter") + 1] if "--filter" in sys.argv else None
    kind = sys.argv[sys.argv.index("--kind") + 1] if "--kind" in sys.argv else ("jsonl" if inp.endswith(".jsonl") else "tsv" if inp.endswith(".tsv") else "txt")
    tw, full, g = load_forbidden(corpus)
    n = bad = 0
    fo = open(out, "w", encoding="utf8") if out else None
    for line in open(inp, encoding="utf8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        n += 1
        if leaks(sentence_of(line, kind), tw, full, g):
            bad += 1
        elif fo:
            fo.write(line + "\n")
    print(f"{inp}: {n} sentences, {bad} offenders" + (f", {n - bad} kept -> {out}" if out else ""))


if __name__ == "__main__":
    main()
