#!/usr/bin/env python3
"""Lint estrito com o alvo JSON do Qt, inclusive Qt 6.4 (sem .rsp/-W).

O CMake continua dono da lista de fontes, imports, resources e do binario Qt.
O relatorio precisa ser novo e valido; warning reprova mesmo se uma versao
mais recente do qmllint retornar zero por tolerar avisos por padrao.

O QMLLINT QUE VALE E' O >= 6.5 (decisao do autor, 2026-10-01). O do Qt 6.4
acusa 6 falsos positivos em codigo correto (`StandardKey.Open`, `push` em
array JS, `Qt.callLater`, medido em 2026-10-01), e um veredito com falso
positivo conhecido nao prova nada nos dois sentidos: o achado pode ser ruido,
e o "limpo" do 6.4 nao e' o "limpo" do 6.5. Por isso, abaixo de 6.5, os
achados sao MOSTRADOS e o lint vira NAO PROVADO, limpo ou nao (o `--estrito` reprova). O
que continua reprovando em qualquer versao e' o relatorio ausente ou
invalido: isso e' a ferramenta quebrada, nao a versao.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys

from unproven import record

TRUSTED_QMLLINT = (6, 5)


def diagnostics(report: object) -> tuple[list[str], int]:
    """Valida o contrato do relatorio e conta arquivos com erro/aviso."""
    if not isinstance(report, dict) or not isinstance(report.get("files"), list):
        raise ValueError("relatorio sem lista de arquivos")
    if not report["files"]:
        raise ValueError("relatorio vazio: nenhum arquivo foi verificado")
    messages: list[str] = []
    failed = 0
    for item in report["files"]:
        if (not isinstance(item, dict) or not isinstance(item.get("filename"), str)
                or not item["filename"] or not isinstance(item.get("success"), bool)
                or not isinstance(item.get("warnings"), list)):
            raise ValueError("entrada de arquivo incompleta")
        bad = not item["success"]
        for warning in item["warnings"]:
            if not isinstance(warning, dict) or not isinstance(warning.get("message"), str):
                raise ValueError("diagnostico incompleto")
            severity = warning.get("type")
            if severity in ("info", "debug"):
                continue
            bad = True
            messages.append(f"{item['filename']}:{warning.get('line', 0)}:"
                            f"{warning.get('column', 0)}: {severity}: {warning['message']}")
        if bad:
            failed += 1
            messages.append(f"reprovado: {item['filename']}")
    return messages, failed


def qmllint_binary(build: pathlib.Path) -> pathlib.Path | None:
    """O qmllint que o alvo CMake chama: o mesmo Qt do build, nao o do PATH."""
    for script in [build / "build.ninja", *build.glob("**/CMakeFiles/*qmllint*/build.make")]:
        if not script.is_file():
            continue
        found = re.search(r"(\S*/qmllint)(?=\s)", script.read_text(encoding="utf-8", errors="replace"))
        if found:
            return pathlib.Path(found.group(1))
    return None


def parse_version(text: str) -> tuple[int, int] | None:
    """`qmllint 6.4.2` -> (6, 4). Sem versao reconhecivel, None."""
    found = re.search(r"\b(\d+)\.(\d+)(?:\.\d+)?\b", text)
    return (int(found.group(1)), int(found.group(2))) if found else None


def qmllint_version(build: pathlib.Path) -> tuple[int, int] | None:
    binary = qmllint_binary(build)
    if binary is None:
        return None
    try:
        output = subprocess.run([str(binary), "--version"], capture_output=True,
                                text=True, check=False, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return parse_version(output.stdout + output.stderr)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_dir", type=pathlib.Path)
    build = parser.parse_args().build_dir.resolve()
    report = build / "kinein-vectis_qmllint.json"
    log = build / "kinein-qmllint-build.log"
    print("== qmllint via CMake/JSON (estrito: zero warnings) ==", flush=True)
    try:
        report.unlink(missing_ok=True)
        with log.open("w", encoding="utf-8") as output:
            result = subprocess.run(
                ["cmake", "--build", str(build), "--target", "kinein-vectis_qmllint_json"],
                stdout=output, stderr=subprocess.STDOUT, check=False,
            )
        messages, failed = diagnostics(json.loads(report.read_text(encoding="utf-8")))
    except (OSError, ValueError) as error:
        print(f"erro: qmllint nao produziu relatorio valido: {error}", file=sys.stderr)
        print(f"log do build: {log}", file=sys.stderr)
        return 1
    for message in messages:
        print(message)
    version = qmllint_version(build)
    # Versao DESCONHECIDA nao ganha o desconto: sem saber qual qmllint rodou,
    # vale a regra estrita de sempre.
    if version is not None and version < TRUSTED_QMLLINT:
        label = ".".join(map(str, version))
        print(f"qmllint {label}: {failed} arquivo(s) com achado, acima. Abaixo do"
              f" {'.'.join(map(str, TRUSTED_QMLLINT))} o veredito nao vale, nem o"
              " limpo nem o sujo (falsos positivos conhecidos).")
        record("qml", f"qmllint estrito (Qt {label})",
               f"so' o qmllint >= {'.'.join(map(str, TRUSTED_QMLLINT))} vale; rode com um Qt"
               " >= 6.5 (KINEIN_QML_RSP aponta o .rsp dele)")
        return 0
    if result.returncode != 0 or failed:
        print(f"erro: QML reprovado ({failed} arquivos; CMake exit {result.returncode}).")
        print(f"log do build: {log}")
        return 1
    print("QML verificado: tudo limpo.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
