#!/usr/bin/env bash
# Catraca de VALOR LITERAL no QML (raio, fonte, duracao, cor fora do Theme).
# O porque esta' no proprio scripts/verificar_qml_tokens.py.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== tokens QML (raio, fonte, duracao e cor vem do Theme) =="

python3 scripts/verificar_qml_tokens.py "$@"

# O degrade da tela de boas-vindas e' DERIVADO de tokens (brandInk, brandEmber),
# misturado em OKLab por um gerador; o bloco gravado no Theme tem que bater com
# as pontas de agora (2026-10-03).
python3 scripts/gerar_degrade_boas_vindas.py --check
