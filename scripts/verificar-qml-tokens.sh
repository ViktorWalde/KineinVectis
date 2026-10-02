#!/usr/bin/env bash
# Catraca de VALOR LITERAL no QML (raio, fonte, duracao, cor fora do Theme).
# O porque esta' no proprio scripts/verificar_qml_tokens.py.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== tokens QML (raio, fonte, duracao e cor vem do Theme) =="

python3 scripts/verificar_qml_tokens.py "$@"
