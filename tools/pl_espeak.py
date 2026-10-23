"""Phonemize Polish words with espeak-ng (notation reference only; not shipped).

usage: pl_espeak.py N        top-N wordfreq words
       pl_espeak.py -        words from stdin
Writes "word<TAB>espeak" lines to stdout.
"""
import sys
import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper
EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
from wordfreq import top_n_list

if sys.argv[1] == '-':
    words = [w.strip() for w in sys.stdin if w.strip()]
else:
    words = top_n_list('pl', int(sys.argv[1]))
b = EspeakBackend('pl', with_stress=True, language_switch='remove-flags', preserve_punctuation=False)
res = b.phonemize(words, separator=Separator(phone='', word=' '), strip=True, njobs=2)
for w, p in zip(words, res):
    print(f"{w}\t{p}")
