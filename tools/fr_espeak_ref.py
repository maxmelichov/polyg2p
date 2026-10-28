"""Phonemize the N most frequent French words with espeak-ng fr-fr (notation reference only; not shipped).
usage: fr_espeak_ref.py OUT [N]  -> OUT: word<TAB>espeak-ipa (NFC)"""
import sys, unicodedata
import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper
EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
from wordfreq import top_n_list
out = sys.argv[1]; n = int(sys.argv[2]) if len(sys.argv) > 2 else 20000
words = [w for w in top_n_list("fr", n * 2) if w.isalpha() or "'" in w or "-" in w][:n]
b = EspeakBackend("fr-fr", preserve_punctuation=False, with_stress=True, language_switch="remove-flags")
res = b.phonemize(words, separator=Separator(phone="", word=" ", syllable=""), njobs=4, strip=True)
with open(out, "w", encoding="utf8") as f:
    for w, p in zip(words, res):
        f.write(f"{w}\t{unicodedata.normalize('NFC', p)}\n")
