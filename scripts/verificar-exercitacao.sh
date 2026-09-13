#!/usr/bin/env bash
# EXERCITACAO: o core contra as ferramentas REAIS desta maquina.
#
# POR QUE ESTE SCRIPT EXISTE (2026-09-04). Relato de uso do autor: "a barra de
# pesquisa nao e' de fato funcional". Medido, era verdade, e o defeito nao
# estava na UI:
#
#   fs.findFiles -> INTERNAL_ERROR
#     "fd falhou: error: the argument '--strip-cwd-prefix[=<when>]' cannot be
#      used with '[path]...'"
#
# O `fd` 10.4.2 passou a aceitar valor nessa opcao e ela virou incompativel com
# passar um caminho — que e' o que o core faz. TODA busca por nome de arquivo
# da IDE falhava, e o gate estava verde com dezessete verificacoes.
#
# Por que passava: o teste de unidade do `find.rs` roda um `fd` FALSO, um script
# que imprime linhas fixas. Fixture inventada nao ve' mudanca de CLI. E' a
# MESMA licao do `probe.rs` em 2026-09-03, e ela custou duas vezes.
#
# O que este gate faz: sobe o core de verdade, abre um workspace de verdade e
# exercita os endpoints que dependem de ferramenta EXTERNA. Nao verifica
# formato de JSON — verifica que a ferramenta ainda responde ao que o core
# pede.
#
# Ferramenta AUSENTE nao reprova (nem toda maquina tem tudo); ferramenta
# PRESENTE que recusa o comando reprova, e e' esse o caso que este gate existe
# para pegar.
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

echo "== exercitacao (o core contra as ferramentas reais) =="

binario="target/debug/kinein-core"
if [ ! -x "$binario" ]; then
    echo "-> compilando o core para exercitar"
    cargo build -q -p kinein-core || exit 1
fi

raiz="$(mktemp -d)"
trap 'rm -rf "$raiz"' EXIT
mkdir -p "$raiz/src"
cat > "$raiz/CMakeLists.txt" <<'CMAKE'
cmake_minimum_required(VERSION 3.24)
project(exercitacao CXX)
set(CMAKE_CXX_STANDARD 23)
add_executable(alvo_da_exercitacao src/main.cpp)
CMAKE
printf 'int main() { return 0; }  // AGULHA_DA_EXERCITACAO\n' > "$raiz/src/main.cpp"
# Um .py ao lado: o indice le Python com a gramatica oficial (2026-09-12), e a
# declaracao tem de aparecer no index.symbols com a linguagem certa.
mkdir -p "$raiz/tools"
printf 'def gera_tabela(n):\n    return list(range(n))\n' > "$raiz/tools/gera.py"
# Um pyproject.toml faz o workspace ser Python TAMBEM (cmake por fora): o
# python.status resolve o interpretador desta maquina e diz se ha' ambiente.
printf '[project]\nname = "exercitacao"\nversion = "0.1.0"\n' > "$raiz/pyproject.toml"
# Uma compile_commands.json escrita a mao, na forma `command` do padrao do
# clang: e' o que o contexto de compilador por arquivo le, e o que prova que o
# job do indice carrega o contexto (nos testes de unidade nao ha' job).
mkdir -p "$raiz/build"
cat > "$raiz/build/compile_commands.json" <<CDB
[{"directory": "$raiz/build", "command": "/usr/bin/c++ -DEXERCITACAO=1 -I$raiz/src -std=c++20 -o main.o -c $raiz/src/main.cpp", "file": "$raiz/src/main.cpp"}]
CDB
# Com o cmake REAL: um configure no build dir da IDE (.kinein/build) com a
# query do file-api (codemodel-v2 + toolchains-v1), como o job de configure
# da IDE faz. Dai saem o MODELO POR ALVO (cmake.targets.list com fontes,
# artefato, padrao) e o "arquivo -> target" do index.context; e a CDB deste
# build dir passa a valer (ele tem precedencia sobre build/). Sem cmake, fica
# a CDB a mao e os dois itens do modelo sao ditos como nao exercitados.
configurado=0
if command -v cmake >/dev/null 2>&1; then
    mkdir -p "$raiz/.kinein/build/.cmake/api/v1/query"
    : > "$raiz/.kinein/build/.cmake/api/v1/query/codemodel-v2"
    : > "$raiz/.kinein/build/.cmake/api/v1/query/toolchains-v1"
    if cmake -S "$raiz" -B "$raiz/.kinein/build" -DCMAKE_EXPORT_COMPILE_COMMANDS=ON >/dev/null 2>&1; then
        configurado=1
    fi
