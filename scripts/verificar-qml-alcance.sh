#!/usr/bin/env bash
# Componente QML EMPACOTADO que nenhuma tela instancia.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-07). O motor vetorial da simulacao foi
# entregue em 2026-09-06 e a tela dele nunca foi ligada. O `SimPlotSystem.qml`
# estava no `QML_FILES` do `CMakeLists`, tinha harness proprio que passava, e
# nao era instanciado em lugar nenhum do app; o `SimSystemController` nascia no
# `AppDomains` e ninguem o lia.
#
#   MEDIDO em 2026-09-07, com o buraco de pe:
#     cmake --build --preset dev-local        EXIT=0
#     scripts/verificar-qml.sh                EXIT=0   (qmllint estrito)
#     scripts/verificar-qml-propriedades.sh   EXIT=0
#     scripts/verificar-qml-fiacao.sh         EXIT=0
#     scripts/verificar-qml-logica.sh         EXIT=0   (o harness DELE passava)
#     scripts/verificar-arquitetura.sh        EXIT=0
#     cargo test --workspace                  649 verdes
#
# O gate inteiro verde, e o efeito para quem usa era PIOR que a ausencia: os
# tres conceitos de sistema apareciam na lista, a tela escalar os aceitava, e
# medido contra o binario real o `sim.checkFormula` aprovava `mu*2` na "Orbita
# de dois corpos" enquanto o `sim.evaluate` devolvia 2.
#
# Nenhum gate existente ve isso, e a razao e' estrutural: cada um confere o
# componente por SI. O qmllint le um arquivo, o de propriedades confere o
# binding onde ele esta escrito, o harness instancia o que o teste pediu. Falta
# alguem perguntando se ALGUMA tela chega ali — que e' a pergunta do usuario.
#
# A regra: todo `.qml` do `QML_FILES` tem de ser instanciado (`Tipo { }`) em
# outro arquivo do modulo, ou — se for `pragma Singleton` — usado pelo nome.
# `Main.qml` e a raiz e quem a instancia e o C++.
#
# Nao e' catraca: nao ha' baseline. Componente entregue e inalcancavel e' zero.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== alcance QML (componente entregue que nenhuma tela abre) =="

python3 scripts/verificar_qml_alcance.py "$@"
