#!/usr/bin/env bash
# Verificacao da frente de DISTRIBUICAO (AppImage).
#
# POR QUE EXISTE. Ate 2026-09-02 a frente tinha cinco scripts que funcionavam e
# **nada os executava**: `appimage` aparecia 0 vezes no `verificar.sh`. E a
# mesma forma exata do `deny.toml` antes de 2026-08-30 e do `shellcheck` antes
# de 2026-08-29 — regra que mora num arquivo que ninguem roda. Aqui o gate nao
# nasce de uma falha silenciosa nova; nasce de uma frente inteira sem
# verificacao, que e o criterio da ARCHITECTURE.md §4 regra 11 aplicado a build
# em vez de codigo (roadmap 30, etapa 8).
#
# O QUE ELE NAO FAZ, E POR QUE. Ele NAO empacota. Gerar o AppImage exige
# Podman, rede e uma compilacao completa dentro do Debian 12 baseline — minutos,
# nao segundos. Gate que demora e' gate que se desliga, e gate desligado e' pior
# que gate nenhum. O empacotamento continua sob demanda
# (`scripts/empacotar-appimage.sh`); aqui ficam as invariantes que podem quebrar
# **entre** dois empacotamentos, todas baratas e deterministicas.
#
# O CHEQUE MAIS IMPORTANTE E O PRIMEIRO. A garantia de abertura do AppImage —
# ele sobe em qualquer maquina — depende de a UI ser 100% 2D: o hook forca
# `QT_QUICK_BACKEND=software` porque driver do host quebrava a inicializacao
# (ADR-0003 e DocsPublic/arquitetura/27 §6.2). No dia em que alguem acrescentar um
# `ShaderEffect` a UI, essa garantia morre **em silencio**: o build passa, o
# gate passa, e so o usuario com driver ruim descobre. Ver
# `DocsPublic/roadmaps/31-simulacao-fisica-matematica.md` §5.1, onde essa colisao e
# uma decisao ainda em aberto.
#
# Uso: bash scripts/verificar-appimage.sh
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "== distribuicao AppImage (invariantes da frente) =="

falhou=0

reprovar() {
    echo "✗ $1" >&2
    falhou=1
}

# 1. A UI continua 2D. Sem isto, o backend software do AppImage e uma mentira.
#    A busca ignora comentario de linha: o proprio hook e este script citam os
#    nomes para explicar a regra, e um gate que se reprova sozinho e ruido.
aceleracao="$(
    grep -rnE 'ShaderEffect|QQuickFramebufferObject|QOpenGL|QSGRenderNode|QtQuick3D|QRhi' \
        ui/qml ui/src 2>/dev/null |
        grep -vE ':[[:space:]]*(//|#)' || true
)"
if [[ -n "$aceleracao" ]]; then
    reprovar "a UI deixou de ser 100% 2D — a garantia de abertura do AppImage cai:"
    printf '%s\n' "$aceleracao" >&2
    echo "  O AppImage forca QT_QUICK_BACKEND=software porque driver do host" >&2
    echo "  quebrava a abertura (ADR-0003, arquitetura/27 §6.2). Aceleracao na" >&2
    echo "  UI e decisao de ARQUITETURA, nao detalhe: ver roadmaps/31 §5.1." >&2
fi

# 2. Tudo o que o empacotamento copia precisa existir. Renomear um destes
#    arquivos nao quebra build nenhum — quebra o AppImage, meses depois.
entradas=(
    packaging/appimage/io.github.viktorwalde.KineinVectis.desktop
    packaging/appimage/io.github.viktorwalde.KineinVectis.appdata.xml
    packaging/appimage/kinein-portable-graphics-hook.sh
    packaging/appimage/KINEIN_LICENSE_NOTICE.txt
    packaging/appimage/Containerfile
    packaging/appimage/Containerfile.smoke
    ui/assets/app-icon.png
    LICENSE-MIT.txt
    LICENSE-APACHE-2.0.txt
    DocsPublic/manual.md
    DocsPublic/tutorial.md
)
for entrada in "${entradas[@]}"; do
    [[ -f "$entrada" ]] || reprovar "entrada de empacotamento ausente: $entrada"
done

# 3. O hook grafico continua escolhendo software POR PADRAO. Trocar o default
#    para `auto` seria uma linha, e devolveria a IDE a dependencia de driver
#    que o AppImage evitou.
if ! grep -Fq 'KINEIN_GRAPHICS_BACKEND:-software' \
    packaging/appimage/kinein-portable-graphics-hook.sh
