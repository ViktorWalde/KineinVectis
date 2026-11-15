"""Utilidades dos testes: espera por condição, estatística, resultado."""

import json
import math
import os
import time
import traceback

RESULTADOS = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "resultados")


def esperar(condicao, timeout_s=10.0, intervalo_s=0.02, descricao="condição"):
    fim = time.monotonic() + timeout_s
    ultimo = None
    while time.monotonic() < fim:
        try:
            ultimo = condicao()
        except Exception as e:  # o canal pode estar trocando de nvim
            ultimo = e
        if ultimo and not isinstance(ultimo, Exception):
            return ultimo
        time.sleep(intervalo_s)
    raise TimeoutError(f"{descricao}: não aconteceu em {timeout_s} s (último: {ultimo!r})"[:2000])


def percentil(valores, p):
    if not valores:
        return None
    v = sorted(valores)
    # O mesmo do medir-performance.sh: posição ceil(p/100 * n), base 1.
    k = max(1, math.ceil(p / 100 * len(v)))
    return v[k - 1]


def resumo(valores):
    return {
        "n": len(valores),
        "mediana": percentil(valores, 50),
        "p95": percentil(valores, 95),
        "pior": percentil(valores, 100),
        "melhor": min(valores) if valores else None,
    }


def gravar(nome, dados):
    os.makedirs(RESULTADOS, exist_ok=True)
    caminho = os.path.join(RESULTADOS, f"{nome}.json")
    with open(caminho, "w") as f:
        json.dump(dados, f, ensure_ascii=False, indent=2)
    return caminho


def rodar(nome, funcao, *args, **kwargs):
    inicio = time.monotonic()
    try:
        dados = funcao(*args, **kwargs)
    except Exception as e:
        dados = {"veredito": "ERRO", "erro": f"{type(e).__name__}: {e}", "traceback": traceback.format_exc()}
    dados["duracao_s"] = round(time.monotonic() - inicio, 1)
    gravar(nome, dados)
    print(f"{nome}: {dados.get('veredito')} ({dados['duracao_s']} s)", flush=True)
    return dados
