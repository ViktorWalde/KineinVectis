#!/usr/bin/env python3
"""Catraca de derivacao duplicada na UI QML.

Ver scripts/verificar-qml-duplicacao.sh para o porque.
"""
from __future__ import annotations

import collections
import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
QML = RAIZ / "ui" / "qml"
BASELINE = RAIZ / "scripts" / "qml-duplicacao-baseline.txt"

# `campo === "valor"`: a UI derivando algo de um valor de dominio.
PADRAO = re.compile(r'(\w+)\s*===\s*"([a-z][a-zA-Z]+)"')


def medir() -> dict[str, set[str]]:
    onde: dict[str, set[str]] = collections.defaultdict(set)
    for arquivo in sorted(QML.rglob("*.qml")):
        texto = arquivo.read_text(encoding="utf-8")
        for m in PADRAO.finditer(texto):
            onde[f"{m.group(1)}==={m.group(2)}"].add(arquivo.name)
    return {k: v for k, v in onde.items() if len(v) > 1}


def ler_baseline() -> dict[str, int]:
    if not BASELINE.is_file():
        return {}
    congelado = {}
    for linha in BASELINE.read_text(encoding="utf-8").splitlines():
        if not linha.strip():
            continue
        chave, quantidade = linha.rsplit(" ", 1)
        congelado[chave] = int(quantidade)
    return congelado


def escrever_baseline(atual: dict[str, set[str]]) -> None:
    corpo = "".join(f"{k} {len(v)}\n" for k, v in sorted(atual.items()))
    BASELINE.write_text(corpo, encoding="utf-8")


def main() -> int:
    if "--atualizar-baseline" in sys.argv:
        atual = medir()
        escrever_baseline(atual)
        print(f"baseline atualizada: {len(atual)} derivacoes duplicadas")
        print("Commite junto com a mudanca que a alterou.")
        return 0

    atual = medir()
    congelado = ler_baseline()
    problemas: list[str] = []

    for chave, arquivos in sorted(atual.items()):
        antes = congelado.get(chave)
        if antes is None:
            problemas.append(
                f"✗ NOVA derivacao duplicada: `{chave}` em {len(arquivos)} arquivos "
                f"({', '.join(sorted(arquivos))})"
            )
        elif len(arquivos) > antes:
            problemas.append(
                f"✗ ESPALHOU: `{chave}` foi de {antes} para {len(arquivos)} arquivos "
                f"({', '.join(sorted(arquivos))})"
            )

    if problemas:
        print("catraca de duplicacao QML FALHOU:", file=sys.stderr)
        for p in problemas:
            print(f"  {p}", file=sys.stderr)
        print(
            "\nA MESMA derivacao em dois arquivos diverge em silencio. Foi assim que\n"
            "`severity` desconhecida ficou AZUL no ProblemsPanel e VERMELHA na\n"
            "EditorGutter, sem nada reclamar.\n"
            "Se sao o MESMO fato: um dono so' (ex.: StatusColors).\n"
            "Se sao fatos DIFERENTES que so' compartilham a string, rode:\n"
            "  python3 scripts/verificar_qml_duplicacao.py --atualizar-baseline",
            file=sys.stderr,
        )
        return 1

    print(f"duplicacao QML: {len(atual)} derivacoes congeladas, nenhuma nova nem espalhada.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
