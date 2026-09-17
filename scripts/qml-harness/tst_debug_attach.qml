import QtQuick
import KineinVectis

Item {
    id: root
    property var requests: []
    property int stops: 0
    QtObject {
        id: bridge
        function debugStart(program, connect) { root.requests.push({ program: program, connect: connect }); }
        function debugStop() { root.stops++; }
    }
    DebugController { id: controller }
    DebugRequestRouter { coreClient: bridge; debugController: controller }
    DebugAttachForm {
        id: form
        available: controller.workspaceRoot !== "" && !controller.starting && !controller.sessionActive
        onAttachRequested: function(host, port) { controller.startDebug("", { host: host, port: port }); }
    }
    Component.onCompleted: {
        let failures = [];
        controller.startDebug("", { host: "localhost", port: 5678 });
        if (requests.length !== 0 || form.available) failures.push("sem workspace");
        controller.workspaceRoot = "/tmp/ws";
        let fields = [];
        let button = null;
        for (let i = 0; i < form.children.length; i++) {
            const child = form.children[i];
            if (child.label !== undefined) fields.push(child);
            if (child.text === "Conectar ao Python") button = child;
        }
        if (fields.length !== 2 || !button) {
            console.error("campos/botao de attach ausentes");
            Qt.exit(1);
            return;
        }
        fields[0].edited("127.0.0.1");
        fields[1].edited("9123");
        button.clicked();
        if (requests.length !== 1 || requests[0].program !== "" || requests[0].connect.host !== "127.0.0.1" || requests[0].connect.port !== 9123) failures.push("endpoint perdeu-se no router");
        controller.startDebug("", { host: "localhost", port: 5678 });
        if (requests.length !== 1 || !controller.starting || form.available) failures.push("pedido duplicado");
        controller.handleRequestFailed("debug.start", "conexao recusada");
        if (controller.starting || !form.available || controller.sessionActive) failures.push("falha bloqueou retry");
        fields[0].edited("localhost");
        fields[1].edited("5678");
        fields[1].accepted();
        controller.handleStarted("debugpy localhost:5678", true);
        if (!controller.attached || controller.starting || form.available) failures.push("modo attach");
        controller.stopDebug();
        if (stops !== 1) failures.push("detach nao saiu");
        controller.handleFinished(0);
        if (controller.attached || controller.sessionActive || !form.available) failures.push("fim nao limpou estado");
        controller.startDebug("/tmp/ws/app.py");
        if (requests.length !== 3 || requests[2].program !== "/tmp/ws/app.py" || Object.keys(requests[2].connect).length !== 0) failures.push("launch mudou");
        controller.clear();
        if (controller.starting || controller.attached) failures.push("clear nao limpou estado");
        if (failures.length) console.error(failures.join("; "));
        Qt.exit(failures.length ? 1 : 0);
    }
}
