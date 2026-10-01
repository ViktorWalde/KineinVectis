#!/usr/bin/env bash
# Gate unico de verificacao do Kinein Vectis.
#
# Roda toda a validacao em sequencia e PARA no primeiro erro (set -e), para
# evitar o caso em que um passo falha mas os seguintes continuam e dao falsa
# sensacao de "tudo passou". Ver DocsPublic/arquitetura/15-engineering-debt-and-refactor.md.
#
# Uso:
#   scripts/verificar.sh            # completo: lint + testes + C++ + builds debug/release + o binario ABRE
#   scripts/verificar.sh --rapido   # rapido: lint + testes + C++ (sem builds finais/smokes)
#   scripts/verificar.sh --estrito  # combina com os dois: NAO PROVADO reprova
#
# NAO PROVADO (2026-10-01, scripts/unproven.py). Gate que verifica a integracao
# com o AMBIENTE (QEMU, debugpy, kit cross, Qt 6.4 em container) e nao tem a
# ferramenta nesta maquina registra o que nao provou, em vez de reprovar (seria
# falso positivo para quem acabou de clonar) ou de sair 0 calado (era o que
# acontecia: o resumo dizia "TUDO VERDE" com tres ciclos que nao rodaram). O
# fim deste script lista tudo, e so' diz "TUDO VERDE" com a lista vazia. Antes
# de release e em CI, rode com --estrito: ai' a lista nao vazia reprova.
#
# Antes de rodar, formate o codigo:  cargo fmt --all
# O gate apenas CHECA a formatacao (cargo fmt --check); ele nao altera arquivos.
# Cada etapa imprime sua funcao; fluxo e responsabilidades estao documentados em
# DocsPublic/contribuindo/07-fluxo-e-responsabilidades-dos-gates.md.
#
# Presets CMake podem ser sobrescritos por ambiente (padrao: dev-local*):
#   KINEIN_PRESET_DEBUG   (padrao: dev-local)
#   KINEIN_PRESET_RELEASE (padrao: dev-local-release)
set -euo pipefail

modo="completo"
strict=0
for arg in "$@"; do
    case "$arg" in
        --rapido | --rapida | -r) modo="rapido" ;;
        --completo | -c) modo="completo" ;;
        --estrito | -e) strict=1 ;;
        *)
            echo "uso: $0 [--rapido|--completo] [--estrito]" >&2
            exit 2
            ;;
    esac
done

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$raiz"

preset_debug="${KINEIN_PRESET_DEBUG:-dev-local}"
preset_release="${KINEIN_PRESET_RELEASE:-dev-local-release}"

# Cada gate de ambiente acrescenta aqui o que nao provou (gate<TAB>o que<TAB>por que).
unproven_file="$(mktemp "${TMPDIR:-/tmp}/kinein-nao-provado.XXXXXX")"
export KINEIN_UNPROVEN_FILE="$unproven_file"

current_step=""
# shellcheck disable=SC2154  # `estado` e' atribuido na 1a instrucao do trap.
trap 'estado=$?; rm -f "$unproven_file"; if [ "$estado" -ne 0 ]; then
    echo ""
    echo "✗ FALHOU em: ${current_step:-inicializacao} (exit $estado)"
fi' EXIT

passo() {
    current_step="$1"
    echo ""
    echo "== $1 =="
    echo "funcao: ${2:?cada etapa precisa de uma descricao nao vazia}"
}

# OS BUILDS SAO OS ULTIMOS PASSOS, E ERA TARDE DEMAIS PARA DESCOBRIR (2026-09-25).
#
# O gate reprovou com "build/dev-local-release is not a directory" depois de 25
# minutos de clang-tidy, testes e harnesses: o preset existia no
# `CMakeUserPresets.json`, mas o diretorio nunca tinha sido configurado nesta
# maquina. Nada disso era sobre o codigo — e o tempo ja' tinha sido gasto.
# Conferir os dois diretorios custa milissegundos e diz o comando que resolve.
passo "os diretorios de build existem" \
    "Falha em milissegundos, e nao depois de 25 minutos, quando falta configurar um preset."
for preset in "$preset_debug" "$preset_release"; do
    if [ ! -d "build/$preset" ]; then
        echo "erro: build/$preset nao existe — configure com 'cmake --preset $preset'." >&2
        exit 1
    fi
done
echo "build/$preset_debug e build/$preset_release: configurados."

# O BUILD DE DEBUG VEM ANTES DE QUEM LE' O BUILD (2026-10-01). O qmllint le' o
# `.qmltypes` e as copias do QML no diretorio de build; o clang-tidy e os
# harnesses tambem dependem do que sai dele. Com o build so' no fim, o lint
# corria contra o build de ONTEM (ou reprovava num clone novo, "configurado mas
# NAO compilado"). Incremental, custa segundos quando nada mudou — e no modo
# rapido tambem, porque um lint contra artefato velho e' um gate que mente.
passo "cmake --build --preset $preset_debug" \
    "Compila a UI de debug ANTES dos gates que leem o build (qmllint, harnesses)."
