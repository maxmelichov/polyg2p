"""Collapse Dutch phoneme strings (espeak-ng `nl`, polyg2p, WikiPron broad) to one broad phone set.

Notation differences that are not pronunciation differences: espeak writes ʋ/ɵ/ʌʊ/ɛɪ/œy/ɪː/ɔː and
h; WikiPron writes ʋ/ʏ/ɑu̯/ɛi̯/œy̯/eː/oː and ɦ, with non-syllabic marks on glides.
"""
import re
import unicodedata


def collapse(s):
    s = unicodedata.normalize("NFC", s)
    s = s.replace("ˈ", "").replace("ˌ", "").replace(" ", "").replace(".", "")
    for a, b in [
        ("ɦ", "h"), ("ɵ", "ʏ"), ("ʌʊ", "ɑu"), ("ʌu", "ɑu"), ("ɑu̯", "ɑu"), ("ɔu̯", "ɑu"), ("ɔʊ", "ɑu"), ("ɛɪ", "ɛi"), ("ɛi̯", "ɛi"),
        ("œy̯", "œy"), ("œʏ", "œy"), ("ɪː", "eː"), ("ɔː", "oː"), ("ɛː", "ɛ"), ("iː", "i"), ("yː", "y"), ("uː", "u"), ("øː", "ø"),
        ("tʲ", "tj"), ("ɲ", "nj"), ("ɡ", "ɣ"), ("g", "ɣ"), ("ʒ", "ʃ"), ("dʒ", "dʃ"), ("ʔ", ""), ("ɾ", "r"), ("ɹ", "r"), ("ɐ", "ə"),
        ("ɛ̃", "ɛn"), ("ɑ̃", "ɑn"), ("ɒ", "ɔ"),
    ]:
        s = s.replace(a, b)
    # glides: i̯ / y̯ / u̯ / ʊ / w after a vowel are all off-glides (eval_acc strips the
    # non-syllabic mark before calling us, so the vowel letter is what is left)
    s = s.replace("i̯", "i").replace("y̯", "i").replace("u̯", "u").replace("ʊ", "u")
    s = s.replace("̯", "").replace("̩", "").replace("̃", "")
    s = s.replace("ɑɪ", "ɑi")
    for v in ("i", "eː", "y", "ɑ", "aː", "oː", "u"):
        s = s.replace(v + "w", v + "u")
    # hiatus glides: WikiPron writes raːdijoː / raːdioː, œyjən, teːjaːtər / teːaːtər; espeak ideːjaːl
    s = re.sub(r"(oː|aː|u)j(?=[aeiouyəɑɛɔʏɪœø])", r"\1i", s)
    s = re.sub(r"(i|eː|œy|ɛi|y|ɑu)j(?=[aeiouyəɑɛɔʏɪœø])", r"\1", s)
    s = re.sub(r"(i|eː)i(?=[aeiouyəɑɛɔʏɪœø])", r"\1", s)
    s = s.replace("ʃ", "sj")  # sj / ʃ
    s = s.replace("sjj", "sj")
    # j between a high front vowel and a following vowel (radio / radijo) is predictable
    s = re.sub(r"(i|eː)j(?=[aeiouyəɑɛɔʏɪœø])", r"\1", s)
    # ts / s before i in -tie are both current
    s = s.replace("tsi", "si")
    # word-final -ən may drop its n (WikiPron lists both); -en- before k/g assimilates
    s = re.sub(r"ə[nŋ]$", "ə", s)
    s = re.sub(r"ə[nŋ](?=[^aeiouyəɑɛɔʏɪœø])", "ə", s)
    s = re.sub(r"(.)\1", r"\1", s)  # degemination across morpheme boundaries
    return s
