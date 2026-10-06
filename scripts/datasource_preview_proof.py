"""Prévia PostgreSQL real, com contagens fora do core e recursos do chamador."""
import subprocess
import time


def exercise(core, project, postgres, password, container, check, command):
    serial = 0

    def sql(text):
        return command(["podman", "exec", container, "psql", "-U", "postgres", "-Atc", text])

    def saved(profile):
        response = core.rpc("datasource.save", {"profile": profile})
        assert "error" not in response
        return next(item for item in response["result"]["profiles"] if item["name"] == profile["name"])

    profile = saved(dict(postgres, name="PostgreSQL-previa", production=False))

    def context(destination=None):
        nonlocal serial
        serial += 1
        return {"clientContext": f"preview:{serial}", "expectedContext": {
            "workspace": str(project), "profile": destination or profile}}

    def params(text, destination=None):
        destination = destination or profile
        return {"name": destination["name"], "sql": text, "preview": True,
                "confirmWrite": True, "password": password, "maxRows": 2, **context(destination)}

    def start(text, destination=None):
        request = params(text, destination)
        response = core.rpc("datasource.query", request)
        assert "error" not in response, response.get("error", {}).get("message")
        check(response["result"]["preview"] and response["result"]["clientContext"] == request["clientContext"], "prévia: aceite correlacionado")
        event = core.event("previewed", request["clientContext"])
        return request, event

    def decide(request, event, decision):
        return core.rpc("datasource.preview.decide", {"name": request["name"],
            "clientContext": request["clientContext"], "expectedContext": request["expectedContext"],
            "previewId": event["previewId"], "decision": decision})

    def ended(request, outcome, timeout=30):
        result = core.event("queried", request["clientContext"], timeout=timeout)
        check(result.get("previewOutcome") == outcome, "prévia: desfecho " + outcome + " (recebido: " + str(result.get("previewOutcome")) + ")")
        return result

    sql("CREATE TABLE kinein_prova.previa (id int PRIMARY KEY, valor int); INSERT INTO kinein_prova.previa VALUES (1,10),(2,20); CREATE SEQUENCE kinein_prova.previa_seq START 1000")
    for text in ("SELECT 1", "CREATE DATABASE previa", "UPDATE kinein_prova.previa SET valor=1; COMMIT",
                 "UPDATE kinein_prova.previa SET valor=1 -- fim\r;COMMIT",
                 "DELETE FROM kinein_prova.previa RETURNING 1 AS é$$;COMMIT;SELECT 1 AS fim$$",
                 "UPDATE kinein_prova.previa SET valor=1; 'literal'"):
        response = core.rpc("datasource.query", params(text))
        check(response.get("error", {}).get("code") == "DATA_SOURCE_PREVIEW_UNAVAILABLE", "prévia: comando inelegível recusado antes do job")
    readonly = saved(dict(profile, name="PostgreSQL-previa-leitura", readOnly=True))
    check(core.rpc("datasource.query", params("DELETE FROM kinein_prova.previa", readonly)).get("error", {}).get("code") == "READ_ONLY_VIOLATION", "prévia: confirmação não contorna somente leitura")
    request = params("INSERT INTO kinein_prova.previa VALUES (3,30)")
    request["confirmWrite"] = False
    check(core.rpc("datasource.query", request).get("error", {}).get("code") == "WRITE_CONFIRMATION_REQUIRED", "prévia: escrita comum exige aviso antes da senha/job")
    request = params("INSERT INTO kinein_prova.previa VALUES (3,30)")
    request.pop("expectedContext")
    check(core.rpc("datasource.query", request).get("error", {}).get("code") == "INVALID_PARAMS", "prévia: contexto obrigatório")

    tls = saved(dict(profile, name="PostgreSQL-TLS-obrigatorio", tls="require"))
    result = core.query(tls["name"], "SELECT 1", password)
    check(not result["success"] and sql("SHOW ssl") == "off", "TLS obrigatório: conexão comum recusa servidor sem TLS")
    request = params("INSERT INTO kinein_prova.previa VALUES (3,30)", tls)
    check("error" not in core.rpc("datasource.query", request), "TLS obrigatório: prévia aceita job")
    check(not ended(request, "failed")["success"], "TLS obrigatório: prévia também recusa servidor sem TLS")

    production = saved(dict(profile, name="PostgreSQL-previa-producao", production=True))
    request = params("UPDATE kinein_prova.previa SET valor=99 WHERE id>=1", production)
    check("error" not in core.rpc("datasource.query", request), "prévia: produção aceita medição silenciosa do filtro")
    measured = core.event("queried", request["clientContext"])
    check(not measured["success"] and measured["access"] == "write"
          and measured["confirmationSql"] == request["sql"] and sql("SELECT sum(valor) FROM kinein_prova.previa") == "30",
          "prévia: filtro global pede nomes de produção sem escrever")
    request, event = start("UPDATE kinein_prova.previa SET valor=11 WHERE id=1", production)
    decide(request, event, "rollback")
    ended(request, "rolledBack")
    check(sql("SELECT sum(valor) FROM kinein_prova.previa") == "30", "prévia: medição recusada liberou a reserva do destino")

    request, event = start("UPDATE kinein_prova.previa SET valor=11 WHERE id=1")
    check(event["rows"] == [["1", "11"]] and event["affected"] == 1 and "RETURNING *" in event["executedSql"], "prévia: amostra real e comando executado")
    check(sql("SELECT valor FROM kinein_prova.previa WHERE id=1") == "10", "prévia: outra conexão ainda vê o valor anterior")
    check(core.rpc("datasource.query", params("INSERT INTO kinein_prova.previa VALUES (4,40)")).get("error", {}).get("code") == "DATA_SOURCE_PREVIEW_UNAVAILABLE", "prévia: destino ocupado recusado antes de conectar")
    old = dict(request, clientContext="preview:old")
    check("error" in decide(old, event, "commit"), "prévia: token antigo não confirma")
    check("error" not in decide(request, event, "commit"), "prévia: COMMIT aceito")
    check("error" in decide(request, event, "commit"), "prévia: decisão repetida recusada")
    check(ended(request, "committed")["success"] and sql("SELECT valor FROM kinein_prova.previa WHERE id=1") == "11", "prévia: COMMIT conferido por psql")

    request, event = start("INSERT INTO kinein_prova.previa VALUES (3,NULL) -- fim")
    check(event["rows"] == [["3", None]], "prévia: NULL preservado")
    check("error" not in decide(request, event, "rollback"), "prévia: ROLLBACK aceito")
    ended(request, "rolledBack")
    check(sql("SELECT count(*) FROM kinein_prova.previa") == "2", "prévia: ROLLBACK conferido por psql")

    request, event = start("UPDATE kinein_prova.previa AS p SET valor=s.v FROM (VALUES (1,90)) AS s(id,v) WHERE p.id=s.id RETURNING p.id,p.valor")
    check(event["rows"] == [["1", "90"]] and event["executedSql"].count("RETURNING") == 1, "prévia: UPDATE FROM e RETURNING explícito")
    decide(request, event, "rollback")
    ended(request, "rolledBack")

    request, event = start("INSERT INTO kinein_prova.previa VALUES (nextval('kinein_prova.previa_seq'),30)")
    check(event["rows"][0][0] == "1000", "prévia: sequência usada na escrita")
    decide(request, event, "rollback")
    ended(request, "rolledBack")
    check(sql("SELECT nextval('kinein_prova.previa_seq')") == "1001", "prévia: rollback não recupera sequência, como o aviso declara")

    request, event = start("DELETE FROM kinein_prova.previa WHERE id=2")
    check(core.rpc("job.cancel", {"jobId": event["jobId"]})["result"]["cancelled"], "prévia: job cancelável")
    check("error" in decide(request, event, "commit"), "prévia: cancelamento recusa COMMIT posterior")
    ended(request, "cancelled")
    check(sql("SELECT count(*) FROM kinein_prova.previa") == "2", "prévia: cancelamento preserva dados")

    request, event = start("DELETE FROM kinein_prova.previa WHERE id=2")
    changed = saved(dict(profile, database="template1"))
    ended(request, "cancelled")
    check("error" in decide(request, event, "commit"), "prévia: perfil substituído recusa a decisão antiga")
    profile = saved(dict(changed, database="postgres"))

    for method, arguments in (("workspace.close", {}), ("workspace.open", {"path": str(project)})):
        request, event = start("DELETE FROM kinein_prova.previa WHERE id=2")
        check("error" not in core.rpc(method, arguments), "prévia: transição " + method)
        ended(request, "cancelled")
        check("error" not in core.rpc("workspace.open", {"path": str(project)}), "prévia: reabrir projeto")
        check("error" in decide(request, event, "commit"), "prévia: projeto reaberto não ressuscita decisão")

    sql("CREATE TABLE kinein_prova.previa_larga AS SELECT i AS id, i AS valor FROM generate_series(1,3000) i")
    request, event = start("UPDATE kinein_prova.previa_larga SET valor=valor+1 WHERE id>0")
    check(event["affected"] == 3000 and len(event["rows"]) == 2 and event["truncated"], "prévia: drena 3000 alterações e retém duas linhas")
    decide(request, event, "rollback")
    ended(request, "rolledBack")
    check(sql("SELECT count(*) FROM kinein_prova.previa_larga WHERE id=valor") == "3000", "prévia: resultado grande desfeito")
    request = params("UPDATE kinein_prova.previa SET valor=12 WHERE id=1 RETURNING repeat('x',17000)")
    check("error" not in core.rpc("datasource.query", request), "prévia: resultado hostil aceito pelo job")
    check(not ended(request, "failed")["success"] and sql("SELECT valor FROM kinein_prova.previa WHERE id=1") == "11", "prévia: célula acima do teto encerra a conexão e desfaz")
    request = params("INSERT INTO kinein_prova.previa VALUES (1,30)")
    check("error" not in core.rpc("datasource.query", request), "prévia: erro SQL no worker")
    ended(request, "failed")

    blocker = subprocess.Popen(["podman", "exec", container, "psql", "-U", "postgres", "-Atc",
        "BEGIN; LOCK TABLE kinein_prova.previa IN ACCESS EXCLUSIVE MODE; SELECT pg_sleep(5); ROLLBACK"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if sql("SELECT count(*) FROM pg_locks WHERE relation='kinein_prova.previa'::regclass AND mode='AccessExclusiveLock' AND granted") == "1":
                break
            time.sleep(0.05)
        else:
            raise AssertionError("lock de prova não foi adquirido")
        request = params("UPDATE kinein_prova.previa SET valor=12 WHERE id=1")
        check("error" not in core.rpc("datasource.query", request), "prévia: consulta bloqueada roda em job")
        ended(request, "failed")
        blocker.wait(timeout=8)
    finally:
        if blocker.poll() is None:
            blocker.terminate()
            blocker.wait(timeout=5)
    check(sql("SELECT valor FROM kinein_prova.previa WHERE id=1") == "11", "prévia: lock_timeout falha sem alterar dados")

    sql("ALTER TABLE kinein_prova.previa ADD CONSTRAINT previa_unique_value UNIQUE(valor) DEFERRABLE INITIALLY DEFERRED")
    request, event = start("INSERT INTO kinein_prova.previa VALUES (99,20)")
    decide(request, event, "commit")
    ended(request, "failed")
    check(sql("SELECT count(*) FROM kinein_prova.previa WHERE id=99") == "0", "prévia: constraint diferida recusa COMMIT e não persiste")
    sql("ALTER TABLE kinein_prova.previa DROP CONSTRAINT previa_unique_value")

    request, event = start("DELETE FROM kinein_prova.previa WHERE id=2")
    print("prévia: aguardando expiração real de 60 segundos", flush=True)
    ended(request, "expired", timeout=75)
    check("error" in decide(request, event, "commit") and sql("SELECT count(*) FROM kinein_prova.previa") == "2", "prévia: prazo expira e preserva dados")

    # Uma trigger diferida mantém COMMIT em execução para testar perda da resposta.
    sql("CREATE FUNCTION kinein_prova.previa_commit_delay() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(5); RETURN NEW; END $$; CREATE CONSTRAINT TRIGGER previa_commit_delay AFTER INSERT ON kinein_prova.previa DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION kinein_prova.previa_commit_delay()")
    request, event = start("INSERT INTO kinein_prova.previa VALUES (100,100)")
    check("error" not in decide(request, event, "commit"), "prévia: COMMIT com trigger diferida aceito")
    deadline = time.monotonic() + 4
    pid = ""
    while time.monotonic() < deadline and not pid:
        pid = sql("SELECT pid FROM pg_stat_activity WHERE pid<>pg_backend_pid() AND state='active' AND query='COMMIT'")
        if not pid:
            time.sleep(0.05)
    assert pid.isdigit(), "COMMIT não entrou na trigger diferida"
    check(not core.rpc("job.cancel", {"jobId": event["jobId"]})["result"]["cancelled"], "prévia: COMMIT aceito não promete cancelamento")
    sql("SELECT pg_terminate_backend(" + pid + ")")
    result = ended(request, "unknown")
    check(not result["success"] and "desconhecido" in result["message"], "prévia: perda da resposta de COMMIT não presume sucesso")
    check(sql("SELECT count(*) FROM kinein_prova.previa WHERE id=100") == "0", "prévia: neste caso a conexão independente confirma rollback")

    request, event = start("INSERT INTO kinein_prova.previa VALUES (101,101)")
    decide(request, event, "commit")
    deadline = time.monotonic() + 4
    pid = ""
    while time.monotonic() < deadline and not pid:
        pid = sql("SELECT pid FROM pg_stat_activity WHERE pid<>pg_backend_pid() AND state='active' AND query='COMMIT'")
        if not pid:
            time.sleep(0.05)
    assert pid.isdigit(), "COMMIT não entrou na trigger diferida"
    # Somente o backend da prova, dentro do contêiner próprio. Produz EOF sem
    # resposta SQL; o postmaster recupera esse servidor descartável.
    command(["podman", "exec", container, "kill", "-9", pid])
    ended(request, "unknown")
    deadline = time.monotonic() + 15
    while True:
        try:
            check(sql("SELECT count(*) FROM kinein_prova.previa WHERE id=101") == "0", "prévia: servidor recuperado confirma saldo após perda da conexão")
            break
        except subprocess.CalledProcessError:
            assert time.monotonic() < deadline, "servidor da prova não recuperou"
            time.sleep(0.1)
    sql("DROP TABLE kinein_prova.previa_larga; DROP TABLE kinein_prova.previa; DROP FUNCTION kinein_prova.previa_commit_delay(); DROP SEQUENCE kinein_prova.previa_seq")

    response = core.rpc("datasource.console", {"name": profile["name"]})
    from pathlib import Path
    Path(response["result"]["path"]).write_text("UPDATE kinein_prova.leituras SET valor=25 WHERE id=2;\n\nINSERT INTO kinein_prova.leituras VALUES (5,50);\n")
