"""R4 na linha de base: o mesmo teste do t_recuperacao, no editor atual.

40 teclas, uma por vez, com 150 ms de quieto entre elas, no meio da fixture
de 60 funções; kill -9 na UI (onde o motor de edição do produto mora) 0 ou
200 ms depois do quadro da última. Depois, o produto reabre com a mesma casa
e o mesmo workspace, e a linha que o editor mostra é comparada com o
digitado; o disco também.

A queda só do core não entra: no produto, o texto vive na UI, e a UI
sobrevive à queda do core. Fica NÃO MEDIDO.

Uso: python3 -I recuperacao_base.py [atraso_ms ...]
"""

import os
import signal
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from desempenho import BASE_CORE, BASE_UI, casa_limpa, fixture_nova  # noqa: E402
from pilha import descendentes, vivos  # noqa: E402
from provas import recuperados  # noqa: E402
from util import gravar  # noqa: E402

ALFABETO = "abcdefghijklmnopqrstuvwxyz"
TECLAS = 40


def ambiente(casa, ws, caminho, extra):
    env = os.environ.copy()
    env.update(casa)
    env.update({"QT_QPA_PLATFORM": "offscreen", "QT_FORCE_STDERR_LOGGING": "1", "KINEIN_CORE_BIN": BASE_CORE,
                "KINEIN_PERF_TYPING": "1", "KINEIN_PERF_TYPING_WORKSPACE": ws,
                "KINEIN_PERF_TYPING_FILE": caminho})
    env.update(extra)
    return env


def rodar(env, ws, log_nome, timeout_s=120):
    """Roda o produto até sair; devolve as linhas KINEIN_PERF, o código de
    saída e os descendentes que sobraram (e os mata)."""
    proc = subprocess.Popen([BASE_UI], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=ws)
    perf = {}
    filhos = set()
    with open(os.path.join(ws, log_nome), "w") as log:
        fim = time.monotonic() + timeout_s
        for linha in proc.stdout:
            log.write(linha)
            filhos |= set(descendentes(proc.pid))
            if "KINEIN_PERF " in linha:
                chave, _, valor = linha.split("KINEIN_PERF ", 1)[1].rstrip("\n").partition("=")
                perf[chave] = valor
            if time.monotonic() > fim:
                proc.kill()
                break
    codigo = proc.wait(10)
    # O core sai sozinho quando a UI morre? Espera um pouco e conta.
    t0 = time.monotonic()
    while vivos(filhos) and time.monotonic() - t0 < 5:
        time.sleep(0.05)
    sobrando = vivos(filhos)
    for pid in sobrando:
        os.kill(pid, signal.SIGKILL)
    return perf, codigo, sobrando


def medir(ws, caminho, atraso_ms):
    casa = casa_limpa()
    digitado = "".join(ALFABETO[i % len(ALFABETO)] for i in range(TECLAS))
    r = {"queda": "ui", "atraso_ms": atraso_ms}
    perf, codigo, sobrando = rodar(ambiente(casa, ws, caminho, {"KINEIN_PERF_CRASH_AFTER_MS": str(atraso_ms)}),
                                   ws, "base-queda.log")
    r["digitados"] = int(perf.get("typing_chars_inserted", -1))
    r["linha_alvo"] = int(perf.get("typing_line", -1))
    r["codigo_de_saida"] = codigo
    r["core_sobrou_depois_da_queda"] = len(sobrando)
    if perf.get("typing_error"):
        r["erro"] = perf["typing_error"]
        return r
    linha = r["linha_alvo"]
    with open(caminho, encoding="utf-8") as f:
        disco = f.read().split("\n")
    r["recuperados_no_disco"] = recuperados(disco, linha, digitado)
    # Reabre com a mesma casa: o que o editor mostra na linha.
    perf2, _, _ = rodar(ambiente(casa, ws, caminho, {"KINEIN_PERF_TYPING_KEYS": "0", "KINEIN_PERF_DUMP_LINE": "1"}),
                        ws, "base-reaberta.log")
    mostrada = perf2.get("typing_line_text")
    r["linha_mostrada_ao_reabrir"] = (mostrada or "")[:80]
    r["recuperados_ao_reabrir"] = recuperados([mostrada or ""], 1, digitado)
    r["recuperados"] = max(r["recuperados_no_disco"], r["recuperados_ao_reabrir"])
    r["perdidos"] = r["digitados"] - r["recuperados"]
    rascunhos = []
    for raiz in (ws, casa["HOME"]):
        for d, _, fs in os.walk(raiz):
            rascunhos += [os.path.relpath(os.path.join(d, f), raiz) for f in fs if "draft" in f or "rascunho" in d]
    r["arquivos_de_rascunho"] = rascunhos[:10]
    return r


if __name__ == "__main__":
    # Os atrasos da medida são 0 e 200 ms; o controle positivo (o teste vê a
    # recuperação quando o rascunho do produto já gravou) é `2500`.
    atrasos = [int(x) for x in sys.argv[1:]] or [0, 200]
    nome = "recuperacao_base" if atrasos == [0, 200] else "recuperacao_base_" + "_".join(f"{a}ms" for a in atrasos)
    casos = []
    for atraso in atrasos:
        caminho, _ = fixture_nova(60)
        r = medir(os.path.dirname(caminho), caminho, atraso)
        print("  ", r, flush=True)
        casos.append(r)
    gravar(nome, {"casos": casos,
                  "nao_medido": "queda só do core: no produto o texto vive na UI, que sobrevive a ela"})
