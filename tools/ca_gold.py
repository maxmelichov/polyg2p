"""Catalan: build a Central-Catalan gold file from WikiPron's cat_latn_narrow.tsv.

The shipped `sources/cat_latn_broad.tsv` has ~100 words; WikiPron's narrow
file has ~99k entries but lists several dialects per word (Central, Valencian,
Balearic, ...) without labels. Heuristics to keep the Central candidates:
  - drop candidates with Valencian/Balearic markers (v, ɫ, s̺) if others remain
  - if the word has an unstressed a/e letter, Central has ə; Balearic has MORE
    ə (stressed ə), so keep the candidates with the fewest ə among those with ə
  - if the word has no unstressed a/e, keep candidates without ə
  - if the word has an unstressed o, prefer candidates with the fewest o/ɔ

usage: ca_gold.py [--fetch] narrow.tsv out_dir
  writes out_dir/cat_latn_broad.tsv (the name eval_acc.py expects), so
  `POLYG2P_SRC=out_dir tools/eval_acc.py ca` evaluates against it.
"""
import sys, os, collections, unicodedata, re, urllib.request

V = "aeiouàèéíòóúïü"


def nuclei(w):
    """(index, accented?) of the nucleus letters, mirroring src/ca.rs"""
    c = list(w); n = len(c); res = []; i = 0; prev_nuc = False
    while i < n:
        ch = c[i]
        if ch not in V:
            i += 1; prev_nuc = False; continue
        prev = c[i - 1] if i > 0 else "#"
        nxt = c[i + 1] if i + 1 < n else "#"
        if ch in "iu" and nxt in V and nxt not in "ïü":
            if ch == "u" and prev in "gq" and not prev_nuc:
                i += 1; prev_nuc = False; continue
            if prev == "#" or prev_nuc:
                i += 1; prev_nuc = False; continue
        elif ch in "iu" and prev_nuc:
            i += 1; prev_nuc = False; continue
        res.append((i, ch in "àèéíòóú")); i += 1; prev_nuc = True
    return res


def stress_index(w):
    nu = nuclei(w)
    if not nu:
        return None
    for i, acc in nu:
        if acc:
            return i
    if len(nu) == 1:
        return nu[0][0]
    last = w[-1]
    if last in "aeiou" or (last == "s" and w[-2] in "aeiou") or w.endswith(("en", "in")):
        return nu[-2][0]
    return nu[-1][0]


def pick(w, ps):
    lw = w.lower()
    clean = [p for p in ps if not any(m in p for m in ("v", "ɫ", "̺"))]
    if clean:
        ps = clean
    si = stress_index(lw)
    nu = nuclei(lw)
    # monosyllables: clitic (unstressed) and stressed readings are both Central
    if si is not None and len(nu) > 1:
        unstr = [i for i, _ in nu if i != si]
        has_ae = any(lw[i] in "ae" for i in unstr)
        has_o = any(lw[i] == "o" for i in unstr)
        if has_ae:
            with_schwa = [p for p in ps if "ə" in p]
            if with_schwa:
                ps = with_schwa
                m = min(p.count("ə") for p in ps)
                ps = [p for p in ps if p.count("ə") == m]
        else:
            no_schwa = [p for p in ps if "ə" not in p]
            if no_schwa:
                ps = no_schwa
        if has_o:
            m = min(len(re.findall("[oɔ]", p)) for p in ps)
            ps = [p for p in ps if len(re.findall("[oɔ]", p)) == m]
    return sorted(set(ps))


def load(path):
    by = collections.OrderedDict()
    for line in open(path, encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        w = unicodedata.normalize("NFC", w)
        by.setdefault(w, []).append(p.replace(" ", "").replace("‿", ""))
    return by


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    inp, out_dir = args[0], args[1]
    if "--fetch" in sys.argv:
        url = "https://raw.githubusercontent.com/CUNY-CL/wikipron/master/data/scrape/tsv/cat_latn_narrow.tsv"
        os.makedirs(os.path.dirname(os.path.abspath(inp)), exist_ok=True)
        urllib.request.urlretrieve(url, inp)
    os.makedirs(out_dir, exist_ok=True)
    by = load(inp)
    n = 0
    with open(os.path.join(out_dir, "cat_latn_broad.tsv"), "w", encoding="utf8") as f:
        for w, ps in by.items():
            for p in pick(w, ps):
                f.write(f"{w}\t{p}\n")
                n += 1
    print(len(by), "words", n, "entries")


if __name__ == "__main__":
    main()
