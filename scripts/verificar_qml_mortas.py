#!/usr/bin/env python3
"""Funcao QML declarada que NINGUEM menciona.

POR QUE ESTE GATE EXISTE (2026-09-26). O Rust recusa `dead_code` e o clang-tidy
olha o C++; o QML nao tinha nada equivalente. Tres coisas mortas atravessaram
uma fatia inteira sem nenhum gate reclamar — `levelsOf`, `pending` e um campo
`dedentTo` que ia do Rust ao QML sem consumidor — e so' foram achadas porque o
autor PERGUNTOU se havia codigo morto.

DUAS CATEGORIAS, e a diferenca importa:

  MORTA          ninguem menciona, em lugar nenhum. Apagar costuma ser o certo.
  SO' NO HARNESS o produto nao usa; o teste usa. Pior que morta: o harness a
                 faz PARECER viva, e o verde do gate vira falsa garantia.

E uma terceira leitura, que o gate NAO decide: funcao que devia estar sendo
usada. `setBreakpointCondition` existe porque o core aceita condicao em
breakpoint; ninguem na UI chama. Apagar seria jogar fora a metade feita. Quem
decide entre LIGAR e APAGAR e' o autor — este gate so' recusa que a duvida fique
invisivel.

A DETECCAO E' POR MENCAO, e nao por chamada: `Qt.callLater(takeFocus)` usa a
funcao sem parenteses, e a primeira versao disto a acusou de morta. Medido: dois
falsos positivos em seis conferidos. Gate que acusa o que nao existe custa mais
confianca que gate nenhum — entao a regra ficou LARGA de proposito, e erra para
o silencio.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
BASELINE = RAIZ / "scripts" / "qml-mortas-baseline.txt"


def ler_todos(pasta: Path, sufixo: str) -> dict[Path, str]:
    return {p: p.read_text(encoding="utf-8", errors="replace") for p in sorted(pasta.rglob(sufixo))}


def encontrar() -> tuple[list[str], list[str]]:
    qml = ler_todos(RAIZ / "ui/qml", "*.qml")
    fora = "\n".join(ler_todos(RAIZ / "ui/qml", "*.js").values())
    fora += "\n".join(ler_todos(RAIZ / "ui/src", "*.cpp").values())
    fora += "\n".join(ler_todos(RAIZ / "ui/src", "*.h").values())
    harness = "\n".join(ler_todos(RAIZ / "scripts/qml-harness", "*.qml").values())

    mortas: list[str] = []
    so_harness: list[str] = []
    for caminho, texto in qml.items():
        for achado in re.finditer(r"^    function ([a-zA-Z_][a-zA-Z0-9_]*)\(", texto, re.M):
            nome = achado.group(1)
            mencao = re.compile(r"\b" + re.escape(nome) + r"\b")
            definicoes = len(re.findall(r"^    function " + re.escape(nome) + r"\(", texto, re.M))
            if len(mencao.findall(texto)) > definicoes:
                continue
            if any(mencao.search(outro) for p, outro in qml.items() if p != caminho):
                continue
            if mencao.search(fora):
                continue
            alvo = so_harness if mencao.search(harness) else mortas
            alvo.append(f"{caminho.relative_to(RAIZ)}::{nome}")
    return sorted(mortas), sorted(so_harness)


def main() -> int:
    print("== funcoes QML sem consumidor (mortas ou so' no harness) ==")
    mortas, so_harness = encontrar()
    atuais = set(mortas) | set(so_harness)

    if "--atualizar-baseline" in sys.argv:
        BASELINE.write_text("\n".join(sorted(atuais)) + "\n", encoding="utf-8")
        print(f"baseline atualizada: {len(atuais)} entrada(s).")
        return 0

    congeladas = set()
    if BASELINE.exists():
        congeladas = {
            linha.strip()
            for linha in BASELINE.read_text(encoding="utf-8").splitlines()
            if linha.strip() and not linha.startswith("#")
        }

    novas = sorted(atuais - congeladas)
    if novas:
        print("catraca de funcoes mortas FALHOU:", file=sys.stderr)
        for entrada in novas:
            categoria = "so' no harness" if entrada in set(so_harness) else "sem consumidor"
            print(f"  ✗ NOVA funcao {categoria}: {entrada}", file=sys.stderr)
        print("", file=sys.stderr)
        print("Escolha uma das tres, e nenhuma delas e' ignorar:", file=sys.stderr)
        print("  LIGAR   se ela devia estar sendo usada (o caso do breakpoint", file=sys.stderr)
        print("          condicional: o core aceita, a UI nunca chamou);", file=sys.stderr)
        print("  APAGAR  se nao devia existir;", file=sys.stderr)
        print("  CONGELAR com decisao registrada:", file=sys.stderr)
        print("          python3 scripts/verificar_qml_mortas.py --atualizar-baseline",
              file=sys.stderr)
        return 1

    resolvidas = sorted(congeladas - atuais)
    if resolvidas:
        print(f"{len(resolvidas)} entrada(s) da baseline nao existem mais — encolher e' aceito;")
        print("rode --atualizar-baseline para limpar.")
    print(f"funcoes QML: {len(mortas)} sem consumidor e {len(so_harness)} so' no harness,")
    print("todas congeladas na baseline; nenhuma nova.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
