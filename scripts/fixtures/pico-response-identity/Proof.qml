import QtQuick
import KineinVectis

// Production CoreClient, IPC routers and controller against reordered replies.
Item {
    id: root
    property int phase: 0
    function require(ok, message) {
        if (ok) return true;
        console.error("PICO_RESPONSE_FAIL " + message);
        Qt.exit(1);
        return false;
    }
    CoreClient { id: client }
    EmbeddedController { id: controller; workspaceRoot: client.workspaceRoot }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
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
                // Run after the event router has invalidated workspace data.
                root.phase = 2;
                Qt.callLater(function() {
                    controller.usb.pico.refresh(); controller.usb.pico.refresh(); step.start();
                });
            }
        }
    }
    Timer { id: step; interval: 150; onTriggered: root.advance() }
    function advance() {
        const pico = controller.usb.pico;
        const name = pico.devices.length > 0 ? pico.devices[0].serialSelector : "";
        if (phase === 2) {
            if (!require(name === "newest-success" && pico.errorText === "" && !pico.busy,
                         "stale success replaced latest query or id was lost")) return;
            phase = 3;
            pico.refresh(); pico.refresh();
        } else if (phase === 3) {
            if (!require(name === "newest-before-stale-error" && pico.errorText === "",
                         "stale failure reached current consumer")) return;
            phase = 4;
            pico.refresh(); client.closeWorkspace();
        } else if (phase === 4) {
            if (!require(!pico.measured && !pico.busy && name === "" && pico.errorText === "",
                         "reply after workspace close escaped")) return;
            phase = 5;
            client.openWorkspace(proofWorkspace);
        } else if (phase === 5) {
            phase = 6;
            pico.refresh();
        } else if (phase === 6) {
            if (!require(!pico.busy && name === "" && pico.errorText === "current-query-failure",
                         "current failure was lost or did not carry its request id")) return;
            phase = 7;
            pico.refresh();
        } else if (phase === 7) {
            if (!require(name === "retry-success" && pico.errorText === "",
                         "retry or duplicate suppression failed")) return;
            phase = 8;
            pico.refresh(); client.openWorkspace(proofWorkspace);
        } else if (phase === 8) {
            if (!require(!pico.measured && !pico.busy && name === "" && pico.errorText === "",
                         "reply after same-root activation escaped")) return;
            console.log("PICO_RESPONSE_REPORT " + JSON.stringify({phases: 8, protocol: client.protocolVersion}));
            Qt.exit(0); return;
        }
        step.start();
    }
    Component.onCompleted: client.start()
}
