import QtQuick
import KineinVectis

// Production bridge, routers and controller; controlled IPC producer.
Item {
    id: root
    property int phase: 0
    property int applyCount: 0
    property int failures: 0
    readonly property string actionId: "cmake.setCStandard"

    function require(condition, message) {
        if (condition) return true;
        console.error("CONFIGACTION_RESPONSE_FAIL " + message);
        Qt.exit(1);
        return false;
    }
    CoreClient { id: client }
    ConfigActionController { id: controller; workspaceRoot: client.workspaceRoot }
    ConfigActionRequestRouter { coreClient: client; configActionController: controller }
    ConfigActionEventRouter { coreClient: client; configActionController: controller }
    Connections {
        target: client
        function onStatusChanged() {
            if (client.connected && root.phase === 0) {
                root.phase = 1;
                client.openWorkspace(proofWorkspace);
            }
        }
        function onConfigActionsListed(actions, activeBuildSystems) {
            if (root.phase === 1) {
                root.phase = 2;
                Qt.callLater(function() {
                    controller.select(root.actionId);
                    controller.setParam("target", "app");
                    root.query("11"); root.query("23"); step.start();
                });
            }
        }
        function onRequestFailed(method, message) {
            if (method === "configAction.preview") root.failures += 1;
        }
    }
    Connections {
        target: controller
        function onApplyRequested(id, params, expected) { root.applyCount += 1; }
    }
    function query(standard) {
        controller.setParam("standard", standard);
        controller.requestPreview();
    }
    Timer { id: step; interval: 150; onTriggered: root.advance() }
    function advance() {
        if (phase === 2) {
            if (!require(controller.previewSummary === "latest-23" && failures === 0,
                         "stale success replaced the latest preview")) return;
            phase = 3;
            query("11"); query("17");
        } else if (phase === 3) {
            if (!require(controller.previewSummary === "latest-17" && controller.errorText === "" && failures === 0,
                         "stale failure revoked current consent")) return;
            phase = 4;
            query("90");
        } else if (phase === 4) {
            controller.apply();
            if (!require(controller.previewId === "" && controller.errorText === "current-query-failure"
                         && failures === 1 && applyCount === 0, "current failure allowed an apply")) return;
            phase = 5;
            query("23");
        } else if (phase === 5) {
            if (!require(controller.previewSummary === "retry-23" && controller.errorText === "",
                         "retry did not restore the preview")) return;
            phase = 6;
            controller.apply();
        } else if (phase === 6) {
            if (!require(applyCount === 1 && controller.statusText === "fixture-applied" && controller.errorText === "",
                         "apply did not carry latest parameters and snapshot")) return;
            console.log("CONFIGACTION_RESPONSE_REPORT " + JSON.stringify({phases: 5, applies: applyCount, failures: failures}));
            Qt.exit(0);
            return;
        }
        step.start();
    }
    Component.onCompleted: client.start()
}
