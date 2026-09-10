#!/usr/bin/env python3
"""Um `python3` FALSO para os testes do oraculo de simulacao.

POR QUE ELE EXISTE. O oraculo (`crates/kinein-core/src/sim/oraculo.rs`) fala com
um processo externo, e a pergunta que os testes precisam responder nao e' "a
resposta chegou?" — e' **o que exatamente foi perguntado**. Foi essa distincao
que a etapa 1 ensinou em 2026-08-30, quando uma sonda ficou verde com o produto
quebrado por olhar a resposta em vez do efeito; o `scripts/fake_lsp_server.py`
nasceu do mesmo aprendizado, e este segue o mesmo molde.

Ele e' invocado como o `python3` de verdade seria — `... -c <programa>` com o
pedido em JSON no stdin — e IGNORA o programa. Isso e' de proposito: o que se
testa aqui e' o contrato entre o core e o processo, nao o `SymPy`.

Controle por ARGUMENTO, nao por ambiente:

    --modo=ok|nosympy|cannotsolve|hang|travaedo|livre
                                                 o que responder
    --log=<arquivo>                              onde gravar o PEDIDO recebido
    --expr=<expressao>                           a resposta, no modo `livre`
    --dim=<veredito>                             o veredito de unidade que ele
                                                 devolve para CADA equacao
                                                 pedida (padrao: coherent)

**Por argumento de proposito.** Este repositorio proibe `unsafe`
(`unsafe_code = "forbid"`), e escrever variavel de ambiente virou `unsafe` na
edicao 2024 do Rust — entao o teste nao pode configurar o falso por ambiente. A
trava esta certa por outro motivo tambem: ambiente e' estado global de processo,
e teste que o escreve contamina o vizinho que roda em paralelo.

Quem chama monta um pequeno invocador por cenario e o aponta como
"interpretador"; e' assim que o oraculo recebe um `Python` diferente sem que
ninguem mexa em estado global.
"""
import json
import sys
import time

# A solucao real do oscilador amortecido com k=2, m=1, c=0,5 e y(0)=1, y'(0)=0,
# copiada da medicao de 2026-09-06 (`roadmaps/31` §19.3.4). Em t=10 ela vale
# 0,0321283198320319.
OSCILADOR = "(sqrt(31)*sin(sqrt(31)*t/4)/31 + cos(sqrt(31)*t/4))*exp(-t/4)"


def opcao(nome: str, padrao: str) -> str:
    prefixo = f"--{nome}="
    for argumento in sys.argv[1:]:
        if argumento.startswith(prefixo):
            return argumento[len(prefixo):]
    return padrao


def main() -> int:
    pedido_bruto = sys.stdin.read()
    destino = opcao("log", "")
    if destino:
        with open(destino, "w", encoding="utf-8") as arquivo:
            arquivo.write(pedido_bruto)

    modo = opcao("modo", "ok")

    if modo == "hang":
        # Nao responde nunca: e' o pendulo nao linearizado, que nao volta em 20 s.
        while True:
            time.sleep(3600)

    def linha(objeto):
        sys.stdout.write(json.dumps(objeto) + "\n")
        sys.stdout.flush()

    # A falta do SymPy e' de TRANSPORTE: ela atinge as duas perguntas.
    if modo == "nosympy":
        linha({"kind": "fatal", "reason": "noSympy"})
        return 0

    try:
        pedido = json.loads(pedido_bruto) if pedido_bruto.strip() else {}
    except Exception:
        pedido = {}

    # A BARATA primeiro, como o programa de verdade: o que ja' foi descarregado
    # sobrevive ao teto que mata o `dsolve`.
    veredito = opcao("dim", "coherent")
    linha({"kind": "dimensoes", "sympy": "1.14.0",
           "itens": [{"label": item.get("label", ""), "verdict": veredito, "detail": ""}
                     for item in pedido.get("dimensoes", [])]})

    if modo == "travaedo":
        # As unidades ja' foram; a EDO nunca vem. E' o pendulo nao linearizado.
        while True:
            time.sleep(3600)

    if pedido.get("edo"):
        if modo == "cannotsolve":
            linha({"kind": "edo", "sympy": "1.14.0", "ok": False, "reason": "cannotSolve"})
        else:
            expressao = opcao("expr", OSCILADOR) if modo == "livre" else OSCILADOR
            linha({"kind": "edo", "sympy": "1.14.0", "ok": True, "expression": expressao})
    return 0


if __name__ == "__main__":
    sys.exit(main())
