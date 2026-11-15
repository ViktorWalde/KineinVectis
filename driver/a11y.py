"""R10: o editor pela árvore de acessibilidade, como um leitor de tela o vê.

Roda num Xvfb próprio e numa sessão D-Bus própria (dbus-run-session), com o
barramento AT-SPI dela: nada toca a sessão gráfica nem o barramento de quem
roda. Lê pela libatspi (gi Atspi), a mesma que o Orca usa.

Uso (o a11y.sh sobe Xvfb e a sessão): python3 -I a11y.py
"""

import os
import signal
import subprocess
import sys
import tempfile
import threading

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import gi  # noqa: E402

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi  # noqa: E402

import fixtures  # noqa: E402
from desempenho import BASE_CORE, BASE_UI, casa_limpa  # noqa: E402
from pilha import TMP, Pilha, descendentes, vivos  # noqa: E402
from provas import abrir, novo_dir  # noqa: E402
from util import esperar, gravar  # noqa: E402

AMBIENTE_GRAFICO = {"QT_QPA_PLATFORM": "xcb", "DISPLAY": os.environ["DISPLAY"],
                    "QT_LINUX_ACCESSIBILITY_ALWAYS_ON": "1", "QT_ACCESSIBILITY": "1",
                    "QT_FORCE_STDERR_LOGGING": "1"}


def aplicacoes():
    desk = Atspi.get_desktop(0)
    saida = []
    for i in range(desk.get_child_count()):
        app = desk.get_child_at_index(i)
        if app is not None:
            saida.append(app)
    return saida


def aplicacao(pid, r, timeout_s=20):
    """Pela pid, não pelo nome: o nome que a aplicação publica é dela. A
    mesma pid pode ter mais de um registro: o tema de plataforma GTK do Qt
    (libqgtk3) carrega a libatk-bridge, que publica uma aplicação sem janela.
    Vale o registro com janela, que é o do Qt."""
    def da_pid():
        saida = []
        for app in aplicacoes():
            try:
                if app.get_process_id() == pid:
                    saida.append(app)
            except Exception:
                pass
        return saida

    def achar():
        com_janela = [a for a in da_pid() if a.get_child_count() > 0]
        return com_janela[0] if com_janela else None
    try:
        app = esperar(achar, timeout_s, intervalo_s=0.2, descricao=f"pid {pid} com janela no barramento AT-SPI")
    except TimeoutError as e:
        vistas = [(a.get_name(), a.get_process_id(), a.get_child_count()) for a in aplicacoes()]
        raise TimeoutError(f"{e}; no barramento: {vistas}") from None
    r["registros_da_pid"] = [(a.get_name(), a.get_child_count()) for a in da_pid()]
    return app


def despejar(no, profundidade=0, saida=None, limite=12):
    """A árvore como a AT-SPI a mostra: papel, nome e estados que importam."""
    saida = [] if saida is None else saida
    if no is None or profundidade > limite or len(saida) > 300:
        return saida
    try:
        estados = no.get_state_set()
        marcas = [n for n, s in (("foco", Atspi.StateType.FOCUSED), ("focável", Atspi.StateType.FOCUSABLE),
                                 ("editável", Atspi.StateType.EDITABLE)) if estados.contains(s)]
        texto = ""
        if "Text" in (no.get_interfaces() or []):
            texto = Atspi.Text.get_text(no, 0, 60)
        saida.append(f"{'  ' * profundidade}{no.get_role_name()} {no.get_name()!r} {marcas} {texto!r}")
        for i in range(no.get_child_count()):
            despejar(no.get_child_at_index(i), profundidade + 1, saida, limite)
    except Exception as e:
        saida.append(f"{'  ' * profundidade}<erro {type(e).__name__}: {e}>")
    return saida


def editaveis(no, profundidade=0, saida=None):
    saida = [] if saida is None else saida
    if no is None or profundidade > 25:
        return saida
    try:
        papel = no.get_role()
    except Exception:
        return saida
    if papel in (Atspi.Role.TEXT, Atspi.Role.EDITBAR, Atspi.Role.ENTRY, Atspi.Role.PARAGRAPH,
                 Atspi.Role.DOCUMENT_TEXT) or (no.get_state_set().contains(Atspi.StateType.EDITABLE)):
        saida.append(no)
    for i in range(no.get_child_count()):
        editaveis(no.get_child_at_index(i), profundidade + 1, saida)
    return saida


def ler(no):
    t = Atspi.Text.get_text(no, 0, -1)
    return {"papel": no.get_role_name(), "nome": no.get_name(), "texto": t[:200],
            "caracteres": Atspi.Text.get_character_count(no), "cursor": Atspi.Text.get_caret_offset(no),
            "focado": no.get_state_set().contains(Atspi.StateType.FOCUSED),
            "editavel": no.get_state_set().contains(Atspi.StateType.EDITABLE)}


