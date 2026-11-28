"""Slovenian: map polyg2p / WikiPron (broad, with pitch-accent marks on the vowels) output to one broad phone set.

Dropped: accent marks, length and stress (position is scored separately), the e/ɛ/ə and o/ɔ quality
choices (lexical in the language, see docs/results/sl.md). The glide `u̯` and a word-final or pre-consonantal
`ʋ` are one phone (`w`): the gold writes the underlying `ʋ`, the module writes the realised glide.
"""
import re
import unicodedata


def collapse_raw(s):
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if not (unicodedata.combining(c) and c != "̯"))
    s = unicodedata.normalize("NFC", s).replace(" ", "")
    s = s.replace("u̯", "w").replace("ˈ", "").replace("ˌ", "")
    s = s.replace("ː", "")
    s = s.replace("ʧ", "tʃ").replace("ʦ", "ts").replace("ʤ", "dʒ").replace("͡", "")
    s = s.replace("ʋ", "v")
    s = re.sub(r"v(?=$|[^aeiouɛɔəyj])", "w", s)
    for a, b in [("ɛ", "e"), ("ə", "e"), ("ɔ", "o"), ("ɾ", "r"), ("ɪ", "j"), ("ŋ", "n"), ("ɡ", "g"), ("ɱ", "m"), ("ɕ", "ʃ"), ("ʑ", "ʒ")]:
        s = s.replace(a, b)
    return s


def collapse(s):
    return re.sub(r"(.)\1", r"\1", s)
