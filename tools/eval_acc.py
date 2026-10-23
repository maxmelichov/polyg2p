"""Word accuracy of polyg2p vs espeak-ng against Wiktionary (WikiPron) pronunciations.

Both systems' outputs and the gold variants are collapsed to a shared broad phone
set so notation choices (flaps, length marks, glide spelling, stress marks —
WikiPron has none) don't count, only real pronunciation differences.

usage: eval_acc.py LANG [--data DIR] [--top N] [--words FILE] [--label NAME]
"""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import argparse, collections, os, re, subprocess, sys, unicodedata

import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper
EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
from wordfreq import word_frequency, top_n_list

GOLD = {"en": "eng_latn_us_broad.tsv", "es": "spa_latn_ca_broad.tsv", "it": "ita_latn_broad.tsv", "de": "deu_latn_broad.tsv"}
ESPEAK = {"en": "en-us", "es": "es", "it": "it", "de": "de"}
CLI = os.path.join(ROOT, "target", "release", "g2p")

COMMON = [("t͡s", "ts"), ("t͡ʃ", "tʃ"), ("d͡ʒ", "dʒ"), ("d͡z", "dz"), ("ʦ", "ts"), ("ʧ", "tʃ"), ("ʤ", "dʒ"), ("ʣ", "dz"),
          ("͡", ""), ("g", "ɡ"), ("ˈ", ""), ("ˌ", ""), (".", ""), (" ", ""), ("‿", ""), ("ʰ", ""), ("ʲ", "ʲ")]
SYLL = [("l̩", "əl"), ("n̩", "ən"), ("m̩", "əm"), ("ɹ̩", "əɹ")]


LENIENT = False


def collapse(s, lang):
    s = unicodedata.normalize("NFC", s.strip())
    for a, b in [("ʤː", "ddʒ"), ("ʧː", "ttʃ"), ("ʦː", "tts"), ("ʣː", "ddz"), ("t͡sː", "tts"), ("d͡zː", "ddz"),
                 ("t͡ʃː", "ttʃ"), ("d͡ʒː", "ddʒ"), ("tsː", "tts"), ("dzː", "ddz"), ("tʃː", "ttʃ"), ("dʒː", "ddʒ")]:
        s = s.replace(a, b)
    for a, b in SYLL:
        s = s.replace(a, b)
    for a, b in COMMON:
        s = s.replace(a, b)
    s = s.replace("̯", "").replace("̩", "").replace("̃", "")
    for mark in ("̟", "̝", "̞", "̠", "̪", "̥", "̬", "̹", "̜", "̈", "ʷ"):
        s = s.replace(mark, "")
    if lang == "en":
        s = s.replace("ɜː", "əɹ").replace("ɜ", "əɹ").replace("ɝ", "əɹ").replace("ɚ", "əɹ")
        s = s.replace("ː", "")
        for a, b in [("ɾ", "t"), ("ʔ", "t"), ("ᵻ", "ɪ"), ("ɐ", "ə"), ("ʌ", "ə"), ("r", "ɹ"), ("ɫ", "l"), ("ɒ", "ɑ"),
                     ("oʊ", "o"), ("eɪ", "e"), ("əʊ", "o")]:
            s = s.replace(a, b)
        for a, b in [("ɛəɹ", "ɛɹ"), ("ɪəɹ", "ɪɹ"), ("iəɹ", "ɪɹ"), ("iɹ", "ɪɹ"), ("ʊəɹ", "ʊɹ"), ("ɑəɹ", "ɑɹ"),
                     ("ʍ", "w"), ("eɹ", "ɛɹ"), ("aɪəɹ", "aɪɹ"), ("aʊəɹ", "aʊɹ")]:
            s = s.replace(a, b)
        s = s.replace("ɔɹ", "oɹ")
        s = re.sub("ɹ+", "ɹ", s)
        if LENIENT:
            s = s.replace("ɪ", "ə")
    elif lang == "es":
        s = s.replace("ː", "")
        for a, b in [("β", "b"), ("ð", "d"), ("ɣ", "ɡ"), ("ɟʝ", "j"), ("ʝ", "j"), ("ɛ", "e"), ("ɔ", "o"), ("ɪ", "i"),
                     ("ʊ", "u"), ("ŋ", "n"), ("ɱ", "m"), ("χ", "x"), ("j", "i"), ("w", "u")]:
            s = s.replace(a, b)
        s = re.sub(r"ɡ(?=[ptkθsfx])", "k", s)  # allophonic voicing before voiceless consonants
    elif lang == "it":
        s = re.sub(r"(.)ː", r"\1\1", s)
        for a, b in [("ɪ", "i"), ("ʊ", "u"), ("ɾ", "r"), ("ŋ", "n"), ("j", "i"), ("w", "u"), ("z", "s")]:
            s = s.replace(a, b)
        for a, b in [("ʎʎ", "ʎ"), ("ɲɲ", "ɲ"), ("ʃʃ", "ʃ"), ("ttʃ", "tʃ"), ("ddʒ", "dʒ"), ("tts", "ts"), ("ddz", "dz"),
                     ("dds", "ds")]:
            s = s.replace(a, b)
    elif lang == "de":
        s = s.replace("ʔ", "")
        for a, b in [("ʁ", "r"), ("ʀ", "r"), ("ɾ", "r"), ("χ", "x"), ("ʋ", "v"), ("ɔɪ", "ɔʏ"), ("ɔø", "ɔʏ"), ("ɑ", "a"),
                     ("ɜ", "ɐ")]:
            s = s.replace(a, b)
        s = s.replace("ː", "")
        s = s.replace("ˑ", "").replace("ɐ", "ər")
        s = re.sub(r"([aeiouyøɛɪʊʏœɔ])ər", r"\1r", s)
        s = re.sub(r"ɪ(?=[aeiouyøɛʊʏœɔə])", "i", s)  # espeak writes ɪ before a vowel
        s = re.sub(r"[eɛ]r(?=[^aeiouyøɛɪʊʏœɔə]|$)", "ər", s)  # vocalized r after e/ɛ
        s = s.replace("ən", "n").replace("əl", "l").replace("əm", "m")
    return s


