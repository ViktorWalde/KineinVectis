"""O texto de um render do terminal, como o gate o le (INF-08).

O ajudante junta so' pela marca `wrapped` do core, com o espaco final que o
render descarta. Os casos sao os do INF-05 no ajudante Rust: a quebra num
espaco, a linha cheia que o programa terminou, a quebra dupla e a linha sem
quebra.
"""
import sys
import unittest
from importlib import import_module
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
debug = import_module("verificar_python_debug")


def row(text: str, cells: int | None = None) -> list[dict]:
    return [{"text": text, "cells": len(text) if cells is None else cells}]


def text_of(lines: list[list[dict]], wrapped: list[int], cols: int = 10) -> str:
    return debug.Core._render_text({"cols": cols, "lines": lines, "wrapped": wrapped})


class RenderTextTests(unittest.TestCase):
    def test_a_break_on_a_space_keeps_the_space(self):
        # "abcdefghi " na coluna 10: o core descarta o espaco final.
        self.assertEqual(text_of([row("abcdefghi"), row("jk")], [0]), "abcdefghi jk")

    def test_a_full_line_the_program_ended_stays_apart(self):
        self.assertEqual(text_of([row("abcdefghij"), row("kl")], []), "abcdefghij\nkl")

    def test_a_double_break_joins_three_rows(self):
        self.assertEqual(text_of([row("abcdefghij"), row("klmno"), row("p")], [0, 1]),
                         "abcdefghijklmno     p")

    def test_a_short_line_and_the_end_of_the_screen(self):
        self.assertEqual(text_of([row("ok"), row(""), row("")], []), "ok")

    def test_spans_are_joined_and_their_cells_counted(self):
        spans = [{"text": "ab", "cells": 2}, {"text": "cd", "cells": 2}]
        self.assertEqual(text_of([spans, row("e")], [0], cols=6), "abcd  e")


if __name__ == "__main__":
    unittest.main()
