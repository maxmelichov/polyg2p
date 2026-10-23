"""Collapse Polish phoneme strings (ours, espeak, WikiPron gold) to a shared broad set.

eval_acc.py has already removed stress, tie bars and combining tildes, so a gold
nasal glide `w̃` arrives as a plain `w`; nasal realizations before fricatives /
word-finally (ɛ̃, ɛw̃, ɛn) and before stops (ɛn, ɛw̃, ɛ̃) are all reduced to the
oral vowel so only real pronunciation differences count.
"""
import re

FRIC = "fvszʃʒɕʑx"
PAIRS = [
    ("ɘ", "ɨ"), ("ɒ", "a"), ("ɫ", "w"), ("ʂ", "ʃ"), ("ʐ", "ʒ"), ("r̝", "ʒ"), ("e", "ɛ"), ("o", "ɔ"), ("é", "ɛ"),
    ("ç", "x"), ("ç", "x"), ("ɣ", "x"), ("ʲi", "i"), ("ʲ", "j"), ("ŋ", "n"), ("ɲj", "ɲ"), ("ː", ""),
    ("ã", "a"),
]


def collapse(s):
    for a, b in PAIRS:
        s = s.replace(a, b)
    # nasal vowel realizations: drop the nasal element before a fricative, word-finally,
    # and before a stop/affricate (ɛn t ~ ɛ̃ t ~ ɛ w̃ t)
    s = re.sub(f"([ɛɔ])[nmɲw](?=[{FRIC}]|$)", r"\1", s)
    s = re.sub(f"([ɛɔ])[nmɲw](?=[ptkbdɡ]|ts|dz|tʃ|dʒ|tɕ|dʑ)", r"\1", s)
    # n before a soft affricate (gold n, espeak/ours ɲ)
    s = re.sub("ɲ(?=tɕ|dʑ)", "n", s)
    # geminates spelled double in gold (lekki) vs single
    s = re.sub(r"([^aɛiɔuɨ])\1", r"\1", s)
    return s
