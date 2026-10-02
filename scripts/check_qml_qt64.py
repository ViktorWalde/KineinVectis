#!/usr/bin/env python3
"""G0.2: QML que o Qt 6.4 do AppImage nunca cria num arquivo Bound.

O MECANISMO (roadmap 53 §5.2.1, provado por caso minimo no Qt 6.4.2). Num
arquivo com `pragma ComponentBehavior: Bound`, o ListView nunca cria `header`,
`footer`, `highlight` nem `section.delegate`, e o Loader nunca cria
`sourceComponent`: o Qt instancia essas partes FORA do contexto em que o
componente nasceu, e o 6.4 recusa — "Cannot instantiate bound component outside
its creation context" (a mensagem esta' na propria libQt6Qml 6.4.2), seguida de
"Component is not ready". A parte some da tela. O Qt 6.10 cria tudo.

O que funciona nas duas versoes, e e' o que esta regra aceita: o Component nasce
num arquivo SEM o pragma (um QtObject "Parts") e a chave recebe um caminho de
membro (`root.parts.section`). `delegate:` de ListView/Repeater funciona em
arquivo Bound e nao entra na regra.

POR QUE UM SCANNER, E NAO UMA REGEX POR LINHA (2026-10-01). A regex via
`section.delegate: Text {}` e nao via a MESMA propriedade na sintaxe agrupada,
`section { delegate: Text {} }` — provado por mutacao: a agrupada passava. O
scanner acompanha os blocos `{ }` e sabe, em cada chave, se esta' num corpo de
objeto (`Item {`), numa propriedade agrupada (`section {`) ou em JavaScript
(`onClicked: {`, `function f() {`), onde `header:` nao e' binding.

Os casos de mutacao abaixo (SELF_TEST) rodam a cada execucao, antes da arvore:
um scanner que deixe de pegar qualquer um deles reprova o proprio gate.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
QML_DIR = ROOT / "ui" / "qml"

# As partes que o Qt instancia fora do contexto de criacao.
INSTANTIATED_KEYS = {"header", "footer", "highlight", "section.delegate", "sourceComponent"}
# Aceito: caminho de membro de outro objeto (a.b ou a.b.c), nada alem disso.
MEMBER_PATH = re.compile(r"^[A-Za-z_]\w*(\.[A-Za-z_]\w*)+$")
BOUND_PRAGMA = re.compile(r"^\s*pragma\s+ComponentBehavior\s*:\s*Bound\b", re.MULTILINE)
BINDING = re.compile(r"^([A-Za-z_][\w.]*)\s*:\s*(.*)$", re.DOTALL)
TYPE_PATH = re.compile(r"^(?:[A-Za-z_][\w.]*\s*:\s*)?([A-Za-z_][\w.]*)$")
INLINE_COMPONENT = re.compile(r"^component\s+\w+\s*:\s*[A-Z][\w.]*$")
VALUE_SOURCE = re.compile(r"^[A-Z][\w.]*\s+on\s+[\w.]+$")
JS_WORDS = {"if", "else", "for", "while", "do", "switch", "try", "catch", "finally", "function",
            "return", "case", "default", "with"}

OBJECT, JAVASCRIPT = "object", "js"


def strip_comments_and_strings(text: str) -> str:
    """Troca comentarios e literais por espacos, preservando linhas e chaves de codigo."""
    out: list[str] = []
    i, size = 0, len(text)
    while i < size:
        char = text[i]
        pair = text[i:i + 2]
        if pair == "//":
            end = text.find("\n", i)
            i = size if end < 0 else end
        elif pair == "/*":
            end = text.find("*/", i + 2)
            chunk = text[i:(size if end < 0 else end + 2)]
            out.append("\n" * chunk.count("\n"))
            i = size if end < 0 else end + 2
        elif char in "\"'`":
            j = i + 1
            while j < size and text[j] != char:
                j += 2 if text[j] == "\\" else 1
            chunk = text[i:j + 1]
            out.append('""' + "\n" * chunk.count("\n"))
            i = j + 1
        else:
            out.append(char)
            i += 1
    return "".join(out)


def classify_block(prefix: str, frame: str) -> str:
    """Que tipo de bloco o `{` abre, pelo texto da instrucao que o precede."""
    if frame == JAVASCRIPT:
        return JAVASCRIPT
    # Lista de objetos (`states: [ State {`, `data: [ A {}, B {`): vale o ultimo item.
    if "[" in prefix or "," in prefix:
        prefix = re.split(r"[\[,]", prefix)[-1].strip()
    if re.fullmatch(r"[a-z_]\w*", prefix) and prefix not in JS_WORDS:
        return "group:" + prefix
    match = TYPE_PATH.match(prefix)
    if match and match.group(1).split(".")[-1][:1].isupper():
        return OBJECT
    if INLINE_COMPONENT.match(prefix) or VALUE_SOURCE.match(prefix):
        return OBJECT
    return JAVASCRIPT


def offending_bindings(text: str) -> list[tuple[int, str]]:
    """(linha, chave) de cada parte instanciada cujo valor nao e' caminho de membro."""
    code = strip_comments_and_strings(text)
    stack = [OBJECT]
    found: list[tuple[int, str]] = []
    statement: list[str] = []
    line = statement_line = 1

    def check(statement_text: str, frame: str) -> None:
        if frame == JAVASCRIPT:
            return
        match = BINDING.match(statement_text)
        if not match:
            return
        key = match.group(1)
        if frame.startswith("group:"):
            key = frame.removeprefix("group:") + "." + key
        if key in INSTANTIATED_KEYS and not MEMBER_PATH.match(match.group(2).strip()):
            found.append((statement_line, key))

    for char in code:
        if char in "{};\n":
            current = "".join(statement).strip()
            frame = stack[-1]
            if char == "{":
                check(current, frame)
                stack.append(classify_block(current, frame))
            else:
                check(current, frame)
                if char == "}" and len(stack) > 1:
                    stack.pop()
            statement = []
            if char == "\n":
                line += 1
            statement_line = line
        else:
            if not statement and char.isspace():
                continue
            if not statement:
                statement_line = line
            statement.append(char)
    check("".join(statement).strip(), stack[-1])
    return found


