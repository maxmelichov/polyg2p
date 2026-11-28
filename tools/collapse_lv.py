"""Latvian: map polyg2p / WikiPron (narrow) output to one broad phone set.

The gold is narrow with pitch-accent marks (acute, grave, circumflex, macron on vowels), so
precomposed macron letters there are tones, not length. All tone marks and all length are
dropped; e/ɛ/æ, o/ɔ/uo and a/ɑ are merged (their choice is lexical, see docs/results/lv.md).
"""
import re
import unicodedata


def collapse(s):
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = s.replace("ː", "").replace("ˀ", "").replace("ʲ", "").replace("ʷ", "")
    for a, b in [("æ", "e"), ("ɛ", "e"), ("ɑ", "a"), ("ɔ", "o"), ("uo", "o"), ("ɾ", "r"), ("ɫ", "l"), ("ŋ", "n"), ("ʒ", "ʃ"),
                 ("ɡ", "g"), ("au", "aw"), ("ai", "aj"), ("ei", "ej"), ("ui", "uj"), ("oi", "oj"), ("ʒ", "ʃ"), ("w", "v"), ("ɪ", "i"), ("ʊ", "u"), ("ə", "e"), ("ĩ", "i"), ("ɐ", "a"), ("ʔ", "")]:
        s = s.replace(a, b)
    s = re.sub(r"(.)\1", r"\1", s)
    return s
