"""Protocolo NAO PROVADO dos gates (2026-10-01). Par do scripts/unproven.sh.

O PROBLEMA. Um gate que verifica a integracao com o AMBIENTE (QEMU, debugpy,
kit cross, Qt 6.4 em container) nao tem o que provar numa maquina sem a
ferramenta. Ate' esta data ele imprimia "nao reprova; o ciclo fica NAO PROVADO
aqui" e saia 0 — e o verificar.sh, que so' ve' codigos de saida, terminava em
"✓ TUDO VERDE". Medido num Ubuntu 24.04 limpo: tres ciclos (embarcado,
depuracao Python, clangd-cross) passaram em 0 s, sem rodar, e o resumo dizia
tudo verde.

O CONTRATO. Ausencia de ferramenta de AMBIENTE nao e' defeito do codigo, e
reprovar seria falso positivo para quem acabou de clonar. Mas tambem nao e'
verde. O gate chama `record()`: a linha sai no stdout com o prefixo fixo
"NAO PROVADO" e, quando o orquestrador exporta KINEIN_UNPROVEN_FILE, vai para
esse arquivo (gate<TAB>o que<TAB>por que). O verificar.sh lista tudo no fim,
nao diz "TUDO VERDE" se a lista nao estiver vazia, e com --estrito reprova.

Ferramenta de que o gate do REPOSITORIO depende (clang-tidy, shellcheck,
cargo-deny, qmllint) NAO usa este protocolo: falta dela reprova, com o comando
que instala.
"""

from __future__ import annotations

import os

ENV_VAR = "KINEIN_UNPROVEN_FILE"


def _clean(text: str) -> str:
    return " ".join(text.split())


def record(gate: str, what: str, why: str) -> None:
    """Diz, e registra para o orquestrador, o que este gate NAO provou aqui."""
    print(f"  - NAO PROVADO: {_clean(what)} ({_clean(why)})", flush=True)
    path = os.environ.get(ENV_VAR)
    if path:
        with open(path, "a", encoding="utf-8") as sink:
            sink.write(f"{_clean(gate)}\t{_clean(what)}\t{_clean(why)}\n")
