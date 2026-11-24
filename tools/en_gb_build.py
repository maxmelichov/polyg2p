"""Build data/en_gb_lex.tsv: the words whose Southern British form the en-gb rules do not reach from the American form.

Source: Britfone 3.0.1 (https://github.com/JoseLlarena/Britfone, MIT), ~16k RP entries in IPA.
A word gets an entry only when the rule output differs from every Britfone variant in more than
secondary stress or the i/schwa choice of an unstressed vowel (those follow the base engine).

usage: en_gb_build.py BRITFONE.csv [--data DIR] [--holdout K] [--report FILE]
  --holdout K   leave out the words with md5(word) % 10 == K (to score the rules + stem fallback on unseen words)
Needs target/release/en_accent built from a tree whose data/en_gb_lex.tsv holds only its header line (rules only).
"""
import argparse, collections, hashlib, os, re, subprocess, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
HEADER = "# word\tSouthern British form; built by tools/en_gb_build.py from Britfone (MIT)\n"


def britfone(path):
    """word -> [forms] in polyg2p's en-gb notation."""
    d = collections.OrderedDict()
    for line in open(path, encoding="utf8"):
        line = line.rstrip("\n")
        if not line or "," not in line:
            continue
        w, p = line.split(",", 1)
        w = re.sub(r"\(\d+\)$", "", w.strip()).lower()
        toks = p.split()
        toks = [t.replace("ɐ", "ʌ").replace("ɛə", "eə").replace("g", "ɡ") for t in toks]
        d.setdefault(w, []).append(fold("".join(toks)))
    return d


UNITS = ["aɪ", "aʊ", "eɪ", "ɔɪ", "əʊ", "ɪə", "eə", "ʊə", "ɑː", "ɔː", "iː", "uː", "ɜː"]
VOW = "aeiouæɑɔəɛɜɪʊʌɒ"


def fold(s):
    s = s.replace("tʃ", "ʧ").replace("dʒ", "ʤ")
    return re.sub("ts$", "ʦ", re.sub("dz$", "ʣ", s))


def loose(s):
    """Differences that are the base engine's to decide: secondary stress, and how an unstressed
    vowel is written (i / ɪ / iː, u / ʊ / uː, ɪ / ə, jʊ / jə)."""
    s = fold(s).replace("ˌ", "")
    out, i, primary = [], 0, False
    while i < len(s):
        if s[i] == "ˈ":
            primary = True
            out.append("ˈ")
            i += 1
            continue
        u = next((x for x in UNITS if s.startswith(x, i)), s[i])
        i += len(u)
        if u[0] in VOW:
            if not primary:
                u = {"i": "ə", "iː": "ə", "ɪ": "ə", "ɪə": "ə", "u": "U", "uː": "U", "ʊ": "U", "ʊə": "Uə"}.get(u, u)
                if u == "U" and out and out[-1] == "j":
                    u = "ə"
            primary = False
        out.append(u)
    return "".join(out).replace("əə", "ə").replace("jə", "jU")


def run(words, data):
    cmd = [os.path.join(ROOT, "target", "release", "en_accent"), "en-gb"] + (["--data", data] if data else [])
    out = subprocess.run(cmd, input="\n".join(words) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
    return [o.split("\t") for o in out[: len(words)]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("britfone")
    ap.add_argument("--data", default=os.environ.get("POLYG2P_DATA"))
    ap.add_argument("--holdout", type=int, default=-1)
    ap.add_argument("--report")
    ap.add_argument("--out", default=os.path.join(ROOT, "data", "en_gb_lex.tsv"))
    a = ap.parse_args()
    bf = britfone(a.britfone)
    words = [w for w in bf if re.fullmatch(r"[a-z']+", w)]
    res = run(words, a.data)
    n = exact = ok = 0
    entries, miss = [], []
    for w, (_, ga, got) in zip(words, res):
        n += 1
        vs = bf[w]
        exact += got in vs
        if any(loose(got) == loose(v) for v in vs):
            ok += 1
            continue
        miss.append((w, ga, got, vs[0]))
        if a.holdout < 0 or int(hashlib.md5(w.encode()).hexdigest(), 16) % 10 != a.holdout:
            entries.append((w, vs[0]))
    with open(a.out, "w", encoding="utf8") as f:
        f.write(HEADER)
        for w, p in sorted(entries):
            f.write(f"{w}\t{p}\n")
    print(f"britfone words {n}; rules alone: exact {exact} ({100 * exact / n:.1f}%), same but for secondary stress / unstressed i-schwa {ok} ({100 * ok / n:.1f}%); entries written {len(entries)}")
    if a.report:
        with open(a.report, "w", encoding="utf8") as f:
            for m in miss:
                f.write("\t".join(m) + "\n")


if __name__ == "__main__":
    main()
