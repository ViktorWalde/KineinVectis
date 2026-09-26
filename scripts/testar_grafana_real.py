#!/usr/bin/env python3
"""Prova do dominio `grafana.*` contra um Grafana DE VERDADE.

Por que existe (2026-09-26): a fatia V7 fechou desenho, regras, fiacao e
seguranca do painel de observabilidade, tudo medido por harness. Nada disso
prova a conversa com um Grafana real — status HTTP, corpo do `/api/health`,
o 401 de uma conta de servico ausente, a forma do `/api/datasources`. A §10
da `especificacoes/grafana-ui-ux-0.3.5.md` faz do "teste real com URL
invalida, sem token, token invalido e valido" um item da entrega, e este
arquivo e' ele.

Os quatro casos, nesta ordem:

  URL invalida     porta onde nao ha' ninguem -> falha util, sem inventar dado
  sem token        `/api/health` responde: alcancou, NAO autenticou
  token invalido   o servidor recusa -> SECRET_REQUIRED, e nao "deu certo"
  token valido     versao, fontes, dashboards e O CRUZAMENTO com o banco daqui

O CRUZAMENTO E' O PONTO. Um link para o Grafana e' um favorito; o que so' a IDE
sabe e' que o `kinein_prova` do projeto e' a fonte `banco-do-projeto` la'.
Por isso o teste sobe TAMBEM um Postgres e cadastra a fonte dos dois lados.

E uma quinta prova, que e' de seguranca e nao de funcionalidade: o token nao
aparece no que o core escreve — nem em stdout, nem em stderr.

Uso: scripts/testar-grafana-real.sh (ele cuida dos containers).
"""
import json
import os
import queue
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORE = os.path.join(REPO, "target", "release", "kinein-core")

GRAFANA = os.environ.get("KINEIN_GRAFANA_URL", "http://localhost:3000")
PG_HOST = os.environ.get("KINEIN_PG_HOST", "localhost")
PG_PORT = int(os.environ.get("KINEIN_PG_PORT", "5432"))
PG_BANCO = os.environ.get("KINEIN_PG_BANCO", "kinein_prova")
# Uma porta onde nao ha' ninguem escutando — o caso "URL invalida" precisa ser
# uma recusa RAPIDA e nao um timeout de rede.
URL_MORTA = os.environ.get("KINEIN_GRAFANA_URL_MORTA", "http://127.0.0.1:1")

falhas = []


def passo(titulo):
    """Cada etapa se anuncia ANTES de rodar: um teste que trava sem dizer onde
    travou custa mais caro que um que falha."""
    print(f"\n== {titulo} ==", flush=True)


def check(nome, cond, detalhe=""):
    print(f"  {'ok    ' if cond else 'FALHOU'} {nome}" + (f"   [{detalhe}]" if detalhe else ""),
          flush=True)
    if not cond:
        falhas.append(nome)


def http(caminho, metodo="GET", corpo=None, usuario=None, token=None, seconds=10.0):
    """Fala com o Grafana pela API dele, que e' o mesmo caminho do core."""
    pedido = urllib.request.Request(GRAFANA + caminho, method=metodo)
    if corpo is not None:
        pedido.add_header("Content-Type", "application/json")
        pedido.data = json.dumps(corpo).encode("utf-8")
    if usuario is not None:
        import base64
        cru = base64.b64encode(usuario.encode("utf-8")).decode("ascii")
        pedido.add_header("Authorization", "Basic " + cru)
    if token is not None:
        pedido.add_header("Authorization", "Bearer " + token)
    with urllib.request.urlopen(pedido, timeout=seconds) as resposta:
        return json.loads(resposta.read().decode("utf-8") or "{}")