then
    reprovar "o hook grafico nao forca mais 'software' por padrao"
fi

# 4. As ferramentas de terceiros continuam PINADAS com checksum. O ADR-0003
#    exige release auditada; baixar 'latest' sem sha256 e a porta de entrada de
#    supply chain que ele fechou.
for pino in LINUXDEPLOY QT_PLUGIN TYPE2_RUNTIME; do
    if ! grep -qE "^${pino}_SHA256=\"[0-9a-f]{64}\"" scripts/empacotar-appimage.sh; then
        reprovar "$pino sem SHA256 de 64 hex em scripts/empacotar-appimage.sh"
    fi
    if ! grep -qE "^${pino}_URL=" scripts/empacotar-appimage.sh; then
        reprovar "$pino sem URL fixada em scripts/empacotar-appimage.sh"
    fi
done

# 5. O smoke estrutural precisa continuar exigindo o que importa. Se alguem
#    esvaziar a lista de caminhos obrigatorios do testar-appimage.sh, ele passa
#    a aprovar qualquer coisa — e ninguem percebe, porque ele fica VERDE.
obrigatorios=(
    'usr/bin/kinein-vectis'
    'usr/bin/kinein-core'
    'apprun-hooks/kinein-portable-graphics-hook.sh'
    'usr/plugins/platforms/libqxcb.so'
    'usr/plugins/platforms/libqwayland-egl.so'
)
for obrigatorio in "${obrigatorios[@]}"; do
    if ! grep -Fq "$obrigatorio" scripts/testar-appimage.sh; then
        reprovar "testar-appimage.sh parou de exigir: $obrigatorio"
    fi
done
if ! grep -Fq 'Loading backend software' scripts/testar-appimage.sh; then
    reprovar "testar-appimage.sh parou de checar o renderer portatil no smoke"
fi
if ! grep -Fq 'KINEIN_PERF first_frame_ms=' scripts/testar-appimage.sh; then
    reprovar "testar-appimage.sh parou de exigir o primeiro frame"
fi

if [[ "$falhou" -ne 0 ]]; then
    echo >&2
    echo "✗ distribuicao AppImage FALHOU" >&2
    exit 1
fi

# 6. Se ha um AppImage entregue, valide-o de verdade. Isto e oportunista de
#    proposito: o gate nao GERA artefato, mas quando existe um, nao o ignora.
DIST_DIR="${KINEIN_APPIMAGE_DIST_DIR:-$REPO_ROOT/dist}"
# `|| true`: sem artefato, o `find` sai 1 e o `set -e` mataria o script AQUI —
# em silencio, com exit 1 e nenhuma mensagem. Foi o que aconteceu na primeira
# execucao deste gate, e e' o motivo de ele ser testado por mutacao antes de
# entrar no `verificar.sh`.
artefato="$(
    find "$DIST_DIR" -maxdepth 1 -type f -name 'Kinein-Vectis-*-x86_64.AppImage' \
        -print 2>/dev/null | sort -V | tail -n 1 || true
)"
if [[ -n "$artefato" ]]; then
    echo "-> AppImage encontrado em dist/: rodando o smoke completo"
    bash scripts/testar-appimage.sh "$artefato"
    # O smoke prova o ARTEFATO, nao que ele corresponde ao HEAD. Dizer isso e'
    # informacao; reprovar seria alarme falso — regerar um AppImage custa
    # minutos, e gate que grita falso ensina a apagar o dist/. Mesma armadilha
    # que a `sonda_soak.py` recusa (binario velho prova o passado), com o
    # remedio proporcional ao custo.
    if [[ -n "$(find crates ui -newer "$artefato" -name '*.rs' -o -newer "$artefato" -name '*.cpp' \
        -o -newer "$artefato" -name '*.qml' 2>/dev/null | head -n 1)" ]]; then
        echo "   aviso: o AppImage em dist/ e mais VELHO que o codigo-fonte."
        echo "          O smoke acima validou o artefato ENTREGUE, nao o HEAD."
        echo "          Antes de distribuir: bash scripts/empacotar-appimage.sh"
    fi
    echo "✓ distribuicao AppImage: invariantes ok + smoke do artefato entregue"
else
    echo "✓ distribuicao AppImage: invariantes ok (sem artefato em dist/ para o smoke)"
    echo "  Para validar um AppImage de verdade:"
    echo "    bash scripts/empacotar-appimage.sh   # gera (Podman + rede)"
    echo "    bash scripts/testar-appimage.sh      # valida o gerado"
fi
