"""Prova o gate de idioma igual em toda distro (roadmaps/59 §2.6, 2026-10-15).

O veredito vem do vocabulario do repositorio, nunca do dicionario do sistema:
sem dicionario nenhum e com um dicionario que aceita `plano` (como o `words`
do Arch), a saida e' a mesma. O dicionario so' sugere o `--accept`.
"""
import contextlib
import importlib.util
import io
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "identifier_language", REPO_ROOT / "scripts" / "check_identifier_language.py")
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def run(*args: str) -> tuple[int, str, str]:
    out, err = io.StringIO(), io.StringIO()
    saved = sys.argv
    sys.argv = ["check_identifier_language.py", *args]
    try:
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = gate.main()
    finally:
        sys.argv = saved
    return code, out.getvalue(), err.getvalue()


class SameVerdictOnEveryDistro(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="kinein-ident-")
        self.saved = (gate.DICTIONARIES, gate.VOCABULARY)
        copy = Path(self.scratch.name) / "vocabulary.txt"
        shutil.copy(gate.VOCABULARY, copy)
        gate.VOCABULARY = copy

    def tearDown(self):
        gate.DICTIONARIES, gate.VOCABULARY = self.saved
        self.scratch.cleanup()

    def dictionary(self, words: list[str]) -> tuple[Path, ...]:
        path = Path(self.scratch.name) / f"words-{len(words)}"
        path.write_text("\n".join(words) + "\n", encoding="utf-8")
        return (path,)

    def test_no_dictionary_and_a_larger_one_give_the_same_verdict(self):
        gate.DICTIONARIES = (Path(self.scratch.name) / "absent",)
        without = run()
        gate.DICTIONARIES = self.dictionary(["plano", "para", "lidos", "lido"])
        larger = run()
        self.assertEqual(without, larger)

    def test_a_new_portuguese_identifier_fails_and_an_english_one_suggests_accept(self):
        gate.DICTIONARIES = self.dictionary(["zymurgy"])
        with tempfile.TemporaryDirectory(prefix="kinein-ident-", dir=REPO_ROOT) as directory:
            source = Path(directory) / "sample.py"
            source.write_text("resposta_nova = 1\n", encoding="utf-8")
            code, _, err = run()
            self.assertEqual(code, 1)
            self.assertIn("`resposta`", err)
            self.assertNotIn("--accept resposta", err)

            source.write_text("zymurgy_level = 1\n", encoding="utf-8")
            code, _, err = run()
            self.assertEqual(code, 1)
            self.assertIn("--accept zymurgy", err)

            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(gate.accept(["zymurgy"]), 0)
            self.assertIn("zymurgy", gate.load_vocabulary())
            code, _, _ = run()
            self.assertEqual(code, 0)

    def test_accept_refuses_what_this_dictionary_does_not_know(self):
        gate.DICTIONARIES = self.dictionary(["zymurgy"])
        before = gate.VOCABULARY.read_text(encoding="utf-8")
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(gate.accept(["plano"]), 1)
        self.assertEqual(gate.VOCABULARY.read_text(encoding="utf-8"), before)


if __name__ == "__main__":
    unittest.main()
