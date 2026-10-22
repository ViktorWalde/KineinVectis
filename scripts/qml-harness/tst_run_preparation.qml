import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property int stops: 0
    property int starts: 0
    function check(condition, message) {
        if (!condition) { failures++; console.error("FALHAS: " + message); }
    }
    RuntimeController {
        id: runtime
        workspaceRoot: "/project"
        onRunStartRequested: root.starts++
        onRunStopRequested: root.stops++
    }
    Component.onCompleted: {
        runtime.startRun("");
        check(runtime.runPending && !runtime.running, "espera começa antes de existir terminal");
        runtime.stopRun();
        check(stops === 1, "Parar cancela também durante preparação");
        runtime.stopRun(); runtime.startRun("");
        check(stops === 1 && starts === 1, "gestos repetidos não duplicam pedidos");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
