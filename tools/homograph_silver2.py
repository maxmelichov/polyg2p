"""Silver v2 for the English heteronym model (round 2). Replaces tools/homograph_silver.py's slot recipe.

Differences from v1 (each one answers a measured failure of round 1):
  * every slot is cue-gated by the words around it (a verb-base slot needs `to`, a modal, do, please, a plural / first /
    second person pronoun, or the sentence start before a determiner / pronoun; a noun slot needs a determiner, possessive,
    preposition or adjective before it, or the sentence start before an auxiliary ...); -ing / -ed forms and every lemma
    that is also an adverb are dropped from noun and adjective slots;
  * sentence-initial slots (imperatives, bare nouns) with a quota of 15% per reading that can start a sentence;
  * author-written frame templates (<= 10% of the output): imperatives and noun frames over WordNet-only fillers;
  * per-word balance: readings share the word's budget by the square root of their gold-train prior (floor 0.15);
    a word gets silver only when every one of its readings has a source;
  * recipe B (word-class substitution for sense pairs) only where every reading of the word has >= 200 source sentences.

Sources: Tatoeba English (CC BY 2.0 FR), WordNet 3.1 (WordNet licence), the gold train split only for the per-word prior
(counts, no text). usage:
  homograph_silver2.py SENTENCES.txt WORDNET_DICT_DIR CORPUS/data OUT.jsonl [--per-word N] [--seed S]
Each line: {"s", "a", "b", "row", "kind": "A|B|C|T", "init": bool, "src"}.
"""
import collections, json, math, os, random, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import homograph_silver as v1  # noqa: E402  (table reader, WordNet forms, recipe-B lists)
import en_homograph_bench as bench  # noqa: E402

WORD = v1.WORD
DET = {"the", "a", "an", "this", "that", "these", "those", "my", "your", "his", "her", "our", "their", "its", "some", "any", "each", "every", "no", "another", "one"}
PREP = {"of", "in", "on", "for", "with", "about", "by", "from", "at", "into", "over", "under", "without", "against", "after", "before"}
MODAL = {"will", "would", "can", "could", "shall", "should", "may", "might", "must", "to", "do", "does", "did", "please", "let's", "don't", "doesn't", "didn't", "not", "never", "always", "and", "or"}
PRON_PL = {"i", "you", "we", "they"}
PRON_3 = {"he", "she", "it"}
OBJ = {"it", "them", "him", "her", "me", "us", "you", "this", "that", "these", "those"}
AUX = {"is", "was", "are", "were", "will", "can", "has", "have", "had", "does", "did", "seems", "looks", "became", "remains", "may", "would", "could", "should"}
COP = {"is", "was", "are", "were", "be", "been", "very", "so", "too", "quite", "looks", "seems", "feels", "more", "most", "rather", "really"}
INIT_Q = 0.15

FRAME_V = ["{V} the {N}.", "Please {V} {det} {N}.", "Do not {V} {pron}.", "{V} it {adv}.", "Let's {V} {det} {N}.", "Always {V} {det} {N}.",
           "Never {V} {det} {N}.", "Who will {V} {det} {N}?", "{V} and {V2} the {N}.", "{V} {pron} {adv}, please.", "Would you {V} {det} {N}?"]
FRAME_N = ["The {N} was {adj}.", "{N} is {adj}.", "I saw a {adj} {N}.", "The {adj} {N} is here.", "Is the {N} {adj}?", "{N} was never {adj}."]
DETS = ["the", "a", "this", "that", "my", "your"]
ADV = ["carefully", "quickly", "again", "today", "tomorrow", "slowly", "later"]
ADJ = ["old", "new", "small", "large", "heavy", "cheap", "quiet", "bright", "broken", "clean", "ready", "strange", "famous", "simple"]


def lemma_set(d, name):
    return {l.split()[0] for l in open(os.path.join(d, name), encoding="latin-1") if not l.startswith(" ") and l.split()[0].isalpha()}


