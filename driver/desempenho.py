"""Item 15 (R1, R2, R12 e CPU): a prova contra o editor atual, pelo mesmo
método do A3.3-1 (medir-performance.sh): a mesma fixture do medir-core.py,
40 teclas na linha do meio, coluna 1, 150 ms de quieto entre elas, offscreen,
janela 1280x800.

A linha de base é o produto no código do 308c1561 (base/arvore), com o
harness instrumentado pelo base/instrumentacao.patch: KINEIN_PERF_OPEN_PROBE
(R2), KINEIN_PERF_HOLD (o processo fica de pé para ler RSS e CPU).

Cada rodada, nos dois lados, roda com HOME e XDG_* próprios, o
XDG_RUNTIME_DIR inclusive (nada lê nem grava a configuração nem os sockets
de quem roda); só o XDG_CACHE_HOME é compartilhado, para o cache de fontes
não pesar diferente entre rodadas.

Uso: python3 -I desempenho.py [rodadas] [funcoes ...]
"""

import os
import signal
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import fixtures  # noqa: E402
from pilha import PROVA, TMP, Pilha, descendentes, ler_cmdline, ler_stat, rss_kib, runtime_privado, vivos  # noqa: E402
from util import esperar, gravar, resumo  # noqa: E402

BASE_UI = os.path.join(PROVA, "base", "arvore", "build", "release", "ui", "kinein-vectis")
BASE_CORE = os.path.join(PROVA, "base", "arvore", "target", "release", "kinein-core")
TICK = os.sysconf("SC_CLK_TCK")
CACHE = os.path.join(TMP, "desempenho-cache")


def casa_limpa():
    casa = tempfile.mkdtemp(prefix="casa-perf-", dir=TMP)
    os.makedirs(CACHE, exist_ok=True)
    return {
        "HOME": casa,
        "XDG_CONFIG_HOME": os.path.join(casa, ".config"),
        "XDG_DATA_HOME": os.path.join(casa, ".local", "share"),
        "XDG_STATE_HOME": os.path.join(casa, ".local", "state"),
        "XDG_CACHE_HOME": CACHE,
        "XDG_RUNTIME_DIR": runtime_privado(),
    }


def pss_kib(pid):
    try:
        with open(f"/proc/{pid}/smaps_rollup") as f:
            for linha in f:
                if linha.startswith("Pss:"):
                    return int(linha.split()[1])
    except OSError:
        return None
    return None


def arvore(raiz):
    return [raiz] + descendentes(raiz)


def cpu_s(pids):
    total = 0
    for pid in pids:
        st = ler_stat(pid)
        if st:
            total += st["utime"] + st["stime"]
    return total / TICK


def memoria(pids):
    por = {}
    for pid in pids:
        nome = os.path.basename(ler_cmdline(pid).split(" ")[0]) or str(pid)
        por[nome] = {"rss_kib": rss_kib(pid), "pss_kib": pss_kib(pid)}
    return {"processos": por,
            "rss_mib": round(sum(v["rss_kib"] or 0 for v in por.values()) / 1024, 1),
            "pss_mib": round(sum(v["pss_kib"] or 0 for v in por.values()) / 1024, 1)}


def primeira_linha_visivel(caminho):
    with open(caminho) as f:
        return f.readline().rstrip("\n")[:40]


# --- a prova ----------------------------------------------------------------

def marcar(log_path, nome):
    with open(log_path, "a") as f:
        f.write(f"KINEIN_PROVA_MARCA {nome}\n")


