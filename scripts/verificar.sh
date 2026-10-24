#!/usr/bin/env bash
# Gate unico de verificacao do Kinein Vectis.
#
# Roda toda a validacao em sequencia e PARA no primeiro erro (set -e), para
# evitar o caso em que um passo falha mas os seguintes continuam e dao falsa
# sensacao de "tudo passou". Ver DocsPublic/arquitetura/15-divida-de-engenharia-e-refatoracao.md.
#
# Uso:
#   scripts/verificar.sh            # completo: lint + testes + C++ + builds debug/release + o binario ABRE
#   scripts/verificar.sh --rapido   # rapido: lint + testes + C++ (sem builds finais/smokes)
#   scripts/verificar.sh --ci-rapida # Rust + gates estaticos, sem Qt/integracoes
#   scripts/verificar.sh --estrito  # combina com os tres perfis: NAO PROVADO reprova
#
# PERFIL ATUAL (autor, 2026-10-07): o Qt local do sistema. Na mesma data o
# desenvolvimento migrou para Arch Linux, pelas versoes estaveis mais recentes
# (Qt 6.12.0 medido; antes 6.10 no Ubuntu). Os gates Qt 6.4 do AppImage
# anterior ficam legados e fora desta sequencia; qmllint, CTest, harnesses e
# abertura locais continuam obrigatorios. Nao muda SDK/CMake.
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

mode="completo"
strict=0
for arg in "$@"; do
    case "$arg" in
        --rapido | --rapida | -r) mode="rapido" ;;
        --completo | -c) mode="completo" ;;
        --ci-rapida) mode="ci-rapida" ;;
        --estrito | -e) strict=1 ;;
        *)
            echo "uso: $0 [--rapido|--completo|--ci-rapida] [--estrito]" >&2
            exit 2
            ;;
    esac
done

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$raiz"

# RECURSOS DA RODADA (2026-10-24, roadmaps/52 §14.47). No mesmo dia o gate e o
# que ficava para tras dele travaram a maquina do autor duas vezes: 7,8 GB de
# restos de testes e provas no /tmp (tmpfs: RAM, e dali para o swap) e o
# paralelismo padrao do Ninja (nucleos + 2) e do cargo (nucleos), que nao olha
# a memoria livre. A rodada garante tres coisas.
#
# 1. SO' RAM. Ela se executa num escopo do systemd do usuario com swap zero e
#    um teto: a memoria disponivel no inicio menos uma reserva para o desktop.
#    Passar do teto encerra um processo DO GATE, e o fim da rodada diz isso;
#    sem o escopo, o OOM killer escolhia o maior processo da maquina (o CLion,
#    7,6 GB, em 2026-10-24 17:03). Sem systemd do usuario (CI, container) ou
#    com KINEIN_GATE_SCOPE=0, a rodada segue sem escopo e diz isso.
# 2. TEMPORARIOS DA RODADA. TMPDIR aponta para ${TMPDIR:-/tmp}/kinein-gate.<pid>,
#    apagada no fim; a de uma rodada morta por SIGKILL (o trap nao roda) e'
#    apagada pela seguinte. Nenhum outro nome do /tmp e' tocado: pode ser da
#    IDE aberta. O runner do cargo (scripts/isolar-temporarios.sh) faz o mesmo
#    por binario de teste, tambem fora do gate.
# 3. PARALELISMO PELA MEMORIA. Sem CMAKE_BUILD_PARALLEL_LEVEL/CARGO_BUILD_JOBS
#    no ambiente, cada um sai do teto dividido pelo pico MEDIDO de um job, sem
#    passar da CPU. Valor do ambiente e' respeitado e dito.
memory_info="${KINEIN_MEMINFO:-/proc/meminfo}"
available_mib="$(awk '/^MemAvailable:/ { print int($2 / 1024) }' "$memory_info")"
swap_used_mib="$(awk '/^SwapTotal:/ { total = $2 } /^SwapFree:/ { free = $2 }
    END { print int((total - free) / 1024) }' "$memory_info")"
