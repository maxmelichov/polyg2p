"""Homograph test: sentence, word, expected-substring, wrong-substring (espeak notation, no stress)."""
import os, subprocess, sys, unicodedata
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CASES = [
    ("I read the book yesterday after I had read the reviews.", "had read", "ɹɛd", None),
    ("She has read every letter.", "read", "ɹɛd", "ɹiːd"),
    ("I like to read before bed.", "read", "ɹiːd", "ɹɛd"),
    ("Please read this aloud.", "read", "ɹiːd", "ɹɛd"),
    ("We have read about it.", "read", "ɹɛd", "ɹiːd"),
    ("They will record the album next week.", "record", "ɹɪkɔːɹd", "ɹɛkɚd"),
    ("He broke the world record.", "record", "ɹɛkɚd", "ɹɪkɔːɹd"),
    ("Keep a record of your expenses.", "record", "ɹɛkɚd", "ɹɪkɔːɹd"),
    ("I live in Boston.", "live", "lɪv", "laɪv"),
    ("They watched a live concert.", "live", "laɪv", "lɪv"),
    ("Where do you live now?", "live", "lɪv", "laɪv"),
    ("The show is broadcast live music every night.", "live", "laɪv", "lɪv"),
    ("Please close the door.", "close", "kloʊz", "kloʊs"),
    ("The station is very close to my house.", "close", "kloʊs", "kloʊz"),
    ("We close at nine.", "close", "kloʊz", "kloʊs"),
    ("She is a close friend.", "close", "kloʊs", "kloʊz"),
    ("A single tear rolled down her cheek.", "tear", "tɪɹ", "tɛɹ"),
    ("Don't tear the paper.", "tear", "tɛɹ", "tɪɹ"),
    ("He was in tears.", "tears", "tɪɹz", "tɛɹz"),
    ("They tear down old buildings.", "tear", "tɛɹ", "tɪɹ"),
    ("The pipes were made of lead.", "lead", "lɛd", "liːd"),
    ("She will lead the team.", "lead", "liːd", "lɛd"),
    ("Old houses often have lead paint.", "lead", "lɛd", "liːd"),
    ("What is the use of that?", "use", "juːs", "juːz"),
    ("I use it every day.", "use", "juːz", "juːs"),
    ("We need to use less water.", "use", "juːz", "juːs"),
    ("The wind is strong today.", "wind", "wɪnd", "waɪnd"),
    ("You have to wind the clock.", "wind", "waɪnd", "wɪnd"),
    ("They present their findings today.", "present", "zɛnt", "zənt"),
    ("I bought her a present.", "present", "zənt", "zɛnt"),
    ("The students will present their projects.", "present", "zɛnt", "zənt"),
    ("We object to the plan.", "object", "əbʤɛkt", "ɑːbʤɪkt"),
    ("A heavy object fell.", "object", "ɑːbʤ", "əbʤ"),
    ("They increase prices every year.", "increase", "ɪnkɹiːs", None),
    ("Can you estimate the cost?", "estimate", "meɪt", "mət"),
    ("That is a rough estimate.", "estimate", "mət", "meɪt"),
    ("Please separate the eggs.", "separate", "ɹeɪt", "ɹɪt"),
    ("They live in separate rooms.", "separate", "ɹɪt", "ɹeɪt"),
    ("I refuse to go.", "refuse", "ɹɪfjuːz", "ɹɛfjuːs"),
    ("Can you excuse me?", "excuse", "ɪkskjuːz", "ɪkskjuːs"),
]


def norm(s):
    s = s.replace("ᵻ", "ɪ")
    s = unicodedata.normalize("NFC", s).replace("ˈ", "").replace("ˌ", "")
    return s.replace("ʤ", "dʒ").replace("ʧ", "tʃ")


def check(outs):
    ok = 0
    bad = []
    for (sent, w, good, wrong), o in zip(CASES, outs):
        o = norm(o)
        g = norm(good)
        hit = g in o and (wrong is None or norm(wrong) not in o)
        ok += hit
        if not hit:
            bad.append((sent, o))
    return ok, bad


ours = subprocess.run([os.path.join(ROOT, "target", "release", "g2p"), "en", "--data", sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "data-out")],
                      input="\n".join(c[0] for c in CASES) + "\n", capture_output=True, text=True).stdout.split("\n")
o_ok, o_bad = check(ours)
print(f"polyg2p: {o_ok}/{len(CASES)}")
try:
    import espeakng_loader
    from phonemizer.backend.espeak.wrapper import EspeakWrapper
    EspeakWrapper.set_library(espeakng_loader.get_library_path()); EspeakWrapper.set_data_path(espeakng_loader.get_data_path())
    from phonemizer.backend import EspeakBackend
    from phonemizer.separator import Separator
    b = EspeakBackend("en-us", with_stress=True, preserve_punctuation=True)
    esp = b.phonemize([c[0] for c in CASES], separator=Separator(phone="", word=" "), strip=True)
    e_ok, e_bad = check(esp)
    print(f"espeak : {e_ok}/{len(CASES)}")
except ImportError:
    pass
for s, o in o_bad:
    print("  ours wrong:", s, "->", o)
