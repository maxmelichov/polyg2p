"""Phonemizer.readings forwards to the engine; runs without the compiled core (stub).
    python -m unittest discover bindings/python/tests
"""
import os
import sys
import types
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

if "polyg2p._polyg2p" not in sys.modules:
    _stub = types.ModuleType("polyg2p._polyg2p")
    _stub.ALL_LANGS = _stub.CORE_LANGS = ("en",)
    _stub.Engine = type("Engine", (), {"__init__": lambda self, *a: None})
    sys.modules["polyg2p._polyg2p"] = _stub

import polyg2p  # noqa: E402


class Readings(unittest.TestCase):
    def test_forwarded(self):
        g = polyg2p.Phonemizer.__new__(polyg2p.Phonemizer)
        g.engine = types.SimpleNamespace(readings=lambda text, lang: [(i, "x1") for i, c in enumerate(text) if c == "行"])
        self.assertEqual(g.readings("银行行长", "zh"), [(1, "x1"), (2, "x1")])


if __name__ == "__main__":
    unittest.main()