reserve_mib=2048
ceiling_mib=$((available_mib - reserve_mib))
if [ "$ceiling_mib" -lt 1024 ]; then
    ceiling_mib=1024
fi
# O teto vem da rodada de fora so' para a de dentro do escopo; um gate que
# roda dentro de outro (o primeiro passo faz isso) calcula o seu.
if [ "${KINEIN_GATE_SCOPE:-}" = inside ] && [ -n "${KINEIN_GATE_CEILING_MIB:-}" ]; then
    ceiling_mib="$KINEIN_GATE_CEILING_MIB"
fi

if [ "${KINEIN_GATE_SCOPE:-1}" = 1 ] && command -v systemd-run >/dev/null 2>&1 \
    && systemd-run --user --scope --quiet --collect true >/dev/null 2>&1; then
    export KINEIN_GATE_SCOPE=inside KINEIN_GATE_CEILING_MIB="$ceiling_mib"
    exec systemd-run --user --scope --quiet --collect \
        -p MemorySwapMax=0 -p "MemoryMax=${ceiling_mib}M" \
        bash "$raiz/scripts/verificar.sh" "$@"
fi
own_cgroup="/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup 2>/dev/null)"
case "${KINEIN_GATE_SCOPE:-1}" in
    inside)
        if [ "$(cat "$own_cgroup/memory.max" 2>/dev/null)" = "$((ceiling_mib * 1024 * 1024))" ]; then
            scope_line="so' RAM: escopo com swap zero e teto de $ceiling_mib MiB"
        else
            scope_line="aviso: o escopo nao aplicou o teto (controlador de memoria nao delegado ao usuario)"
        fi
        ;;
    0) scope_line="sem escopo de memoria (KINEIN_GATE_SCOPE=0)" ;;
    *) scope_line="sem escopo de memoria (systemd do usuario indisponivel)" ;;
esac

tmp_base="${TMPDIR:-/tmp}"
for stale in "$tmp_base"/kinein-gate.*; do
    owner="${stale##*.}"
    case "$owner" in
        '' | *[!0-9]*) continue ;;
    esac
    if [ -d "$stale" ] && ! kill -0 "$owner" 2>/dev/null; then
        rm -rf "$stale"
    fi
done
run_tmp="$tmp_base/kinein-gate.$$"
mkdir -p "$run_tmp"
export TMPDIR="$run_tmp"

# Picos medidos em 2026-10-24 (52 §14.47 e §14.48). C++: a UI inteira com 18
# compiladores chegou a 6.453 MiB (358 por job; o maior cc1plus, 704), e 500
# deixa margem. Rust: o maior rustc, 994 MiB, e o maior clippy-driver, 985.
# clang-tidy: o maior, 626 MiB. Harness QML: o maior, 196 MiB
# (tst_grafana_web, com o WebEngine; a mediana, 48).
cores="$(nproc)"
jobs_within() {
    local count=$((ceiling_mib / $1))
    if [ "$count" -lt 1 ]; then
        count=1
    elif [ "$count" -gt "$2" ]; then
        count="$2"
    fi
    echo "$count"
}
# Variavel, rotulo, MiB de pico de um job e teto da CPU; o valor do ambiente vale.
jobs_line=""
choose_jobs() {
    local origin="do ambiente"
    if [ -z "${!1:-}" ]; then
        printf -v "$1" '%s' "$(jobs_within "$3" "$4")"
        origin="pela memoria"
    fi
    export "${1?}"
    jobs_line="$jobs_line$2 ${!1} ($origin), "
}
choose_jobs CMAKE_BUILD_PARALLEL_LEVEL C++ 500 $((cores + 2))
choose_jobs CARGO_BUILD_JOBS cargo 1000 "$cores"
choose_jobs KINEIN_TIDY_JOBS clang-tidy 700 "$cores"
choose_jobs KINEIN_QML_JOBS "harnesses QML" 200 "$cores"

