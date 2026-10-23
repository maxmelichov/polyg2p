"""Compare polyg2p word output with espeak's on the sampled word list."""
import os
SRC = os.environ.get("POLYG2P_SRC", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sources"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
import sys, subprocess
from diffstat import report
lang, ref_file = sys.argv[1], sys.argv[2]
top = int(sys.argv[3]) if len(sys.argv) > 3 else 30
extra = sys.argv[4:]
ref = [l.rstrip("\n").split("\t") for l in open(ref_file, encoding="utf8")]
inp = "\n".join(w for w, _ in ref) + "\n"
out = subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), lang] + extra, input=inp, capture_output=True, text=True).stdout.split("\n")
import unicodedata
report([(w, [o.rstrip('.')], unicodedata.normalize('NFD', e)) for (w, e), o in zip(ref, out)], top)
