#!/usr/bin/env bash
# Transicao de workspace: estado por-workspace tem UM dono.
#
# POR QUE ESTE SCRIPT EXISTE (2026-08-29). O `Core` guarda estado que pertence
# ao workspace ABERTO — `workspace`, `fswatch`, `syntax`, `workspace_edits`, a
# raiz do LSP e a store de rascunhos `drafts`. Havia TRES caminhos que trocavam
# o workspace ativo (`workspace.open`, `workspace.createProject`,
# `workspace.close`), cada um com a sua copia da lista. As copias divergiram:
#
#   open           trocava tudo, inclusive `drafts`
#   createProject  trocava tudo MENOS `drafts`   <- bug
#   close          soltava tudo MENOS `drafts`   <- bug
#
# Consequencia medida: depois de criar um projeto pelo assistente, o autosave do
# projeto NOVO ia para o banco `.kinein/kinein.db` do projeto ANTERIOR — ou
# respondia "persistencia local de rascunhos indisponivel" quando nao havia
# anterior. A rede de seguranca de dados (docs/seguranca/23) desligava sem que
# nada reclamasse: build verde, clippy verde, 271 testes verdes.
#
# Classe de falha: "estado que precisa ser trocado JUNTO, espalhado por N
# caminhos". Nenhum compilador pega — cada caminho compila sozinho. Por isso o
# gate e' estrutural: `activate_workspace`/`deactivate_workspace` sao os UNICOS
# lugares autorizados a mexer em `self.workspace` e em `self.drafts`.
#
# Se voce esta aqui porque o gate reprovou: nao adicione a sua linha no seu
# handler. Adicione o estado novo dentro de `activate_workspace` (e o inverso em
# `deactivate_workspace`) e chame o metodo. Ver ARCHITECTURE.md §4 regra 11.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== transicao de workspace (estado por-workspace com um dono) =="

python3 - <<'PY'
import pathlib, re, sys

ALVO = pathlib.Path("crates/kinein-core/src/handlers/workspace.rs")

# Campos do Core que sao do WORKSPACE ABERTO: trocar um sem trocar os outros e'
# o defeito que este gate existe para impedir.
CAMPOS = ("workspace", "drafts")

# Os unicos donos autorizados da troca.
DONOS = ("fn activate_workspace", "fn deactivate_workspace")

if not ALVO.exists():
    print(f"✗ {ALVO} nao existe", file=sys.stderr)
    sys.exit(1)

fontes = sorted(pathlib.Path("crates/kinein-core/src").rglob("*.rs"))
linhas_dono = set()

texto_alvo = ALVO.read_text().split("\n")
for i, linha in enumerate(texto_alvo):
    if any(dono in linha for dono in DONOS):
        # O corpo do dono vai ate a proxima linha com o mesmo recuo fechando `}`.
        recuo = len(linha) - len(linha.lstrip())
        for j in range(i, len(texto_alvo)):
            linhas_dono.add(j + 1)
            if j > i and texto_alvo[j].strip() == "}" and (
                len(texto_alvo[j]) - len(texto_alvo[j].lstrip())
            ) == recuo:
                break

padrao = re.compile(
    r"self\.(" + "|".join(CAMPOS) + r")\s*(=[^=]|\.take\(\)|\.replace\()"
)

achados = []
for f in fontes:
    em_teste = False
    for numero, linha in enumerate(f.read_text().split("\n"), start=1):
        if "#[cfg(test)]" in linha:
            em_teste = True
        if em_teste:
            continue
        if not padrao.search(linha):
            continue
        if f == ALVO and numero in linhas_dono:
            continue
        achados.append((str(f), numero, linha.strip()))

for arquivo, numero, linha in achados:
    print(f"✗ transicao fora do dono: {arquivo}:{numero}", file=sys.stderr)
    print(f"    {linha}", file=sys.stderr)
    print(
        "    Estado por-workspace so muda em activate_workspace/"
        "deactivate_workspace\n"
        "    (crates/kinein-core/src/handlers/workspace.rs). Ponha o estado novo"
        " la\n    e chame o metodo — senao o proximo caminho vai esquecer uma"
        " peca.",
        file=sys.stderr,
    )

if achados:
    print(f"\n✗ transicao de workspace FALHOU ({len(achados)})", file=sys.stderr)
    sys.exit(1)

print(
    "transicao de workspace: "
    f"{', '.join('self.' + c for c in CAMPOS)} so mudam no dono unico."
)
PY
