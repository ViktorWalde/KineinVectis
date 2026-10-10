#!/usr/bin/env python3
"""Adaptador de banco FALSO para os testes da ponte (D1b.2, arquitetura/39 §6.1).

Fala a API externa 1.0 por linhas JSON no stdio, como um adaptador de verdade,
e cada MODO reproduz um risco que a ponte precisa conter:

  normal            handshake, chunk e terminal; `driver.shutdown` responde e sai
  slow_init         demora 10 s para responder o `initialize`
  big_line          a resposta da consulta passa do `messageBytes` negociado
  crash_after_write morre ao receber `driver.query`, sem responder
  late              responde a consulta depois de 400 ms
  stray_chunk       manda um chunk de uma operacao que ninguem pediu
  noisy             despeja 20.000 linhas no stderr antes de tudo
  ignore_shutdown   nao responde o `shutdown` e ignora EOF e TERM
  helper <arquivo>  sobe um auxiliar no mesmo grupo e grava o PID no arquivo

Uso: fake_driver_adapter.py <modo> [arquivo]
"""

import json
import os
import signal
import subprocess
import sys
import time

MODE = sys.argv[1] if len(sys.argv) > 1 else "normal"

INIT = {
    "api": {"min": {"major": 1, "minor": 0}, "max": {"major": 1, "minor": 0}},
    "adapterId": "native.fake",
    "adapterVersion": "0.1.0",
    "engines": ["postgres"],
    "operations": [
        "open", "test", "introspect", "query", "impact",
        "preview", "decide", "cancel", "close", "shutdown",
    ],
    "limits": {
        "messageBytes": 4096, "inFlight": 2, "rows": 100, "columns": 8,
        "cellBytes": 1024, "retainedBytes": 4096, "catalogueItems": 50,
    },
}


def send(message):
    sys.stdout.write(json.dumps(message, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def terminal(request_id, context, kind):
    send({"jsonrpc": "2.0", "id": request_id,
          "result": {"context": context, "sequence": 0, "result": {"kind": kind}}})


def main():
    if MODE == "noisy":
        for index in range(20000):
            sys.stderr.write(f"linha de diagnostico {index}\n")
        sys.stderr.flush()
    if MODE == "helper":
        # O proprio Python, e nao o `sleep`: o Windows nao tem `sleep`.
        helper = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
        with open(sys.argv[2], "w", encoding="utf-8") as pid_file:
            pid_file.write(str(helper.pid))
    if MODE == "ignore_shutdown":
        signal.signal(signal.SIGTERM, signal.SIG_IGN)

    for line in sys.stdin:
        message = json.loads(line)
        method = message.get("method")
        request_id = message.get("id")
        context = (message.get("params") or {}).get("context")
        if method == "driver.initialize":
            if MODE == "slow_init":
                time.sleep(10)
            send({"jsonrpc": "2.0", "id": request_id, "result": INIT})
        elif method == "driver.query":
            if MODE == "crash_after_write":
                os._exit(3)
            if MODE == "late":
                time.sleep(0.4)
            if MODE == "stray_chunk":
                stray = dict(context, operationId="ninguem-pediu")
                send({"jsonrpc": "2.0", "method": "driver.chunk",
                      "params": {"context": stray, "sequence": 0, "payload": {}}})
                continue
            cell = "x" * (8000 if MODE == "big_line" else 1)
            send({"jsonrpc": "2.0", "method": "driver.chunk",
                  "params": {"context": context, "sequence": 0,
                             "payload": {"kind": "rows", "columns": ["a"], "rows": [[cell]]}}})
            terminal(request_id, context, "query")
        elif method == "driver.shutdown":
            if MODE == "ignore_shutdown":
                while True:
                    time.sleep(1)
            terminal(request_id, context, "shutdown")
            return
        elif method in ("driver.test", "driver.cancel", "driver.close"):
            terminal(request_id, context, method.split(".")[1])
        else:
            send({"jsonrpc": "2.0", "id": request_id,
                  "error": {"code": -32601, "message": "metodo desconhecido"}})
    if MODE == "ignore_shutdown":
        while True:
            time.sleep(1)


if __name__ == "__main__":
    main()
