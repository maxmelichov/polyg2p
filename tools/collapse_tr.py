"""Broad-phone collapse for Turkish (used by eval_acc.py).

Maps polyg2p / espeak output and the WikiPron gold to one broad inventory so
only real pronunciation differences count: lax/tense vowel allophones, dark l,
tap vs trill, palatal k/g, aspiration, vowel length and the varying spellings of
ğ (long vowel, doubled vowel, j, ɰ) are all folded. espeak writes lax ü as ø and ö
as œ, so ö and ü have to share one broad symbol (y); Turkish spelling never
confuses them, so nothing is lost.
"""
import re
import unicodedata

PAIRS = [
    ("ɑ", "a"), ("ɐ", "a"), ("ä", "a"), ("á", "a"), ("ó", "o"), ("ú", "u"), ("é", "e"), ("í", "i"),
    ("æ", "e"), ("ɛ", "e"), ("ɔ", "o"), ("œ", "y"), ("ø", "y"), ("ɪ", "i"), ("ʊ", "u"), ("ʏ", "y"), ("ə", "e"),
    ("ɫ", "l"), ("ɾ", "r"), ("ɹ", "r"), ("ʀ", "r"), ("c", "k"), ("ɟ", "ɡ"), ("ç", "h"), ("x", "h"), ("χ", "h"),
    ("ʋ", "v"), ("β", "v"), ("w", "v"), ("ɲ", "n"), ("ŋ", "n"), ("ɰ", ""), ("ɣ", ""), ("ʔ", ""), ("ʲ", ""), ("ʰ", ""),
    ("ɦ", "h"), ("ɸ", "f"), ("ʎ", "l"), ("ɒ", "o"), ("ʌ", "a"),
    ("ː", ""), ("ˑ", ""),
]
MARKS = ("̧", "̟", "̞", "̪", "̥", "̩", "̃", "̈", "̯", "́",
         "̊", "̚", "̤", "̰", "̝", "̠")
V = "aeiouyɯ"


def collapse(s):
    s = unicodedata.normalize("NFD", s)
    s = s.replace("c\u0327", "h")  # ç (palatal h allophone in the gold)
    for mark in MARKS:
        s = s.replace(mark, "")
    s = unicodedata.normalize("NFC", s)
    for a, b in PAIRS:
        s = s.replace(a, b)
    # ğ after a front vowel: j, a long vowel or nothing in the gold; drop a j that
    # follows e/i and does not precede a back or rounded vowel, on both sides
    s = re.sub(r"(?<=[ei])j(?![aouyɯ])", "", s)
    # doubled identical vowels (gold "a a" for ağa, our aː) -> one
    s = re.sub(rf"([{V}])\1+", r"\1", s)
    # geminates
    s = re.sub(rf"([^{V}])\1", r"\1", s)
    return s
