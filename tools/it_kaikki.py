"""Italian stress / vowel-openness lexicon from the kaikki.org extract of en.wiktionary (explicit stress marks and e/o quality).

    curl -O https://kaikki.org/dictionary/Italian/kaikki.org-dictionary-Italian.jsonl
    python tools/it_kaikki.py kaikki.org-dictionary-Italian.jsonl it_lex_in.tsv > it_lex_final.tsv
    target/release/build_lex it_lex_final.tsv data-it/it.lex            # a folder that holds only it.lex

Every single-word headword with a /ˈ.../ transcription becomes `word<TAB>stress|open|z` in the format of it.lex
(tools/build_data.py): stress = number of vowel letters after the stressed one, open = o / c for a stressed e or o
(ɛ ɔ / e o), z = v / u for z written dz / ts. Wiktionary lists the preferred pronunciation first, so the first
transcription of the first entry that has one decides (`--policy unanimous` keeps only words whose transcriptions all agree).
it_lex_in.tsv (the WikiPron-derived entries of build_data.py) supplies every word this source does not settle.

Licence: Wiktionary text is dual-licensed CC BY-SA 4.0 / GFDL; like it.lex itself this is a data pack outside the Apache wheel.
"""
import json, os, re, sys, unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from it_stress import align, vowel_letters, VL

IPA_V = "aeiouɛɔ"


def parse(word, ipa):
    """(stress, open, z) of one transcription, or None when it has no primary stress or does not align with the spelling."""
    ipa = ipa.strip("/[]").replace("͡", "").replace("̯", "").replace("̪", "")
    if "ˈ" not in ipa or " " in ipa:
        return None
    pre, post = ipa.split("ˈ", 1)
    j = sum(c in IPA_V for c in pre)
    wl = word.lower()
    a = align(wl, pre + post)
    if a is None:
        return None
    m, _, nuc = a
    if j not in m or j >= len(nuc):
        return None
    st = len(vowel_letters(wl)) - 1 - m[j]
    op = {"ɛ": "o", "ɔ": "o", "e": "c", "o": "c"}.get(nuc[j], "-")
    z = "-"
    if "z" in wl:
        z = "-" if ("dz" in ipa) == ("ts" in ipa) else "v" if "dz" in ipa else "u"
    return st, op, z


def main():
    src, base = sys.argv[1], sys.argv[2]
    policy = sys.argv[sys.argv.index("--policy") + 1] if "--policy" in sys.argv else "first"
    cand = {}
    for line in open(src, encoding="utf8"):
        d = json.loads(line)
        w = unicodedata.normalize("NFC", d["word"])
        if not re.fullmatch(f"[A-Za-z{VL}]+", w) or w.lower() in ("a",):
            continue
        for s in d.get("sounds", []):
            ipa = s.get("ipa")
            if ipa:
                r = parse(w, ipa)
                if r:
                    cand.setdefault(w, []).append(r)
    lex = {}
    for w, rs in cand.items():
        if policy == "first":
            lex[w] = rs[0]
        elif policy == "last":
            lex[w] = rs[-1]
        elif policy == "closed":  # a closed reading anywhere wins (the first one among them)
            lex[w] = next((r for r in rs if r[1] == "c"), rs[0])
        elif len(set(rs)) == 1:
            lex[w] = rs[0]
    out = {}
    for line in open(base, encoding="utf8"):
        w, v = line.rstrip("\n").split("\t")[:2]
        out[w] = v
    for w, (st, op, z) in lex.items():
        out[w] = f"{st}|{op}|{z}"
    sys.stdout.write("".join(f"{w}\t{v}\n" for w, v in sorted(out.items())))
    print(f"{len(lex)} kaikki words ({policy}), {len(out)} merged", file=sys.stderr)


if __name__ == "__main__":
    main()
