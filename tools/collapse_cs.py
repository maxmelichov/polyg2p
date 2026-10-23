"""Czech: map polyg2p / espeak / WikiPron (narrow) output to one broad phone set.

Length (ː) and the palatal series are kept — they are phonemic — but narrow
detail (ɪ/i, ɛ/e, ɦ/h, u̯/ʊ, schwa epenthesis, voicing diacritics, syllabic
marks, geminates at morpheme boundaries) is removed.
"""
import re


def collapse(s):
    for a, b in [("͡", ""), ("‿", ""), ("ˈ", ""), ("ˌ", ""), (".", ""), (" ", ""), ("ʦ", "ts"), ("ʧ", "tʃ"),
                 ("ʣ", "dz"), ("ʤ", "dʒ"), ("g", "ɡ")]:
        s = s.replace(a, b)
    for a, b in [("u̯", "u"), ("i̯", "j"), ("ʊ", "u"), ("ɪ", "i"), ("ɛ", "e"), ("ɔ", "o"), ("ɦ", "h"), ("ɣ", "h"),
                 ("̊", ""), ("̥", ""), ("̬", ""), ("r̝", "ř"), ("ʑ", "z"), ("ɱ", "m"), ("ŋ", "n"), ("ʔ", ""),
                 ("ɫ", "l"), ("ə", ""), ("dʒ", "ǆ"), ("tʃ", "č"), ("dz", "ʣ"), ("ts", "c")]:
        s = s.replace(a, b)
    # espeak writes ej as eɪ; i after a vowel is the glide j
    s = re.sub(r"([aeiou]ː?)i(?!ː)", r"\1j", s)
    # syllabic marks: r̩ l̩ m̩ -> plain consonants (the vowel-less cluster is the same)
    s = s.replace("̩", "")
    # geminates arising at morpheme boundaries (oddělení, nejjasnější): Wiktionary writes one
    s = re.sub(r"([^aeiouː])\1", r"\1", s)
    return s
