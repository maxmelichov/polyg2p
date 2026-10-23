"""Collapse Hungarian phoneme strings (ours, espeak, WikiPron narrow gold) to a shared broad set.

eval_acc.py has already removed stress marks, tie bars and spaces. The gold is
the *narrow* WikiPron file, so its allophones are folded here: a = ɒ/ɑ, h = h/ɦ/x/ç,
nasal place assimilation (ŋk, mb, ɲɟ), the glide inserted in hiatus (fiú = fijuː),
nasalized vowels. Length is kept (hal/hall), but every long consonant is written
one way (CC = Cː), a long consonant next to another consonant is shortened, and
the merged affricates of t+s / t+sz (tʃː, tsː) equal the unmerged cluster.
"""
import re

VOW = "ɒaɛeioøuy"
PAIRS = [
    ("ɑ", "ɒ"), ("ɦ", "h"), ("xː", "h"), ("x", "h"), ("ç", "h"), ("ʎ", "j"), ("ʝ", "j"), ("ɫ", "l"), ("ɾ", "r"),
    ("ʔ", ""), ("ɱ", "n"), ("ɛː", "eː"), ("ĩ", "i"), ("ã", "ɒ"), ("ẽ", "ɛ"), ("õ", "o"), ("ũ", "u"), ("ə", "ɛ"),
]


def collapse(s):
    for a, b in PAIRS:
        s = s.replace(a, b)
    s = s.replace("̃", "")
    # affricates as single symbols (eval_acc wrote a long affricate as a doubled one)
    for a, b in [("tts", "ʦː"), ("ddz", "ʣː"), ("ttʃ", "ʧː"), ("ddʒ", "ʤː"), ("tʃ", "ʧ"), ("ts", "ʦ"), ("dʒ", "ʤ"),
                 ("dz", "ʣ")]:
        s = s.replace(a, b)
    # doubled consonants -> C + ː
    s = re.sub(r"([^" + VOW + r"ː])\1", r"\1ː", s)
    # a long consonant next to another consonant is short (hallgat, mondd)
    s = re.sub(r"([^" + VOW + r"])ː(?=[^" + VOW + r"ː])", r"\1", s)
    s = re.sub(r"(?<=[^" + VOW + r"ː])([^" + VOW + r"])ː", r"\1", s)
    # homorganic nasals before stops/palatals
    s = s.replace("ŋ", "n")
    s = re.sub(r"m(?=[pbfv])", "n", s)
    s = re.sub(r"ɲ(?=[ɟc])", "n", s)
    # hiatus glide the narrow gold inserts next to i and between e-vowels (fiú, dió, női, beleértve)
    s = re.sub(r"i(ː?)j(?=[" + VOW + "])", r"i\1", s)
    s = re.sub(r"([" + VOW + r"]ː?)j(?=iː?)", r"\1", s)
    s = re.sub(r"([" + VOW + r"]ː?)j(?=eː)", r"\1", s)
    # t+s / t+sz: merged long affricate = cluster = short affricate
    s = s.replace("tʃ", "ʃ").replace("tʧ", "ʧ").replace("tʦ", "ʦ").replace("ʧː", "ʧ").replace("ʦː", "ʦ")
    s = s.replace("ʦs", "ʦ").replace("ʧʃ", "ʧ")
    return s
