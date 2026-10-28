"""Agreement of polyg2p `ro` with espeak-ng on a word list (notation study).

usage: ro_cmp_espeak.py ESPEAK.tsv [top]   (file from ro_espeak_top.py)
Reports exact agreement, stress-position agreement (primary stress index among
vowels), and the most frequent segment differences with examples.
"""
import collections, difflib, os, re, subprocess, sys, unicodedata

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
ref = [l.rstrip("\n").split("\t") for l in open(sys.argv[1], encoding="utf8")]
top = int(sys.argv[2]) if len(sys.argv) > 2 else 40
inp = "\n".join(w for w, _ in ref) + "\n"
out = subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), "ro"], input=inp, capture_output=True, text=True).stdout.split("\n")


def norm(p):
    p = unicodedata.normalize("NFD", p).replace("ʧ", "tʃ").replace("ʤ", "dʒ").replace("ʦ", "ts").replace("ʲʲ", "ʲ")
    return p.rstrip(".")


V = "aeiouəɨɔy"


def stress_idx(p):
    """index (among vowels) of the primary stress; -1 if none"""
    k = 0
    pend = False
    prev = ""
    for ch in p:
        if ch == "ˈ":
            pend = True
            prev = ""
            continue
        if ch == "ˌ":
            prev = ""
            continue
        if ch in V:
            if ch == "a" and prev in "eɔ":
                prev = ch
                continue
            if pend:
                return k
            k += 1
        prev = ch
    return -1


exact = segs = stress = n = 0
diffs = collections.Counter(); ex = collections.defaultdict(list)
sdiff = []
for (w, e), o in zip(ref, out):
    e, o = norm(e), norm(o)
    if " " in e or " " in o:
        continue
    n += 1
    if e == o:
        exact += 1
    es, os_ = re.sub("[ˈˌ]", "", e), re.sub("[ˈˌ]", "", o)
    if es == os_:
        segs += 1
        si, so = stress_idx(e), stress_idx(o)
        if si == so:
            stress += 1
        elif si >= 0 and so >= 0:
            sdiff.append((w, o, e))
    else:
        for tag, a1, a2, b1, b2 in difflib.SequenceMatcher(a=os_, b=es, autojunk=False).get_opcodes():
            if tag != "equal":
                k = f"{os_[a1:a2]}>{es[b1:b2]}"
                diffs[k] += 1
                if len(ex[k]) < 4:
                    ex[k].append(f"{w}:{o}|{e}")
print(f"words {n}: exact {exact/n:.1%}, same segments {segs/n:.1%}, same segments+stress {stress/n:.1%}")
print("--- segment differences (ours>espeak)")
for k, v in diffs.most_common(top):
    print(v, k, ex[k])
print("--- stress differences (word ours espeak), first", top)
for w, o, e in sdiff[:top]:
    print(w, o, e)
