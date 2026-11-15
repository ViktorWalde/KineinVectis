"""Clientes dos dois canais de controle da prova: o do supervisor (o papel do
core) e o da UI. Os dois falam JSON por linha num socket Unix."""

import json
import socket
import time


class Canal:
    def __init__(self, caminho, espera_s=20.0):
        fim = time.monotonic() + espera_s
        while True:
            try:
                self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                self.sock.connect(caminho)
                break
            except OSError:
                self.sock.close()
                if time.monotonic() > fim:
                    raise
                time.sleep(0.05)
        self.arquivo = self.sock.makefile("rw", encoding="utf-8")
        self.proximo = 1

    def pedir(self, op, timeout_s=60.0, **campos):
        pedido = dict(campos, op=op, id=self.proximo)
        self.proximo += 1
        self.sock.settimeout(timeout_s)
        self.arquivo.write(json.dumps(pedido) + "\n")
        self.arquivo.flush()
        linha = self.arquivo.readline()
        if not linha:
            raise ConnectionError(f"canal fechou durante {op}")
        resposta = json.loads(linha)
        if not resposta.get("ok"):
            raise RuntimeError(f"{op}: {resposta.get('erro')}")
        return resposta.get("resultado")

    def fechar(self):
        for f in (self.arquivo, self.sock):
            try:
                f.close()
            except OSError:
                pass
