#!/usr/bin/env python3
"""Catraca de VALOR LITERAL no QML (0.3.9, roadmap 58 §4.3 e 53 §13.1).

A direcao visual da 0.3.6-0.3.9 (bordas arredondadas, movimento fluido) so'
vira regra do sistema de componentes se raio, fonte, duracao e cor vierem do
`Theme.qml`. Medido em 2026-10-01: 34 raios, 497 tamanhos de fonte, 9 duracoes
e 4 cores escritos a mao fora dele. E' CATRACA, nao proibicao: o que existe
fica congelado em `qml-tokens-baseline.txt`, por arquivo e por categoria, e so'
pode DIMINUIR. Literal novo — num arquivo novo ou a mais num antigo — reprova.

  verificar_qml_tokens.py                        confere
  verificar_qml_tokens.py --atualizar-baseline   regrava (so' se nada cresceu)
"""
from __future__ import annotations

import collections
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
QML_DIR = ROOT / "ui" / "qml"
BASELINE = ROOT / "scripts" / "qml-tokens-baseline.txt"
THEME = "Theme.qml"

# Categoria -> padrao de um literal. `color: "#..."` so' conta fora do Theme,
# que e' onde a cor tem de morar.
PATTERNS = {
    "radius": re.compile(r"\bradius:\s*\d"),
    "fontSize": re.compile(r"\bfont\.pixelSize:\s*\d"),
    "duration": re.compile(r"\bduration:\s*\d"),
    "color": re.compile(r'\bcolor:\s*"#'),
}


def strip_comments(text: str) -> str:
    """Tira `// ...` de cada linha: um literal citado num comentario nao conta."""
    return "\n".join(re.sub(r"//.*$", "", line) for line in text.splitlines())


def measure() -> dict[tuple[str, str], int]:
    counts: dict[tuple[str, str], int] = collections.Counter()
    for path in sorted(QML_DIR.rglob("*.qml")):
        text = strip_comments(path.read_text(encoding="utf-8"))
        rel = path.relative_to(ROOT).as_posix()
        for category, pattern in PATTERNS.items():
            if category == "color" and path.name == THEME:
                continue
            found = len(pattern.findall(text))
            if found:
                counts[(rel, category)] = found
    return dict(counts)


def read_baseline() -> dict[tuple[str, str], int]:
    frozen = {}
    if BASELINE.is_file():
        for line in BASELINE.read_text(encoding="utf-8").splitlines():
            if line.strip() and not line.startswith("#"):
                rel, category, count = line.split("\t")
                frozen[(rel, category)] = int(count)
    return frozen


def write_baseline(current: dict[tuple[str, str], int]) -> None:
    header = ("# Literais de raio, fonte, duracao e cor no QML, CONGELADOS (catraca: so' descem).\n"
              "# arquivo<TAB>categoria<TAB>ocorrencias. Gerado por verificar_qml_tokens.py"
              " --atualizar-baseline.\n")
    body = "".join(f"{rel}\t{cat}\t{n}\n" for (rel, cat), n in sorted(current.items()))
    BASELINE.write_text(header + body, encoding="utf-8")


def totals(counts: dict[tuple[str, str], int]) -> dict[str, int]:
    out = {category: 0 for category in PATTERNS}
    for (_, category), n in counts.items():
        out[category] += n
    return out


def main() -> int:
    current = measure()
    frozen = read_baseline()
    grown = [(key, n, frozen.get(key, 0)) for key, n in sorted(current.items())
             if n > frozen.get(key, 0)]
    if "--atualizar-baseline" in sys.argv:
        # A primeira gravacao congela o que existe; depois, so' encolhe.
        if grown and BASELINE.is_file():
            print("erro: a linha de base so' encolhe; ha' literal novo:", file=sys.stderr)
            for (rel, category), n, before in grown:
                print(f"  {rel}: {category} {before} -> {n}", file=sys.stderr)
            return 1
        write_baseline(current)
        print("linha de base: " + ", ".join(f"{c} {n}" for c, n in totals(current).items()))
        return 0
    if grown:
        print("erro: valor literal novo no QML — use o token do Theme.qml"
              " (radius*, fontSize*, motion*, cores):", file=sys.stderr)
        for (rel, category), n, before in grown:
            print(f"  {rel}: {category} {before} -> {n}", file=sys.stderr)
        return 1
    shrunk = sum(1 for key, n in frozen.items() if current.get(key, 0) < n)
    summary = ", ".join(f"{c} {n}" for c, n in totals(current).items())
    print(f"tokens QML: nenhum literal novo; congelados: {summary}"
          + (f" ({shrunk} pares ja' encolheram: rode --atualizar-baseline)" if shrunk else "."))
    return 0


if __name__ == "__main__":
    sys.exit(main())
