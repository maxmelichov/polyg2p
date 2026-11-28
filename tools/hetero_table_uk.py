#!/usr/bin/env python3
"""Writes data/hetero_uk.tsv: Ukrainian stress homographs (lexical, grammatical-form and aspect pairs).
Each entry lists the readings as spelled with an acute accent after the stressed vowel (hand-written);
the phonemes are the engine's own output for the plain spelling with the stress mark moved to the
stressed vowel (src/uk.rs has no stress-input entry point and is not edited). The engine writes the
stress mark directly before the stressed vowel and has no stress-dependent vowel changes except that mark.
Classes: 0 = first reading, 1 = second. `tag` is `-`: there is no permissive teacher for Ukrainian,
labels come from tools/hetero_silver.py cue rules and hand-labelled sentences."""
import subprocess, sys, unicodedata

BIN = sys.argv[1] if len(sys.argv) > 1 else "target/release/g2p"
V = "аеєиіїоуюя"
PH_V = ["iː", "a", "o", "u", "ɛ", "ɪ"]
# plain word: (reading 0, reading 1, note)
PAIRS = [
    ("замок", "за́мок", "замо́к", "castle / lock"),
    ("мука", "му́ка", "мука́", "torment / flour"),
    ("атлас", "а́тлас", "атла́с", "atlas / satin"),
    ("орган", "о́рган", "орга́н", "organ (body, institution) / musical organ"),
    ("дорога", "доро́га", "дорога́", "road / dear (short fem. adjective)"),
    ("брати", "бра́ти", "брати́", "to take / brothers"),
    ("плачу", "пла́чу", "плачу́", "I cry / I pay"),
    ("мала", "ма́ла", "мала́", "had / small (fem.)"),
    ("руки", "ру́ки", "руки́", "hands (pl.) / of the hand (gen. sg.)"),
    ("ноги", "но́ги", "ноги́", "legs (pl.) / of the leg"),
    ("книжки", "кни́жки", "книжки́", "books (pl.) / of the book"),
    ("сестри", "се́стри", "сестри́", "sisters (pl.) / of the sister"),
    ("гори", "го́ри", "гори́", "mountains (pl.) / burn! (imperative) / of the mountain"),
    ("води", "во́ди", "води́", "waters (pl.) / of the water"),
    ("голови", "го́лови", "голови́", "heads (pl.) / of the head"),
    ("слова", "слова́", "сло́ва", "words (pl.) / of the word"),
    ("міста", "міста́", "мі́ста", "cities (pl.) / of the city"),
    ("поля", "поля́", "по́ля", "fields (pl.) / of the field"),
    ("моря", "моря́", "мо́ря", "seas (pl.) / of the sea"),
    ("викидати", "викида́ти", "ви́кидати", "to throw out (imperf.) / (perf.)"),
    ("насипати", "насипа́ти", "наси́пати", "to pour (imperf.) / (perf.)"),
    ("розрізати", "розріза́ти", "розрі́зати", "to cut (imperf.) / (perf.)"),
    ("скликати", "склика́ти", "скли́кати", "to call together (imperf.) / (perf.)"),
    ("пізнаю", "пізна́ю", "пізнаю́", "I will recognize (perf.) / I recognize (imperf.)"),
    ("узнаю", "узна́ю", "узнаю́", "I will find out / I recognize"),
]


def stress_idx(spelled):
    d = unicodedata.normalize("NFD", spelled)
    i = d.index("́")
    return sum(1 for c in d[:i] if c in V) - 1


def move(ph, idx):
    ph = ph.replace("ˈ", "")
    out, n, k = "", 0, 0
    while k < len(ph):
        for v in ("iː",) + tuple(x for x in PH_V if x != "iː"):
            if ph.startswith(v, k):
                if n == idx:
                    out += "ˈ"
                out += v
                n += 1
                k += len(v)
                break
        else:
            out += ph[k]
            k += 1
    return out


words = [p[0] for p in PAIRS]
ph = subprocess.run([BIN, "uk", "--words"], input="\n".join(words) + "\n", capture_output=True, text=True).stdout.split("\n")
rows = ["# Ukrainian stress homographs. Readings: engine phonemes of the plain spelling with the stress mark moved (tools/hetero_table_uk.py).",
        "# word\tk\treading\ttag\tnote\tmode"]
for (w, a, b, note), rule in zip(PAIRS, ph):
    for k, sp in enumerate((a, b)):
        rows.append(f"{w}\t{k}\tph:{move(rule, stress_idx(sp))}\t-\t{unicodedata.normalize('NFC', sp)}  {note}\t")
open("data/hetero_uk.tsv", "w", encoding="utf8").write("\n".join(rows) + "\n")
print("\n".join(rows[:12]))
