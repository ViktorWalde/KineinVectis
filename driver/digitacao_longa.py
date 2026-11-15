"""INF-26: a queda da UI depois de uma digitação longa, sem pausa de 1,5 s.

Usa o recuperacao_base.py da cópia limpa do 6e828799 (a mesma base do 65
§20), com o número de teclas trocado. Cada caso: N teclas a 150 ms de quieto,
kill -9 na UI `atraso` ms depois da última; reabre e conta o recuperado.

Uso: python3 -I digitacao_longa.py <raiz da cópia> <saida.json> N:atraso ...
"""

import json
import os
import sys
import time

raiz = sys.argv[1]
sys.path.insert(0, os.path.join(raiz, "driver"))

import recuperacao_base as rb  # noqa: E402
from desempenho import fixture_nova  # noqa: E402

original = rb.ambiente


def com_teclas(n):
    def ambiente(casa, ws, caminho, extra):
        env = original(casa, ws, caminho, extra)
        if "KINEIN_PERF_TYPING_KEYS" not in extra:
            env["KINEIN_PERF_TYPING_KEYS"] = str(n)
            env["KINEIN_PERF_TYPING_TIMEOUT_MS"] = "300000"
        return env
    return ambiente


casos = []
for arg in sys.argv[3:]:
    n, atraso = (int(x) for x in arg.split(":"))
    rb.TECLAS = n
    rb.ambiente = com_teclas(n)
    caminho, _ = fixture_nova(60)
    inicio = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    t0 = time.monotonic()
    r = rb.medir(os.path.dirname(caminho), caminho, atraso)
    r["teclas_pedidas"] = n
    r["duracao_s"] = round(time.monotonic() - t0, 1)
    r["inicio"] = inicio
    print(r, flush=True)
    casos.append(r)

with open(sys.argv[2], "w", encoding="utf-8") as f:
    json.dump({"casos": casos}, f, ensure_ascii=False, indent=1)
