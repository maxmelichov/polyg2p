"""Which spelling contexts does espeak write a reduced vowel as ɪ/ᵻ rather than ə?

Aligns CMUdict vowel phones to the word's vowel-letter groups (by order, when
the counts match), and espeak's vowels the same way, then tabulates espeak's
symbol for every CMU AH0/IH0 by (letter group, following letters, final?).
Output guides the hand-written rules in en_convert.py; nothing from espeak is
stored.
"""
import os, re, sys, collections, unicodedata
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import eval_acc as E
from en_convert import load_cmudict
from wordfreq import top_n_list

lex = load_cmudict("sources/cmudict.dict")
words = [w for w in top_n_list("en", 60000) if w in lex and re.fullmatch(r"[a-z]+", w)][:25000]
esp = E.espeak(words, "en")
VG = re.compile(r"[aeiouy]+")
EV = re.compile(r"[ˈˌ]?(?:aɪ|aʊ|eɪ|oʊ|ɔɪ|ɪə|ɛə|ʊə|ɑː|ɔː|iː|uː|ɜː|[æɑɔəɛɜɪʊʌɚᵻɐiuaeo])ɹ?")
EVOW = re.compile(r"[ˈˌ]?(aɪ|aʊ|eɪ|oʊ|ɔɪ|ɑː|ɔː|iː|uː|ɜː|æ|ɑ|ɔ|ə|ɛ|ɜ|ɪ|ʊ|ʌ|ɚ|ᵻ|ɐ|i|u|a|e|o)")
tab = collections.defaultdict(collections.Counter)
ex = collections.defaultdict(list)
for w, e in zip(words, esp):
    e = unicodedata.normalize("NFC", e)
    phones = lex[w][0].split()
    groups = [(m.start(), m.end()) for m in VG.finditer(w)]
    # silent final e (not the only vowel): drop from alignment
    if len(groups) > 1 and w.endswith("e") and groups[-1] == (len(w) - 1, len(w)) and not w.endswith(("le", "re")):
        groups = groups[:-1]
    vph = [i for i, p in enumerate(phones) if p[-1].isdigit()]
    evs = [m.group(1) for m in EVOW.finditer(e)]
    if len(groups) != len(vph) or len(evs) != len(vph):
        continue
    for k, (gi, pi) in enumerate(zip(groups, vph)):
        p = phones[pi]
        if p not in ("AH0", "IH0"):
            continue
        g = w[gi[0]:gi[1]]
        after = w[gi[1]:gi[1] + 3]
        final = k == len(groups) - 1
        key = (p, g, after if final else after[:1] + "…", "F" if final else "M")
        tab[key][evs[k]] += 1
        if len(ex[key]) < 3:
            ex[key].append(f"{w}:{e}")
rows = sorted(tab.items(), key=lambda kv: -sum(kv[1].values()))
for key, c in rows[:140]:
    n = sum(c.values())
    print(n, key, dict(c.most_common(3)), ex[key])