cmake --build --preset "$preset_debug"
export KINEIN_QML_BUILD_DIR="$raiz/build/$preset_debug"

passo "cargo fmt --all --check" \
    "Confere a formatacao Rust sem modificar os arquivos."
cargo fmt --all --check

passo "cargo test --workspace --all-features --no-fail-fast -- --test-threads=1" \
    "Executa os testes Rust em UMA thread (a paralela tem corrida de ETXTBSY), todos os crates."
# UMA THREAD, e isto foi MEDIDO em 2026-09-24.
#
# Varios testes escrevem um executavel e o rodam. Em paralelo, basta outra
# thread dar `fork` na janela entre escrever e executar: o filho HERDA o
# descritor aberto para escrita (o `CLOEXEC` do Rust so' fecha no `exec`, nao no
# `fork`), e o `exec` do primeiro volta `ETXTBSY` — "Text file busy". Como o
# teste estoura segurando o mutex `EXECUTAVEIS`, os chamadores que usam
# `.lock().unwrap()` envenenam em seguida, e uma corrida vira cinco falhas.
#
# O mutex do `lib.rs` nao resolve a classe: ele serializa quem ESCREVE, e o
# problema e' qualquer `fork` concorrente. Serializar os testes resolve.
#
# CUSTO MEDIDO: 11,5 s em paralelo contra 40,7 s em serie. Vinte e nove segundos
# num gate de ~20 minutos, contra reprovacoes aleatorias que custam o gate
# inteiro — e que ensinam a ignorar vermelho, que e' o dano de verdade.
# --no-fail-fast: sem ele o cargo para no PRIMEIRO crate que falha e os
# outros nem rodam — medido em 2026-10-01, duas falhas do kinein-core
# esconderam o resultado de kinein-protocol e kinein-config. O gate reprova
# do mesmo jeito; so' passa a mostrar TUDO o que esta' vermelho de uma vez.
cargo test --workspace --all-features --no-fail-fast -- --test-threads=1

passo "cargo clippy --workspace --all-targets --all-features -- -D warnings" \
    "Reprova qualquer diagnostico do Clippy em codigo, testes e alvos Rust."
cargo clippy --workspace --all-targets --all-features -- -D warnings

passo "scripts/verificar-deny.sh" \
    "Audita licencas, vulnerabilidades, fontes e duplicacoes das dependencias Rust."
bash scripts/verificar-deny.sh

passo "scripts/verificar-shell.sh" \
    "Aplica ShellCheck aos scripts rastreados que sustentam os outros gates."
bash scripts/verificar-shell.sh

passo "scripts/verificar-appimage.sh" \
    "Confere as invariantes baratas da distribuicao AppImage sem empacotar."
bash scripts/verificar-appimage.sh

passo "scripts/verificar-cpp.sh" \
    "Valida formatacao e analise estatica do C++ da ponte/UI."
scripts/verificar-cpp.sh

passo "scripts/verificar-cpp-testes.sh" \
    "Roda os testes C++ da UI pelo CTest; reprova se nenhum for declarado."
bash scripts/verificar-cpp-testes.sh --preset "$preset_debug"

passo "scripts/verificar-qml.sh" \
    "Executa o qmllint estrito no modulo QML usando os metadados do build."
scripts/verificar-qml.sh

passo "scripts/check_identifier_language.py" \
    "Reprova identificador novo em portugues (Rust, C++, QML, Python, shell); legado so' desce."
python3 scripts/check_identifier_language.py

passo "scripts/verificar-qml-qt64.sh" \
    "Recusa parte que o Qt 6.4 do AppImage nunca cria num arquivo com pragma Bound."
bash scripts/verificar-qml-qt64.sh

passo "scripts/verificar-qml-fiacao.sh" \
    "Detecta bindings QML auto-referentes que entregariam valores nulos ou errados."
bash scripts/verificar-qml-fiacao.sh

passo "scripts/verificar-qml-propriedades.sh" \
    "Detecta propriedades, sinais, ancoras e indentacoes de binding QML invalidos."
bash scripts/verificar-qml-propriedades.sh

passo "scripts/verificar-qml-mortas.sh" \
    "Recusa funcao QML nova que ninguem menciona — o dead_code que o QML nao tem."
bash scripts/verificar-qml-mortas.sh

passo "scripts/verificar-qml-duplicacao.sh" \
    "Impede que a mesma regra derivada seja copiada e possa divergir entre QMLs."
bash scripts/verificar-qml-duplicacao.sh

passo "scripts/verificar-qml-alcance.sh" \
    "Reprova componente QML entregue pelo modulo que nenhuma tela alcanca."
