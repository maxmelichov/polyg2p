"""Derive Ukrainian stress data from Wiktionary (WikiPron ukr_cyrl_narrow, CC BY-SA).

WikiPron strips stress marks, but the narrow transcription reduces unstressed
vowels (а→ɐ, е/и→e, у→ʊ), so the stressed vowel is recoverable for most words:
it is the unique full vowel among a ɛ ɪ u, or, when there is none, the unique
о/і (which do not reduce).

Stress index = number of orthographic vowel letters AFTER the stressed one
(0 = final, 1 = penult, ...).

Outputs (in the current directory unless --out is given):
  uk_lex.tsv      word <TAB> stress_from_end[,alt...]      (CC BY-SA, data pack only)
  uk_suffix.tsv   ending <TAB> stress_from_end <TAB> count <TAB> purity   (aggregate)
  uk_stem.tsv     word start <TAB> stress_from_start <TAB> count            (aggregate;
                  only starts whose words all stress the same syllable)

Lookup order in uk.rs: built-in table, lexicon, longest word start in uk_stem.tsv,
longest ending in uk_suffix.tsv, penult.

With --eval WORDS, holds the listed words out of the suffix statistics and reports
how well the ending rule predicts their stress (what rules-only mode will do).
"""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import argparse, collections, re, sys, unicodedata

VL = "аеєиіїоуюя"
FULL = {"a": "а", "ɛ": "е", "ɪ": "и", "u": "у"}  # stressed-only vowel qualities
NUCLEI = set("aɐɛeɪiɔoʊu")


def vowel_letters(w):
    return [i for i, ch in enumerate(w) if ch in VL]


def nuclei(p):
    """IPA nuclei of a narrow transcription (skipping non-syllabic i̯ u̯)."""
    if "ä" in p or "ɯ" in p or "̣" in p:
        return None  # too narrow: skip the entry
    out = []
    chars = list(p)
    for k, ch in enumerate(chars):
        if ch in NUCLEI:
            nxt = chars[k + 1] if k + 1 < len(chars) else ""
            if nxt == "̯":  # non-syllabic
                continue
            out.append(ch)
    return out


def stress_of(w, p):
    """Set of possible stress indices (from the end) for one word/pronunciation."""
    vl = vowel_letters(w)
    nuc = nuclei(p)
    if nuc is None or len(nuc) != len(vl):
        return None
    nv = len(vl)
    if nv == 1:
        return {0}
    full = [k for k, n in enumerate(nuc) if n in FULL]
    if len(full) == 1:
        return {nv - 1 - full[0]}
    if len(full) > 1:
        return None
    amb = [k for k, n in enumerate(nuc) if n in "ɔi"]
    if len(amb) == 1:
        return {nv - 1 - amb[0]}
    return None


def load(path):
    rows = collections.defaultdict(list)
    for line in open(path, encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        w = unicodedata.normalize("NFC", w).lower().replace("’", "'")
        if not re.fullmatch(r"[а-яіїєґ']+", w):
            continue
        rows[w].append(unicodedata.normalize("NFC", p.replace(" ", "")))
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=".")
    ap.add_argument("--eval", default=None, help="file of words to hold out and score")
    ap.add_argument("--min", type=int, default=3)
    ap.add_argument("--purity", type=float, default=0.75)
    ap.add_argument("--maxlen", type=int, default=9)
    ap.add_argument("--stem-min", type=int, default=2)
    ap.add_argument("--stem-purity", type=float, default=1.0)
    a = ap.parse_args()
    rows = load(os.path.join(SRC, "ukr_cyrl_narrow.tsv"))
    lex = {}
    for w, ps in rows.items():
        votes = set()
        for p in ps:
            s = stress_of(w, p)
            if s:
                votes |= s
        if votes:
            lex[w] = sorted(votes)
    with open(os.path.join(a.out, "uk_lex.tsv"), "w", encoding="utf8") as f:
        for w, st in sorted(lex.items()):
            f.write(f"{w}\t{','.join(map(str, st))}\n")
    known = {w: st[0] for w, st in lex.items() if len(st) == 1 and len(vowel_letters(w)) >= 2}
    print("words", len(rows), "with stress", len(lex), "unambiguous polysyllables", len(known), file=sys.stderr)
    held = set()
    if a.eval:
        held = {l.strip() for l in open(a.eval, encoding="utf8")}
    train = {w: st for w, st in known.items() if w not in held}
    stats = collections.defaultdict(collections.Counter)
    for w, st in train.items():
        for k in range(1, a.maxlen + 1):
            if len(w) > k:
                stats[w[-k:]][st] += 1
    table = {}
    for suf, cnt in stats.items():
        tot = sum(cnt.values())
        st, c = cnt.most_common(1)[0]
        if tot >= a.min and c / tot >= a.purity:
            table[suf] = (st, tot, c / tot)
    # longest-match lookup: an ending that predicts the same as its longest
    # shorter suffix in the table adds nothing
    kept = {}
    for suf in sorted(table, key=len):
        st = table[suf][0]
        shorter = next((kept[suf[k:]][0] for k in range(1, len(suf)) if suf[k:] in kept), None)
        if shorter != st:
            kept[suf] = table[suf]
    with open(os.path.join(a.out, "uk_suffix.tsv"), "w", encoding="utf8") as f:
        for suf, (st, tot, pur) in sorted(kept.items()):
            f.write(f"{suf}\t{st}\t{tot}\t{pur:.2f}\n")
    table = {s: v[0] for s, v in kept.items()}
    # word starts with a fixed stressed syllable counted from the start (stem stress)
    sstats = collections.defaultdict(collections.Counter)
    for w, st in train.items():
        nv = len(vowel_letters(w))
        for k in range(4, 9):
            if len(w) > k:
                sstats[w[:k]][nv - 1 - st] += 1
    stem = {}
    for pre, cnt in sstats.items():
        tot = sum(cnt.values())
        st, c = cnt.most_common(1)[0]
        if tot >= a.stem_min and c / tot >= a.stem_purity:
            stem[pre] = (st, tot)
    skept = {}
    for pre in sorted(stem, key=len):
        shorter = next((skept[pre[:k]][0] for k in range(len(pre) - 1, 0, -1) if pre[:k] in skept), None)
        if shorter != stem[pre][0]:
            skept[pre] = stem[pre]
    with open(os.path.join(a.out, "uk_stem.tsv"), "w", encoding="utf8") as f:
        for pre, (st, tot) in sorted(skept.items()):
            f.write(f"{pre}\t{st}\t{tot}\n")
    stem = {s: v[0] for s, v in skept.items()}
    dist = collections.Counter(known.values())
    print("stress distribution", sorted(dist.items()), file=sys.stderr)
    if a.eval:
        ok = tot = 0
        default = collections.Counter()
        for w in held:
            if w not in known:
                continue
            tot += 1
            st = None
            nv = len(vowel_letters(w))
            for k in range(min(8, len(w) - 1), 3, -1):
                if w[:k] in stem and stem[w[:k]] < nv:
                    st = nv - 1 - stem[w[:k]]
                    break
            if st is None:
                for k in range(a.maxlen, 0, -1):
                    if len(w) > k and w[-k:] in table:
                        st = table[w[-k:]]
                        break
            if st is None or st >= nv:
                default[known[w]] += 1
                st = 1 if nv >= 2 else 0
            ok += st == known[w]
        print(f"held-out stress accuracy {ok}/{tot} = {ok/max(tot,1):.1%}; no-rule words by true stress {sorted(default.items())}", file=sys.stderr)


if __name__ == "__main__":
    main()