def achar_editor(app, r, filtro=lambda no: True):
    """O editor na árvore; sem ele, a árvore vista fica no resultado."""
    try:
        return esperar(lambda: [n for n in editaveis(app) if filtro(n)], 10, intervalo_s=0.3,
                       descricao="editor na árvore")
    except TimeoutError:
        r["arvore_vista"] = despejar(app)
        raise


def caso_prova(r):
    d = novo_dir("a11y")
    arq = fixtures.escrever(d, "a.txt", "primeira linha\nsegunda linha\n")
    p = Pilha(d, nome="a11y", env=AMBIENTE_GRAFICO)
    try:
        p.esperar_anexado()
        abrir(p, arq)
        p.ui.pedir("foco")
        app = aplicacao(p.pid_ui(), r)
        nos = achar_editor(app, r)
        r["editaveis_achados"] = len(nos)
        no = nos[0]
        r["antes"] = ler(no)
        p.ui.pedir("digitar", texto="Olá ")
        r["depois_de_digitar"] = esperar(lambda: (lambda x: x if x["texto"].startswith("Olá ") else None)(ler(no)), 5,
                                          descricao="texto novo pela AT-SPI")
        p.ui.pedir("teclas", seq=["Down"])
        r["depois_da_seta"] = esperar(lambda: (lambda x: x if x["texto"].startswith("segunda") else None)(ler(no)), 5,
                                      descricao="linha nova pela AT-SPI")
    finally:
        p.fechar()
    ok = r.get("antes", {}).get("editavel") and r.get("depois_de_digitar", {}).get("cursor") == 4 and \
        "depois_da_seta" in r
    r["veredito"] = "CAMINHO VIÁVEL" if ok else "FALHOU"
    r["limite"] = "a interface de texto expõe só a linha do cursor, não o documento inteiro"


def caso_base(r):
    ws = tempfile.mkdtemp(prefix="a11y-base-", dir=TMP)
    caminho, _ = fixtures.fixture_desempenho(ws, 2)
    env = os.environ.copy()
    env.update(casa_limpa())
    env.update(AMBIENTE_GRAFICO)
    # Zero teclas: o harness abre o arquivo e para. A AT-SPI só lê o documento
    # (sem gerenciador de janelas, a janela xcb não ganha foco para as teclas).
    env.update({"KINEIN_CORE_BIN": BASE_CORE, "KINEIN_PERF_TYPING": "1", "KINEIN_PERF_TYPING_WORKSPACE": ws,
                "KINEIN_PERF_TYPING_FILE": caminho, "KINEIN_PERF_TYPING_KEYS": "0", "KINEIN_PERF_HOLD": "1"})
    proc = subprocess.Popen([BASE_UI], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                            cwd=ws)
    pronto = threading.Event()

    def drenar():
        # O log vai inteiro para o arquivo, também depois do hold: um pipe
        # cheio travaria a UI enquanto a AT-SPI pergunta a ela.
        with open(os.path.join(ws, "base-ui.log"), "w") as log:
            for linha in proc.stdout:
                log.write(linha)
                if "typing_hold=1" in linha or "typing_error" in linha:
                    r["harness"] = linha.strip()
                    pronto.set()
        pronto.set()

    threading.Thread(target=drenar, daemon=True).start()
    try:
        pronto.wait(120)
        if "typing_hold=1" not in r.get("harness", ""):
            raise RuntimeError(f"o harness do produto não chegou ao hold: {r.get('harness')}")
        app = aplicacao(proc.pid, r)
        nos = achar_editor(app, r, lambda n: Atspi.Text.get_character_count(n) > 1000)
        r["editaveis_achados"] = len(nos)
        r["editor"] = ler(nos[0])
    finally:
        filhos = descendentes(proc.pid)
        proc.send_signal(signal.SIGTERM)
        try:
            proc.wait(10)
        except subprocess.TimeoutExpired:
            proc.kill()
        for pid in vivos(filhos):
            os.kill(pid, signal.SIGKILL)
    r["veredito"] = "EXPÕE O DOCUMENTO" if r.get("editor", {}).get("caracteres", 0) > 1000 else "FALHOU"


if __name__ == "__main__":
    resultado = {}
    for nome, f in (("prova", caso_prova), ("base", caso_base)):
        resultado[nome] = r = {}
        try:
            f(r)
        except Exception as e:  # o resultado diz o que falhou, com o que se viu até ali
            r.update({"veredito": "ERRO", "erro": f"{type(e).__name__}: {e}"[:1500]})
        print(nome, resultado[nome].get("veredito"), resultado[nome].get("erro", ""), flush=True)
    gravar("a11y", resultado)
