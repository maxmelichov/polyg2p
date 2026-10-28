"""polyg2p: fast grapheme-to-phoneme for TTS, espeak-compatible IPA, Apache-2.0.

English, Spanish, Italian, German, Russian and Hebrew run in the Rust core
(Hebrew needs the ``he.g2p`` tagger in the data folder). Yiddish, and Hebrew
when ``he.g2p`` is missing, are routed to their Python engines when installed:

* he -> renikud-onnx (``pip install renikud-onnx``), only without ``he.g2p``
* yi -> phonikud-yi (the ``yiddish_phonikud`` package from phonikud-yi-blue-tts)

    >>> from polyg2p import Phonemizer
    >>> g2p = Phonemizer()
    >>> g2p("I have read the record.", "en")
    'aɪ hæv ɹˈɛd ðə ɹˈɛkɚd.'

Output follows the conventions BlueTTS trains on (espeak symbols after its
``normalize_text``: NFD, affricate ligatures), and stays inside its 256-symbol
vocabulary (see ``unknown_symbols``).
"""

from __future__ import annotations

import os
import re
import threading
from pathlib import Path
from typing import Callable, Optional

from ._polyg2p import ALL_LANGS, CORE_LANGS, Engine
from .vocab import unknown_symbols, VOCAB

__all__ = ["Phonemizer", "Engine", "unknown_symbols", "VOCAB", "SUPPORTED_LANGS", "default_data_dir"]

SUPPORTED_LANGS = tuple(dict.fromkeys(list(CORE_LANGS) + ["he", "yi"]))
_ALIASES = {"ge": "de", "en-us": "en", "eng": "en", "heb": "he", "iw": "he", "rus": "ru", "yid": "yi", "ji": "yi"}
_INLINE = re.compile(r"<(\w+)>(.*?)</\1>", re.DOTALL)
_TAG = re.compile(r"</?\w+>")
_HEBREW = re.compile(r"[֐-׿]")


def default_data_dir() -> Optional[Path]:
    """$POLYG2P_DATA, else the data folder shipped inside the wheel."""
    env = os.environ.get("POLYG2P_DATA")
    if env:
        return Path(env)
    p = Path(__file__).with_name("data")
    return p if p.is_dir() else None


def norm_lang(lang: str) -> str:
    lang = lang.lower()
    return _ALIASES.get(lang, lang)


class Phonemizer:
    """Multilingual phonemizer.

    Args:
        data_dir: folder with lexicon/model files for the Rust core (default: shipped data).
        hebrew: optional callable text -> IPA overriding the renikud backend.
        yiddish: optional callable text -> IPA overriding the phonikud-yi backend.
        russian: optional callable text -> IPA overriding the built-in Russian path.
        hebrew_speaker / hebrew_target_speaker: renikud speaker hints (0 unknown, 1 male, 2 female).
    """

    def __init__(
        self,
        data_dir: Optional[os.PathLike] = None,
        hebrew: Optional[Callable[[str], str]] = None,
        yiddish: Optional[Callable[[str], str]] = None,
        russian: Optional[Callable[[str], str]] = None,
        hebrew_speaker: int = 0,
        hebrew_target_speaker: int = 0,
    ):
        d = Path(data_dir) if data_dir else default_data_dir()
        self.engine = Engine(str(d) if d else None)
        self._he, self._yi, self._ru = hebrew, yiddish, russian
        self._he_spk = (hebrew_speaker, hebrew_target_speaker)
        self._lock = threading.Lock()

    # ------------------------------------------------------------ backends
    def _hebrew(self, text: str) -> str:
        if self._he is None:
            with self._lock:
                if self._he is None:
                    try:
                        from renikud_onnx import G2P  # type: ignore
                    except ImportError as e:
                        raise RuntimeError("Hebrew needs renikud-onnx: pip install renikud-onnx") from e
                    g = G2P()
                    spk, tgt = self._he_spk

                    def he(t: str, _g=g) -> str:
                        try:
                            return _g.phonemize(t, speaker=spk, target_speaker=tgt)
                        except TypeError:  # plain renikud-onnx without speaker hints
                            return _g.phonemize(t)

                    self._he = he
        return self._he(text)

    def _yiddish(self, text: str) -> str:
        if self._yi is None:
            with self._lock:
                if self._yi is None:
                    try:
                        from yiddish_phonikud import engine  # type: ignore
                    except ImportError as e:
                        raise RuntimeError("Yiddish needs the phonikud-yi engine (yiddish_phonikud)") from e
                    engine.load()
                    self._yi = engine.text_to_ipa
        return self._yi(text)

    def _russian(self, text: str) -> str:
        if self._ru is not None:
            return self._ru(text)
        if "ru" in CORE_LANGS:
            return self.engine.phonemize(text, "ru")
        raise RuntimeError("this build has no Russian core; pass russian=callable")

    # ------------------------------------------------------------ API
    def phonemize_segment(self, text: str, lang: str) -> str:
        lang = norm_lang(lang)
        text = _TAG.sub("", text).strip()
        if not text:
            return ""
        if lang == "he" or (lang not in ("yi",) and _HEBREW.search(text) and lang not in CORE_LANGS):
            # the native Hebrew core wins when this build has it and no override was passed
            if self._he is None and "he" in CORE_LANGS:
                out = self.engine.phonemize(text, "he")
                if out:  # empty = the Rust core has no he.g2p in its data dir
                    return out
            return self._hebrew(text)
        if lang == "yi":
            return self._yiddish(text)
        if lang == "ru":
            return self._russian(text)
        if lang in CORE_LANGS:
            return self.engine.phonemize(text, lang)
        raise ValueError(f"unsupported language: {lang} (supported: {', '.join(SUPPORTED_LANGS)})")

    def phonemize(self, text: str, lang: str = "en", tags: bool = False) -> str:
        """Phonemize ``text``. Inline ``<xx>…</xx>`` spans switch language.

        With ``tags=True`` each segment is wrapped as ``<lang>…</lang>`` (BlueTTS style).
        """
        lang = norm_lang(lang)
        pieces: list[tuple[str, str]] = []
        last = 0
        for m in _INLINE.finditer(text):
            if m.start() > last:
                pieces.append((lang, text[last:m.start()]))
            pieces.append((norm_lang(m.group(1)), m.group(2)))
            last = m.end()
        if last < len(text):
            pieces.append((lang, text[last:]))
        out = []
        for lg, seg in pieces:
            ph = self.phonemize_segment(seg, lg)
            if ph:
                out.append(f"<{lg}>{ph}</{lg}>" if tags else ph)
        return "".join(out) if tags else " ".join(out)

    __call__ = phonemize

    def phonemize_batch(self, texts: list[str], lang: str = "en") -> list[str]:
        lang = norm_lang(lang)
        if lang in CORE_LANGS and not any(_INLINE.search(t) for t in texts):
            return self.engine.phonemize_batch(texts, lang)
        return [self.phonemize(t, lang) for t in texts]
