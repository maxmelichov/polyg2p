"""Pool extraction for the English heteronym model (round 3). One streaming pass over a plain-text source.

For every sentence of 4-40 words (lower-case dedupe) that holds a word of data/en_homograph.tsv, writes one record per
occurrence: {"s", "a", "b" (byte offsets), "word", "init" (target is the first word)}. Per word at most --cap occurrences are
kept (reservoir sample over the whole stream); every sentence-initial occurrence is kept besides (own cap --init-cap).
Sentences that overlap a held-out sentence (tools/homograph_leakcheck.py: whole sentence or a 4-word window with a table
word, held-out = corpus eval split, tests/en_homograph_cases*.tsv, dev, dev2) are dropped.
The unit finder of the engine (`en_homograph --units`) is applied later, when the pool is labelled.

usage: homograph_pool.py --kind lines|tatoeba|gutenberg|europarl --in PATH [PATH ...] --out OUT.jsonl --corpus CORPUS/data
         [--cap 20000] [--init-cap 20000] [--seed 0] [--max-bytes N]
kinds: lines = one sentence per line; tatoeba = id<TAB>lang<TAB>text (bz2 ok); gutenberg = parquet with a `text` column (books;
       header/footer between the `*** START OF` / `*** END OF` marks removed, paragraphs un-wrapped, then split);
       europarl = a tgz of one-sentence-per-line files.
"""
import argparse, bz2, collections, io, json, os, random, re, sys, tarfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import homograph_leakcheck as LK  # noqa: E402

TOK = re.compile(r"(?<![A-Za-z'])[A-Za-z]+(?![A-Za-z'])")
SPLIT = re.compile(r"(?<=[.!?])[\"')\]”’]*\s+(?=[\"'(“‘]?[A-Z])")


def lines_of(kind, paths, max_bytes):
    n = 0
    for p in paths:
        if kind == "lines":
            for l in open(p, encoding="utf8", errors="replace"):
                yield l.rstrip("\n")
        elif kind == "tatoeba":
            f = bz2.open(p, "rt", encoding="utf8") if p.endswith(".bz2") else open(p, encoding="utf8")
            for l in f:
                x = l.rstrip("\n").split("\t")
                if len(x) >= 3 and x[1] == "eng":
                    yield x[2]
        elif kind == "europarl":
            with tarfile.open(p) as t:
                for m in t:
                    if m.isfile() and m.name.endswith(".en"):
                        for l in io.TextIOWrapper(t.extractfile(m), encoding="utf8", errors="replace"):
                            yield l.rstrip("\n")
        elif kind == "gutenberg":
            import pyarrow.parquet as pq
            pf = pq.ParquetFile(p)
            for rg in range(pf.num_row_groups):
                for book in pf.read_row_group(rg, columns=["text"]).column("text").to_pylist():
                    n += len(book)
                    if max_bytes and n > max_bytes:
                        return
                    a = book.find("*** START OF")
                    if a >= 0:
                        a = book.find("\n", a) + 1
                    else:
                        continue
                    b = book.find("*** END OF", a)
                    body = book[a:b if b >= 0 else len(book)]
                    for para in re.split(r"\n\s*\n", body):
                        para = " ".join(para.split())
                        if len(para) < 30:
                            continue
                        for s in SPLIT.split(para):
                            yield s


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--kind", required=True)
    ap.add_argument("--in", dest="inp", nargs="+", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--corpus", required=True)
    ap.add_argument("--cap", type=int, default=20000)
    ap.add_argument("--init-cap", type=int, default=20000)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--max-bytes", type=int, default=0)
    a = ap.parse_args()
    rnd = random.Random(a.seed)
    tw, full, g = LK.load_forbidden(a.corpus)
    res = collections.defaultdict(list); seen_n = collections.Counter(); init = collections.defaultdict(list)
    seen = set(); nsent = ncand = nleak = 0
    for s in lines_of(a.kind, a.inp, a.max_bytes):
        s = s.strip()
        nw = s.count(" ") + 1
        if nw < 4 or nw > 40:
            continue
        toks = [(m.group().lower(), m.start(), m.end()) for m in TOK.finditer(s)]
        hits = [(w, x, y) for w, x, y in toks if w in tw]
        if not hits:
            continue
        key = s.lower()
        if key in seen:
            continue
        seen.add(key); nsent += 1
        if LK.leaks(s, tw, full, g):
            nleak += 1
            continue
        for w, x, y in hits:
            ncand += 1
            rec = {"s": s, "a": len(s[:x].encode("utf8")), "b": len(s[:y].encode("utf8")), "word": w, "init": x <= 1 and (toks[0][1] == x)}
            if rec["init"]:
                if len(init[w]) < a.init_cap:
                    init[w].append(rec)
                continue
            seen_n[w] += 1
            if len(res[w]) < a.cap:
                res[w].append(rec)
            else:
                j = rnd.randrange(seen_n[w])
                if j < a.cap:
                    res[w][j] = rec
    with open(a.out, "w", encoding="utf8") as f, open(a.out + ".counts.tsv", "w") as fc:
        fc.write("word\tseen_noninitial\tkept_noninitial\tinitial\n")
        for w in sorted(set(res) | set(init)):
            for r in res[w] + init[w]:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
            fc.write(f"{w}\t{seen_n[w]}\t{len(res[w])}\t{len(init[w])}\n")
    print(f"{a.out}: {nsent} sentences with a table word, {nleak} leak-dropped, {ncand} occurrences; words {len(set(res)|set(init))}")


if __name__ == "__main__":
    main()
