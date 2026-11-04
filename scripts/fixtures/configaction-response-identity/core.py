#!/usr/bin/env python3
"""Controlled configuration previews, deliberately delivered out of order."""
import json
import sys

sequence = 0
pending = None
action_id = "cmake.setCStandard"


def response(request_id, result=None, error=None):
    message = {"jsonrpc": "2.0", "id": request_id}
    message["error" if error else "result"] = error if error else result
    print(json.dumps(message), flush=True)


def failure(request_id, message):
    response(request_id, error={"code": "INVALID_PARAMS", "message": message})


def preview(request_id, standard, summary):
    response(request_id, {"id": action_id, "title": "C standard", "summary": summary,
                          "files": [{"path": "CMakeLists.txt", "before": "original-" + standard,
                                     "after": "selected-" + standard}], "notes": [], "report": []})


for line in sys.stdin:
    request = json.loads(line)
    request_id = request.get("id")
    method = request.get("method")
    params = request.get("params", {})
    if request_id is None:
        continue
    if method == "core.ping":
        response(request_id, {"protocolVersion": "0.199.0"})
    elif method == "workspace.open":
        response(request_id, {"root": params["path"], "name": "C fixture", "kind": "cmake",
                              "capabilities": {"buildSystems": ["cmake"]}})
    elif method == "configAction.list":
        response(request_id, {"activeBuildSystems": ["cmake"], "actions": [{
            "id": action_id, "title": "C standard", "description": "Select C standard",
            "scope": "cmake", "category": "CMake", "effect": "edit", "risk": "medium",
            "riskLabel": "configuration", "riskExplanation": "edits configuration",
            "state": "available", "affects": ["CMakeLists.txt"], "docs": [],
            "params": [{"name": "target", "label": "Target", "required": True},
                       {"name": "standard", "label": "Standard", "required": True}]
        }]})
    elif method == "configAction.preview":
        sequence += 1
        if sequence in (1, 3):
            pending = request_id
        elif sequence == 2:
            preview(request_id, "23", "latest-23")
            preview(pending, "11", "stale-11")
        elif sequence == 4:
            preview(request_id, "17", "latest-17")
            failure(pending, "stale-query-failure")
        elif sequence == 5:
            failure(request_id, "current-query-failure")
        elif sequence == 6:
            preview(request_id, "23", "retry-23")
        else:
            failure(request_id, "unexpected-query")
    elif method == "configAction.apply":
        if params.get("params", {}).get("standard") != "23" or params.get("expected") != [
            {"path": "CMakeLists.txt", "content": "original-23"}
        ]:
            failure(request_id, "incorrect-consent")
        else:
            response(request_id, {"id": action_id, "message": "fixture-applied", "files": ["CMakeLists.txt"]})
    else:
        response(request_id, {})
