"""Catalan: map polyg2p / espeak / WikiPron output to one broad phone set.

Open/closed mid vowels and the reduced vowel are kept (phonemic in Central
Catalan); allophonic detail is removed: spirant vs stop b/d/g, tap vs trill,
velarized l, apical s, glide vs vowel spelling of i/u, geminates at
morpheme boundaries (l·l, nn, bbl), ɱ, ɲʃ/ɲs, and dʒ/ʒ for word-initial j.
"""
import re


def collapse(s):
    for a, b in [("͡", ""), ("‿", ""), ("ˈ", ""), ("ˌ", ""), (".", ""), (" ", ""), ("ʦ", "ts"), ("ʧ", "tʃ"),
                 ("ʣ", "dz"), ("ʤ", "dʒ"), ("g", "ɡ"), ("ː", "")]:
        s = s.replace(a, b)
    for a, b in [("β", "b"), ("ð", "d"), ("ɣ", "ɡ"), ("ɫ", "l"), ("̺", ""), ("ɾ", "r"), ("ɱ", "m"), ("j", "i"),
                 ("w", "u"), ("ɲʃ", "ɲs"), ("tʃ", "č"), ("dʒ", "ǆ"), ("dz", "ʣ"), ("ts", "c")]:
        s = s.replace(a, b)
    # word-initial j: Wiktionary writes both dʒ and ʒ
    if s.startswith("ǆ"):
        s = "ʒ" + s[1:]
    # ŋn / ɡn (tècnic): same cluster
    s = s.replace("ŋn", "ɡn")
    # geminates (col·legi, Anna, poble) and ss across x+c (excepte)
    s = re.sub(r"([^aeiouəɛɔ])\1", r"\1", s)
    return s
