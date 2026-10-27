import QtQuick
import KineinVectis

// Retorno da aba Placa (0.4.1h4b2b4c, roadmaps/52 §14.37): o botão do chip
// sugerido reflete o kit, e o monitor aberto não herda a recusa de uma
// execução anterior. Controllers, roteador e superfície reais; ponte falsa.
Item {
    id: root
    width: 420
    height: 900
    property int failures: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("FALHOU: " + message); }
    }
    // Por índice: o Qt 6.4 do AppImage não itera children com for..of (52 §14.27).
    function find(item, name) {
        if (item.objectName === name) return item;
        for (let i = 0; i < item.children.length; ++i) {
            const found = find(item.children[i], name);
            if (found) return found;
        }
        return null;
    }
    QtObject {
        id: client
        property bool connected: true
        signal statusChanged()
        signal workspaceChanged()
        signal picotoolListResolved(var result, double requestId)
        signal picotoolListFailed(double requestId, string message)
        signal dfuListResolved(var result, double requestId)
        signal dfuListFailed(double requestId, string message)
        signal probesResolved(var probes, bool toolAvailable, string rawOutput, string hint)
        signal buildSizeResolved(var sections, var regions, bool toolAvailable, string tool, string rawOutput)
        signal projectModelResolved(var model)
        signal cmakeTargetsResolved(var targets, string origin)
        signal projectChanged(var model)
        signal serialPortsResolved(var ports, string hint)
        signal serialMonitorOpened(string id, string command, string tool)
        signal serialIdentifyStarted(string jobId, string command)
        signal serialIdentified(var outcome)
        signal flashProposalResolved(var proposal, double requestId)
        signal flashProposalFailed(double requestId, string message)
        signal serialAccessResolved(var channels)
        signal serialFilesStarted(string jobId, string command)
        signal serialFilesResolved(var outcome)
        signal requestFailed(string method, string message)
    }
    QtObject {
        id: toolchain
        property string chip: ""
        property string sysroot: ""
        property string targetTriple: ""
        function installedFirmwares() { return []; }
        function applyKit(sysroot, triple, chip) { toolchain.chip = chip; }
    }
    RuntimeController { id: runtime; workspaceRoot: "/fixture" }
    EmbeddedController { id: controller; workspaceRoot: "/fixture" }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller; runtimeController: runtime }
    Connections {
        target: controller.identity
        function onKitChipRequested(chip) { toolchain.applyKit("", "", chip); }
    }
    EmbeddedPanel { id: panel; anchors.fill: parent; controller: controller; toolchainController: toolchain }

    Component.onCompleted: {
        controller.open();
        controller.tab = "board";
        controller.identity.identify("/fixture-port");
        client.serialIdentified({ device: "/fixture-port", success: true,
                                  identity: { chip: "ESP32", chipDescription: "ESP32-D0WD-V3" },
                                  target: { chip: "esp32", flashEngine: "esptool" } });
        Qt.callLater(checkSurface);
    }
    function checkSurface() {
        const apply = find(panel, "identityApplyChip");
        check(apply !== null && apply.visible, "o botão do chip sugerido aparece");
        if (apply !== null) {
            check(apply.enabled && apply.text === qsTr("Usar chip no kit"), "kit sem chip: o botão aplica");
            apply.clicked();
            check(toolchain.chip === "esp32", "o clique pede o chip sugerido ao kit");
            check(!apply.enabled && apply.text === qsTr("Chip no kit"), "o kit com o chip sugerido aparece na tela");
            toolchain.chip = "esp32s3";
            check(apply.enabled && apply.text === qsTr("Usar chip no kit"), "outro chip no kit: o botão volta a aplicar");
        }

        runtime.runErrorText = "Contexto, ferramenta, porta ou arquivos mudaram desde a prévia.";
        client.serialMonitorOpened("monitor-1", "idf.py monitor", "idf.py");
        check(runtime.runErrorText === "", "o monitor aberto não fica sob a recusa anterior");
        check(runtime.terminalsModel.count === 1 && runtime.terminalsModel.get(0).termId === "monitor-1",
              "o monitor abre a aba dele");
        client.requestFailed("serial.monitor", "porta ocupada");
        check(controller.errorText === "porta ocupada" && runtime.terminalsModel.count === 1,
              "a falha do monitor continua no painel de Embarcados");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
