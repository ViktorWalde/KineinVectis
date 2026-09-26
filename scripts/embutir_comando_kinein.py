#!/usr/bin/env python3
"""Copia o modelo do comando `kinein` para dentro do instalador entregue.

QUEM BAIXA RECEBE DOIS ARQUIVOS — o AppImage e o instalador —, e o instalador
precisa bastar sozinho. O modelo continua tendo um dono so' (`scripts/kinein.in`);
aqui ele e' COPIADO para o lugar do marcador, sem ninguem reescrever o corpo.
Escrever o comando duas vezes seria a mesma derivacao em dois arquivos, e a
primeira correcao que so' um recebesse os faria divergir em silencio.
"""

import sys
from pathlib import Path

MARCADOR = "@MODELO_EMBUTIDO@"
DELIMITADOR = "FIM_DO_MODELO"


def main() -> int:
    if len(sys.argv) != 3:
        print(f"uso: {sys.argv[0]} <kinein.in> <instalador>", file=sys.stderr)
        return 2

    modelo = Path(sys.argv[1]).read_text(encoding="utf-8").rstrip("\n")
    instalador = Path(sys.argv[2])
    texto = instalador.read_text(encoding="utf-8")

    if texto.count(MARCADOR) != 1:
        print(
            f"esperava um {MARCADOR} no instalador, achei {texto.count(MARCADOR)}",
            file=sys.stderr,
        )
        return 1

    # O heredoc do instalador e' aspado, entao nada expande la' dentro; o unico
    # jeito de estragar o arquivo e' o modelo conter uma linha igual ao
    # delimitador.
    if any(linha.strip() == DELIMITADOR for linha in modelo.splitlines()):
        print(f"o modelo tem uma linha {DELIMITADOR} e fecharia o heredoc", file=sys.stderr)
        return 1

    instalador.write_text(texto.replace(MARCADOR, modelo), encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
