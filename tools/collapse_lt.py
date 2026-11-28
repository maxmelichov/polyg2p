"""Lithuanian: map polyg2p / WikiPron (narrow, with pitch-accent digits) output to one broad phone set.

The gold is narrow: stress digits `¹ ²`, half-long `ˑ`, centring and advancing marks. All of those, length and the
quality of the vowels (a ɐ ɑ, e ɛ æ, i ɪ, u ʊ, o ɔ) are dropped (lexical / predictable); palatalisation `ʲ`, the
consonants and the diphthongs are kept. Stress position is scored separately.
"""
import re
import unicodedata


def collapse_raw(s):
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = s.replace(" ", "").replace("¹", "").replace("²", "").replace("ˈ", "").replace("ˌ", "")
    s = s.replace("ː", "").replace("ˑ", "").replace("͡", "")
    s = s.replace("ʧ", "tʃ").replace("ʤ", "dʒ").replace("ʦ", "ts").replace("ʣ", "dz")
    for a, b in [("ɐ", "a"), ("ɑ", "a"), ("ɒ", "a"), ("ä", "a"), ("ɛ", "e"), ("æ", "e"), ("ɪ", "i"), ("ʊ", "u"), ("ɔ", "o"), ("ɣ", "h"),
                 ("ʋ", "v"), ("ɾ", "r"), ("ɫ", "l"), ("ɕ", "ʃ"), ("ʑ", "ʒ"), ("ʂ", "ʃ"), ("g", "ɡ"), ("ʰ", ""), ("ʷ", ""), ("ˠ", "")]:
        s = s.replace(a, b)
    return s


def collapse(s):
    return re.sub(r"(.)\1", r"\1", s)
