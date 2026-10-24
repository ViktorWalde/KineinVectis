#!/usr/bin/env python3
# A fake `pio` for the PIO Home tests (platformio/tests.rs, 52 §14.45, §14.46).
#
# `system info` names this interpreter; `home` serves the session page (200),
# the root (403), the shutdown and the IDE agent's WebSocket (`wsrpc`), where
# the test injects the page's commands through `commands.jsonl` and reads the
# answers in `results.jsonl`. Files beside it (`mode-*`) change how it behaves.
import base64, hashlib, http.server, json, os, queue, struct, sys, threading, time
from pathlib import Path
here = Path(__file__).resolve().parent
args = sys.argv[1:]
if args[:2] == ["system", "info"]:
    print(json.dumps({"core_version": {"value": "6.2.0"}, "python_exe": {"value": sys.executable}}))
    sys.exit(0)
with open(here / "home.launches", "a") as launches:
    launches.write(json.dumps(args) + "\n")
(here / "home.cwd").write_text(os.getcwd())
if (here / "mode-exit").exists():
    print("Error: fixture refuses to start", flush=True)
    sys.exit(3)
port = int(args[args.index("--port") + 1])
prefix = "/session/%s/" % args[args.index("--session-id") + 1]

def frames(sock):
    buffer = b""
    def read(size):
        nonlocal buffer
        while len(buffer) < size:
            chunk = sock.recv(65536)
            if not chunk:
                raise ConnectionError("closed")
            buffer += chunk
        data, buffer = buffer[:size], buffer[size:]
        return data
    while True:
        first, second = read(2)
        size = second & 0x7F
        if size == 126:
            size = struct.unpack("!H", read(2))[0]
        elif size == 127:
            size = struct.unpack("!Q", read(8))[0]
        mask = read(4)
        yield first & 0x0F, bytes(b ^ mask[i % 4] for i, b in enumerate(read(size)))

def send(sock, text):
    data = text.encode()
    if len(data) < 126:
        header = struct.pack("!BB", 0x81, len(data))
    else:
        header = struct.pack("!BBH", 0x81, 126, len(data))
    sock.sendall(header + data)

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *ignored):
        pass
    def do_GET(self):
        if self.path == prefix + "wsrpc" and self.headers.get("Upgrade", "").lower() == "websocket":
            if (here / "mode-no-ws").exists():
                self.send_response(404)
                self.end_headers()
                return
            return self.websocket()
        ok = self.path == prefix and not (here / "mode-wrong").exists()
        self.send_response(200 if ok else 403)
        self.end_headers()
        self.wfile.write(b"home" if ok else b"Protected PlatformIO Home session")
    def do_POST(self):
        if self.path != prefix + "__shutdown__":
            self.send_response(403)
            self.end_headers()
            return
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"Server has been shutdown!")
        self.wfile.flush()
        (here / "home.shutdown").write_text("yes")
        os._exit(0)
    def websocket(self):
        key = self.headers["Sec-WebSocket-Key"] + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
        self.protocol_version = "HTTP/1.1"
        self.send_response(101)
        self.send_header("Upgrade", "websocket")
        self.send_header("Connection", "Upgrade")
        self.send_header("Sec-WebSocket-Accept", base64.b64encode(hashlib.sha1(key.encode()).digest()).decode())
        self.end_headers()
        self.wfile.flush()
        self.close_connection = True
        sock = self.connection
        incoming = queue.Queue()
        def reader():
            try:
                for frame in frames(sock):
                    incoming.put(frame)
            except Exception:
                incoming.put((-1, b""))
        threading.Thread(target=reader, daemon=True).start()
        # As home/rpc/handlers/ide.py does: a command stays pending until its
        # result arrives, and every pass hands each pending command to all the
        # listeners waiting. `mode-overtake` handles a result only after the
        # next listen, as the server's parallel tasks may.
        listening, pending, loaded, held, opened = [], [], set(), None, time.time()
        def process():
            for command in pending:
                while listening:
                    send(sock, json.dumps({"jsonrpc": "2.0", "id": listening.pop(), "result": command}))
        def answer(message):
            with open(here / "results.jsonl", "a") as results:
                results.write(json.dumps(message["params"]) + "\n")
            pending[:] = [command for command in pending if command["id"] != message["params"][0]]
            send(sock, json.dumps({"jsonrpc": "2.0", "id": message["id"], "result": True}))
        while True:
            try:
                opcode, payload = incoming.get(timeout=0.05)
                if opcode in (8, -1):
                    (here / "ws.closed").write_text("frame" if opcode == 8 else "dropped")
                    return
                message = json.loads(payload)
                if message["method"] == "ide.listen_commands":
                    with open(here / "listens", "a") as listens:
                        listens.write("+")
                    listening.append(message["id"])
                    process()
                    if held is not None:
                        answer(held)
                        held = None
                elif message["method"] == "ide.on_command_result":
                    if (here / "mode-overtake").exists():
                        held = message
                    else:
                        answer(message)
            except queue.Empty:
                pass
            if (here / "mode-ws-close").exists() and time.time() - opened > 1:
                sock.sendall(b"\x88\x02\x03\xe8")
                return
            injected = here / "commands.jsonl"
            if injected.exists():
                for line in injected.read_text().splitlines():
                    command = json.loads(line)
                    if command["id"] not in loaded:
                        loaded.add(command["id"])
                        pending.append(command)
            process()

server = http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler)
if (here / "mode-silent").exists():
    time.sleep(60)
    sys.exit(0)
if (here / "mode-short").exists():
    threading.Timer(1.0, lambda: os._exit(7)).start()
print("PIO Home has been started. Press Ctrl+C to shutdown.", flush=True)
server.serve_forever()
