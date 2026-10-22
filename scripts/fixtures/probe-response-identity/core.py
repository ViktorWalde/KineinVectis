#!/usr/bin/env python3
"""Core IPC fixture: deliberate stale probe replies, without tools or hardware."""
import json
import sys

sequence = 0
pending = None


def send_response(request_id, result=None, error=None):
    message = {"jsonrpc": "2.0", "id": request_id}
    message["error" if error else "result"] = error if error else result
    print(json.dumps(message), flush=True)


def failure(request_id, message):
    send_response(request_id, error={"code": "INVALID_REQUEST", "message": message})


def probes(request_id, name):
    send_response(request_id, {"probes": [{"name": name, "vid": "1234", "pid": "5678", "serial": "S1"}],
                         "toolAvailable": True, "rawOutput": name})


for line in sys.stdin:
    request = json.loads(line)
    request_id = request.get("id")
    method = request.get("method")
    if request_id is None:
        continue
    if method == "core.ping":
        send_response(request_id, {"protocolVersion": "0.187.0"})
    elif method == "workspace.open":
        if request["params"]["path"].endswith("/rejected"):
            failure(request_id, "workspace-open-failure")
            if pending is not None:
                probes(pending, "survives-open-failure")
                pending = None
        else:
            send_response(request_id, {"root": request["params"]["path"], "name": "Probe fixture", "kind": "cmake",
                                 "capabilities": {"buildSystems": []}})
            if pending is not None:
                probes(pending, "survives-same-root" if sequence == 10 else "stale-after-open")
                pending = None
    elif method == "workspace.close":
        send_response(request_id, {})
        if pending is not None:
            failure(pending, "stale-after-close")
            pending = None
    elif method == "probe.list":
        sequence += 1
        if sequence in (1, 3, 5, 6, 9, 10):
            pending = request_id
        elif sequence == 2:
            probes(request_id, "newest-success")
            probes(pending, "stale-success")
            pending = None
        elif sequence == 4:
            probes(request_id, "newest-before-stale-error")
            failure(pending, "stale-error")
            pending = None
        elif sequence == 7:
            failure(request_id, "current-query-failure")
        elif sequence == 8:
            probes(request_id, "retry-success")
            probes(request_id, "duplicate-retry-success")
        else:
            failure(request_id, "unexpected-query")
    else:
        send_response(request_id, {})
