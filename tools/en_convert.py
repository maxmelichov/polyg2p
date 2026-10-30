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


VG = re.compile(r"[aeiouy]+")
SIBILANT = ("S", "Z", "SH", "ZH", "CH", "JH")


def align_vowels(word, ps):
    """Map each vowel phone index to its spelled vowel-letter group (by order), or
    None when the counts don't line up. A silent final e is dropped first."""
    w = word.lower()
    groups = [(m.start(), m.end()) for m in VG.finditer(w)]
    vph = [i for i, p in enumerate(ps) if is_vowel(p)]
    if len(groups) == len(vph) + 1 and w.endswith("e") and groups[-1][0] == len(w) - 1:
        groups = groups[:-1]
    if len(groups) == len(vph) + 1 and "qu" in w:
        groups = [g for g in groups if not (w[g[0]:g[1]].startswith("u") and g[0] > 0 and w[g[0] - 1] == "q")]
    if len(groups) == len(vph) + 1 and len(groups) > 1 and re.search(r"e[sd]$", w) and groups[-1] == (len(w) - 2, len(w) - 1):
        groups = groups[:-1]  # minutes, changed
    while len(groups) < len(vph):
        # two vowel phones inside one letter group: actual = u + a, create = e + a
        for k, (a, b) in enumerate(groups):
            if b - a >= 2 and w[a:b] in ("ua", "ea", "eo", "ia", "io", "iu", "ie", "uo", "ue", "oa", "oe", "ao", "ii", "eu"):
                groups[k:k + 1] = [(a, a + 1), (a + 1, b)]
                break
        else:
            break
    # realize / create: a magic final e plus one letter pair carrying two phones
    me = re.search(r"e[sd]?$", w)
    if len(groups) == len(vph) and len(groups) > 1 and me and groups[-1][0] == me.start() \
            and ps[vph[-1]][:-1] in ("AY", "EY", "OW", "UW", "IY") and vph[-1] + 1 < len(ps):
        for k, (a, b) in enumerate(groups[:-1]):
            if b - a >= 2 and w[a:b] in ("ea", "ia", "io", "ua", "eo", "ie", "eu"):
                groups = groups[:k] + [(a, a + 1), (a + 1, b)] + groups[k + 1:-1]
                break
    if len(groups) != len(vph):
        return {}
    return {pi: (w[a:b], w[b:b + 3], k == 0, k == len(groups) - 1, w[b:]) for k, (pi, (a, b)) in enumerate(zip(vph, groups))}


def reduced(base, spelled, ps, i, word):
    """espeak's symbol for a CMUdict AH0/IH0 given its spelling: ɪ for a written
    i, ᵻ in -ed/-es/-est and pretonic re-/de-/be- prefixes, ɐ for initial a-, …
    (derived with tools/en_reduced.py; the pronunciation itself is CMUdict's)."""
    n = len(ps)
    prev = ps[i - 1] if i else None
    nxt = ps[i + 1] if i + 1 < n else None
    g, after, initial, final = spelled[:4]
    pretonic = nxt is not None and not is_vowel(nxt) and any(p.endswith("1") for p in ps[i + 1:i + 4] if is_vowel(p)) and not any(is_vowel(p) for p in ps[:i])
    if g in ("i", "y"):
        if after.startswith("bl") or after.startswith("bil"):
            return "ə"  # possible, responsibility
        if final and after in ("l", "ls") and base == "AH":
            return "ə"  # civil, council, april
        return "ɪ"
    if g == "e":
        if final:
            if after.startswith("d") and nxt == "D" and i + 2 == n:
                return "ᵻ" if prev in ("T", "D") else "ə"
            if after.startswith("s") and nxt == "Z" and i + 2 == n:
                return "ᵻ" if prev in SIBILANT else "ə"
            if after in ("st", "t", "ts") or after.startswith("st") or after.startswith("dg"):
                return "ɪ"  # basket, knowledge
            if after.startswith("ss"):
                return "ɪ" if base == "IH" else "ə"
            if after.startswith("n") and base == "IH" and word not in ("women",):
                return "ə"
            return "ɪ" if base == "IH" else "ə"
        if after.startswith("x"):
            return "ɪ" if base == "IH" else "ə"
        if (after[:1] == "t" and after[1:2] and after[1:2] not in "aeiouy") or after.startswith("dg"):
            return "ɪ"  # basketball, knowledgeable
        if pretonic and initial and base == "IH":
            return "ɪ" if after.startswith("n") else "ᵻ"
        if pretonic and base == "AH" and after[:1] in ("c", "d", "b", "f", "l", "p", "t", "h", "v", "g", "s") and word.find(g) <= 1 and not after.startswith("n"):
            return "ᵻ"
        return "ɪ" if base == "IH" else "ə"
    if g.endswith("a"):
        if final and after.startswith("ge"):
            return "ɪ"  # damage, language
        if initial and pretonic and base == "AH" and nxt not in ("R",):
            return "ɐ"
        return "ɪ" if base == "IH" else "ə"
    if g == "u":
        if base == "AH" and not final:
            if initial and after.startswith("n"):
                return "ʌ"
            if initial and after.startswith("p"):
                return "ʌ"
            if prev == "Y" and not after.startswith("l"):
                return "uː"  # ˈɑːɹɡjuːmənt; -ular keeps ə (pɑpjələɹ)
            if prev in ("JH", "CH") and after[:1] in ("c", "t"):
                return "uː"
        if final and after.startswith("te") and base == "AH":
            return "ɪ"  # minute(s)
        return "ɪ" if base == "IH" else "ə"
    return "ɪ" if base == "IH" else "ə"


