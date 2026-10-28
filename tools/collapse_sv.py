"""Swedish: map polyg2p / espeak / WikiPron output to one broad phone set.

Vowel length and the long/short qualities are phonemic and kept (ʉː ~ ʉ,
ʏ ~ y, ɛ ~ e ~ ə, æ ~ ɛ before r, œː ~ øː are notation). Tone accents (¹ ²),
consonant length (WikiPron marks it inconsistently; espeak only tː kː),
doubled consonants and the retroflex spelling (ʈ ɖ ɳ ɭ ʂ vs r + t d n l s;
espeak drops the r before t) are removed. The sj-sound is ɧ (espeak `sx`,
Wiktionary ɧ/ʃ/ʂ in loans), tj is ɕ (t͡ɕ in some entries).
"""
import re

PAIRS = [
    ("¹", ""), ("²", ""), ("‿", ""), ("↓", ""), ("ˇ", ""), ("ˑ", ""), ("͜", ""), ("͡", ""),
    ("ʉ̟", "ʉ"), ("ʉː", "ʉ"), ("ʏ", "y"), ("ə", "ɛ"), ("e", "ɛ"), ("æ", "ɛ"), ("ɶ", "œ"), ("œː", "øː"),
    ("ø", "ø"), ("ɑ", "a"), ("ɑː", "ɑː"), ("ɘ", "ɛ"), ("ɜ", "ɛ"), ("ɐ", "a"),
    ("tʃ", "ɕ"), ("sx", "ɧ"), ("ʃ", "ɧ"), ("x", "ɧ"), ("ʒ", "ɧ"), ("tɕ", "ɕ"), ("ʝ", "j"), ("ɾ", "r"), ("ɹ", "r"), ("ʁ", "r"),
    ("ɽ", "rd"), ("ʈ", "rt"), ("ɖ", "rd"), ("ɳ", "rn"), ("ɭ", "rl"), ("ʂ", "rs"), ("ʋ", "v"),
    ("tɧ", "ɧ"), ("ɡs", "ks"), ("ŋɡ", "ŋ"),
    ("\u0302", ""), ("\u030c", ""), ("\u0301", ""), ("\u0300", ""), ("\u0304", ""), ("\u0306", ""),
]
VOW = "aɛiɪoɔuʊʉɵyøœɑ"


def collapse(s):
    s = s.replace("aː", "ɑː")  # a few gold entries write aː for long a
    for a, b in PAIRS:
        s = s.replace(a, b)
    # "ɑ" without length (gold) is the short a; "ɑː" is long
    s = re.sub("ɑ(?!ː)", "a", s)
    # "ø" short (rare in gold) is œ; "o" short is ɔ, "u" short is ʊ, "i" short is ɪ
    s = re.sub("ø(?!ː)", "œ", s)
    s = re.sub("o(?!ː)", "ɔ", s)
    s = re.sub("u(?!ː)", "ʊ", s)
    s = re.sub("i(?!ː)", "ɪ", s)
    # consonant length and doubled consonants
    s = re.sub(f"([^{VOW}ː])ː", r"\1", s)
    s = re.sub(f"([^{VOW}ː])\\1", r"\1", s)
    # ŋk ~ nk (Wiktionary writes both), ɪ + long vowel ~ j (period: pɛrɪuːd ~ pɛrjuːd)
    s = s.replace("ŋk", "nk")
    s = re.sub("ɪ(?=[aɛɔʊuoøœɑ]ː)", "j", s)
    # final -ig: Wiktionary lists both ɪ and ɪɡ; compare without the g
    s = re.sub("ɪɡ$", "ɪ", s)
    # -ien: ɪɛn ~ jɛn (Spanien)
    s = re.sub("ɪɛn$", "jɛn", s)
    return s
