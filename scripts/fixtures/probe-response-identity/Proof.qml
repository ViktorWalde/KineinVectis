import QtQuick
import KineinVectis

Item {
    id: root
    property int phase: 0
    property var names: []
    property var failures: []
    function require(condition, message) {
        if (condition) return true;
        console.error("PROBE_RESPONSE_FAIL " + message);
        Qt.exit(1);
        return false;
    }
    CoreClient { id: client }
    Connections {
        target: client
        function onStatusChanged() {
            if (client.connected && root.phase === 0) {
                root.phase = 1;
                client.openWorkspace(proofWorkspace);
            }
        }
        function onWorkspaceChanged() {
            if (root.phase === 1 && client.workspaceRoot !== "") {
                root.phase = 2;
                client.probeList();
                client.probeList();
                step.start();
            }
        }
        function onProbesResolved(probes, toolAvailable, rawOutput, hint) {
            root.names.push(probes.length > 0 ? probes[0].name : "empty");
        }
        function onRequestFailed(method, message, code) {
            if (method === "probe.list") root.failures.push(message);
        }
    }
    Timer {
        id: step
        interval: 150
        onTriggered: root.advance()
    }
    function advance() {
        if (phase === 2) {
            if (!require(names.length === 1 && names[0] === "newest-success",
                         "old success replaced the latest query: " + JSON.stringify(names))) return;
            if (!require(failures.length === 0, "unexpected initial failure")) return;
            phase = 3;
            client.probeList();
            client.probeList();
        } else if (phase === 3) {
            if (!require(names.length === 2 && names[1] === "newest-before-stale-error"
                         && failures.length === 0, "old failure escaped to current consumers")) return;
            phase = 4;
            client.probeList();
            client.closeWorkspace();
        } else if (phase === 4) {
            if (!require(names.length === 2 && failures.length === 0 && client.workspaceRoot === "",
                         "response after workspace close escaped")) return;
            phase = 5;
            client.probeList();
            client.openWorkspace(proofWorkspace);
        } else if (phase === 5) {
            if (!require(names.length === 2 && failures.length === 0 && client.workspaceRoot === proofWorkspace,
                         "response after workspace open escaped")) return;
            phase = 6;
            client.probeList();
        } else if (phase === 6) {
            if (!require(failures.length === 1 && failures[0] === "current-query-failure",
                         "current failure was lost or duplicated")) return;
            phase = 7;
            client.probeList();
        } else if (phase === 7) {
            if (!require(names.length === 3 && names[2] === "retry-success" && failures.length === 1,
                         "retry did not deliver exactly one result")) return;
            phase = 8;
            client.probeList();
            client.openWorkspace(proofWorkspace + "/rejected");
        } else if (phase === 8) {
            if (!require(names.length === 4 && names[3] === "survives-open-failure"
                         && failures.length === 1, "failed workspace open orphaned the current query")) return;
            phase = 9;
            client.probeList();
            client.openWorkspace(proofWorkspace);
        } else if (phase === 9) {
            if (!require(names.length === 5 && names[4] === "survives-same-root"
                         && failures.length === 1, "same-root reopen orphaned the current query")) return;
            console.log("PROBE_RESPONSE_REPORT " + JSON.stringify({accepted: names, failures: failures}));
            Qt.exit(0);
            return;
        }
        step.start();
    }
    Component.onCompleted: client.start()
}
