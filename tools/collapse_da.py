"""Danish: the module is scored against its own lexicon source (held-out words), see docs/results/da.md.
This collapse is for WikiPron `dan_latn_narrow.tsv` comparisons only: drops diacritics, aspiration, stød,
stress and length, and merges the open front/back vowel pair a æ ɑ and the voiceless-stop notation."""
import re
import unicodedata


def collapse_raw(s):
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = s.replace(" ", "").replace("ˈ", "").replace("ˌ", "").replace("ː", "").replace("ˀ", "")
    for a, b in [("ʰ", ""), ("ˢ", ""), ("æ", "a"), ("ɑ", "a"), ("ɐ", "a"), ("ʌ", "ɔ"), ("ɒ", "ɔ"), ("ɶ", "œ"), ("ʋ", "v"), ("g", "ɡ")]:
        s = s.replace(a, b)
    return s


def collapse(s):
    return re.sub(r"(.)\1", r"\1", s)
