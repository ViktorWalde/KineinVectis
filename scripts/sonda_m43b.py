#!/usr/bin/env python3
"""Sonda e2e da fatia M4.3b via stdio JSON-RPC.

Parte B (lsp.restart): abre o repo, faz fs.read de um .rs (dispara o
rust-analyzer via did_open), reinicia o LSP e confere que a resposta lista
"rust" como reiniciado.

Parte C (jobs órfãos no shutdown): abre o repo, dispara build.run (cargo),
espera o filho cargo aparecer, manda core.shutdown e confere que, quando o
core sai, o processo cargo NÃO ficou órfão (foi morto pelo drain do Drop).
"""
import json
import os
import subprocess
import sys
import time

REPO = "/home/viktor/KineinVectis"
CORE = os.path.join(REPO, "target", "debug", "kinein-core")


def make_rpc(proc):
    state = {"id": 0}

    def rpc(method, params, want_response=True):
        state["id"] += 1
        want = state["id"]
        proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": want, "method": method, "params": params}) + "\n")
        proc.stdin.flush()
        if not want_response:
            return None
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError(f"EOF esperando {method}")
            msg = json.loads(line)
            if msg.get("id") == want:
                return msg
    return rpc


def child_pids(pid, comm_substr):
    """PIDs filhos de `pid` cujo comm contém `comm_substr`."""
    found = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat") as fh:
                fields = fh.read().rsplit(")", 1)
            comm = fields[0].split("(", 1)[1]
            ppid = int(fields[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        if ppid == pid and comm_substr in comm:
            found.append(int(entry))
    return found


def pid_alive(pid):
    return os.path.exists(f"/proc/{pid}")


fails = []


def check(cond, label):
    print(("OK  " if cond else "FALHOU ") + label)
    if not cond:
        fails.append(label)


def parte_b():
    proc = subprocess.Popen([CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True)
    rpc = make_rpc(proc)
    try:
        rpc("workspace.open", {"path": REPO})
        # Dispara o rust-analyzer (did_open do .rs).
        rpc("fs.read", {"path": "crates/kinein-core/src/lib.rs"})
        time.sleep(1)
        resp = rpc("lsp.restart", {})
        restarted = resp.get("result", {}).get("restarted", [])
        check("rust" in restarted, f"lsp.restart reinicia rust (got {restarted})")
    finally:
        proc.stdin.close()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


def parte_c():
    proc = subprocess.Popen([CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True)
    rpc = make_rpc(proc)
    rpc("workspace.open", {"path": REPO})
    rpc("build.run", {})
    # Espera o cargo filho aparecer.
    cargo = []
    deadline = time.time() + 8
    while time.time() < deadline:
        cargo = child_pids(proc.pid, "cargo")
        if cargo:
            break
        time.sleep(0.1)
    check(bool(cargo), f"build.run gerou processo cargo filho (pids {cargo})")

    # Shutdown limpo: o Drop do JobManager deve matar o cargo.
    rpc("core.shutdown", {}, want_response=True)
    proc.stdin.close()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
    time.sleep(0.3)
    orphans = [p for p in cargo if pid_alive(p)]
    check(not orphans, f"nenhum cargo órfão após shutdown (vivos: {orphans})")
    # Limpeza defensiva se algo escapou.
    for p in orphans:
        try:
            os.kill(p, 9)
        except OSError:
            pass


def main():
    print("== Sonda M4.3b ==")
    print("-- Parte B: lsp.restart --")
    parte_b()
    print("-- Parte C: jobs órfãos no shutdown --")
    parte_c()
    if fails:
        print(f"\nSONDA M4.3b FALHOU: {len(fails)} checagem(ns)")
        return 1
    print("\nSONDA M4.3b: tudo verde")
    return 0


if __name__ == "__main__":
    sys.exit(main())
