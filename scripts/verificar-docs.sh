#!/usr/bin/env bash
# Veracidade dos .md: numero afirmado sobre AGORA tem que bater com o disco.
#
# POR QUE ESTE SCRIPT EXISTE (2026-07-17). A §1.1 da ARCHITECTURE ja diz que a
# causa da volta do monolito foi mecanica: "regra que mora so em .md nao segura
# arquitetura; ela apodrece em silencio enquanto o gate fica verde". A catraca
# resolveu isso para o CODIGO. Os .md continuaram apodrecendo:
#
#   17-architecture-hygiene-plan.md  "Status: concluido para o estado atual"
#     afirmava ShellWorkspaceHost = 248 linhas   (real: 576, 2.3x)
#     afirmava EditorController   = 318 linhas   (real: 1070, 3.4x)
#
# E' o MESMO vicio que a §1.1 documenta ("a §6 ja afirmou 'Main.qml tem 336
# linhas' enquanto ele tinha 700") — sobrevivendo em outro arquivo, dez dias
# depois de a §1.1 ser escrita sobre ele.
#
# A REGRA, e ela e' o coracao deste script:
#
#   Numero COM data e' REGISTRO   -> "Medicao observada em 2026-07-04: 2967 linhas"
#                                    historico, nao se mede contra hoje.
#   Numero SEM data e' AFIRMACAO  -> "ShellWorkspaceHost tem 248 linhas"
#     SOBRE AGORA                    tem que ser verdade, ou e' mentira.
#
# Se voce precisa citar um numero antigo, DATE-O. Custa 4 palavras e mantem o
# registro honesto sem enganar quem le como se fosse o estado de hoje.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== veracidade dos .md (numero sem data = afirmacao sobre agora) =="

python3 - <<'PY'
import pathlib, re, subprocess, sys

# LOGS sao registro por natureza: citam o que era verdade na epoca.
LOGS = ("ContextoIA.md", "PLANO_ORGANIZACAO_E_HANDOFF.md")
DIRS_LOG = ("docs/diario/", "docs/adr/")

rastreados = [x for x in subprocess.run(
    ["git", "ls-files"], capture_output=True, text=True).stdout.split("\n") if x]

def achar(alvo):
    p = pathlib.Path(alvo)
    if p.exists():
        return p
    cand = [f for f in rastreados if f == alvo or f.endswith("/" + alvo)]
    return pathlib.Path(cand[0]) if len(cand) == 1 else None

def linhas_fora_de_teste(p):
    """Mesma contagem da catraca (§4 regra 10): o corte e' o `#[cfg(test)]`."""
    t = p.read_text()
    corte = t.find("#[cfg(test)]")
    if corte >= 0:
        t = t[:corte]
    return len(t.split("\n")) - 1

# O que marca um numero como REGISTRO e nao como afirmacao sobre agora.
DATADO = re.compile(
    r'\d{4}-\d{2}-\d{2}|medi[cç][aã]o observada|medido em|HIST[OÓ]RICO|SUPERADO'
    r'|FORA DE ESCOPO|era \d+|foi de \d+|\d+ -> \d+|\d+ → \d+', re.I)

RECLAMACAO = re.compile(
    r'`?([A-Za-z0-9_][A-Za-z0-9_./\-]*\.(?:qml|rs|cpp|h))`?[^\n]{0,40}?'
    r'\*{0,2}(\d{2,4})\*{0,2}\s*linhas')

achados = []
for p in sorted(pathlib.Path(".").rglob("*.md")):
    s = str(p)
    if "build/" in s or ".git/" in s or s in LOGS or s.startswith(DIRS_LOG):
        continue
    linhas = p.read_text().split("\n")
    # Uma tabela datada de medicoes data o BLOCO inteiro, nao so a 1a linha
    # ("Medicao observada em 2026-07-04:" vem ANTES da cerca). Sem isto o gate
    # acusa o meio do bloco e vira alarme falso — que ensina a ignorar.
    inicio_cerca = None
    cabecalho = ""
    for i, linha in enumerate(linhas):
        # Uma secao "## Resultado 2026-07-06" data TUDO que esta dentro dela.
        if linha.startswith("#"):
            cabecalho = linha
        if linha.strip().startswith("```"):
            inicio_cerca = i if inicio_cerca is None else None
            continue
        base = inicio_cerca if inicio_cerca is not None else i
        janela = cabecalho + "\n" + "\n".join(linhas[max(0, base - 4):i + 2])
        if DATADO.search(janela):
            continue
        for m in RECLAMACAO.finditer(linha):
            alvo, dito = m.group(1), int(m.group(2))
            f = achar(alvo)
            if f is None:
                continue
            real = linhas_fora_de_teste(f)
            if abs(real - dito) > 2:
                achados.append((s, i + 1, alvo, dito, real, linha.strip()[:70]))

for s, i, alvo, dito, real, ctx in achados:
    print(f"✗ {s}:{i}", file=sys.stderr)
    print(f"    afirma {alvo} = {dito} linhas; o disco tem {real}", file=sys.stderr)
    print(f"    “{ctx}”", file=sys.stderr)
    print(f"    Corrija o numero, ou DATE a frase se ela e' registro do passado.",
          file=sys.stderr)

if achados:
    print(f"\n✗ veracidade dos .md FALHOU ({len(achados)})", file=sys.stderr)
    sys.exit(1)
print("veracidade dos .md: nenhum numero sem data diverge do disco.")
PY
