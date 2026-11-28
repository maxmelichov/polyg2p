"""Silver training examples for the English heteronym model (tools/train_homograph.py).

No labels from a corpus of heteronyms: ordinary sentences with one word swapped for a
heteronym whose reading the swap decides. Three recipes (plan section 4):

  A  slot substitution for part-of-speech heteronyms. WordNet lemmas that occur in exactly
     one of index.noun / index.verb / index.adj give forms of known class (noun singular and
     plural, verb base / 3sg / past, adjective); a sentence token that is such a form is
     replaced by a table word with a reading of that class ("They will [bring->record] it":
     verb; "He bought a [violin->record]": noun). Semantically odd, syntactically right.
  B  word-class substitution for lexical heteronyms (lead/metal <- iron, wind/air <- breeze,
     bass/fish <- trout, ...): author-written lists below.
  C  "lead" written for "led": he / she / it + led|took|brought|drove|guided -> lead (lɛd).

Sources (not stored in the repository): Tatoeba English sentences (CC BY 2.0 FR) and the
WordNet 3.1 database files (WordNet licence). Nothing from the Wikipedia Homograph Data.

usage: homograph_silver.py TATOEBA.tsv[.bz2] WORDNET_DICT_DIR TABLE.tsv OUT.jsonl [--per-slot N] [--seed S]
Each output line: {"s": sentence, "a": byte start, "b": byte end, "row": table row, "kind": "A|B|C", "src": replaced token}
"""
import bz2, collections, json, os, random, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
WORD = re.compile(r"[A-Za-z]+(?:'[a-z]+)?")

# ---- recipe B: reading -> source words that stand in for it (author-written)
B_RULES = {
    ("lead", "metal"): "iron copper zinc mercury tin arsenic nickel",
    ("read", "past"): None,      # handled with context cues (recipe A, see READ_*)
    ("wind", "air"): "breeze storm gale",
    ("wind", "turn"): "twist coil wrap",
    ("tear", "cry"): "teardrop sob",
    ("tear", "rip"): "rip split",
    ("bass", "fish"): "trout salmon perch",
    ("bass", "music"): "guitar tenor baritone drums",
    ("bow", "bend"): "nod kneel",
    ("bow", "knot"): "arrow ribbon violin",
    ("live", "adj"): "living recorded",
    ("live", "verb"): "reside dwell stay",
    ("close", "adj"): "near tight",
    ("close", "verb"): "shut end",
    ("wound", "injury"): "injury injure",
    ("wound", "turned"): "coiled wrapped twisted",
    ("row", "line"): "line column queue",
    ("row", "quarrel"): "quarrel argument",
    ("minute", "time"): "second hour moment",
    ("minute", "tiny"): "tiny minuscule",
    ("dove", "bird"): "pigeon sparrow",
    ("dove", "dived"): "jumped plunged dived",
    ("sow", "pig"): "pig hog",
    ("sow", "plant"): "plant scatter",
    ("polish", "nationality"): "Czech German French",
    ("polish", "wax"): "wax shine varnish",
}
C_SUBJ = {"he", "she", "it"}
C_VERBS = {"led", "took", "brought", "drove", "guided"}
# "read": the tense is only certain with a cue (a bare "they read" is both)
READ_PAST_PREV = {"he", "she", "it", "had", "has", "have", "was", "were"}
READ_BASE_PREV = {"will", "would", "can", "could", "shall", "should", "may", "might", "must", "to", "do", "does", "please", "let's", "didn't", "don't", "doesn't"}
# slots of recipe A not used for a word because another recipe owns it
S_WORDS = {"abuses", "analyses", "axes", "confines", "diagnoses", "lives", "uses", "winds"}
A_SKIP = {("lead", "PAST"), ("read", "PAST"), ("read", "V")}


def read_table(path):
    rows = []
    for l in open(path, encoding="utf8"):
        if l.startswith("#") or not l.strip():
            continue
        w, r, c, ipa = l.rstrip("\n").split("\t")
        rows.append((w, r, set(c.split(",")), ipa))
    return rows


