"""Drop-in replacement for BlueTTS's ``TextProcessor`` (src/blue_onnx/__init__.py).

    # before: from blue_onnx import TextProcessor   (renikud + espeak)
    from polyg2p.bluetts import TextProcessor        # renikud + polyg2p, no espeak

Same constructor and ``phonemize(text, lang)`` contract: inline ``<xx>…</xx>``
spans are honoured and the result is re-wrapped in ``<lang>…</lang>`` tags so
BlueTTS's ``UnicodeProcessor`` can route segments as before.
"""

from __future__ import annotations

from typing import Optional

from . import Phonemizer


class TextProcessor:
    def __init__(self, renikud_path: Optional[str] = None, speaker: int = 0, target_speaker: int = 0, data_dir=None):
        hebrew = None
        if renikud_path:
            from renikud_onnx import G2P  # type: ignore

            g = G2P(renikud_path)
            hebrew = lambda t: g.phonemize(t, speaker=speaker, target_speaker=target_speaker)  # noqa: E731
        self._p = Phonemizer(data_dir=data_dir, hebrew=hebrew, hebrew_speaker=speaker, hebrew_target_speaker=target_speaker)

    def _phonemize_segment(self, content: str, lang: str) -> str:
        return self._p.phonemize_segment(content, lang)

    def phonemize(self, text: str, lang: str = "he") -> str:
        return self._p.phonemize(text, lang, tags=True)
