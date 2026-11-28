"""Ukrainian stress lexicon from the kaikki.org extract of en.wiktionary (explicit stress marks).

    curl -O https://kaikki.org/dictionary/Ukrainian/kaikki.org-dictionary-Ukrainian.jsonl
    python tools/uk_kaikki.py kaikki.org-dictionary-Ukrainian.jsonl kaikki_lex.tsv [uk_lex.tsv] > uk_lex_final.tsv
    target/release/build_lex uk_lex_final.tsv data-uk/uk.lex       # a folder that holds only uk.lex

Every headword and inflected form that Wiktionary writes with a combining acute accent becomes `word<TAB>n`, n = number
of vowel letters after the stressed one (0 = final), alternatives comma-separated in the order Wiktionary lists them.
If a WikiPron-derived `uk_lex.tsv` (tools/uk_stress.py) is given, its words are appended where this source has none; where
both have a word the explicit marks win (the two agree on the first stress for 97.8% of the 41k shared words).

Licence: Wiktionary text is dual-licensed CC BY-SA 4.0 / GFDL (https://en.wiktionary.org/wiki/Wiktionary:Copyrights); the
lexicon is a data pack outside the Apache wheel, like de.lex and it.lex. dict_uk / ukrainian-word-stress data
(CC BY-NC-SA, non-commercial) is not used.
"""
import json, sys, unicodedata

ACUTE = "́"
V = "аеєиіїоуюя"


def main():
    src, out = sys.argv[1], sys.argv[2]
    lex = {}
    for line in open(src, encoding="utf8"):
        d = json.loads(line)
        for c in [d.get("word", "")] + [f.get("form", "") for f in d.get("forms", [])]:
            nf = unicodedata.normalize("NFD", c)
            if ACUTE not in nf or " " in nf or "-" in nf:
                continue
            i = nf.index(ACUTE)
            pre = unicodedata.normalize("NFC", nf[:i])  # NFC first: й is и + breve in NFD and is not a vowel
            plain = unicodedata.normalize("NFC", nf.replace(ACUTE, "")).lower()
            if not pre or pre[-1].lower() not in V:
                continue
            idx = sum(ch in V for ch in plain) - sum(ch in V for ch in pre.lower())
            if idx not in lex.setdefault(plain, []):
                lex[plain].append(idx)
    with open(out, "w", encoding="utf8") as f:
        f.write("".join(f"{w}\t{','.join(map(str, i))}\n" for w, i in sorted(lex.items())))
    merged = {w: ",".join(map(str, i)) for w, i in lex.items()}
    if len(sys.argv) > 3:
        for l in open(sys.argv[3], encoding="utf8"):
            w, v = l.rstrip("\n").split("\t")[:2]
            merged.setdefault(w, v)
    sys.stdout.write("".join(f"{w}\t{v}\n" for w, v in sorted(merged.items())))
    print(f"{len(lex)} kaikki words, {len(merged)} merged", file=sys.stderr)


if __name__ == "__main__":
    main()
