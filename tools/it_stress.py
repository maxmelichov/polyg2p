"""Derive Italian stress / vowel-openness data from Wiktionary (WikiPron, CC BY-SA).

Stress index = number of orthographic vowel letters AFTER the stressed one
(0 = final, 1 = penult, ...), counting every vowel letter, silent or glide.
WikiPron strips stress marks, but ɛ/ɔ occur only in stressed syllables, so words
containing exactly one of them have recoverable stress.

Outputs:
  it_lex.tsv     word <TAB> stress_from_end|-  <TAB> o (open) / c (closed) / -
  it_suffix.tsv  ending <TAB> stress_from_end <TAB> count <TAB> purity
  it_open.tsv    ending-of-stressed-syllable context <TAB> open-probability <TAB> count
"""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import collections, re, sys, unicodedata

VL = "aeiouàèéìòóùíú"
ACC = {"à": "a", "è": "e", "é": "e", "ì": "i", "ò": "o", "ó": "o", "ù": "u", "í": "i", "ú": "u"}
IPA_V = "aeiouɛɔ"
COMPAT = {"a": "a", "e": "eɛ", "i": "i", "o": "oɔ", "u": "u"}


def vowel_letters(w):
    return [(i, ch) for i, ch in enumerate(w) if ch in VL]


def align(word, ipa):
    """Map IPA nuclei to orthographic vowel letter indices (or None)."""
    letters = vowel_letters(word)
    nuc = [c for c in ipa if c in IPA_V]
    L, N = len(letters), len(nuc)
    INF = 10**9
    # dp[i][j]: cost aligning first i letters with first j nuclei
    dp = [[INF] * (N + 1) for _ in range(L + 1)]
    bp = [[None] * (N + 1) for _ in range(L + 1)]
    dp[0][0] = 0
    for i in range(L + 1):
        for j in range(N + 1):
            if dp[i][j] >= INF:
                continue
            if i < L:  # skip letter (silent i / glide i,u)
                b = ACC.get(letters[i][1], letters[i][1])
                c = 1 if b in "iu" else 5
                if dp[i][j] + c < dp[i + 1][j]:
                    dp[i + 1][j] = dp[i][j] + c
                    bp[i + 1][j] = "skip"
            if i < L and j < N:
                b = ACC.get(letters[i][1], letters[i][1])
                c = 0 if nuc[j] in COMPAT[b] else 4
                if dp[i][j] + c < dp[i + 1][j + 1]:
                    dp[i + 1][j + 1] = dp[i][j] + c
                    bp[i + 1][j + 1] = "match"
    if dp[L][N] >= 4:
        return None
    m = {}
    i, j = L, N
    while i > 0 or j > 0:
        op = bp[i][j]
        if op == "match":
            m[j - 1] = i - 1
            i, j = i - 1, j - 1
        elif op == "skip":
            i -= 1
        else:
            return None
    return m, letters, nuc


def main():
    rows = collections.defaultdict(set)
    for line in open(os.path.join(SRC, "ita_latn_broad.tsv"), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        w = unicodedata.normalize("NFC", w)
        if not re.fullmatch(f"[A-Za-z{VL}]+", w):
            continue
        rows[w].add(p.replace(" ", "").replace("͡", ""))
    lex = {}
    for w, ps in rows.items():
        wl = w.lower()
        letters = vowel_letters(wl)
        nv = len(letters)
        if nv == 0:
            continue
        # written final accent decides stress outright
        acc = [k for k, (_, ch) in enumerate(letters) if ch in ACC and k == nv - 1]
        stress_votes, open_votes, hiatus = set(), set(), set()
        all_open = True
        for p in ps:
            opens = [c for c in p if c in "ɛɔ"]
            if len(opens) > 1:
                continue
            a = align(wl, p)
            if a is None:
                continue
            m, _, nuc = a
            if opens:
                j = next(j for j, c in enumerate(nuc) if c in "ɛɔ")
                if j in m:
                    stress_votes.add(nv - 1 - m[j])
                    open_votes.add("o")
            else:
                open_votes.add("c")
                all_open = False
            # a syllabic i before the final vowel (fattoria, zia, Algeria: "i a", not
            # the glide "j a") is the stressed vowel of a word in hiatus
            j = len(nuc) - 2
            hiatus.add(len(nuc) == nv and j >= 0 and nuc[j] == "i" and letters[nv - 2][1] == "i")
        # an open vowel far from the end is a compound's secondary stress, and a
        # reading without any open vowel means the readings may differ in stress
        if not all_open:
            stress_votes = set()
        stress_votes = {s for s in stress_votes if 1 <= s <= 2}
        if not stress_votes and hiatus == {True}:
            stress_votes = {1}
        if acc:
            stress_votes = {0}
        st = stress_votes.pop() if len(stress_votes) == 1 else None
        op = open_votes.pop() if len(open_votes) == 1 else "-"
        if nv == 1:
            st = 0
        lex[w] = (st, op)
    with open("it_lex.tsv", "w", encoding="utf8") as f:
        for w, (st, op) in sorted(lex.items()):
            f.write(f"{w}\t{'-' if st is None else st}\t{op}\n")
    known = {w: st for w, (st, _) in lex.items() if st is not None and len(vowel_letters(w)) >= 2}
    print("words", len(lex), "with known stress", len(known), file=sys.stderr)
    # suffix -> stress distribution (stress index measured from the end, so it
    # transfers between words sharing an ending)
    stats = collections.defaultdict(collections.Counter)
    for w, st in known.items():
        for k in range(2, 8):
            if len(w) > k:
                stats[w[-k:]][st] += 1
    with open("it_suffix.tsv", "w", encoding="utf8") as f:
        for suf, cnt in sorted(stats.items()):
            tot = sum(cnt.values())
            st, c = cnt.most_common(1)[0]
            if tot >= 3 and c / tot >= 0.75:
                f.write(f"{suf}\t{st}\t{tot}\t{c/tot:.2f}\n")
    dist = collections.Counter(known.values())
    print("stress distribution", dist, file=sys.stderr)


if __name__ == "__main__":
    main()
