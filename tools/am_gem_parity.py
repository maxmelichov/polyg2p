"""Parity of the Amharic gemination layer: polyg2p am_gem_check (Rust) vs am_gem.py (reference), byte for byte.
usage: am_gem_parity.py MODEL CLI AM_G2P_DIR SENTENCES.txt GOLDWORDS.txt [--fuzz N] [--table]
  SENTENCES: one fidel sentence per line; GOLDWORDS: one word per line; AM_G2P_DIR: folder with the am_g2p.py reference
Inputs: the sentences, the gold words, N fuzz inputs (random letters, real words with proclitics / suffix letters, random marks).
Differences are only counted as float-order ties when the margin logit[1]-logit[0] of a flagged letter is below 1e-4.
Exit 1 on any mismatch. Without a model (MODEL = -) both sides must equal the plain am_g2p reading."""
import sys, os, re, random, subprocess, multiprocessing as mp
a = sys.argv[1:]; model, cli, d, sf, gf = a[:5]; fz_n = int(a[a.index("--fuzz") + 1]) if "--fuzz" in a else 350000; use_table = "--table" in a
os.environ["AM_G2P_DIR"] = d; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import am_gem_ref as G
A = G.A
def fold(s):   # polyg2p's output convention (src/fold.rs): affricate ligatures; ts only at the end of a word
    s = s.replace("tʃ", "ʧ").replace("dʒ", "ʤ").replace("dz", "ʣ"); return re.sub(r"ts(?![\w\u0300-\u036f])", "ʦ", s)
sents = [l.rstrip("\n") for l in open(sf, encoding="utf-8") if l.strip()]; gold = [l.split("\t")[0].strip() for l in open(gf, encoding="utf-8") if l.strip()]
rng = random.Random(1128); real = sorted({w for t in sents for w in re.findall("[ሀ-ፚ]+", t)})
table = None
if use_table:
    table = "/tmp/am_gem_parity_table.tsv"
    with open(table, "w", encoding="utf-8") as f:
        for w in rng.sample(real, min(400, len(real))):
            lw = G.letters(w); ok = G.guards(lw); marked = "".join(ch + ("፟" if ok[i] and rng.random() < 0.3 else "") for i, ch in enumerate(lw)); f.write(f"{w}\t{marked}\n")
gem = G.Gem(None if model == "-" else model, table)
cmd = [cli, model] + ([table] if table else [])
def rust(lines):
    r = subprocess.run(cmd, input="\n".join(lines) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")[:-1]
    assert len(r) == len(lines), (len(r), len(lines)); return [x.split("\t")[2] for x in r]
def ref(t): return fold(G.g2p_gem(t, gem))
def chunks(lines, k):
    n = max(1, len(lines) // k + 1); return [lines[i:i + n] for i in range(0, len(lines), n)]
def _ref_many(ls): return [ref(t) for t in ls]
def compare(name, lines):
    lines = [t for t in lines if not A.unknown(t) and "\n" not in t]
    with mp.Pool(4) as p: R = [x for c in p.map(_ref_many, chunks(lines, 4)) for x in c]
    out = []
    for i in range(0, len(lines), 20000): out += rust(lines[i:i + 20000])
    bad = [(t, r, f) for t, r, f in zip(lines, out, R) if r != f]
    print(f"{name}: {len(lines)} inputs, Rust != reference: {len(bad)}", flush=True)
    for t, r, f in bad[:5]: print("   ", repr(t[-70:]), "\n      rust", repr(r[-80:]), "\n      ref ", repr(f[-80:]))
    return len(bad), bad
tot = 0
tot += compare("sentences", sents)[0]; tot += compare("gold words", gold)[0]
PRE = ["", "", "የ", "በ", "ለ", "ከ", "እንደ", "የሚ", "ስለ"]; SUF = ["", "", "ን", "ም", "ህ", "ሽ", "ኝ", "ና", "ንም", "ኛ", "ኛው", "ቱ", "ተኛ", "ቷ", "ች", "ዎች", "ው", "ዋ"]
MARK = ["።", "፣", "፤", "፥", "፦", "፧", "፡", "፡፡", "::", "፡-", ".", ",", "?", "!", ":", " ", "  ", "-", "'", "«", "»", "…", "(", ")", "—", "፨", "፟", "́", ";"]
FID = [chr(o) for o in range(0x1200, 0x135B) if A._fidel(chr(o))]; SIX = [c for c in FID if not A._fidel(c)[1]]
def rword():
    k = rng.random()
    if k < 0.35: return "".join(rng.choice(SIX if rng.random() < 0.6 else FID) for _ in range(rng.randint(1, 8)))
    if k < 0.45: return rng.choice(PRE) + rng.choice([s for s, *_ in A.STEMS] + sorted(A.CLUSTER)) + rng.choice(["", "ት", "ተ", "ቱ", "ቅ", "ቁ", "ቶስ", "ቲያን", "ን", "ም"]) + rng.choice(SUF)
    if k < 0.55:
        st = rng.choice(A.PERF); return rng.choice(PRE + list(A.HEADS) + list(A.HEADS_A) + ["መ", "ባ", "ላ"]) + rng.choice([st, st[1:], st[:-1]]) + rng.choice(["ን", "ሽ", "ህ", "ን", "ሽ", "ም", "ኝ", "ንም", "ክ", ""])
    return rng.choice(PRE) + rng.choice(real) + rng.choice(SUF)
nw = int(fz_n * 6 / 7); nl = fz_n - nw
fz = [rword() for _ in range(nw)] + ["".join(rng.choice([rword(), rword(), rng.choice(MARK)]) + rng.choice(["", " ", " ", rng.choice(MARK)]) for _ in range(rng.randint(1, 12))) for _ in range(nl)]
bad, bl = compare(f"fuzz ({nw} words, {nl} lines)", fz); tot += bad
if bad and gem.t:   # margins of the mismatching inputs
    ties = 0
    for t, r, f in bl:
        fl, mar = gem.flags(G.tokens(t)); ms = [abs(m) for ms_ in mar for m in ms_ if m is not None]
        ties += bool(ms) and min(ms) < 1e-4
    print(f"mismatches with a letter margin below 1e-4 (float-order ties): {ties} of {bad}")
sys.exit(1 if tot else 0)
