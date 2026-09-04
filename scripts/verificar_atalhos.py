"""O atalho que a paleta ANUNCIA e' o atalho que a IDE OBEDECE.

POR QUE ESTE SCRIPT EXISTE (2026-09-04). Relato de uso do autor: "nao consigo
acessar as bibliotecas". A causa nao era o painel — era o atalho. O core
declarava `Ctrl+Alt+L` para `library.list`, a paleta mostrava esse texto ao
lado do comando, e a UI ligava `Ctrl+Alt+L` em FORMATAR ARQUIVO, porque o
`format.text` declarava o MESMO atalho. Quem apertava formatava o arquivo.

Medido no mesmo dia, a falha nao era uma:

    Ctrl+Alt+L   library.list      a UI formatava o arquivo
    Ctrl+Alt+D   datasource.list   a UI iniciava o DEBUG (alias de debug.start)
    Ctrl+O       workspace.open    nenhum Shortcut: apertar nao fazia nada
    Alt+Enter    lsp.codeActions   a UI liga Alt+Return; no Qt sao teclas
                                   DIFERENTES (Key_Enter e' o do numerico)

Nenhuma delas produzia erro. O build passava, os quinze gates passavam, e a
unica forma de descobrir era apertar a tecla — que e' a definicao de falha
SILENCIOSA da §4 regra 11.

O QUE ESTE GATE EXIGE, e o contrato e' de mao dupla:

  1. dois comandos NAO podem declarar o mesmo `default_shortcut`;
  2. todo comando com `default_shortcut` tem um `Shortcut` na UI anotado com
     `// comando: <id>`, e a sequencia dele bate com a declarada;
  3. todo `Shortcut` anotado aponta para um comando que EXISTE e declara
     aquele atalho;
  4. a mesma sequencia nao pode estar em dois `Shortcut` diferentes.

A anotacao e' a costura, e ela e' deliberada: sem um elo explicito nao ha' como
uma maquina saber que `onActivated: root.libraryController.open()` implementa
`library.list`. Escrever o elo custa uma linha e torna a mentira impossivel.
"""

import pathlib
import re
import sys

RAIZ = pathlib.Path(__file__).resolve().parent.parent

# Atalhos padrao do Qt: portateis por desenho, e por isso escritos como
# `StandardKey.X` em vez do texto da tecla. O mapa e' o que liga os dois
# mundos; so' entra aqui o que a UI realmente usa.
TECLAS_PADRAO = {
    "StandardKey.Save": "Ctrl+S",
    "StandardKey.Open": "Ctrl+O",
}


def comandos_declarados():
    """id -> atalho, lidos dos descriptors do core."""
    declarados = {}
    pasta = RAIZ / "crates/kinein-core/src/commands"
    for arquivo in sorted(pasta.glob("*.rs")):
        texto = arquivo.read_text()
        for bloco in re.finditer(
            r'id:\s*"([^"]+)"\.to_owned\(\),(.*?)requires_workspace', texto, re.S
        ):
            atalho = re.search(r'default_shortcut:\s*Some\("([^"]+)"', bloco.group(2))
            if atalho:
                declarados[bloco.group(1)] = atalho.group(1)
    return declarados


def atalhos_da_ui():
    """Lista de (arquivo, linha, comando anotado ou None, sequencias)."""
    encontrados = []
    for arquivo in sorted((RAIZ / "ui/qml").rglob("*.qml")):
        linhas = arquivo.read_text().split("\n")
        for indice, linha in enumerate(linhas):
            if linha.strip() != "Shortcut {":
                continue
            fim = indice
            while fim < len(linhas) and linhas[fim].strip() != "}":
                fim += 1
            bloco = "\n".join(linhas[indice:fim])
            anotado = re.search(r"//\s*comando:\s*([\w.]+)", bloco)
            listadas = re.search(r'sequences?:\s*(\[[^\]]*\]|"[^"]*")', bloco)
            sequencias = re.findall(r'"([^"]+)"', listadas.group(1)) if listadas else []
            for chave, texto in TECLAS_PADRAO.items():
                if chave in bloco:
                    sequencias.append(texto)
            encontrados.append(
                (
                    arquivo.relative_to(RAIZ),
                    indice + 1,
                    anotado.group(1) if anotado else None,
                    sequencias,
                )
            )
    return encontrados


def main():
    declarados = comandos_declarados()
    na_ui = atalhos_da_ui()
    falhas = []

    duplicados = {}
    for comando, atalho in declarados.items():
        duplicados.setdefault(atalho, []).append(comando)
    for atalho, comandos in sorted(duplicados.items()):
        if len(comandos) > 1:
            falhas.append(
                f"atalho `{atalho}` declarado por MAIS DE UM comando: "
                f"{', '.join(sorted(comandos))}"
            )

    vistas = {}
    for arquivo, linha, _, sequencias in na_ui:
        for sequencia in sequencias:
            vistas.setdefault(sequencia, []).append(f"{arquivo}:{linha}")
    for sequencia, locais in sorted(vistas.items()):
        if len(locais) > 1:
            falhas.append(
                f"sequencia `{sequencia}` ligada em MAIS DE UM Shortcut: "
                f"{', '.join(locais)}"
            )

    anotados = {}
    for arquivo, linha, comando, sequencias in na_ui:
        if comando is None:
            continue
        anotados.setdefault(comando, []).append((arquivo, linha, sequencias))

    for comando, atalho in sorted(declarados.items()):
        alvos = anotados.get(comando)
        if not alvos:
            falhas.append(
                f"`{comando}` anuncia `{atalho}` na paleta e NAO tem Shortcut "
                f"anotado na UI — apertar nao faz o que a paleta promete"
            )
            continue
        if len(alvos) > 1:
            falhas.append(f"`{comando}` tem mais de um Shortcut anotado")
        arquivo, linha, sequencias = alvos[0]
        if atalho not in sequencias:
            falhas.append(
                f"`{comando}` anuncia `{atalho}` mas o Shortcut de "
                f"{arquivo}:{linha} liga {sequencias or '(nada)'}"
            )

    for comando, alvos in sorted(anotados.items()):
        if comando not in declarados:
            arquivo, linha, _ = alvos[0]
            falhas.append(
                f"{arquivo}:{linha} diz implementar `{comando}`, que nao "
                f"declara atalho nenhum no core"
            )

    if falhas:
        print("✗ atalhos: a paleta promete o que a IDE nao faz", file=sys.stderr)
        for falha in falhas:
            print(f"  {falha}", file=sys.stderr)
        print(
            "\n  A anotacao `// comando: <id>` acima do Shortcut e' o elo. "
            "Sem ela\n  nenhuma maquina sabe que aquele onActivated implementa "
            "aquele comando.",
            file=sys.stderr,
        )
        return 1

    print(
        f"atalhos: {len(declarados)} comandos com atalho, todos ligados ao que "
        f"a paleta anuncia."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