class Core:
    """O binario real falando JSON-RPC por stdio, com os eventos guardados.

    O stderr e' CAPTURADO, e nao descartado: parte da prova e' que o token nao
    aparece nele.
    """

    def __init__(self, home):
        env = dict(os.environ, HOME=home, XDG_CONFIG_HOME=os.path.join(home, ".config"))
        self.proc = subprocess.Popen(
            [CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, bufsize=1, env=env)
        self._id = 0
        self.inbox = queue.Queue()
        self.eventos = []
        self.saida_crua = []
        self.erro_cru = []
        threading.Thread(target=self._reader, daemon=True).start()
        threading.Thread(target=self._stderr, daemon=True).start()

    def _reader(self):
        for linha in self.proc.stdout:
            self.saida_crua.append(linha)
            self.inbox.put(linha)

    def _stderr(self):
        for linha in self.proc.stderr:
            self.erro_cru.append(linha)

    def rpc(self, metodo, params=None, seconds=30.0, tolerar_erro=False):
        self._id += 1
        meu = self._id
        self.proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": meu, "method": metodo,
             "params": params if params is not None else {}}) + "\n")
        self.proc.stdin.flush()
        prazo = time.time() + seconds
        while time.time() < prazo:
            try:
                msg = json.loads(self.inbox.get(timeout=0.1))
            except queue.Empty:
                continue
            if msg.get("method", "").startswith("event."):
                self.eventos.append(msg)
            if msg.get("id") == meu:
                if "error" in msg:
                    if tolerar_erro:
                        return {"__erro__": msg["error"]}
                    raise AssertionError(f"{metodo}: {msg['error']}")
                return msg.get("result", {})
        raise AssertionError(f"{metodo}: sem resposta em {seconds}s")

    def evento(self, nome, seconds=60.0):
        prazo = time.time() + seconds
        while time.time() < prazo:
            for e in self.eventos:
                if e.get("method") == nome:
                    self.eventos.remove(e)
                    return e.get("params", {})
            try:
                msg = json.loads(self.inbox.get(timeout=0.1))
            except queue.Empty:
                continue
            if msg.get("method", "").startswith("event."):
                self.eventos.append(msg)
        raise AssertionError(f"{nome} nao chegou em {seconds}s")

    def sondar(self, token=None, seconds=60.0):
        """`grafana.probe` e a espera do evento — o par que a UI usa.

        Devolve `("evento", params)` quando a sonda foi aceita e respondeu, ou
        `("erro", erro)` quando o core recusou na hora. Os dois sao respostas
        legitimas, e distingui-las e' metade do que esta prova mede.
        """
        params = {} if token is None else {"token": token}
        resposta = self.rpc("grafana.probe", params, tolerar_erro=True)
        if "__erro__" in resposta:
            return ("erro", resposta["__erro__"])
        return ("evento", self.evento("event.grafana.probed", seconds=seconds))

    def shutdown(self):
        try:
            self.rpc("core.shutdown", seconds=3)
        except Exception:
            pass
        self.proc.terminate()


def esperar_grafana(seconds=90.0):
    """O container sobe antes de a API responder; esperar o /api/health e' o
    unico jeito honesto de saber quando comecar."""
    prazo = time.time() + seconds
    ultimo = ""
    while time.time() < prazo:
        try:
            saude = http("/api/health", seconds=3)
            if saude.get("database"):
                return saude
        except Exception as erro:  # noqa: BLE001 — qualquer falha aqui e' "ainda nao"
            ultimo = str(erro)
        time.sleep(1.0)
    raise AssertionError(f"o Grafana nao respondeu em {seconds}s ({ultimo})")


def criar_token(nome):
    """Conta de servico + token, pela API, como um usuario faria.

    O Grafana 11 aposentou as API keys em favor de contas de servico; usar o
    caminho aposentado provaria o passado.
    """
    conta = http("/api/serviceaccounts", "POST",
                 {"name": nome, "role": "Admin", "isDisabled": False},
                 usuario="admin:admin")
    criado = http(f"/api/serviceaccounts/{conta['id']}/tokens", "POST",
                  {"name": nome + "-token"}, usuario="admin:admin")
    return criado["key"]


def cadastrar_fonte(token):
    """A fonte de dados do lado do GRAFANA, apontando para o mesmo Postgres.

    O cruzamento compara nome do banco E host (porta, quando declarada). Por
    isso os dois lados precisam dizer a mesma coisa — e e' exatamente isso que
    a tela promete ao autor.
    """
    return http("/api/datasources", "POST", {
        "name": "banco-do-projeto",
        "type": "grafana-postgresql-datasource",
        "access": "proxy",
        "url": f"{PG_HOST}:{PG_PORT}",
        "database": PG_BANCO,
        "user": "kinein",
        "jsonData": {"sslmode": "disable", "postgresVersion": 1600},
        "secureJsonData": {"password": "kinein"},
    }, token=token)


