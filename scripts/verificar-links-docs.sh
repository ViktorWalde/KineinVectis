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

# CITACAO DE CAMINHO (2026-10-01). Codigo, scripts e comentarios citam
# documentos por caminho (DocsPublic/<pasta>/<nome>.md), sem link Markdown,
# e ate' esta data nada conferia: medido num checkout limpo, 11 citacoes em 6
# alvos apontavam para documentos renomeados ou removidos meses antes, inclusive
# em Rust e C++. Antes da reorganizacao de nomes dos docs, toda citacao
# explicita `DocsPublic/...` em qualquer arquivo rastreado precisa existir. Em
# `.md`, bloco de codigo e' exemplo e fica de fora, como os links acima.
CITATION = re.compile(r"DocsPublic/[A-Za-z0-9_./\-]+?\.(?:md|json)")
tracked = subprocess.run(
    ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
    capture_output=True, text=True, check=True,
).stdout.rstrip("\0").split("\0")
dead_citations = []
citations = 0
for name in tracked:
    if name.startswith(("KV0.3/", "DocsPublic/iconografia/")):
        continue
    path = pathlib.Path(name)
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        continue
    in_fence = False
    for number, line in enumerate(text.split("\n"), start=1):
        if name.endswith(".md") and line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for target in CITATION.findall(line):
            citations += 1
            if not pathlib.Path(target).exists():
                dead_citations.append((name, number, target))

# INDICE DA PASTA (2026-10-01). Cada pasta do DocsPublic tem um README que e' o
# indice dela; um documento que nao aparece nele, PELO NOME, e' invisivel para
# quem chega (o indice das especificacoes listava `onboarding-*.md` e omitia
# tres arquivos). A iconografia e' um pacote importado, com indices proprios.
missing_from_index = []
for readme in sorted(pathlib.Path("DocsPublic").rglob("README.md")):
    if "iconografia" in readme.parts:
        continue
    index_text = readme.read_text(encoding="utf-8")
    for document in sorted(readme.parent.glob("*.md")):
        if document.name != "README.md" and document.name not in index_text:
            missing_from_index.append((str(readme), document.name))

for readme, document in missing_from_index:
    print(f"✗ fora do indice {readme}: {document}", file=sys.stderr)

# SO' O PUBLICO (2026-10-01, decisao do autor). A documentacao publica so' se
# refere a documentacao publica: as notas internas do autor sao esbocos, fora do
# repositorio, e um texto que manda o leitor a um arquivo que ele nao tem
# confunde quem vem colaborar. Medido nesta data: ~300 mencoes em 40 arquivos,
# inclusive instrucoes ("guarde em ...", "leia ..."). Reprova a mencao em
# qualquer arquivo rastreado; ficam de fora so' os que usam o caminho POR
# FUNCAO (excluir da copia, ignorar no git, tratar como registro) e o pacote
# 0.3.5 ja' lancado (KV0.3/, com checksums).
PRIVATE = re.compile(r"DocsPrivate|AGENTS\.md|GUIAIA|ContextoIA|PONTO_ATUAL")
PRIVATE_BY_FUNCTION = {
    ".gitignore", "scripts/exportar-copia-limpa.sh", "scripts/verificar-docs.sh",
    "scripts/check_identifier_language.py", "scripts/verificar-links-docs.sh",
}
private_mentions = []
for name in tracked:
    if name in PRIVATE_BY_FUNCTION or name.startswith("KV0.3/"):
        continue
    try:
        text = pathlib.Path(name).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        continue
    for number, line in enumerate(text.split("\n"), start=1):
        if PRIVATE.search(line):
            private_mentions.append((name, number, line.strip()[:90]))

for name, number, line in private_mentions:
    print(f"✗ mencao a documento interno em {name}:{number}: {line}", file=sys.stderr)

for name, number, target in dead_citations:
    print(f"✗ citacao morta em {name}:{number}: {target}", file=sys.stderr)

if quebrados or dead_citations or missing_from_index or private_mentions:
    print(f"\n✗ links de documentacao FALHOU ({len(quebrados)} links,"
          f" {len(dead_citations)} citacoes, {len(missing_from_index)} fora do indice,"
          f" {len(private_mentions)} mencoes a documento interno)", file=sys.stderr)
    sys.exit(1)

print(f"links de documentacao: {verificados} links relativos e {citations} citacoes"
      " de caminho, nenhum morto; todo documento no indice da sua pasta;"
      " nenhuma mencao a documento interno.")
PY
