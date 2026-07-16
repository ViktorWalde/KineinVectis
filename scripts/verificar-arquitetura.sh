#!/usr/bin/env bash
# Catraca da regra de split (ARCHITECTURE.md §6 e §4 do core).
#
# POR QUE EXISTE. A regra de split ja existia e era boa; ela so nao era
# verificada por ninguem. Resultado medido em 2026-07-16: 20 arquivos acima do
# limite, e o `Main.qml` saiu de 336 linhas ("estado validado" escrito na
# ARCHITECTURE.md em 2026-07-06) para 700 em dez dias. Regra que mora so em .md
# nao segura arquitetura — ela apodrece em silencio enquanto o gate fica verde.
#
# POR QUE CATRACA E NAO LIMITE DURO. Falhar nos 20 hoje so ensinaria a
# desligar o script. A catraca e pragmatica: o debito existente fica congelado
# na baseline e so pode DIMINUIR. Arquivo novo acima do limite reprova; arquivo
# em debito que CRESCE reprova. Encolher e sempre aceito, e a baseline deve ser
# atualizada junto (o proprio script diz como).
#
# Uso: bash scripts/verificar-arquitetura.sh
#      bash scripts/verificar-arquitetura.sh --atualizar-baseline
set -uo pipefail
cd "$(dirname "$0")/.."

BASELINE="scripts/arquitetura-baseline.txt"

# Limites da ARCHITECTURE.md: QML visual 300, controller/store 400, C++ 500.
listar() {
    python3 - <<'PY'
import pathlib, subprocess

def limite(caminho: str) -> int:
    if caminho.startswith("ui/src/"):
        return 500
    # Controller/store/host e o composition root sao "logica": 400.
    nome = caminho.rsplit("/", 1)[-1]
    if ("Controller" in nome or "Host" in nome or nome == "Main.qml"):
        return 400
    return 300  # QML visual

alvos = subprocess.check_output(
    ["find", "ui/qml", "ui/src", "-name", "*.qml", "-o", "-name", "*.cpp",
     "-o", "-name", "*.h"], text=True).split()
for caminho in sorted(alvos):
    linhas = len(pathlib.Path(caminho).read_text().splitlines())
    lim = limite(caminho)
    if linhas > lim:
        print(f"{caminho} {linhas} {lim}")
PY
}

if [ "${1:-}" = "--atualizar-baseline" ]; then
    listar > "$BASELINE"
    echo "baseline atualizada: $(wc -l < "$BASELINE") arquivos em debito"
    echo "Commite junto com a mudanca que a alterou."
    exit 0
fi

if [ ! -f "$BASELINE" ]; then
    echo "erro: $BASELINE ausente. Gere com --atualizar-baseline." >&2
    exit 1
fi

atual="$(listar)"
falhou=0

# 1. Arquivo NOVO acima do limite: reprova. Nao se paga debito com debito.
while read -r caminho linhas lim; do
    [ -z "$caminho" ] && continue
    if ! grep -q "^$caminho " "$BASELINE"; then
        echo "✗ NOVO acima do limite: $caminho ($linhas linhas, limite $lim)" >&2
        echo "  Quebre antes de crescer em cima (ARCHITECTURE.md §6)." >&2
        falhou=1
    fi
done <<< "$atual"

# 2. Arquivo em debito que CRESCEU: reprova. A catraca so gira para um lado.
while read -r caminho linhas lim; do
    [ -z "$caminho" ] && continue
    antes="$(awk -v c="$caminho" '$1==c {print $2}' "$BASELINE")"
    if [ -n "$antes" ] && [ "$linhas" -gt "$antes" ]; then
        echo "✗ CRESCEU em debito: $caminho ($antes -> $linhas, limite $lim)" >&2
        echo "  Este arquivo ja passa do limite; nao pode engordar." >&2
        falhou=1
    fi
done <<< "$atual"

if [ "$falhou" -ne 0 ]; then
    echo >&2
    echo "✗ catraca de arquitetura FALHOU" >&2
    echo "  Encolher e sempre aceito; depois rode --atualizar-baseline." >&2
    exit 1
fi

restantes="$(printf '%s' "$atual" | grep -c . || true)"
echo "✓ arquitetura: catraca ok ($restantes arquivos em debito, nenhum novo/crescido)"
