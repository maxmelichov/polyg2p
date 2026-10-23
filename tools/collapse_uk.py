"""Collapse Ukrainian transcriptions (WikiPron narrow, espeak, polyg2p) to one broad set.

The gold file is narrow: it writes the в allophones (ʋ w ʍ u̯), dark ɫ, i̯ for й after
a vowel, semi-palatalized labials/velars before і (pʲ kʲ ...), consonant length, and
unstressed-vowel reduction (а→ɐ, е/и→e, у→ʊ, о→o before a stressed u/i). This maps
all of that, and espeak's notation (β h t̪ iː ʎ ɲ), to the same broad form.

Stress: the gold has no stress marks but encodes stress in the vowel reduction.
`eval_acc.py` strips ˈ before calling this plugin, so there the reduced vowels are
merged with the full ones (ɐ→a, e→ɛ, ɪ→ɛ, ʊ→u) and stress is NOT scored: only the
segmental rules are. `uk_eval.py` sets STRESS_AWARE = True and passes raw strings:
then a transcription with a stress mark is reduced the way the gold is, so a wrong
stress shows up as a vowel error.
"""
import re
import unicodedata

STRESS_AWARE = False

VOWELS = "aɐɛeɪiɔoʊu"
_LIG = [("t͡ʃ", "tʃ"), ("d͡ʒ", "dʒ"), ("t͡s", "ts"), ("d͡z", "dz"), ("ʧ", "tʃ"), ("ʤ", "dʒ"), ("ʦ", "ts"), ("ʣ", "dz"),
        ("͡", ""), ("g", "ɡ")]
_NARROW = "̪̝̞̣̺̹̟̠̩̜̬̥̈̃"


def _reduce(s):
    """Apply the gold's unstressed-vowel reduction to a stress-marked transcription."""
    out = []
    nv = sum(1 for ch in s if ch in VOWELS)
    stressed = False
    seen = 0
    for ch in s:
        if ch == "ˈ":
            stressed = True
            continue
        if ch == "ˌ":
            continue
        if ch in VOWELS:
            st = stressed or nv == 1
            stressed = False
            seen += 1
            if st:
                ch = {"ɐ": "a", "e": "ɛ", "ʊ": "u"}.get(ch, ch)
            else:
                ch = {"a": "ɐ", "ɛ": "e", "ɪ": "e", "u": "ʊ"}.get(ch, ch)
        out.append(ch)
    return "".join(out)


def collapse(s):
    s = unicodedata.normalize("NFD", s.strip()).replace(" ", "").replace(".", "")
    for a, b in _LIG:
        s = s.replace(a, b)
    s = s.replace("iː", "i").replace("uː", "u").replace("ä", "a").replace("ɯ", "ɪ")
    # non-syllabic vowels: й after a vowel, в after a vowel
    s = s.replace("i̯", "j").replace("u̯", "w").replace("̯", "")
    for m in _NARROW:
        s = s.replace(m, "")
    nv = sum(1 for ch in s if ch in VOWELS)
    if STRESS_AWARE and "ˈ" in s and nv > 1:
        s = _reduce(s)
    else:
        s = s.replace("ˈ", "").replace("ˌ", "")
        # monosyllables: the gold reads clitics unstressed (ʋe for ви), so stress is not scored there
        if not STRESS_AWARE or nv <= 1:
            for a, b in [("ɐ", "a"), ("e", "ɛ"), ("ɪ", "ɛ"), ("ʊ", "u")]:
                s = s.replace(a, b)
    s = s.replace("ɔ", "o")
    # в and its allophones; after a vowel before a consonant / word end it is the glide
    for a, b in [("β", "w"), ("ʋ", "w"), ("ʍ", "w"), ("v", "w")]:
        s = s.replace(a, b)
    s = re.sub(f"(?<=[{VOWELS}])u(?=[^{VOWELS}ʲ]|$)", "w", s)
    for a, b in [("ɦ", "h"), ("ɫ", "l"), ("ʎ", "lʲ"), ("ɲ", "nʲ"), ("ɬ", "ʃ"), ("ɣ", "x")]:
        s = s.replace(a, b)
    # semi-palatalized labials, velars and hushers (narrow only), and j after a labial before a vowel
    s = re.sub(r"(?<=[pbmfwkɡxhʃʒ])ʲ", "", s)
    s = re.sub(f"(?<=[pbmfw])j(?=[{VOWELS}])", "", s)
    # length and doubled consonants are not scored; a hard+soft pair counts as the soft one
    s = s.replace("ː", "")
    s = re.sub(r"(ts|dz|tʃ|dʒ|[^aɐɛeɪiɔoʊuʲ])ʲ?\1(ʲ?)", lambda m: m.group(1) + ("ʲ" if "ʲ" in m.group(0) else ""), s)
    s = s.replace("j", "i").replace("ii", "i")
    s = re.sub(f"(?<=[{VOWELS}])i(?=[{VOWELS}])", "j", s)
    return s
