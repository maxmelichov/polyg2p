"""Rebuild polyg2p's data files from their public sources.

    python tools/build_data.py --out data-out                # en (CMUdict, BSD)
    python tools/build_data.py --out data-out --ccbysa       # + de / it lexicons (CC BY-SA sources)

Steps (run from any working directory; intermediates go to --work):
  en  CMUdict -> espeak-style IPA -> en.lex (+ en_train.tsv for train_g2p.py)
  it  WikiPron ita -> stress/openness tables (data/it_suffix.tsv, data/it_open.tsv) and it.lex
  de  ipa-dict de (Wiktionary) -> espeak-style IPA -> de.lex

The English OOV model is trained separately:
    python tools/train_g2p.py WORK/en_train.tsv models/en --epochs 20 --d 128 --ff 384 --layers 3
    cp models/en/g2p.bin data-out/en.g2p
Requires the release binaries: cargo build --release (g2p, build_lex).
"""
import argparse, os, re, subprocess, sys, unicodedata, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
SOURCES = {
    "cmudict.dict": "https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict",
    "cmudict.LICENSE": "https://raw.githubusercontent.com/cmusphinx/cmudict/master/LICENSE",
    "ita_latn_broad.tsv": "https://raw.githubusercontent.com/CUNY-CL/wikipron/master/data/scrape/tsv/ita_latn_broad.tsv",
    "ipadict_de.txt": "https://raw.githubusercontent.com/open-dict-data/ipa-dict/master/data/de.txt",
}
CCBYSA = {"ita_latn_broad.tsv", "ipadict_de.txt"}


def fetch(src, ccbysa):
    os.makedirs(src, exist_ok=True)
    for name, url in SOURCES.items():
        if name in CCBYSA and not ccbysa:
            continue
        p = os.path.join(src, name)
        if not os.path.exists(p):
            print("download", url)
            urllib.request.urlretrieve(url, p)


def build_lex(tsv, out):
    subprocess.run([os.path.join(ROOT, "target", "release", "build_lex"), tsv, out], check=True)


def edit(a, b):
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def build_en(work, out):
    from en_convert import load_cmudict, arpa_to_espeak
    lex = load_cmudict(os.path.join(os.environ["POLYG2P_SRC"], "cmudict.dict"))
    tsv = os.path.join(work, "en_lex.tsv")
    fix = {}
    for line in open(os.path.join(HERE, "en_fix.tsv"), encoding="utf8"):
        if line.strip() and not line.startswith("#"):
            w, e = line.rstrip("\n").split("\t")
            fix[w] = unicodedata.normalize("NFD", e)
    with open(tsv, "w", encoding="utf8") as f, open(os.path.join(work, "en_train.tsv"), "w", encoding="utf8") as t:
        for w, prons in lex.items():
            vs = []
            for p in prons:
                e = unicodedata.normalize("NFD", arpa_to_espeak(p, w))
                if e not in vs:
                    vs.append(e)
            if w in fix:
                vs = [fix[w]] + [v for v in vs if v != fix[w]]
            f.write(w + "\t" + "\t".join(vs) + "\n")
            if re.fullmatch(r"[a-z][a-z'\-\.]*", w) and len(w) <= 30:
                for e in vs:
                    t.write(f"{w}\t{e}\n")
    build_lex(tsv, os.path.join(out, "en.lex"))


def build_it(work, out):
    cwd = os.getcwd()
    os.chdir(work)
    try:
        subprocess.run([sys.executable, os.path.join(HERE, "it_stress.py")], check=True)
        subprocess.run([sys.executable, os.path.join(HERE, "it_stress_eval.py"), "0.3"], check=True)
        subprocess.run([sys.executable, os.path.join(HERE, "it_open.py")], check=True)
    finally:
        os.chdir(cwd)
    import shutil
    shutil.copy(os.path.join(work, "it_suffix2.tsv"), os.path.join(ROOT, "data", "it_suffix.tsv"))
    shutil.copy(os.path.join(work, "it_open.tsv"), os.path.join(ROOT, "data", "it_open.tsv"))
    tsv = os.path.join(work, "it_lex_in.tsv")
    seen = set()
    with open(os.path.join(work, "it_lex.tsv"), encoding="utf8") as f, open(tsv, "w", encoding="utf8") as o:
        for line in f:
            w, st, op, z = line.rstrip("\n").split("\t")
            seen.add(w)
            o.write(f"{w}\t{st}|{op}|{z}\n")
        # unstressed WikiPron entries carry no stress or openness, but their presence
        # lets the rules recognise verb forms (mangiano -> mangiare)
        for line in open(os.path.join(os.environ["POLYG2P_SRC"], "ita_latn_broad.tsv"), encoding="utf8"):
            w = line.split("\t")[0]
            if w not in seen and " " not in w:
                seen.add(w)
                o.write(f"{w}\t|\n")
    build_lex(tsv, os.path.join(out, "it.lex"))


