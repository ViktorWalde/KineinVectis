"""A pilha da prova: UI (Qt Quick, offscreen) -> supervisor (o papel do core)
-> nvim, com os dois canais de controle, e a árvore de processos lida do /proc."""

import json
import os
import shutil
import signal
import subprocess
import tempfile
import time

from controle import Canal

PROVA = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
UI_BIN = os.path.join(PROVA, "ui", "build", "kinein-prova-ui")
SUP_BIN = os.path.join(PROVA, "supervisor", "target", "release", "kinein-prova-supervisor")
RUNTIME = os.path.join(PROVA, "runtime")
NVIM = os.path.realpath(shutil.which("nvim") or "nvim")
TMP = os.path.join(PROVA, "tmp")


def ler_stat(pid):
    try:
        with open(f"/proc/{pid}/stat") as f:
            dados = f.read()
    except OSError:
        return None
    fim = dados.rindex(")")
    campos = dados[fim + 2:].split()
    return {
        "nome": dados[dados.index("(") + 1:fim],
        "estado": campos[0],
        "ppid": int(campos[1]),
        "utime": int(campos[11]),
        "stime": int(campos[12]),
    }


def ler_cmdline(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as f:
            return f.read().replace(b"\0", b" ").decode(errors="replace").strip()
    except OSError:
        return ""


def rss_kib(pid):
    try:
        with open(f"/proc/{pid}/status") as f:
            for linha in f:
                if linha.startswith("VmRSS:"):
                    return int(linha.split()[1])
    except OSError:
        return None
    return None


def processos():
    tabela = {}
    for nome in os.listdir("/proc"):
        if nome.isdigit():
            st = ler_stat(int(nome))
            if st:
                tabela[int(nome)] = st
    return tabela


def descendentes(raiz):
    tabela = processos()
    filhos = {}
    for pid, st in tabela.items():
        filhos.setdefault(st["ppid"], []).append(pid)
    saida, pilha = [], [raiz]
    while pilha:
        p = pilha.pop()
        for f in filhos.get(p, []):
            saida.append(f)
            pilha.append(f)
    return saida


def vivos(pids):
    """Os que ainda existem e não são zumbis."""
    resto = []
    for p in pids:
        st = ler_stat(p)
        if st and st["estado"] != "Z":
            resto.append(p)
    return resto


class Pilha:
    def __init__(self, projeto, *, nome="pilha", confiavel=True, env=None, abrir_canais=True, base=None):
        os.makedirs(TMP, exist_ok=True)
        # Reusar a base é reabrir a IDE depois de uma queda: o estado (e o
        # rascunho) é o mesmo.
        self.base = base or tempfile.mkdtemp(prefix=f"{nome}-", dir=TMP)
        self.projeto = os.path.realpath(projeto)
        self.estado = os.path.join(self.base, "estado")
        # Os sockets numa pasta curta (o sun_path tem 108 bytes), no lugar
        # que o produto usaria; uma pasta nova a cada subida, apagada no fim.
        self.run = tempfile.mkdtemp(prefix="kp-", dir=os.environ.get("XDG_RUNTIME_DIR") or "/tmp")
        self.ui_sock = os.path.join(self.run, "ui.sock")
        self.log_path = os.path.join(self.base, "ui.log")
        comando = [UI_BIN, "--controle", self.ui_sock, "--", SUP_BIN, "--projeto", self.projeto,
                   "--estado", self.estado, "--run", self.run, "--runtime", RUNTIME, "--nvim", NVIM]
        if not confiavel:
            comando.append("--nao-confiavel")
        ambiente = os.environ.copy()
        # Sempre offscreen: a prova nunca abre janela na sessão gráfica de quem
        # roda. Só quem pede outra plataforma (o Xvfb do teste de AT-SPI) muda.
        ambiente["QT_QPA_PLATFORM"] = "offscreen"
        # O log do Qt no ui.log da rodada: sem isto, com o stderr fora de um
        # terminal, o Qt escreve no journal do sistema.
        ambiente["QT_FORCE_STDERR_LOGGING"] = "1"
        ambiente.update(env or {})
        self.ambiente = ambiente
        self.comando = comando
        self.t0 = time.monotonic()
        self.log = open(self.log_path, "a")
        self.proc = subprocess.Popen(comando, env=ambiente, stdout=self.log, stderr=subprocess.STDOUT,
                                     cwd=self.base)
        self.ui = None
        self.sup = None
        if abrir_canais:
            self.ui = Canal(self.ui_sock)
            self.sup = Canal(os.path.join(self.run, "controle.sock"))

    def esperar_anexado(self, timeout_ms=20000):
        r = self.ui.pedir("esperar_anexado", timeout_ms=timeout_ms, timeout_s=timeout_ms / 1000 + 5)
        if not r["cumprido"]:
            raise RuntimeError("a UI não anexou ao nvim: " + json.dumps(r["estado"].get("eventos_supervisor"))[:2000])
        return r

    def pid_ui(self):
        return self.proc.pid

    def arvore(self):
        """{pid: (nome, cmdline)} de tudo que a UI criou."""
        return {p: (ler_stat(p) or {}).get("nome", "?") for p in descendentes(self.proc.pid)}

    def esperar_grade(self, texto, timeout_ms=10000, ausente=False):
        return self.ui.pedir("esperar_grade", contem=texto, ausente=ausente, timeout_ms=timeout_ms,
                             timeout_s=timeout_ms / 1000 + 5)

    def lua(self, codigo, *args):
        return self.sup.pedir("lua", code=codigo, args=list(args))

    def fechar(self):
        try:
            if self.ui:
                self.ui.pedir("sair", timeout_s=5)
        except Exception:
            pass
        try:
            self.proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait()
        for c in (self.ui, self.sup):
            if c:
                c.fechar()
        self.log.close()
        shutil.rmtree(self.run, ignore_errors=True)

    def matar(self, pid, sinal=signal.SIGKILL):
        os.kill(pid, sinal)
