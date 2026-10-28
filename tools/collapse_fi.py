"""Finnish: map polyg2p / espeak / WikiPron (broad) output to one broad phone set.

Vowel and consonant length are phonemic and kept (as doubled consonants, `ː`
on vowels). Removed: stress, the non-syllabic mark on diphthong offglides
(gold `ɑi̯` = espeak `ai`), boundary gemination `ˣ` (a sandhi feature of the
following word), espeak's allophonic `ɪ` for unstressed i, and the gold's
`ɑ`/`ʋ` vs espeak's `a`/`v` symbol choice.
"""
import re


def collapse(s):
    for a, b in [("͡", ""), ("‿", ""), ("ˈ", ""), ("ˌ", ""), (".", ""), (" ", ""), ("ˣ", ""), ("̯", ""),
                 ("g", "ɡ"), ("ɑ", "a"), ("ʋ", "v"), ("w", "v"), ("ɪ", "i"), ("ʔ", ""), ("ə", "e"), ("ɤ", "e"),
                 ("ã", "a"), ("̃", ""), ("ʦ", "ts"), ("ʧ", "tʃ"), ("ʣ", "dz"), ("ʤ", "dʒ"), ("ɕ", "ʃ")]:
        s = s.replace(a, b)
    # long consonants: one convention (doubled); long vowels keep ː
    s = re.sub(r"([^aeiouyæøː])ː", r"\1\1", s)
    # nasal place before k/p is predictable (espeak writes ŋk/mp, Wiktionary nk/np at compound seams)
    s = s.replace("ŋk", "nk").replace("ŋɡ", "nɡ").replace("mp", "np").replace("mb", "nb")
    return s
