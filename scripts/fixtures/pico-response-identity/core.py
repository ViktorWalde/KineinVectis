#!/usr/bin/env python3
"""Reorder real IPC responses without invoking any USB tool."""
import json
import sys

pending = None
sequence = 0
workspace = ""
intent = None


def send(request_id, result=None, error=None):
    response = {"jsonrpc": "2.0", "id": request_id}
    response.update({"error": error} if error else {"result": result})
    print(json.dumps(response), flush=True)


def listed(request_id, name):
    send(request_id, {"complete": True, "toolAvailable": True,
        "devices": [{"chip": "rp2040", "serialSelector": name, "flashSizeBytes": 2097152}], "rawOutput": name})


def failed(request_id, message):
    send(request_id, error={"code": "INVALID_REQUEST", "message": message})


for line in sys.stdin:
    request = json.loads(line)
    request_id = request["id"]
    method = request["method"]
    if method == "core.ping":
        send(request_id, {"protocolVersion": "0.197.0"})
    elif method == "workspace.open":
        workspace = request["params"]["path"]
        send(request_id, {"root": request["params"]["path"], "name": "Pico fixture", "kind": "cmake",
                          "capabilities": {"buildSystems": []}})
        if pending is not None:
            listed(pending, "stale-after-open")
            pending = None
    elif method == "workspace.close":
        send(request_id, {})
        if pending is not None:
            failed(pending, "stale-after-close")
            pending = None
    elif method == "picotool.list":
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
            listed(request_id, "E660583883355B21")
        else:
            failed(request_id, "unexpected-query")
    elif method == "runConfig.flashProposal":
        expected = {"native": True, "engine": "picotool", "target": "firmware", "pico": {
            "image": ".kinein/build/firmware a'b.uf2", "destination": {
                "chip": "rp2040", "serialSelector": "E660583883355B21", "flashSizeBytes": 2097152}}}
        assert request["params"] == expected, request["params"]
        intent = {"version": 1, "workspace": workspace, "selection": expected,
                  "program": "/fixture/picotool", "args": ["load", "--verify"], "deviceKey": "pico", "fingerprint": "a"*64}
        send(request_id, {"name": "Gravar UF2", "command": "literal UF2 argv", "engine": "picotool", "source": [], "warnings": [], "flash": intent})
    elif method == "runConfig.save":
        assert request["params"] == {"name": "Gravar UF2", "command": "literal UF2 argv", "flash": intent}, request["params"]
        send(request_id, {})
    elif method == "run.start":
        assert request["params"] == {"flash": intent}, request["params"]
        send(request_id, {"command": "literal UF2 argv", "terminalId": "pico-native-terminal", "executionKey": "pico-proof", "workspace": workspace})
    else:
        send(request_id, {})
