#!/usr/bin/env python3
"""Sonda e2e da rede de seguranca de dados (docs/seguranca/23, pilar 2).

Este arquivo existe porque o item P2.6 daquele documento dizia "[x] validado
e2e" citando uma sonda que NUNCA foi commitada. Um `[x]` que aponta para
artefato ausente e mentira: ninguem conseguia re-rodar a prova do pilar 2.
Reescrita em 2026-08-30 para provar EXATAMENTE o que o item afirma, contra o
binario REAL falando JSON-RPC por stdio:

  1. crash com SIGKILL (sem shutdown limpo) -> o rascunho sobrevive e volta no
     `workspace.open` seguinte, com o conteudo NAO salvo;
  2. salvar o arquivo (`fs.write`) LIMPA o rascunho — senao a proxima abertura
     "recuperaria" um buffer identico ao disco e marcaria a aba como
     modificada sem motivo;
  3. a escrita e ATOMICA: nenhum `.kinein-tmp-*` fica para tras no diretorio;
  4. rascunho de arquivo APAGADO e descartado, nao recuperado (recuperar
     reabriria a aba de um arquivo que nao existe mais).

ISOLAMENTO. O binario real liga persistencia GLOBAL
(`Core::enable_persistence`), que grava a lista de projetos recentes em
`$XDG_CONFIG_HOME/kinein-vectis`. A sonda redireciona essa variavel para um
diretorio temporario: rodar a prova NAO pode sujar os dados do autor. Foi
exatamente esse tipo de vazamento que, antes de 2026-08-29, fez a suite de
testes apagar a lista de recentes.
"""
import json
import os
import sqlite3
import queue
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORE = os.path.join(REPO, "target", "debug", "kinein-core")

ARQUIVO = "src/main.rs"
NO_DISCO = "fn main() {\n    println!(\"salvo\");\n}\n"
NAO_SALVO = "fn main() {\n    println!(\"NAO SALVO — so existe no buffer\");\n}\n"

fails = []


def check(nome, cond, detalhe=""):
    print(f"  {'ok  ' if cond else 'FALHOU'} {nome}" + (f"  [{detalhe}]" if detalhe else ""))
    if not cond:
        fails.append(nome)


