#!/usr/bin/env bash
# Exportador da copia limpa do codigo (M7.3 de DocsPublic/roadmaps/21; A6 da fila).
#
# Gera, numa arvore SEPARADA, uma copia do projeto contendo o codigo e apenas
# tres arquivos Markdown: README.md, DocsPublic/manual.md e DocsPublic/tutorial.md.
# Tudo o mais que e
# Markdown, e todo o material interno/pessoal, fica de fora — inclusive como
# NOME de caminho.
#
# O que este script NAO faz, de proposito:
#   - nao cria repositorio, nao roda `git init`, nao adiciona remote, nao faz push;
#   - nao altera a visibilidade do repositorio-fonte;
#   - nao copia `.git/` nem historico. Um eventual espelho comeca com historico
#     proprio a partir desta arvore.
#
# Uso:
#   bash scripts/exportar-copia-limpa.sh --dry-run            # padrao: so relata
#   bash scripts/exportar-copia-limpa.sh --out /caminho/dest  # escreve de fato
#
# Antes de usar para valer, o autor deve confirmar a lista de nomes proibidos
# (item A6). O default e dry-run justamente para isso.

set -euo pipefail

REPO_ROOT="$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$REPO_ROOT"

DRY_RUN=1
OUT=""
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=1 ;;
        --out) shift; OUT="${1:-}"; DRY_RUN=0 ;;
        -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
        *) echo "argumento desconhecido: $1" >&2; exit 2 ;;
    esac
    shift
done

# ---------------------------------------------------------------- allowlist
# Somente o que esta aqui entra. Acrescentar item exige decisao consciente.
ALLOW_DIRS="crates ui cmake packaging schemas scripts"
ALLOW_FILES="
README.md
DocsPublic/manual.md
DocsPublic/tutorial.md
LICENSE-MIT.txt
LICENSE-APACHE-2.0.txt
Cargo.toml
Cargo.lock
CMakeLists.txt
CMakePresets.json
rust-toolchain.toml
rustfmt.toml
clippy.toml
deny.toml
.clang-format
.clang-tidy
.clangd
.editorconfig
.gitattributes
.gitignore
imagens/icon-amber.png
"

# Markdown permitido na copia. Qualquer outro .md e um erro, nao um aviso.
ALLOWED_MD="README.md DocsPublic/manual.md DocsPublic/tutorial.md"

# ------------------------------------------------------------------ denylist
# Caminhos que nao podem aparecer NEM COMO NOME na copia. Sao a camada pessoal
# (faixa X, definida em DocsPublic/README.md) e o material interno de engenharia.
#
# As tres arvores de documentacao sao TODAS internas: `DocsPublic/` e' faixa T
# (tecnica, para quem compila) e `DocsPrivate/`/`DocsPrivate/legado/` sao faixa X.
# Publico e' so o que estiver em ALLOWED_MD.
DENY="
DocsPrivate/historico/PONTO_ATUAL.md
GUIAIA.md
AGENTS.md
docs
DocsPrivate
DocsPrivate/legado
imagens/bugs
.claude
.codex
.agents
.idea
.kinein
.git
build
target
dist
"

fail=0
note() { printf '  %s\n' "$*"; }
bad()  { printf '  ✗ %s\n' "$*" >&2; fail=1; }

echo "== copia limpa: montando a lista =="

# Lista final = arquivos rastreados pelo git dentro da allowlist.
# `git ls-files` ja exclui build/target/dist (ignorados) e .git.
LIST_FILE="$(mktemp)"
trap 'rm -f "$LIST_FILE"' EXIT

for f in $ALLOW_FILES; do
    [ -n "$f" ] || continue
    if git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
        echo "$f" >> "$LIST_FILE"
    fi
done
for d in $ALLOW_DIRS; do
    # Markdown de subdiretorio (ex.: ui/README.md) e interno: a copia leva
    # apenas os tres da raiz. A verificacao abaixo continua como rede de
    # seguranca caso alguem acrescente um .md aqui sem pensar.
    git ls-files "$d" 2>/dev/null | grep -v '\.md$' >> "$LIST_FILE" || true
done
sort -u -o "$LIST_FILE" "$LIST_FILE"
total=$(wc -l < "$LIST_FILE")
note "arquivos selecionados: $total"

