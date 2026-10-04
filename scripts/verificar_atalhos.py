"""O que a IDE OFERECE, a IDE FAZ: atalhos da paleta e itens de menu.

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

O MENU DA BARRA DE TITULO entrou aqui em 2026-09-04, no mesmo relato de uso
("quero acessar visualmente pelo mouse"), porque a falha e' da mesma familia:
o `AppMenuBar` declara `action: "X"` e o `ShellHeaderHost` trata `case "X"`.
Se os dois discordarem, clicar no item NAO FAZ NADA — sem erro, sem log, sem
nada na tela. Item de menu que nao faz nada e' pior que item ausente: o
ausente o autor procura em outro lugar; o morto ele repete.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Atalhos padrao do Qt: portateis por desenho, e por isso escritos como
# `StandardKey.X` em vez do texto da tecla. O mapa e' o que liga os dois
# mundos; so' entra aqui o que a UI realmente usa.
STANDARD_KEYS = {
    "StandardKey.Save": "Ctrl+S",
    "StandardKey.Open": "Ctrl+O",
}


def declared_commands():
    """id -> atalho, lidos dos descriptors do core."""
    declared = {}
    folder = ROOT / "crates/kinein-core/src/commands"
    for path in sorted(folder.glob("*.rs")):
        source = path.read_text()
        for block in re.finditer(
            r'id:\s*"([^"]+)"\.to_owned\(\),(.*?)requires_workspace', source, re.S
        ):
            shortcut = re.search(r'default_shortcut:\s*Some\("([^"]+)"', block.group(2))
            if shortcut:
                declared[block.group(1)] = shortcut.group(1)
    return declared


def ui_shortcuts():
    """Lista de (arquivo, linha, comando anotado ou None, sequencias)."""
    found = []
    for path in sorted((ROOT / "ui/qml").rglob("*.qml")):
        lines = path.read_text().split("\n")
        for index, line in enumerate(lines):
            if line.strip() != "Shortcut {":
                continue
            end = index
            while end < len(lines) and lines[end].strip() != "}":
                end += 1
            block = "\n".join(lines[index:end])
            annotated = re.search(r"//\s*comando:\s*([\w.]+)", block)
            listed = re.search(r'sequences?:\s*(\[[^\]]*\]|"[^"]*")', block)
            sequences = re.findall(r'"([^"]+)"', listed.group(1)) if listed else []
            for key, source in STANDARD_KEYS.items():
                if key in block:
                    sequences.append(source)
            found.append(
                (
                    path.relative_to(ROOT),
                    index + 1,
                    annotated.group(1) if annotated else None,
                    sequences,
                )
            )
    return found


def menu_and_handling():
    """(acoes declaradas no menu, acoes tratadas pelo host)."""
    host = (ROOT / "ui/qml/shell/ShellHeaderHost.qml").read_text()
    # Os itens do menu da barra de titulo vivem em `ui/qml/shell/AppMenu*.qml`.
    # O padrao de nome E' o acoplamento, e ele e' deliberado: em 2026-09-04 o
    # `AppMenuBar` foi partido e os itens foram para o `AppMenuItems`; um gate
    # preso a UM arquivo teria reprovado por causa de uma refatoracao legitima,
    # e gate que reprova a toa ensina a ser ignorado.
    #
    # Casar pela FORMA (`{ label:..., action:... }`) em todo o QML foi tentado e
    # e' pior: o painel de git tem menu proprio com a mesma forma e outro
    # despachante, e os itens dele apareceriam aqui como falha.
    paths = sorted((ROOT / "ui/qml/shell").glob("AppMenu*.qml"))
    offered = set()
    for path in paths:
        source = path.read_text()
        offered |= set(re.findall(r'action:\s*"([^"]+)"', source))
        # Desde 2026-10-03 o item nasce de `root.item(<rotulo>, "acao", ...)` (o
        # item ganhou icone e atalho; a forma literal virou funcao). O rotulo
        # pode ser condicional (`cargo ? qsTr("…") : qsTr("…")`).
        offered |= set(re.findall(r'root\.item\((?:[^()"]|"[^"]*"|\((?:[^()"]|"[^"]*")*\))*?,\s*"([^"]+)"\s*,', source))
    offered.discard("")
    if not offered:
        raise SystemExit(
            "verificar_atalhos: nenhum item de menu encontrado em "
            "ui/qml/shell/AppMenu*.qml — o padrao de nome mudou?"
        )
    handled = set(re.findall(r'case\s*"([^"]+)":', host))
    # Acao montada em tempo de execucao (`"workspace.recent.open:" + index`)
    # nao aparece como `case`: o host casa por PREFIXO, guardado num `const`.
    # O gate le' esses prefixos em vez de reclamar de um literal que nunca
    # existiu inteiro.
    prefixes = set(re.findall(r'const\s+\w*[Pp]refix\w*\s*=\s*"([^"]+)"', host))
    return offered, handled, prefixes


def main():
    declared = declared_commands()
    in_ui = ui_shortcuts()
    failures = []

    duplicates = {}
    for command, shortcut in declared.items():
        duplicates.setdefault(shortcut, []).append(command)
    for shortcut, commands in sorted(duplicates.items()):
        if len(commands) > 1:
            failures.append(
                f"atalho `{shortcut}` declarado por MAIS DE UM comando: "
                f"{', '.join(sorted(commands))}"
            )

    seen = {}
    for path, line, _, sequences in in_ui:
        for sequence in sequences:
            seen.setdefault(sequence, []).append(f"{path}:{line}")
    for sequence, places in sorted(seen.items()):
        if len(places) > 1:
            failures.append(
                f"sequencia `{sequence}` ligada em MAIS DE UM Shortcut: "
                f"{', '.join(places)}"
            )

    annotated_by_command = {}
    for path, line, command, sequences in in_ui:
        if command is None:
            continue
        annotated_by_command.setdefault(command, []).append((path, line, sequences))

    for command, shortcut in sorted(declared.items()):
        targets = annotated_by_command.get(command)
        if not targets:
            failures.append(
                f"`{command}` anuncia `{shortcut}` na paleta e NAO tem Shortcut "
                f"anotado na UI — apertar nao faz o que a paleta promete"
            )
            continue
        if len(targets) > 1:
            failures.append(f"`{command}` tem mais de um Shortcut anotado")
        path, line, sequences = targets[0]
        if shortcut not in sequences:
            failures.append(
                f"`{command}` anuncia `{shortcut}` mas o Shortcut de "
                f"{path}:{line} liga {sequences or '(nada)'}"
            )

    from_menu, from_host, prefixes = menu_and_handling()
    for action in sorted(from_menu):
        if action in from_host:
            continue
        if any(action.startswith(p) for p in prefixes):
            continue
        failures.append(
            f"o menu oferece `{action}` e o ShellHeaderHost nao trata: clicar "
            f"nao faz NADA, sem erro nenhum"
        )
    for action in sorted(from_host):
        if action not in from_menu:
            failures.append(
                f"o ShellHeaderHost trata `{action}` e nenhum item de menu o "
                f"oferece — codigo morto, ou item esquecido"
            )

    # O ATALHO QUE O MENU MOSTRA (2026-10-03): o item mostra a tecla do
    # catalogo; quando o comando tem outro nome (`commandOf`) ou a tecla so'
    # existe na UI (`uiShortcuts`), o mapa do AppMenuItems e' a costura — e
    # costura sem conferencia e' a mentira de 2026-09-04 de novo.
    items_source = (ROOT / "ui/qml/shell/AppMenuItems.qml").read_text()
    bound = {sequence for _, _, _, sequences in in_ui for sequence in sequences}
    for name in ("commandOf", "uiShortcuts"):
        mapping = re.search(r"property var " + name + r":\s*\(\{(.*?)\}\)", items_source, re.S)
        if not mapping:
            failures.append(f"AppMenuItems.qml sem o mapa `{name}` — o padrao mudou?")
            continue
        for action, value in re.findall(r'"([^"]+)":\s*"([^"]+)"', mapping.group(1)):
            if action not in from_menu:
                failures.append(f"`{name}` cita `{action}`, que nenhum item de menu oferece")
            if name == "commandOf" and value not in declared:
                failures.append(f"o menu mostra a tecla de `{value}` para `{action}`, e esse comando nao declara atalho")
            if name == "uiShortcuts" and value not in bound:
                failures.append(f"o menu anuncia `{value}` para `{action}`, e nenhum Shortcut liga essa tecla")

    for command, targets in sorted(annotated_by_command.items()):
        if command not in declared:
            path, line, _ = targets[0]
            failures.append(
                f"{path}:{line} diz implementar `{command}`, que nao "
                f"declara atalho nenhum no core"
            )

    if failures:
        print("✗ atalhos: a paleta promete o que a IDE nao faz", file=sys.stderr)
        for falha in failures:
            print(f"  {falha}", file=sys.stderr)
        print(
            "\n  A anotacao `// comando: <id>` acima do Shortcut e' o elo. "
            "Sem ela\n  nenhuma maquina sabe que aquele onActivated implementa "
            "aquele comando.",
            file=sys.stderr,
        )
        return 1

    print(
        f"atalhos: {len(declared)} comandos com atalho, todos ligados ao que "
        f"a paleta anuncia."
    )
    print(f"menu: {len(from_menu)} itens, todos com tratamento no host.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