def wordnet_forms(d):
    """class -> {form}: noun sg 'N', noun pl 'NPL', verb base 'V', 3sg 'V3', past 'VP', adjective 'A'."""
    def lemmas(p):
        s = set()
        for l in open(os.path.join(d, p), encoding="latin-1"):
            if l.startswith(" "):
                continue
            w = l.split()[0]
            if w.isalpha() and w.islower():
                s.add(w)
        return s
    noun, verb, adj = lemmas("index.noun"), lemmas("index.verb"), lemmas("index.adj")
    only = lambda a, *o: {w for w in a if not any(w in x for x in o) and len(w) >= 3}
    n1, v1, a1 = only(noun, verb, adj), only(verb, noun, adj), only(adj, noun, verb)
    # participial adjectives (wanted, closed, interesting) are verb forms in context
    a1 = {w for w in a1 if not w.endswith(("ed", "ing"))}
    def exc(p):
        m = collections.defaultdict(list)
        for l in open(os.path.join(d, p), encoding="latin-1"):
            f = l.split()
            if len(f) >= 2:
                m[f[1]].append(f[0])
        return m
    nexc, vexc = exc("noun.exc"), exc("verb.exc")
    def plural(w):
        if w in nexc:
            return list(nexc[w])
        if re.search(r"(s|x|z|ch|sh)$", w):
            return [w + "es"]
        if re.search(r"[^aeiou]y$", w):
            return [w[:-1] + "ies"]
        return [w + "s"]
    def third(w):
        if w in vexc and any(f.endswith("s") for f in vexc[w]):
            return [f for f in vexc[w] if f.endswith("s")]
        return plural(w) if w not in nexc else [w + "es" if re.search(r"(s|x|z|ch|sh|o)$", w) else w + "s"]
    def past(w):
        irr = [f for f in vexc.get(w, []) if not f.endswith(("s", "ing"))]
        if irr:
            return irr
        if w.endswith("e"):
            return [w + "d"]
        if re.search(r"[^aeiou]y$", w):
            return [w[:-1] + "ied"]
        return [w + "ed"]
    forms = {"N": set(n1), "A": set(a1), "V": set(v1)}
    forms["NPL"] = {p for w in n1 for p in plural(w)}
    forms["V3"] = {p for w in v1 for p in third(w)}
    forms["VP"] = {p for w in v1 for p in past(w)}
    # a form that belongs to two classes (leaves = leaf + leave) tells nothing
    count = collections.Counter(f for s in forms.values() for f in s)
    allw = noun | verb | adj
    own = {"N": n1, "A": a1, "V": v1}
    # keep a form only if it is unambiguous and not itself a lemma of some other part of speech
    return {c: {f for f in s if count[f] == 1 and (f not in allw or f in own.get(c, ()))} for c, s in forms.items()}


def load_sentences(path, seed):
    op = bz2.open if path.endswith(".bz2") else open
    out = []
    with op(path, "rt", encoding="utf8") as f:
        for l in f:
            p = l.rstrip("\n").split("\t")
            if len(p) < 3:
                continue
            s = p[2].strip()
            n = len(s.split())
            if 4 <= n <= 25 and s.isascii() and s[0].isupper():
                out.append(s)
    return out