# ------------------------------------------------------- verificacao 1: deny
echo "== verificando a denylist (nem como caminho) =="
for path in $DENY; do
    [ -n "$path" ] || continue
    if grep -qE "(^|/)${path//./\\.}(/|$)" "$LIST_FILE"; then
        bad "caminho proibido presente na lista: $path"
    fi
done
[ "$fail" = "0" ] && note "nenhum caminho da camada pessoal na lista"

# --------------------------------------------- verificacao 2: markdown extra
echo "== verificando Markdown (so 3 permitidos) =="
md_found="$(grep -E '\.md$' "$LIST_FILE" || true)"
for m in $md_found; do
    ok=0
    for a in $ALLOWED_MD; do [ "$m" = "$a" ] && ok=1; done
    [ "$ok" = "1" ] || bad "Markdown fora da allowlist: $m"
done
md_count=$(printf '%s\n' "$md_found" | grep -c . || true)
note "markdown na copia: $md_count (esperado: 3)"
[ "$md_count" = "3" ] || bad "contagem de Markdown diferente de 3"

# ------------------------------------------- verificacao 3: auditoria de segredos
echo "== auditoria de segredos =="
secret_hits=0
while IFS= read -r f; do
    case "$f" in *.png|*.svg|*.ico|*.zip|Cargo.lock) continue ;; esac
    [ -f "$f" ] || continue
    if grep -nEI '(BEGIN [A-Z ]*PRIVATE KEY|ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|AKIA[0-9A-Z]{16})' "$f" >/dev/null 2>&1; then
        bad "possivel segredo em: $f"
        secret_hits=$((secret_hits + 1))
    fi
done < "$LIST_FILE"
[ "$secret_hits" = "0" ] && note "nenhum padrao de segredo encontrado"

# Caminho absoluto da maquina do autor vazando na copia.
#
# `xargs -0` com a lista separada por NUL, e nao word splitting de `$(tr)`: o
# segundo quebra em caminho com espaco -- e um caminho que o grep nao le e' um
# vazamento que a auditoria nao ve. Numa checagem de SEGREDO, falso negativo e'
# o modo de falha caro.
home_leak="$(tr '\n' '\0' < "$LIST_FILE" \
    | xargs -0 grep -rlI "/home/${USER}/" 2>/dev/null \
    | grep -v exportar-copia-limpa || true)"
if [ -n "$home_leak" ]; then
    bad "caminho absoluto do autor vaza em: $(echo "$home_leak" | tr '\n' ' ')"
fi

# ------------------------------------------------------------------ resultado
echo
if [ "$fail" != "0" ]; then
    echo "✗ FALHOU: a copia limpa NAO esta pronta. Corrija os itens acima." >&2
    exit 1
fi

if [ "$DRY_RUN" = "1" ]; then
    echo "== DRY-RUN: lista final verificavel =="
    sed 's/^/  /' "$LIST_FILE"
    echo
    echo "✓ verificacoes passaram. Nada foi escrito."
    echo "  Para gerar: bash scripts/exportar-copia-limpa.sh --out <destino>"
    echo "  Confirme antes a lista de nomes proibidos (A6)."
    exit 0
fi

# ------------------------------------------------------------------ escrita
if [ -z "$OUT" ]; then
    echo "erro: --out exige um caminho de destino." >&2
    exit 2
fi
case "$OUT" in
    "$REPO_ROOT"|"$REPO_ROOT"/*)
        echo "erro: o destino nao pode ficar dentro do repositorio-fonte." >&2
        exit 2 ;;
esac
if [ -e "$OUT" ] && [ -n "$(ls -A "$OUT" 2>/dev/null)" ]; then
    echo "erro: destino existe e nao esta vazio: $OUT" >&2
    exit 2
fi

echo "== escrevendo em: $OUT =="
mkdir -p "$OUT"
while IFS= read -r f; do
    mkdir -p "$OUT/$(dirname "$f")"
    cp -p "$f" "$OUT/$f"
done < "$LIST_FILE"

echo
echo "✓ copia limpa gerada: $OUT"
echo "  $total arquivos · $md_count Markdown · sem .git/ e sem historico."
echo "  Nenhum repositorio foi criado e nada foi publicado — isso e decisao sua."
