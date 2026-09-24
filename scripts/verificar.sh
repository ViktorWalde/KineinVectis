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
case "${1:-}" in
    --rapido | --rapida | -r) modo="rapido" ;;
    --completo | -c | "") modo="completo" ;;
    *)
        echo "uso: $0 [--rapido|--completo]" >&2
        exit 2
        ;;
esac

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$raiz"

preset_debug="${KINEIN_PRESET_DEBUG:-dev-local}"
preset_release="${KINEIN_PRESET_RELEASE:-dev-local-release}"

etapa=""
# shellcheck disable=SC2154  # `estado` e' atribuido na 1a instrucao do trap.
trap 'estado=$?; if [ "$estado" -ne 0 ]; then
    echo ""
    echo "✗ FALHOU em: ${etapa:-inicializacao} (exit $estado)"
fi' EXIT

passo() {
    etapa="$1"
    echo ""
    echo "== $1 =="
    echo "funcao: ${2:?cada etapa precisa de uma descricao nao vazia}"
}

passo "cargo fmt --all --check" \
    "Confere a formatacao Rust sem modificar os arquivos."
cargo fmt --all --check

passo "cargo test --workspace --all-features" \
    "Executa os testes Rust de todo o workspace com todos os recursos."
cargo test --workspace --all-features

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

passo "scripts/verificar-qml-fiacao.sh" \
    "Detecta bindings QML auto-referentes que entregariam valores nulos ou errados."
bash scripts/verificar-qml-fiacao.sh

passo "scripts/verificar-qml-propriedades.sh" \
    "Detecta propriedades, sinais, ancoras e indentacoes de binding QML invalidos."
bash scripts/verificar-qml-propriedades.sh

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

if [ "$modo" = "completo" ]; then
    passo "cmake --build --preset $preset_debug" \
        "Compila a UI de desenvolvimento com as protecoes do preset selecionado."
    cmake --build --preset "$preset_debug"

    # "Compila" e "abre" sao afirmacoes diferentes (2026-09-10: dois builds
    # verdes que abortavam ao abrir). Roda DEPOIS do build, e para cada preset:
    # o objeto obsoleto vive na arvore, nao no fonte.
    passo "scripts/verificar-binario-abre.sh --preset $preset_debug" \
        "Abre o binario debug recem-compilado e exige primeiro frame sem aviso QML."
    bash scripts/verificar-binario-abre.sh --preset "$preset_debug"

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

etapa=""
echo ""
if [ "$modo" = "completo" ]; then
    echo "✓ TUDO VERDE (completo — builds e abertura dos presets selecionados validados)"
else
    echo "✓ TUDO VERDE (rapido — sem builds finais/smokes; rode --completo antes de release)"
fi
