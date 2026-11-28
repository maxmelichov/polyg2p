"""Shared helpers for the heteronym context models (src/hetero.rs, tools/hetero_*.py).

The Rust engine is the single source of the sentence normalisation: `dump()`
runs `target/release/hetero_check <lang> --dump`, which prints the encoder input
text, the engine's words with character offsets and the table words with the
class the rules produced. Python never re-implements the tokenizer.
"""
from __future__ import annotations

import json
import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "target", "release", "hetero_check")
UPOS = ["ADJ", "ADP", "ADV", "AUX", "CCONJ", "DET", "INTJ", "NOUN", "NUM", "PART", "PRON", "PROPN", "PUNCT",
        "SCONJ", "SYM", "VERB", "X"]
UPOS_ID = {u: i for i, u in enumerate(UPOS)}  # 17 = unknown


def load_table(lang, root=ROOT):
    """word -> list of dict(k, reading, tag, note, mode)"""
    t = {}
    with open(os.path.join(root, "data", f"hetero_{lang}.tsv"), encoding="utf8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            c = line.split("\t") + [""] * 6
            t.setdefault(c[0], []).append(dict(k=int(c[1]), reading=c[2], tag=c[3], note=c[4], mode=c[5]))
    return t


def lookup(table, w):
    """rows for a word: exact entry, else a `*suffix` entry (same rule as src/hetero.rs)"""
    r = table.get(w)
    if r is not None:
        return r
    for k, rows in table.items():
        if k.startswith("*") and len(w) > len(k) + 1 and w.endswith(k[1:]):
            return rows
    return None


def tag_match(tag, upos, feats, lemma):
    """tag = alternatives separated by '/', each `UPOS[:Feat=Val[:Feat=Val]]` or `lemma=xxx`; '-' never matches."""
    if not tag or tag == "-":
        return False
    for alt in tag.split("/"):
        parts = alt.split(":")
        head = parts[0]
        if head.startswith("lemma="):
            if lemma != head[6:]:
                continue
            ok = all(feats.get(p.split("=")[0]) == p.split("=")[1] for p in parts[1:])
        else:
            if head != upos:
                continue
            ok = True
            for p in parts[1:]:
                k, v = p.split("=")
                if v.startswith("!"):
                    ok &= feats.get(k) != v[1:]
                else:
                    ok &= feats.get(k) == v
        if ok:
            return True
    return False


def class_of(rows, upos, feats, lemma):
    for r in rows:
        if tag_match(r["tag"], upos, feats, lemma):
            return r["k"]
    return None


def dump(lang, sentences, data=None, chunk=20000):
    """-> list of dict(t, w=[[off, word, punct]], occ=[[word_idx, word, rule_k]])"""
    out = []
    cmd = [BIN, lang] + (["--data", data] if data else []) + ["--dump"]
    for i in range(0, len(sentences), chunk):
        inp = "\n".join(s.replace("\n", " ").replace("\t", " ") for s in sentences[i:i + chunk]) + "\n"
        r = subprocess.run(cmd, input=inp, capture_output=True, text=True, check=True)
        lines = r.stdout.split("\n")
        if lines and lines[-1] == "":
            lines.pop()
        assert len(lines) == len(sentences[i:i + chunk]), (len(lines), len(sentences[i:i + chunk]))
        out.extend(json.loads(l) for l in lines)
    return out


def read_conllu(path):
    """-> list of (text, tokens) with tokens = list of dict(form, lemma, upos, feats); multiword
    tokens appear as one surface token (upos '_'), their parts are skipped."""
    sents, toks, text = [], [], None
    with open(path, encoding="utf8") as f:
        for line in f:
            line = line.rstrip("\n")
            if line.startswith("# text = "):
                text = line[9:]
            elif not line:
                if toks and text:
                    sents.append((text, toks))
                toks, text = [], None
            elif line[0] != "#":
                c = line.split("\t")
                if "." in c[0]:
                    continue
                if "-" in c[0]:
                    a, b = c[0].split("-")
                    toks.append(dict(form=c[1], lemma="_", upos="_", feats={}, span=(int(a), int(b))))
                    continue
                i = int(c[0])
                if toks and toks[-1].get("span") and toks[-1]["span"][0] <= i <= toks[-1]["span"][1]:
                    continue
                feats = {} if c[5] == "_" else dict(p.split("=", 1) for p in c[5].split("|"))
                toks.append(dict(form=c[1], lemma=c[2], upos=c[3], feats=feats))
    return sents
