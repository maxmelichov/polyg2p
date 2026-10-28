# Dutch (`nl`)

Rules-only engine (`src/nl.rs`), espeak-ng `nl` notation. Gold: WikiPron `nld_latn_broad.tsv`
(58 539 entries, 45 826 single-word headwords). Both outputs are collapsed with
`tools/collapse_nl.py` (ɦ/h, ʏ/ɵ, ɛi̯/ɛɪ, œy̯/œy, ɑu̯/ʌʊ, ɪː/eː, ɔː/oː before r, ts/s in -tie,
hiatus glides, optional final -n of -en) so only pronunciation differences count.

## Numbers

`POLYG2P_SRC=sources .venv/bin/python tools/eval_acc.py nl` — top 18 342 frequent words that
have a Wiktionary entry, frequency-weighted:

| system | word acc | PER |
|---|---|---|
| polyg2p (rules only) | **99.1 %** | **3.2 %** |
| espeak-ng `nl` | 96.8 % | 5.2 % |
| polyg2p + optional `nl.lex` | 99.9 % | 0.0 % (circular: the lexicon is built from the gold) |

Unweighted, over all 45 826 single-word Wiktionary headwords (names and rare loans included),
the rules agree with Wiktionary on 36 995 = 80.7 % (`tools/nl_build_lex.py` reports this while
building the lexicon).

## What is in the engine

* Letter groups: vowel digraphs/trigraphs (aa ee oo uu ie oe eu ui ij ei ou au aai ooi oei eeuw
  ieuw uw ouw auw eau ai oi), diaeresis letters start a new group (ideeën, geëist, coördinatie),
  `-eum` and `i+eu`/`i+ee` split (museum, serieus, dieet), y/qu as consonants.
* Syllable openness → vowel length: a/e/o/u long before one consonant + vowel (ch, ng, nk, ck, sj,
  tj, x count as two; th/ph/qu as one), loan clusters C+r and b/p/k/f/c+l keep the syllable open
  (zebra, publiek) but t/d+l do not (atlas).
* Schwa: trailing chain of unstressed syllables (-e, -en, -er, -el, -em, -es after a single
  consonant, -end, -erd, -eld, -elt, -ig, -lijk, -eren/-elen/-enen, -elijk, -erij, -ering,
  -enis) with the -el/-es/-ent/-ens/-eld/-elt/-ern/-est contrasts (hotel/tafel, succes/huisjes,
  moment/betekent); medial unstressed e: open → ə unless followed by the final unstressed
  syllable (werkgever, werknemer, medeleven) or by r+i (materiaal); closed er/el/en/em + C → ə
  after a full syllable (Nederland, boekenkast) but ɛ after a reduced one (medewerker) and
  before loan clusters nt/ns/nc/ntr (studenten, concentratie).
* The -eren decision (stud-EREN vs ver-ANDER-en) comes from `data/nl_eren.tsv`, aggregate
  counts of ə vs eː per two letters before -er(en) derived from WikiPron by `tools/nl_eren.py`.
* Morphology: unstressed prefixes be-/ge-/ver-/her-/ont-/er-/inter- with checks that the rest is a
  stem with a full syllable (beter, gevel, verder, geving, gener- are not prefixed; ont+v → ɔntf);
  stressed separable prefixes (aan, uit, op, af, in, om, over, onder, door, voor, mee, toe, weg,
  terug, samen, tegen, achter, binnen, buiten, boven, tussen, bij, mis, on, opper …) with
  onset validation (geldig is not ge+ldig) and nasal assimilation (aan/in/on + g/k → ŋ);
  -heid/-heden split off; -tie compound heads (politiebureau, informatiebron) split off.
* Consonants: ɣ (x word-finally, before/after voiceless, in -gisch), sch → sx / s in -isch(e),
  initial ch → ʃ (k before r/l), tch → tʃ, c → s/k, ti/ci + vowel → ʃ (nationaal, sociaal,
  patiënt), -tie → tsi after a vowel, si after a consonant, -rsie → rzi, intervocalic s → z in
  loans (basis, visie) but not after diphthongs or in -ese/-isch, -age → ʒ (≥3 syllables), gn+e →
  nj, w → ʋ (v in wr-), final h silent, words without vowels are spelled (sms, dvd).
