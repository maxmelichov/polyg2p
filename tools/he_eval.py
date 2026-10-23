"""Evaluate the native Hebrew engine (Rust CLI) against RenikudPlus (teacher).

Two test sets, one metric (word accuracy: a predicted word counts when its IPA
equals the reference word's IPA after stripping edge punctuation; words are
aligned with difflib when the counts differ):

  * held-out teacher labels: the first --heldout rows of build-data/he_labels.jsonl
    (never trained on) -> student vs teacher agreement
  * gold: heb-tts-benchmark saspeech_100_gold_gt.csv (hand-annotated IPA) ->
    teacher and student each scored against the human gold

usage: he_eval.py --labels build-data/he_labels.jsonl --data data-out
                  [--gold /path/to/heb-tts-benchmark] [--heldout 2000] [--teacher]
--teacher also runs RenikudPlus on the gold set (cached in build-data/gold_teacher.tsv).
"""
from __future__ import annotations

import argparse
import csv
import difflib
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CARGO = os.environ.get("CARGO", os.path.expanduser("~/.cargo/bin/cargo"))
if not os.path.exists(CARGO):
    CARGO = "cargo"
PUNCT = ".,?!:;\"'()-—…«»"


def words(ipa: str) -> list[str]:
    out = []
    for w in ipa.split():
        w = w.strip(PUNCT)
        if w:
            out.append(w)
    return out


def word_acc(preds: list[str], refs: list[str]) -> tuple[int, int]:
    ok = tot = 0
    for p, r in zip(preds, refs):
        pw, rw = words(p), words(r)
        tot += len(rw)
        if len(pw) == len(rw):
            ok += sum(a == b for a, b in zip(pw, rw))
        else:
            sm = difflib.SequenceMatcher(a=rw, b=pw, autojunk=False)
            ok += sum(m.size for m in sm.get_matching_blocks())
    return ok, tot


def run_rust(texts: list[str], data: str, lang: str = "he") -> list[str]:
    exe = os.path.join(ROOT, "target", "release", "g2p")
    subprocess.run([CARGO, "build", "--release", "--bin", "g2p"], cwd=ROOT, check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    res = subprocess.run([exe, lang, "--data", data], input="\n".join(texts) + "\n",
                         capture_output=True, text=True, check=True)
    out = res.stdout.split("\n")
    if out and out[-1] == "":
        out.pop()
    assert len(out) == len(texts), (len(out), len(texts))
    return out


def run_teacher(texts: list[str], cache: str) -> list[str]:
    if os.path.exists(cache):
        rows = dict(l.rstrip("\n").split("\t", 1) for l in open(cache, encoding="utf8"))
        if all(t in rows for t in texts):
            return [rows[t] for t in texts]
    from renikud_onnx import G2P
    g = G2P()
    outs = [g.phonemize(t) for t in texts]
    with open(cache, "w", encoding="utf8") as f:
        for t, o in zip(texts, outs):
            f.write(f"{t}\t{o}\n")
    return outs


def report(name: str, preds, refs, show=0):
    ok, tot = word_acc(preds, refs)
    sent = sum(words(p) == words(r) for p, r in zip(preds, refs))
    print(f"{name}: word acc {ok}/{tot} = {ok/tot:.4f}; sentence exact {sent}/{len(refs)} = {sent/len(refs):.4f}")
    n = 0
    for p, r in zip(preds, refs):
        if words(p) != words(r) and n < show:
            n += 1
            print("   ref:", r)
            print("   got:", p)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--labels", default=os.path.join(ROOT, "build-data", "he_labels.jsonl"))
    ap.add_argument("--data", default=os.path.join(ROOT, "data-out"))
    ap.add_argument("--gold", default="/Users/maxm/Documents/heb2phonemes/heb-tts-benchmark")
    ap.add_argument("--heldout", type=int, default=2000)
    ap.add_argument("--teacher", action="store_true", help="also score RenikudPlus on the gold set")
    ap.add_argument("--show", type=int, default=0, help="print N mismatching sentences per set")
    a = ap.parse_args()

    # ---- held-out teacher agreement
    rows = []
    with open(a.labels, encoding="utf8") as f:
        for line in f:
            rows.append(json.loads(line))
            if len(rows) >= a.heldout:
                break
    texts = [r["t"] for r in rows]
    student = run_rust(texts, a.data)
    report(f"held-out ({len(rows)} sentences) student vs teacher", student, [r["ipa"] for r in rows], a.show)
    exact = sum(s == r["ipa"] for s, r in zip(student, rows))
    print(f"   byte-identical sentences: {exact}/{len(rows)}")

    # ---- gold benchmark
    gt_path = os.path.join(a.gold, "saspeech_100_gold_gt.csv")
    syn_path = os.path.join(a.gold, "saspeech_100_gold_synthesis.csv")
    if not os.path.exists(gt_path):
        print("no gold benchmark at", a.gold)
        return
    gold = {r["id"]: r["phonemes"] for r in csv.DictReader(open(gt_path, encoding="utf8"))}
    syn = {r["id"]: r["text"] for r in csv.DictReader(open(syn_path, encoding="utf8"))}
    ids = [i for i in gold if i in syn]
    g_text = [syn[i] for i in ids]
    g_ref = [gold[i] for i in ids]
    g_student = run_rust(g_text, a.data)
    report(f"gold ({len(ids)} sentences) student vs human", g_student, g_ref, a.show)
    if a.teacher:
        g_teacher = run_teacher(g_text, os.path.join(ROOT, "build-data", "gold_teacher.tsv"))
        report(f"gold ({len(ids)} sentences) teacher (RenikudPlus) vs human", g_teacher, g_ref, a.show)
        report(f"gold ({len(ids)} sentences) student vs teacher", g_student, g_teacher)


if __name__ == "__main__":
    main()
