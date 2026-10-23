"""The 256-symbol phoneme vocabulary BlueTTS models use (ids are list positions
in BlueTTS's text_vocab.py; this module only needs the set, to validate output)."""

VOCAB: frozenset = frozenset('_^$ !\'(),-.:;?abcdefhijklmnopqrstuvwxyzæçðøħŋœǀǁǂǃɐɑɒɓɔɕɖɗɘəɚɛɜɞɟɠɡɢɣɤɥɦɧɨɪɫɬɭɮɯɰɱɲɳɴɵɶɸɹɺɻɽɾʀʁʂʃʄʈʉʊʋʌʍʎʏʐʑʒʔʕʘʙʛʜʝʟʡʢʲˈˌːˑ˞βθχᵻⱱ0123456789̧̪̯̩̃ʰˤε↓#"↑̺̻gʦXABCDEFGHIJKLMNOPQRSTUVWYZʤɝʧʼʴʱʷˠ→↗↘¡¿…«»*~/\\&̴̛̥̬̹̜̝̞̟̠̰̘̙͈̤̼̆̈̚͜͡ˀˁʾʿʻˉˊˋˆ˥˦˧˨˩̀́̂̄̌̇ʣ')

# symbols that are control slots in the vocabulary and must never appear in output
CONTROL = frozenset("_^$")


def unknown_symbols(text: str) -> list:
    """Symbols in ``text`` outside the vocabulary (they would become PAD)."""
    seen = {}
    for ch in text:
        if ch not in VOCAB or ch in CONTROL:
            seen.setdefault(ch, None)
    return list(seen)
