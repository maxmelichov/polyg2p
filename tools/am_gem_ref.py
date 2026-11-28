"""Amharic gemination layer on top of am_g2p (v3 reference). Separate file on purpose: removable, never edits am_g2p.py.
am_g2p is imported from $AM_G2P_DIR (required); with empty flags word() == am_g2p.word() (checked by selftest()).
Fidel does not write long consonants. flags = set of letter indices (into the letters _fidel accepts) that are long.
  guards (2.1): never word-initial, never a vowel carrier (rows አ ዐ), never h (rows ሀ ሐ ኀ ኸ), never a word-final bare letter.
  R1: a bare consonant directly before a long consonant takes ɨ (ሀብታምነት -> habtamɨnːət).
  R2: a long bare consonant that is not word-final takes ɨ (ምሽት -> mɨʃːɨt, አምስት -> amːɨst).
  The stem table is skipped when a flag falls on a stem letter or on the letter right after the stem.
  ː is written after the whole consonant (tʃʼː, lʷːa).
Tagger input (round 2): one folded word per call (no sentence context), words folded by the one-to-one homophone fold FOLD."""
import os, sys, re, importlib.util
_D = os.environ.get("AM_G2P_DIR")
if not _D: raise SystemExit("set AM_G2P_DIR to the folder that holds the reference am_g2p.py (the Python twin of src/am.rs)")
_spec = importlib.util.spec_from_file_location("am_g2p_ref", os.path.join(_D, "am_g2p.py")); A = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(A)
FOLD = str.maketrans({"ሠ": "ሰ", "ሡ": "ሱ", "ሢ": "ሲ", "ሣ": "ሳ", "ሤ": "ሴ", "ሥ": "ስ", "ሦ": "ሶ", "ጸ": "ፀ", "ጹ": "ፁ", "ጺ": "ፂ", "ጻ": "ፃ", "ጼ": "ፄ", "ጽ": "ፅ", "ጾ": "ፆ",
    "ዐ": "አ", "ዑ": "ኡ", "ዒ": "ኢ", "ዓ": "አ", "ዔ": "ኤ", "ዕ": "እ", "ዖ": "ኦ", "ኣ": "አ", "ሃ": "ሀ", "ሐ": "ሀ", "ሑ": "ሁ", "ሒ": "ሂ", "ሓ": "ሀ", "ሔ": "ሄ", "ሕ": "ህ", "ሖ": "ሆ",
    "ኀ": "ሀ", "ኁ": "ሁ", "ኂ": "ሂ", "ኃ": "ሀ", "ኄ": "ሄ", "ኅ": "ህ", "ኆ": "ሆ", "ኸ": "ሀ", "ኹ": "ሁ", "ኺ": "ሂ", "ኻ": "ሀ", "ኼ": "ሄ", "ኽ": "ህ", "ኾ": "ሆ"})
GLIDE = {"j", "w"}                          # R2 does not apply before a glide-initial syllable (ኢትዮጵያ itjopʼːja: WikiPron has no ɨ there)
R1 = R2 = True                               # the two syllabification additions (switches only for the A3 ablation)
fold = lambda w: w.translate(FOLD)           # one-to-one: positions are kept
def letters(w): return "".join(ch for ch in w if A._fidel(ch) is not None)
def guards(w):
    """w = fidel letters only -> list of bool: may position i be long?"""
    seg = [A._fidel(ch) for ch in w]; n = len(seg)
    return [i > 0 and c != "ʔ" and c.rstrip("ʷ") != "h" and not (i == n - 1 and not v) for i, (c, v) in enumerate(seg)]
