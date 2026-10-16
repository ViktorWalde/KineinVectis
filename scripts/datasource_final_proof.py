"""Passo 15d (roadmaps/59 §5.6.1): o que faltava provar contra servidores reais.

TLS verify-full com uma CA de teste, o aviso de `UPDATE … FROM`, o nome
`localhost` pelo resolvedor com prazo (o `::1` recusa e o driver cai no
`127.0.0.1`, onde o contêiner publica), o SQLite integrado e dois jobs ao
mesmo tempo no mesmo perfil. Tudo em diretórios e contêineres temporários.
"""
from pathlib import Path
import subprocess
import tempfile
import time


def certificates(directory: Path, command) -> None:
    """Uma CA de teste, um certificado para `localhost` e uma segunda CA."""
    def openssl(*args):
        command(["openssl", *args], cwd=directory)

    openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "2",
            "-keyout", "ca.key", "-out", "ca.pem", "-subj", "/CN=Kinein prova CA")
    openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "2",
            "-keyout", "other.key", "-out", "other-ca.pem", "-subj", "/CN=Outra CA")
    openssl("req", "-newkey", "rsa:2048", "-nodes", "-keyout", "server.key",
            "-out", "server.csr", "-subj", "/CN=localhost")
    (directory / "server.ext").write_text(
        "subjectAltName=DNS:localhost\nbasicConstraints=CA:FALSE\n"
        "keyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
    openssl("x509", "-req", "-in", "server.csr", "-CA", "ca.pem", "-CAkey", "ca.key",
            "-CAcreateserial", "-out", "server.crt", "-days", "2", "-extfile", "server.ext")


def tls_server(directory: Path, name: str, command, ready) -> int:
    """PostgreSQL com `ssl=on` e o certificado de `localhost`."""
    shim = ("cp /certs/server.crt /certs/server.key /tmp/ && chown postgres /tmp/server.* "
            "&& chmod 600 /tmp/server.key && exec docker-entrypoint.sh postgres -c ssl=on "
            "-c ssl_cert_file=/tmp/server.crt -c ssl_key_file=/tmp/server.key")
    command(["podman", "run", "--rm", "--pull=never", "-d", "--name", name,
             "--label", "io.kinein.prova=banco", "-p", "127.0.0.1::5432",
             "-e", "POSTGRES_HOST_AUTH_METHOD=trust", "-v", f"{directory}:/certs:ro",
             "--entrypoint", "sh", "docker.io/library/postgres:16-alpine", "-c", shim])
    ready(name, "pg")
    return int(command(["podman", "port", name, "5432"]).rsplit(":", 1)[1])


def tested(core, profile) -> dict:
    """O `datasource.test` de um perfil salvo, com o evento dele."""
    response = core.rpc("datasource.save", {"profile": profile})
    assert "error" not in response, response
    accepted = core.rpc("datasource.test", {"name": profile["name"]})
    assert "error" not in accepted, accepted
    return core.event("tested", timeout=30)


def exercise(core, project, postgres, password, containers, check, command, ready):
    with tempfile.TemporaryDirectory(prefix="kinein-tls-") as raw:
        directory = Path(raw)
        certificates(directory, command)
        name = containers[0] + "-tls"
        containers.append(name)
        port = tls_server(directory, name, command, ready)
        base = {"engine": "postgres", "port": port, "database": "postgres", "user": "postgres",
                "tls": "require", "caFile": str(directory / "ca.pem")}
        good = tested(core, dict(base, name="TLS-nome-certo", host="localhost"))
        check(good.get("ok") is True, "TLS verify-full: CA de teste e nome certo conectam")
        by_ip = tested(core, dict(base, name="TLS-pelo-ip", host="127.0.0.1"))
        check(by_ip.get("ok") is False and "certificate" in str(by_ip.get("message", "")).lower(),
              "TLS verify-full: pelo IP o nome nao bate e e' recusado")
        other = tested(core, dict(base, name="TLS-outra-ca", host="localhost",
                                  caFile=str(directory / "other-ca.pem")))
        check(other.get("ok") is False, "TLS verify-full: outra CA e' recusada")
        plain = tested(core, dict(base, name="TLS-desligado", host="127.0.0.1", tls="disable",
                                  caFile=None))
        check(plain.get("ok") is True, "TLS desligado continua conectando (o servidor aceita os dois)")

    # O nome `localhost` passa pelo resolvedor com prazo e cai no IPv4.
    by_name = dict(postgres, name="PostgreSQL-localhost", host="localhost")
    check("error" not in core.rpc("datasource.save", {"profile": by_name}), "perfil por nome")
    answer = core.query(by_name["name"], "SELECT 1", password)
    check(answer["success"] and answer["rows"] == [["1"]],
          "nome `localhost`: o resolvedor entrega os enderecos e o driver chega no 127.0.0.1")

    # O aviso com `UPDATE … FROM` conta as linhas do alvo (15c).
    for sql in ["CREATE TABLE kinein_prova.fonte (id int, valor int)",
                "INSERT INTO kinein_prova.fonte VALUES (2, 99)"]:
        check(core.query(postgres["name"], sql, password)["success"], "PostgreSQL: " + sql.split()[0])
    response = core.rpc("datasource.impact", {
        "name": postgres["name"], "password": password,
        "sql": "UPDATE kinein_prova.leituras SET valor = f.valor FROM kinein_prova.fonte f "
               "WHERE kinein_prova.leituras.id = f.id"})
    check("error" not in response, "UPDATE … FROM: medicao aceita")
    impact = core.event("impact")["statements"][0]
    check(impact.get("rows") == 1 and impact.get("severity") == "write" and not impact.get("note"),
          "UPDATE … FROM: conta 1 linha do alvo, sem erro de SQL")

    # Dois jobs ao mesmo tempo no mesmo perfil, cada evento com o seu contexto.
    contexts = []
    for index in range(2):
        context = f"concorrencia:{index}:{time.monotonic_ns()}"
        contexts.append(context)
        response = core.rpc("datasource.query", {
            "name": postgres["name"], "password": password, "clientContext": context,
            "sql": f"SELECT {index + 1}, pg_sleep(0.5)"})
        check("error" not in response, f"job {index + 1} aceito")
    results = {context: core.event("queried", context, timeout=30) for context in contexts}
    check(all(result["success"] for result in results.values())
          and [results[c]["rows"][0][0] for c in contexts] == ["1", "2"],
          "dois jobs no mesmo perfil: cada um com o seu resultado e o seu contexto")

    # O SQLite integrado, do arquivo criado pela IDE ate' a leitura.
    created = core.rpc("datasource.create", {"kind": "sqliteFile", "name": "telemetria"})
    check("error" not in created, "SQLite: criar o arquivo")
    for sql in ["CREATE TABLE leituras (placa TEXT, valor INTEGER)",
                "INSERT INTO leituras VALUES ('pico', 1), ('esp32', 2)"]:
        check(core.query("telemetria", sql)["success"], "SQLite: " + sql.split()[0])
    rows = core.query("telemetria", "SELECT placa FROM leituras ORDER BY valor")["rows"]
    check(rows == [["pico"], ["esp32"]], "SQLite: ler o que escreveu")
    check((Path(project) / "data" / "telemetria.sqlite").is_file(), "SQLite: o arquivo mora no projeto")
    subprocess.run(["podman", "rm", "-f", "-v", name], capture_output=True, check=False)
