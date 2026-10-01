#!/usr/bin/env python3
"""Identifiers are English, always (contribuindo/08, author decision 2026-09-25/2026-10-01).

Every identifier in Rust, C++, QML/JS, Python and shell is split into words
(camelCase, PascalCase, snake_case, SCREAMING_CASE). Each word of 4+ letters
must be in the system English dictionary or in the project allowlist of
technical terms (scripts/identifier-language-allowlist.txt). Comments and
string literals are stripped first: comments and on-screen text are Portuguese
by rule, and are not identifiers.

Legacy: the Portuguese names that existed on 2026-10-01 are frozen in
scripts/identifier-language-baseline.txt (file, word, count). A ratchet, like
the other gates: a count may only go DOWN; a new word in a file, or a higher
count, fails. The sweep (roadmap 53 §G0) brings the baseline to zero.

Usage:
    check_identifier_language.py                    # gate
    check_identifier_language.py --report           # unknown words with counts
    check_identifier_language.py --update-baseline  # rewrite (only shrinks)
"""
from __future__ import annotations

import io
import re
import subprocess
import tokenize
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST = ROOT / "scripts" / "identifier-language-allowlist.txt"
BASELINE = ROOT / "scripts" / "identifier-language-baseline.txt"
DICTIONARIES = (Path("/usr/share/dict/american-english"), Path("/usr/share/dict/words"))
MIN_WORD = 4

SUFFIXES = {
    ".rs": "c", ".cpp": "c", ".h": "c", ".qml": "c", ".js": "c",
    ".py": "python", ".sh": "shell",
}
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
WORD = re.compile(r"[A-Z]+(?=[A-Z][a-z])|[A-Z]?[a-z]+|[A-Z]+")


def load_english() -> set[str]:
    for path in DICTIONARIES:
        if path.exists():
            words = set()
            for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
                word = line.strip().lower()
                if word.isascii() and word.isalpha():
                    words.add(word.removesuffix("'s"))
            return words
    sys.exit("erro: dicionario de ingles ausente (/usr/share/dict/american-english);"
             " instale o pacote wamerican")


def load_allowlist() -> set[str]:
    if not ALLOWLIST.exists():
        return set()
    words = set()
    for line in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        words.update(line.split("#", 1)[0].lower().split())
    return words


def strip_c_like(text: str) -> str:
    """Remove // and /* */ comments and "..." '...' `...` literals, keeping lines."""
    out = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            j = text.find("\n", i)
            i = n if j < 0 else j
        elif c == "/" and nxt == "*":
            j = text.find("*/", i + 2)
            chunk = text[i:(n if j < 0 else j + 2)]
            out.append("\n" * chunk.count("\n"))
            i = n if j < 0 else j + 2
        elif c == "r" and re.match(r'b?r#*"', text[i:i + 8]) and (i == 0 or not text[i - 1].isalnum() and text[i - 1] != "_"):
            m = re.match(r'(b?r)(#*)"', text[i:])
            close = '"' + m.group(2)
            j = text.find(close, i + m.end())
            chunk = text[i:(n if j < 0 else j + len(close))]
            out.append("\n" * chunk.count("\n"))
            i = n if j < 0 else j + len(close)
        elif c in "\"'`":
            # Rust lifetimes ('a) and char literals: treat 'x' only when closed nearby.
            if c == "'" and not re.match(r"'(\\.|[^'\\])'", text[i:i + 4]):
                out.append(c)
                i += 1
                continue
            j = i + 1
            while j < n and text[j] != c:
                if text[j] == "\\":
                    j += 1
                elif text[j] == "\n" and c != "`":
                    break
                j += 1
            chunk = text[i:j + 1]
            out.append("\n" * chunk.count("\n"))
            i = j + 1
        else:
            out.append(c)
            i += 1
    return "".join(out)


def python_names(text: str) -> list[tuple[int, str]]:
    """NAME tokens only: docstrings, comments and strings never reach the check."""
    names = []
    try:
        for tok in tokenize.generate_tokens(io.StringIO(text).readline):
            if tok.type == tokenize.NAME:
                names.append((tok.start[0], tok.string))
    except (tokenize.TokenError, IndentationError, SyntaxError):
        pass
    return names


SHELL_NAME = re.compile(
    r"(?:^|[\s;(])(?:local\s+|export\s+|readonly\s+|declare\s+-?\w*\s+)?([A-Za-z_]\w*)(?=\+?=)"
    r"|\$\{?([A-Za-z_]\w*)"
    r"|^\s*(?:function\s+)?([A-Za-z_][\w-]*)\s*\(\)"
    r"|\bfor\s+([A-Za-z_]\w*)\s+in\b")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_]\w*)\1")


def shell_names(text: str) -> list[tuple[int, str]]:
    """Variable and function names. A heredoc fed to Python is checked as Python;
    any other heredoc body (plain text, file content) is skipped."""
    names, end_marker, body, body_start, body_is_python = [], None, [], 0, False
    for lineno, line in enumerate(text.splitlines(), 1):
        if end_marker is not None:
            if line.strip() == end_marker:
                if body_is_python:
                    names.extend((body_start + n - 1, name)
                                 for n, name in python_names("\n".join(body)))
                end_marker, body = None, []
            else:
                body.append(line)
            continue
        code = strip_hash(line)
        for groups in SHELL_NAME.findall(code):
            for name in groups:
                if name:
                    names.append((lineno, name))
        # On the raw line: stripping quotes would erase the `'PY'` marker.
        m = HEREDOC.search(line)
        if m and "#" not in line[: m.start()]:
            end_marker, body_start = m.group(2), lineno + 1
            body_is_python = re.search(r"\bpython3?\b", line[: m.start()]) is not None
    return names