echo "== recursos da rodada =="
echo "memoria: $available_mib MiB disponiveis, $swap_used_mib MiB de swap em uso; $scope_line"
echo "paralelismo: $jobs_line$cores nucleos"
tmp_line="temporarios: $run_tmp, apagados no fim"
if [ "$(stat -f -c %T "$tmp_base" 2>/dev/null)" = tmpfs ]; then
    tmp_used_mib="$(df -m --output=used "$tmp_base" | tail -n 1 | tr -d ' ')"
    tmp_line="$tmp_line; $tmp_base e' tmpfs (RAM) com $tmp_used_mib MiB ocupados"
fi
echo "$tmp_line"
if [ "$available_mib" -lt $((reserve_mib + 4096)) ]; then
    echo "aviso: so' $available_mib MiB disponiveis; feche programas antes do gate." >&2
fi
if [ "${tmp_used_mib:-0}" -gt 1024 ]; then
    echo "aviso: $tmp_base ocupa $tmp_used_mib MiB de RAM; veja o que e' lixo com" \
        "'du -xsh $tmp_base/* | sort -rh | head'." >&2
fi

preset_debug="${KINEIN_PRESET_DEBUG:-dev-local}"
preset_release="${KINEIN_PRESET_RELEASE:-dev-local-release}"

# Cada gate de ambiente acrescenta aqui o que nao provou (gate<TAB>o que<TAB>por que).
unproven_file="$(mktemp "${TMPDIR:-/tmp}/kinein-nao-provado.XXXXXX")"
export KINEIN_UNPROVEN_FILE="$unproven_file"

current_step=""
# shellcheck disable=SC2154  # `estado` e' atribuido na 1a instrucao do trap.
trap 'estado=$?; rm -rf "$run_tmp"; if [ "$estado" -ne 0 ]; then
    echo ""
    echo "✗ FALHOU em: ${current_step:-inicializacao} (exit $estado)"
    report_memory_kills
fi' EXIT

# O kernel so' encerra processo da rodada por memoria dentro do escopo; o
# memory.events dele conta quantos. Sem isso a falha parecia de compilador.
report_memory_kills() {
    local kills
    kills="$(awk '/^oom_kill / { print $2 }' "$own_cgroup/memory.events" 2>/dev/null)"
    if [ "${KINEIN_GATE_SCOPE:-}" = inside ] && [ "${kills:-0}" -gt 0 ]; then
        echo "✗ a rodada passou do teto de $ceiling_mib MiB: o kernel encerrou $kills processo(s) dela."
        echo "  Feche programas (o teto sai da memoria disponivel no inicio) ou fixe menos"
        echo "  jobs em CMAKE_BUILD_PARALLEL_LEVEL/CARGO_BUILD_JOBS."
    fi
}

# Quanto cada etapa levou, para que a otimizacao do gate parta de numero.
step_clock=$SECONDS
step_durations=()
close_step() {
    if [ -n "$current_step" ]; then
        step_durations+=("$((SECONDS - step_clock))"$'\t'"$current_step")
        echo "($((SECONDS - step_clock)) s)"
    fi
}

passo() {
    close_step
    current_step="$1"
    step_clock=$SECONDS
    echo ""
    echo "== $1 =="
    echo "funcao: ${2:?cada etapa precisa de uma descricao nao vazia}"
}

passo "scripts/test_verification_profiles.py" \
    "Prova perfis, fail-fast, NAO PROVADO estrito, recursos da rodada e runner do cargo, sem ferramentas externas."
python3 scripts/test_verification_profiles.py

if [ "$mode" != "ci-rapida" ]; then
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
fi

passo "cargo fmt --all --check" \
    "Confere a formatacao Rust sem modificar os arquivos."
cargo fmt --all --check

passo "cargo test --workspace --all-features --no-fail-fast --locked" \
    "Executa os testes Rust de todos os crates, em paralelo (a corrida de ETXTBSY foi resolvida na causa)."