fi

resposta="$(
    {
        printf '{"jsonrpc":"2.0","id":1,"method":"workspace.open","params":{"path":"%s"}}\n' "$raiz"
        printf '{"jsonrpc":"2.0","id":2,"method":"fs.findFiles","params":{"query":"main"}}\n'
        printf '{"jsonrpc":"2.0","id":3,"method":"fs.search","params":{"query":"AGULHA_DA_EXERCITACAO"}}\n'
        printf '{"jsonrpc":"2.0","id":4,"method":"fs.list","params":{"path":"%s/src"}}\n' "$raiz"
        printf '{"jsonrpc":"2.0","id":5,"method":"cmake.targets.list","params":{}}\n'
        printf '{"jsonrpc":"2.0","id":6,"method":"serial.list","params":{}}\n'
        printf '{"jsonrpc":"2.0","id":7,"method":"container.status","params":{}}\n'
        printf '{"jsonrpc":"2.0","id":8,"method":"container.list","params":{}}\n'
        printf '{"jsonrpc":"2.0","id":9,"method":"serial.monitor","params":{"device":"/dev/null"}}\n'
        printf '{"jsonrpc":"2.0","id":10,"method":"project.model","params":{}}\n'
        sleep 2
        # O indice ja' esta' pronto e registrou as pastas no watcher: um arquivo
        # criado AGORA, numa pasta NOVA que nenhuma tela listou, tem de entrar
        # sozinho (inotify real -> event.fs.changed -> reindex).
        mkdir -p "$raiz/src/tarde"
        printf 'int chegou_tarde(void) { return 1; }\n' > "$raiz/src/tarde/tarde.c"
        sleep 1
        printf '{"jsonrpc":"2.0","id":11,"method":"index.status","params":{}}\n'
        printf '{"jsonrpc":"2.0","id":12,"method":"index.symbols","params":{"query":"main"}}\n'
        printf '{"jsonrpc":"2.0","id":13,"method":"index.context","params":{"path":"src/main.cpp"}}\n'
        printf '{"jsonrpc":"2.0","id":14,"method":"index.symbols","params":{"query":"gera_tabela"}}\n'
        printf '{"jsonrpc":"2.0","id":15,"method":"index.symbols","params":{"query":"chegou_tarde"}}\n'
        printf '{"jsonrpc":"2.0","id":16,"method":"cmake.targets.list","params":{}}\n'
        printf '{"jsonrpc":"2.0","id":17,"method":"python.status","params":{}}\n'
        # Criar o ambiente com a ferramenta REAL desta maquina (uv se houver,
        # senao `python3 -m venv .venv`): o job roda, o evento sai, e o status
        # seguinte tem de apontar o .venv — e' o gesto de um clique da fatia 1.
        printf '{"jsonrpc":"2.0","id":18,"method":"python.createEnvironment","params":{}}\n'
        sleep 6
        printf '{"jsonrpc":"2.0","id":19,"method":"python.status","params":{}}\n'
        sleep 3
    } | "$binario" 2>/dev/null
)"

falhou=0
verifica() {
    local id="$1" nome="$2" esperado="$3"
    local linha
    linha="$(printf '%s\n' "$resposta" | grep -o "\"id\":$id,.*" | head -1)"
    if [ -z "$linha" ]; then
        echo "  ✗ $nome: o core nao respondeu" >&2
        falhou=1
        return
    fi
    case "$linha" in
        *'"TOOL_NOT_FOUND"'*)
            echo "  - $nome: ferramenta ausente nesta maquina (nao reprova)"
            return
            ;;
        *'"error"'*)
            echo "  ✗ $nome: $linha" >&2
            falhou=1
            return
            ;;
    esac
    case "$linha" in
        *"$esperado"*) echo "  ok $nome" ;;
        *)
            echo "  ✗ $nome: respondeu sem encontrar \`$esperado\`" >&2
            echo "     $linha" >&2
            falhou=1
            ;;
    esac
}

