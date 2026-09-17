"""Real debugpy TCP attach proof, using the existing Core RPC harness."""
from __future__ import annotations

import json
import pathlib
import subprocess
import time


def wait_for(check, message: str, timeout: float = 15.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = check()
        if value:
            return value
        time.sleep(0.05)
    raise AssertionError(message)


def verify_attach(core, root: pathlib.Path, interpreter: pathlib.Path) -> None:
    workspace = root / "attach"
    workspace.mkdir()
    service = workspace / "service.py"
    service.write_text(
        "import time\n"
        "from pathlib import Path\n"
        "def tick(value):\n"
        "    doubled = value * 2\n"
        "    Path('heartbeat').write_text(str(value))\n"
        "    return doubled\n"
        "value = 0\n"
        "while not Path('finish').exists():\n"
        "    value += 1\n"
        "    tick(value)\n"
        "    time.sleep(0.03)\n"
    )
    # debugpy.listen on port 0 allocates the endpoint without a bind/rebind race.
    # Readiness is a file, not a TCP probe that would become a DAP client.
    bootstrap = (
        "import debugpy,json,runpy; from pathlib import Path; "
        "host,port=debugpy.listen(('127.0.0.1',0)); "
        "Path('endpoint.json').write_text(json.dumps({'host':host,'port':port})); "
        "debugpy.wait_for_client(); runpy.run_path('service.py')"
    )
    heartbeat = workspace / "heartbeat"

    def beats() -> int:
        try:
            return int(heartbeat.read_text())
        except (FileNotFoundError, ValueError):
            return 0

    with open(workspace / "service.log", "w", encoding="utf-8") as log:
        proc = subprocess.Popen([str(interpreter), "-c", bootstrap], cwd=workspace,
                                stdout=log, stderr=log)
        try:
            endpoint_path = workspace / "endpoint.json"
            wait_for(endpoint_path.exists, "debugpy nao publicou a porta")
            endpoint = json.loads(endpoint_path.read_text())
            core.rpc("workspace.open", {"path": str(workspace)})
            bp = {"file": str(service), "breakpoints": [{"line": 4}]}
            core.rpc("debug.setBreakpoints", bp)
            for phase in ["paused", "running", "workspace.close"]:
                core.rpc("debug.setBreakpoints", bp)
                result = core.rpc("debug.start", {"connect": endpoint})
                assert result["attached"] is True, result
                assert core.evento("event.debug.started")["attached"] is True
                stopped = core.evento("event.debug.stopped")
                assert stopped["reason"] == "breakpoint" and stopped["line"] == 4, stopped
                frames = core.rpc("debug.stackTrace")["frames"]
                assert frames[0]["name"] == "tick", frames
                frame_id = frames[0]["id"]
                variables = core.rpc("debug.variables", {"frameId": frame_id})["variables"]
                value = next(int(v["value"]) for v in variables if v["name"] == "value")
                evaluated = core.rpc("debug.evaluate", {"expression": "value * 2", "frameId": frame_id})
                assert int(evaluated["result"]) == value * 2, evaluated
                if phase == "running":
                    core.rpc("debug.setBreakpoints", {"file": str(service), "breakpoints": []})
                    core.rpc("debug.continue")
                    wait_for(lambda: beats() > value, "continue nao retomou o Python")
                before = beats()
                core.rpc("workspace.close" if phase == "workspace.close" else "debug.stop")
                core.evento("event.debug.finished")
                wait_for(lambda: beats() > before, f"{phase}: Python nao continuou apos detach")
                assert proc.poll() is None, (phase, proc.returncode)
                print(f"attach TCP: {phase}, breakpoint/locals/evaluate, detach preservou PID {proc.pid}")
            (workspace / "finish").touch()
            assert proc.wait(timeout=10) == 0
        except Exception:
            log.flush()
            print((workspace / "service.log").read_text(errors="replace"))
            raise
        finally:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
