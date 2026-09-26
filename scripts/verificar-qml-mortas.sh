#!/usr/bin/env bash
# Funcao QML declarada que NINGUEM menciona — o `dead_code` que o QML nao tem.
#
# O porque, a diferenca entre "morta" e "so' no harness", e por que a deteccao
# e' por MENCAO e nao por chamada, estao no cabecalho do
# `scripts/verificar_qml_mortas.py`.
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
exec python3 scripts/verificar_qml_mortas.py "$@"
