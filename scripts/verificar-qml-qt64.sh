#!/usr/bin/env bash
# G0.2: compatibilidade do QML com o Qt 6.4 do AppImage (Debian 12) — o que o
# Qt mais novo do checkout aceita e o pacote NAO (roadmap 53 §5.2.1).
#
# A regra, o mecanismo no Qt 6.4.2 e os casos de mutacao que o gate roda a cada
# execucao estao em scripts/check_qml_qt64.py. Ate' 2026-10-01 era uma regex
# por linha dentro deste arquivo, e a sintaxe agrupada `section { delegate: }`
# passava por ela.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 scripts/check_qml_qt64.py
