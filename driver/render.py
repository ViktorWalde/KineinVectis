"""O custo de desenhar um quadro, nos dois lados, durante a digitação: o que
a janela mínima da prova não paga e a IDE inteira paga. Rodada separada (o
log do scenegraph pesa), fora das medidas do desempenho.py.

Uso: python3 -I render.py [rodadas]
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from desempenho import fixture_nova, rodada_base, rodada_prova  # noqa: E402
from util import gravar, resumo  # noqa: E402

LOG = {"QT_LOGGING_RULES": "qt.scenegraph.time.renderloop.debug=true"}
QUADRO = re.compile(r"Frame rendered with '(\w+)' renderloop in ([0-9.]+)\s*ms(.*)")
PARTE = re.compile(r"(\w+)=([0-9.]+)")


# Onde a digitação começa e acaba no log de cada lado.
JANELA = {"base": ("KINEIN_PERF typing_line=", "KINEIN_PERF typing_hold="),
          "prova": ("KINEIN_PROVA_MARCA digitacao_inicio", "KINEIN_PROVA_MARCA digitacao_fim")}


def quadros(log, inicio, fim):
    """Os quadros da rodada inteira e os da janela da digitação."""
    todos, digitando = [], []
    dentro = False
    with open(log, errors="replace") as f:
        for linha in f:
            if inicio in linha:
                dentro = True
            elif fim in linha:
                dentro = False
            m = QUADRO.search(linha)
            if m:
                q = {"loop": m.group(1), "total_ms": float(m.group(2))}
                q.update({k: float(v) for k, v in PARTE.findall(m.group(3))})
                todos.append(q)
                if dentro:
                    digitando.append(q)
    return todos, digitando


def resumir(qs):
    chaves = sorted({k for q in qs for k in q if k != "loop"})
    return {"quadros": len(qs), "loops": sorted({q["loop"] for q in qs}),
            **{k: resumo([q[k] for q in qs if k in q]) for k in chaves}}


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 3
    r = {}
    for nome, f in (("base", rodada_base), ("prova", rodada_prova)):
        todos, digitando = [], []
        for _ in range(n):
            rod = f(*fixture_nova(60), extra_env=LOG)
            t, d = quadros(rod["log"], *JANELA[nome])
            todos += t
            digitando += d
        r[nome] = {"rodada_inteira": resumir(todos), "digitando": resumir(digitando)}
        print(nome, len(todos), len(digitando), r[nome]["digitando"].get("total_ms"), flush=True)
    gravar("render", r)


if __name__ == "__main__":
    main()
