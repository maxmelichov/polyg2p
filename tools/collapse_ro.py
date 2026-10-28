"""Broad-phone collapse for Romanian (used by eval_acc.py).

Maps polyg2p / espeak output and the WikiPron gold to one broad inventory so
only real pronunciation differences count. Folded: the glide spellings of the
diphthongs (e̯a / ea, o̯a / ɔa, aj / aɪ, aw / aʊ / au̯), palatalisation and the
whispered final i (Cʲ, Cʲʲ, iʲ → i), hiatus vs glide for i/u (j → i, w → u: the
gold itself is inconsistent here), tap vs trill, ŋ, palatal k/g, length, and
espeak's `y` for â/î (a notation bug of its `ro` voice, not a pronunciation),
ɡz/ks for `x` (the gold has both for `ex-` words), and e̯a after tʃ/dʒ, where the
gold writes both `t͡ʃe̯a` and `t͡ʃa` (`aceasta` / `această`), and an i-glide after tʃ/dʒ
(`t͡ʃoban` / `t͡ʃioban`).
"""
import re
import unicodedata

PAIRS = [
    ("ʲʲ", "ʲ"), ("iʲ", "ii"), ("iː", "ii"), ("ʲ", "i"), ("ɡz", "ks"), ("tʃea", "tʃa"), ("dʒea", "dʒa"), ("ɪ", "i"), ("ʊ", "u"), ("ɔ", "o"), ("ɛ", "e"), ("ø", "e"), ("y", "ɨ"),
    ("j", "i"), ("w", "u"), ("ɾ", "r"), ("ŋ", "n"), ("c", "k"), ("ɟ", "ɡ"), ("ɫ", "l"), ("ə̆", "ə"), ("ʔ", ""),
    ("ː", ""), ("ˑ", ""),
]
MARKS = ("̯", "̪", "̞", "̟", "̝", "̠", "̥", "̩", "̃", "̈", "̆", "́", "̊", "̚")
V = "aeiouəɨ"


def collapse(s):
    s = unicodedata.normalize("NFD", s)
    for mark in MARKS:
        s = s.replace(mark, "")
    s = unicodedata.normalize("NFC", s)
    for a, b in PAIRS:
        s = s.replace(a, b)
    # doubled consonants (înnoda, accent after folding) and the gold's "i i" for -ii
    s = re.sub(rf"([^{V}])\1", r"\1", s)
    s = re.sub(r"i{3,}", "ii", s)
    # the gold's epenthetic j in an -ie/-ia hiatus (iunie → juniје): ii + vowel → i + vowel
    s = re.sub(rf"ii(?=[{V}])", "i", s)
    # an i-glide after tʃ/dʒ is not distinctive (gold: t͡ʃoban / t͡ʃioban, pit͡ʃor / pit͡ʃior)
    s = re.sub(r"(tʃ|dʒ)i(?=[aou])", r"\1", s)
    return s
