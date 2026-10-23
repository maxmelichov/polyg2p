# Modern Greek (`el`)

Rules only (`src/el.rs`, no lexicon, no model). Notation follows espeak-ng's `el`
voice; pronunciations follow Standard Modern Greek where espeak deviates.

## Accuracy vs WikiPron (`sources/ell_grek_broad.tsv`)

`POLYG2P_SRC=... .venv/bin/python tools/eval_acc.py el` — the 20k most frequent
wordfreq words that have a WikiPron entry (4,641 words, frequency-weighted) and,
for a second view, every WikiPron headword (19,108 words, uniform weights).
Collapse rules: `tools/collapse_el.py` (stress, flap/trill, ɟ/ɡ, ɪ/i, the nasal
of μπ/ντ/γκ and double consonants are neutralized). The default collapse also
neutralizes synizesis (ια vs ʝa/ça/ʎa/ɲa), because WikiPron lists both readings
for many words; `POLYG2P_EL_STRICT=1` keeps it.

| set | collapse | polyg2p word acc | polyg2p PER | espeak word acc | espeak PER |
|---|---|---|---|---|---|
| top-20k, freq-weighted | default | **99.3%** | 0.2% | 80.8% | 0.8% |
| top-20k, freq-weighted | strict (synizesis counts) | **99.2%** | 0.5% | 79.5% | 1.6% |
| all 19,108 headwords, uniform | default | **98.1%** | 0.3% | 93.1% | 1.0% |
| all 19,108 headwords, uniform | strict | **95.9%** | 0.7% | 88.8% | 1.6% |

Where espeak loses (frequency-weighted set): it never voices σ before voiced
consonants (κόσμος kˈosmos, gold kozmos), reads the articles η and ο as the letter
names ήτα/όμικρον, writes γγ+consonant as ŋɡ (συγγνώμη, gold ŋɣ), keeps ι+vowel
as a hiatus in most words (κάποια kˈapiˌa, παιδιά pˌeðiˈa, μάτια mˈatiˌa) and
has a few broken entries (διακόσια ðʲakˈoʂa).

## What the engine does

- Letters: monotonic and polytonic input (Greek Extended block folded to base +
  accent), combining marks (NFD), diaeresis, final ς, capital accents; Latin
  letters are transliterated so foreign words read aloud (facebook → fasˈebuk).
- Vowels: αι=e, ει/οι/υι=i, ου=u; αυ/ευ/ηυ → a/e/i + v before vowels and voiced
  consonants, f before voiceless ones and word-finally (αυτός aftˈos, αυγό avɣˈo,
  ταυ tˈaf, εμπνευσμένο ˌembnevzmˈeno). Diaeresis and an accent on the first letter
  block the digraph (καΐκι kaˈici, Μάιος mˈaɪos). An unstressed ι/η/ϊ after a
  vowel is the non-syllabic ɪ espeak writes (ρολόι rolˈoɪ, προϊόντα proɪˈonda).
- Consonants: μπ/ντ/γκ = b/d/ɡ word-initially, mb/nd/ŋɡ inside a word (gold
  prefers the nasal 583:68; espeak writes d/b); mp before a voiceless consonant
  (πέμπτη); γγ = ŋɡ before a vowel, ŋɣ before a consonant; γχ/γξ = ŋx/ŋks; ν before
  a velar = ŋ (Κονγκό koŋɡˈo); τσ/τζ = ts/dz; ξ/ψ = ks/ps; double consonants
  single; σ → z before β γ δ ζ μ ν λ ρ (κόσμος kˈozmos, σβήνω zvˈino).
