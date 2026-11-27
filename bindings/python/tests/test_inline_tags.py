"""Inline <xx>...</xx> tags: a known code switches language, an unknown one is refused.

Runs without the compiled core: a stub stands in for polyg2p._polyg2p.
    python -m unittest discover bindings/python/tests
"""
import os
import sys
import types
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

_CORE = ("en", "en-gb", "es")
_stub = types.ModuleType("polyg2p._polyg2p")
_stub.ALL_LANGS = _CORE
_stub.CORE_LANGS = _CORE


class _Engine:
    def __init__(self, *_a):
        pass

    def phonemize(self, text, lang):
        return f"{lang}:{text}"


_stub.Engine = _Engine
sys.modules["polyg2p._polyg2p"] = _stub

import polyg2p  # noqa: E402


def make():
    g = polyg2p.Phonemizer.__new__(polyg2p.Phonemizer)
    g.engine = _Engine()
    g._he = g._yi = g._ru = None
    return g


class InlineTags(unittest.TestCase):
    def test_known_codes_switch(self):
        g = make()
        out = g.phonemize("a <es>b</es> c <en-gb>d</en-gb>", "en", tags=True)
        self.assertEqual(out, "<en>en:a</en><es>es:b</es><en>en:c</en><en-gb>en-gb:d</en-gb>")

    def test_unknown_codes_are_refused(self):
        g = make()
        for text in ["x <spa>hola</spa> y", "x <en-za>far</en-za> y", "x <en-xx>far</en-xx> y", "x <b>bold</b> y"]:
            with self.assertRaises(ValueError, msg=text):
                g.phonemize(text, "en")


if __name__ == "__main__":
    unittest.main()