def build_de(work, out):
    from de_convert import lex_to_espeak
    V = "aeiouyøɛɪʊʏœɔɐæɑ"
    rows = []
    for line in open(os.path.join(os.environ["POLYG2P_SRC"], "ipadict_de.txt"), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        if " " in w or not w:
            continue
        vs = []
        for x in p.split(", "):
            e = lex_to_espeak(x, w)
            if "ˈ" not in e:
                m = re.search("[%s]" % V, e)
                if m:
                    e = e[: m.start()] + "ˈ" + e[m.start():]
            e = unicodedata.normalize("NFD", e)
            if e not in vs:
                vs.append(e)
        rows.append((w, vs))
    # ipa-dict's first variant is sometimes an odd one (tun -> tjuːn): order the
    # variants by closeness to the rule-based reading
    multi = [r for r in rows if len(r[1]) > 1]
    rules = subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), "de"], input="\n".join(w for w, _ in multi) + "\n",
                           capture_output=True, text=True, check=True).stdout.split("\n")
    strip = lambda x: unicodedata.normalize("NFC", x).replace("ˈ", "").replace("ˌ", "")  # noqa: E731
    order = {w: sorted(vs, key=lambda v: edit(strip(v), strip(r))) for (w, vs), r in zip(multi, rules)}
    # WikiPron (deu_latn_broad) settles the variant order where it knows the word
    # (Bus: ipa-dict lists buːs first, WikiPron has only bʊs) and adds missing words
    wp = {}
    for line in open(os.path.join(os.environ["POLYG2P_SRC"], "deu_latn_broad.tsv"), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        e = unicodedata.normalize("NFD", lex_to_espeak(p.replace(" ", ""), w))
        if "ˈ" not in e:
            m = re.search("[%s]" % V, e)
            if m:
                e = e[: m.start()] + "ˈ" + e[m.start():]
        wp.setdefault(w, []).append(e)
    have = {w for w, _ in rows}
    bare = lambda x: strip(x).replace("ː", "")  # noqa: E731
    # WikiPron carries no stress: take the stress pattern from the rules/compound
    # splitter running on the ipa-dict-only lexicon (Auseinanderstreben keeps its
    # ˌaʊsaɪnˈandɜ... instead of a default first-syllable stress)
    extra = [w for w in wp if w not in have and " " not in w]
    tmp = os.path.join(work, "de_tmp")
    os.makedirs(tmp, exist_ok=True)
    tsv0 = os.path.join(tmp, "de_lex0.tsv")
    with open(tsv0, "w", encoding="utf8") as f:
        for w, vs in rows:
            f.write(w + "\t" + "\t".join(order.get(w, vs)) + "\n")
    build_lex(tsv0, os.path.join(tmp, "de.lex"))
    rule_out = subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), "de", "--data", tmp, "--words"],
                              input="\n".join(extra) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
    NUC = re.compile("[ˈˌ]?[%sː]+" % V)
    def restress(wpv, ref):
        nuc_r = NUC.findall(unicodedata.normalize("NFC", ref))
        plain = unicodedata.normalize("NFC", wpv).replace("ˈ", "")
        nuc_w = list(NUC.finditer(plain))
        if len(nuc_r) != len(nuc_w):
            return wpv
        out, pos = [], 0
        for m, r in zip(nuc_w, nuc_r):
            out.append(plain[pos:m.start()])
            out.append(r[0] if r[0] in "ˈˌ" else "")
            out.append(m.group())
            pos = m.end()
        out.append(plain[pos:])
        return unicodedata.normalize("NFD", "".join(out))
    tsv = os.path.join(work, "de_lex.tsv")
    with open(tsv, "w", encoding="utf8") as f:
        for w, vs in rows:
            vs = order.get(w, vs)
            if w in wp:
                # pick the ipa-dict variant closest to WikiPron (Bus: bʊs not buːs)
                g = [bare(x) for x in wp[w]]
                vs = sorted(vs, key=lambda v: min(edit(bare(v), x) for x in g))
            f.write(w + "\t" + "\t".join(vs) + "\n")
        for w, ref in zip(extra, rule_out):
            vs = [restress(v, ref) for v in dict.fromkeys(wp[w])]
            f.write(w + "\t" + "\t".join(vs) + "\n")
    build_lex(tsv, os.path.join(out, "de.lex"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--work", default="build-data")
    ap.add_argument("--src", default=os.path.join(ROOT, "sources"))
    ap.add_argument("--ccbysa", action="store_true", help="also build de/it data from CC BY-SA sources")
    ap.add_argument("--only", default=None, help="build just this language (en, it, de)")
    a = ap.parse_args()
    os.environ["POLYG2P_SRC"] = os.path.abspath(a.src)
    sys.path.insert(0, HERE)
    os.makedirs(a.out, exist_ok=True)
    os.makedirs(a.work, exist_ok=True)
    fetch(a.src, a.ccbysa)
    if a.only in (None, "en"):
        build_en(a.work, a.out)
    if a.ccbysa and a.only in (None, "it"):
        build_it(os.path.abspath(a.work), a.out)
    if a.ccbysa and a.only in (None, "de"):
        build_de(a.work, a.out)
    print("done ->", a.out)


if __name__ == "__main__":
    main()