def rodada_prova(caminho, linhas, extra_env=None):
    d = os.path.dirname(caminho)
    p = Pilha(d, nome="perf", env={**casa_limpa(), **(extra_env or {})})
    r = {"log": p.log_path}
    try:
        p.esperar_anexado()
        p.ui.pedir("esperar_quieto", quieto_ms=300)
        st = p.ui.pedir("estado")
        r["primeiro_quadro_ms"] = st["primeiro_quadro_ms"]
        r["editor_quadro_ms"] = st["editor_quadro_ms"]
        r["janela"] = st["janela"]
        # R2: do pedido de abrir ao quadro que mostra o texto do arquivo.
        ab = p.ui.pedir("abertura", path=caminho, esperado=primeira_linha_visivel(caminho), timeout_ms=60000,
                        timeout_s=70)
        r["abertura_ms"] = ab["ms"]
        alvo = linhas // 2
        p.lua("vim.api.nvim_win_set_cursor(0, {..., 0})", alvo)
        esperar(lambda: p.lua("return vim.api.nvim_get_mode().mode") == "i", 5, descricao="inserção")
        p.ui.pedir("esperar_quieto", quieto_ms=300)
        pids = arvore(p.pid_ui())
        cpu0 = cpu_s(pids)
        t0 = time.monotonic()
        # Marcas no log (O_APPEND, como a UI): o render.py recorta os quadros
        # da digitação, como faz com o typing_line/typing_hold da base.
        marcar(p.log_path, "digitacao_inicio")
        dig = p.ui.pedir("digitacao", teclas=40, quieto_ms=150, timeout_ms=180000, timeout_s=190)
        marcar(p.log_path, "digitacao_fim")
        r["cpu_digitando_s"] = round(cpu_s(pids) - cpu0, 3)
        r["duracao_digitando_s"] = round(time.monotonic() - t0, 2)
        r["memoria_depois"] = memoria(arvore(p.pid_ui()))
        r["amostras_ms"] = dig["amostras_ms"]
        r["primeiro_quadro_qualquer_ms"] = dig["primeiro_quadro_qualquer_ms"]
        # As teclas entraram no buffer, na linha certa, na ordem certa.
        buf = p.sup.pedir("texto", path=caminho)["lines"]
        r["linha_alvo"] = alvo
        r["teclas_no_buffer"] = buf[alvo - 1].startswith(dig["digitado"])
        r["linhas_no_buffer"] = len(buf)
        r["erros_de_protocolo"] = p.ui.pedir("estado")["erros_de_protocolo"]
    finally:
        p.fechar()
    return r


# --- a linha de base -----------------------------------------------------------

