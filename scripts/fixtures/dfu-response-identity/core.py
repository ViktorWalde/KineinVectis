#!/usr/bin/env python3
"""Reorder real IPC responses without invoking any USB tool."""
import json
import sys

pending = None
sequence = 0


def send(request_id, result=None, error=None):
    response = {"jsonrpc": "2.0", "id": request_id}
    response.update({"error": error} if error else {"result": result})
    print(json.dumps(response), flush=True)


def listed(request_id, name):
    send(request_id, {"complete": True, "toolAvailable": True,
                      "interfaces": [{"mode": "dfu", "name": name}], "rawOutput": name})


def failed(request_id, message):
    send(request_id, error={"code": "INVALID_REQUEST", "message": message})


for line in sys.stdin:
    request = json.loads(line)
    request_id = request["id"]
    method = request["method"]
    if method == "core.ping":
        send(request_id, {"protocolVersion": "0.193.0"})
    elif method == "workspace.open":
        send(request_id, {"root": request["params"]["path"], "name": "DFU fixture", "kind": "cmake",
                          "capabilities": {"buildSystems": []}})
        if pending is not None:
            listed(pending, "stale-after-open")
            pending = None
    elif method == "workspace.close":
        send(request_id, {})
        if pending is not None:
            failed(pending, "stale-after-close")
            pending = None
    elif method == "dfu.list":
        assert request["params"] == {}
        sequence += 1
        if sequence in (1, 3, 5, 8):
            pending = request_id
        elif sequence == 2:
            listed(request_id, "newest-success")
            listed(pending, "stale-success")
            pending = None
        elif sequence == 4:
            listed(request_id, "newest-before-stale-error")
            failed(pending, "stale-error")
            pending = None
        elif sequence == 6:
            failed(request_id, "current-query-failure")
        elif sequence == 7:
            listed(request_id, "retry-success")
            listed(request_id, "duplicate-retry-success")
        else:
            failed(request_id, "unexpected-query")
    else:
        send(request_id, {})
