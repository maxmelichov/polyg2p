#!/usr/bin/env python3
"""Labels for the heteronym context models (src/hetero.rs, tools/train_ctx.py).

  stats  --lang fr --sentences S.txt --suffix ent --out stats.tsv
         Stanza over every sentence that has a word ending in SUFFIX (or listed in
         --words); prints per word the tag signatures seen. Used to extend a table mechanically.
  label  --lang fr --sentences S.txt --out silver/fr.jsonl [--max-per-word 6000 --random 20000]
         training rows: {"t": encoder text, "p": [[offset, upos_id]], "h": [[offset, k, word]]}.
         `h` = teacher label (Stanza UPOS/feats/lemma mapped through the table's tag column).
  gold   --lang fr --ud A.conllu B.conllu --out gold/fr.tsv --source NAME
         eval rows from a UD treebank: sentence, word, nth occurrence, class, source.

Run with a Python that has torch + stanza (GPU optional). The Rust engine
(target/release/hetero_check) supplies the text normalisation (hetero_common.dump).
"""
from __future__ import annotations

import argparse
import collections
import glob
import json
import os
import random
import re
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from hetero_common import UPOS_ID, class_of, dump, load_table, lookup, read_conllu  # noqa: E402

WORD = re.compile(r"[^\W\d_]+(?:['’\-][^\W\d_]+)*")
PKG = {  # (pos package, lemma package, tokenize package): chosen per treebank licence, see docs/heteronyms.md
    "fr": ("gsd_charlm", "gsd_nocharlm", "gsd"),
    "de": ("combined_charlm", "combined_nocharlm", "combined"),
    "pt": ("bosque_charlm", "bosque_nocharlm", "bosque"),
    "nl": ("alpino_charlm", "alpino_nocharlm", "alpino"),
    "ro": ("rrt_nocharlm", "rrt_nocharlm", "rrt"),
    "it": ("markit_charlm", "markit_nocharlm", "markit"),
}


def pipeline(lang, gpu_frac):
    import torch
    import stanza
    use_gpu = torch.cuda.is_available()
    if use_gpu:
        torch.cuda.set_per_process_memory_fraction(gpu_frac)
    p, l, t = PKG[lang]
    return stanza.Pipeline(lang, processors="tokenize,pos,lemma", package={"pos": p, "lemma": l, "tokenize": t},
                           tokenize_pretokenized=True, dir=os.environ.get("STANZA_RESOURCES_DIR", "stanza"),
                           use_gpu=use_gpu, verbose=False)


def read_sentences(path, maxlen=300):
    with open(path, encoding="utf8") as f:
        for line in f:
            s = line.strip()
            if 8 <= len(s) <= maxlen and not any(c.isdigit() for c in s):
                yield s


def tokens_of(d):
    """Stanza input for one dumped sentence: words and punctuation characters; map token -> word index."""
    toks, widx = [], []
    for j, (_o, w, p) in enumerate(d["w"]):
        toks.append(w)
        widx.append(j)
        for c in p:
            toks.append(c)
            widx.append(-1)
    return toks, widx


def tag_all(nlp, dumps, batch=512):
    """-> per dump: list over words of (upos, feats dict, lemma)"""
    res = []
    for i in range(0, len(dumps), batch):
        chunk = dumps[i:i + batch]
        tw = [tokens_of(d) for d in chunk]
        doc = nlp([t for t, _ in tw])
        for (toks, widx), sent in zip(tw, doc.sentences):
            row = [None] * (max(widx) + 1 if widx else 0)
            for w, j in zip(sent.words, widx):
                if j >= 0:
                    f = {} if not w.feats else dict(p.split("=", 1) for p in w.feats.split("|"))
                    row[j] = (w.upos or "X", f, w.lemma or "")
            res.append(row)
    return res


def cmd_stats(a):
    nlp = pipeline(a.lang, a.gpu_frac)
    words = set(open(a.words, encoding="utf8").read().split()) if a.words else set()
    sufs = tuple(a.suffix or [])
    pick, per = [], collections.Counter()
    for s in read_sentences(a.sentences):
        ws = [w.lower() for w in WORD.findall(s)]
        hit = [w for w in ws if (w in words or (sufs and w.endswith(sufs))) and per[w] < a.max_per_word]
        if hit:
            for w in set(hit):
                per[w] += 1
            pick.append(s)
        if len(pick) >= a.limit:
            break
    ds = dump(a.lang, pick)
    tags = tag_all(nlp, ds)
    sig = collections.defaultdict(collections.Counter)
    for d, tg in zip(ds, tags):
        for j, (_o, w, _p) in enumerate(d["w"]):
            if (w in words or (sufs and w.endswith(sufs))) and tg[j]:
                u, f, l = tg[j]
                keys = [u] + [f"{k}={f[k]}" for k in a.feats if k in f]
                sig[w]["|".join(keys) + f"|lemma={l}"] += 1
    with open(a.out, "w", encoding="utf8") as o:
        for w, c in sorted(sig.items(), key=lambda x: -sum(x[1].values())):
            o.write(f"{w}\t{sum(c.values())}\t" + "\t".join(f"{k}:{v}" for k, v in c.most_common(6)) + "\n")
    print("sentences", len(pick), "words", len(sig))