* Voicing: final devoicing, coda devoicing before any consonant that cannot start a syllable with it
  (wereldwijd, gezondheid, advies → ɑtfis), v/z/ɣ → f/s/x after voiceless obstruents (except z
  after x), ɣ/v/z/b stay voiced before d (vreugde, hoofden), geminates reduced, t+d → d
  (totdat), d+j → tj (hondje), af+d → ɑv, mis+d → mɪz.
* Stress `ˈ` directly before the stressed vowel (espeak's placement): first full syllable of the
  stem, suffix stress for -ie (not -lie/-erie/-tie), -tie/-sie (syllable before), -eren/-ering
  when the table says eː, -iteit, -uur, -eel, -eur, -oon, -ist, -isme, -iek, -ier, -aal, -aat,
  -ine, -ijn, -oen, -eau, -ees, -ant, -ent, -erij, -ij, -age, -ië (two syllables before), -é;
  compound tails get `ˌ`.
* A built-in exception table (~1 000 entries): reduced function words (de, het, een, er, je …),
  frequent English/French loans (computer, website, team, journalist, route …), numbers, and
  words the rules get wrong (bijzonder, natuurlijk, hersenen, namelijk …).
* `cardinal` (één, eenentwintig, tweeëntwintig, honderddrieëntwintig, tweeduizend vijfhonderd,
  drie miljard), `DECIMAL = komma`, `LETTERS`, `symbol` (% procent, € euro, @ apenstaartje …).
* Optional lexicon: if `POLYG2P_DATA/nl.lex` exists it is memory-mapped lazily (`OnceLock`) and
  consulted before the rules; unknown words are then split into lexicon words with s/en/e linkers
  the way `de.rs` does. Build it with `tools/nl_build_lex.py` + `build_lex` (CC BY-SA, data pack
  only). The engine does not see the `--data` flag of the CLI, only the environment variable.

## Error analysis (2 472 of 18 342 words wrong, 702 of which espeak gets right)

Grouped by the most common collapsed-phone substitution (count of words):

| count | ours → gold | examples | cause |
|---|---|---|---|
| 192 | ə → eː | Almere, arresteren, strategie, Gerard | pretonic medial e in loans/names keeps eː; no spelling cue |
| 174 | ɛ → ə | hogeschool, doodgeschoten, satelliet, eraf | linking -e- before sch/st in compounds; ge- inside compounds |
| 160 | ə → ɛ | daadwerkelijk, commercieel, diversiteit | er/el + C in compounds and Latin loans |
| 149 | eː → ə | bevorderen, benadering, Jeroen | -eren table ties and names |
| 107 | f → v | uitvoeren, woordvoerder, hulpverlening | WikiPron keeps v after voiceless in some compounds (tf 199 : tv 102 overall) |
| 92 | aː → ɑ | Canada, abonnement, Arabisch, manieren | short unstressed a in loans (kanaal, papier have aː) |
| 85 | s → z | universiteit, cursus, opzetten | -rs- voicing in loans, z after p |
| 80 | ŋ → n / n → ŋ | overeenkomst, ongeacht, aanpak | nasal assimilation at compound seams is inconsistent in the gold |
| 77 | ɪ → i | villa, illegaal, registratie | tense i in closed syllables of loans |
| ~250 | misc. | magazine, frame, drone, handicap, tournee | English/French spellings not in the exception table |

Also notable: WikiPron itself is inconsistent for several frequent items (gesproken ɔ, strijd
without t, gegeten with x, nieuwe ʋ vs vernieuwen u̯, afdeling v vs huisdeur s), which caps what
rules can reach.

## Known gaps

* No compound splitter without the lexicon: linking -e-/-en- before consonants, ŋ/n at seams and
  secondary stress inside compounds are heuristic (werkgever, boekenkast are right; hogeschool,
  daadwerkelijk are not).
* Loanword layers (English, French, Latin) beyond the exception table are read with Dutch
  letter values (frame → fraːmə, tournee → tʌʊrneː).
* Stress is not evaluated by the gold (WikiPron has none); the suffix list and first-syllable
  default are plausible but untested against data.
* Ordinal suffixes written as digits + e (21e) are not handled (that lives in `num.rs`).
* `join` is a no-op: no cross-word assimilation (e.g. "het boek" → hə(d)buk).
