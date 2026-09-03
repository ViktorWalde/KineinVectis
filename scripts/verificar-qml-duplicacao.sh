#!/usr/bin/env bash
# Catraca de DERIVACAO DUPLICADA na UI QML.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-03). A mesma derivacao escrita em dois
# arquivos diverge, e diverge em SILENCIO. Medido, nao suposto:
#
#   ProblemsPanel.severityColor   severidade desconhecida -> infoSoft  (azul)
#   EditorGutter.diagnosticColor  severidade desconhecida -> errorSoft (vermelho)
#
# O MESMO diagnostico aparecia azul no painel e vermelho na sarjeta. Os dois
# arquivos compilavam, o qmllint passava, os 449 testes passavam e o gate
# inteiro ficava verde. Nenhuma rede existente pega isso, porque cada copia,
# sozinha, esta certa.
#
# E' CATRACA, nao limite: a duplicacao existente fica congelada em
# `qml-duplicacao-baseline.txt` e so' pode DIMINUIR. Derivacao nova em dois
# arquivos reprova; derivacao que se espalha para mais um arquivo reprova.
#
# Falso positivo tem saida honesta e explicita: `status === "detected"` e' o
# estado de uma FERRAMENTA em tres arquivos e o de uma BIBLIOTECA num quarto —
# mesma string, fatos diferentes. Nesse caso a baseline se atualiza, e a
# atualizacao fica no commit para alguem poder discordar.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== duplicacao QML (mesma derivacao em mais de um arquivo) =="

python3 scripts/verificar_qml_duplicacao.py "$@"
