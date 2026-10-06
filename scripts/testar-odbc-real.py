#!/usr/bin/env python3
"""Prova ODBC real com driver SQLite ja instalado, sem download nem instalacao.

--driver aponta a biblioteca local. Cria somente DSN, banco e projeto
temporarios. --ui abre a IDE com HOME real e ODBC/XDG isolados; fechar limpa.
"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import tempfile
from datasource_rpc import Core, REPO


def check(condition, label):
    assert condition, label
    print("ok: " + label, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--driver", type=Path, required=True)
    parser.add_argument("--ui", action="store_true")
    parser.add_argument("--metadata", type=Path)
    args = parser.parse_args()
    driver = args.driver.resolve(strict=True)
    check(driver.is_file() and not any(c in str(driver) for c in "\r\n"), "biblioteca local valida")
    directory = Path(tempfile.mkdtemp(prefix="kinein-odbc-prova-"))
    env = os.environ.copy()
    for kind in ("CONFIG", "CACHE", "DATA", "STATE", "RUNTIME"):
        path = directory / ("xdg-" + kind.lower())
        path.mkdir(mode=0o700)
        env["XDG_" + kind + ("_DIR" if kind == "RUNTIME" else "_HOME")] = str(path)
    env["ODBCSYSINI"] = str(directory)
    env["ODBCINI"] = str(directory / "odbc.ini")
    database = directory / "prova.sqlite"
    (directory / "odbcinst.ini").write_text(
        f"[KV SQLite]\nDriver={driver}\n\n[KV Ausente]\nDriver={directory}/ausente.so\n")
    (directory / "odbc.ini").write_text(
        f"[KVProva]\nDriver=KV SQLite\nDatabase={database}\nTimeout=1000\n\n[KVAusente]\nDriver=KV Ausente\n")
    strange = 'x"; DROP TABLE leituras;--'
    with sqlite3.connect(database) as connection:
        connection.executescript("CREATE TABLE leituras(id INTEGER PRIMARY KEY, valor TEXT);"
                                 "INSERT INTO leituras VALUES(1,'primeira'),(2,NULL),(3,'terceira');")
        connection.execute('CREATE TABLE "' + strange.replace('"', '""') + '" (id INTEGER)')
        connection.execute('INSERT INTO "' + strange.replace('"', '""') + '" VALUES(42)')
        connection.execute("CREATE TABLE grande(valor TEXT)")
        connection.execute("INSERT INTO grande VALUES(?)", ("x" * (17 * 1024),))
    project = directory / "projeto"
    project.mkdir()
    (project / "pyproject.toml").write_text('[project]\nname="prova-odbc"\nversion="0.0.0"\n')
    core = None
    try:
        core = Core(env)
        check("error" not in core.rpc("workspace.open", {"path": str(project)}), "workspace temporario")
        listed = core.rpc("datasource.odbc.sources", {})["result"]["sources"]
        check({s["dsn"] for s in listed} == {"KVProva", "KVAusente"}, "lista inclui driver inexistente sem carrega-lo")
        check(all(set(s) == {"dsn", "driver", "identity"} for s in listed), "lista nao expoe atributos de conexao")
        profile = {"name": "ODBC-prova", "engine": "odbc", "host": "", "port": 0,
                   "database": "KVProva", "user": "", "secretSource": "automatic"}
        check("error" not in core.rpc("datasource.save", {"profile": profile}), "perfil ODBC salvo sem segredo")
        for method, extra in (("test", {}), ("introspect", {}), ("query", {"sql": "SELECT 1"})):
            refused = core.rpc("datasource." + method, {"name": profile["name"], **extra})
            check(refused.get("error", {}).get("code") == "DRIVER_APPROVAL_REQUIRED", method + " recusa antes do job")
        details = refused["error"]["details"]
        authorization = {k: details[k] for k in ("name", "identity", "workspace")}
        check("error" not in core.rpc("datasource.odbc.authorize", authorization), "gesto explicito autoriza sessao")
        check("error" not in core.rpc("datasource.test", {"name": profile["name"]}), "teste aceito apos gesto")
        check(core.event("tested")["ok"], "conexao real pelo driver SQLite")
        check("error" not in core.rpc("datasource.introspect", {"name": profile["name"]}), "catalogo aceito")
        structure = core.event("introspected")
        check(structure["ok"], "SQLTables/SQLColumns reais")
        tables = [table for schema in structure["schemas"] for table in schema["tables"]]
        table = next(t for t in tables if t["name"] == "leituras")
        check([c["name"] for c in table["columns"]] == ["id", "valor"], "colunas exatas do catalogo")
        strange_table = next(t for t in tables if t["name"] == strange)
        result = core.query(profile["name"], strange_table["readSql"])
        check(result["success"] and result["rows"] == [["42"]], "nome com aspas/ponto e virgula e somente identificador")
        result = core.query(profile["name"], "SELECT * FROM leituras ORDER BY id")
        check(result["success"] and result["rows"][1][1] is None, "NULL preservado na grade")
        response = core.rpc("datasource.query", {"name": profile["name"], "sql": "SELECT * FROM leituras", "maxRows": 1})
        check("error" not in response, "leitura limitada aceita")
        limited = core.event("queried")
        check(limited["success"] and limited["rowCount"] == 1 and limited["truncated"], "teto sinalizado sem LIMIT de outro dialeto")
        oversized = core.query(profile["name"], "SELECT * FROM grande")
        check(not oversized["success"] and "ODBC" in oversized["message"], "celula acima de 16 KiB recusada sem corte silencioso")
        sql = "DELETE FROM leituras WHERE id=3"
        refused = core.rpc("datasource.query", {"name": profile["name"], "sql": sql})
        check(refused.get("error", {}).get("code") == "WRITE_CONFIRMATION_REQUIRED", "escrita exige confirmacao generica")
        core.rpc("datasource.impact", {"name": profile["name"], "sql": sql})
        impact = core.event("impact")
        check(impact["severity"] == "destructive" and impact["statements"][0].get("rows") is None, "impacto local nao inventa contagem")
        written = core.query(profile["name"], sql, confirmed=True)
        check(written["success"] and written.get("affected") == 1, "escrita confirmada pelo driver real")
        core.close()
        check(not core.errors, "core encerra sem avisos no terminal")
        core = None
        # A IDE nova deve pedir o driver novamente: aprovacao nao foi salva.
        (project / ".kinein" / "consoles").mkdir(exist_ok=True)
        (project / ".kinein" / "consoles" / "ODBC-prova.sql").write_text(
            "SELECT * FROM leituras ORDER BY id;\n\nDELETE FROM leituras WHERE id=2;\n")
        if args.ui:
            env["QT_QPA_PLATFORM"] = "xcb"
            metadata = {"project": str(project), "directory": str(directory), "driver": str(driver), "cleaned": False}
            if args.metadata:
                args.metadata.write_text(json.dumps(metadata))
            print("IDE de prova: " + str(project), flush=True)
            with (directory / "ui.log").open("w") as log:
                ui = subprocess.Popen([str(REPO / "build/dev-local/ui/kinein-vectis"), str(project)], env=env, stdout=log, stderr=log)
                ui.wait()
            check((directory / "ui.log").stat().st_size == 0, "IDE encerra com terminal silencioso")
    finally:
        if core is not None:
            core.close()
        shutil.rmtree(directory)
        if args.metadata and args.metadata.exists():
            metadata = json.loads(args.metadata.read_text())
            metadata["cleaned"] = True
            args.metadata.write_text(json.dumps(metadata))


if __name__ == "__main__":
    main()