def swap(s, a, b, new):
    """Replace bytes-equal span [a:b) of the ASCII sentence by `new` (case of the old token kept); returns (sentence, byte start, byte end)."""
    old = s[a:b]
    if old[:1].isupper() and not new[:1].isupper():
        new = new[:1].upper() + new[1:]
    elif old[:1].islower() and new[:1].isupper() and a > 0:
        pass  # a capitalised heteronym (Polish) stays capitalised in running text
    return s[:a] + new + s[b:], a, a + len(new)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opt = lambda k, d: type(d)(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d
    per, seed = opt("--per-slot", 150), opt("--seed", 0)
    tat, wn, table_path, out_path = args[:4]
    rnd = random.Random(seed)
    table = read_table(table_path)
    index = {(w, r): i for i, (w, r, c, ipa) in enumerate(table)}
    words = {w for w, *_ in table}
    func = {l.split("\t")[0] for l in open(os.path.join(HERE, "..", "data", "en_func.tsv"), encoding="utf8")}
    forms = wordnet_forms(wn)
    drop = words | func
    for c in forms:
        forms[c] -= drop
    print({c: len(s) for c, s in forms.items()}, file=sys.stderr)
    sents = load_sentences(tat, seed)
    rnd.shuffle(sents)
    print(len(sents), "sentences", file=sys.stderr)
    # one pass: every slot token by class, and the B / C / read hits
    slots = collections.defaultdict(list)          # class -> [(sent, a, b, form)]
    byword = collections.defaultdict(list)         # source word of B -> [(sent, a, b)]
    b_src = {w for v in B_RULES.values() if v for w in v.lower().split()}
    c_hits, read_hits = [], {"past": [], "base": []}
    for si, s in enumerate(sents):
        toks = [(m.group(), m.start(), m.end()) for m in WORD.finditer(s)]
        low = [t[0].lower() for t in toks]
        for k, (t, a, b) in enumerate(toks):
            if t.islower():
                for c, fs in forms.items():
                    if t in fs:
                        slots[c].append((si, a, b, t))
                        break
            if low[k] in b_src and (t.islower() or k > 0 and t[0].isupper() and low[k] in ("czech", "german", "french")):
                byword[low[k]].append((si, a, b))
            if k > 0 and low[k - 1] in C_SUBJ and low[k] in C_VERBS and t.islower():
                c_hits.append((si, a, b))
    out, kinds = [], collections.Counter()
    def add(si, a, b, row, kind, src):
        s2, a2, b2 = swap(sents[si], a, b, table[row][0])
        out.append({"s": s2, "a": a2, "b": b2, "row": row, "kind": kind, "src": src})
        kinds[(kind, table[row][0], table[row][1])] += 1
    # --- A
    for (w, r, cls, ipa) in table:
        row = index[(w, r)]
        for slot_class, table_class in (("N", "N"), ("A", "A"), ("V", "V"), ("VP", "PAST"), ("NPL", "N"), ("V3", "V")):
            if (w, table_class) in A_SKIP:
                continue
            # base words take base slots, -s words the plural / 3sg slots
            s_word = w in S_WORDS
            if (slot_class in ("NPL", "V3")) != s_word:
                continue
            if table_class not in cls:
                continue
            same = [rr for (ww, rr, cc, _) in table if ww == w and table_class in cc]
            if len(same) != 1:
                continue
            pool = slots[slot_class]
            if not pool:
                continue
            for si, a, b, form in rnd.sample(pool, min(per, len(pool))):
                add(si, a, b, row, "A", form)
    # --- read, with cues
    for si, s in enumerate(sents):
        toks = [(m.group().lower(), m.start(), m.end()) for m in WORD.finditer(s)]
        for k in range(1, len(toks) - 1):
            t, a, b = toks[k]
            if t in forms["VP"] and toks[k - 1][0] in READ_PAST_PREV:
                read_hits["past"].append((si, a, b))
            if t in forms["V"] and toks[k - 1][0] in READ_BASE_PREV:
                read_hits["base"].append((si, a, b))
    for kind_, reading in (("past", "past"), ("base", "present")):
        pool = read_hits[kind_]
        for si, a, b in rnd.sample(pool, min(per * 3, len(pool))):
            add(si, a, b, index[("read", reading)], "A", "read-cue")
    # --- B
    for (w, r), srcs in B_RULES.items():
        if not srcs:
            continue
        pool = [h for sw in srcs.lower().split() for h in byword.get(sw, [])]
        for si, a, b in rnd.sample(pool, min(per * 3, len(pool))):
            add(si, a, b, index[(w, r)], "B", sents[si][a:b].lower())
    # --- C
    for v in sorted(C_VERBS):
        pool = [h for h in c_hits if sents[h[0]][h[1]:h[2]].lower() == v]
        for si, a, b in rnd.sample(pool, min(per, len(pool))):
            add(si, a, b, index[("lead", "metal")], "C", sents[si][a:b])
    rnd.shuffle(out)
    with open(out_path, "w", encoding="utf8") as f:
        for o in out:
            f.write(json.dumps(o, ensure_ascii=False) + "\n")
    cnt = collections.Counter(o["kind"] for o in out)
    print(f"{len(out)} silver examples {dict(cnt)}; {len({o['row'] for o in out})} of {len(table)} readings covered", file=sys.stderr)
    per_class = collections.defaultdict(list)
    for o in out:
        per_class[(o["kind"], table[o["row"]][0], table[o["row"]][1])].append(o)
    with open(out_path + ".counts.tsv", "w", encoding="utf8") as f:
        for k, v in sorted(kinds.items()):
            f.write("\t".join(k) + f"\t{v}\n")


if __name__ == "__main__":
    main()
