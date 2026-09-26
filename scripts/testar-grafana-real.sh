#!/usr/bin/env bash
# Fluxo REAL do dominio `grafana.*` contra um Grafana de verdade, em container.
#
# Por que NAO esta' no `verificar.sh`: ele exige podman e REDE para baixar as
# imagens. O gate e' offline por principio. Mesma separacao que o AppImage e o
# Remote SSH ja' usam: `verificar-*` e' a catraca barata e hermetica;
# `testar-*` e' a prova pesada que se roda de proposito.
#
# O que ele prova esta' em scripts/testar_grafana_real.py. Em uma frase: que os
# quatro casos da §10 da especificacao — URL invalida, sem token, token
# invalido e token valido — se comportam contra a HTTP API real, e que o
# CRUZAMENTO entre o banco deste projeto e a fonte de dados do Grafana
# acontece de verdade.
#
# ISOLAMENTO: uma HOME temporaria e um projeto temporario. Nada e' escrito no
# workspace do autor. Os containers sao EFEMEROS (`--rm`) e removidos no fim,
# inclusive se algo falhar.
#
# REDE DO HOST de proposito: o cruzamento compara o host que o Grafana declara
# na fonte com o host do perfil do projeto. Numa rede propria do podman, o
# Grafana chamaria o Postgres de `kinein-pg-prova` e o projeto o chamaria de
# `localhost` — nao casariam, e o teste estaria medindo a rede em vez da regra.
#
# Uso: bash scripts/testar-grafana-real.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

IMAGEM_GRAFANA="${KINEIN_GRAFANA_IMAGEM:-docker.io/grafana/grafana:11.2.0}"
IMAGEM_PG="${KINEIN_PG_IMAGEM:-docker.io/library/postgres:16-alpine}"
CONTAINER_GRAFANA="kinein-grafana-prova-$$"
CONTAINER_PG="kinein-pg-prova-$$"
PORTA_GRAFANA="${KINEIN_GRAFANA_PORTA:-3000}"
PORTA_PG="${KINEIN_PG_PORTA:-5432}"
BANCO="kinein_prova"

if ! command -v podman >/dev/null 2>&1; then
    echo "erro: este teste precisa do podman (o repo ja' o usa no AppImage)." >&2
    exit 1
fi
if [ ! -x target/release/kinein-core ]; then
    echo "erro: falta target/release/kinein-core. Rode:" >&2
    echo "      cargo build --release -p kinein-core" >&2
    exit 1
fi

# PORTA OCUPADA E' RECUSA, E NAO SURPRESA. Subir por cima de um servico do
# autor — ou falhar la' na frente por "conexao recusada" — seria pior que
# parar aqui dizendo o que esta' no caminho.
porta_ocupada() {
    ss -ltn 2>/dev/null | awk '{print $4}' | grep -qE "[:.]$1\$"
}
for porta in "$PORTA_GRAFANA" "$PORTA_PG"; do
    if porta_ocupada "$porta"; then
        echo "erro: a porta $porta ja' esta' em uso." >&2
        echo "      Pare quem a ocupa, ou rode com:" >&2
        echo "      KINEIN_GRAFANA_PORTA=3001 KINEIN_PG_PORTA=5433 $0" >&2
        exit 1
    fi
done

HOME_TESTE="$(mktemp -d)"
PROJETO="$(mktemp -d)"
limpar() {
    podman rm -f "$CONTAINER_GRAFANA" "$CONTAINER_PG" >/dev/null 2>&1 || true
    rm -rf "$HOME_TESTE" "$PROJETO"
}
trap limpar EXIT

echo "== Postgres de verdade (podman) =="
podman run -d --rm --name "$CONTAINER_PG" --network host \
    -e POSTGRES_USER=kinein \
    -e POSTGRES_PASSWORD=kinein \
    -e POSTGRES_DB="$BANCO" \
    -e PGPORT="$PORTA_PG" \
    "$IMAGEM_PG" >/dev/null
echo "   $CONTAINER_PG em localhost:$PORTA_PG, banco $BANCO"

echo "== Grafana de verdade (podman) =="
podman run -d --rm --name "$CONTAINER_GRAFANA" --network host \
    -e GF_SERVER_HTTP_PORT="$PORTA_GRAFANA" \
    -e GF_SECURITY_ADMIN_USER=admin \
    -e GF_SECURITY_ADMIN_PASSWORD=admin \
    "$IMAGEM_GRAFANA" >/dev/null
echo "   $CONTAINER_GRAFANA em localhost:$PORTA_GRAFANA"

# Um projeto minimo: a raiz precisa existir e ser abrivel; o resto do teste
# cria o que precisa pelo proprio core.
printf 'prova do dominio grafana\n' > "$PROJETO/README.md"

KINEIN_GRAFANA_URL="http://localhost:$PORTA_GRAFANA" \
KINEIN_PG_HOST=localhost \
KINEIN_PG_PORT="$PORTA_PG" \
KINEIN_PG_BANCO="$BANCO" \
KINEIN_PROJETO_TESTE="$PROJETO" \
KINEIN_HOME_TESTE="$HOME_TESTE" \
    python3 scripts/testar_grafana_real.py