bash scripts/verificar-qml-alcance.sh

passo "scripts/verificar-fiacao-ipc.sh" \
    "Inspeciona referencias na cadeia IPC, do metodo/evento ao consumidor QML."
bash scripts/verificar-fiacao-ipc.sh

passo "scripts/verificar-exercitacao.sh" \
    "Exercita o core real contra as ferramentas externas disponiveis na maquina."
bash scripts/verificar-exercitacao.sh

passo "scripts/verificar-embarcado.sh" \
    "Prova no QEMU o ciclo de depuracao embarcada sem exigir placa fisica."
bash scripts/verificar-embarcado.sh

passo "scripts/verificar-python-debug.sh" \
    "Prova MicroPython por porta e a depuracao Python com um debugpy real."
bash scripts/verificar-python-debug.sh

passo "scripts/verificar-clangd-cross.sh" \
    "Prova que o clangd encontra os cabecalhos C++ do compilador cross do kit."
bash scripts/verificar-clangd-cross.sh

passo "scripts/verificar-atalhos.sh" \
    "Mantem atalhos, paleta e tratamento dos itens de menu coerentes e sem colisao."
bash scripts/verificar-atalhos.sh

passo "scripts/verificar-docs.sh" \
    "Compara contagens de linhas sem data nos documentos com os arquivos citados."
bash scripts/verificar-docs.sh

passo "scripts/verificar-links-docs.sh" \
    "Reprova links Markdown relativos cujo alvo nao existe no repositorio."
bash scripts/verificar-links-docs.sh

passo "scripts/verificar-arquitetura.sh" \
    "Aplica limites por categoria de arquivo e impede crescimento da divida de tamanho."
bash scripts/verificar-arquitetura.sh

passo "scripts/verificar-transicao-workspace.sh" \
    "Garante que a troca do estado por workspace continua com um unico dono."
bash scripts/verificar-transicao-workspace.sh

passo "scripts/verificar-qml-logica.sh" \
    "Executa em modo headless a logica real dos controllers e componentes QML."
scripts/verificar-qml-logica.sh

passo "scripts/verificar-qml-logica-qt64.sh" \
    "Roda os mesmos harnesses QML no Qt 6.4 do AppImage (container Debian 12)."
bash scripts/verificar-qml-logica-qt64.sh

if [ "$modo" = "completo" ]; then
    # "Compila" e "abre" sao afirmacoes diferentes (2026-09-10: dois builds
    # verdes que abortavam ao abrir). Roda DEPOIS do build, e para cada preset:
    # o objeto obsoleto vive na arvore, nao no fonte.
    passo "scripts/verificar-binario-abre.sh --preset $preset_debug" \
        "Abre o binario debug recem-compilado e exige primeiro frame sem aviso QML."
    bash scripts/verificar-binario-abre.sh --preset "$preset_debug"

    passo "scripts/check_terminal_quiet.py --preset $preset_debug" \
        "Abre pela linha de comando num pty: volta em < 300 ms e nao imprime nada (como code .)."
    python3 scripts/check_terminal_quiet.py --preset "$preset_debug"

    passo "cargo build --release -p kinein-core" \
        "Compila o core Rust em release, como sera consumido pela distribuicao."
    cargo build --release -p kinein-core

    passo "cmake --build --preset $preset_release" \
        "Compila a UI release; ela inicia o core como processo separado."
    cmake --build --preset "$preset_release"

    passo "scripts/verificar-binario-abre.sh --preset $preset_release" \
        "Abre o binario release recem-compilado e exige primeiro frame sem aviso QML."
    bash scripts/verificar-binario-abre.sh --preset "$preset_release"
fi

# O que esta maquina NAO provou vem antes do veredito, e muda o veredito.
unproven_count=0
if [ -s "$unproven_file" ]; then
    unproven_count="$(wc -l <"$unproven_file")"
    echo ""
    echo "== NAO PROVADO nesta maquina ($unproven_count) =="
    while IFS=$'\t' read -r gate what why; do
        echo "  - [$gate] $what: $why"
    done <"$unproven_file"
    if [ "$strict" -eq 1 ]; then
        current_step="--estrito: $unproven_count item(ns) NAO PROVADO(S) acima"
        exit 1
    fi
fi

current_step=""
echo ""
if [ "$modo" = "completo" ]; then
    scope="completo — builds e abertura dos presets selecionados validados"
else
    scope="rapido — sem builds finais/smokes; rode --completo antes de release"
fi
if [ "$unproven_count" -eq 0 ]; then
    echo "✓ TUDO VERDE ($scope)"
else
    echo "✓ VERDE no que esta maquina prova ($scope)"
    echo "  $unproven_count item(ns) NAO PROVADO(S) acima: instale a ferramenta ou rode onde ela existe;"
    echo "  --estrito os reprova (antes de release e em CI)."
fi
