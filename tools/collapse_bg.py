"""Bulgarian: map polyg2p / WikiPron (narrow) output to one broad phone set.

Merged: o/ɔ, e/ɛ, dark/clear l, centralised-vowel marks, w/v, n/ŋ, m/ɱ. Kept: the stress-dependent
vowel reduction (a vs ɐ, ɤ vs ɐ) and the palatalisation mark (ʲ).
"""
import re


def collapse(s):
    for a, b in [("ɔ", "o"), ("ɛ", "e"), ("ɫ", "l"), ("w", "v"), ("ŋ", "n"), ("ɱ", "m"), ("ɡ", "g"), ("ə", "ɐ"), ("ʃt", "ʃt"),
                 ("ɤ̞", "ɤ"), ("͡", "")]:
        s = s.replace(a, b)
    s = s.replace("̟", "").replace("̞", "").replace("̯", "")
    s = re.sub(r"(.)\1", r"\1", s)
    return s
