#!/usr/bin/env python3
"""Writes data/hetero_fr.tsv: French words whose reading depends on the part of speech.
-ent: noun/adjective /ɑ̃/ vs verb 3pl (silent); -tions: plural noun /sjɔ̃/ vs 1pl verb /tjɔ̃/ (open class `*tions`);
sens, vis, lis, fier, os, and four loanwords in -er. Hand-written. `plus`, `tous`, `six`, `dix` are decided by
position in src/fr.rs and are not listed."""
import os
# words kept on the rules: the evidence floor (a word needs 30 training decisions in each class, or 15+ gold
# decisions with the model 10 points above the rules), see docs/heteronyms.md
RULE_MODE = set(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hetero_rule_fr.txt")).read().split())
rows = ["# French heteronyms: classes decided by part of speech. Hand-written; readings use the engine's own word function (respell:) or explicit phonemes (ph:).",
        "# word\tk\treading\ttag\tnote\tmode"]


def add(w, k, r, tag, note=""):
    rows.append(f"{w}\t{k}\t{r}\t{tag}\t{note}\t{'rule' if w in RULE_MODE else ''}")



ents = "couvent président résident violent parent content somnolent émergent excellent expédient équivalent négligent influent affluent convergent divergent ferment".split()
alt = {"influent": "influant", "affluent": "affluant", "émergent": "émergeant"}
VT = "VERB:VerbForm=Fin:Person=3:Number=Plur/AUX:Person=3:Number=Plur"
for w in ents:
    add(w, 0, ("respell:" + alt[w]) if w in alt else "rule", "NOUN/ADJ/ADV/PROPN", "-ent noun or adjective /ɑ̃/")
    add(w, 1, "respell:" + w[:-2], VT, "-ent verb 3pl (silent)")
add("*tions", 0, "rule", "NOUN", "plural noun -tions /sjɔ̃/")
add("*tions", 1, "sub:sj>tj", "VERB:Person=1/AUX:Person=1", "1pl imperfect or subjunctive -tions /tjɔ̃/ (kept on the rules: on running text the model turned plural nouns such as supplications and stations into the verb reading)")
add("sens", 0, "rule", "NOUN", "noun sens")
add("sens", 1, "respell:sent", "VERB", "verb sentir 1sg/2sg")
add("fils", 0, "rule", "lemma=fils", "son")
add("fils", 1, "respell:fil", "lemma=fil", "threads")
add("vis", 0, "rule", "NOUN", "screw")
add("vis", 1, "respell:vi", "VERB", "verb voir/vivre 1sg/2sg")
add("lis", 0, "rule", "NOUN", "lily")
add("lis", 1, "respell:li", "VERB/INTJ", "verb lire 1sg/2sg/imperative")
add("fier", 0, "rule", "ADJ", "proud")
add("fier", 1, "respell:fié", "VERB", "se fier (to trust)")
add("os", 0, "rule", "NOUN:Number=Sing", "bone, singular /ɔs/")
add("os", 1, "ph:ˈo", "NOUN:Number=Plur", "bones, plural /o/")
add("reporter", 0, "ph:ʁəpɔʁtˈɛʁ", "NOUN", "noun (journalist)")
add("reporter", 1, "ph:ʁəpɔʁtˈe", "VERB", "verb (postpone)")
add("supporter", 0, "ph:sypɔʁtˈɛʁ", "NOUN", "noun (fan)")
add("supporter", 1, "ph:sypɔʁtˈe", "VERB", "verb (to bear)")
add("scanner", 0, "ph:skanˈɛʁ", "NOUN", "noun")
add("scanner", 1, "ph:skanˈe", "VERB", "verb")
add("boxer", 0, "ph:bɔksˈœʁ", "NOUN", "noun (boxer)")
add("boxer", 1, "ph:bɔksˈe", "VERB", "verb")
open("data/hetero_fr.tsv", "w", encoding="utf8").write("\n".join(rows) + "\n")
