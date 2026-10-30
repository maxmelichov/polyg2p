"""German lexicon IPA (Wiktionary-style) -> espeak de notation."""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import re, unicodedata
from en_convert import fold
REPL = [
    ("ɐ̯", "ɾ"), ("aɪ̯", "aɪ"), ("aʊ̯", "aʊ"), ("ɔɪ̯", "ɔø"), ("ɔʏ̯", "ɔø"),
    ("ʁ", "r"), ("ɐ", "ɐ"), ("g", "ɡ"), ("t͡s", "ts"), ("t͡ʃ", "tʃ"), ("p͡f", "pf"),
    ("d͡ʒ", "dʒ"), ("̯", ""), ("ʔ", ""), (".", ""), ("‿", ""), ("\u0311", ""), ("\u0361", ""),
]
V = "aeiouyøɛɪʊʏœəɔɐæɑɜ"
def lex_to_espeak(ipa, word=""):
    s = unicodedata.normalize("NFC", ipa.strip().strip("/"))
    s = re.sub("[\u0329\u030d]+", "\u0329", s)  # WikiPron sometimes doubles the syllabic mark
    s = s.replace("ʁɐ̯", "ɐ").replace("n̩", "ən").replace("l̩", "əl").replace("m̩", "əm").replace("ŋ̩", "ən")
    s = s.replace("ʊʁ", "ʊɐ̆") if False else s
    for a, b in REPL:
        s = s.replace(a, b)
    # -er schwa: espeak writes ɜ
    s = s.replace("ɐ", "ɜ")
    # ʊ + r before a consonant -> ʊɐ (durch, Kurs)
    s = re.sub(r"ʊ[rɾ](?=[^%s ˈˌː]|$)" % V, "ʊɐ", s)
    # r after a plosive, or after a vowel before a consonant/end -> ɾ
    s = re.sub(r"(?<=[pbtdkɡ])r", "ɾ", s)
    s = re.sub(r"(?<=[%sː])r(?=[ˈˌ]?[^%s ˈˌ]|$)" % (V, V), "ɾ", s)
    # long a is ɑː in espeak
    s = s.replace("aː", "ɑː")
    # unstressed tense vowels: espeak keeps length (zu -> tsuː), i before a vowel -> ɪ
    s = re.sub(r"i(?=[%s])" % V, "ɪ", s)
    s = re.sub(r"(?<![ˈˌ])([iuoey])(?![ː%s])" % V, r"\1ː", s)
    # Duden -ig: ɪç (Wiktionary often gives the southern ɪk)
    lw = word.lower()
    if "ig" in lw and "ik" not in lw:
        s = re.sub(r"ɪk(?=$|t|s|ʦ)", "ɪç", s)
    # espeak writes the stress mark right before the vowel, Wiktionary at syllable onset
    s = re.sub(r"([ˈˌ])([^aeiouyøɛɪʊʏœəɔɐæɑ]*)", r"\2\1", s)
    s = s.replace("tʃ", "ʧ").replace("dʒ", "ʤ")
    return fold(s)
def load(path):
    lex = {}
    for line in open(path, encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        lex[w] = [lex_to_espeak(x, w) for x in p.split(", ")]
    return lex
if __name__ == "__main__":
    from diffstat import report
    lex = load(os.path.join(SRC, "ipadict_de.txt"))
    ref = [l.rstrip("\n").split("\t") for l in open("es_de.tsv", encoding="utf8")]
    report([(w, lex.get(w), e) for w, e in ref], 40)
