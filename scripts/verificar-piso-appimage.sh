#!/usr/bin/env bash
# Piso de compatibilidade do AppImage: prova, no artefato, que ele roda no
# sistema mais ANTIGO que prometemos suportar.
#
# Uso:
#   bash scripts/verificar-piso-appimage.sh <AppDir|arquivo.AppImage>
#
# POR QUE ESTE SCRIPT EXISTE (2026-08-21). O autor passou a exigir que o
# artefato rode em Ubuntu 22.04 LTS em diante, em distros baseadas ou nao em
# Ubuntu. A medicao do AppImage de 2026-07-19 mostrou que ele CABIA nesse piso
# — teto GLIBC_2.35, exatamente o do 22.04 — mas por ACIDENTE: o builder e
# Debian 12 (glibc 2.36) e nada travava o teto. Uma dependencia nova, um
# `apt` do container ou um bump de toolchain que toque um simbolo 2.36 quebra
# o 22.04 em SILENCIO, e so um relato de usuario descobre.
#
# Compatibilidade de ELF nao se promete em .md: mede-se no binario. Este script
# transforma o acidente em propriedade verificada, na hora do empacotamento.
#
# O piso e a glibc, NAO o kernel. O kernel GA do Ubuntu 22.04 e 5.15 (6.x so
# via HWE), e "toda distro com kernel 6.x" nao e satisfazivel neste piso — o
# Amazon Linux 2023 tem kernel 6.1 e glibc 2.34. A promessa honesta e
# verificavel e glibc >= 2.35 + libstdc++ >= GCC 12, que cobre Ubuntu 22.04+,
# Debian 12+, Fedora 36+ e openSUSE Leap 15.6+.
set -Eeuo pipefail

# ---------------------------------------------------------------------------
# O PISO. Mudar qualquer numero aqui MUDA O CONTRATO com quem baixa o
# artefato, e exige editar a ADR-0003 e o Tutorial.md no mesmo commit.
# ---------------------------------------------------------------------------
TETO_GLIBC="2.35"      # Ubuntu 22.04 LTS (jammy)
TETO_GLIBCXX="3.4.30"  # libstdc++ do GCC 12, que o jammy ja traz
TETO_CXXABI="1.3.13"   # idem

# Bibliotecas que vem do HOST de proposito: pilha grafica, fontes e runtime de
# base. Empacota-las e o erro classico do AppImage — o driver de GPU do host
# carrega o libstdc++ do host, e um libstdc++ nosso mais VELHO quebraria o
# driver. Esta lista e a fronteira do bundle, e ela e explicita: soname que
# nao esteja empacotado NEM aqui e dependencia pendurada.
LIBS_DO_HOST=(
    ld-linux-x86-64.so.2
    libc.so.6
    libm.so.6
    libdl.so.2
    libpthread.so.0
    librt.so.1
    libresolv.so.2
    libgcc_s.so.1
    libstdc++.so.6
    libEGL.so.1
    libGL.so.1
    libGLX.so.0
    libGLdispatch.so.0
    libOpenGL.so.0
    libX11.so.6
    libX11-xcb.so.1
    libxcb.so.1
    libwayland-client.so.0
    libwayland-server.so.0
    libfontconfig.so.1
    libfreetype.so.6
    libharfbuzz.so.0
    libICE.so.6
    libSM.so.6
    libz.so.1
    libcom_err.so.2
    libgpg-error.so.0
)