def edit(a, b):
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def load_gold(lang):
    g = collections.defaultdict(set)
    for line in open(os.path.join(SRC, GOLD[lang]), encoding="utf8"):
        w, p = line.rstrip("\n").split("\t")
        g[unicodedata.normalize("NFC", w)].add(p)
    return g


def ours(words, lang, data):
    cmd = [CLI, lang] + (["--data", data] if data else [])
    out = subprocess.run(cmd, input="\n".join(words) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
    return [o.rstrip(".,!?;:") for o in out[: len(words)]]


def espeak(words, lang):
    b = EspeakBackend(ESPEAK[lang], with_stress=True, language_switch="remove-flags")
    return b.phonemize(words, separator=Separator(phone="", word=" "), strip=True, njobs=2)


def score(words, preds, gold, lang, weights):
    tot = ok = 0.0
    per_e = per_n = 0
    for w, p in zip(words, preds):
        cands = {collapse(x, lang) for x in gold[w]}
        cp = collapse(p, lang)
        wt = weights[w]
        tot += wt
        if cp in cands:
            ok += wt
            best = 0
        else:
            best = min(edit(cp, c) for c in cands)
        per_e += best
        per_n += min(len(c) for c in cands)
    return ok / tot, per_e / max(per_n, 1)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("lang")
    ap.add_argument("--data", default=None)
    ap.add_argument("--top", type=int, default=20000)
    ap.add_argument("--words", default=None, help="file of words to evaluate (uniform weights)")
    ap.add_argument("--label", default="")
    ap.add_argument("--no-espeak", action="store_true")
    ap.add_argument("--show", type=int, default=0)
    ap.add_argument("--lenient", action="store_true", help="en: treat unstressed ɪ/ə as equal")
    a = ap.parse_args()
    global LENIENT
    LENIENT = a.lenient
    lang = a.lang
    gold = load_gold(lang)
    if a.words:
        words = [w.strip() for w in open(a.words, encoding="utf8") if w.strip() in gold]
        weights = {w: 1.0 for w in words}
        kind = f"{len(words)} listed words (uniform)"
    else:
        cand = top_n_list(lang, a.top * 3)
        words = []
        for w in cand:
            # WikiPron keys are case-sensitive (German nouns); try both
            for k in (w, w.capitalize()):
                if k in gold:
                    words.append(k)
                    break
            if len(words) >= a.top:
                break
        weights = {w: word_frequency(w.lower(), lang) for w in words}
        kind = f"top {len(words)} frequent words (frequency-weighted)"
    po = ours(words, lang, a.data)
    acc, per = score(words, po, gold, lang, weights)
    print(f"[{lang}] {kind} {a.label}")
    print(f"  polyg2p : word acc {acc:.1%}   PER {per:.1%}")
    if not a.no_espeak:
        pe = espeak(words, lang)
        acc_e, per_e = score(words, pe, gold, lang, weights)
        print(f"  espeak  : word acc {acc_e:.1%}   PER {per_e:.1%}")
    if a.show:
        n = 0
        for w, p in zip(words, po):
            cands = {collapse(x, lang) for x in gold[w]}
            if collapse(p, lang) not in cands:
                print("   ", w, collapse(p, lang), sorted(cands)[:3])
                n += 1
                if n >= a.show:
                    break


if __name__ == "__main__":
    main()
