"""Build an unbiased-ish ending->stress table and evaluate on held-out words vs espeak."""
import collections, random, sys, unicodedata
VL = "aeiouàèéìòóùíú"
lex = {}
for line in open('it_lex.tsv'):
    w, st, op = line.rstrip('\n').split('\t')[:3]
    lex[w] = (None if st == '-' else int(st), op)
words = sorted(lex)
random.seed(1)
test = set(random.sample([w for w in words if lex[w][0] is not None and sum(c in VL for c in w) >= 3], 1500))
def vpos(w): return [i for i, c in enumerate(w) if c in VL]
def syllabic(w, p):
    nxt = p + 1 < len(w) and w[p + 1] in VL
    return not nxt or w[p] in "àèéìòóùíú" or w[p] not in "iu"
# counts: for each ending (k letters) how many words have it, and how many are known
# to be stressed at index s (from open-vowel evidence)
tot = collections.Counter()
hit = collections.defaultdict(collections.Counter)
for w, (st, op) in lex.items():
    if w in test: continue
    for k in range(2, 8):
        if len(w) > k:
            tot[w[-k:]] += 1
            if st is not None:
                hit[w[-k:]][st] += 1
def default(w):
    vp = vpos(w); nv = len(vp)
    syl = [p for p in vp if syllabic(w, p)]
    p = syl[-2] if len(syl) >= 2 else (syl[-1] if syl else vp[-1])
    return nv - 1 - vp.index(p)
TH = float(sys.argv[1]) if len(sys.argv) > 1 else 0.4
table = {}
for suf, n in tot.items():
    if n < 4: continue
    for st, c in hit[suf].items():
        if st != 1 and c / n >= TH:
            table[suf] = st
        if st == 1 and c / n >= TH:
            table[suf] = st
def pred(w):
    if w[-1] in "àèéìòóù": return 0
    for k in range(7, 1, -1):
        if len(w) > k and w[-k:] in table:
            return table[w[-k:]]
    return default(w)
ok = sum(pred(w) == lex[w][0] for w in test)
print(f"threshold {TH}: table {len(table)} entries, held-out stress acc {ok/len(test):.3f}")
# espeak on the same words
try:
    ref = {}
    import subprocess, os
    from phonemizer.backend import EspeakBackend
    from phonemizer.separator import Separator
    tl = sorted(test)
    b = EspeakBackend('it', with_stress=True)
    out = b.phonemize(tl, separator=Separator(phone='', word=' '), strip=True)
    IV = "aeiouɛɔɪʊ"
    eok = 0
    for w, p in zip(tl, out):
        # stressed vowel index from end among IPA vowels; map to letters approximately
        # by counting syllables after the stress mark
        if 'ˈ' not in p: continue
        after = p[p.index('ˈ') + 1:]
        first = True; n_after = 0
        prev_v = False
        for ch in after:
            v = ch in IV
            if v and not prev_v and not first: n_after += 1
            if v: first = False
            prev_v = v
        # convert our letter-based index to syllables-after for comparison
        vp = vpos(w); st = lex[w][0]
        sp = vp[len(vp) - 1 - st]
        syl_after = sum(1 for q in vp if q > sp and syllabic(w, q))
        eok += (n_after == syl_after)
    print(f"espeak held-out stress acc {eok/len(test):.3f}")
    # same syllable-based metric for ours
    ook = 0
    for w in tl:
        vp = vpos(w)
        ours = vp[len(vp) - 1 - pred(w)]; gold = vp[len(vp) - 1 - lex[w][0]]
        ook += ours == gold
    print(f"ours (letter position) {ook/len(test):.3f}")
except Exception as e:
    print("espeak eval failed", e)
with open('it_suffix2.tsv', 'w') as f:
    for s, st in sorted(table.items()):
        f.write(f"{s}\t{st}\n")