def cmd_label(a):
    table = load_table(a.lang)
    nlp = pipeline(a.lang, a.gpu_frac)
    rnd = random.Random(0)
    pick, rest, per = [], [], collections.Counter()
    for s in read_sentences(a.sentences):
        ws = {w.lower() for w in WORD.findall(s)}
        hit = [w for w in ws if lookup(table, w)]
        if hit:
            if any(per[w] < a.max_per_word for w in hit):
                for w in hit:
                    per[w] += 1
                pick.append(s)
        elif rnd.random() < 0.02 and len(rest) < 4 * a.random:
            rest.append(s)
    rnd.shuffle(rest)
    rest = rest[:a.random]
    sents = pick + rest
    print("selected", len(pick), "table sentences +", len(rest), "random", flush=True)
    ds = dump(a.lang, sents)
    tags = tag_all(nlp, ds)
    n_lab = collections.Counter()
    with open(a.out, "w", encoding="utf8") as o:
        for d, tg in zip(ds, tags):
            if len(d["t"]) > 300 or not d["w"]:
                continue
            p = [[w[0], UPOS_ID.get(tg[j][0], 17)] for j, w in enumerate(d["w"]) if tg[j]]
            h = []
            for j, w, _rk in d["occ"]:
                if tg[j] is None:
                    continue
                k = class_of(lookup(table, w), *tg[j])
                if k is not None:
                    h.append([d["w"][j][0], k, w])
                    n_lab[(w, k)] += 1
            o.write(json.dumps({"t": d["t"], "p": p, "h": h}, ensure_ascii=False) + "\n")
    with open(a.out + ".counts.tsv", "w", encoding="utf8") as o:
        for (w, k), n in sorted(n_lab.items(), key=lambda x: -x[1]):
            o.write(f"{w}\t{k}\t{n}\n")
    print("labelled occurrences", sum(n_lab.values()), "word-classes", len(n_lab))


def held_out(text):
    """silver-eval sentences: never trained on (train_ctx.py skips them with the same test)"""
    return zlib.crc32(text.encode("utf8")) % 10 == 0


def cmd_silver(a):
    from hetero_silver import SILVER
    rule = SILVER[a.lang]
    table = load_table(a.lang)
    pick, per = [], collections.Counter()
    for s in read_sentences(a.sentences):
        ws = {w.lower() for w in WORD.findall(s)}
        hit = [w for w in ws if lookup(table, w)]
        if hit and any(per[w] < a.max_per_word for w in hit):
            for w in hit:
                per[w] += 1
            pick.append(s)
    ds = dump(a.lang, pick)
    n_tr, n_ev = collections.Counter(), collections.Counter()
    with open(a.out, "w", encoding="utf8") as o, open(a.out_eval, "w", encoding="utf8") as ev:
        for s, d in zip(pick, ds):
            words = [w[1] for w in d["w"]]
            h, c = [], []
            for j, w, _rk in d["occ"]:
                r = rule(words, j)
                if r is None:
                    continue
                k, cue = r
                if held_out(d["t"]):
                    ev.write(f"{s}\t{w}\t{words[:j].count(w)}\t{k}\tsilver:cue\n")
                    n_ev[(w, k)] += 1
                else:
                    h.append([d["w"][j][0], k, w])
                    a0 = d["w"][cue][0]
                    c.append([a0, a0 + len(words[cue])])
                    n_tr[(w, k)] += 1
            if h:
                o.write(json.dumps({"t": d["t"], "p": [], "h": h, "c": c}, ensure_ascii=False) + "\n")
    print("silver train", sum(n_tr.values()), "eval", sum(n_ev.values()), n_tr.most_common(10))


def cmd_gold(a):
    table = load_table(a.lang)
    rows = []
    for path in a.ud:
        src = os.path.basename(path)
        sents = read_conllu(path)
        ds = dump(a.lang, [t for t, _ in sents])
        for (text, toks), d in zip(sents, ds):
            if not d["occ"]:
                continue
            ours = [w[1] for w in d["w"]]
            for j, w, _rk in d["occ"]:
                n_ours = ours.count(w)
                ud = [t for t in toks if t["form"].lower().replace("’", "'") == w]
                if len(ud) != n_ours:
                    continue
                nth = ours[:j].count(w)
                t = ud[nth]
                if t["upos"] == "_":
                    continue
                k = class_of(lookup(table, w), t["upos"], t["feats"], t["lemma"])
                if k is not None:
                    rows.append((text, w, nth, k, src))
    with open(a.out, "w", encoding="utf8") as o:
        for r in rows:
            o.write("\t".join(map(str, r)) + "\n")
    print("gold decisions", len(rows), collections.Counter((r[1], r[3]) for r in rows).most_common(12))


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("stats", "label", "gold", "silver"):
        p = sub.add_parser(name)
        p.add_argument("--lang", required=True)
        p.add_argument("--out", required=True)
        p.add_argument("--gpu-frac", type=float, default=0.125)
        if name == "silver":
            p.add_argument("--out-eval", required=True)
        if name != "gold":
            p.add_argument("--sentences", required=True)
            p.add_argument("--max-per-word", type=int, default=6000)
        if name == "stats":
            p.add_argument("--suffix", action="append")
            p.add_argument("--words")
            p.add_argument("--limit", type=int, default=150000)
            p.add_argument("--feats", nargs="*", default=["VerbForm", "Number", "Person"])
        if name == "label":
            p.add_argument("--random", type=int, default=20000)
        if name == "gold":
            p.add_argument("--ud", nargs="+", required=True)
    a = ap.parse_args()
    {"stats": cmd_stats, "label": cmd_label, "gold": cmd_gold, "silver": cmd_silver}[a.cmd](a)


if __name__ == "__main__":
    main()
