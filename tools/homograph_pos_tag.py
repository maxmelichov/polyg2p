"""POS tags of the target tokens (run with a Python that has spaCy + en_core_web_sm; used as a keep/drop flag only).
usage: homograph_pos_tag.py IN.jsonl OUT.tsv   (IN: records with s, a, b byte offsets; OUT: one line per record, the fine tag or '-')"""
import json, sys, spacy
nlp = spacy.load("en_core_web_sm", disable=["ner", "parser", "lemmatizer"])
recs = [json.loads(l) for l in open(sys.argv[1], encoding="utf8")]
out = open(sys.argv[2], "w")
for rec, doc in zip(recs, nlp.pipe((r["s"] for r in recs), batch_size=512, n_process=4)):
    a = len(rec["s"].encode("utf8")[:rec["a"]].decode("utf8"))
    tag = "-"
    for t in doc:
        if t.idx <= a < t.idx + len(t.text):
            tag = t.tag_; break
        if t.idx > a:
            break
    out.write(tag + "\n")
