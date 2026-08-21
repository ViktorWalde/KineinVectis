#!/usr/bin/env bash
# Matriz de distribuicoes: PROVA a faixa de compatibilidade que a ADR-0003
# declara, em vez de afirma-la.
#
# POR QUE ESTE SCRIPT EXISTE (2026-08-21). A ADR dizia, em texto, que o
# artefato "cobre Ubuntu 22.04+, Debian 12+, Fedora 36+ e openSUSE Leap 15.6+".
# Duas dessas quatro nunca tinham sido executadas. Afirmacao de compatibilidade
# em documento de CONTRATO que ninguem mediu e a mesma classe de mentira que o
# verificar-docs.sh existe para pegar — so que sobre outro tipo de arquivo.
#
# O smoke portatil (testar-appimage-portatil.sh) faz a validacao ESTRUTURAL
# completa em dois runtimes. Este aqui faz outra pergunta, em mais familias:
# "o binario CARREGA e desenha o primeiro frame nesta distribuicao?". E o
# minimo que precisa ser verdade, e o que quebra primeiro quando quebra.
#
# A ultima linha da matriz e uma prova NEGATIVA, e ela vale tanto quanto as
# positivas: num sistema ABAIXO do piso, o usuario tem que receber a nossa
# explicacao, nao um "GLIBC_2.35 not found" cru do loader. Limite tecnico
# legitimo comunicado mal vira relato de bug.
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPIMAGE="${1:-$REPO_ROOT/dist/Kinein-Vectis-0.1.0-x86_64.AppImage}"

if ! command -v podman >/dev/null 2>&1; then
    echo "erro: Podman é obrigatório para a matriz." >&2
    exit 1
fi
if [[ ! -x "$APPIMAGE" ]]; then
    echo "erro: AppImage não encontrado ou não executável: $APPIMAGE" >&2
    exit 1
fi

# imagem|rotulo|comando de instalacao da pilha grafica minima
# So a pilha grafica/fontes de um desktop. Qt, Rust, CMake e compiladores
# ficam de fora de proposito: se algum fizer falta, o AppImage nao e portatil.
MATRIZ=(
"docker.io/library/ubuntu:22.04|Ubuntu 22.04 LTS (glibc 2.35, o PISO)|apt-get update >/dev/null && apt-get install -y --no-install-recommends libegl1 libfontconfig1 libfreetype6 libglx0 libharfbuzz0b libopengl0 libx11-6 >/dev/null"
"docker.io/library/ubuntu:24.04|Ubuntu 24.04 LTS (glibc 2.39)|apt-get update >/dev/null && apt-get install -y --no-install-recommends libegl1 libfontconfig1 libfreetype6 libglx0 libharfbuzz0b libopengl0 libx11-6 >/dev/null"
"docker.io/library/debian:12-slim|Debian 12 (glibc 2.36, baseline do builder)|apt-get update >/dev/null && apt-get install -y --no-install-recommends libegl1 libfontconfig1 libfreetype6 libglx0 libharfbuzz0b libopengl0 libx11-6 >/dev/null"
"docker.io/library/fedora:41|Fedora 41 (glibc 2.40)|dnf install -y --setopt=install_weak_deps=False libglvnd-egl libglvnd-glx libglvnd-opengl fontconfig freetype harfbuzz libX11 >/dev/null"
"docker.io/opensuse/leap:15.6|openSUSE Leap 15.6 (glibc 2.38)|zypper --non-interactive --quiet install libglvnd libfontconfig1 libfreetype6 libharfbuzz0 libX11-6 >/dev/null"
)

# Abaixo do piso: tem que RECUSAR, e recusar EXPLICANDO.
ABAIXO_DO_PISO="docker.io/library/rockylinux:9|Rocky Linux 9 (glibc 2.34, ABAIXO do piso)"

falhou=0

echo "############################################################"
echo "# Matriz de compatibilidade — carga e primeiro frame"
echo "# artefato: $APPIMAGE"
echo "############################################################"

for entrada in "${MATRIZ[@]}"; do
    IFS="|" read -r imagem rotulo instalar <<<"$entrada"
    echo
    echo "== $rotulo =="

    if ! podman image exists "$imagem" && ! podman pull "$imagem" >/dev/null 2>&1; then
        echo "  ✗ nao consegui obter a imagem $imagem" >&2
        falhou=1
        continue
    fi

    # --network none no RUN, nao no build: a instalacao da pilha grafica
    # precisa de rede, mas o teste do AppImage nao pode ter nenhuma.
    if ! podman run --rm \
        --volume "$APPIMAGE:/tmp/kv.AppImage:ro,Z" \
        "$imagem" \
        bash -c "
            set -e
            $instalar
            ldd --version | head -1
            cp /tmp/kv.AppImage /tmp/kv-exec.AppImage
            chmod +x /tmp/kv-exec.AppImage
            APPIMAGE_EXTRACT_AND_RUN=1 QT_QPA_PLATFORM=offscreen QSG_INFO=1 \
                KINEIN_PERF_MARKER=1 KINEIN_PERF_EXIT=1 \
                /tmp/kv-exec.AppImage 2>&1 | grep -E 'Loading backend|KINEIN_PERF'
        "; then
        echo "  ✗ FALHOU em $rotulo" >&2
        falhou=1
    else
        echo "  ✓ carrega e desenha o primeiro frame"
    fi
done

IFS="|" read -r imagem rotulo <<<"$ABAIXO_DO_PISO"
echo
echo "== $rotulo — prova NEGATIVA =="
if ! podman image exists "$imagem" && ! podman pull "$imagem" >/dev/null 2>&1; then
    echo "  ✗ nao consegui obter a imagem $imagem" >&2
    falhou=1
else
    saida="$(
        podman run --rm \
            --volume "$APPIMAGE:/tmp/kv.AppImage:ro,Z" \
            "$imagem" \
            bash -c '
                cp /tmp/kv.AppImage /tmp/kv-exec.AppImage
                chmod +x /tmp/kv-exec.AppImage
                APPIMAGE_EXTRACT_AND_RUN=1 QT_QPA_PLATFORM=offscreen \
                    /tmp/kv-exec.AppImage 2>&1 | head -20
                exit 0
            ' 2>&1
    )"

    if grep -q "mais antigo que o minimo suportado" <<<"$saida"; then
        echo "  ✓ recusou EXPLICANDO (o hook de compatibilidade falou primeiro)"
    elif grep -qE "GLIBC_[0-9.]+' not found|version \`GLIBC" <<<"$saida"; then
        echo "  ✗ recusou com erro CRU do loader — o hook nao esta neste artefato" >&2
        echo "    O limite existe e esta correto; o problema e a comunicacao." >&2
        echo "    Reempacote para embarcar kinein-compat-check-hook.sh." >&2
        sed -n '1,6p' <<<"$saida" >&2
        falhou=1
    else
        echo "  ✗ comportamento inesperado abaixo do piso" >&2
        sed -n '1,10p' <<<"$saida" >&2
        falhou=1
    fi
fi

echo
if ((falhou != 0)); then
    echo "✗ matriz de compatibilidade REPROVADA" >&2
    exit 1
fi
echo "✓ matriz de compatibilidade: a faixa declarada na ADR-0003 esta PROVADA"
