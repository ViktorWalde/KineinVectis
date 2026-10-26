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
                      "interfaces": [{"mode": "dfu", "name": name, "vid": "0483", "pid": "df11",
                                      "serial": "FIXTURE-1", "path": "1-2", "configuration": 1,
                                      "interface": 0, "alternate": 0, "release": "2200", "deviceNumber": 2}], "rawOutput": name})


def failed(request_id, message):
    send(request_id, error={"code": "INVALID_REQUEST", "message": message})


for line in sys.stdin:
    request = json.loads(line)
    request_id = request["id"]
    method = request["method"]
    if method == "core.ping":
        send(request_id, {"protocolVersion": "0.194.0"})
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
        elif sequence == 9:
            listed(request_id, "native-choice")
        else:
            failed(request_id, "unexpected-query")
    elif method == "runConfig.flashProposal":
        params = request["params"]
        assert params == {"native": True, "engine": "dfu-util", "dfu": {
            "image": "firmware a'b.dfu", "destination": {"vid": "0483", "pid": "df11",
                "serial": "FIXTURE-1", "path": "1-2", "configuration": 1, "interface": 0, "alternate": 0}}}, params
        send(request_id, {"command": "literal dfu fixture", "engine": "dfu-util",
                          "flash": {"version": 1, "selection": params}})
    else:
        send(request_id, {})
