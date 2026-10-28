"""Indonesian: map polyg2p / espeak / WikiPron output to one broad phone set.

Only the schwa/e contrast and the consonants are phonemic; everything else that
the three sources write differently is folded: lax vowel allophones (ɪ ʊ ɛ ɔ),
the diphthong spellings (aɪ / a i̯ / a i), the glottal stop WikiPron writes for
final k and before initial vowels, unreleased stops (k̚ t̚), espeak's ç for sy,
inserted glides in hiatus (ija, uwa) and stress marks.
"""
import re
import unicodedata

V = "aeiouə"


def collapse(s):
    s = unicodedata.normalize("NFD", s)
    for mark in ("̚", "̯", "̂", "̆", "̃", "̪", "ʰ", "ː", "ˈ", "ˌ", ".", " ", "͡", "ʲ", "̩"):
        s = s.replace(mark, "")
    s = unicodedata.normalize("NFC", s)
    for a, b in [("ʧ", "tʃ"), ("ʤ", "dʒ"), ("g", "ɡ"), ("ɛ", "e"), ("ɔ", "o"), ("ɪ", "i"), ("ʊ", "u"), ("ä", "a"),
                 ("ɑ", "a"), ("ʌ", "a"), ("ɘ", "ə"), ("ç", "ʃ"), ("ʒ", "ʒ"), ("ɾ", "r")]:
        s = s.replace(a, b)
    # glottal stop: final k in WikiPron (tidaʔ), otherwise a hiatus/onset mark
    s = re.sub("ʔ$", "k", s)
    s = s.replace("ʔ", "")
    # glide after a vowel at the end or before a consonant is the second half of a diphthong (pay -> pai)
    s = re.sub(rf"(?<=[{V}])j(?![{V}])", "i", s)
    s = re.sub(rf"(?<=[{V}])w(?![{V}])", "u", s)
    # inserted glides in hiatus: ija -> ia, uwa -> ua
    s = re.sub(rf"ij(?=[{V}])", "i", s)
    s = re.sub(rf"uw(?=[{V}])", "u", s)
    # n assimilated to a following palatal affricate (WikiPron: məɲdʒadi) is still n
    s = re.sub(r"[ɲŋ](?=tʃ|dʒ)", "n", s)
    # a i / a u before a vowel: WikiPron writes the glide (pakajan, pulawan)
    s = re.sub(rf"(?<=a)j(?=[{V}])", "i", s)
    s = re.sub(rf"(?<=a)w(?=[{V}])", "u", s)
    # geminates
    s = re.sub(rf"([^{V}])\1", r"\1", s)
    return s
