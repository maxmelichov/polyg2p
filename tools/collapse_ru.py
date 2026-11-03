"""Collapse Russian transcriptions (WikiPron narrow, espeak, polyg2p) to one broad set.

The gold (Wiktionary's ru-IPA output, which Russian-G2P and so polyg2p emulate) has
no stress marks but encodes stress through vowel reduction: ə/ɐ for unstressed а/о,
ɪ for unstressed е/я/и, ʊ for unstressed у, ɵ/ɨ only under stress. espeak writes
stressed a/o as ɑ/o, unstressed о as ʌ, and ʃ/ʒ for ш/ж. Everything is mapped to
the gold's own symbols; stress marks are then dropped, so a wrong stress shows up
as a vowel error (as in the gold) rather than as a stress mark in the wrong place.
"""
import re

_NARROW = "̪̝̞̣̺̹̟̠̩̜̬̥̈̃͡"


def collapse(s):
    for a, b in [("t͡ɕ", "tɕ"), ("t͡s", "ts"), ("d͡ʑ", "dʑ"), ("ʧ", "tɕ"), ("ʦ", "ts"), ("ʨ", "tɕ"), ("g", "ɡ")]:
        s = s.replace(a, b)
    s = "".join(ch for ch in s if ch not in _NARROW)
    s = s.replace("⁽ʲ⁾", "").replace("ˠ", "")  # the gold's optional palatalization, velarized т in ты
    # eval_acc has already turned ʧ into tʃ; in Russian that is ч
    s = s.replace("tʃʲ", "tɕ").replace("tʃ", "tɕ").replace("dʒʲ", "dʑ").replace("dʒ", "dʑ")
    # espeak notation
    s = s.replace("ʃʲ", "ɕ").replace("ʒʲ", "ʑ").replace("ʃ", "ʂ").replace("ʒ", "ʐ")
    # assimilative palatalization of s/z/f/v before a soft consonant is optional in the gold
    s = re.sub(r"([szfvmnt])ʲ(?=[^ʲ]ʲ)", r"\1", s)
    s = re.sub(r"sʲ?ə$", "sə", s)  # reflexive -ся: sʲə in Russian-G2P, sə in the gold
    s = s.replace("tɕʂ", "tʂʂ").replace("ttɕ", "tɕ")  # лучше, матч
    s = s.replace("ʌ", "ɐ").replace("ɑ", "a").replace("ɛ", "e").replace("ʉ", "u")
    s = s.replace("ɫ", "l").replace("ɭ", "l")
    s = s.replace("ː", "").replace("ˑ", "")
    s = s.replace("ˈ", "").replace("ˌ", "").replace("ʲʲ", "ʲ")
    # the gold's reduced vowels: merge the two unstressed-a grades and ə-like symbols
    s = s.replace("ɐ", "ə")
    s = re.sub(r"[ɪɨ]", "i", s)  # unstressed е/я/и grades, and ы after hard consonants
    s = s.replace("ʊ", "u").replace("ɵ", "o")
    return s
