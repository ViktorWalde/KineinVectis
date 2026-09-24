#!/usr/bin/env python3
"""Fluxo REAL de Remote SSH contra um sshd de verdade num container.

Por que existe (2026-09-24): todo o dominio `remote.*` era provado contra um
`ssh` FALSO — um script de shell que imprimia o que o teste queria ler. Isso
prova despacho e parser, e nao prova NADA sobre OpenSSH real: host key,
BatchMode, `ssh -G`, `ssh-copy-id`, rsync do outro lado. O autor pediu o fluxo
real; este arquivo e' ele.

O que roda de verdade, nesta ordem:

  remote.discover   le' o `~/.ssh/config` da HOME de teste e acha o alias
  remote.resolve    `ssh -G` real explica o alias
  remote.save       perfil com host/porta/usuario explicitos
  remote.probe      FALHA: sem chave, o sshd real recusa -> failure=authentication
  remote.command    kind=copyId compoe a linha do `ssh-copy-id`
  (a linha roda)    com a senha respondida num pty, como a IDE promete
  remote.probe      PASSA: arquitetura, kernel e ferramentas medidas no alvo
  remote.deploy     rsync de verdade para dentro do container
  remote.command    kind=shell compoe a linha do shell

LIMITE DITO. A conexao por ALIAS nao da' para isolar aqui: medido nesta data, o
OpenSSH nao honra `$HOME` para achar o `~/.ssh/config` (usa a base de senhas).
A descoberta e a resolucao ficam isoladas porque o core passa `-F` apontando o
mesmo arquivo; ja' `ssh`/`ssh-copy-id`/`rsync` compostos pelo core nao levam
`-F`, e nao vao levar so' para agradar um teste. Por isso as operacoes que
CONECTAM usam um perfil com host/porta/usuario explicitos — igualmente real.

Uso: scripts/testar-remote-ssh.sh (ele cuida do container e da HOME de teste).
"""
import json
import os
import pty
import queue
import subprocess
import sys
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORE = os.path.join(REPO, "target", "release", "kinein-core")

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


