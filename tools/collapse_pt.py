"""Broad phone collapse for Brazilian Portuguese (polyg2p / espeak pt-br / WikiPron).

Called by eval_acc.py after the common step (stress marks, tie bars, nasal
tildes already removed). Maps the three notations to one broad set so that only
real pronunciation differences count: r-sounds, coda s/z/ʃ, final a/e/o
reduction, glide spelling, t/d palatalization, espeak's æ y ʊ eɪ ɐ̃ʊ̃ symbols.
Open vs closed e/o (ɛ/e, ɔ/o) is kept: it is the hard part of the language.
"""
import re, unicodedata


def collapse(s):
    # the gold uses precomposed nasal vowels (ẽ ĩ ũ õ ã); decompose and drop the tilde
    s = unicodedata.normalize("NFD", s).replace("\u0303", "")
    s = s.replace("ʲ", "").replace("ʷ", "").replace("ː", "").replace("ɾə", "r").replace("ə", "")
    # espeak nasal spellings (tildes are already stripped): eɪŋ/eɪm -> e, ɐʊ -> aw
    s = s.replace("ɐy", "aj").replace("ɐʊ", "aw").replace("eɪŋ", "ej")
    s = re.sub(r"eɪm(?=[pb])", "ej", s)
    s = s.replace("ŋ", "")
    s = re.sub(r"m(?=[pb]|$)", "", s)  # nasal coda m (gold writes a nasal vowel only)
    # rhotics: coda (before a consonant or final) all become r; strong vs flap elsewhere
    s = re.sub(r"[ɾɻɹrʁhxχɣ](?=[^aeiouɐɛɔæʊɪyjw]|$)", "r", s)
    s = re.sub(r"[ʁhxχɣ]", "R", s)
    s = re.sub(r"[ɾɻɹ]", "r", s)
    # coda sibilants: s
    s = re.sub(r"[ʃʒz](?=[^aeiouɐɛɔæʊɪyjwlr]|$)", "s", s)
    s = re.sub(r"z(?=$)", "s", s)
    s = s.replace("lj", "ʎ")
    # t/d + i (affricates were folded to tʃ/dʒ by the common step)
    s = s.replace("tʃi", "ti").replace("dʒi", "di").replace("tʃy", "ti").replace("dʒy", "di")
    s = s.replace("tʃ", "ti").replace("dʒ", "di")  # final noite -> nojtʃ
    s = s.replace("ʊ", "u").replace("ɪ", "i").replace("y", "i").replace("æ", "a").replace("ɐ", "a")
    s = s.replace("j", "i").replace("w", "u")
    # final unstressed vowel reduction: both o/u and e/i are accepted in the gold
    s = re.sub(r"o(s?)$", r"u\1", s)
    s = re.sub(r"e(s?)$", r"i\1", s)
    s = s.replace("ou", "o").replace("ei", "e")  # pouco/poco, peixe/peʃe
    s = re.sub(r"(.)\1", r"\1", s)
    return s