def rodada_base(caminho, linhas, extra_env=None):
    ws = os.path.dirname(caminho)
    env = os.environ.copy()
    env.update(casa_limpa())
    env.update(extra_env or {})
    env.update({
        "QT_QPA_PLATFORM": "offscreen",
        "QT_FORCE_STDERR_LOGGING": "1",
        "KINEIN_CORE_BIN": BASE_CORE,
        "KINEIN_PERF_MARKER": "1",
        "KINEIN_PERF_TYPING": "1",
        "KINEIN_PERF_TYPING_WORKSPACE": ws,
        "KINEIN_PERF_TYPING_FILE": caminho,
        "KINEIN_PERF_OPEN_PROBE": "1",
        "KINEIN_PERF_HOLD": "1",
    })
    log_path = os.path.join(ws, "base-ui.log")
    proc = subprocess.Popen([BASE_UI], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                            cwd=ws)
    r = {"amostras_ms": [], "log": log_path}
    cpu0 = t0 = None
    fim = time.monotonic() + 240
    try:
        with open(log_path, "w") as log:
            for linha in proc.stdout:
                log.write(linha)
                if time.monotonic() > fim:
                    r["erro"] = "tempo esgotado"
                    break
                if "KINEIN_PERF" not in linha:
                    continue
                chave, _, valor = linha.split("KINEIN_PERF ", 1)[1].strip().partition("=")
                if chave == "first_frame_ms":
                    r["primeiro_quadro_ms"] = float(valor)
                elif chave == "open_to_frame_ms":
                    r["abertura_ms"] = float(valor)
                elif chave == "typing_window":
                    r["janela"] = valor
                elif chave == "typing_file_lines":
                    r["linhas_no_editor"] = int(valor)
                elif chave == "typing_line":
                    r["linha_alvo"] = int(valor)
                    # O harness espera o quieto e manda a primeira tecla: a CPU
                    # conta daqui (o quieto de 150 ms entra nos dois lados).
                    pids = arvore(proc.pid)
                    cpu0, t0 = cpu_s(pids), time.monotonic()
                elif chave == "typing_sample_ms":
                    r["amostras_ms"].append(float(valor))
                elif chave == "typing_chars_inserted":
                    r["caracteres_inseridos"] = int(valor)
                elif chave == "typing_error":
                    r["erro"] = valor
                    break
                elif chave == "typing_hold":
                    pids = arvore(proc.pid)
                    r["cpu_digitando_s"] = round(cpu_s(pids) - cpu0, 3)
                    r["duracao_digitando_s"] = round(time.monotonic() - t0, 2)
                    r["memoria_depois"] = memoria(pids)
                    break
    finally:
        filhos = descendentes(proc.pid)
        proc.send_signal(signal.SIGTERM)
        try:
            proc.wait(10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        for pid in vivos(filhos):
            os.kill(pid, signal.SIGKILL)
    r["teclas_no_documento"] = r.get("caracteres_inseridos") == len(r["amostras_ms"]) == 40
    return r


def ttf_base(n):
    """O A da medir-performance.sh: o primeiro quadro da janela do produto."""
    valores = []
    for _ in range(n):
        env = os.environ.copy()
        env.update(casa_limpa())
        env.update({"QT_QPA_PLATFORM": "offscreen", "QT_FORCE_STDERR_LOGGING": "1", "KINEIN_CORE_BIN": BASE_CORE,
                    "KINEIN_PERF_MARKER": "1", "KINEIN_PERF_EXIT": "1"})
        # O marcador sai pelo log do Qt, que vai ao stderr (QT_FORCE_STDERR_LOGGING).
        out = subprocess.run([BASE_UI], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                             timeout=30).stdout
        for linha in out.splitlines():
            if "first_frame_ms=" in linha:
                valores.append(float(linha.split("first_frame_ms=")[1].split()[0]))
                break
    return valores


def resumir(rodadas, chave):
    vals = [r[chave] for r in rodadas if isinstance(r.get(chave), (int, float))]
    return resumo(vals) if vals else None


def fixture_nova(funcoes):
    """Cada rodada com a sua cópia: o produto grava rascunho em .kinein/ e
    salva sozinho no disco depois de 2 s; nenhuma rodada herda texto da outra."""
    d = tempfile.mkdtemp(prefix=f"perf-{funcoes}-", dir=TMP)
    return fixtures.fixture_desempenho(d, funcoes)


def resumo_lado(nome, rodadas):
    todas = [a for r in rodadas for a in r.get("amostras_ms", [])]
    saida = {
        "tecla_quadro_ms": resumo(todas) if todas else None,
        "abertura_ms": resumir(rodadas, "abertura_ms"),
        "primeiro_quadro_ms": resumir(rodadas, "primeiro_quadro_ms"),
        "cpu_digitando_s": resumir(rodadas, "cpu_digitando_s"),
        "rss_mib": resumo([r["memoria_depois"]["rss_mib"] for r in rodadas if "memoria_depois" in r]),
        "pss_mib": resumo([r["memoria_depois"]["pss_mib"] for r in rodadas if "memoria_depois" in r]),
        "rodadas": rodadas,
    }
    if nome == "prova":
        qualquer = [a for r in rodadas for a in r.get("primeiro_quadro_qualquer_ms", []) if a >= 0]
        saida["primeiro_quadro_qualquer_ms"] = resumo(qualquer) if qualquer else None
        saida["editor_quadro_ms"] = resumir(rodadas, "editor_quadro_ms")
    return saida


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    tamanhos = [int(x) for x in sys.argv[2:]] or [2, 60, 487]
    for funcoes in tamanhos:
        _, linhas = fixture_nova(funcoes)
        print(f"== {funcoes} funções: {linhas} linhas", flush=True)
        # Uma rodada de aquecimento de cada lado, descartada (cache de disco,
        # cache de fontes).
        rodada_base(*fixture_nova(funcoes))
        rodada_prova(*fixture_nova(funcoes))
        rodadas = {"base": [], "prova": []}
        # Alternadas: a deriva da máquina pesa igual nos dois lados.
        for i in range(n):
            for nome, funcao in (("base", rodada_base), ("prova", rodada_prova)):
                r = funcao(*fixture_nova(funcoes))
                rodadas[nome].append(r)
                med = resumo(r["amostras_ms"])["mediana"] if r.get("amostras_ms") else None
                print(f"   {nome} {i + 1}: abertura {r.get('abertura_ms')} ms, mediana tecla {med} ms, "
                      f"cpu {r.get('cpu_digitando_s')} s, erro {r.get('erro')}", flush=True)
        gravar(f"desempenho-{linhas}", {"funcoes": funcoes, "linhas": linhas, "rodadas_por_lado": n,
                                         "base": resumo_lado("base", rodadas["base"]),
                                         "prova": resumo_lado("prova", rodadas["prova"])})
    gravar("desempenho-ttf-base", {"primeiro_quadro_ms": resumo(ttf_base(n))})


if __name__ == "__main__":
    main()
