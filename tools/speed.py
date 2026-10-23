"""Throughput: polyg2p CLI vs espeak (phonemizer, as BlueTTS calls it), single core."""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import sys, time, subprocess, os
import espeakng_loader
from phonemizer.backend.espeak.wrapper import EspeakWrapper
EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
from phonemizer.backend import EspeakBackend
from phonemizer.separator import Separator
EL = {"en": "en-us", "es": "es", "it": "it", "de": "de"}
DATA = os.environ.get("POLYG2P_DATA", os.path.join(ROOT, "data-out"))
for lang in sys.argv[1:]:
    lines = [l.strip() for l in open(os.path.join(SRC, "ud", f"{lang}.txt")) if l.strip()]
    words = sum(len(l.split()) for l in lines)
    reps = int(os.environ.get("REPS", "5"))
    inp = "\n".join(lines * reps) + "\n"
    t = time.perf_counter()
    subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), lang, "--data", DATA], input=inp, capture_output=True, text=True, check=True)
    ours = time.perf_counter() - t
    b = EspeakBackend(EL[lang], preserve_punctuation=True, with_stress=True, language_switch="remove-flags")
    sep = Separator(phone="", word=" ", syllable="")
    t = time.perf_counter()
    for l in lines:  # per-sentence calls, like a TTS request path
        b.phonemize([l], separator=sep)
    esp_lat = (time.perf_counter() - t) / len(lines)
    t = time.perf_counter()
    b.phonemize(lines, separator=sep, njobs=1)
    esp_batch = time.perf_counter() - t
    n = len(lines)
    print(f"{lang}: {n} sents/{words} words | polyg2p {n*reps/ours:,.0f} sent/s ({words*reps/ours:,.0f} w/s, incl. process start) | "
          f"espeak batch {n/esp_batch:,.0f} sent/s | espeak per-call {esp_lat*1000:.2f} ms/sent")
