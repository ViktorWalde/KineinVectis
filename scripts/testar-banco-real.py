#!/usr/bin/env python3
"""Prova datasource.* contra PostgreSQL e MongoDB reais, por JSON-RPC.

Usa imagens ja' disponiveis no Podman (nunca baixa), portas no loopback e
containers temporarios. Senha gerada vive so' em memoria. Com --ui, abre a
IDE no projeto de teste com HOME real e XDG isolado; fechar a IDE limpa tudo.
"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import tempfile
import time
from datasource_rpc import Core

REPO = Path(__file__).resolve().parent.parent

def command(args, **options):
    return subprocess.run(args, check=True, text=True, capture_output=True, **options).stdout.strip()


def check(ok, description):
    assert ok, description
    print("ok: " + description, flush=True)

def ready(container, engine):
    deadline = time.monotonic() + 60
    args = ["pg_isready", "-h", "127.0.0.1", "-U", "postgres"] if engine == "pg" else [
        "mongosh", "--quiet", "--eval", "db.adminCommand({ping:1}).ok"]
    while time.monotonic() < deadline:
        result = subprocess.run(["podman", "exec", container, *args],
                                capture_output=True, text=True)
        if result.returncode == 0:
            return
        time.sleep(0.5)
    raise AssertionError("servidor de teste nao ficou pronto")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ui", action="store_true")
    parser.add_argument("--metadata", type=Path)
    args = parser.parse_args()
    directory = Path(tempfile.mkdtemp(prefix="kinein-banco-prova-"))
    containers = []
    core = None
    password = secrets.token_hex(24)
    env = os.environ.copy()
    for kind in ("CONFIG", "CACHE", "DATA", "STATE", "RUNTIME"):
        path = directory / ("xdg-" + kind.lower())
        path.mkdir(mode=0o700)
        env["XDG_" + kind + ("_DIR" if kind == "RUNTIME" else "_HOME")] = str(path)
    env["KINEIN_TEST_PG_PASSWORD"] = password
    project = directory / "projeto"
    project.mkdir()
    (project / "pyproject.toml").write_text('[project]\nname = "prova-banco"\nversion = "0.0.0"\n')
    (project / "main.py").write_text('print("Projeto temporario da prova do Banco")\n')
    ui_logs = directory / "ui.log"
    try:
        ports = {}
        for engine, image, internal in (
            ("pg", "docker.io/library/postgres:16-alpine", 5432),
            ("mongo", "docker.io/library/mongo:7", 27017)):
            name = directory.name + "-" + engine
            launch_env = os.environ.copy()
            extra = []
            if engine == "pg":
                launch_env["POSTGRES_PASSWORD"] = password
                extra = ["-e", "POSTGRES_PASSWORD"]
            command(["podman", "run", "--rm", "--pull=never", "-d", "--name", name,
                     "--label", "io.kinein.prova=banco",
                     "-p", f"127.0.0.1::{internal}", *extra, image], env=launch_env)
            containers.append(name)
            ready(name, engine)
            ports[engine] = int(command(["podman", "port", name, str(internal)]).rsplit(":", 1)[1])
        core = Core(env)
        check("error" not in core.rpc("workspace.open", {"path": str(project)}), "workspace temporario")
        pg = {"name": "PostgreSQL-prova", "engine": "postgres", "host": "127.0.0.1",
              "port": ports["pg"], "database": "postgres", "user": "postgres",
              "secretSource": "prompt"}
        mongo = {"name": "MongoDB-prova", "engine": "mongo", "host": "127.0.0.1",
                 "port": ports["mongo"], "database": "kinein_prova", "user": ""}
        for profile in (pg, mongo):
            check("error" not in core.rpc("datasource.save", {"profile": profile}),
                  "perfil " + profile["name"])
        missing = core.rpc("datasource.query", {"name": pg["name"], "sql": "SELECT 1"})
        check(missing.get("error", {}).get("code") == "SECRET_REQUIRED", "senha pedida por codigo")
        bad = core.query(pg["name"], "SELECT 1", password="senha-incorreta")
        check(not bad["success"] and bad["secretRequired"], "senha incorreta recusada")
        for sql in [
            "CREATE SCHEMA kinein_prova",
            "CREATE TABLE kinein_prova.leituras (id int PRIMARY KEY, valor int)",
            "INSERT INTO kinein_prova.leituras VALUES (1,10),(2,20),(3,30)",
            "UPDATE kinein_prova.leituras SET valor=11 WHERE id=1"]:
            check(core.query(pg["name"], sql, password)["success"], "PostgreSQL: " + sql.split()[0])
        whole = "UPDATE kinein_prova.leituras SET valor=99 WHERE id>0"
        blocked = core.query(pg["name"], whole, password)
        check(blocked.get("confirmationSql") == whole, "PostgreSQL: filtro que pega todos nao escreve")
        unchanged = core.query(pg["name"], "SELECT valor FROM kinein_prova.leituras ORDER BY id", password)
        check(unchanged["rows"] == [["11"], ["20"], ["30"]], "PostgreSQL: dados intactos apos recusa")
        for name, sql in [(pg["name"], "DROP SCHEMA kinein_prova CASCADE"),
                          (mongo["name"], 'sensores.deleteOne({"placa":"esp32"})')]:
            response = core.rpc("datasource.query", {"name": name, "sql": sql})
            check(response.get("error", {}).get("code") == "WRITE_CONFIRMATION_REQUIRED",
                  name + ": remocao recusada antes do job")
        mixed = core.rpc("datasource.query", {"name": pg["name"],
                         "sql": "SHOW server_version; COMMIT; DROP SCHEMA kinein_prova CASCADE"})
        check(mixed.get("error", {}).get("code") == "WRITE_CONFIRMATION_REQUIRED",
              "PostgreSQL: leitura inicial nao esconde remocao no lote")
        response = core.rpc("datasource.impact", {"name": pg["name"],
                            "password": password, "sql": "DROP SCHEMA kinein_prova CASCADE"})
        check("error" not in response, "PostgreSQL: medicao aceita")
        measured = core.event("impact")
        check(measured["statements"][0]["rows"] == 1, "DROP SCHEMA conta a tabela")
        inserted = core.query(mongo["name"],
            'sensores.insertMany([{"placa":"esp32","v":1},{"placa":"pico","v":2},{"placa":"esp32","v":3}])')
        check(inserted.get("affected") == 3, "MongoDB: inserir documentos")
        updated = core.query(mongo["name"], 'sensores.updateMany({"placa":"esp32"},{"$inc":{"v":10}})')
        check(updated.get("affected") == 2, "MongoDB: alterar com filtro")
        whole_mongo = 'sensores.updateMany({"placa":{"$exists":true}},{"$set":{"v":99}})'
        check(core.query(mongo["name"], whole_mongo).get("confirmationSql") == whole_mongo,
              "MongoDB: filtro que pega todos nao escreve")
        core.rpc("datasource.impact", {"name": mongo["name"],
                                      "sql": 'sensores.deleteMany({"placa":"esp32"})'})
        measured = core.event("impact")
        check(measured["statements"][0]["rows"] == 2 and measured["statements"][0]["totalRows"] == 3,
              "MongoDB: contagem exata do mesmo filtro")
        deleted = core.query(mongo["name"], 'sensores.deleteOne({"placa":"esp32"})', confirmed=True)
        check(deleted.get("affected") == 1, "MongoDB: apagar documento confirmado")
        check(core.query(mongo["name"], "sensores.find({})")["rowCount"] == 2, "MongoDB: ler depois da escrita")
        check(core.query(mongo["name"], "sensores.drop()", confirmed=True).get("affected") == 2,
              "MongoDB: remover colecao confirmado")
        check(core.query(mongo["name"], 'telemetria.sensores.insertOne({"v":1})')["success"],
              "MongoDB: colecao com ponto")
        check("error" not in core.rpc("datasource.impact", {
            "name": mongo["name"], "sql": "telemetria.sensores.drop()"}), "MongoDB: impacto de nome com ponto")
        dot_impact = core.event("impact")
        check(dot_impact["statements"][0]["targets"] == ["telemetria.sensores"],
              "MongoDB: alvo conserva o nome completo")
        check(core.query(mongo["name"], "telemetria.sensores.drop()", confirmed=True)["affected"] == 1,
              "MongoDB: remover colecao com ponto")
        core.query(mongo["name"],
            'sensores.insertMany([{"placa":"esp32","v":1},{"placa":"pico","v":2},{"placa":"esp32","v":3}])')
        check(password not in "".join(core.raw + core.errors), "senha ausente no stdout e stderr do core")
        check(all(password not in path.read_text(errors="replace")
                  for path in project.rglob("*") if path.is_file()), "senha ausente no projeto")
        # A UI recebe a senha so' pelo ambiente; o perfil persiste apenas o nome.
        pg["secretSource"] = "environment"
        pg["secretVariable"] = "KINEIN_TEST_PG_PASSWORD"
        core.rpc("datasource.save", {"profile": pg})
        for name, content in (
            (pg["name"], "SELECT * FROM kinein_prova.leituras ORDER BY id;\n\nDELETE FROM kinein_prova.leituras WHERE id=1;\n\nDROP SCHEMA kinein_prova CASCADE;\n"),
            (mongo["name"], 'sensores.find({})\n\nsensores.insertOne({"placa":"nova","v":4})\n\nsensores.deleteMany({"placa":"esp32"})\n\nsensores.drop()\n')):
            response = core.rpc("datasource.console", {"name": name})
            Path(response["result"]["path"]).write_text(content)
        core.close()
        core = None
        if args.metadata:
            args.metadata.write_text(json.dumps({"directory": str(directory), "project": str(project),
                "ports": ports, "containers": containers, "uiLog": str(ui_logs)}, indent=2))
        if args.ui:
            env["KINEIN_CORE_BIN"] = str(REPO / "target/debug/kinein-core")
            env["QT_QPA_PLATFORM"] = "xcb"
            print("IDE pronta para a prova com mouse/teclado.", flush=True)
            with ui_logs.open("w") as log:
                subprocess.run([str(REPO / "build/dev-local/ui/kinein-vectis"), str(project)],
                               env=env, stdout=log, stderr=log, check=True)
            check(password not in ui_logs.read_text(), "senha ausente no log da UI")
        print("Banco real: tudo passou.", flush=True)
    finally:
        try:
            if core:
                core.close()
        finally:
            for name in reversed(containers):
                subprocess.run(["podman", "rm", "-f", name], capture_output=True, check=False)
            if args.metadata:
                # O resumo de limpeza continua disponivel sem senha nem dados do banco.
                args.metadata.write_text(json.dumps({"cleaned": True, "containers": containers}))
            shutil.rmtree(directory)

if __name__ == "__main__":
    main()
