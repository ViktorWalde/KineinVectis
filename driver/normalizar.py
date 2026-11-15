"""Tira dos resultados o caminho da máquina em que a prova rodou.

A raiz da prova vira $PROVA e a pasta pessoal vira ~; nada mais muda. Roda
depois das medidas e antes de os resultados entrarem no git.

Uso: python3 -I normalizar.py
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from pilha import PROVA  # noqa: E402
from util import RESULTADOS  # noqa: E402

CASA = os.path.expanduser("~")


def normalizar(texto):
    # A raiz da prova primeiro: ela costuma morar dentro da pasta pessoal.
    return texto.replace(PROVA, "$PROVA").replace(CASA, "~")


if __name__ == "__main__":
    for nome in sorted(os.listdir(RESULTADOS)):
        caminho = os.path.join(RESULTADOS, nome)
        with open(caminho, encoding="utf-8", errors="surrogateescape") as f:
            texto = f.read()
        novo = normalizar(texto)
        if novo != texto:
            with open(caminho, "w", encoding="utf-8", errors="surrogateescape") as f:
                f.write(novo)
            print(f"normalizado: {nome}")