def strip_hash(text: str) -> str:
    """Remove # comments and quoted strings (shell/python), keeping lines."""
    lines = []
    for line in text.splitlines():
        cleaned, quote, k = [], None, 0
        while k < len(line):
            ch = line[k]
            if quote:
                if ch == "\\":
                    k += 2
                    continue
                if ch == quote:
                    quote = None
                k += 1
                continue
            if ch in "\"'":
                quote = ch
            elif ch == "#" and (k == 0 or line[k - 1] in " \t;"):
                break
            else:
                cleaned.append(ch)
            k += 1
        lines.append("".join(cleaned))
    return "\n".join(lines)


def tracked_files() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    files = []
    for name in out.split("\0"):
        if not name or name.startswith(("KV0.", "dist/", "build/", "target/", "DocsPublic/", "DocsPrivate/")):
            continue
        path = ROOT / name
        if path.suffix in SUFFIXES and path.is_file():
            files.append(path)
    return sorted(files)


def words_of(identifier: str) -> list[str]:
    return [w.lower() for w in WORD.findall(identifier)]


def load_baseline() -> dict[tuple[str, str], int]:
    frozen = {}
    if BASELINE.exists():
        for line in BASELINE.read_text(encoding="utf-8").splitlines():
            if line and not line.startswith("#"):
                rel, word, count = line.split("\t")
                frozen[(rel, word)] = int(count)
    return frozen


def main() -> int:
    report = "--report" in sys.argv
    update = "--update-baseline" in sys.argv
    english = load_english()
    allowed = load_allowlist()
    unknown: Counter[str] = Counter()
    per_file: Counter[tuple[str, str]] = Counter()
    where: dict[str, set[str]] = defaultdict(set)
    for path in tracked_files():
        text = path.read_text(encoding="utf-8", errors="ignore")
        kind = SUFFIXES[path.suffix]
        if kind == "python":
            pairs = python_names(text)
        elif kind == "shell":
            pairs = shell_names(text)
        else:
            pairs = [(lineno, ident)
                     for lineno, line in enumerate(strip_c_like(text).splitlines(), 1)
                     for ident in IDENT.findall(line)]
        rel = str(path.relative_to(ROOT))
        for lineno, ident in pairs:
                for word in words_of(ident):
                    if len(word) < MIN_WORD or word in english or word in allowed:
                        continue
                    # Inflections the dictionary may lack (plural, -ed, -ing).
                    if any(word.endswith(s) and word[: -len(s)] in english
                           for s in ("s", "es", "ed", "ing", "er", "ers", "able")):
                        continue
                    unknown[word] += 1
                    where[word].add(f"{rel}:{lineno}")
                    per_file[(rel, word)] += 1
    if report:
        for word, count in unknown.most_common():
            sample = sorted(where[word])[:2]
            print(f"{count:5d}  {word:24s} {' '.join(sample)}")
        print(f"\n{len(unknown)} palavras fora do ingles, {sum(unknown.values())} ocorrencias")
        return 0
    frozen = load_baseline()
    grown = sorted(key for key, count in per_file.items() if count > frozen.get(key, 0))
    if update:
        if grown and BASELINE.exists():
            print("erro: a linha de base so' encolhe; ha' palavra nova ou contagem maior:",
                  file=sys.stderr)
            for rel, word in grown[:40]:
                print(f"  {rel}: `{word}`", file=sys.stderr)
            return 1
        lines = ["# Identificadores em portugues CONGELADOS em 2026-10-01 (catraca: so' descem).",
                 "# arquivo<TAB>palavra<TAB>ocorrencias. Gerado por"
                 " check_identifier_language.py --update-baseline."]
        lines += [f"{rel}\t{word}\t{count}" for (rel, word), count in sorted(per_file.items())]
        BASELINE.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"linha de base: {len(per_file)} pares arquivo/palavra,"
              f" {sum(per_file.values())} ocorrencias")
        return 0
    if grown:
        print("erro: identificador fora do ingles (contribuindo/08: identificadores"
              " sempre em ingles). Novo ou acima da linha de base:", file=sys.stderr)
        for rel, word in grown[:40]:
            locs = sorted(loc for loc in where[word] if loc.startswith(rel + ":"))[:3]
            print(f"  {', '.join(locs)}: `{word}`", file=sys.stderr)
        print("  Traduza o nome. Se for termo tecnico ingles legitimo (sigla, nome"
              " de ferramenta), acrescente-o a scripts/identifier-language-allowlist.txt"
              " com o motivo.", file=sys.stderr)
        return 1
    shrinkable = sum(1 for key, count in frozen.items() if per_file.get(key, 0) < count)
    remaining = sum(per_file.values())
    print(f"idioma dos identificadores: nada novo em portugues; legado congelado:"
          f" {remaining} ocorrencias"
          + (f" ({shrinkable} pares ja' encolheram: rode --update-baseline)" if shrinkable else "."))
    return 0


if __name__ == "__main__":
    sys.exit(main())
