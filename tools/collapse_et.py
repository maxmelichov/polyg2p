"""Estonian: map polyg2p / WikiPron (broad) output to one broad phone set.

Removed: quantity degree 3 (ːː, half-length ˑ), consonant length, palatalisation (ʲ), the
lenis/voiceless mark on b d g (b̥ d̥ ɡ̊), glide marks, n/ŋ, ʒ̊. Kept: vowel quality. All length is dropped because WikiPron marks only some quantity degrees.
"""
import re


def collapse(s):
    s = s.replace("ˑ", "").replace("ʲ", "").replace("̥", "").replace("̊", "")
    s = s.replace("ŋk", "ng")  # ng is [ŋk]: the gold writes it n g
    for a, b in [("ɡ", "g"), ("ŋ", "n"), ("ʒ", "ʃ"), ("z", "s"), ("ʋ", "v"), ("ɑ̯", "ɑ"), ("ó", "o")]:
        s = s.replace(a, b)
    # the gold writes quantity degrees 2/3 inconsistently (aabits, Eesti unmarked; maa, kapp marked): all length is dropped
    s = s.replace("ː", "")
    s = re.sub(r"(.)\1", r"\1", s)
    return s