def arpa_to_espeak(phones, word=""):
    ps = phones.split() if isinstance(phones, str) else list(phones)
    n = len(ps)
    lw = word.lower()
    # CMUdict's post-tonic "2" marks full-vowel quality, not a stress espeak would
    # write: thirty TH ER1 D IY2 -> θˈɜːɾi, president ... EH2 N T -> ...ənt,
    # realize ... AY2 Z -> ɹˈiːəlaɪz
    prim = next((k for k, q in enumerate(ps) if q.endswith("1")), None)
    if prim is not None:
        for k in range(prim + 1, n):
            q = ps[k]
            if not q.endswith("2"):
                continue
            rest = ps[k + 1:]
            if q[:-1] == "IY":
                ps[k] = "IY0"
            elif q[:-1] == "EH" and rest in (["N", "T"], ["N", "T", "S"]) and re.search(r"ents?$", lw):
                ps[k] = "AH0"
            elif q[:-1] == "AY" and rest[:1] == ["Z"] and re.search(r"i[sz](e|ed|es|ing)$", lw):
                ps[k] = "AY0"
    out = []
    spelled = align_vowels(word, ps) if word else {}
    skip_w = False
    for i, p in enumerate(ps):
        prev = ps[i - 1] if i else None
        nxt = ps[i + 1] if i + 1 < n else None
        if p == "W" and skip_w:
            skip_w = False
            continue
        if is_vowel(p):
            base, s = p[:-1], int(p[-1])
            mark = "ˈ" if s == 1 else ("ˌ" if s == 2 else "")
            if base in ("AH", "IH") and s == 0 and i in spelled:
                v = reduced(base, spelled[i], ps, i, lw)
            elif base == "AH":
                v = "ʌ" if s else "ə"
            elif base == "ER":
                # dᵻɹˈɛkt: espeak's reduced vowel in the dir- prefix
                v = "ɜː" if s else ("ᵻ" if (i == 1 and lw.startswith("dir") and (nxt == "R" or (nxt and is_vowel(nxt)))) else "ɚ")
            elif base == "IY":
                if s == 0 and i in spelled and spelled[i][0] == "e" and spelled[i][2] and nxt and not is_vowel(nxt) and any(q[-1] in "12" for q in ps[i + 1:i + 4] if is_vowel(q)):
                    v = "ᵻ"  # ɹᵻpˈɔːɹt, ɹᵻspˌɑːnsəbˈɪlɪɾi: pretonic re-/de-/pre-
                elif s == 0 and i + 1 == n and lw.endswith("day") and len(lw) > 3:
                    v = "eɪ"  # fɹˈaɪdeɪ
                else:
                    v = "iː" if s else "i"
            elif base == "UW" and s == 0:
                v = "u"
            elif base == "UW" and nxt == "R" and i + 2 == n and lw.endswith("oor"):
                v = "ʊ"  # pʊɹ
            elif base == "AA" and nxt == "R":
                v = "ɑː"
            elif base == "AA" and s and i in spelled and (spelled[i][0] in ("au", "aw") or (spelled[i][0] == "a" and re.match(r"l(l(ing|ed|er|s|y|en)?$|[^aeiouylmfkv]|$)", spelled[i][4]))):
                v = "ɔː"  # caused, falling, salt: espeak keeps the ɔː of au/aw/al spellings
            elif base == "AA" and s and i in spelled and spelled[i][0] == "o" and (
                    nxt == "F" or (nxt in ("S", "NG", "G") and re.match(r"(ss|st|ng|g)([^aeiouy]|$|(ing|ed|es|er|y|ly|en)$)", spelled[i][4]))):
                v = "ɔː"  # cost, soft, off, long, dog: lot-cloth split as espeak writes it
            elif base == "AE" and s == 0 and i == 0 and nxt in ("D", "K") and i + 2 < n and not is_vowel(ps[i + 2]):
                v = "ɐ"  # ɐdvˈaɪs, ɐksˈɛptᵻd
            elif base == "AY" and s == 0:
                v = "aɪ"
            elif base == "AO" and nxt == "R":
                v = "ɔː"
            elif base == "IH" and s == 0 and nxt in ("D", "Z") and i + 2 == n and prev in ("T", "D", "S", "Z", "SH", "ZH", "CH", "JH"):
                v = "ᵻ"  # espeak's reduced vowel in -ed / -es suffixes
            else:
                v = VOW[base]
            if base == "AH" and s == 0 and nxt == "W" and i + 2 < n and ps[i + 2] == "AH0" and "ua" in lw:
                v = "uː"  # ˈæktʃuːəl, jˈuːʒuːəli (the glide is not written)
                skip_w = True
            if base == "AA" and s == 0 and i + 1 == n and lw.endswith("a"):
                v = "ə"  # ˈæfɹɪkə

            if base == "ER" and nxt and is_vowel(nxt):
                v += "ɹ"  # espeak keeps an audible r before a following vowel
            out.append(mark + v)
        else:
            if p == "K" and prev == "NG" and nxt == "TH":
                continue  # strength, length: espeak writes ŋθ
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
            if p == "D" and "d" not in lw and prev and nxt and is_vowel(nxt) and nxt[-1] == "0" and (is_vowel(prev) or prev == "R"):
                c = "ɾ"  # thirty TH ER1 D IY2: CMUdict writes the flap as D
            out.append(c)
    # CMUdict marks both halves of some compounds primary (nineteen, etcetera);
    # espeak keeps one primary, the last, and demotes the others
    prim = [k for k, tok in enumerate(out) if tok.startswith("ˈ")]
    for k in prim[:-1]:
        out[k] = "ˌ" + out[k][1:]
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