def gold_prior(corpus, table):
    cands = bench.load_wordids(corpus)
    row_of = {}
    for i, (w, r, c, ipa) in enumerate(table):
        row_of[(w, bench.match(ipa, cands[w]))] = i
    cnt = collections.Counter()
    for h in cands:
        for r in bench.read_tsv(os.path.join(corpus, "train", h + ".tsv")):
            cnt[row_of[(h, r[1])]] += 1
    return cnt


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opt = lambda k, d: type(d)(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d
    per_word, seed = opt("--per-word", 260), opt("--seed", 0)
    sent_path, wn, corpus, out_path = args[:4]
    rnd = random.Random(seed)
    table = v1.read_table(os.path.join(HERE, "..", "data", "en_homograph.tsv"))
    index = {(w, r): i for i, (w, r, c, ipa) in enumerate(table)}
    byw = collections.defaultdict(list)
    for i, (w, r, c, ipa) in enumerate(table):
        byw[w].append(i)
    words = set(byw)
    func = {l.split("\t")[0] for l in open(os.path.join(HERE, "..", "data", "en_func.tsv"), encoding="utf8")}
    adv = lemma_set(wn, "index.adv")
    forms = v1.wordnet_forms(wn)
    for c in forms:
        forms[c] -= words | func | adv
    for c in ("N", "NPL", "A"):
        forms[c] = {f for f in forms[c] if not f.endswith(("ing", "ed", "ly"))}
    prior = gold_prior(corpus, table)
    # ---- sentences
    sents = []
    for l in open(sent_path, encoding="utf8"):
        s = l.rstrip("\n").strip()
        n = len(s.split())
        if 3 <= n <= 25 and s.isascii() and s[0].isupper():
            sents.append(s)
    rnd.shuffle(sents)
    print(len(sents), "sentences", file=sys.stderr)
    # ---- one pass: cue-gated slot hits. pool[(slot, init)] = [(si, a, b, form)]
    pool = collections.defaultdict(list)
    b_src = {w for v in v1.B_RULES.values() if v for w in v.lower().split()}
    byword_b = collections.defaultdict(list)
    c_hits = []
    freq = collections.Counter()
    read_hits = {"past": [], "base": []}
    for si, s in enumerate(sents):
        toks = [(m.group(), m.start(), m.end()) for m in WORD.finditer(s)]
        low = [t[0].lower() for t in toks]
        for k, (t, a, b) in enumerate(toks):
            lw = low[k]
            freq[lw] += 1
            prev = low[k - 1] if k else ""
            nxt = low[k + 1] if k + 1 < len(toks) else ""
            first = k == 0
            if lw in b_src and (t.islower() or first or t[0].isupper() and lw in ("czech", "german", "french")):
                byword_b[lw].append((si, a, b))
            if k > 0 and prev in v1.C_SUBJ and lw in v1.C_VERBS and t.islower():
                c_hits.append((si, a, b))
            if not (t.islower() or first and t[0].isupper() and t[1:].islower()):
                continue
            ok = lambda c: lw in forms[c]
            if ok("V") and (prev in MODAL or prev in PRON_PL or (first and (nxt in DET or nxt in OBJ))):
                pool[("V", first)].append((si, a, b, lw))
            if ok("V3") and prev in PRON_3:
                pool[("V3", False)].append((si, a, b, lw))
            if ok("VP") and (prev in PRON_3 or prev in PRON_PL):
                pool[("VP", False)].append((si, a, b, lw))
            if ok("N") and ((prev in DET or prev in PREP or prev in forms["A"]) and not first or first and (nxt in AUX or nxt in PREP or nxt in forms["N"] or nxt in forms["NPL"])):
                pool[("N", first)].append((si, a, b, lw))
            if ok("NPL") and ((prev in DET or prev in PREP or prev in forms["A"]) and not first or first and (nxt in AUX or nxt in PREP or nxt in forms["N"])):
                pool[("NPL", first)].append((si, a, b, lw))
            if ok("A") and prev in COP and not first:
                pool[("A", False)].append((si, a, b, lw))
            if k >= 1 and lw in forms["VP"] and prev in v1.READ_PAST_PREV:
                read_hits["past"].append((si, a, b))
            if k >= 1 and lw in forms["V"] and prev in v1.READ_BASE_PREV:
                read_hits["base"].append((si, a, b))
    for k, v in sorted(pool.items()):
        print(k, len(v), file=sys.stderr)
    # ---- candidates per row: (si, a, b, kind, init, src)
    cand = collections.defaultdict(list)
    skip_cls = {("lead", "PAST"), ("read", "PAST"), ("read", "V"), ("aged", "A"), ("aged", "PAST"), ("blessed", "A"), ("blessed", "PAST"), ("learned", "A"), ("learned", "PAST")}
    s_words = v1.S_WORDS
    for w, rows in byw.items():
        for ri in rows:
            _, r, cls, _ = table[ri]
            for slot, tcls in (("N", "N"), ("A", "A"), ("V", "V"), ("VP", "PAST"), ("NPL", "N"), ("V3", "V")):
                if (w, tcls) in skip_cls or (slot in ("NPL", "V3")) != (w in s_words) or tcls not in cls:
                    continue
                same = [x for x in rows if tcls in table[x][2]]
                if len(same) != 1:
                    continue
                for init in (False, True):
                    for (si, a, b, form) in pool.get((slot, init), []):
                        cand[ri].append((si, a, b, "A", init, form))
    # read with cues (the tense is certain only with them)
    for kind_, reading in (("past", "past"), ("base", "present")):
        for (si, a, b) in read_hits[kind_]:
            cand[index[("read", reading)]].append((si, a, b, "A", False, "read-cue"))
    # recipe B only where every reading of the word has >= 200 sources
    bw = collections.defaultdict(dict)
    for (w, r), srcs in v1.B_RULES.items():
        if srcs:
            bw[w][r] = [h for sw in srcs.lower().split() for h in byword_b.get(sw, [])]
    for w, d in bw.items():
        if len(d) == len(byw[w]) and all(len(h) >= 200 for h in d.values()):
            for r, hits in d.items():
                for (si, a, b) in hits:
                    cand[index[(w, r)]].append((si, a, b, "B", False, sents[si][a:b].lower()))
    for (si, a, b) in c_hits:
        cand[index[("lead", "metal")]].append((si, a, b, "C", False, sents[si][a:b].lower()))
    # ---- balance per word
    out, kinds = [], collections.Counter()
    for w, rows in sorted(byw.items()):
        if not all(cand.get(ri) for ri in rows):
            continue
        sh = {ri: (prior[ri] + 1) ** 0.75 for ri in rows}
        tot = sum(sh.values())
        sh = {ri: max(0.15, v / tot) for ri, v in sh.items()}
        tot = sum(sh.values())
        for ri in rows:
            n = round(per_word * sh[ri] / tot)
            c = cand[ri]
            ini = [x for x in c if x[4]]
            non = [x for x in c if not x[4]]
            n_init = min(len(ini), math.ceil(n * INIT_Q)) if ini else 0
            pick = rnd.sample(ini, n_init) + rnd.sample(non, min(n - n_init, len(non)))
            seen = set()
            for (si, a, b, kind, init, src) in pick:
                if (si, a) in seen:
                    continue
                seen.add((si, a))
                s2, a2, b2 = v1.swap(sents[si], a, b, w)
                out.append({"s": s2, "a": a2, "b": b2, "row": ri, "kind": kind, "init": init, "src": src})
                kinds[(kind, w, table[ri][1], "init" if init else "")] += 1
    # ---- templates (<= 10% of the final file)
    common = lambda c: sorted(f for f in forms[c] if freq[f] >= 15 and f not in ("cassiterite",))
    nouns, verbs = common("N"), common("V")
    n_t = int(0.10 * len(out) / 0.90)
    tmpl = []
    have = {table[o['row']][0] for o in out}   # templates only for words that also have balanced two-sided slot silver
    vrows = [(w, ri) for w, rows in byw.items() if w in have for ri in rows if "V" in table[ri][2] and len([x for x in rows if "V" in table[x][2]]) == 1 and (w, "V") not in skip_cls and w not in s_words]
    nrows = [(w, ri) for w, rows in byw.items() if w in have for ri in rows if "N" in table[ri][2] and len([x for x in rows if "N" in table[x][2]]) == 1 and (w, "N") not in skip_cls and w not in s_words and len(rows) == 2]
    for i in range(n_t):
        if i % 2 == 0 and vrows and nouns and verbs:
            w, ri = rnd.choice(vrows)
            f = rnd.choice(FRAME_V)
            s = f.format(V=w.capitalize() if f.startswith("{V}") else w, N=rnd.choice(nouns), det=rnd.choice(DETS), pron=rnd.choice(sorted(OBJ - {"this", "that", "these", "those"})), adv=rnd.choice(ADV), V2=rnd.choice(verbs))
            init = f.startswith("{V}")
        elif nrows and nouns:
            w, ri = rnd.choice(nrows)
            f = rnd.choice(FRAME_N)
            s = f.format(N=w.capitalize() if f.startswith("{N}") else w, adj=rnd.choice(ADJ))
            init = f.startswith("{N}")
        else:
            continue
        m = re.search(r"(?<![A-Za-z'])" + re.escape(w) + r"(?![A-Za-z'])", s, re.I)
        out.append({"s": s, "a": m.start(), "b": m.end(), "row": ri, "kind": "T", "init": init, "src": "template"})
        kinds[("T", w, table[ri][1], "init" if init else "")] += 1
    rnd.shuffle(out)
    with open(out_path, "w", encoding="utf8") as f:
        for o in out:
            f.write(json.dumps(o, ensure_ascii=False) + "\n")
    with open(out_path + ".counts.tsv", "w", encoding="utf8") as f:
        for k, v in sorted(kinds.items()):
            f.write("\t".join(k) + f"\t{v}\n")
    byk = collections.Counter(o["kind"] for o in out)
    print(f"{len(out)} silver examples {dict(byk)}; {len({o['row'] for o in out})} of {len(table)} readings covered; "
          f"{len({table[o['row']][0] for o in out})} words", file=sys.stderr)


if __name__ == "__main__":
    main()
