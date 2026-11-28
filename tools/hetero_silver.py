"""Cue rules for heteronym labels that are safe by construction ("silver").

Each rule looks at the engine's words of a sentence and returns (k, cue_index) for the table word at
position j, or None. The cue word is recorded so the trainer can delete it with p=0.5 (the model must not
learn the cue alone). Silver labels are reported apart from human gold and from teacher labels.
"""
FR_SUBJ = {"je", "j'", "tu"}
FR_DET = {"le", "un", "ce", "ton", "son", "mon", "notre", "votre", "leur", "quel", "aucun", "ce", "cet", "du", "au", "au"}
FR_DET_F = {"la", "une", "cette", "ta", "sa", "ma", "notre", "votre", "leur", "quelle"}
FR_PL = {"les", "des", "ces", "mes", "tes", "ses", "nos", "vos", "leurs", "plusieurs", "quelques", "de", "d'"}
FR_BE = {"est", "suis", "es", "sommes", "êtes", "sont", "très", "si", "tellement", "bien", "étais", "était", "serai", "sera", "soit", "trop", "aussi", "plus", "moins", "un", "une", "le", "la"}
FR_REFL = {"se", "te", "me", "nous", "vous", "s'", "t'", "m'"}


def fr(words, j):
    w = words[j]
    p1 = words[j - 1] if j >= 1 else ""
    p2 = words[j - 2] if j >= 2 else ""
    if w in ("sens", "vis", "lis"):
        if p1 in FR_SUBJ or (p1 == "ne" and p2 in FR_SUBJ) or (p1 in FR_REFL and p2 in FR_SUBJ):
            return 1, j - 1 if p1 in FR_SUBJ else j - 2
        if p1 in FR_DET or p1 in FR_DET_F or p1 in ("de", "du", "en", "sans", "double"):
            if w == "lis" and p1 not in ("le", "un", "des", "les", "ces", "du", "de"):
                return None
            return 0, j - 1
    if w == "fier":
        if p1 in FR_REFL or (p1 == "pas" and p2 in ("ne", "de")) or p1 in ("à", "vous", "nous"):
            return (1, j - 1) if p1 in FR_REFL else None
        if p1 in FR_BE:
            return 0, j - 1
    if w == "fils" and (p1 in ("mon", "ton", "son", "notre", "votre", "leur", "ce", "ton", "le", "un", "du") or p1 == "petit-"):
        return 0, j - 1
    if w.endswith("tions") and len(w) > 6:
        if p1 in ("nous",) or (p1 in ("n'", "ne") and p2 == "nous"):
            return 1, j - 1 if p1 == "nous" else j - 2
        if p1 in FR_PL or p1 in ("quelques", "nos", "vos", "ses"):
            return 0, j - 1
    if w in ("couvent", "président", "résident", "violent", "parent", "content", "somnolent", "émergent", "excellent", "expédient",
             "équivalent", "négligent", "influent", "affluent", "convergent", "divergent", "ferment"):
        if p1 in ("ils", "elles", "qui") or (p1 == "ne" and p2 in ("ils", "elles", "qui")):
            return 1, j - 1 if p1 != "ne" else j - 2
        if p1 in FR_DET or p1 in FR_DET_F or p1 in ("les", "des", "ces", "mes", "ses"):
            return 0, j - 1
    return None


PT_DET = {"o", "um", "meu", "seu", "teu", "nosso", "vosso", "do", "no", "pelo", "ao", "este", "esse", "aquele", "qual", "outro", "mesmo", "bom", "grande", "novo", "primeiro", "último"}
PT_NEG = {"não", "nunca", "já", "ainda", "também", "só", "me", "te", "se", "lhe", "o", "a"}


def pt(words, j):
    p1 = words[j - 1] if j >= 1 else ""
    p2 = words[j - 2] if j >= 2 else ""
    if p1 == "eu" or (p1 in PT_NEG and p2 == "eu") or (p1 in ("me", "te", "se") and p2 in ("não", "eu")) :
        return 1, j - 1 if p1 == "eu" else j - 2
    if p1 in PT_DET and p1 not in ("o", "a") or p1 == "o" and p2 not in ("eu", "não", "nunca", "que"):
        return 0, j - 1
    return None


