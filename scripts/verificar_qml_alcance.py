#!/usr/bin/env python3
"""Componente QML que o modulo ENTREGA e ninguem instancia.

Ver scripts/verificar-qml-alcance.sh para o porque.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
QML = RAIZ / "ui" / "qml"
CPP = RAIZ / "ui" / "src"
CMAKE = RAIZ / "ui" / "CMakeLists.txt"

# A raiz do modulo: quem a instancia e o C++, com QQmlApplicationEngine.
RAIZ_DO_MODULO = "Main"

# `qml/sim/SimPanel.qml` dentro do bloco QML_FILES.
ENTREGUE = re.compile(r"^\s*(qml/[\w/]+\.qml)\s*$", re.MULTILINE)


def entregues() -> dict[str, Path]:
    """Os .qml que o `qt_add_qml_module` empacota, por nome de tipo."""
    texto = CMAKE.read_text(encoding="utf-8")
    saida: dict[str, Path] = {}
    for caminho in ENTREGUE.findall(texto):
        arquivo = RAIZ / "ui" / caminho
        if arquivo.is_file():
            saida[arquivo.stem] = arquivo
    return saida


# O `pragma` vem DEPOIS do cabecalho de comentario nos arquivos deste
# repositorio, entao ele se procura por linha e nao no inicio do texto.
SINGLETON = re.compile(r"^pragma Singleton\b", re.MULTILINE)


def alcancado(nome: str, arquivo: Path, fontes: dict[Path, str], cpp: str) -> bool:
    singleton = SINGLETON.search(fontes[arquivo]) is not None
    # Singleton nao se instancia: usa-se pelo nome (`Theme.accent`).
    instanciacao = re.compile(r"(?:^|[^\w.])" + re.escape(nome) + r"\s*\{")
    uso = re.compile(r"(?:^|[^\w.])" + re.escape(nome) + r"\s*\.")
    for outro, texto in fontes.items():
        if outro == arquivo:
            continue
        if singleton:
            if uso.search(texto):
                return True
        elif instanciacao.search(texto):
            return True
    # O C++ pode carregar um tipo pela URL do modulo.
    return nome in cpp


def main() -> int:
    fontes = {f: f.read_text(encoding="utf-8") for f in sorted(QML.rglob("*.qml"))}
    cpp = "\n".join(
        f.read_text(encoding="utf-8") for f in sorted(CPP.rglob("*.cpp"))
    ) + "\n".join(f.read_text(encoding="utf-8") for f in sorted(CPP.rglob("*.h")))

    orfaos = [
        (nome, arquivo)
        for nome, arquivo in sorted(entregues().items())
        if nome != RAIZ_DO_MODULO and not alcancado(nome, arquivo, fontes, cpp)
    ]

    if orfaos:
        print("alcance QML FALHOU:", file=sys.stderr)
        for nome, arquivo in orfaos:
            relativo = arquivo.relative_to(RAIZ)
            print(f"  ✗ {relativo} ({nome}) e' empacotado e ninguem o instancia", file=sys.stderr)
        print(
            "\nUm componente que o modulo ENTREGA e nenhuma tela abre e' uma de duas\n"
            "coisas, e as duas sao defeito: uma tela que o usuario nao alcanca, ou\n"
            "peso morto no binario. Foi assim que o `SimPlotSystem` ficou pronto,\n"
            "testado e invisivel enquanto o motor vetorial ja' respondia.\n"
            "Ligue o componente a uma tela, ou tire-o do QML_FILES.",
            file=sys.stderr,
        )
        return 1

    print(f"alcance QML: {len(entregues())} componentes entregues, todos alcancaveis.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
