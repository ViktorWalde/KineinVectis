#!/usr/bin/env python3
"""Sonda e2e do terminal: D2.1 e D2.2 (docs/roadmaps/24).

Este arquivo existe porque as fatias D2.1 e D2.2 estao marcadas [FEITA]
citando uma sonda `sonda_terminal.py` que NUNCA foi commitada. Um item que
aponta para artefato ausente e mentira. Reescrita em 2026-08-30 para provar
EXATAMENTE as tres afirmacoes que aqueles itens fazem, contra o binario REAL
falando JSON-RPC por stdio:

  D2.1  "comando aparece no grid"   -> o que o shell imprime chega em
        `event.terminal.render` como linhas de spans, ja renderizadas;
  D2.1  "resize 100x30 reflete"     -> `terminal.resize` reflui o PTY E o
        emulador: o proximo render volta com cols=100 e rows=30, e o programa
        do outro lado enxerga a largura nova (`tput cols`);
  D2.2  "scroll traz o inicio do historico" -> `terminal.scroll` com o offset
        maximo mostra a PRIMEIRA linha da saida, que ja saiu da tela.

RELACAO COM A OUTRA SONDA. `sonda_scrollback.py` cobre o CLAMP do offset e o
multi-sessao (D2.3). Esta cobre o que os itens D2.1/D2.2 afirmam e que nao
estava provado em lugar nenhum: eco no grid, reflow do resize visto pelo
PROGRAMA, e o topo do historico.

ISOLAMENTO: `XDG_CONFIG_HOME` vai para um diretorio temporario — rodar a prova
nao pode sujar a lista de projetos recentes do autor.
"""
import json
import os
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORE = os.path.join(REPO, "target", "debug", "kinein-core")

fails = []


def check(nome, cond, detalhe=""):
    print(f"  {'ok  ' if cond else 'FALHOU'} {nome}" + (f"  [{detalhe}]" if detalhe else ""))
    if not cond:
        fails.append(nome)


