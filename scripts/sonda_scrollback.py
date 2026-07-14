#!/usr/bin/env python3
"""Sonda e2e do terminal: scrollback (B1/B2) + multi-sessão (D2.3).

Prova, contra o core REAL via stdio:
  1. terminal.open num workspace;
  2. gera MUITO mais output do que cabe na tela (seq 1..300 num grid de 24
     linhas) -> o histórico tem que existir;
  3. o event.terminal.render passa a trazer scrollbackMax > 0 (era esse dado
     que a UI não tinha, e sem ele não dá pra desenhar barra nem saber se há o
     que rolar);
  4. terminal.scroll { offset } faz o render voltar com scrollback == offset
     (o core clampa e ecoa a VERDADE) e o conteúdo da tela muda.
"""
import json
import os
import queue
import subprocess
import sys
import threading
import time

REPO = "/home/viktor/KineinVectis"
CORE = os.path.join(REPO, "target", "debug", "kinein-core")

proc = subprocess.Popen(
    [CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL, text=True, bufsize=1)

_id = 0
renders = []
inbox = queue.Queue()


def _reader():
    # readline() bloqueia pra sempre quando o core está quieto (o terminal só
    # emite render quando há mudança) — por isso a leitura vive numa thread e o
    # pump só faz get() com timeout.
    for line in proc.stdout:
        inbox.put(line)


threading.Thread(target=_reader, daemon=True).start()


def send(method, params):
    global _id
    _id += 1
    proc.stdin.write(json.dumps(
        {"jsonrpc": "2.0", "id": _id, "method": method, "params": params}) + "\n")
    proc.stdin.flush()
    return _id


def pump(until_id=None, seconds=2.0):
    """Drena mensagens por `seconds`, guardando os renders; para na resposta
    de `until_id` se ela chegar antes."""
    deadline = time.time() + seconds
    found = None
    while time.time() < deadline:
        try:
            line = inbox.get(timeout=0.1)
        except queue.Empty:
            continue
        msg = json.loads(line)
        if msg.get("method") == "event.terminal.render":
            renders.append(msg["params"])
        if until_id is not None and msg.get("id") == until_id:
            found = msg
            until_id = None  # segue drenando os renders que vierem atrás
    return found


def last_render(term_id=None):
    for r in reversed(renders):
        if term_id is None or r.get("id") == term_id:
            return r
    return None


def screen_text(r):
    out = []
    for line in r["lines"]:
        out.append("".join(span["text"] for span in line))
    return "\n".join(out)


fails = []


def check(name, cond, detail=""):
    print(f"  {'ok  ' if cond else 'FALHOU'} {name}" + (f"  [{detail}]" if detail else ""))
    if not cond:
        fails.append(name)


print("== abrindo workspace + terminal ==")
pump(send("workspace.open", {"path": REPO}), 5)
opened = pump(send("terminal.open", {}), 3)
TID = opened["result"]["id"]
check("terminal.open devolve um id", bool(TID), f"id={TID}")
pump(send("terminal.resize", {"id": TID, "cols": 80, "rows": 24}), 2)

print("== gerando 300 linhas (grid tem 24) ==")
send("terminal.input", {"id": TID, "data": "seq 1 300\r"})
pump(seconds=3.0)

r = last_render(TID)
check("render chegou", r is not None)
if r is None:
    proc.kill()
    sys.exit(1)

check("render vem etiquetado com o id", r.get("id") == TID)
check("render traz scrollbackMax", "scrollbackMax" in r, f"campos={sorted(r.keys())}")
check("render traz scrollback", "scrollback" in r)
smax = r.get("scrollbackMax", 0)
check("scrollbackMax > 0 (há histórico)", smax > 0, f"scrollbackMax={smax}")
check("está ao vivo (scrollback == 0)", r.get("scrollback") == 0)

vivo = screen_text(r)
check("tela ao vivo mostra o FIM (300)", "300" in vivo)

print("== rolando 20 linhas pra cima ==")
pump(send("terminal.scroll", {"id": TID, "offset": 20}), 3)
pump(seconds=1.0)
r2 = last_render(TID)
check("scrollback ecoa o offset pedido", r2.get("scrollback") == 20,
      f"scrollback={r2.get('scrollback')}")
check("scrollbackMax segue > 0", r2.get("scrollbackMax", 0) > 0)
rolado = screen_text(r2)
check("a tela MUDOU ao rolar", rolado != vivo)
check("tela rolada não mostra mais o fim (300)", "300" not in rolado)

print("== pedindo offset absurdo (9999): o core tem que CLAMPAR ==")
n_before = len(renders)
pump(send("terminal.scroll", {"id": TID, "offset": 9999}), 3)
pump(seconds=1.0)
r3 = last_render(TID)
check("offset clampado ao histórico real", r3.get("scrollback") == r3.get("scrollbackMax"),
      f"scrollback={r3.get('scrollback')} max={r3.get('scrollbackMax')}")

# ---- D2.3: multi-sessão ----------------------------------------------------
print("== abrindo um SEGUNDO terminal (D2.3) ==")
opened2 = pump(send("terminal.open", {}), 3)
TID2 = opened2["result"]["id"]
check("segundo terminal abre", bool(TID2), f"id={TID2}")
check("ids são distintos", TID2 != TID, f"{TID} vs {TID2}")

pump(send("terminal.resize", {"id": TID2, "cols": 80, "rows": 24}), 2)
send("terminal.input", {"id": TID2, "data": "echo SEGUNDA-SESSAO\r"})
pump(seconds=2.5)

r_b = last_render(TID2)
check("o segundo terminal renderiza", r_b is not None)
check("a saída caiu na sessão CERTA", "SEGUNDA-SESSAO" in screen_text(r_b))

r_a = last_render(TID)
check("a PRIMEIRA sessão não foi contaminada",
      "SEGUNDA-SESSAO" not in screen_text(r_a))

print("== fechando o segundo: o primeiro tem que sobreviver ==")
pump(send("terminal.close", {"id": TID2}), 3)
pump(seconds=0.5)
# O terminal 1 ficou ROLADO lá em cima (offset 278) nos testes acima. Saída
# nova cai no FUNDO, fora da view — por isso a UI faz snap-to-bottom ao digitar.
# A sonda tem que imitar isso, senão testaria a coisa errada.
pump(send("terminal.scroll", {"id": TID, "offset": 0}), 2)
send("terminal.input", {"id": TID, "data": "echo AINDA-VIVO\r"})
pump(seconds=2.5)
r_a2 = last_render(TID)
check("fechar um terminal não mata o outro", "AINDA-VIVO" in screen_text(r_a2))

send("core.shutdown", {})
time.sleep(0.3)
proc.kill()

print()
if fails:
    print(f"✗ FALHOU: {fails}")
    sys.exit(1)
print("✓ terminal e2e (scrollback + multi-sessão): tudo verde")
