#!/usr/bin/env python3
"""Russian: rules / dictionary-first / `ru-ctx.g2p` against sentences whose reading a person chose (measurement only).

  HETERO_WORK=dir HETERO_RU_LEX=data-dir tools/hetero_ru_measure.py stress|classes|yo

stress:  <dir>/gold/ru_stress_gold.json = [[sentence, word, "stressed form with + after the stressed vowel"], ...]
yo:      Tatoeba sentences (<dir>/raw/tat_rus.txt) where the writer typed ё: the ё is removed for the test and the
         engine must restore it; negatives are plain е words in sentences that use ё elsewhere.
"First dictionary candidate" = ru_check with POLYG2P_RU_CTX=0. Gold sets are not committed.
"""
import json, subprocess, re, os, collections, sys
W=os.environ['HETERO_WORK'].rstrip('/')+'/'
LEX=os.environ['HETERO_RU_LEX']
def report(sents, env_ctx):
    env=dict(os.environ)
    if not env_ctx: env['POLYG2P_RU_CTX']='0'
    p=subprocess.run(['target/release/ru_check','--report','--lex',LEX],input="\n".join(sents)+"\n",capture_output=True,text=True,env=env)
    out=p.stdout.split("\n"); res=[];i=0
    for _ in sents:
        ipa=out[i];i+=1; ws=[]
        while i<len(out) and out[i]!="": ws.append(out[i].split("\t"));i+=1
        i+=1; res.append((ipa,ws))
    return res
VOW=set("aɐəɪʊiuoeɛɨɵɜʉæyɔɑɒ")
def stressed_syl(ipa):
    if 'ˈ' not in ipa: return None
    return sum(1 for c in ipa.split('ˈ')[0] if c in VOW)
def gold_syl(f): return sum(1 for c in f.split('+')[0] if c in 'аеёиоуыэюя')-1
def pick(ws,w):
    for x in ws:
        if x[0].lower().strip('.,!?"«»—-:;')==w: return x
if sys.argv[1]=='stress':
    rows=json.load(open(W+'gold/ru_stress_gold.json'))
    sents=[r[0] for r in rows]
    a=report(sents,True); b=report(sents,False)
    st=collections.defaultdict(lambda:[0,0,0])
    for (s,w,form),(ia,wa),(ib,wb) in zip(rows,a,b):
        g=gold_syl(form); xa,xb=pick(wa,w),pick(wb,w)
        if not xa or not xb: continue
        d=st[w]; d[0]+=1; d[1]+= stressed_syl(xb[2])==g; d[2]+= stressed_syl(xa[2])==g
    tot=[0,0,0]
    for w,d in st.items():
        print(f"{w:<10}{d[0]:>4} dict-first {d[1]/d[0]:.2f}  ctx {d[2]/d[0]:.2f}"); [tot.__setitem__(i,tot[i]+d[i]) for i in range(3)]
    print("TOTAL",tot[0],round(tot[1]/tot[0],3),round(tot[2]/tot[0],3))
if sys.argv[1]=='classes':
    lex={'замок','мука','орган','атлас','хлопок','уже','стоит','большая','дорог','парить'}
    rows=json.load(open(W+'gold/ru_stress_gold.json'))
    sents=[r[0] for r in rows]; a=report(sents,True); b=report(sents,False)
    c=collections.defaultdict(lambda:[0,0,0])
    for (s,w,form),(ia,wa),(ib,wb) in zip(rows,a,b):
        g=gold_syl(form); xa,xb=pick(wa,w),pick(wb,w)
        if not xa or not xb: continue
        k='lexical' if w in lex else 'grammatical'
        d=c[k]; d[0]+=1; d[1]+=stressed_syl(xb[2])==g; d[2]+=stressed_syl(xa[2])==g
    for k,d in c.items(): print(k,d[0],round(d[1]/d[0],3),round(d[2]/d[0],3))
if sys.argv[1]=='yo':
    import re
    Y=['всё','чём','нём','берёт','узнаём','осёл','нёбо','ещё','идёт','живёт','даёт','моём','твоём','своём','пришёл','нёс','несёт','тёмный','всём','чём']
    pos=collections.defaultdict(list); neg=collections.defaultdict(list)
    tg={'все':'всё','всё':'всё','чем':'чём','нем':'нём','берет':'берёт','узнаем':'узнаём','осел':'осёл','небо':'нёбо'}
    for s in open(W+'raw/tat_rus.txt',encoding='utf8'):
        s=s.strip()
        if not(10<=len(s)<=140): continue
        ws=[x.lower() for x in re.findall(r"\w+",s)]
        if 'ё' in s:
            for yw in ('всё','чём','нём','берёт','узнаём','осёл','нёбо'):
                if ws.count(yw)==1 and len(pos[yw])<80: pos[yw].append(s)
            # negatives: sentence with ё elsewhere and the plain form present
            for plain in ('все','чем','нем','берет','узнаем','осел','небо'):
                if ws.count(plain)==1 and len(neg[plain])<80: neg[plain].append(s)
    rows=[]  # (sentence_with_e, word_e, expect_yo)
    for yw,v in pos.items():
        for s in v: rows.append((s.replace('ё','е').replace('Ё','Е'),yw.replace('ё','е'),1))
    for pw,v in neg.items():
        for s in v: rows.append((s.replace('ё','е').replace('Ё','Е'),pw,0))
    # the negative sentences lose ё elsewhere too, which is the test condition
    sents=[r[0] for r in rows]; a=report(sents,True); b=report(sents,False)
    st=collections.defaultdict(lambda:[0,0,0])
    for (s,w,yo),(ia,wa),(ib,wb) in zip(rows,a,b):
        xa,xb=pick(wa,w),pick(wb,w)
        if not xa or not xb: continue
        isyo=lambda x: ('ɵ' in x[2] or 'jo' in x[2] or 'ʲo' in x[2] or 'ɵ' in x[2]) 
        key=(w,yo); d=st[key]; d[0]+=1; d[1]+= isyo(xb)==bool(yo); d[2]+= isyo(xa)==bool(yo)
    tot=[0,0,0]
    for k,d in sorted(st.items()):
        print(k,d[0],round(d[1]/d[0],2),round(d[2]/d[0],2)); [tot.__setitem__(i,tot[i]+d[i]) for i in range(3)]
    print('TOTAL',tot[0],round(tot[1]/tot[0],3),round(tot[2]/tot[0],3))
