"""Cliente JSON-RPC compartilhado pelas provas reais de bancos."""
import json
from pathlib import Path
import queue
import subprocess
import threading
import time

REPO = Path(__file__).resolve().parent.parent

class Core:
    def __init__(self, env):
        self.process = subprocess.Popen(
            [str(REPO / "target/debug/kinein-core")], env=env,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, bufsize=1)
        self.inbox = queue.Queue()
        self.events = []
        self.raw = []
        self.errors = []
        self.identifier = 0
        threading.Thread(target=self.read, daemon=True).start()
        threading.Thread(target=self.read_errors, daemon=True).start()

    def read(self):
        for line in self.process.stdout:
            self.raw.append(line)
            self.inbox.put(json.loads(line))

    def read_errors(self):
        self.errors.extend(self.process.stderr)

    def wait(self, predicate, timeout=30):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                value = self.inbox.get(timeout=0.1)
            except queue.Empty:
                continue
            if predicate(value):
                return value
            if "method" in value:
                self.events.append(value)
        raise AssertionError("core sem resposta no prazo")

    def rpc(self, method, params):
        self.identifier += 1
        identifier = self.identifier
        self.process.stdin.write(json.dumps({
            "jsonrpc": "2.0", "id": identifier, "method": method, "params": params}) + "\n")
        self.process.stdin.flush()
        return self.wait(lambda value: value.get("id") == identifier)

    def event(self, name, context=None, *, timeout=30):
        method = "event.datasource." + name
        def matches(value):
            return value.get("method") == method and (
                context is None or value.get("params", {}).get("clientContext") == context)
        for index, value in enumerate(self.events):
            if matches(value):
                return self.events.pop(index)["params"]
        return self.wait(matches, timeout)["params"]

    def query(self, name, sql, password=None, confirmed=False, *, context=None, confirmation=None, max_rows=None):
        params = {"name": name, "sql": sql, "confirmWrite": confirmed}
        if context is not None:
            params.update(context)
        if confirmation is not None:
            params["confirmation"] = confirmation
        if max_rows is not None:
            params["maxRows"] = max_rows
        if password is not None:
            params["password"] = password
        response = self.rpc("datasource.query", params)
        assert "error" not in response, response.get("error", {}).get("message")
        return self.event("queried", context["clientContext"] if context else None)

    def close(self):
        if self.process.poll() is not None:
            return
        try:
            self.rpc("core.shutdown", {})
        finally:
            self.process.terminate()
            self.process.wait(timeout=5)