def main():
    if not os.path.exists(CORE):
        print("erro: falta target/release/kinein-core", file=sys.stderr)
        return 2

    projeto = os.environ["KINEIN_PROJETO_TESTE"]
    casa = os.environ["KINEIN_HOME_TESTE"]

    passo("o Grafana de verdade responde")
    saude = esperar_grafana()
    check("/api/health respondeu", bool(saude.get("version")),
          f"versao {saude.get('version')}, banco {saude.get('database')}")

    passo("conta de servico e fonte de dados, pela API")
    token = criar_token("kinein-prova")
    check("token de conta de servico criado", token.startswith("glsa_"),
          "prefixo glsa_ (conta de servico, nao API key aposentada)")
    fonte = cadastrar_fonte(token)
    check("fonte de dados cadastrada no Grafana", fonte.get("id") is not None,
          f"{PG_HOST}:{PG_PORT}/{PG_BANCO}")

    core = Core(casa)
    try:
        passo("o workspace e o perfil de banco DESTE projeto")
        core.rpc("workspace.open", {"path": projeto})
        core.rpc("datasource.save", {"profile": {
            "name": "banco-do-projeto",
            "engine": "postgres",
            "host": PG_HOST,
            "port": PG_PORT,
            "database": PG_BANCO,
            "user": "kinein",
            "secretSource": "prompt",
        }})
        perfis = core.rpc("datasource.list")
        check("o projeto tem um banco cadastrado", len(perfis.get("profiles", [])) == 1)

        passo("caso 1 — URL invalida")
        core.rpc("grafana.save", {"profile": {"url": URL_MORTA, "tokenSource": "none"}})
        forma, resultado = core.sondar()
        if forma == "erro":
            check("recusou com motivo, sem inventar dado", True,
                  str(resultado.get("message", ""))[:60])
        else:
            check("nao alcancou", resultado.get("reachable") is False,
                  str(resultado.get("message", ""))[:60])
            check("nao disse estar autenticado", resultado.get("authenticated") is not True)
            check("nao inventou versao", not resultado.get("version"))

        passo("caso 2 — sem token")
        core.rpc("grafana.save", {"profile": {"url": GRAFANA, "tokenSource": "none"}})
        forma, resultado = core.sondar()
        check("a sonda respondeu por evento", forma == "evento", forma)
        if forma == "evento":
            check("alcancou o Grafana", resultado.get("reachable") is True)
            check("NAO autenticou", resultado.get("authenticated") is not True)
            # SEM TOKEN NAO E' RECUSA: ninguem ofereceu nada para ser negado.
            check("e isso nao e' recusa", resultado.get("authRefused") is not True)
            check("mesmo assim mediu a versao", bool(resultado.get("version")),
                  str(resultado.get("version")))
            check("sem token, a API nao lista fontes",
                  len(resultado.get("dataSources") or []) == 0)

        passo("caso 3 — token invalido")
        forma, resultado = core.sondar(token="glsa_token_que_nao_existe_000000000000")
        check("a sonda respondeu por evento", forma == "evento", forma)
        if forma == "evento":
            check("nao se declarou autenticado", resultado.get("authenticated") is not True,
                  str(resultado.get("message", ""))[:60])
            check("nao trouxe fonte nenhuma", len(resultado.get("dataSources") or []) == 0)
            # A DIFERENCA QUE ESTE TESTE DESCOBRIU (2026-09-26): token recusado
            # NAO vira `SECRET_REQUIRED`. Ele chega como sonda bem-sucedida com
            # `authenticated: false` — que e' tambem o que se ve quando ninguem
            # ofereceu token. Sem um campo proprio, a tela dizia "sem
            # autenticacao" a quem acabara de colar a credencial errada, e nao
            # oferecia caminho de volta. Dai' o `authRefused` da 0.136.0.
            check("o core DIZ que foi recusa, e nao ausencia",
                  resultado.get("authRefused") is True,
                  "authRefused")

        passo("caso 4 — token valido, e O CRUZAMENTO")
        forma, resultado = core.sondar(token=token)
        check("a sonda respondeu por evento", forma == "evento", forma)
        if forma == "evento":
            check("alcancou", resultado.get("reachable") is True)
            check("autenticou", resultado.get("authenticated") is True)
            check("e nao marcou recusa", resultado.get("authRefused") is not True)
            fontes = resultado.get("dataSources") or []
            check("listou a fonte de dados", len(fontes) >= 1,
                  ", ".join(f.get("name", "?") for f in fontes))
            casamentos = resultado.get("matches") or []
            check("CRUZOU o banco do projeto com a fonte do Grafana",
                  len(casamentos) >= 1,
                  "; ".join(f"{c.get('profileName')} -> {c.get('dataSourceName')}"
                            f" ({c.get('reason')})" for c in casamentos))

        passo("prova de seguranca — o token nao vaza pelo que o core escreve")
        saida = "".join(core.saida_crua)
        erro = "".join(core.erro_cru)
        check("o token nao aparece no stdout do core", token not in saida)
        check("o token nao aparece no stderr do core", token not in erro)
        check("o token nao foi gravado no perfil do workspace",
              token not in _ler_tudo(os.path.join(projeto, ".kinein")))
    finally:
        core.shutdown()

    print()
    if falhas:
        print(f"✗ {len(falhas)} falha(s): " + ", ".join(falhas), file=sys.stderr)
        return 1
    print("✓ grafana real: os quatro casos e o cruzamento, contra uma instancia de verdade")
    return 0


def _ler_tudo(raiz):
    """Todo o conteudo de texto sob uma pasta, para procurar o que nao pode
    estar la'."""
    pedacos = []
    for pasta, _, arquivos in os.walk(raiz):
        for arquivo in arquivos:
            try:
                with open(os.path.join(pasta, arquivo), "r", encoding="utf-8",
                          errors="replace") as origem:
                    pedacos.append(origem.read())
            except OSError:
                continue
    return "".join(pedacos)


if __name__ == "__main__":
    sys.exit(main())
