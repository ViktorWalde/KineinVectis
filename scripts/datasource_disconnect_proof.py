"""Desconexão real: drena consulta, revoga prévia e aguarda pools MongoDB."""
import time
from pathlib import Path


def exercise(core, project, postgres, mongo, password, containers, check, command):
    serial = 0

    def sql(text):
        return command(["podman", "exec", containers[0], "psql", "-U", "postgres", "-Atc", text])

    def mongo_command(text):
        return command(["podman", "exec", containers[1], "mongosh", "--quiet", "--eval", text])

    def saved(profile):
        response = core.rpc("datasource.save", {"profile": profile})
        assert "error" not in response
        return next(item for item in response["result"]["profiles"] if item["name"] == profile["name"])

    pg = saved(dict(postgres, name="PostgreSQL-desconectar", production=False))
    other = saved(dict(pg, name="PostgreSQL-vizinho"))
    mg = saved(dict(mongo, name="MongoDB-desconectar", production=False))
    consoles = []
    for profile in (pg, other, mg):
        response = core.rpc("datasource.console", {"name": profile["name"]})
        path = Path(response["result"]["path"])
        path.write_text("-- rascunho preservado 😀\n")
        consoles.append(path)
    original = (project / ".kinein/datasources.json").read_bytes()

    def context(profile):
        nonlocal serial
        serial += 1
        return {"clientContext": f"disconnect-real:{serial}", "expectedContext": {
            "workspace": str(project), "profile": profile}}

    def wait_for(predicate, message):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if predicate():
                return
            time.sleep(0.05)
        raise AssertionError(message)

    def disconnect(profile):
        request = {"name": profile["name"], **context(profile)}
        started = time.monotonic()
        response = core.rpc("datasource.disconnect", request)
        check("error" not in response and response["result"]["clientContext"] == request["clientContext"],
              "desconectar: aceite correlacionado sem senha")
        check(time.monotonic() - started < 1, "desconectar: despacho não espera pelo driver")
        return request, response["result"]

    def closed(request, accepted):
        event = core.event("disconnected", request["clientContext"])
        check(event["success"] and event["jobId"] == accepted["jobId"] and event["name"] == request["name"],
              "desconectar: encerramento correlacionado")
        return event

    def no_pg_connections():
        wait_for(lambda: sql("SELECT count(*) FROM pg_stat_activity WHERE client_addr IS NOT NULL") == "0",
                 "conexão TCP do core ainda presente depois da desconexão")
        check(True, "PostgreSQL: nenhuma sessão TCP do core, conferido por psql independente")

    sql("CREATE TABLE kinein_prova.desconectar (id int PRIMARY KEY, valor int); INSERT INTO kinein_prova.desconectar VALUES (1,10),(2,20)")
    running = {"name": pg["name"], "sql": "SELECT pg_sleep(3),id FROM kinein_prova.desconectar WHERE id=1",
               "password": password, **context(pg)}
    check("error" not in core.rpc("datasource.query", running), "PostgreSQL: consulta lenta aceita")
    wait_for(lambda: sql("SELECT count(*) FROM pg_stat_activity WHERE client_addr IS NOT NULL AND state='active' AND query LIKE '%pg_sleep(3)%'") == "1",
             "consulta lenta não iniciou")
    request, accepted = disconnect(pg)
    check("error" not in core.rpc("core.ping", {}), "desconectar: core continua respondendo")
    check("error" in core.rpc("datasource.query", {"name": pg["name"], "sql": "SELECT 1", "password": password, **context(pg)}),
          "desconectar: nova operação no destino recusada antes de conectar")
    check(core.query(other["name"], "SELECT 7", password, context=context(other))["rows"] == [["7"]],
          "desconectar: outro destino continua livre")
    with core.inbox.mutex:
        queued = list(core.inbox.queue)
    check(not any(event.get("method") == "event.datasource.disconnected" and event.get("params", {}).get("clientContext") == request["clientContext"]
                  for event in core.events + queued), "desconectar: nenhum sucesso antecipado durante a consulta")
    closed(request, accepted)
    check(core.event("queried", running["clientContext"])["success"], "desconectar: consulta já aceita terminou")
    no_pg_connections()

    def preview(text):
        request = {"name": pg["name"], "sql": text, "password": password,
                   "preview": True, "confirmWrite": True, **context(pg)}
        check("error" not in core.rpc("datasource.query", request), "PostgreSQL: prévia aceita")
        return request, core.event("previewed", request["clientContext"])

    def decide(request, event):
        return core.rpc("datasource.preview.decide", {"name": request["name"], "decision": "commit",
            "previewId": event["previewId"], "clientContext": request["clientContext"],
            "expectedContext": request["expectedContext"]})

    pending, event = preview("UPDATE kinein_prova.desconectar SET valor=99 WHERE id=1")
    check(event["affected"] == 1 and sql("SELECT valor FROM kinein_prova.desconectar WHERE id=1") == "10",
          "PostgreSQL: prévia pendente ainda não persistiu")
    request, accepted = disconnect(pg)
    closed(request, accepted)
    check(core.event("queried", pending["clientContext"])["previewOutcome"] == "cancelled",
          "desconectar: prévia sem decisão encerrada por ROLLBACK")
    check("error" in decide(pending, event) and sql("SELECT valor FROM kinein_prova.desconectar WHERE id=1") == "10",
          "desconectar: decisão antiga recusada e dados intactos")
    no_pg_connections()

    sql("CREATE FUNCTION kinein_prova.desconectar_delay() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(3); RETURN NEW; END $$; CREATE CONSTRAINT TRIGGER desconectar_delay AFTER INSERT ON kinein_prova.desconectar DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION kinein_prova.desconectar_delay()")
    committing, event = preview("INSERT INTO kinein_prova.desconectar VALUES (3,30)")
    check("error" not in decide(committing, event), "PostgreSQL: COMMIT aceito antes de desconectar")
    wait_for(lambda: sql("SELECT count(*) FROM pg_stat_activity WHERE client_addr IS NOT NULL AND state='active' AND query='COMMIT'") == "1",
             "COMMIT diferido não iniciou")
    request, accepted = disconnect(pg)
    closed(request, accepted)
    check(core.event("queried", committing["clientContext"])["previewOutcome"] == "committed"
          and sql("SELECT valor FROM kinein_prova.desconectar WHERE id=3") == "30",
          "desconectar: aguarda COMMIT já aceito sem prometer rollback")
    no_pg_connections()
    check(core.query(pg["name"], "SELECT sum(valor) FROM kinein_prova.desconectar", password, context=context(pg))["rows"] == [["60"]],
          "desconectar: nova consulta explícita funciona após encerramento")

    def no_mongo_connections():
        result = mongo_command("print(db.getSiblingDB('admin').aggregate([{$currentOp:{allUsers:true,idleConnections:true}},{$match:{$or:[{appName:'kinein-vectis'},{'clientMetadata.application.name':'kinein-vectis'}]}},{$count:'n'}]).toArray().map(x=>x.n).join('') || '0')")
        check(result == "0", "MongoDB: pool kinein-vectis encerrado, conferido por currentOp independente")

    for text in ('desconectar.insertMany([{"id":1,"v":10},{"id":2,"v":20}])',
                 'desconectar.updateOne({"id":1},{"$set":{"v":11}})', 'desconectar.find({})'):
        check(core.query(mg["name"], text, context=context(mg))["success"], "MongoDB: caminho existente " + text.split('(')[0])
        request, accepted = disconnect(mg)
        closed(request, accepted)
        no_mongo_connections()
    for method, terminal, extra in (("datasource.test", "tested", {}), ("datasource.introspect", "introspected", {}),
                                    ("datasource.impact", "impact", {"sql": 'desconectar.deleteOne({"id":1})'})):
        operation = {"name": mg["name"], **context(mg), **extra}
        check("error" not in core.rpc(method, operation), "MongoDB: " + method + " aceito")
        core.event(terminal, operation["clientContext"])
        request, accepted = disconnect(mg)
        closed(request, accepted)
        no_mongo_connections()
    check(mongo_command("print(db.getSiblingDB('kinein_prova').desconectar.find({},{_id:0}).sort({id:1}).toArray().map(x=>x.v).join(','))") == "11,20",
          "MongoDB: escrita/leitura e dados preservados por consulta independente")
    check((project / ".kinein/datasources.json").read_bytes() == original
          and all(path.read_text() == "-- rascunho preservado 😀\n" for path in consoles),
          "desconectar: perfis e arquivos dos consoles intactos")
