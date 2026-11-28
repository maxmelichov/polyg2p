# Italian (`it`): stress and e/o quality against an independent lexicon (2026-11-28)

PhonItalia 1.10 (non-commercial, evaluation only, nothing from it is in the repo) scores 5,000 sampled forms (dev and test
halves cut by md5 parity, test scored once). Italian stress is counted with the glide of -zione/-ia/-io read as notation
(an unstressed i or u directly before the stress mark is the glide j or w), so `tsiˈone` and `tsːjˈone` have the same nucleus count.

Changes:
- Polysyllabic articles and articulated prepositions (della, alla, una, sulla ...) carry the ordinary primary mark, as
  Wiktionary writes them (/ˈdal.la/, /ˈu.na/); only the monosyllabic clitics in `UNSTRESSED` stay bare. They had `ˌ`.
- `tools/it_kaikki.py` builds `it.lex` from the stress-marked IPA of single-word headwords in the kaikki.org extract of
  en.wiktionary, merged over the WikiPron entries (the first transcription listed decides; the alternatives `last`,
  `closed` and `unanimous` scored lower on dev). It settles stress, open/closed e and o and the voicing of z for 85k words.

| test half (n = 2,496 words, 2,411 with stress) | before | after | paired 95 % CI of the gain |
|---|---|---|---|
| stress position, with `it.lex` | 94.2 % | **96.5 %** | +1.5 .. +3.2 |
| word accuracy, with `it.lex` | 95.2 % | 95.9 % | +0.1 .. +1.2 |
| stress position, rules only | 90.8 % | 91.1 % | +0.2 .. +0.6 |
| word accuracy, rules only | 91.8 % | 91.8 % | 0.0 |

What stays wrong is mostly the lexicon's own disagreement with Wiktionary: Wiktionary writes an open vowel for potrebbe,
sarebbe, centri, remi, tutela and lists both qualities for esempio, membro, debba; the lexicon has the closed one. Forms
Wiktionary has no transcription for (conditionals in -ebbe, many derived forms) fall back to the rules.
