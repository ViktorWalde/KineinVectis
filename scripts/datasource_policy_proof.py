"""Prova integrada da política, usando os servidores temporários de testar-banco-real."""
import sqlite3
from pathlib import Path


def exercise(core, project, postgres, mongo, password, containers, check, command):
    serial = 0

    def saved(profile):
        response = core.rpc("datasource.save", {"profile": profile})
        assert "error" not in response
        return next(item for item in response["result"]["profiles"] if item["name"] == profile["name"])

    def context(profile):
        nonlocal serial
        serial += 1
        return {"clientContext": f"proof:{serial}", "expectedContext": {"workspace": str(project), "profile": profile}}

    def refusal(method, params, code, label):
        response = core.rpc(method, params)
        check(response.get("error", {}).get("code") == code, label)

    path = project / "politica.sqlite"
    with sqlite3.connect(path) as connection:
        connection.executescript("CREATE TABLE itens (id INTEGER PRIMARY KEY, valor INTEGER); INSERT INTO itens VALUES (1,10),(2,20);")
    sqlite = {"name": "SQLite-producao", "engine": "sqlite", "host": "", "port": 0,
              "database": str(path), "user": ""}
    pg = saved(dict(postgres, name="PostgreSQL-producao", production=True))
    mg = saved(dict(mongo, name="MongoDB-producao", production=True))
    sq = saved(dict(sqlite, production=True))
    cases = (
        (pg, "SELECT id FROM kinein_prova.leituras ORDER BY id", "INSERT INTO kinein_prova.leituras VALUES (4,40)",
         "DELETE FROM kinein_prova.leituras WHERE id=4", "leituras", password),
        (mg, "sensores.find({})", 'sensores.insertOne({"placa":"politica","v":4})',
         'sensores.deleteOne({"placa":"politica"})', "sensores", None),
        (sq, "SELECT id FROM itens ORDER BY id", "INSERT INTO itens VALUES (4,40)",
         "DELETE FROM itens WHERE id=4", "itens", None),
    )
    for profile, read, insert, delete, target, secret in cases:
        name = profile["name"]
        readonly = saved(dict(profile, name=name + "-leitura", readOnly=True))
        ctx = context(readonly)
        result = core.query(readonly["name"], read, secret, context=ctx, max_rows=100)
        initial = result["rowCount"]
        check(result["success"] and result["access"] == "read" and result["clientContext"] == ctx["clientContext"], name + ": leitura autorizada e correlacionada")
        attacks = [insert, delete]
        if profile["engine"] != "mongo":
            attacks += ["SELECT 1; COMMIT; " + delete, "WITH x AS (" + delete + " RETURNING *) SELECT * FROM x"]
            table = "kinein_prova.leituras" if profile["engine"] == "postgres" else "itens"
            attacks += ["SELECT 1 -- comment\r; COMMIT; DELETE FROM " + table,
                        "SELECT 1 AS é$$; COMMIT; DELETE FROM " + table + "; SELECT 1 AS fim$$"]
        for text in attacks:
            refusal("datasource.query", {"name": readonly["name"], "sql": text, "confirmWrite": True,
                "password": secret,
                "confirmation": {"connection": readonly["name"], "target": target}, **context(readonly)},
                "READ_ONLY_VIOLATION", name + ": somente leitura recusa escrita/lote/CTE antes do job")
        check(core.query(readonly["name"], read, secret)["rowCount"] == initial, name + ": recusas preservam registros")
        for method in ("datasource.test", "datasource.introspect", "datasource.query", "datasource.impact", "datasource.destroy"):
            ctx = context(profile)
            ctx["expectedContext"]["profile"] = dict(profile, host="destino-diferente")
            params = {"name": name, **ctx}
            if method in ("datasource.query", "datasource.impact"):
                params["sql"] = read
            refusal(method, params, "DATA_SOURCE_CONTEXT_CHANGED", name + ": destino diferente recusado em " + method)
        for method, event in (("datasource.test", "tested"), ("datasource.introspect", "introspected")):
            ctx = context(profile)
            params = {"name": name, **ctx}
            if secret:
                params["password"] = secret
            check("error" not in core.rpc(method, params), name + ": " + method + " aceito")
            check(core.event(event, ctx["clientContext"])["ok"], name + ": evento de " + event + " correlacionado")
        refusal("datasource.query", {"name": name, "sql": insert, **context(profile)},
                "WRITE_CONFIRMATION_REQUIRED", name + ": inserção em produção exige aviso")
        check(core.query(name, insert, secret, True, context=context(profile))["access"] == "write", name + ": inserção confirmada")
        ctx = context(profile)
        params = {"name": name, "sql": delete, **ctx}
        if secret:
            params["password"] = secret
        check("error" not in core.rpc("datasource.impact", params), name + ": medir remoção")
        impact = core.event("impact", ctx["clientContext"])
        check(impact["confirmationTarget"] == target and impact["requiresConnection"], name + ": core fornece nomes exigidos")
        refusal("datasource.query", {"name": name, "sql": delete, "confirmWrite": True, **context(profile),
            "confirmation": {"connection": name[:-1], "target": target}}, "WRITE_CONFIRMATION_REQUIRED", name + ": nome parcial recusado")
        result = core.query(name, delete, secret, True, context=context(profile), confirmation={"connection": name, "target": target})
        check(result["success"] and result["affected"] == 1 and result["access"] == "write", name + ": remoção com os nomes completos")
        check(core.query(name, read, secret)["rowCount"] == initial, name + ": saldo de registros correto")
        response = core.rpc("datasource.console", {"name": name})
        Path(response["result"]["path"]).write_text(read + ";\n\n" + insert + ";\n\n" + delete + ";\n" if profile["engine"] != "mongo"
                                                     else read + "\n\n" + insert + "\n\n" + delete + "\n")
        response = core.rpc("datasource.console", {"name": readonly["name"]})
        Path(response["result"]["path"]).write_text(insert + (";\n" if profile["engine"] != "mongo" else "\n"))

    # Conferência fora do core, pelas ferramentas de cada servidor/arquivo.
    check(command(["podman", "exec", containers[0], "psql", "-U", "postgres", "-Atc",
        "SELECT count(*) FROM kinein_prova.leituras"]) == "3", "PostgreSQL: contagem independente")
    check(command(["podman", "exec", containers[1], "mongosh", "--quiet", "--eval",
        "db.getSiblingDB('kinein_prova').sensores.countDocuments({})"]) == "3", "MongoDB: contagem independente")
    with sqlite3.connect(path) as connection:
        check(connection.execute("SELECT count(*) FROM itens").fetchone()[0] == 2, "SQLite: contagem independente")
    readonly = saved(dict(sq, name="SQLite-producao-leitura", readOnly=True))
    refusal("datasource.destroy", {"name": readonly["name"], "data": True, **context(readonly)},
            "READ_ONLY_VIOLATION", "SQLite: somente leitura preserva arquivo")
    check(path.exists(), "SQLite: arquivo continua presente")

    command(["podman", "exec", containers[0], "psql", "-U", "postgres", "-c", "CREATE DATABASE kinein_policy_drop"])
    drop = saved(dict(pg, name="PostgreSQL-remocao", database="kinein_policy_drop"))
    confirmation = {"connection": drop["name"], "target": drop["database"]}
    refusal("datasource.destroy", {"name": drop["name"], "data": True, "confirmation": confirmation, **context(drop)},
            "SECRET_REQUIRED", "PostgreSQL: remoção pede senha sem apagá-la do perfil")
    ctx = context(drop)
    response = core.rpc("datasource.destroy", {"name": drop["name"], "data": True, "password": password,
        "confirmation": confirmation, **ctx})
    check("error" not in response and response["result"]["clientContext"] == ctx["clientContext"], "PostgreSQL: remoção aceita com senha de sessão")
    check(core.event("destroyed", ctx["clientContext"])["success"], "PostgreSQL: remoção concluída e correlacionada")
    check(command(["podman", "exec", containers[0], "psql", "-U", "postgres", "-Atc",
        "SELECT count(*) FROM pg_database WHERE datname='kinein_policy_drop'"]) == "0", "PostgreSQL: banco removido, prova independente")
