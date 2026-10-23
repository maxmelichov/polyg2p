"""CMUdict ARPAbet -> espeak en-us notation (as seen after BlueTTS normalize_text).

The pronunciation comes from CMUdict (BSD); only the symbol conventions follow espeak.
"""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import re

VOW = {
    "AA": "ɑː", "AE": "æ", "AO": "ɔː", "AW": "aʊ", "AY": "aɪ", "EH": "ɛ",
    "EY": "eɪ", "IH": "ɪ", "OW": "oʊ", "OY": "ɔɪ", "UH": "ʊ", "UW": "uː",
}
CONS = {
    "B": "b", "CH": "ʧ", "D": "d", "DH": "ð", "F": "f", "G": "ɡ", "HH": "h",
    "JH": "ʤ", "K": "k", "L": "l", "M": "m", "N": "n", "NG": "ŋ", "P": "p",
    "R": "ɹ", "S": "s", "SH": "ʃ", "T": "t", "TH": "θ", "V": "v", "W": "w",
    "Y": "j", "Z": "z", "ZH": "ʒ",
}


def is_vowel(p):
    return p[-1].isdigit()


def arpa_to_espeak(phones, word=""):
    ps = phones.split() if isinstance(phones, str) else list(phones)
    n = len(ps)
    out = []
    for i, p in enumerate(ps):
        prev = ps[i - 1] if i else None
        nxt = ps[i + 1] if i + 1 < n else None
        if is_vowel(p):
            base, s = p[:-1], int(p[-1])
            mark = "ˈ" if s == 1 else ("ˌ" if s == 2 else "")
            if base == "AH":
                v = "ʌ" if s else "ə"
            elif base == "ER":
                v = "ɜː" if s else "ɚ"
            elif base == "IY":
                v = "iː" if s else "i"
            elif base == "UW" and s == 0:
                v = "u"
            elif base == "AA" and nxt == "R":
                v = "ɑː"
            elif base == "AO" and nxt == "R":
                v = "ɔː"
            elif base == "IH" and s == 0 and nxt in ("D", "Z") and i + 2 == n and prev in ("T", "D", "S", "Z", "SH", "ZH", "CH", "JH"):
                v = "ᵻ"  # espeak's reduced vowel in -ed / -es suffixes
            else:
                v = VOW[base]
            if base == "ER" and nxt and is_vowel(nxt):
                v += "ɹ"  # espeak keeps an audible r before a following vowel
            out.append(mark + v)
        else:
            c = CONS[p]
            # flapping: t/d after a vowel (or r) and before an unstressed vowel
            if p == "T" and nxt == "AH0" and i + 3 == n and ps[i + 2] == "N" and prev and is_vowel(prev):
                out.append("ʔ")
                continue
            if p == "N" and prev == "AH0" and i >= 2 and ps[i - 2] == "T" and i + 1 == n and is_vowel(ps[i - 3]) if i >= 3 else False:
                out[-1] = "n\u0329"
                continue
            if p == "T" and prev and nxt and is_vowel(nxt) and nxt[-1] == "0" and (is_vowel(prev) or prev == "R"):
                c = "ɾ"
            out.append(c)
    # espeak drops a secondary stress that sits right before the primary
    for k, tok in enumerate(out):
        if tok.startswith("ˌ"):
            nxt_v = next((t for t in out[k + 1:] if t.lstrip("ˈˌ")[:1] in VOWEL_START), "")
            if nxt_v.startswith("ˈ"):
                out[k] = tok[1:]
    return fold("".join(out))


VOWEL_START = set("aeiouæɑɔəɛɜɪʊʌɚᵻ")
_FOLD = [(re.compile(r"ts(?=[^\w]|$)"), "ʦ"), (re.compile(r"dz"), "ʣ")]


def fold(s):
    """Same ligature folding BlueTTS's normalize_text applies to espeak output."""
    for pat, rep in _FOLD:
        s = pat.sub(rep, s)
    return s


def load_cmudict(path):
    lex = {}
    for line in open(path, encoding="utf8"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        w, *p = line.split()
        base = re.sub(r"\(\d+\)$", "", w)
        lex.setdefault(base, []).append(" ".join(p))
    return lex


if __name__ == "__main__":
    import sys, collections
    lex = load_cmudict(os.path.join(SRC, "cmudict.dict"))
    ref = [l.rstrip("\n").split("\t") for l in open("es_en.tsv", encoding="utf8")]
    agree = 0
    diffs = collections.Counter()
    ex = collections.defaultdict(list)
    import difflib
    for w, e in ref:
        if w not in lex: continue
        cands = [arpa_to_espeak(p) for p in lex[w]]
        if e in cands:
            agree += 1
            continue
        c = cands[0]
        sm = difflib.SequenceMatcher(a=c, b=e, autojunk=False)
        for tag, a1, a2, b1, b2 in sm.get_opcodes():
            if tag != "equal":
                k = f"{c[a1:a2]}>{e[b1:b2]}"
                diffs[k] += 1
                if len(ex[k]) < 3:
                    ex[k].append(f"{w}:{c}|{e}")
    print(f"exact agree {agree}/{len(ref)} = {agree/len(ref):.1%}")
    for k, v in diffs.most_common(int(sys.argv[1]) if len(sys.argv) > 1 else 40):
        print(v, k, ex[k])
