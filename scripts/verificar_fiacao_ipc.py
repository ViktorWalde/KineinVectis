#!/usr/bin/env python3
"""Fiacao IPC de ponta a ponta: o que existe numa camada e nenhuma outra liga.

POR QUE EXISTE (2026-09-18, pente-fino). A varredura de 2026-09-10
(`DocsPublic/roadmaps/40` §8) achou cinco vezes a mesma classe de falha — o
core calcula, a ponte transporta, nenhuma tela mostra — e a §8.4 disse que
essa classe nao tinha gate: "falta o andar de cima". Este e' o andar de cima.

Quatro perguntas, cada uma com a lista de excecoes DITAS (com o motivo):

  1. metodo roteado no core que nenhum cliente (C++ ou CLI) pede
  2. evento `event.*` que o core emite e o C++ nao trata (ou trata e descarta)
  3. sinal do CoreClient (fora NOTIFY de Q_PROPERTY) que nenhum QML/C++ escuta
  4. sinal QML declarado que ninguem trata (`onX`)

Nao e' catraca: nao ha baseline. O que sobra fora das excecoes reprova, e a
excecao so entra com motivo escrito aqui.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent

# 1. Metodos que existem para OUTRO cliente ou para o futuro dito.
METODOS_SEM_CLIENTE_ACEITOS = {
    "job.list": "recuperacao apos crash da UI e a CLI: a lista de jobs vivos (40 §8.2)",
}
# 2. Eventos que o C++ recebe e descarta DE PROPOSITO.
EVENTOS_DESCARTADOS_ACEITOS: dict[str, str] = {}
# 3. Sinais do CoreClient sem ouvinte, com o motivo.
SINAIS_SEM_OUVINTE_ACEITOS = {
    "containerActionAccepted": "o jobId do aceite; o desfecho chega por event.job.* (JobsController)",
    "dataSourceTestAccepted": "idem: o veredito vem por event.datasource.tested",
    "grafanaProbeAccepted": "idem: event.grafana.probed",
    "pythonEnvironmentAccepted": "idem: event.python.finished",
    "pythonStubsAccepted": "idem: event.python.stubs",
    "environmentScanStarted": "a tela usa a Q_PROPERTY scanningEnvironment; o total e' informativo",
    "environmentScanFinished": "idem; a lista chega por toolsListed",
}
# 4. Sinais QML sem tratador, com o motivo.
SINAIS_QML_SEM_TRATADOR_ACEITOS: dict[str, str] = {}


def ler(caminho: Path) -> str:
    return caminho.read_text(encoding="utf-8", errors="replace")


def arquivos(pasta: str, *sufixos: str) -> list[Path]:
    return [p for p in (RAIZ / pasta).rglob("*") if p.suffix in sufixos]


def metodos_do_core() -> set[str]:
    """Os bracos `"dominio.metodo" => ...` dos roteadores (`handlers/`, `lib.rs`).

    So' linhas que chamam um `*_response(`/`Continue(`: o mesmo padrao de
    string aparece em ids de configAction e em nomes de arquivo (`boot.py`),
    que nao sao metodos.
    """
    padrao = re.compile(r'"([a-z][A-Za-z]*(?:\.[a-zA-Z]+)+)"')
    achados: set[str] = set()
    fontes = arquivos("crates/kinein-core/src/handlers", ".rs") + [
        RAIZ / "crates/kinein-core/src/lib.rs",
        RAIZ / "crates/kinein-core/src/handlers.rs",
    ]
    braco = re.compile(
        r'((?:"[a-zA-Z.]+"\s*\|\s*)*"[a-zA-Z.]+")\s*=>\s*(?:Some\(|\{|RequestOutcome|self\.|Self::)'
    )
    for p in fontes:
        # Um braco pode quebrar linha entre os `|` (debug.continue | … | debug.stop).
        texto = re.sub(r"\s*\n\s*\|", " |", ler(p))
        for m in braco.finditer(texto):
            for nome in padrao.findall(m.group(1)):
                if not nome.startswith("event."):
                    achados.add(nome)
    return achados


def metodos_dos_clientes() -> set[str]:
    achados: set[str] = set()
    padrao = re.compile(r'"([a-z][A-Za-z]*(?:\.[a-zA-Z]+)+)"')
    for p in arquivos("ui/src", ".cpp", ".h") + arquivos("crates/kinein-cli/src", ".rs"):
        achados.update(m.group(1) for m in padrao.finditer(ler(p)))
    return achados


def eventos_do_core() -> set[str]:
    achados: set[str] = set()
    for p in arquivos("crates/kinein-core/src", ".rs"):
        if "/tests/" in str(p):
            continue
        texto = ler(p)
        achados.update(re.findall(r'"(event\.[a-zA-Z.]+)"', texto))
        # `format!("event.{domain}.started")` em `emit_build_event`: os
        # dominios sao os literais passados nas chamadas dela.
        dominios = set(re.findall(r'emit_build_event\(\s*ctx,\s*"([a-z]+)"', texto))
        for sufixo in re.findall(r'format!\("event\.\{domain\}\.([a-zA-Z]+)"', texto):
            for dominio in dominios or {"build", "quality"}:
                achados.add(f"event.{dominio}.{sufixo}")
    return achados


def eventos_tratados_no_cpp() -> tuple[set[str], set[str]]:
    """(tratados, descartados): descartado = `if (method == "event.x") { return true; }` sem emit."""
    tratados: set[str] = set()
    descartados: set[str] = set()
    padrao = re.compile(r'method == QStringLiteral\("(event\.[a-zA-Z.]+)"\)\)\s*\{(.*?)\n    \}', re.S)
    for p in arquivos("ui/src", ".cpp"):
        texto = ler(p)
        tratados.update(re.findall(r'QStringLiteral\("(event\.[a-zA-Z.]+)"\)', texto))
        for m in padrao.finditer(texto):
            corpo = m.group(2)
            if "emit " not in corpo and "handle" not in corpo and "set" not in corpo and "append" not in corpo:
                descartados.add(m.group(1))
    return tratados, descartados


def sinais_do_coreclient() -> tuple[set[str], set[str]]:
    texto = ler(RAIZ / "ui/src/core_client.h")
    notify = set(re.findall(r"NOTIFY ([a-zA-Z]+)", texto))
    sinais: set[str] = set()
    em_signals = False
    for linha in texto.splitlines():
        if re.match(r"^signals:", linha):
            em_signals = True
            continue
        if re.match(r"^(public|private|protected)( slots)?:", linha):
            em_signals = False
            continue
        if em_signals:
            m = re.search(r"void ([a-zA-Z]+)\(", linha)
            if m:
                sinais.add(m.group(1))
    return sinais, notify


def texto_qml_e_cpp() -> str:
    partes = [ler(p) for p in arquivos("ui/qml", ".qml") + arquivos("ui/src", ".cpp", ".h")]
    return "\n".join(partes)


def sinais_qml() -> list[tuple[str, str, Path]]:
    achados = []
    for p in arquivos("ui/qml", ".qml"):
        for m in re.finditer(r"^\s*signal ([a-zA-Z]+)", ler(p), re.M):
            achados.append((p.stem, m.group(1), p))
    return achados


def main() -> int:
    print("== fiacao IPC de ponta a ponta (metodo, evento, sinal C++, sinal QML) ==")
    falhas: list[str] = []

    # 1
    orfaos = sorted(metodos_do_core() - metodos_dos_clientes())
    for m in orfaos:
        if m in METODOS_SEM_CLIENTE_ACEITOS:
            continue
        falhas.append(f"metodo roteado no core sem cliente: {m}")

    # 2
    tratados, descartados = eventos_tratados_no_cpp()
    for e in sorted(eventos_do_core() - tratados):
        falhas.append(f"evento do core que o C++ nao trata: {e}")
    for e in sorted(descartados):
        if e not in EVENTOS_DESCARTADOS_ACEITOS:
            falhas.append(f"evento tratado no C++ e DESCARTADO (sem emit): {e}")

    # 3
    sinais, notify = sinais_do_coreclient()
    corpo = texto_qml_e_cpp()
    for sig in sorted(sinais - notify):
        cap = sig[0].upper() + sig[1:]
        ouvido = re.search(rf"\bon{cap}\b", corpo) or re.search(rf"&CoreClient::{sig}\b", corpo)
        if not ouvido and sig not in SINAIS_SEM_OUVINTE_ACEITOS:
            falhas.append(f"sinal do CoreClient sem ouvinte: {sig}")

    # 4
    qml = "\n".join(ler(p) for p in arquivos("ui/qml", ".qml"))
    for comp, sig, _ in sinais_qml():
        cap = sig[0].upper() + sig[1:]
        chave = f"{comp}.{sig}"
        if not re.search(rf"\bon{cap}\b", qml) and chave not in SINAIS_QML_SEM_TRATADOR_ACEITOS:
            falhas.append(f"sinal QML sem tratador: {chave}")

    if falhas:
        print("fiacao IPC FALHOU:")
        for f in falhas:
            print(f"  ✗ {f}")
        print("\nDe um dono ao que sobrou, ou escreva a excecao COM MOTIVO em scripts/verificar_fiacao_ipc.py.")
        return 1
    print(
        f"fiacao IPC: {len(metodos_do_core())} metodos, {len(eventos_do_core())} eventos, "
        f"{len(sinais)} sinais C++ e {len(sinais_qml())} sinais QML — todos com dono "
        f"({len(METODOS_SEM_CLIENTE_ACEITOS) + len(SINAIS_SEM_OUVINTE_ACEITOS)} excecoes ditas)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
