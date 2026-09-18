#!/usr/bin/env bash
# Fiacao IPC de ponta a ponta — o "andar de cima" que a varredura de
# 2026-09-10 (DocsPublic/roadmaps/40 §8.4) disse que faltava: metodo que
# ninguem pede, evento que o core emite e o C++ descarta, sinal do CoreClient
# sem ouvinte, sinal QML sem tratador. Nasceu no pente-fino de 2026-09-18.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
python3 scripts/verificar_fiacao_ipc.py