class Core:
    def __init__(self, config_home):
        env = dict(os.environ, XDG_CONFIG_HOME=config_home)
        self.proc = subprocess.Popen(
            [CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, bufsize=1, env=env)
        self._id = 0
        self.inbox = queue.Queue()
        self.renders = []
        threading.Thread(target=self._reader, daemon=True).start()

    def _reader(self):
        # readline() bloqueia pra sempre com o core quieto (render so sai quando
        # ha mudanca), por isso a leitura vive numa thread.
        for line in self.proc.stdout:
            self.inbox.put(line)

    def send(self, method, params):
        self._id += 1
        self.proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": self._id, "method": method, "params": params}) + "\n")
        self.proc.stdin.flush()
        return self._id

    def pump(self, until_id=None, seconds=2.0):
        deadline = time.time() + seconds
        achado = None
        while time.time() < deadline:
            try:
                msg = json.loads(self.inbox.get(timeout=0.1))
            except queue.Empty:
                continue
            if msg.get("method") == "event.terminal.render":
                self.renders.append(msg["params"])
            if until_id is not None and msg.get("id") == until_id:
                achado = msg
                until_id = None  # segue drenando os renders que vierem atras
        return achado

    def last_render(self, tid):
        for r in reversed(self.renders):
            if r.get("id") == tid:
                return r
        return None

    def shutdown(self):
        self.send("core.shutdown", {})
        time.sleep(0.3)
        self.proc.kill()


def tela(render):
    """Texto visivel do grid, uma linha por linha do render.

    O `rstrip` por linha nao e' cosmetico: o emulador entrega a linha PREENCHIDA
    ate a largura do grid, entao `"\\n1\\n"` nunca casaria com uma linha que e'
    "1" seguido de 99 espacos. Comparar sem isso testa o padding, nao o
    conteudo.
    """
    return "\n".join(
        "".join(s["text"] for s in linha).rstrip() for linha in render["lines"])


def main():
    if not os.path.exists(CORE):
        print(f"binario ausente: {CORE}\nrode antes: cargo build -p kinein-core", file=sys.stderr)
        return 1

    config = tempfile.mkdtemp(prefix="kinein-sonda-config-")
    raiz = tempfile.mkdtemp(prefix="kinein-sonda-terminal-")
    core = None
    try:
        core = Core(config)
        core.pump(core.send("workspace.open", {"path": raiz}), 5)
        aberto = core.pump(core.send("terminal.open", {}), 3)
        if aberto is None or "result" not in aberto:
            print("terminal.open nao respondeu (sem PTY neste ambiente?)", file=sys.stderr)
            return 1
        tid = aberto["result"]["id"]
        check("terminal.open devolve um id", bool(tid), f"id={tid}")
        core.pump(core.send("terminal.resize", {"id": tid, "cols": 80, "rows": 24}), 2)

        print("== D2.1: o comando aparece no GRID ==")
        core.send("terminal.input", {"id": tid, "data": "echo COMANDO-NO-GRID\r"})
        core.pump(seconds=3.0)
        r = core.last_render(tid)
        check("chegou render para a sessao", r is not None)
        if r is None:
            return 1
        check("o render vem etiquetado com o id da sessao", r.get("id") == tid)
        check("a saida do comando esta no grid", "COMANDO-NO-GRID" in tela(r))
        check("o grid vem como linhas de spans (a UI so desenha)",
              isinstance(r["lines"], list) and all(isinstance(l, list) for l in r["lines"]))

        print("== D2.1: resize 100x30 reflui o PTY e o emulador ==")
        core.pump(core.send("terminal.resize", {"id": tid, "cols": 100, "rows": 30}), 3)
        core.pump(seconds=1.0)
        r = core.last_render(tid)
        check("o render volta com as colunas novas", r.get("cols") == 100, f"cols={r.get('cols')}")
        check("o render volta com as linhas novas", r.get("rows") == 30, f"rows={r.get('rows')}")
        check("o grid tem mesmo 30 linhas", len(r["lines"]) == 30, f"n={len(r['lines'])}")

        # Nao basta o RENDER dizer 100: o processo do outro lado do PTY tem que
        # enxergar a largura nova, senao o reflow seria so cosmetico no nosso
        # lado e um `less`/`vim` continuaria desenhando com 80.
        core.send("terminal.input", {"id": tid, "data": "tput cols\r"})
        core.pump(seconds=3.0)
        check("o PROGRAMA no PTY enxerga 100 colunas (tput cols)",
              "100" in tela(core.last_render(tid)))

        print("== D2.2: scroll traz o INICIO do historico ==")
        core.send("terminal.input", {"id": tid, "data": "clear; seq 1 400\r"})
        core.pump(seconds=4.0)
        r = core.last_render(tid)
        vivo = tela(r)
        smax = r.get("scrollbackMax", 0)
        check("ha historico acumulado", smax > 0, f"scrollbackMax={smax}")
        check("ao vivo mostra o FIM da saida", "400" in vivo)
        check("ao vivo NAO mostra o comeco", "\n1\n" not in vivo)

        core.pump(core.send("terminal.scroll", {"id": tid, "offset": smax}), 3)
        core.pump(seconds=1.0)
        r2 = core.last_render(tid)
        topo = tela(r2)
        check("a tela mudou ao rolar", topo != vivo)
        check("o topo do historico mostra o comeco da saida", "\n1\n" in topo or topo.startswith("1\n"),
              f"primeiras linhas={topo.splitlines()[:3]}")
        check("o scrollback ecoa onde a view esta", r2.get("scrollback") == smax,
              f"scrollback={r2.get('scrollback')} max={smax}")

        print()
        if fails:
            print(f"✗ FALHOU: {fails}")
            return 1
        print("✓ terminal e2e (D2.1 grid+resize, D2.2 topo do historico): tudo verde")
        return 0
    finally:
        if core is not None:
            core.shutdown()
        shutil.rmtree(raiz, ignore_errors=True)
        shutil.rmtree(config, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
