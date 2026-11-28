#!/usr/bin/env python3
"""Writes data/hetero_it.tsv: Italian stress pairs. The engine honours written accents, so each reading is a
`respell:` of the accented spelling (grave on the stressed vowel). Tags are what a POS/lemma tagger gives for
each reading. Hand-written; frequent words only."""
# word: [(accented spelling, tag), (accented spelling, tag)]
P = {
 "ancora": [("rule", "ADV"), ("ph:ˈaŋkora", "NOUN")],
 "principi": [("princìpi", "lemma=principio"), ("prìncipi", "lemma=principe")],
 "subito": [("sùbito", "ADV"), ("subìto", "VERB")],
 "capitano": [("capitàno", "NOUN"), ("càpitano", "VERB")],
 "tendine": [("tendìne", "lemma=tendina"), ("tèndine", "lemma=tendine")],
 "ambito": [("àmbito", "NOUN"), ("ambìto", "VERB/ADJ")],
 "compito": [("còmpito", "NOUN"), ("compìto", "VERB/ADJ")],
 "desideri": [("desidèri", "NOUN"), ("desìderi", "VERB")],
 "benefici": [("benèfici", "ADJ"), ("benefìci", "NOUN")],
 "leggere": [("lèggere", "VERB"), ("leggère", "ADJ")],
 "perdono": [("pèrdono", "VERB"), ("perdóno", "NOUN")],
 "seguito": [("sèguito", "NOUN"), ("seguìto", "VERB")],
 "viola": [("ph:vˈiola", "VERB"), ("ph:vjˈɔla", "NOUN/ADJ")],
 "circuito": [("circùito", "NOUN"), ("circuìto", "VERB")],
 "intuito": [("intùito", "NOUN"), ("intuìto", "VERB")],
 "capito": [("càpito", "lemma=capitare"), ("capìto", "lemma=capire")],
 "arbitri": [("àrbitri", "lemma=arbitro"), ("arbìtri", "lemma=arbitrio")],
 "condomini": [("condòmini", "lemma=condomino"), ("condomìni", "lemma=condominio")],
 "pagano": [("pàgano", "VERB"), ("pagàno", "NOUN/ADJ")],
 "impari": [("ìmpari", "ADJ"), ("impàri", "VERB")],
 "martiri": [("màrtiri", "lemma=martire"), ("martìri", "lemma=martirio")],
 "nettare": [("nèttare", "NOUN"), ("nettàre", "VERB")],
 "presidi": [("prèsidi", "lemma=preside"), ("presìdi", "lemma=presidio")],
 "decade": [("dècade", "NOUN"), ("decàde", "VERB")],
}
rows = ["# Italian stress pairs, decided by part of speech or lemma. Readings: the engine's own output for the accented spelling.",
        "# word\tk\treading\ttag\tnote\tmode"]
# words kept on the rules: the gate (50+ human-gold decisions with the model below the rules) and the evidence floor
# (a word needs 30 training decisions in each class, or 15+ gold decisions with the model 10 points above the rules);
# the list is tools/hetero_rule_it.txt
RULE_MODE = set(open(__import__("os").path.join(__import__("os").path.dirname(__import__("os").path.abspath(__file__)), "hetero_rule_it.txt")).read().split())
for w, rs in P.items():
    for k, (sp, tag) in enumerate(rs):
        reading = sp if sp == "rule" or sp.startswith("ph:") else f"respell:{sp}"
        rows.append(f"{w}\t{k}\t{reading}\t{tag}\t{sp}\t{'rule' if w in RULE_MODE else ''}")
open("data/hetero_it.tsv", "w", encoding="utf8").write("\n".join(rows) + "\n")
