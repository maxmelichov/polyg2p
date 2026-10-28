"""Phonemize arbitrary French text with espeak-ng fr-fr, one line per argument or stdin line (notation study only)."""
import sys, unicodedata
import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper
EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
lines = sys.argv[1:] or [l.rstrip("\n") for l in sys.stdin if l.strip()]
b = EspeakBackend("fr-fr", preserve_punctuation=True, with_stress=True, language_switch="remove-flags")
res = b.phonemize(lines, separator=Separator(phone="", word=" ", syllable=""), njobs=1, strip=True)
for w, p in zip(lines, res):
    print(f"{w}\t{unicodedata.normalize('NFC', p)}")
