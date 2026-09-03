#!/usr/bin/env bash
# Propriedades QML: pega o binding para propriedade/sinal QUE NAO EXISTE no alvo.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-03). A etapa 11.1 do roadmaps/34 moveu 85
# bindings do `ShellWorkspaceHost` para o `ShellEditorHost` novo. Ao terminar,
# a pergunta obvia foi: se eu tivesse digitado UM nome errado, quem reclamaria?
#
#   MEDIDO por mutacao, antes de escrever este script:
#     `findQuery:` -> `findQeury:` no ShellEditorHost.qml
#     cmake --build --preset dev-local        EXIT=0   (nem um aviso)
#     scripts/verificar-qml-fiacao.sh         EXIT=0
#     scripts/verificar-qml-logica.sh         EXIT=0
#     scripts/verificar-arquitetura.sh        EXIT=0
#
# Ou seja: o gate inteiro ficava verde com o Find quebrado. O QML so reclama de
# propriedade inexistente quando o componente e' INSTANCIADO, e nenhum teste do
# harness instancia os hosts do shell — eles so' nascem na IDE de verdade. Falha
# silenciosa, que e' a unica razao pela qual um gate nasce aqui (ARCHITECTURE.md
# §4 regra 11).
#
# A regra: dentro de um bloco `Tipo { ... }` de um componente DESTE repositorio,
# todo `nome:` no nivel do bloco tem que ser propriedade declarada do tipo, um
# `on<Sinal>`, um `on<Prop>Changed`, ou uma propriedade de base do QtQuick.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== propriedades QML (binding para propriedade inexistente) =="

python3 scripts/verificar_qml_propriedades.py