class Core:
    """Um processo do core, falando JSON-RPC por stdio."""

    def __init__(self, config_home):
        env = dict(os.environ, XDG_CONFIG_HOME=config_home)
        self.proc = subprocess.Popen(
            [CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, bufsize=1, env=env)
        self._id = 0
        self.inbox = queue.Queue()
        threading.Thread(target=self._reader, daemon=True).start()

    def _reader(self):
        for line in self.proc.stdout:
            self.inbox.put(line)

    def call(self, method, params, seconds=5.0):
        """Envia e espera a RESPOSTA daquele id, ignorando eventos no meio."""
        self._id += 1
        pedido = self._id
        self.proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": pedido, "method": method, "params": params}) + "\n")
        self.proc.stdin.flush()
        deadline = time.time() + seconds
        while time.time() < deadline:
            try:
                msg = json.loads(self.inbox.get(timeout=0.1))
            except queue.Empty:
                continue
            if msg.get("id") == pedido:
                return msg
        return None

    def crash(self):
        """SIGKILL: o core morre SEM shutdown limpo, como num crash de verdade."""
        self.proc.send_signal(signal.SIGKILL)
        self.proc.wait(timeout=5)

    def shutdown(self):
        self.call("core.shutdown", {}, 2.0)
        time.sleep(0.2)
        self.proc.kill()


def temps_soltos(diretorio):
    return [n for n in os.listdir(diretorio) if ".kinein-tmp-" in n]


def linhas_na_store(raiz, caminho):
    """Conta as linhas da store SQLite para um caminho.

    Olhar a STORE, e nao a resposta do `workspace.open`, e' deliberado. O
    `recover_drafts` ja descarta rascunho identico ao disco, entao depois de um
    save o rascunho orfao fica INVISIVEL na resposta mesmo continuando gravado.
    Uma sonda que so olhasse a resposta ficaria verde com o
    `clear_draft_after_save` destruido — medido em 2026-08-30, e foi o teste de
    mutacao que mostrou.
    """
    db = os.path.join(raiz, ".kinein", "kinein.db")
    if not os.path.exists(db):
        return 0
    con = sqlite3.connect(db)
    try:
        return con.execute(
            "SELECT COUNT(*) FROM drafts WHERE path = ?", (caminho,)).fetchone()[0]
    finally:
        con.close()


def main():
    if not os.path.exists(CORE):
        print(f"binario ausente: {CORE}\nrode antes: cargo build -p kinein-core", file=sys.stderr)
        return 1

    raiz = tempfile.mkdtemp(prefix="kinein-sonda-drafts-")
    config = tempfile.mkdtemp(prefix="kinein-sonda-config-")
    try:
        os.makedirs(os.path.join(raiz, "src"))
        with open(os.path.join(raiz, "Cargo.toml"), "w") as f:
            f.write('[package]\nname = "sonda"\nversion = "0.1.0"\nedition = "2021"\n')
        alvo = os.path.join(raiz, ARQUIVO)
        with open(alvo, "w") as f:
            f.write(NO_DISCO)

        print("== 1. autosave + CRASH (SIGKILL, sem shutdown) ==")
        core = Core(config)
        aberto = core.call("workspace.open", {"path": raiz})
        check("workspace.open respondeu", aberto is not None and "result" in aberto)
        if aberto is None or "result" not in aberto:
            return 1
        check("projeto novo abre SEM rascunho pendente",
              aberto["result"].get("drafts", []) == [],
              f"drafts={aberto['result'].get('drafts')}")

        salvo = core.call("draft.save", {"path": alvo, "content": NAO_SALVO})
        check("draft.save aceito", salvo is not None and "result" in salvo)
        core.crash()

        print("== 2. o rascunho tem que SOBREVIVER ao crash ==")
        core = Core(config)
        aberto = core.call("workspace.open", {"path": raiz})
        drafts = aberto["result"].get("drafts", []) if aberto and "result" in aberto else []
        check("workspace.open devolve o rascunho recuperado", len(drafts) == 1,
              f"n={len(drafts)}")
        if drafts:
            check("o rascunho aponta para o arquivo certo", drafts[0].get("path") == alvo)
            check("o conteudo recuperado e o NAO SALVO",
                  drafts[0].get("content") == NAO_SALVO)
        check("o disco continua com o conteudo ANTIGO (nada foi escrito sozinho)",
              open(alvo).read() == NO_DISCO)

        print("== 3. salvar LIMPA o rascunho, e a escrita e atomica ==")
        # `fs.write` e' a escrita CONFLICT-SAFE do ADR-0001: exige o ultimo
        # conteudo que o editor viu no disco (`expectedContent`) e recusa se o
        # disco mudou por baixo. Nao existe caminho de escrita cega.
        escrito = core.call("fs.write", {
            "path": alvo, "content": NAO_SALVO, "expectedContent": NO_DISCO})
        check("fs.write aceito", escrito is not None and "result" in escrito,
              "" if escrito and "result" in escrito else str(escrito))
        check("o disco agora tem o conteudo do buffer", open(alvo).read() == NAO_SALVO)
        sobras = temps_soltos(os.path.join(raiz, "src"))
        check("nenhum .kinein-tmp-* ficou para tras", sobras == [], f"sobras={sobras}")

        recusado = core.call("fs.write", {
            "path": alvo, "content": "sobrescrita cega\n",
            "expectedContent": "conteudo que o disco NAO tem mais\n"})
        check("a barreira compare-before-save RECUSA disco divergente",
              recusado is not None and "error" in recusado)
        check("e o disco fica intacto depois da recusa", open(alvo).read() == NAO_SALVO)

        # A checagem tem que ser AQUI, no mesmo processo e antes de qualquer
        # reabertura: o `recover_drafts` limpa sozinho o rascunho identico ao
        # disco, entao olhar a store depois de reabrir esconderia um
        # `clear_draft_after_save` quebrado. Medido por mutacao em 2026-08-30 —
        # a primeira versao desta sonda ficou VERDE com ele destruido.
        na_store = linhas_na_store(raiz, alvo)
        check("salvar limpou a linha na store, NA HORA", na_store == 0,
              f"linhas={na_store}")
        core.crash()

        core = Core(config)
        aberto = core.call("workspace.open", {"path": raiz})
        drafts = aberto["result"].get("drafts", []) if aberto and "result" in aberto else []
        check("depois de salvar, nao ha mais rascunho a recuperar", drafts == [],
              f"drafts={drafts}")

        print("== 4. rascunho de arquivo APAGADO e descartado ==")
        core.call("draft.save", {"path": alvo, "content": "buffer de um arquivo que vai sumir\n"})
        core.crash()
        os.remove(alvo)

        core = Core(config)
        aberto = core.call("workspace.open", {"path": raiz})
        drafts = aberto["result"].get("drafts", []) if aberto and "result" in aberto else []
        check("rascunho de arquivo inexistente NAO e recuperado", drafts == [],
              f"drafts={drafts}")
        core.shutdown()

        print()
        if fails:
            print(f"✗ FALHOU: {fails}")
            return 1
        print("✓ rede de seguranca (pilar 2, docs/seguranca/23 P2.6): tudo verde")
        return 0
    finally:
        shutil.rmtree(raiz, ignore_errors=True)
        shutil.rmtree(config, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
