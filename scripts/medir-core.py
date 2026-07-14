#!/usr/bin/env python3
"""Medições de performance do kinein-core via stdio (fatia M4.2).

Chamado por scripts/medir-performance.sh. Mede, tudo LOCAL (sem rede):
  B  workspace.open no repo Kinein (o "workspace grande"), mediana de N.
  C  fs.read de um .txt sintético de 10k linhas (proxy de abrir arquivo
     grande no editor — lado core), mediana de N.
  D2 RSS do core em regime + RSS do LSP Rust (rust-analyzer, best-effort)
     após workspace.open + fs.read de um .rs (que dispara did_open).

Imprime linhas `chave=valor` para o shell parsear.
"""
import json
import os
import subprocess
import sys
import tempfile
import time


def spawn(core):
    return subprocess.Popen(
        [core], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, text=True,
    )


def make_rpc(proc):
    state = {"id": 0}

    def rpc(method, params):
        state["id"] += 1
        want = state["id"]
        proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": want, "method": method, "params": params}) + "\n")
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError(f"EOF esperando {method}")
            msg = json.loads(line)
            if msg.get("id") == want:
                return msg
    return rpc


def median_ms(samples):
    s = sorted(samples)
    n = len(s)
    if n == 0:
        return "n/d"
    mid = s[n // 2] if n % 2 else (s[n // 2 - 1] + s[n // 2]) / 2
    return f"{mid:.1f}"


def vmrss_kb(pid):
    try:
        with open(f"/proc/{pid}/status") as fh:
            for ln in fh:
                if ln.startswith("VmRSS:"):
                    return int(ln.split()[1])
    except OSError:
        pass
    return 0


def descendants_rss_kb(pid):
    """Soma o VmRSS dos processos filhos (rust-analyzer é filho do core)."""
    total = 0
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat") as fh:
                ppid = int(fh.read().split()[3])
        except (OSError, IndexError, ValueError):
            continue
        if ppid == pid:
            total += vmrss_kb(int(entry))
    return total


def measure_workspace_open(core, root, n):
    samples = []
    for _ in range(n):
        proc = spawn(core)
        rpc = make_rpc(proc)
        t0 = time.perf_counter()
        rpc("workspace.open", {"path": root})
        samples.append((time.perf_counter() - t0) * 1000.0)
        proc.stdin.close()
        proc.wait(timeout=5)
    return median_ms(samples)


def measure_fs_read_big(core, n):
    tmp = tempfile.mkdtemp(prefix="kinein-perf-")
    big = os.path.join(tmp, "big.txt")
    with open(big, "w") as fh:
        for i in range(10_000):
            fh.write(f"linha {i:05d} — conteudo sintetico para medir leitura de arquivo grande\n")
    proc = spawn(core)
    rpc = make_rpc(proc)
    rpc("workspace.open", {"path": tmp})
    samples = []
    for _ in range(n):
        t0 = time.perf_counter()
        rpc("fs.read", {"path": "big.txt"})
        samples.append((time.perf_counter() - t0) * 1000.0)
    proc.stdin.close()
    proc.wait(timeout=5)
    return median_ms(samples)


def measure_rss_with_lsp(core, root):
    proc = spawn(core)
    rpc = make_rpc(proc)
    rpc("workspace.open", {"path": root})
    # fs.read de um .rs dispara did_open -> rust-analyzer sobe e indexa.
    rpc("fs.read", {"path": "crates/kinein-core/src/lib.rs"})
    time.sleep(8)  # janela fixa de indexação do rust-analyzer
    core_rss = vmrss_kb(proc.pid)
    lsp_rss = descendants_rss_kb(proc.pid)
    proc.stdin.close()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
    return core_rss, lsp_rss


def main():
    core, root, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
    print(f"workspace_open_ms={measure_workspace_open(core, root, n)}")
    print(f"fs_read_10k_ms={measure_fs_read_big(core, n)}")
    core_rss, lsp_rss = measure_rss_with_lsp(core, root)
    print(f"core_rss_mb={core_rss // 1024}")
    print(f"lsp_rss_mb={(lsp_rss // 1024) if lsp_rss else 'n/d (rust-analyzer ausente?)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
