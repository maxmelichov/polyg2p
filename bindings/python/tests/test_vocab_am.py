"""The symbols the Amharic engine writes are all in the vocabulary (the ejective mark ʼ and ʷ included).

    python -m unittest discover bindings/python/tests
"""
import importlib.util
import os
import unittest

_spec = importlib.util.spec_from_file_location("vocab", os.path.join(os.path.dirname(__file__), "..", "python", "polyg2p", "vocab.py"))
vocab = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(vocab)

# every symbol src/am.rs can produce after fold.rs (tests/am.rs: output_symbols)
AM = "əuiaeɨo hlmsrʃkbvtnɲʔwzʒjdɡŋpfʧʤʼʷ.,?!:"


class AmharicVocab(unittest.TestCase):
    def test_symbols(self):
        self.assertEqual(vocab.unknown_symbols(AM), [])

    def test_ejective_mark(self):
        self.assertIn("ʼ", vocab.VOCAB)
        self.assertEqual(vocab.unknown_symbols("tʼəʤ kʼʷankʼʷa sʼəhaj ʧʼəw pʼapʼas"), [])


if __name__ == "__main__":
    unittest.main()
