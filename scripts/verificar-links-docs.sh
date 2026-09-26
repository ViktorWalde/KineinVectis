#!/usr/bin/env bash
# Links de documentacao: alvo que nao existe no disco.
#
# POR QUE ESTE SCRIPT EXISTE (2026-08-29). A documentacao deste repositorio ja
# foi reorganizada por `git mv` uma vez (2026-07-16: 21 arquivos movidos, 410
# referencias reescritas) e sera de novo. `git mv` NAO atualiza link nenhum: o
# arquivo muda de lugar e todo `[texto](caminho/antigo.md)` continua apontando
# para o vazio. Nada reclama — nao ha compilador de Markdown, o gate fica verde,
# e o proximo a ler cai num link morto e conclui que o documento nao existe.
#
# Essa e' a classe: RENOMEAR/MOVER quebra referencia em silencio. Um repositorio
# que trata documento como contrato precisa que o contrato seja alcancavel.
#
# O que e' verificado: todo link Markdown `[...](alvo)` relativo, em todo `.md`
# rastreado ou novo nao ignorado pelo git, cujo alvo nao existe. Ancoras (`#secao`) sao cortadas
# antes de resolver; links externos (http, https, mailto) sao ignorados — este
# gate e' offline por principio e nao faz rede.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== links de documentacao (alvo inexistente) =="

python3 - <<'PY'
import pathlib, re, subprocess, sys
from urllib.parse import unquote

CODIGO_EM_LINHA = re.compile(r"`[^`]*`")
LINK = re.compile(r'\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)')
EXTERNO = ("http://", "https://", "mailto:", "#")

arquivos = sorted(set(subprocess.run(
    ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "*.md"],
    capture_output=True, text=True, check=True
).stdout.rstrip("\0").split("\0")))

quebrados = []
verificados = 0
for nome in arquivos:
    caminho = pathlib.Path(nome)
    try:
        texto = caminho.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        continue
    # Um link dentro de bloco de codigo e' EXEMPLO, nao referencia. O mesmo
    # vale para CODIGO EM LINHA (2026-09-25): um registro que explica a sintaxe
    # de imagem do Markdown escreve `![alt](alvo)` entre crases, e "alvo" nao e'
    # um arquivo que deva existir. Sem isto, documentar Markdown reprova o gate.
    fora_de_codigo, dentro = [], False
    for linha in texto.split("\n"):
        if linha.lstrip().startswith("```"):
            dentro = not dentro
            continue
        if not dentro:
            fora_de_codigo.append(CODIGO_EM_LINHA.sub("", linha))
    for numero, linha in enumerate(fora_de_codigo, start=1):
        for alvo in LINK.findall(linha):
            if alvo.startswith(EXTERNO):
                continue
            limpo = unquote(alvo.split("#", 1)[0]).strip()
            if not limpo:
                continue
            verificados += 1
            if not (caminho.parent / limpo).exists():
                quebrados.append((nome, linha.strip()[:100], limpo))

for nome, linha, alvo in quebrados:
    print(f"✗ link morto em {nome}", file=sys.stderr)
    print(f"    alvo: {alvo}", file=sys.stderr)
    print(f"    linha: {linha}", file=sys.stderr)

if quebrados:
    print(f"\n✗ links de documentacao FALHOU ({len(quebrados)})", file=sys.stderr)
    sys.exit(1)

print(f"links de documentacao: {verificados} links relativos, nenhum morto.")
PY
