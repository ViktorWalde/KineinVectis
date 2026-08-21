#!/usr/bin/env bash
# Roda o smoke do AppImage em runtimes limpos, sem SDK de Qt/Rust.
#
# DOIS runtimes desde 2026-08-21, e a razao e o contrato: o artefato promete
# rodar de Ubuntu 22.04 LTS em diante. Compatibilidade com o piso nao se prova
# no sistema em que a coisa foi construida — prova-se no MAIS VELHO que a
# promessa cobre. O Debian 12 continua porque e o baseline do builder: ele
# responde "funciona fora do ambiente de desenvolvimento?". O Ubuntu 22.04
# responde "funciona no piso que prometemos?". Sao perguntas diferentes.
#
# Isto e complementar, nao redundante, ao scripts/verificar-piso-appimage.sh:
# aquele MEDE os simbolos do binario, este EXECUTA o binario.

set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v podman >/dev/null 2>&1; then
    echo "erro: Podman é obrigatório para o smoke portátil." >&2
    exit 1
fi

# nome:tag|Containerfile|descricao
RUNTIMES=(
    "localhost/kinein-vectis-appimage-smoke:jammy|Containerfile.smoke-jammy|Ubuntu 22.04 LTS — o PISO prometido"
    "localhost/kinein-vectis-appimage-smoke:bookworm|Containerfile.smoke|Debian 12 — o baseline do builder"
)

for entrada in "${RUNTIMES[@]}"; do
    IFS="|" read -r imagem containerfile descricao <<< "$entrada"

    echo
    echo "######################################################################"
    echo "# $descricao"
    echo "######################################################################"

    echo "==> construindo runtime mínimo de validação"
    podman build \
        --file "$REPO_ROOT/packaging/appimage/$containerfile" \
        --tag "$imagem" \
        "$REPO_ROOT"

    echo "==> testando sem rede, Qt/Rust SDKs ou compiladores no host convidado"
    # Fedora/SELinux exige rótulo também no mount somente-leitura do smoke.
    podman run --rm --network none \
        --volume "$REPO_ROOT:/workspace:ro,Z" \
        --workdir /workspace \
        "$imagem" \
        bash scripts/testar-appimage.sh
done

echo
echo "✓ smoke portátil: verde nos dois runtimes (piso e baseline)"
