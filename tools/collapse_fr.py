"""Broad-phone collapse for the French evaluation (used by eval_acc.py).

Maps polyg2p, espeak-ng and Wiktionary output to one broad phone set so only
real pronunciation differences count:
  - r variants (ʁ ʀ r ɾ ɹ) -> ʁ; ɥ -> y (espeak writes the glide as y); length marks dropped
  - i/u before a vowel are glides (gold writes both "klijɑ̃" and "kliɑ̃")
  - ə inside a word is optional (e-caduc) unless it is the only vowel (le, de)
  - e/ɛ, o/ɔ, ø/œ are distinguished only in the last syllable of the word; elsewhere the
    contrast is neutralized in modern French and Wiktionary is inconsistent
The nasal tilde is already stripped by eval_acc.py's common pass.
"""
import re

VOW = "aeɛiouɔøœəɑyɪʊɐ"


def collapse(s):
    s = s.replace("aː", "ɑ")
    s = s.replace("‿", "").replace("-", "").replace("ː", "").replace("ˑ", "")
    for a, b in [("ʀ", "ʁ"), ("r", "ʁ"), ("ɾ", "ʁ"), ("ɹ", "ʁ"), ("ɥ", "y"), ("ɪ", "i"), ("ʊ", "u"), ("ɐ", "a"),
                 ("ŋɡ", "ŋ"), ("ʔ", ""), ("ɡn", "ɲ")]:
        s = s.replace(a, b)
    s = s.replace("ij", "i").replace("ji", "i")
    # glides before vowels
    s = re.sub(f"i(?=[{VOW}])", "j", s)
    s = re.sub(f"u(?=[{VOW}])", "w", s)
    # e-caduc is optional except as the sole vowel
    if any(c in VOW.replace("ə", "") for c in s):
        s = s.replace("ə", "")
    # vowel quality: keep only in the final syllable
    vs = [m.start() for m in re.finditer(f"[{VOW}]", s)]
    if len(vs) >= 2:
        head = s[: vs[-1]]
        tail = s[vs[-1]:]
        head = head.replace("ɛ", "e").replace("ɔ", "o").replace("œ", "ø")
        s = head + tail
    return s


def collapse_raw(s):
    """Runs before eval_acc's common stripping: keep nasal vowels distinct from
    their oral counterparts by rewriting them to capital letters (ɛ̃/œ̃ -> E,
    ɑ̃ -> A, ɔ̃ -> O) before the tilde is removed."""
    import unicodedata
    s = unicodedata.normalize("NFD", s)
    for a, b in [("ɛ̃", "E"), ("œ̃", "E"), ("ɑ̃", "A"), ("ã", "A"), ("ɔ̃", "O"), ("õ", "O")]:
        s = s.replace(a, b)
    return unicodedata.normalize("NFC", s)