# EM PARALELO de novo desde 2026-10-03.
#
# De 2026-09-24 a 2026-10-03 o gate rodou em UMA thread. Varios testes
# escrevem um executavel e o rodam; em paralelo, outra thread dava `fork` na
# janela entre escrever e executar, o filho herdava o descritor de escrita (o
# `CLOEXEC` do Rust so' fecha no `exec`, nao no `fork`) e o `exec` do script
# voltava `ETXTBSY` ("Text file busy"). Serializar escondia a classe.
#
# A CAUSA foi resolvida: todo executavel de teste e' gravado por
# `crate::write_executable` (lib.rs), que nunca abre o executavel para escrita
# neste processo — o conteudo vai para um arquivo de passagem e quem cria o
# executavel e' o `install -m 755`. MEDIDO: 25 rodadas paralelas seguidas da
# suite do core sem falha (antes: 3 falhas em 12), e 13 s em paralelo contra
# ~44 s em serie para o workspace.
#
# --no-fail-fast: sem ele o cargo para no PRIMEIRO crate que falha e os
# outros nem rodam — medido em 2026-10-01, duas falhas do kinein-core
# esconderam o resultado de kinein-protocol e kinein-config. O gate reprova
# do mesmo jeito; so' passa a mostrar TUDO o que esta' vermelho de uma vez.
cargo test --workspace --all-features --no-fail-fast --locked

passo "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings" \
    "Reprova qualquer diagnostico do Clippy em codigo, testes e alvos Rust."
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

passo "scripts/verificar-deny.sh" \
    "Audita licencas, vulnerabilidades, fontes e duplicacoes das dependencias Rust."
bash scripts/verificar-deny.sh

passo "scripts/verificar-shell.sh" \
    "Aplica ShellCheck aos scripts rastreados que sustentam os outros gates."
bash scripts/verificar-shell.sh

passo "scripts/verificar-appimage.sh" \
    "Confere as invariantes baratas da distribuicao AppImage sem empacotar."
if [ "$mode" = "ci-rapida" ]; then
    bash scripts/verificar-appimage.sh --invariantes
else
    bash scripts/verificar-appimage.sh
fi

if [ "$mode" != "ci-rapida" ]; then
    passo "scripts/verificar-cpp.sh" \
        "Valida formatacao e analise estatica do C++ da ponte/UI."
    scripts/verificar-cpp.sh

    passo "scripts/verificar-cpp-testes.sh" \
        "Roda os testes C++ da UI pelo CTest; reprova se nenhum for declarado."
    bash scripts/verificar-cpp-testes.sh --preset "$preset_debug"

    passo "scripts/verificar-qml.sh" \
        "Executa o qmllint estrito no modulo QML usando os metadados do build."
    scripts/verificar-qml.sh
fi

passo "scripts/check_identifier_language.py" \
    "Reprova identificador novo em portugues (Rust, C++, QML, Python, shell); legado so' desce; o mesmo veredito em toda distro."
python3 scripts/test_identifier_language.py
python3 scripts/check_identifier_language.py

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

passo "scripts/verificar-qml-tokens.sh" \
    "Impede raio, fonte, duracao e cor escritos a mao no QML (catraca: so' descem)."
bash scripts/verificar-qml-tokens.sh

passo "scripts/verificar-qml-alcance.sh" \
    "Reprova componente QML entregue pelo modulo que nenhuma tela alcanca."
bash scripts/verificar-qml-alcance.sh

passo "scripts/verificar-fiacao-ipc.sh" \
    "Inspeciona referencias na cadeia IPC, do metodo/evento ao consumidor QML."
bash scripts/verificar-fiacao-ipc.sh

if [ "$mode" != "ci-rapida" ]; then
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
fi

passo "scripts/verificar-atalhos.sh" \
    "Mantem atalhos, paleta e tratamento dos itens de menu coerentes e sem colisao."
