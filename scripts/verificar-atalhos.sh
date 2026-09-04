#!/usr/bin/env bash
# O atalho que a paleta ANUNCIA e o atalho que a IDE OBEDECE.
#
# Nasceu de relato de uso em 2026-09-04 ("nao consigo acessar as bibliotecas"):
# o core declarava Ctrl+Alt+L para `library.list`, a paleta mostrava, e a UI
# ligava esse atalho em FORMATAR ARQUIVO. O detalhe todo esta no cabecalho do
# `verificar_atalhos.py`.
#
# Uso: bash scripts/verificar-atalhos.sh
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

echo "== atalhos (a paleta anuncia o que a IDE obedece) =="
python3 scripts/verificar_atalhos.py