def trace(w, gem=()):
    """[(phone, letter index or -1)]; consonant phones carry ː when long"""
    w = letters(w)
    seg = [A._fidel(ch) for ch in w]; n = len(seg); bare = [c != "ʔ" and not v for c, v in seg]
    ok = guards(w); g = [i in gem and ok[i] for i in range(n)]
    k0, st = A._stem(w); out = []; j = 0
    pl = set(A.PROCL.values())
    for p in st:                                 # the consonants of a stem carry their letter index (gold alignment, tier tests)
        idx = -1
        if p not in A.VOW and p != "ʔ" and p not in pl:
            for q in range(j, k0):
                if seg[q][0] == p: idx = q; j = q + 1; break
        out.append((p, idx))
    if any(g[:k0 + 1]): k0, out = 0, []
    coda = 0 if not out or out[-1][0] in A.VOW else 1
    perf = getattr(A, "_perf", None)
    fin = n >= 2 and bare[n - 2] and bare[n - 1] and (
        A._coda_ok(seg[n - 2][0], seg[n - 1][0]) or w in A.CLUSTER or (w[0] in A.PROCL and w[1:] in A.CLUSTER)
        or (perf is not None and perf(w) and A._son_ok(seg[n - 2][0], seg[n - 1][0]) and (seg[n - 1][0] != "h" or A.SON.get(seg[n - 2][0]) == 3)))
    for i, (c, v) in enumerate(seg):
        if i < k0: continue
        last = i == n - 1; m = "ː" if g[i] else ""; gn = not last and g[i + 1]
        if c == "ʔ":
            if out and coda == 0: out.append(("ʔ", i))
            if not (last and not v and out and coda == 0): out.append((v or "ɨ", -1))
            coda = 0
        elif v: out += [(c + m, i), (v, -1)]; coda = 0
        elif not last and (not out or coda > 0): out += [(c + m, i), ("ɨ", -1)]; coda = 0
        elif not last and ((i == n - 3 and fin) or (not bare[i + 1] and seg[i + 1][0] == "ɲ" and seg[i + 1][1])
                           or (c, seg[i + 1][0].rstrip("ʷ")) in A.AFF or (R2 and g[i] and not (not bare[i + 1] and seg[i + 1][0] in GLIDE)) or (R1 and gn)):
            out += [(c + m, i), ("ɨ", -1)]; coda = 0
        elif not last or coda == 0: out.append((c + m, i)); coda = 1
        elif coda == 1 and fin: out.append((c, i)); coda = 2
        else: out += [("ɨ", -1), (c, i)]; coda = 1
    return out
def word(w, gem=()): return "".join(p for p, _ in trace(w, gem))
def rule(w):
    """tier-P2 rule: ɲ and plain tʃ are long after a vowel when a vowel follows or the word ends (guards applied)"""
    t = trace(w); g = set(); ok = guards(letters(w)); V = set("aeiouəɨ")
    for k, (p, i) in enumerate(t):
        if p in ("ɲ", "tʃ") and k > 0 and t[k - 1][0][-1] in V and (k == len(t) - 1 or t[k + 1][0][0] in V) and ok[i]: g.add(i)
    return g
def tokens(text):
    """words (fidel letters, unfolded) and marks of the reference tokenizer: [(word or None, mark or None)]"""
    out = []
    for m in A._TOK.finditer(A.marks(text)):
        if m.group(1):
            if word(m.group(1)): out.append([m.group(1), ""])
        elif out and not (out[-1][1] and m.group(2) in ".,:"): out[-1][1] += m.group(2)
    return out
def tagger_input(text):
    """(string of 2.3, [(word index, offset of its first letter in the string)]) ; the words are the ones that carry letters"""
    toks = tokens(text); parts = []; pos = []; k = 0
    for w, mk in toks:
        lw = letters(w); pos.append(k); parts.append(fold(lw) + mk); k += len(parts[-1]) + 1
    return " ".join(parts), pos
def g2p(text, flags=None):
    """flags: per word (same order as tokens(text)) a set of letter indices; None = no marks (== am_g2p.g2p)"""
    out = []
    for n, (w, mk) in enumerate(tokens(text)):
        out.append(word(w, flags[n] if flags else ()) + mk)
    return " ".join(out)
def selftest(words):
    bad = [w for w in words if word(w) != A.word(w)]
    assert not bad, f"am_gem.word != am_g2p.word on {len(bad)} words, e.g. {bad[:5]}: re-sync the loop with the current am_g2p.word"
    return len(words)