UK_GEN_PREP = {"без", "для", "до", "від", "біля", "навколо", "після", "замість", "коло", "крім", "проти", "поруч", "серед", "окрім", "протягом", "щодо", "навпроти", "поблизу"}
UK_PL = {"ці", "ті", "мої", "твої", "наші", "ваші", "всі", "усі", "які", "деякі", "дві", "три", "чотири", "свої", "інші", "різні", "нові", "старі", "великі", "малі", "обидві", "обидва"}
UK_GRAM = {"руки", "ноги", "книжки", "сестри", "гори", "води", "голови", "слова", "міста", "поля", "моря"}
UK_FLOUR = {"пшенична", "житня", "кукурудзяна", "вівсяна", "гречана", "рисова", "біла", "борошно"}


def uk(words, j):
    w = words[j]
    p1 = words[j - 1] if j >= 1 else ""
    if w in UK_GRAM:
        if p1 in UK_PL:
            return 0, j - 1
        if p1 in UK_GEN_PREP:
            return 1, j - 1
    if w == "мука" and p1 in UK_FLOUR:
        return 1, j - 1
    return None


IT_AUX = {"ho", "hai", "ha", "abbiamo", "avete", "hanno", "avevo", "aveva", "avevano", "avrà", "avrei"}
IT_DET = {"il", "un", "lo", "i", "dei", "del", "al", "dal", "nel", "mio", "tuo", "suo", "nostro", "vostro", "gli", "ai", "dai", "nei", "miei", "tuoi", "suoi", "grandi", "due", "tre"}


def it(words, j):
    w = words[j]
    p1 = words[j - 1] if j >= 1 else ""
    p2 = words[j - 2] if j >= 2 else ""
    if w in ("capito", "seguito", "subito"):
        if p1 in IT_AUX or (p1 in ("non", "già", "mai", "ancora", "sempre") and p2 in IT_AUX):
            return 1, j - 1 if p1 in IT_AUX else j - 2
        if w == "seguito" and p1 in ("in", "al", "nel", "il", "un", "col"):
            return 0, j - 1
    if w == "perdono":
        if p1 in ("il", "un", "di", "chiedo", "chiedere", "chiedi", "mio", "suo"):
            return 1, j - 1
        if p1 in ("loro", "essi", "esse", "gli", "li"):
            return 0, j - 1
    if w == "leggere":
        if p1 in ("da", "a", "per", "di", "poter", "sapere", "voglio", "vuole", "può", "posso", "devo"):
            return 0, j - 1
        if p1 in ("sono", "più", "molto", "troppo", "così", "piuttosto", "erano", "poco", "ma"):
            return 1, j - 1
    if w == "compito" and p1 in ("il", "un", "mio", "nostro", "del", "al", "dal", "suo", "tuo", "questo", "quel"):
        return 0, j - 1
    if w == "desideri":
        if p1 in ("tu", "che", "se", "ciò"):
            return 1, j - 1
        if p1 in ("i", "dei", "due", "miei", "tuoi", "suoi", "nostri", "vostri", "questi"):
            return 0, j - 1
    if w == "impari" and p1 in ("tu", "che", "se", "non"):
        return 1, j - 1
    if w == "pagano":
        if p1 in ("loro", "essi", "non", "che") and (p1 != "non" or p2 in ("loro", "essi", "che")):
            return 0, j - 1 if p1 != "non" else j - 2
        if p1 in ("i", "dei", "gli", "ai", "i"):
            return 1, j - 1
    if w == "benefici" and p1 in ("i", "dei", "gli", "ai", "miei", "suoi", "grandi", "numerosi", "molti"):
        return 1, j - 1
    if w == "tendine" and p1 in ("le", "delle", "queste", "le mie", "alle", "sulle"):
        return 0, j - 1
    return None


SILVER = {"fr": fr, "pt": pt, "uk": uk, "it": it}
