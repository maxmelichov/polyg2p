"""Ending-based open/closed probability for stressed e/o (reads it_lex.tsv, it_suffix.tsv)."""
import collections
VL = "aeiouàèéìòóùíú"
suf = {}
for line in open('it_suffix2.tsv'):
    s, st = line.rstrip('\n').split('\t')
    suf[s] = int(st)
def pred(w):
    for k in range(7, 1, -1):
        if len(w) > k and w[-k:] in suf:
            return suf[w[-k:]]
    return 1  # Italian default: penult
stats = collections.defaultdict(collections.Counter)
for line in open('it_lex.tsv'):
    w, st, op = line.rstrip('\n').split('\t')
    if op == '-':
        continue
    if st == '-':
        if op != 'c':
            continue
        st = pred(w)
    st = int(st)
    vs = [i for i, c in enumerate(w) if c in VL]
    if len(vs) <= st:
        continue
    pos = vs[len(vs) - 1 - st]
    if w[pos] not in 'eo':
        continue
    tail = w[pos:]
    for k in range(2, len(tail) + 1):
        stats[tail[:k] if k < len(tail) else tail + '#'][op] += 1
    stats[w[pos] + '*'][op] += 1
with open('it_open.tsv', 'w') as f:
    for key, c in sorted(stats.items()):
        tot = c['o'] + c['c']
        if tot >= 4:
            f.write(f"{key}\t{c['o']/tot:.2f}\t{tot}\n")