# (nome, QML, chaves esperadas). Cada um ja' foi um furo ou e' o vizinho dele.
SELF_TEST: list[tuple[str, str, list[str]]] = [
    ("pontuada", "ListView {\n    section.delegate: Text {}\n}\n", ["section.delegate"]),
    ("agrupada", "ListView {\n    section {\n        property: \"a\"\n        delegate: Text {}\n    }\n}\n",
     ["section.delegate"]),
    ("agrupada numa linha", "ListView { section { property: \"a\"; delegate: Text {} } }\n",
     ["section.delegate"]),
    ("valor na linha seguinte", "ListView {\n    header:\n        Item {}\n}\n", ["header"]),
    ("Loader inline", "Loader {\n    sourceComponent: Component { Item {} }\n}\n", ["sourceComponent"]),
    ("dentro de lista", "Item {\n    data: [\n        ListView { footer: Item {} }\n    ]\n}\n", ["footer"]),
    ("caminho de membro", "ListView {\n    header: root.parts.header\n    section.delegate: parts.section\n}\n",
     []),
    ("agrupada com caminho", "ListView {\n    section { delegate: root.parts.section }\n}\n", []),
    ("delegate comum", "ListView {\n    delegate: Item {}\n}\n", []),
    ("javascript nao e' binding", "Item {\n    function f() {\n        const o = { header: 1 }\n    }\n"
     "    onX: { header: 2 }\n}\n", []),
    ("comentario e string", "ListView {\n    // header: Item {}\n    property string s: \"header: Item {}\"\n}\n", []),
    ("outro grupo", "Text {\n    font { bold: true; family: \"x\" }\n    anchors { fill: parent }\n}\n", []),
]


def run_self_test() -> list[str]:
    errors = []
    for name, qml, expected in SELF_TEST:
        got = [key for _, key in offending_bindings(qml)]
        if got != expected:
            errors.append(f"caso '{name}': esperado {expected}, o scanner deu {got}")
    return errors


def main() -> int:
    print("== QML compativel com o Qt 6.4 do AppImage ==")
    self_errors = run_self_test()
    if self_errors:
        print("erro: o proprio scanner do G0.2 regrediu (casos de mutacao em check_qml_qt64.py):",
              file=sys.stderr)
        for error in self_errors:
            print(f"  {error}", file=sys.stderr)
        return 1

    offenders = []
    bound_files = 0
    for path in sorted(QML_DIR.rglob("*.qml")):
        text = path.read_text(encoding="utf-8")
        if not BOUND_PRAGMA.search(strip_comments_and_strings(text)):
            continue
        bound_files += 1
        lines = text.splitlines()
        for number, key in offending_bindings(text):
            source = lines[number - 1].strip() if number <= len(lines) else ""
            offenders.append(f"{path.relative_to(ROOT)}:{number}: [{key}] {source}")

    if offenders:
        print("erro: parte instanciada pelo Qt declarada num arquivo Bound — o Qt 6.4"
              " do AppImage nunca a cria:", file=sys.stderr)
        for offender in offenders:
            print(f"  {offender}", file=sys.stderr)
        print("  Mova o Component para um QtObject SEM o pragma (ver"
              " ui/qml/git/GitChangesListParts.qml) e aponte a chave para ele.",
              file=sys.stderr)
        return 1
    print(f"QML Qt 6.4: {bound_files} arquivos com pragma Bound, nenhuma parte que o 6.4 recusa"
          f" ({len(SELF_TEST)} casos de mutacao do scanner verdes).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
