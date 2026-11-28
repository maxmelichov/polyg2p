"""English homographs in context, scored on Google's Wikipedia Homograph Data.

    git clone https://github.com/google-research-datasets/WikipediaHomographData
    python tools/en_homograph_bench.py WikipediaHomographData/data --data data-out

The set (Gorman, Mazovetskiy & Nikolaev 2018, Apache-2.0) labels one homograph per
Wikipedia sentence with a pronunciation id: 162 homographs, about 100 sentences
each, split 90/10 into train and eval. Nothing from it is stored in this
repository; pass the path of your own copy.

Each system phonemizes the whole sentence. The phonemes it gives the labelled word
are matched to the nearest of that homograph's pronunciations (phone edit distance
after collapsing notation, plus the syllable that carries primary stress); a tie
between two pronunciations counts as wrong. The baseline answers every sentence
with the homograph's most frequent label in the train split.

options: --split eval|train   which split to score (default eval)
         --no-espeak          skip espeak-ng (needs `phonemizer` otherwise)
         --show N             list N misses of polyg2p
         --map                print, per homograph, each distinct output and the label it was matched to
         --tsv FILE           write the per-homograph table
"""
import argparse, collections, csv, os, re, subprocess, sys, unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
VOWELS = set("aeiouæɑɔəɛɜɪʊʌɚɐᵻ")


def read_tsv(path):
    with open(path, encoding="utf8", newline="") as f:
        rows = list(csv.reader(f, delimiter="\t", quotechar='"'))
    return rows[1:]


def collapse(p):
    """Phones in a shared broad notation, and the index of the stressed syllable."""
    p = unicodedata.normalize("NFC", p)
    p = p.replace("ʤ", "dʒ").replace("ʧ", "tʃ").replace("ʣ", "dz").replace("ʦ", "ts").replace("ɡ", "g").replace("'", "ˈ")
    p = re.sub(r"[0-9ː\s]", "", p)
    stress, syl, prev_v = None, 0, False
    for c in p:
        if c == "ˈ":
            if stress is None:
                stress = syl
            prev_v = False
            continue
        if c == "ˌ":
            prev_v = False
            continue
        v = c in VOWELS
        if v and not prev_v:
            syl += 1
        prev_v = v
    p = p.replace("ˈ", "").replace("ˌ", "")
    p = p.replace("ɜɹ", "ɚ").replace("əɹ", "ɚ").replace("ɜ", "ɚ").replace("ɚɹ", "ɚ")
    p = p.replace("ᵻ", "ɪ").replace("ɐ", "ə").replace("ɾ", "t").replace("ɔ", "ɑ").replace("ɪɹ", "iɹ")
    return p, stress


# vowels that the two notations write differently for the same sound
NEAR = [set("ɪə"), set("iɪ"), set("uʊ"), set("ʌə")]


def edit(a, b):
    sub = lambda x, y: 0 if x == y else 0.25 if any(x in s and y in s for s in NEAR) else 1
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + sub(ca, cb)))
        prev = cur
    return prev[-1]


def match(ph, cands):
    """The wordid whose pronunciation is nearest to `ph`; None when empty or tied."""
    if not ph:
        return None
    a, sa = collapse(ph)
    multi = len({collapse(p)[0] for _, p in cands}) < len(cands)  # differ in stress only
    scored = []
    for wid, pron in cands:
        b, sb = collapse(pron)
        d = edit(a, b)
        if sa is not None and sb is not None and sa != sb:
            d += 1.5
        elif multi and sa is None:
            d += 0.75
        scored.append((d, wid))
    scored.sort()
    if len(scored) > 1 and scored[0][0] == scored[1][0]:
        return None
    return scored[0][1]


def run_polyg2p(items, data):
    cmd = [os.path.join(ROOT, "target", "release", "en_homograph")] + (["--data", data] if data else [])
    inp = "".join(f"{s}\t{a}\t{b}\n" for _, _, s, a, b in items)
    return subprocess.run(cmd, input=inp, capture_output=True, text=True, check=True).stdout.split("\n")[: len(items)]


