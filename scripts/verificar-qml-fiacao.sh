#!/usr/bin/env bash
# Fiacao QML: pega o binding AUTO-REFERENTE `x: x`.
#
# POR QUE ESTE SCRIPT EXISTE (2026-07-17). O commit 0686213 (AppDomains) escreveu
# `coreClient: coreClient` em 15 roteadores. Parece obvio e esta ERRADO:
#
#   Regra do QML, MEDIDA (Qt 6.11.1), nao lembrada:
#     `x: x` resolve para o `id: x` do MESMO arquivo, se existir  -> certo.
#     Sem esse id, resolve para a PROPRIEDADE DO PROPRIO ALVO     -> auto-
#     referencia. A propriedade do objeto RAIZ do arquivo PERDE.
#
# Resultado: `coreClient` chegou `null` em todos os roteadores, as `Connections`
# ficaram inertes e a IDE parou de ler pastas e de criar projetos. O build passou,
# o qmllint disse "limpo", o boot foi ate o primeiro frame sem um aviso, e o Qt
# nao reclama de binding loop. Falha 100% silenciosa — a unica classe de bug que
# nenhum gate deste projeto pegava.
#
# A regra: dentro de um arquivo, `x: x` so e' correto se existir `id: x` nele.
# Se `x` e' propriedade do root, escreva `x: root.x`.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== fiacao QML (binding auto-referente) =="

python3 - "$@" <<'PY'
import pathlib, re, sys

ABRE_COMPONENTE = re.compile(r'^\s*[A-Z][A-Za-z0-9_.]*\s*\{\s*$')
achados = []

def eh_binding_qml(linhas, idx):
    """True se a linha idx e' um binding de propriedade QML (e nao um literal JS).

    O bloco que a contem decide: componente (`Foo {`) e' QML; `function`, arrow
    ou `({` e' JavaScript, onde `x: x` e' um literal de objeto valido e comum.
    """
    recuo = len(linhas[idx]) - len(linhas[idx].lstrip())
    for j in range(idx - 1, -1, -1):
        linha = linhas[j]
        if not linha.strip():
            continue
        r = len(linha) - len(linha.lstrip())
        if r < recuo and linha.rstrip().endswith(("{", "({", "=> {")):
            return bool(ABRE_COMPONENTE.match(linha))
    return False

for base in ("ui/qml", "scripts/qml-harness"):
    for f in sorted(pathlib.Path(base).rglob("*.qml")):
        texto = f.read_text()
        linhas = texto.split("\n")
        ids = set(re.findall(r'^\s*id:\s*([A-Za-z_][A-Za-z0-9_]*)\s*$', texto, re.M))
        for i, linha in enumerate(linhas):
            m = re.match(r'^\s+([a-z_][A-Za-z0-9_]*):\s*\1\s*$', linha)
            if not m or m.group(1) in ids:
                continue
            if eh_binding_qml(linhas, i):
                achados.append((str(f), i + 1, m.group(1)))

for arquivo, linha, nome in achados:
    print(f"✗ auto-referencia: {arquivo}:{linha}", file=sys.stderr)
    print(f"    `{nome}: {nome}` — nao ha `id: {nome}` neste arquivo, entao o lado", file=sys.stderr)
    print(f"    direito resolve para a PROPRIA propriedade do alvo: chega null.", file=sys.stderr)
    print(f"    Escreva `{nome}: root.{nome}` (ou o id certo).", file=sys.stderr)

if achados:
    print(f"\n✗ fiacao QML FALHOU ({len(achados)})", file=sys.stderr)
    sys.exit(1)
print("fiacao QML: nenhum binding auto-referente.")
PY
