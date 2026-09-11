#!/usr/bin/env bash
# O binario que sai do `cmake --build` ABRE.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-11). Em 2026-09-10 os dois builds do
# checkout abortavam ao abrir — SIGABRT com display real e offscreen — e o gate
# de dezenove verificacoes estava VERDE. O AppImage em dist/ abria, porque o
# `verificar-appimage.sh` executa o ARTEFATO; nada executava o binario que o
# proprio gate acabava de compilar. "Compila" e "abre" sao afirmacoes
# diferentes, e o repositorio so' tinha rede para a primeira.
#
#   MEDIDO em 2026-09-11, com o defeito de pe':
#     cmake --build --preset dev-local                    EXIT=0
#     scripts/verificar.sh (19 verificacoes)              VERDE
#     build/dev-local/ui/kinein-vectis                    SIGABRT, stderr VAZIO
#     journalctl:  ASSERT: "!this->isShared() || b == e"
#                  in qarraydataops.h, line 286
#
# A CAUSA nao era o Qt 6.11.2 nem o QML compilado em AOT, como a hipotese do
# dia anterior dizia: era o rpm. Ele instala header com o mtime de quando o
# pacote foi CONSTRUIDO (maio), e o ninja compara mtime — entao os 11 objetos
# compilados na tarde de 2026-09-04, antes de o Qt subir de 6.11.1 para 6.11.2
# as 22:21, ficaram "VALID" para sempre. O binario era uma MISTURA: o
# `editor_highlighter_folding.cpp.o` (6.11.1) e o `GlobalShortcuts` compilado
# em AOT (6.11.2) definiam o mesmo `QGenericArrayOps<QVariant>::copyAppend`
# inline, o linker dobrou o COMDAT na versao velha, e o chamador novo passou
# `this` num layout que a velha nao entende. Violacao de ODR: aborta na
# primeira `QVariantList{...}` que o QML compilado monta. `cmake --preset X`
# (o remedio registrado para a troca de Qt) reconfigura e NAO invalida objeto
# nenhum; so' `--clean-first` ou remover os objetos resolve.
#
# E o stderr estava vazio porque o Qt do Fedora e' compilado com journald: sem
# tty, o assert vai para o journal. Por isso o smoke forca
# QT_FORCE_STDERR_LOGGING=1 — um SIGABRT mudo custou uma sessao inteira.
#
# O que este gate confere, nesta ordem, porque a primeira EXPLICA a segunda:
#   1. nenhum objeto da arvore e' mais velho (mtime) que a chegada ao disco
#      (ctime) de uma dependencia que o ninja registrou para ele — e, se for,
#      diz qual e imprime o comando que o remove;
#   2. o binario chega ao primeiro frame offscreen (KINEIN_PERF_MARKER, o
#      mesmo mecanismo do smoke do AppImage) e sai com 0.
#
# Provado por mutacao nas duas metades: `touch -d` num objeto para antes da
# instalacao do Qt reprova em (1) e nomeia o header; o binario de 2026-09-10 no
# lugar do relinkado reprova em (2) e mostra o assert.
#
# Sem catraca: binario que nao abre e' zero.
#
# Uso:
#   scripts/verificar-binario-abre.sh --preset dev-local
#   scripts/verificar-binario-abre.sh --build-dir build/dev-local
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== o binario que saiu do build ABRE (${*:-sem alvo}) =="

python3 scripts/verificar_binario_abre.py "$@"