class Core:
    """O binario real falando JSON-RPC por stdio, com os eventos guardados."""

    def __init__(self, home, config_home):
        env = dict(os.environ, HOME=home, XDG_CONFIG_HOME=config_home)
        self.proc = subprocess.Popen(
            [CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, bufsize=1, env=env)
        self._id = 0
        self.inbox = queue.Queue()
        self.eventos = []
        threading.Thread(target=self._reader, daemon=True).start()

    def _reader(self):
        for linha in self.proc.stdout:
            self.inbox.put(linha)

    def rpc(self, metodo, params=None, seconds=20.0):
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

    def shutdown(self):
        try:
            self.rpc("core.shutdown", seconds=3)
        except Exception:
            pass
        self.proc.terminate()


def rodar_com_senha(linha, home, senha, seconds=45):
    """Roda a linha num pty e responde a senha — o caminho que a IDE promete.

    A IDE nunca digita senha: ela manda a linha ao TERMINAL e o `ssh` pergunta
    la'. Aqui o pty faz o papel da pessoa, para a prova ser automatica.
    """
    env = dict(os.environ, HOME=home)
    pid, fd = pty.fork()
    if pid == 0:
        os.execvpe("/bin/sh", ["/bin/sh", "-c", linha], env)
        os._exit(127)
    buf, enviado, prazo = b"", False, time.time() + seconds
    while time.time() < prazo:
        try:
            dados = os.read(fd, 4096)
        except OSError:
            break
        if not dados:
            break
        buf += dados
        baixo = buf.lower()
        if not enviado and (b"password:" in baixo or b"senha" in baixo):
            os.write(fd, senha.encode() + b"\n")
            enviado = True
    _, status = os.waitpid(pid, 0)
    return os.waitstatus_to_exitcode(status), buf.decode(errors="replace")


def main():
    home, porta, projeto = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    alias, usuario, senha = "kinein-lab", "kinein", "kinein-teste"
    core = Core(home, os.path.join(home, "config"))
    try:
        core.rpc("workspace.open", {"path": projeto})

        passo("descoberta e explicacao (le' config, nao conecta)")
        fora = core.rpc("remote.discover")
        nomes = [a["name"] for a in fora["aliases"]]
        check("remote.discover acha o alias do ~/.ssh/config", alias in nomes, str(nomes))
        origem = next((a["source"] for a in fora["aliases"] if a["name"] == alias), "")
        check("a origem e' dita", origem == "~/.ssh/config", origem)

        resolvido = core.rpc("remote.resolve", {"host": alias})
        check("ssh -G REAL resolve o alias",
              resolvido.get("hostName") == "127.0.0.1"
              and resolvido.get("port") == porta
              and resolvido.get("user") == usuario,
              json.dumps({k: resolvido.get(k) for k in ("hostName", "port", "user")}))
        check("nenhum ProxyCommand vazou", "proxyCommand" not in json.dumps(resolvido)
              or resolvido.get("proxyCommand") is False)

        passo("o alvo, e a sonda ANTES da chave")
        core.rpc("remote.save", {"target": {
            "name": alias, "host": "127.0.0.1", "user": usuario, "port": porta}})
        core.rpc("remote.probe", {"name": alias})
        ev = core.evento("event.remote.probed")
        check("o sshd REAL recusa sem chave", ev.get("success") is False)
        check("a causa tipada e' authentication", ev.get("failure") == "authentication",
              str(ev.get("failure")))
        check("a mensagem diz o gesto", "ssh-copy-id" in (ev.get("error") or ""),
              (ev.get("error") or "")[:70])

        passo("o gesto: copiar a chave, com a linha visivel antes")
        cmd = core.rpc("remote.command", {"name": alias, "kind": "copyId"})
        linha = cmd["command"]
        check("o core compos a linha com a porta do perfil",
              linha.startswith("ssh-copy-id") and f"-p {porta}" in linha
              and f"{usuario}@127.0.0.1" in linha, linha)
        check("a procedencia diz de quem e' a chave",
              any("SUA" in s for s in cmd["source"]))
        codigo, saida = rodar_com_senha(linha, home, senha)
        check("a linha RODOU e instalou a chave", codigo == 0, saida.strip()[-90:])

        passo("a sonda DEPOIS da chave: o que o alvo e'")
        core.rpc("remote.probe", {"name": alias})
        ev = core.evento("event.remote.probed")
        check("agora a sonda passa", ev.get("success") is True, str(ev.get("error"))[:70])
        check("sem falha quando passa", ev.get("failure") is None)
        check("mediu a arquitetura", bool(ev.get("arch")), str(ev.get("arch")))
        check("mediu o kernel", "Linux" in (ev.get("kernel") or ""), str(ev.get("kernel")))
        achadas = {t["id"]: t["found"] for t in ev.get("tools", [])}
        check("achou rsync e python3 no alvo",
              achadas.get("rsync") and achadas.get("python3"), str(achadas))
        check("e diz que gdbserver NAO esta' la'", achadas.get("gdbserver") is False,
              "o veredito precisa mostrar ferramenta faltando")

        passo("deploy real (rsync para dentro do container)")
        os.makedirs(os.path.join(projeto, "build"), exist_ok=True)
        with open(os.path.join(projeto, "build", "app"), "w") as f:
            f.write("#!/bin/sh\necho kinein\n")
        core.rpc("remote.deploy", {"name": alias})
        ev = core.evento("event.remote.deployed")
        check("o deploy chegou ao alvo", ev.get("success") is True, str(ev.get("error"))[:70])
        check("usou rsync", "rsync" in (ev.get("command") or ""), (ev.get("command") or "")[:70])

        passo("a linha do shell")
        shell = core.rpc("remote.command", {"name": alias, "kind": "shell"})
        check("o shell e' uma linha ssh para o alvo",
              shell["command"].startswith("ssh ")
              and f"{usuario}@127.0.0.1" in shell["command"], shell["command"])
    finally:
        core.shutdown()

    print()
    if falhas:
        print(f"✗ FLUXO REAL REPROVOU em {len(falhas)}: {', '.join(falhas)}")
        return 1
    print("✓ fluxo real de Remote SSH: descoberta, resolucao, sonda, ssh-copy-id,")
    print("  sonda de novo, deploy por rsync e shell — contra um sshd de verdade")
    return 0


if __name__ == "__main__":
    sys.exit(main())