# ---------------------------------------------------------------- the optional tagger / table (reference of polyg2p am::Gem)
import json as _json, struct as _struct
import numpy as np
class Gem:
    """Reads a G2PT file with numpy (int8 rows dequantized as q * scale[row]), float32 forward mirroring tagger.rs.
    A long letter is logit[1] > logit[0] of the `stress` head (the threshold is baked into its bias). table: {folded word: [bool]}."""
    def __init__(self, model_path=None, table_path=None):
        self.t = {}; self.cfg = None
        if model_path:
            b = open(model_path, "rb").read(); assert b[:4] == b"G2PT"; hl = _struct.unpack("<I", b[4:8])[0]; h = _json.loads(b[8:8 + hl]); d = b[8 + hl:]
            self.cfg = h["config"]; self.vocab = h["vocab"]; meta = {m["name"]: m for m in h["tensors"]}
            for m in h["tensors"]:
                if m["name"].endswith(".scale"): continue
                n = int(np.prod(m["shape"]))
                if m.get("dtype") == "i8":
                    sm = meta[m["name"] + ".scale"]; sc = np.frombuffer(d, "<f4", m["shape"][0], sm["offset"]); q = np.frombuffer(d, np.int8, n, m["offset"]).reshape(m["shape"])
                    self.t[m["name"]] = (q.astype(np.float32) * sc[:, None]).astype(np.float32)
                else: self.t[m["name"]] = np.frombuffer(d, "<f4", n, m["offset"]).reshape(m["shape"]).astype(np.float32)
        self.table = None
        if table_path:
            self.table = {}
            for l in open(table_path, encoding="utf-8"):
                a = l.rstrip("\n").split("\t")
                if len(a) < 2: continue
                w = []; f = []
                for ch in a[1]:
                    if ch == "፟":
                        if f: f[-1] = True
                    elif A._fidel(ch) is not None: w.append(fold(ch)); f.append(False)
                self.table.setdefault("".join(w), f)
    def empty(self): return not self.t and not self.table
    def _ln(self, x, p):
        m = x.mean(-1, keepdims=True); v = ((x - m) ** 2).mean(-1, keepdims=True); return ((x - m) / np.sqrt(v + 1e-5) * self.t[p + ".weight"] + self.t[p + ".bias"]).astype(np.float32)
    def _lin(self, x, p): return (x @ self.t[p + ".weight"].T + self.t[p + ".bias"]).astype(np.float32)
    def logits(self, text):
        c = self.cfg; ids = [self.vocab.get(ch, 1) for ch in text[: c["max_len"]]]; L = len(ids); d = c["d"]; H = c["heads"]; hd = d // H
        x = self.t["emb.weight"][ids] + self.t["pos.weight"][:L]
        for l in range(c["layers"]):
            p = f"enc.{l}"; h = self._ln(x, p + ".ln1"); q = self._lin(h, p + ".sa.q"); k = self._lin(h, p + ".sa.k"); v = self._lin(h, p + ".sa.v"); att = np.zeros((L, d), np.float32)
            for hh in range(H):
                s = q[:, hh * hd:(hh + 1) * hd] @ k[:, hh * hd:(hh + 1) * hd].T * np.float32(1 / np.sqrt(hd)); s = np.exp(s - s.max(-1, keepdims=True)); s = s / s.sum(-1, keepdims=True)
                att[:, hh * hd:(hh + 1) * hd] = s @ v[:, hh * hd:(hh + 1) * hd]
            x = x + self._lin(att, p + ".sa.o"); h = self._ln(x, p + ".ln3"); x = x + self._lin(np.maximum(self._lin(h, p + ".ff1"), 0), p + ".ff2")
        return self._lin(self._ln(x, "norm"), "stress")
    @staticmethod
    def windows(t, mx):
        out = []; s = 0
        while s < len(t):
            e = min(len(t), s + mx)
            if e < len(t):
                k = t.rfind(" ", s, e)
                if k > s: e = k
            out.append((s, e)); s = e + 1 if e < len(t) and t[e] == " " else e
        return out
    def flags(self, toks):
        """toks = [(word, marks)] -> per word (set of long letter idx, margins list). margin = logit[1] - logit[0] (None without a model).
        Round 2: the tagger reads ONE word at a time (the folded word alone, marks do not reach it), once per distinct word; a word the table
        knows is not sent to it. Table rows without a mark are an explicit 'never long' (heteronym types)."""
        ls = [letters(w) for w, _ in toks]; res = [set() for _ in toks]; mar = [[None] * len(l) for l in ls]; seen = {}
        for j, l in enumerate(ls):
            key = fold(l)
            if self.t and not (self.table is not None and key in self.table):
                if key not in seen:
                    r = set(); m = [None] * len(l)
                    for s, e in self.windows(key, self.cfg["max_len"]):
                        lg = self.logits(key[s:e])
                        for q in range(e - s):
                            m[s + q] = float(lg[q, 1] - lg[q, 0])
                            if m[s + q] > 0: r.add(s + q)
                    seen[key] = (r, m)
                res[j] = set(seen[key][0]); mar[j] = list(seen[key][1])
            if self.table is not None and key in self.table: res[j] = {i for i, x in enumerate(self.table[key]) if x}
            ok = guards(l); res[j] = {i for i in res[j] if ok[i]}
        return res, mar
def g2p_gem(text, gem):
    """the sentence reading with the optional layer (== g2p(text) when gem is empty)"""
    toks = tokens(text)
    if gem is None or gem.empty(): return g2p(text)
    fl, _ = gem.flags(toks)
    return " ".join(word(w, fl[n]) + mk for n, (w, mk) in enumerate(toks))
