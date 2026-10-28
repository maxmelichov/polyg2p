"""Phonemize the N most frequent Romanian words (wordfreq) with espeak-ng.

Reference only (notation study / agreement figures); nothing from it is shipped.
usage: ro_espeak_top.py N OUT.tsv
"""
import sys

import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper

EspeakWrapper.set_library(espeakng_loader.get_library_path())
EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
from wordfreq import top_n_list

n, out = int(sys.argv[1]), sys.argv[2]
words = [w for w in top_n_list("ro", n) if w.isalpha()]
b = EspeakBackend("ro", preserve_punctuation=False, with_stress=True, language_switch="remove-flags")
res = b.phonemize(words, separator=Separator(phone="", word=" ", syllable=""), njobs=2, strip=True)
with open(out, "w", encoding="utf8") as f:
    for w, p in zip(words, res):
        f.write(f"{w}\t{p}\n")
