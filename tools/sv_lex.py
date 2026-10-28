"""Convert the WikiPron Swedish gold (swe_latn_broad.tsv) to espeak-style notation
for the optional data pack lexicon `sv.lex` (CC BY-SA, never compiled into src/).

usage: sv_lex.py sources/swe_latn_broad.tsv out.tsv && target/release/build_lex out.tsv data/sv.lex

Notation changes (see src/sv.rs): ʉː -> ʉ, ʏ -> y, æ -> ɛ, œ̞ -> œ, ɧ -> sx, retroflex
ʈ ɖ ɳ ɭ -> rt rd rn rl, t͡ɕ -> ɕ, consonant length dropped, tone accents ¹ ² -> ˈ before
the stressed vowel (one long vowel = stressed when the entry has no accent mark).
"""
import re, sys, unicodedata

MAP = [("ʉː", "ʉ"), ("ʏ", "y"), ("æː", "ɛː"), ("æ", "ɛ"), ("œː", "øː"), ("ɶː", "øː"), ("ɶ", "œ"), ("ɧ", "sx"),
       ("ʈ", "rt"), ("ɖ", "rd"), ("ɳ", "rn"), ("ɭ", "rl"), ("ɽ", "rd"), ("tɕ", "ɕ"), ("ɡs", "ks"), ("ɛɵ", "ɛʉ"),
       ("ɪː", "iː"), ("ʊː", "uː"), ("aː", "ɑː"), ("ɒː", "oː"), ("ɒ", "ɔ"), ("ɐ", "a"), ("ɘ", "ə"), ("ɜ", "ə"),
       ("ɾ", "r"), ("ɹ", "r"), ("ʁ", "r"), ("ʋ", "v"), ("ʝ", "j"), ("kʰ", "k"), ("tʰ", "t"), ("z", "s"), ("ʒ", "ʃ"),
       ("x", "sx"), ("‿", ""), ("↓", ""), ("ˇ", ""), ("ˑ", ""), ("ˠ", "")]
VOW = "aɑɛeiɪoɔuʊʉɵyøœə"
OK = set("abdefhijklmnoprstuvwxyɑɛɪɔʊʉɵøœəŋɕʃʂɡːˈˌ")


def convert(p):
    out = []
    pending = False
    for x in p.split(" "):
        if x in ("¹", "²"):
            pending = True
            continue
        # tone/quality diacritics (â ě ʉ̟ œ̞ ...) and tie bars carry no phonemic information here
        x = "".join(ch for ch in unicodedata.normalize("NFD", x) if not (0x300 <= ord(ch) <= 0x36F))
        for a, b in MAP:
            x = x.replace(a, b)
        if x and x[0] in VOW and pending:
            out.append("ˈ")
            pending = False
        out.append(x)
    s = "".join(out)
    # short qualities for bare letters
    s = re.sub("e(?!ː)", "ɛ", s)
    s = re.sub("o(?!ː)", "ɔ", s)
    s = re.sub("u(?!ː)", "ʊ", s)
    s = re.sub("i(?!ː)", "ɪ", s)
    s = re.sub("ɑ(?!ː)", "a", s)
    s = re.sub("ø(?!ː)", "œ", s)
    s = re.sub(f"([^{VOW}ː])ː", r"\1", s)  # consonant length
    s = re.sub(f"([^{VOW}ː])\\1", r"\1", s)
    s = s.replace("tɕ", "ɕ")
    if "ˈ" not in s:
        nuclei = list(re.finditer(f"[{VOW}]ː?", s))
        longs = [m for m in nuclei if m.group().endswith("ː") or "ʉ" in m.group()]
        tgt = longs[0] if len(longs) == 1 else (nuclei[0] if nuclei else None)
        if tgt is None:
            return None
        s = s[: tgt.start()] + "ˈ" + s[tgt.start():]
    if any(ch not in OK for ch in s):
        return None
    return s


def main():
    seen = {}
    for line in open(sys.argv[1], encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        w = unicodedata.normalize("NFC", w).lower()
        if " " in w or not w.isalpha():
            continue
        c = convert(p)
        if c is None:
            continue
        # prefer variants that carry an accent mark and avoid the ʃ/w spellings
        score = (("¹" in p or "²" in p), "ʃ" not in c and "w" not in c)
        if w not in seen or score > seen[w][1]:
            seen[w] = (c, score)
    with open(sys.argv[2], "w", encoding="utf8") as f:
        for w, (c, _) in sorted(seen.items()):
            f.write(f"{w}\t{c}\n")
    print(len(seen), "entries", file=sys.stderr)


if __name__ == "__main__":
    main()
