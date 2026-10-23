import sys, difflib, collections
def report(pairs, top=30):
    """pairs: (word, ours_list, espeak)"""
    agree=0; diffs=collections.Counter(); ex=collections.defaultdict(list); n=0
    for w, cands, e in pairs:
        if not cands: continue
        n+=1
        if e in cands: agree+=1; continue
        c=cands[0]
        for tag,a1,a2,b1,b2 in difflib.SequenceMatcher(a=c,b=e,autojunk=False).get_opcodes():
            if tag!='equal':
                k=f"{c[a1:a2]}>{e[b1:b2]}"; diffs[k]+=1
                if len(ex[k])<3: ex[k].append(f"{w}:{c}|{e}")
    print(f"exact agree {agree}/{n} = {agree/max(n,1):.1%}")
    for k,v in diffs.most_common(top): print(v,k,ex[k])
