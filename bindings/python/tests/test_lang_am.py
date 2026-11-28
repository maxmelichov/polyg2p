"""Amharic language codes: the router resolves every spelling the Rust core accepts (Lang::parse: am, amh, am-et, am_et).

Runs without the compiled core: a stub stands in for polyg2p._polyg2p.
    python -m unittest discover bindings/python/tests
"""
import os
import sys
import types
import unittest
from unittest import mock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

if "polyg2p._polyg2p" not in sys.modules:
    _stub = types.ModuleType("polyg2p._polyg2p")
    _stub.ALL_LANGS = _stub.CORE_LANGS = ("en", "am")
    _stub.Engine = type("Engine", (), {"__init__": lambda self, *a: None, "phonemize": lambda self, text, lang: f"{lang}:{text}"})
    sys.modules["polyg2p._polyg2p"] = _stub

import polyg2p  # noqa: E402


class AmharicCodes(unittest.TestCase):
    def test_aliases(self):
        for code in ("am", "AM", "amh", "AMH", "am-et", "am-ET", "am_ET", "am_et", " am "):
            self.assertEqual(polyg2p.norm_lang(code), "am", code)

    def test_inline_alias_is_rewritten(self):
        g = polyg2p.Phonemizer.__new__(polyg2p.Phonemizer)
        g._he = g._yi = g._ru = None
        seen = []

        class Engine:
            def phonemize(self, text, lang):
                seen.append(lang)
                return text

        g.engine = Engine()
        with mock.patch.object(polyg2p, "CORE_LANGS", ("en", "am")):
            self.assertEqual(g.phonemize("<amh>x</amh> z <am_ET>y</am_ET>", "am-et", tags=True), "<am>x</am><am>z</am><am>y</am>")
        self.assertEqual(seen, ["am", "am", "am"])


if __name__ == "__main__":
    unittest.main()
