#!/usr/bin/env bash
# Hook carregado pelo AppRun ANTES de qualquer coisa do Qt.
#
# POR QUE ESTE ARQUIVO EXISTE (2026-08-21). O artefato tem um piso declarado —
# glibc >= 2.35, o do Ubuntu 22.04 LTS — e ate aqui quem estivesse abaixo dele
# recebia isto, do loader, sem mais contexto:
#
#   ./Kinein-Vectis-0.1.0-x86_64.AppImage: /lib64/libc.so.6: version
#   `GLIBC_2.35' not found (required by .../libQt6Core.so.6)
#
# Isso e verdade e e inutil. Nao diz o que instalar, nao diz se o problema e a
# maquina ou o download, e manda a pessoa procurar. Um limite tecnico legitimo
# comunicado assim vira relato de bug.
#
# O limite continua existindo: o que muda e que ele passa a se explicar. A
# checagem e do PISO, nao um teste de suporte — quem passa daqui pode falhar
# por outro motivo, e ai o loader fala, como sempre falou.

# O piso vive em UM lugar so no repositorio: scripts/verificar-piso-appimage.sh
# cobra o mesmo numero no empacotamento. Ao mudar um, mude o outro — a ADR-0003
# trata os dois como a mesma decisao.
KINEIN_PISO_GLIBC="2.35"

kinein_glibc_do_host() {
    # getconf e a via direta e existe na propria glibc. Em sistema que nao seja
    # glibc (musl, por exemplo) ele nao responde isso, e o `ldd --version` de
    # reserva tambem nao vai casar — os dois falhando e justamente o sinal de
    # "nao sei", tratado como nao-bloqueio la embaixo.
    local versao
    versao="$(getconf GNU_LIBC_VERSION 2>/dev/null | awk '{print $2}')"
    if [[ -z "$versao" ]]; then
        versao="$(ldd --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+$')"
    fi
    printf '%s' "$versao"
}

kinein_versao_menor_que() {
    # sort -V compara versao de verdade. Sem ele "2.9" > "2.35" e a checagem
    # barraria justamente os sistemas novos — o inverso do que ela existe para
    # fazer. Mesmo cuidado do verificar-piso-appimage.sh.
    [[ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | head -1)" == "$1" && "$1" != "$2" ]]
}

kinein_glibc_encontrada="$(kinein_glibc_do_host)"

# NAO SEI nao bloqueia. Um hook que impede a IDE de abrir porque nao conseguiu
# ler a versao da libc trocaria uma falha explicada por uma falha inventada.
# Sem certeza, deixa seguir: o loader continua sendo a autoridade final.
if [[ -n "$kinein_glibc_encontrada" ]] &&
    kinein_versao_menor_que "$kinein_glibc_encontrada" "$KINEIN_PISO_GLIBC"; then
    cat >&2 <<MENSAGEM
Kinein Vectis: este sistema e mais antigo que o minimo suportado.

  glibc encontrada neste computador : $kinein_glibc_encontrada
  glibc minima exigida              : $KINEIN_PISO_GLIBC

O AppImage cobre Ubuntu 22.04 LTS ou mais novo, e qualquer distribuicao —
baseada em Ubuntu ou nao — com glibc $KINEIN_PISO_GLIBC ou posterior. Entre elas:
Debian 12+, Fedora 36+, openSUSE Leap 15.6+.

Distribuicoes com glibc anterior a $KINEIN_PISO_GLIBC nao sao cobertas por este
arquivo. Isso inclui RHEL 9, Rocky Linux 9, AlmaLinux 9 e Amazon Linux 2023,
que trazem glibc 2.34. Nesses sistemas o AppImage nao roda mesmo forcando: as
bibliotecas Qt embutidas exigem simbolos que a libc de la nao tem.

O download nao esta corrompido e voce nao fez nada errado — e um limite de
compatibilidade binaria. Para conferir a sua versao:  ldd --version
MENSAGEM
    exit 1
fi

unset KINEIN_PISO_GLIBC kinein_glibc_encontrada
unset -f kinein_glibc_do_host kinein_versao_menor_que