if (( $# != 1 )); then
    echo "uso: bash scripts/verificar-piso-appimage.sh <AppDir|arquivo.AppImage>" >&2
    exit 2
fi

ALVO="$1"
TEMPORARIO=""

limpar() {
    if [[ -n "$TEMPORARIO" && -d "$TEMPORARIO" ]]; then
        rm -rf -- "$TEMPORARIO"
    fi
}
trap limpar EXIT

if [[ -f "$ALVO" ]]; then
    # Recebeu o .AppImage pronto: extrai para inspecionar por dentro.
    TEMPORARIO="$(mktemp -d)"
    ALVO_ABSOLUTO="$(cd "$(dirname -- "$ALVO")" && pwd)/$(basename -- "$ALVO")"
    (cd "$TEMPORARIO" && "$ALVO_ABSOLUTO" --appimage-extract >/dev/null)
    APPDIR="$TEMPORARIO/squashfs-root"
elif [[ -d "$ALVO" ]]; then
    APPDIR="$ALVO"
else
    echo "erro: nao e diretorio nem arquivo: $ALVO" >&2
    exit 2
fi

for ferramenta in objdump readelf file find; do
    if ! command -v "$ferramenta" >/dev/null 2>&1; then
        echo "erro: '$ferramenta' e necessario e nao esta no PATH." >&2
        exit 1
    fi
done

echo "== piso de compatibilidade do AppImage =="
echo "   alvo: $APPDIR"
echo "   piso: glibc <= $TETO_GLIBC · GLIBCXX <= $TETO_GLIBCXX · CXXABI <= $TETO_CXXABI"

mapfile -t ELFS < <(
    find "$APPDIR" -type f \( -name '*.so' -o -name '*.so.*' -o -perm -u+x \) -print |
        while IFS= read -r arquivo; do
            if file -b "$arquivo" | grep -q '^ELF'; then
                printf '%s\n' "$arquivo"
            fi
        done
)

if (( ${#ELFS[@]} == 0 )); then
    echo "erro: nenhum ELF encontrado em $APPDIR — o AppDir esta vazio?" >&2
    exit 1
fi

echo "   ELFs inspecionados: ${#ELFS[@]}"

falhou=0

# --- 1. Teto de simbolo versionado ------------------------------------------
# `sort -V` compara versao de verdade: sem ele "2.9" > "2.35" e o gate mentiria
# exatamente ao contrario do que deve fazer.
maior_versao() {
    local prefixo="$1" arquivo="$2"
    objdump -p "$arquivo" 2>/dev/null |
        grep -oE "${prefixo}_[0-9][0-9a-z.]*" |
        sed "s/^${prefixo}_//" |
        sort -uV |
        tail -1
}

excede() {
    local encontrada="$1" teto="$2"
    [[ -n "$encontrada" ]] || return 1
    [[ "$(printf '%s\n%s\n' "$encontrada" "$teto" | sort -V | tail -1)" != "$teto" ]]
}

verificar_teto() {
    local prefixo="$1" teto="$2" arquivo="$3"
    local encontrada
    encontrada="$(maior_versao "$prefixo" "$arquivo")"
    if excede "$encontrada" "$teto"; then
        echo "✗ ${prefixo}_${encontrada} > ${prefixo}_${teto}: ${arquivo#"$APPDIR/"}" >&2
        return 1
    fi
    return 0
}

for arquivo in "${ELFS[@]}"; do
    verificar_teto GLIBC "$TETO_GLIBC" "$arquivo" || falhou=1
    verificar_teto GLIBCXX "$TETO_GLIBCXX" "$arquivo" || falhou=1
    verificar_teto CXXABI "$TETO_CXXABI" "$arquivo" || falhou=1
done

if (( falhou == 0 )); then
    echo "  ok: nenhum ELF exige simbolo acima do piso"
fi

# --- 2. Dependencia pendurada -----------------------------------------------
# `ldd` NAO serve aqui, e este e o ponto: dentro do builder o Qt do sistema
# esta instalado, entao uma lib ausente do AppDir mas presente em /usr/lib
# resolve e o gate passaria. Foi assim que libQt6WlShellIntegration.so.6
# ficou de fora do artefato de 2026-07-19 sem ninguem notar. A pergunta certa
# nao e "o linker acha?", e "esta DENTRO do pacote, ou na lista do host?".
mapfile -t EMPACOTADAS < <(
    find "$APPDIR" -type f \( -name '*.so' -o -name '*.so.*' \) -printf '%f\n' | sort -u
)

esta_empacotada() {
    local soname="$1"
    local candidata
    for candidata in "${EMPACOTADAS[@]}"; do
        [[ "$candidata" == "$soname" ]] && return 0
    done
    return 1
}

vem_do_host() {
    local soname="$1"
    local candidata
    for candidata in "${LIBS_DO_HOST[@]}"; do
        [[ "$candidata" == "$soname" ]] && return 0
    done
    return 1
}

penduradas=0
for arquivo in "${ELFS[@]}"; do
    while IFS= read -r soname; do
        [[ -n "$soname" ]] || continue
        if ! esta_empacotada "$soname" && ! vem_do_host "$soname"; then
            echo "✗ dependencia PENDURADA: $soname" >&2
            echo "    exigida por: ${arquivo#"$APPDIR/"}" >&2
            echo "    nao esta no AppDir e nao esta na lista de libs do host." >&2
            penduradas=1
        fi
    done < <(readelf -d "$arquivo" 2>/dev/null | sed -n 's/.*NEEDED.*\[\(.*\)\]/\1/p')
done

if (( penduradas == 0 )); then
    echo "  ok: nenhuma dependencia pendurada"
else
    falhou=1
fi

if (( falhou != 0 )); then
    echo >&2
    echo "✗ piso de compatibilidade REPROVADO" >&2
    echo "  O artefato nao cumpre a promessa de rodar em Ubuntu 22.04 LTS." >&2
    echo "  Simbolo acima do teto: alguma dependencia subiu de versao — ou o" >&2
    echo "  builder precisa de base mais antiga (ver ADR-0003, plano B)." >&2
    echo "  Dependencia pendurada: a lib falta no AppDir. Empacote-a, ou" >&2
    echo "  declare-a em LIBS_DO_HOST se for mesmo do sistema." >&2
    exit 1
fi

echo "✓ piso de compatibilidade: ok (Ubuntu 22.04 LTS em diante)"
