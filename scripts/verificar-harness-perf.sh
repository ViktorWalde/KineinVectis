#!/usr/bin/env bash
# O harness de digitacao AINDA CONSEGUE SE INSTALAR?
#
# POR QUE ESTE ARQUIVO EXISTE (2026-08-21). Em 2026-07-16 o `editorController`
# mudou do Main.qml para o AppDomains.qml. Id de QML e escopado por componente,
# entao o `objectForName` do contexto do Main.qml deixou de encontra-lo — e o
# harness parou de rodar NAQUELE DIA, em silencio.
#
# A consequencia so apareceu um mes depois: a mediana de 7,4 ms que o L0 dava
# como fechada foi a ULTIMA que rodou. Um mes de fatias de editor, layout e
# shell passou sem que ninguem pudesse medir digitacao. O numero nao estava
# errado — estava CONGELADO, que e pior, porque parece vivo.
#
# A licao e a mesma da §1.1 da ARCHITECTURE em outra roupa: **ferramenta de
# medicao fora do gate apodrece calada.** Este script e a rede que faltava.
#
# O QUE ELE NAO FAZ, de proposito: nao mede latencia e nao compara com
# orcamento. Numero de performance varia com maquina, carga e compositor;
# transformar isso em gate produziria reprovacao aleatoria, e gate que reprova
# sem motivo e desligado pela equipe em uma semana. O que se verifica aqui e a
# unica coisa deterministica e a unica que quebrou: o harness ACHA os objetos
# de que precisa. A medicao continua sendo gesto do dev, pelo
# scripts/medir-performance.sh.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

UI_BIN="${KINEIN_UI_BIN:-}"
if [[ -z "$UI_BIN" ]]; then
    for candidato in \
        "$REPO_ROOT/build/linux-clang-debug-strict/ui/kinein-vectis" \
        "$REPO_ROOT/build/dev-local/ui/kinein-vectis"; do
        if [[ -x "$candidato" ]]; then
            UI_BIN="$candidato"
            break
        fi
    done
fi
CORE_BIN="${KINEIN_CORE_BIN:-$REPO_ROOT/target/debug/kinein-core}"

if [[ ! -x "$UI_BIN" ]]; then
    echo "erro: binario da UI nao encontrado; compile o preset debug antes." >&2
    exit 1
fi
if [[ ! -x "$CORE_BIN" ]]; then
    echo "erro: binario do core nao encontrado ($CORE_BIN)." >&2
    exit 1
fi

echo "== harness de digitacao: consegue se instalar? =="

WORKSPACE="$(mktemp -d -t kinein-harness-XXXXXX)"
trap 'rm -rf -- "$WORKSPACE"' EXIT
python3 "$REPO_ROOT/scripts/medir-core.py" --emit-fixture "$WORKSPACE" >/dev/null

# UMA tecla e timeout curto: o que se prova e o ATTACH, nao a medicao. Com as
# 40 teclas do fluxo normal isto levaria dezenas de segundos dentro de um gate
# que ja e o gargalo do loop de quem desenvolve.
SAIDA="$(
    QT_QPA_PLATFORM=offscreen \
    QT_FORCE_STDERR_LOGGING=1 \
    KINEIN_CORE_BIN="$CORE_BIN" \
    KINEIN_PERF_TYPING=1 \
    KINEIN_PERF_TYPING_WORKSPACE="$WORKSPACE" \
    KINEIN_PERF_TYPING_FILE="$WORKSPACE/fixture.rs" \
    KINEIN_PERF_TYPING_KEYS=1 \
    KINEIN_PERF_TYPING_TIMEOUT_MS=45000 \
    timeout 90 "$UI_BIN" 2>&1 || true
)"

ERRO="$(printf '%s\n' "$SAIDA" | sed -n 's/.*typing_error=\(.*\)/\1/p' | head -n1)"
if [[ -n "$ERRO" ]]; then
    echo "✗ o harness de digitacao NAO conseguiu rodar:" >&2
    echo "    $ERRO" >&2
    echo >&2
    echo "  Isto ja aconteceu em silencio por um mes (2026-07-16 a 2026-08-21):" >&2
    echo "  mover um controller de componente QML quebra o objectForName, porque" >&2
    echo "  id de QML e escopado por componente. Se voce moveu algo do Main.qml" >&2
    echo "  ou do AppDomains.qml, o harness precisa alcancar pelo caminho novo" >&2
    echo "  (ui/src/typing_perf_harness.cpp)." >&2
    echo >&2
    echo "  NAO desligue este gate para seguir: sem ele, o L0 volta a ter um" >&2
    echo "  numero congelado que parece vivo." >&2
    exit 1
fi

# Amostra e o sinal POSITIVO: o harness nao so achou os objetos, ele fechou o
# ciclo tecla -> frame. Sem isto, um attach que trava passaria por sucesso.
if ! printf '%s\n' "$SAIDA" | grep -q "typing_sample_ms="; then
    echo "✗ o harness instalou mas nao produziu nenhuma amostra." >&2
    echo "  O ciclo tecla -> frame nao fechou; ver a saida completa com" >&2
    echo "  KINEIN_PERF_TYPING=1 no binario da UI." >&2
    exit 1
fi

echo "harness de digitacao: instala e mede (a latencia em si e' gesto do dev,"
echo "                      por scripts/medir-performance.sh)."
