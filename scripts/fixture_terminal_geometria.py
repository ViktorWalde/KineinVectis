#!/usr/bin/env python3
"""Fixture de geometria do terminal (R0, docs/roadmaps/26).

Emite um padrao DETERMINISTICO no stdout para comparar a grade da Kinein com a
de um terminal externo com a mesma fonte/tamanho. Nao le arquivo, nao acessa
rede, nao toca no workspace e nao imprime nada do usuario — o conteudo e fixo e
versionado, entao dois runs sao identicos byte a byte.

Uso:
    python3 scripts/fixture_terminal_geometria.py            # padrao completo
    python3 scripts/fixture_terminal_geometria.py --regua    # so a regua
    python3 scripts/fixture_terminal_geometria.py --cursor   # so DECSCUSR
    python3 scripts/fixture_terminal_geometria.py --alt      # tela alternada

O ponto da regua: cada celula da linha "123456789" marca uma coluna. Se o
cursor da IDE nao cair exatamente sobre o digito da coluna onde o VT diz que ele
esta, a grade do renderer divergiu da grade do emulador — que e o defeito que o
R0 investiga. A barra de box drawing imita a moldura de uma TUI (Claude/Codex)
sem precisar abrir o agente.
"""

import argparse
import sys

ESC = "\x1b"
CSI = ESC + "["


def regua(cols=80):
    """Regua de colunas: dezenas em cima, unidades embaixo."""
    dezenas = "".join(str((c // 10) % 10) for c in range(cols))
    unidades = "".join(str(c % 10) for c in range(cols))
    return f"{dezenas}\r\n{unidades}\r\n"


def barra_tui(cols=60):
    """Moldura de box drawing, como a borda de uma TUI."""
    topo = "╭" + "─" * (cols - 2) + "╮"
    meio = "│" + " " * (cols - 2) + "│"
    base = "╰" + "─" * (cols - 2) + "╯"
    return f"{topo}\r\n{meio}\r\n{base}\r\n"


def glifos():
    """Classes de glifo que mudam metrica efetiva (fallback de fonte)."""
    linhas = [
        "ASCII      |MMMMMMMMMM|  avanco base",
        "largo/CJK  |界界界界界|  cada um ocupa 2 celulas",
        "combinante |áéíóú|  1 celula cada",
        "emoji      |\U0001f600\U0001f600\U0001f600|  fallback de familia",
        "box        |─│┌┐└┘┼|  moldura de TUI",
    ]
    return "\r\n".join(linhas) + "\r\n"


def cursores():
    """DECSCUSR: cada forma, steady e blinking, com o cursor parado numa
    coluna conhecida para o olho comparar contra a regua."""
    formas = [
        ("0 reset  ", 0),
        ("1 bloco pisc", 1),
        ("2 bloco     ", 2),
        ("3 underl pisc", 3),
        ("4 underl    ", 4),
        ("5 barra pisc", 5),
        ("6 barra     ", 6),
    ]
    saida = []
    for rotulo, codigo in formas:
        # O cursor para logo apos o rotulo; a regua acima diz em que coluna.
        saida.append(f"{rotulo}{ESC}[{codigo} q")
    return "\r\n".join(saida) + "\r\n"


def extremos(cols=80):
    """Cursor na primeira e na ultima coluna — onde o erro de arredondamento
    e maior (o desvio cresce com a coluna)."""
    linhas = [
        f"{CSI}1Gprimeira coluna: o cursor deve cobrir o 'p'",
        f"{CSI}{cols}Gx  <- ultima coluna ({cols})",
    ]
    return "\r\n".join(linhas) + "\r\n"


def padrao_completo(cols=80):
    partes = [
        f"{CSI}2J{CSI}H",
        "== fixture de geometria (R0) — compare com um terminal externo ==\r\n\r\n",
        regua(cols),
        "\r\n",
        barra_tui(min(60, cols)),
        "\r\n",
        glifos(),
        "\r\n",
        cursores(),
        "\r\n",
        extremos(cols),
        f"\r\n{ESC}[6 q",
    ]
    return "".join(partes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cols", type=int, default=80, help="largura da regua")
    parser.add_argument("--regua", action="store_true", help="so a regua")
    parser.add_argument("--cursor", action="store_true", help="so DECSCUSR")
    parser.add_argument("--alt", action="store_true", help="em tela alternada")
    args = parser.parse_args()

    if args.regua:
        saida = regua(args.cols)
    elif args.cursor:
        saida = regua(args.cols) + cursores()
    else:
        saida = padrao_completo(args.cols)

    if args.alt:
        # 1049: tela alternada, como Claude/vim/htop usam.
        saida = f"{CSI}?1049h" + saida

    sys.stdout.write(saida)
    sys.stdout.flush()

    if args.alt:
        # Segura a tela alternada ate o Enter, senao ela some na hora.
        try:
            input()
        except EOFError:
            pass
        sys.stdout.write(f"{CSI}?1049l")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
