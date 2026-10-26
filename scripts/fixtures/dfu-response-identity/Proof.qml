import QtQuick
import KineinVectis

// Production CoreClient, IPC routers and controller against reordered replies.
Item {
    id: root
    property int phase: 0
    property bool nativeRequested: false
    function require(ok, message) {
        if (ok) return true;
        console.error("DFU_RESPONSE_FAIL " + message);
        Qt.exit(1);
        return false;
    }
    CoreClient { id: client }
    EmbeddedController { id: controller; workspaceRoot: client.workspaceRoot }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    Connections {
        target: controller.flash
        function onRunRequested(command, flash) {
            root.nativeRequested = command === "" && flash.selection.dfu.image === "firmware a'b.dfu";
        }
    }
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
                    controller.dfu.refresh(); controller.dfu.refresh(); step.start();
                });
            }
        }
    }
    Timer { id: step; interval: 150; onTriggered: root.advance() }
    function advance() {
        const dfu = controller.dfu;
        const name = dfu.interfaces.length > 0 ? dfu.interfaces[0].name : "";
        if (phase === 2) {
            if (!require(name === "newest-success" && dfu.errorText === "" && !dfu.busy,
                         "stale success replaced latest query or id was lost")) return;
            phase = 3;
            dfu.refresh(); dfu.refresh();
        } else if (phase === 3) {
            if (!require(name === "newest-before-stale-error" && dfu.errorText === "",
                         "stale failure reached current consumer")) return;
            phase = 4;
            dfu.refresh(); client.closeWorkspace();
        } else if (phase === 4) {
            if (!require(!dfu.measured && !dfu.busy && name === "" && dfu.errorText === "",
                         "reply after workspace close escaped")) return;
            phase = 5;
            client.openWorkspace(proofWorkspace);
        } else if (phase === 5) {
            phase = 6;
            dfu.refresh();
        } else if (phase === 6) {
            if (!require(!dfu.busy && name === "" && dfu.errorText === "current-query-failure",
                         "current failure was lost or did not carry its request id")) return;
            phase = 7;
            dfu.refresh();
        } else if (phase === 7) {
            if (!require(name === "retry-success" && dfu.errorText === "",
                         "retry or duplicate suppression failed")) return;
            phase = 8;
            dfu.refresh(); client.openWorkspace(proofWorkspace);
        } else if (phase === 8) {
            if (!require(!dfu.measured && !dfu.busy && name === "" && dfu.errorText === "",
                         "reply after same-root activation escaped")) return;
            phase = 9;
            dfu.refresh();
        } else if (phase === 9) {
            if (!require(dfu.interfaces.length === 1, "native destination listing missing")) return;
            phase = 10;
            controller.flash.selectEngine("dfu-util");
            controller.flash.dfu.setImage("firmware a'b.dfu");
            controller.flash.dfu.select(dfu.interfaces[0]);
            controller.flash.propose("/dev/ignored", 999);
        } else if (phase === 10) {
            if (!require(controller.flash.found && controller.flash.proposal.flash.selection.dfu.destination.serial === "FIXTURE-1",
                         "native DFU selection was lost in real Qt serialization or dispatch")) return;
            controller.flash.run();
            if (!require(nativeRequested, "native DFU intent was converted into a manual command")) return;
            console.log("DFU_RESPONSE_REPORT " + JSON.stringify({phases: 10, protocol: client.protocolVersion}));
            Qt.exit(0);
            return;
        }
        step.start();
    }
    Component.onCompleted: client.start()
}
