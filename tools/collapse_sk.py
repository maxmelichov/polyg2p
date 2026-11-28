"""Slovak: map polyg2p / WikiPron (broad) output to one broad phone set.

Length and the palatal series are kept (phonemic). Removed: o/ɔ, e/ɛ, ɦ/h and ɫ/l notation,
the dark/clear l, v/w/ʋ glide spelling, the glide/vowel distinction of u̯ in ô/au, narrow
geminate length on consonants, the vowel-less 'syllabic' mark (already stripped by eval_acc).
"""
import re


def collapse(s):
    for a, b in [("ɔ", "o"), ("ɛ", "e"), ("ɦ", "h"), ("ɫ", "l"), ("ʋ", "v"), ("w", "v"), ("ʊ", "u"), ("aʊ", "au"),
                 ("eʊ", "eu"), ("oʊ", "ov"), ("ou", "ov"), ("ʑ", "ʒ"), ("ɡ", "g"), ("ŋ", "n"), ("e̞", "e"), ("o̞", "o"), ("ø", "e"),
                 ("œ", "e"), ("y", "i"), ("ɐ", "a"), ("ɪ̯", "i"), ("i̯", "i"), ("ɪ", "i"), ("u̯", "u")]:
        s = s.replace(a, b)
    s = re.sub(r"([^aeiouæː])ː", r"\1", s)  # consonant length (geminates) is not marked by rule
    s = re.sub(r"([^aeiouæː])\1", r"\1", s)
    return s
