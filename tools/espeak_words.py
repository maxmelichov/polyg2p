"""Phonemize lexicon headwords with espeak (baseline/notation reference only; not shipped)."""
import sys, re, random
import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper
EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
from text_vocab import normalize_text
lang, inp, out, n = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
words = []
seen = set()
for line in open(inp, encoding='utf8'):
    w = line.split('\t')[0].strip()
    if ' ' in w or not w or w in seen: continue
    seen.add(w); words.append(w)
random.seed(0); random.shuffle(words); words = words[:n]
b = EspeakBackend(lang, preserve_punctuation=False, with_stress=True, language_switch='remove-flags')
res = b.phonemize(words, separator=Separator(phone='', word=' ', syllable=''), njobs=2, strip=True)
with open(out, 'w', encoding='utf8') as f:
    for w, p in zip(words, res):
        f.write(f"{w}\t{normalize_text(p).rstrip('.')}\n")
