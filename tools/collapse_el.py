"""Collapse rules for Greek (eval_acc.py plugin): map polyg2p, espeak and WikiPron
output to a shared broad phone set so only real pronunciation differences count.

Neutralized: stress (stripped by eval_acc), flap vs trill, palatal ɟ vs ɡ (espeak
has no ɟ), ɪ (espeak's unstressed post-vocalic ι), the nasal of μπ/ντ/γκ
(WikiPron writes both "pede" and "pende" for πέντε), double consonants, and
synizesis: an unstressed ι before a vowel (ia vs ja/ça/ʎa/ɲa), on which WikiPron
is inconsistent (it lists both ʝ/ç and i variants for the same words).
Set POLYG2P_EL_STRICT=1 to keep synizesis distinct."""
import os
import re

STRICT = os.environ.get("POLYG2P_EL_STRICT") == "1"


def collapse(s):
    s = s.replace("ʲ", "").replace("‿", "").replace("ː", "").replace("ⁿ", "n")
    for a, b in [("ɪ", "i"), ("ɾ", "r"), ("ɟ", "ɡ"), ("ɱ", "m"), ("ɛ", "e"), ("ɔ", "o"), ("ɐ", "a"), ("w", "u"),
                 ("ʂ", "sç"), ("j", "ʝ")]:
        s = s.replace(a, b)
    # nasal + voiced stop: ŋɡ mb nd -> ɡ b d (and the ŋ of γχ/γξ)
    s = re.sub(r"[mnŋ]([bdɡ])", r"\1", s)
    s = re.sub(r"ŋ(?=[xçk])", "n", s)
    # double consonants
    s = re.sub(r"([^aeiou])\1", r"\1", s)
    if not STRICT:
        # synizesis-neutral: glides back to i + vowel
        s = s.replace("ʎ", "li").replace("ɲ", "ni")
        s = re.sub(r"[çʝ]i?(?=[aou])", "i", s)
        s = re.sub(r"ii+", "i", s)
    return s
