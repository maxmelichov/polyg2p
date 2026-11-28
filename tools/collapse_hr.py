"""Croatian: map polyg2p / WikiPron (broad, with pitch accents) output to one broad phone set.

Dropped: pitch-accent marks, vowel length (the gold marks only the post-accent length), syllabic marks.
Kept: segments, the palatal series, voicing. Stress position is scored separately from the lexicon.
"""
import re
import unicodedata


def collapse(s):
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = s.replace("ː", "").replace("ˈ", "")
    for a, b in [("ʋ", "v"), ("ɡ", "g"), ("ɕ", "ɕ"), ("ʑ", "ʑ"), ("ʂ", "ʃ"), ("ɛ", "e"), ("ə", "r"), ("ŋ", "n"), ("h", "x")]:
        s = s.replace(a, b)
    s = re.sub(r"(.)\1", r"\1", s)
    return s