def run_espeak(items):
    from phonemizer.backend import EspeakBackend
    from phonemizer.separator import Separator
    b = EspeakBackend("en-us", with_stress=True, language_switch="remove-flags", preserve_punctuation=False)
    sep = Separator(phone="", word=" ")
    words = lambda s: [w for w in s.split() if re.search(r"\w", w)]
    full = b.phonemize([s for _, _, s, _, _ in items], separator=sep, strip=True, njobs=8)
    # the sentence up to and including the word: used when the word counts differ (numbers, symbols)
    pre = b.phonemize([s.encode("utf8")[:int(e)].decode("utf8") for _, _, s, _, e in items], separator=sep, strip=True, njobs=8)
    out, direct = [], 0
    for (_, _, s, a, _), f, p in zip(items, full, pre):
        k = len(words(s.encode("utf8")[:int(a)].decode("utf8")))
        ws, fs = words(s), f.split()
        if len(ws) == len(fs) and k < len(fs):
            direct += 1
            out.append(fs[k])
        else:
            out.append(p.split()[-1] if p.split() else "")
    return out, direct


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus", help="WikipediaHomographData/data")
    ap.add_argument("--data", default=None)
    ap.add_argument("--split", default="eval")
    ap.add_argument("--no-espeak", action="store_true")
    ap.add_argument("--show", type=int, default=0)
    ap.add_argument("--map", action="store_true")
    ap.add_argument("--tsv", default=None)
    a = ap.parse_args()

    cands = collections.defaultdict(list)
    for h, wid, _label, pron, *_ in read_tsv(os.path.join(a.corpus, "wordids.tsv")):
        cands[h].append((wid, pron))
    load = lambda split: [r[:5] for h in sorted(cands) for r in read_tsv(os.path.join(a.corpus, split, h + ".tsv"))]
    train, items = load("train"), load(a.split)
    major = {}
    for h in cands:
        c = collections.Counter(r[1] for r in train if r[0] == h)
        major[h] = c.most_common(1)[0][0] if c else cands[h][0][0]

    systems = {"polyg2p": run_polyg2p(items, a.data)}
    if not a.no_espeak:
        try:
            systems["espeak-ng"], direct = run_espeak(items)
            print(f"espeak-ng: {direct}/{len(items)} sentences aligned word for word, the rest read up to the word", file=sys.stderr)
        except ImportError:
            print("phonemizer is not installed: espeak-ng skipped", file=sys.stderr)
    preds = {name: [match(ph, cands[it[0]]) for it, ph in zip(items, outs)] for name, outs in systems.items()}
    preds["majority"] = [major[it[0]] for it in items]

    covered = set()
    probe = subprocess.run([os.path.join(ROOT, "target", "release", "en_accent"), "en-gb"] + (["--data", a.data] if a.data else []),
                           input="\n".join(sorted(cands)) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
    for h, line in zip(sorted(cands), probe):
        if line.rstrip("\n").split("\t")[-1] == "H":
            covered.add(h)

    names = list(preds)
    per = {h: collections.Counter() for h in cands}
    for i, it in enumerate(items):
        per[it[0]]["n"] += 1
        for nm in names:
            per[it[0]][nm] += preds[nm][i] == it[1]
    n = len(items)
    print(f"{a.split} split: {n} sentences, {len(cands)} homographs; polyg2p has context rules for {len(covered)}")
    print(f"{'':<22}" + "".join(f"{nm:>11}" for nm in names))
    tot = lambda hs, nm: sum(per[h][nm] for h in hs) / max(1, sum(per[h]["n"] for h in hs))
    mac = lambda hs, nm: sum(per[h][nm] / per[h]["n"] for h in hs if per[h]["n"]) / max(1, sum(1 for h in hs if per[h]["n"]))
    rest = set(cands) - covered
    for label, hs in (("all (micro)", set(cands)), ("all (macro)", None), ("with rules", covered), ("without rules", rest)):
        if hs is None:
            print(f"{label:<22}" + "".join(f"{mac(set(cands), nm):>10.1%} " for nm in names))
        else:
            print(f"{label + ' n=' + str(sum(per[h]['n'] for h in hs)):<22}" + "".join(f"{tot(hs, nm):>10.1%} " for nm in names))
    print()
    rows = sorted(cands, key=lambda h: (per[h]["polyg2p"] / max(1, per[h]["n"]), h))
    lines = ["homograph\trules\tn\t" + "\t".join(names)]
    for h in rows:
        lines.append(f"{h}\t{'yes' if h in covered else 'no'}\t{per[h]['n']}\t" + "\t".join(f"{per[h][nm]}" for nm in names))
    print("\n".join(lines))
    if a.tsv:
        with open(a.tsv, "w", encoding="utf8") as f:
            f.write("\n".join(lines) + "\n")
    if a.map:
        print("\noutput -> label")
        for nm, outs in systems.items():
            seen = collections.Counter((it[0], ph, p) for it, ph, p in zip(items, outs, preds[nm]))
            for (h, ph, p), c in sorted(seen.items(), key=lambda x: (x[0][0], str(x[0][1]))):
                print(f"  {nm}\t{h}\t{ph or '-'}\t{p or 'TIE/NONE'}\t{c}")
    if a.show:
        print("\npolyg2p misses")
        k = 0
        for it, ph, p in zip(items, systems["polyg2p"], preds["polyg2p"]):
            if p != it[1] and k < a.show:
                k += 1
                print(f"  {it[0]}\twant {it[1]}\tgot {ph or '-'} ({p})\t{it[2][:140]}")


if __name__ == "__main__":
    main()
