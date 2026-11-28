"""Finnish through the compiled core: inflected numerals, gemination by context.

Skipped when the extension is not built (maturin develop --release).
    python -m unittest discover bindings/python/tests
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

try:
    from polyg2p import _polyg2p
except ImportError:  # extension not built
    _polyg2p = None


@unittest.skipIf(_polyg2p is None or not hasattr(_polyg2p, "Engine"), "compiled core not available")
class Finnish(unittest.TestCase):
    def setUp(self):
        self.e = _polyg2p.Engine(None)

    def test_inflected_numeral(self):
        self.assertEqual(self.e.phonemize("5:ssä", "fi"), "vˈiːdessæ")

    def test_infinitive_only_after_governor(self):
        self.assertEqual(self.e.phonemize("Tom pitää siitä.", "fi"), "tˈom pˈitæː sˈiːtæ.")
        self.assertEqual(self.e.phonemize("Voisitko antaa hänelle tämän?", "fi"), "vˈoisɪtko ˈantaːh hˈænelle tːˈæmæn?")

    def test_long_number_does_not_panic(self):
        self.assertTrue(self.e.phonemize("9" * 40, "uk"))


if __name__ == "__main__":
    unittest.main()
