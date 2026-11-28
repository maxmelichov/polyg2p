"""Rebuild data/uk_stress_exceptions.tsv: frequent word forms whose stress the compiled-in tables get wrong.

    python tools/uk_exceptions.py FORMS.tsv kaikki_lex.tsv target/release/g2p > data/uk_stress_exceptions.tsv

FORMS.tsv: `word<TAB>count`, the most frequent forms of a Ukrainian text (we use the top 5,000). kaikki_lex.tsv: output of
tools/uk_kaikki.py (Wiktionary's accented forms). A form is listed when Wiktionary gives exactly one stressed vowel for it and
the engine WITHOUT the lexicon pack puts the mark elsewhere. Build the binary with an empty data/uk_stress_exceptions.tsv
(header line only), or the rows already there hide the errors they fix.
"""
import subprocess, sys

V = "аеєиіїоуюя"
forms = [l.split("\t")[0] for l in open(sys.argv[1], encoding="utf8")]
lex = {}
for l in open(sys.argv[2], encoding="utf8"):
    w, v = l.rstrip("\n").split("\t")[:2]
    lex[w] = [int(x) for x in v.split(",")]
out = subprocess.run([sys.argv[3], "uk"], input="\n".join(forms) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
print("# Ukrainian word stress that the compiled-in ending/stem statistics get wrong: frequent forms of a Ukrainian text pool for which the "
      "en.wiktionary Ukrainian extract (explicit acute accents) gives exactly one stressed vowel and the statistics give another. "
      "word<TAB>stress (vowel index from the end, 0 = last). Context-free forms only: forms with two Wiktionary readings are not "
      "listed; that needs context. Read before the built-in table and the lexicon, after nothing but the homograph pins.")
print("# Source: en.wiktionary (CC BY-SA 4.0 / GFDL), accented headwords and inflected forms (kaikki.org extract).")
rows = []
for w, p in zip(forms, out):
    nv = sum(c in V for c in w)
    if nv < 2 or w not in lex or len(set(lex[w])) != 1 or lex[w][0] >= nv:
        continue
    if "ˈ" not in p:
        continue
    # vowel symbols from the mark to the end of the word = the stress index from the end + 1 (iː is one symbol, ː is skipped)
    got = sum(ch in "aeiouyɛɪ" for ch in p.split("ˈ", 1)[1]) - 1
    if got != lex[w][0]:
        rows.append(f"{w}\t{lex[w][0]}")
print("\n".join(sorted(rows)))
