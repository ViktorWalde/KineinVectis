#!/usr/bin/env python3
"""Exercita o dominio `sim.*` contra o binario REAL, por stdio.

Existe por causa de um achado de 2026-09-06 (roadmaps/31 §19.0): a coluna
`exato` do `sim.run` vem do CONCEITO e nao da FORMULA que o usuario escreveu.
Quando as duas divergem — o que o `sim.checkFormula` permite, porque ele confere
ligacao e nao fisica — a IDE mostra um "erro absoluto" calculado contra a
solucao de OUTRA equacao.

Este script nao e' gate: ele imprime a medicao para quem for decidir o conserto.
Gate exige decisao de desenho que ainda nao foi tomada (o catalogo NAO guarda a
formula, por decisao registrada em arquitetura/34 §4.2).

Uso:
    cargo build --release -p kinein-core
    python3 scripts/exercitar-sim-oraculo.py
"""
import json
import pathlib
import subprocess
import sys

RAIZ = pathlib.Path(__file__).resolve().parent.parent
CORE = RAIZ / "target" / "release" / "kinein-core"


def abrir():
    if not CORE.exists():
        sys.exit(f"binario ausente: {CORE}\nrode: cargo build --release -p kinein-core")
    return subprocess.Popen(
        [str(CORE)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, text=True,
    )


def fazer_rpc(proc):
    estado = {"id": 0}

    def rpc(metodo, params):
        estado["id"] += 1
        quero = estado["id"]
        proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": quero, "method": metodo, "params": params}) + "\n")
        proc.stdin.flush()
        while True:
            linha = proc.stdout.readline()
            if not linha:
                sys.exit("o core fechou a saida antes de responder")
            msg = json.loads(linha)
            if msg.get("id") == quero:
                return msg

    return rpc


# O oscilador amortecido, com todas as variaveis LIGADAS pelo usuario.
LIGACOES = [
    {"variable": "x", "quantity": "y"},
    {"variable": "v", "quantity": "dy"},
    {"variable": "k", "quantity": "k"},
    {"variable": "m", "quantity": "m"},
    {"variable": "c", "quantity": "c"},
]
VALORES = [
    {"quantity": "k", "value": 2.0},
    {"quantity": "m", "value": 1.0},
    {"quantity": "c", "value": 0.5},
]
BASE = {
    "concept": "oscilador-amortecido",
    "bindings": LIGACOES,
    "values": VALORES,
    "initial": {"y": 1.0, "dy": 0.0},
    "duration": 10.0,
    "step": 0.1,
    "method": "rk4",
    "samples": 50,
}

# A quarta coluna e' a verdade do que foi DIGITADO, obtida com o `dsolve` do
# SymPy em 2026-09-06 e escrita aqui para o script nao exigir Python com SymPy
# instalado (ele NAO esta nesta maquina — roadmaps/31 §19.3.1).
FORMULAS = [
    ("canonica",              "-(k/m)*x - (c/m)*v",     0.0321283198320319),
    ("sinal trocado",         "(k/m)*x - (c/m)*v",      83178.3437510285),
    ("mola nao linear",       "-(k/m)*x*x*x - (c/m)*v", None),
    ("amortecimento em dobro", "-(k/m)*x - 2*(c/m)*v",  0.00687927723617439),
]


def main():
    proc = abrir()
    rpc = fazer_rpc(proc)

    print("== o catalogo, lido do binario ==")
    conceitos = rpc("sim.catalog", {})["result"]["concepts"]
    formas = {}
    for c in conceitos:
        formas[c["form"]] = formas.get(c["form"], 0) + 1
    print(f"  {len(conceitos)} conceitos: {formas}")
    print(f"  integraveis: {[c['id'] for c in conceitos if c['form'] in ('ode1', 'ode2')]}")

    print()
    print("== a coluna `exato` NAO olha a formula digitada ==")
    print(f"  {'formula':<24} {'check':>6} {'numerico':>16} {'exato da IDE':>14} "
          f"{'erro na tela':>13} {'erro REAL':>12}")
    for rotulo, formula, verdade in FORMULAS:
        checagem = rpc("sim.checkFormula", {
            "concept": BASE["concept"], "formula": formula, "bindings": LIGACOES})
        ok = checagem["result"]["ok"]
        corrida = rpc("sim.run", dict(BASE, formula=formula))
        resultado = corrida.get("result")
        if not resultado or not resultado.get("accuracy"):
            print(f"  {rotulo:<24} {str(ok):>6}   {str(corrida)[:60]}")
            continue
        exatidao = resultado["accuracy"]
        real = "n/d" if verdade is None else f"{abs(exatidao['numeric'] - verdade):.3e}"
        print(f"  {rotulo:<24} {str(ok):>6} {exatidao['numeric']:>16.6f} "
              f"{exatidao['exact']:>14.6f} {exatidao['absoluteError']:>13.3e} {real:>12}")

    print()
    print("  A coluna `exato` e' a mesma nas quatro linhas: ela vem de")
    print("  catalogo::exata(conceito.id) e ignora a formula (sim/corrida.rs).")
    print("  `erro REAL` veio do dsolve resolvendo o que foi DIGITADO (2026-09-06);")
    print("  `nao linear` nao tem: o dsolve responde NotImplementedError, que e' a")
    print("  resposta certa e ainda assim melhor que o numero que a IDE mostra.")

    proc.stdin.close()
    proc.wait(timeout=5)


if __name__ == "__main__":
    main()
