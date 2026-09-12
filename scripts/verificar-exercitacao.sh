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
add_executable(alvo_da_exercitacao src/main.cpp)
CMAKE
printf 'int main() { return 0; }  // AGULHA_DA_EXERCITACAO\n' > "$raiz/src/main.cpp"

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

if [ "$falhou" -ne 0 ]; then
    echo
    echo "✗ exercitacao: uma ferramenta real recusou o que o core pediu" >&2
    echo "  Teste de unidade com binario FALSO nao pega isto — foi assim que" >&2
    echo "  o \`fd\` quebrou a busca inteira com o gate verde." >&2
    exit 1
fi

echo "exercitacao: os endpoints com ferramenta externa responderam."
