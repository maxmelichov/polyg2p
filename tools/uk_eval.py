"""Stress-aware eval_acc for Ukrainian.

eval_acc.py strips the stress marks before the per-language collapse runs, so for
Ukrainian — where the gold encodes stress only through vowel reduction — it scores
segments but not stress. This runs the same evaluation (same word list, weights,
espeak baseline, options) with `collapse_uk.STRESS_AWARE = True`, passing the raw
transcriptions to the collapse so a wrong stress counts as an error.

usage: same as eval_acc.py (e.g. uk_eval.py uk --show 40 [--data DIR])
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import collapse_uk
import eval_acc

collapse_uk.STRESS_AWARE = True
eval_acc.collapse = lambda s, lang: collapse_uk.collapse(s)

if __name__ == "__main__":
    eval_acc.main()