- Palatals: κ/χ/γ → c/ç/j before e, i (και ce, χέρι çˈeri, γυναίκα jinˈeka).
- Synizesis: an unaccented ι/υ/η/ει/οι between a consonant and a vowel becomes a
  glide — ʎ after λ, ɲ after ν, mɲ after μ, absorbed by c/ç/j, ç after other
  voiceless and j after voiced consonants (παιδιά peðjˈa, ποιος pçˈos, μια mɲˈa,
  ήλιος ˈiʎos, χρόνια xrˈoɲa, δουλειά ðuʎˈa). The hiatus is kept in the learned
  patterns: ρ-clusters (τριάντα, χρειάζομαι), initial δια-/διε- except a short
  list of everyday words (διάλογος vs διαβάζω), περι-, -ιότητα, -ποιώ, ει before
  a stressed vowel mid-word (σημειώνω), -ιος/-ιο nouns (κύριος, σχέδιο, στάδιο;
  ήλιος, γέλιο, μπάνιο, όποιο glide), -ειa after most consonants (τέλεια,
  συνέπεια, οικογένεια; άδεια, αλήθεια glide), -ύρια/-ήρια (εκατομμύρια),
  ιέ except after λ ν ρ τ (πιέζω vs καριέρα), before an accented i (χρησιμοποιεί).
- Stress: the tonos is the primary stress; every written accent is marked
  (enclitic groups: όνομά μου ˈonomˈa mu); unaccented polysyllables (capitals)
  take the penult; monosyllables are stressed except articles, clitics and
  particles (και, το, να, θα, μου…); the auxiliaries είμαι/έχω and από, αλλά, μαζί
  get the ˌ-only reading espeak uses (είναι ˌine). Secondary stress on every
  second syllable away from the primary, as espeak (ðˌisekˌatomˈiriˌa).
- Sentence pass (`join`): final ν of τον/την/δεν/μην/σαν/έναν/αυτόν/στον/στην
  voices a following stop and assimilates (τον πατέρα tom batˈera, την κόρη tiŋ
  ɡˈori, δεν ξέρω ðˈeŋ ɡzˈero); a word-final s voices before a voiced consonant
  (τους γονείς tuz ɣonˈis); elided clitics glue to the next word (σ' αυτό saftˈo,
  γι' αυτό jaftˈo).
- Numbers: neuter cardinals, εκατόν before a following number, feminine forms
  with χιλιάδες (τρεις χιλιάδες, διακόσιες μία χιλιάδες), εκατομμύριο/-α,
  δισεκατομμύριο; decimal κόμμα. Symbols: % τοις εκατό, € ευρώ, $ δολάρια,
  £ λίρες, & και, + συν, @ παπάκι, ° βαθμοί, = ίσον. Latin acronyms are spelled
  with Greek-style letter names (ei, bi, si, …).

## Error analysis (strict collapse, frequency-weighted set)

Of the remaining 0.8% weighted errors, most are WikiPron artefacts or genuinely
variable words:

- Gold inconsistency on palatalization (ιταλική "italiki", ακούστηκε "akustike",
  κοινωνία "kinonia" — WikiPron writes k before i for some entries) and on
  synizesis (διάφορα ðʝafora vs διάταξη ðiataksi for the same prefix; εργαστήριο
  erɣastirʝo vs εκατομμύρια ekatomiria).
- Wrong gold forms (στοίβα → stivazo, πρόσθεση → prosθesis, φτάσαμε → ftasane,
  κι → "c", η → ita).
- Loanwords keeping an unvoiced nt/mp cluster (ίντερνετ, ντοκιμαντέρ, βαμπίρ,
  καμπάνια, σνακ, σλόγκαν): the rules apply the native nd/mb/z reading.
- Learned vs popular synizesis the rules get wrong: ενάντια (enadia), Καλιφόρνια,
  βιολί (vʝoli), Βέλγιο, συνήθεια/προμήθεια (gold hiatus; αλήθεια glide),
  ομόνοια, εμβόλια, παράλια.

## Known gaps

- Synizesis is decided by rules, not a lexicon: learned/popular doublets
  (χρόνια "years" [ˈxroɲa] vs χρόνια "chronic" [ˈxronia]) and rarer learned words
  in -ια/-ιος can come out glided when they should not (or vice versa).
- Loanwords: no detection of foreign nt/mp (ίντερνετ) or of σ before a sonorant
  that stays voiceless (σλόγκαν); Latin-script words are transliterated letter
  by letter, not read as English.
- Enclitic stress shift that is not written (ο άνθρωπος μου without the second
  accent) is not added.
- Final-ν sandhi is applied only after the listed clitics, not after every word
  ending in ν; cross-word s-voicing is applied before every voiced consonant.
- Unaccented polysyllables (all-caps text) fall back to penultimate stress.