verifica 2 "fs.findFiles (fd)" "src/main.cpp"
verifica 3 "fs.search (rg)" "AGULHA_DA_EXERCITACAO"
verifica 4 "fs.list" "main.cpp"
verifica 5 "cmake.targets.list" "alvo_da_exercitacao"
# serial.list le o sysfs e o udevadm DESTA maquina: sem placa a lista e' vazia
# e a resposta ainda tem `ports` — e' isso que se verifica. Com uma placa no
# USB (medido em 2026-09-11 com um ESP32 em /dev/ttyUSB0), a entrada traz o
# veredito de acesso e o estado do ModemManager, e a porta NAO e' aberta.
verifica 6 "serial.list (sysfs + udevadm)" '"ports"'
# container.status responde SEMPRE (motor ausente e' um estado, nao um erro);
# container.list depende do motor: sem docker/podman vem TOOL_NOT_FOUND e nao
# reprova. Com o Podman rootless desta maquina (2026-09-12), a lista vem com os
# containers do autor e o motor identificado como podman.
verifica 7 "container.status (docker|podman)" '"reachable"'
verifica 8 "container.list (ps --format json)" '"containers"'
# serial.monitor abre o monitor do kit (tio|picocom|minicom|espflash) numa aba
# de terminal; sem nenhum instalado vem TOOL_NOT_FOUND e nao reprova. Com um
# deles, a resposta traz `tool` — a ferramenta falhar DENTRO da aba (/dev/null
# nao e' porta) e' assunto da aba, nao deste gate.
verifica 9 "serial.monitor (tio|picocom|minicom|espflash)" '"tool"'
# project.model le os marcadores do workspace (o de exercitacao e' CMake puro):
# responde sempre, com `embedded` — false aqui, e nenhum alvo inventado.
verifica 10 "project.model (o modelo do projeto)" '"embedded"'
# O indice do projeto INTEIRO e' construido em job ao abrir o workspace; dois
# segundos depois o de exercitacao (um .cpp) esta' `ready`, e `main` — a
# funcao do src/main.cpp — e' achada sem LSP nenhum.
verifica 11 "index.status (o indice do projeto)" '"ready"'
verifica 12 "index.symbols (main sem LSP)" '"name":"main"'
# O contexto de compilador por arquivo: o job carregou a CDB junto do indice
# (`cdbEntries` no status) e o src/main.cpp volta com a sua unidade — o -std=
# separado do resto. Sem CDB a resposta traz `hint`, nao erro.
verifica 11 "index.status (contexto carregado no job)" '"cdbEntries":1'
if [ "$configurado" -eq 1 ]; then
    # O configure real escreveu a CDB em .kinein/build: -std=gnu++23 vem do
    # CMAKE_CXX_STANDARD 23 (extensoes GNU sao o padrao do CMake). Medido em
    # 2026-09-12: com 20 o GCC 16.2 desta maquina ja' e' C++20 por padrao e o
    # CMake NAO escreve flag nenhuma — a CDB sai sem -std= e o gate mentiria.
    verifica 13 "index.context (a unidade do src/main.cpp, CDB do configure real)" '"standard":"gnu++23"'
    verifica 13 "index.context (o target que compila o arquivo, pelo file-api)" '"targets":["alvo_da_exercitacao"]'
    verifica 16 "cmake.targets.list (o modelo por alvo: fontes e artefato)" '"sources":1'
    verifica 16 "cmake.targets.list (o padrao da linguagem pelo file-api)" '"standard":"23"'
    verifica 16 "cmake.targets.list (artefato absoluto do file-api)" 'alvo_da_exercitacao"]'
else
    verifica 13 "index.context (a unidade do src/main.cpp, CDB a mao)" '"standard":"c++20"'
    echo "  - cmake.targets.list / arquivo->target: cmake ausente nesta maquina (nao exercitado)"
fi
verifica 14 "index.symbols (Python pela gramatica)" '"language":"python"'
verifica 15 "index.symbols (arquivo nascido depois, em pasta nova, pelo watcher)" '"name":"chegou_tarde"'
# Python (bloco B do roadmaps/41, fatia 1): sem python3 nem uv na maquina o
# createEnvironment recusa com motivo (nao reprova); com um deles, o .venv
# nasce e o status passa a apontar para ele.
verifica 17 "python.status (o interpretador desta maquina)" '"hasEnvironment"'
if printf '%s\n' "$resposta" | grep -q '"id":18,.*"jobId"'; then
    verifica 19 "python.status depois de python.createEnvironment (o .venv nasceu)" '"origin":".venv"'
else
    echo "  - python.createEnvironment: sem python3 nem uv nesta maquina (nao exercitado)"
fi

if [ "$falhou" -ne 0 ]; then
    echo
    echo "✗ exercitacao: uma ferramenta real recusou o que o core pediu" >&2
    echo "  Teste de unidade com binario FALSO nao pega isto — foi assim que" >&2
    echo "  o \`fd\` quebrou a busca inteira com o gate verde." >&2
    exit 1
fi

echo "exercitacao: os endpoints com ferramenta externa responderam."