bash scripts/verificar-atalhos.sh

passo "scripts/verificar-docs.sh" \
    "Compara contagens de linhas sem data nos documentos com os arquivos citados."
bash scripts/verificar-docs.sh

passo "scripts/module_map.py --check" \
    "Reprova o mapa de modulos que divergiu do codigo e ciclo novo entre modulos do core."
python3 scripts/module_map.py --check

passo "scripts/verificar-links-docs.sh" \
    "Reprova links Markdown relativos cujo alvo nao existe no repositorio."
bash scripts/verificar-links-docs.sh

passo "scripts/verificar-arquitetura.sh" \
    "Aplica limites por categoria de arquivo e impede crescimento da divida de tamanho."
bash scripts/verificar-arquitetura.sh

passo "scripts/verificar-adiados-windows.sh" \
    "Conta os testes adiados no Windows por fatia e os cfg(unix); os dois so' descem."
bash scripts/verificar-adiados-windows.sh

passo "scripts/verificar-transicao-workspace.sh" \
    "Garante que a troca do estado por workspace continua com um unico dono."
bash scripts/verificar-transicao-workspace.sh

if [ "$mode" != "ci-rapida" ]; then
    passo "scripts/verificar-qml-logica.sh" \
        "Executa em modo headless a logica real dos controllers e componentes QML."
    scripts/verificar-qml-logica.sh
fi

if [ "$mode" = "completo" ]; then
    # "Compila" e "abre" sao afirmacoes diferentes (2026-09-10: dois builds
    # verdes que abortavam ao abrir). Roda DEPOIS do build, e para cada preset:
    # o objeto obsoleto vive na arvore, nao no fonte.
    passo "scripts/verificar-binario-abre.sh --preset $preset_debug" \
        "Abre o binario debug recem-compilado e exige primeiro frame sem aviso QML."
    bash scripts/verificar-binario-abre.sh --preset "$preset_debug"

    passo "scripts/check_terminal_quiet.py --preset $preset_debug" \
        "Abre pela linha de comando num pty: volta em < 300 ms e nao imprime nada (como code .)."
    python3 scripts/check_terminal_quiet.py --preset "$preset_debug"

    # O adaptador SQLite mora AO LADO do core (arquitetura/39 §5.2): sem ele
    # em target/release, o launcher sobe o core sem o "Adaptador: o da IDE"
    # (achado no 9a.3, 2026-10-09). O debug vem do `cargo test --workspace`.
    passo "cargo build --release -p kinein-core -p kinein-adapter-sqlite" \
        "Compila o core Rust e o adaptador SQLite em release, como serao consumidos pela distribuicao."
    cargo build --release -p kinein-core -p kinein-adapter-sqlite

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

close_step
current_step=""
echo ""
echo "== tempo: $SECONDS s; as etapas mais longas =="
printf '%s\n' "${step_durations[@]}" | sort -rn | head -n 5 \
    | while IFS=$'\t' read -r seconds name; do printf '  %5d s  %s\n' "$seconds" "$name"; done
echo ""
if [ "$mode" = "completo" ]; then
    scope="completo — builds e abertura dos presets selecionados validados"
elif [ "$mode" = "ci-rapida" ]; then
    scope="CI rapida — Rust e gates estaticos; sem UI compilada, harnesses ou integracoes"
else
    scope="rapido — sem builds finais/smokes; rode --completo antes de release"
fi
if [ "$unproven_count" -eq 0 ] && [ "$mode" = "ci-rapida" ]; then
    echo "✓ VERDE no perfil $scope"
elif [ "$unproven_count" -eq 0 ]; then
    echo "✓ TUDO VERDE ($scope)"
else
    echo "✓ VERDE no que esta maquina prova ($scope)"
    echo "  $unproven_count item(ns) NAO PROVADO(S) acima: instale a ferramenta ou rode onde ela existe;"
    echo "  --estrito os reprova (antes de release e em CI)."
fi
