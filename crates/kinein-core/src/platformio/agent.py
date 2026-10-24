"""The IDE agent of PIO Home (52 §14.46).

The core runs this in PlatformIO's own interpreter (`python -I -c`) as soon as
PIO Home answers, with the session's WebSocket address as the only argument.
It keeps one `ide.listen_commands` pending, prints every command the page sends
as a JSON line, sends back the result the core writes to stdin, and only then
listens again. It exits
when stdin closes: without its last client, PIO Home goes back to stopping
itself when idle.

Only the standard library: a minimal RFC 6455 client (text, ping, close,
fragments), enough for the JSON-RPC PIO Home speaks.
"""
import base64
import collections
import json
import os
import socket
import struct
import sys
import threading
import time
from urllib.parse import urlparse


class Socket:
    """A WebSocket client over a blocking socket; sends are serialized."""

    def __init__(self, address):
        url = urlparse(address)
        self.sock = socket.create_connection((url.hostname, url.port), timeout=30)
        self.lock = threading.Lock()
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall((
            f"GET {url.path} HTTP/1.1\r\nHost: {url.hostname}:{url.port}\r\n"
            "Upgrade: websocket\r\nConnection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n").encode())
        head = b""
        while b"\r\n\r\n" not in head:
            chunk = self.sock.recv(4096)
            if not chunk:
                raise ConnectionError("PIO Home closed the handshake")
            head += chunk
        head, self.buffer = head.split(b"\r\n\r\n", 1)
        if not head.startswith(b"HTTP/1.1 101"):
            raise ConnectionError(head.split(b"\r\n", 1)[0].decode(errors="replace"))
        self.sock.settimeout(None)

    def send(self, opcode, payload=b""):
        mask = os.urandom(4)
        size = len(payload)
        if size < 126:
            header = struct.pack("!BB", 0x80 | opcode, 0x80 | size)
        elif size < 65536:
            header = struct.pack("!BBH", 0x80 | opcode, 0x80 | 126, size)
        else:
            header = struct.pack("!BBQ", 0x80 | opcode, 0x80 | 127, size)
        masked = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        with self.lock:
            self.sock.sendall(header + mask + masked)

    def read(self, size):
        while len(self.buffer) < size:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise ConnectionError("PIO Home closed the connection")
            self.buffer += chunk
        data, self.buffer = self.buffer[:size], self.buffer[size:]
        return data

    def message(self):
        """The next text message; answers pings; None on close."""
        parts = []
        while True:
            first, second = self.read(2)
            opcode, size = first & 0x0F, second & 0x7F
            if size == 126:
                size = struct.unpack("!H", self.read(2))[0]
            elif size == 127:
                size = struct.unpack("!Q", self.read(8))[0]
            mask = self.read(4) if second & 0x80 else None
            payload = self.read(size)
            if mask:
                payload = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
            if opcode == 0x8:
                return None
            if opcode == 0x9:
                self.send(0xA, payload)
                continue
            if opcode in (0x1, 0x0):
                parts.append(payload)
                if first & 0x80:
                    return b"".join(parts).decode()


def emit(value):
    sys.stdout.write(json.dumps(value) + "\n")
    sys.stdout.flush()


def main():
    link = Socket(sys.argv[1])
    lock = threading.Lock()
    state = {"counter": 0, "listening": None}
    # PIO Home keeps a command until its result arrives and hands every
    # pending one to each `listen_commands`, and it runs each request in its
    # own task: a listen can overtake the result before it. So the next
    # listen goes out only after the result, and an id seen before is not
    # passed on again.
    seen = collections.deque(maxlen=256)

    def call(method, params):
        with lock:
            state["counter"] += 1
            identity = state["counter"]
            if method == "ide.listen_commands":
                state["listening"] = identity
        link.send(0x1, json.dumps({"jsonrpc": "2.0", "id": identity, "method": method,
                                   "params": params}).encode())

    def answer():
        for line in sys.stdin:
            try:
                result = json.loads(line)["result"]
                call("ide.on_command_result", [result["id"], result.get("value")])
            except (ValueError, KeyError, TypeError):
                continue
            call("ide.listen_commands", [])
        # The core is gone: leave, so PIO Home can stop itself when idle.
        try:
            link.send(0x8, struct.pack("!H", 1000))
        finally:
            os._exit(0)

    threading.Thread(target=answer, daemon=True).start()
    call("ide.listen_commands", [])
    emit({"ready": True})
    while True:
        text = link.message()
        if text is None:
            os._exit(0)
        reply = json.loads(text)
        with lock:
            if reply.get("id") != state["listening"]:
                continue
        command = reply.get("result")
        if not isinstance(command, dict) or command.get("id") in seen:
            if "error" in reply:
                time.sleep(0.5)
            call("ide.listen_commands", [])
            continue
        seen.append(command.get("id"))
        emit({"command": command})


# `-c` runs as __main__; the stdio proof imports `Socket` to play the page.
if __name__ == "__main__":
    try:
        main()
    except (OSError, ConnectionError, ValueError) as error:
        emit({"error": str(error)})
        sys.exit(2)
