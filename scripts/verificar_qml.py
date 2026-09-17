#!/usr/bin/env python3
"""Lint estrito com o alvo JSON do Qt, inclusive Qt 6.4 (sem .rsp/-W).

O CMake continua dono da lista de fontes, imports, resources e do binario Qt.
O relatorio precisa ser novo e valido; warning reprova mesmo se uma versao
mais recente do qmllint retornar zero por tolerar avisos por padrao.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys


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
    if result.returncode != 0 or failed:
        print(f"erro: QML reprovado ({failed} arquivos; CMake exit {result.returncode}).")
        print(f"log do build: {log}")
        return 1
    print("QML verificado: tudo limpo.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
