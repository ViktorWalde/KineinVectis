#!/usr/bin/env bash
# Compatibilidade do QML com o Qt 6.4 do AppImage (Debian 12) — o que o Qt mais
# novo do checkout aceita e o pacote NAO (roadmap 53 §5.2.1, 2026-10-01).
#
# PROVADO POR CASO MINIMO no Qt 6.4.2: num arquivo com
# `pragma ComponentBehavior: Bound`, o ListView nunca cria `header`, `footer`,
# `highlight` nem `section.delegate`, e o Loader nunca cria `sourceComponent` —
# nem inline, nem por id do proprio arquivo. Cada tentativa e' um
# "QQmlComponent: Component is not ready" ou "Cannot instantiate bound
# component outside its creation context", e a parte some da tela (os
# cabecalhos de pasta do Git sumiam no pacote). O Qt 6.10 do checkout cria
# tudo, e por isso nenhum outro gate via.
#
# O que FUNCIONA nas duas versoes, e e' o que esta regra aceita: o Component
# nasce num arquivo SEM o pragma (um QtObject "Parts") e a chave recebe um
# caminho de membro (`root.parts.section`). `delegate:` de ListView/Repeater
# funciona em arquivo Bound e nao entra na regra.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "== QML compativel com o Qt 6.4 do AppImage =="
python3 - <<'PY'
import pathlib, re, sys

KEYS = re.compile(r"^\s*(header|footer|highlight|section\.delegate|sourceComponent)\s*:\s*(.*)$")
# Aceito: caminho de membro de outro objeto (a.b ou a.b.c), sem chave aberta.
MEMBER_PATH = re.compile(r"^[A-Za-z_]\w*(\.[A-Za-z_]\w*)+\s*(//.*)?$")

offenders = []
bound_files = 0
for f in sorted(pathlib.Path("ui/qml").rglob("*.qml")):
    text = f.read_text(encoding="utf-8")
    if "pragma ComponentBehavior: Bound" not in text:
        continue
    bound_files += 1
    for n, line in enumerate(text.splitlines(), 1):
        m = KEYS.match(line)
        if m and not MEMBER_PATH.match(m.group(2).strip()):
            offenders.append(f"{f}:{n}: {line.strip()}")

if offenders:
    print("erro: parte instanciada pelo Qt declarada num arquivo Bound — o Qt 6.4"
          " do AppImage nunca a cria:", file=sys.stderr)
    for r in offenders:
        print(f"  {r}", file=sys.stderr)
    print("  Mova o Component para um QtObject SEM o pragma (ver"
          " ui/qml/git/GitChangesListParts.qml) e aponte a chave para ele.",
          file=sys.stderr)
    raise SystemExit(1)
print(f"QML Qt 6.4: {bound_files} arquivos com pragma Bound, nenhuma parte que o 6.4 recusa.")
PY
