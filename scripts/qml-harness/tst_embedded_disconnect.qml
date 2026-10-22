import QtQuick
import KineinVectis

// Real controllers, routers and surface; no device or external tool is opened.
Item {
    id: root
    width: 360
    height: 680
    property int failures: 0
    property int calls: 0
    property int beforeReconnect: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error(message); }
    }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const found = find(child, name);
            if (found) return found;
        }
        return null;
    }
    QtObject {
        id: client
        property bool connected: false
        signal statusChanged()
        signal probesResolved(var probes, bool toolAvailable, string rawOutput, string hint)
        signal buildSizeResolved(var sections, var regions, bool toolAvailable, string tool, string rawOutput)
        signal projectModelResolved(var model)
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
        function probeList() { root.calls++; }
        function buildSize(program) { root.calls++; }
        function serialList() { root.calls++; }
        function projectModel() { root.calls++; }
        function serialMonitor(device, baud) { root.calls++; }
        function serialIdentify(device) { root.calls++; }
        function runConfigFlashProposal(device, engine, size, firmware) { return ++root.calls; }
        function serialAccess(device) { root.calls++; }
        function serialFiles(device, action, path, local) { root.calls++; }
    }
    EmbeddedController { id: controller; workspaceRoot: "/fixture" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    EmbeddedPanel { id: panel; anchors.fill: parent; controller: controller }
    Component.onCompleted: {
        controller.open();
        controller.identity.identify("/fixture-port");
        controller.files.list("/fixture-port");
        controller.flash.propose("/fixture-port", 0);
        controller.access.diagnose("/fixture-port");
        controller.measureSize();
        controller.openMonitor("/fixture-port");
        check(calls === 0, "Pedido desconectado nunca chega à ponte parada");
        check(!controller.busy && !controller.portsBusy && !controller.projectBusy
              && !controller.sizeBusy && !controller.identity.busy && !controller.files.busy
              && !controller.flash.busy && !controller.access.busy,
              "Recusa de transporte termina todas as esperas antes de qualquer aceite");
        check(controller.errorText !== "" && controller.identity.errorText !== ""
              && controller.files.errorText !== "" && controller.flash.errorText !== "",
              "Recusas chegam aos donos existentes");
        client.connected = true;
        client.statusChanged();
        check(calls === 0, "Reconectar não repete gestos recusados");
        controller.open();
        controller.selectedPort = "/fixture-port";
        controller.tab = "board";
        controller.identity.identify("/fixture-port");
        client.serialIdentifyStarted("job_1", "fake-tool");
        controller.files.list("/fixture-port");
        client.serialFilesStarted("job_2", "fake-files");
        controller.files.remove("old.py");
        controller.flash.engine = "esptool";
        controller.flash.firmware = "chosen-firmware";
        controller.flash.propose("/fixture-port", 0);
        controller.sizeBusy = true;
        client.connected = false;
        client.statusChanged();
        check(!controller.busy && !controller.portsBusy && !controller.projectBusy
              && !controller.sizeBusy && !controller.identity.busy && !controller.files.busy
              && !controller.flash.busy, "Queda após aceite não deixa nenhuma espera presa");
        check(!controller.files.hasPending && !controller.flash.found,
              "Confirmação de escrita e prévia não sobrevivem à conexão");
        check(!controller.portsMeasured && controller.boardVerdict === "Dispositivos a verificar",
              "Sem snapshot, painel não declara que a placa está desconectada");
        check(controller.selectedPort === "/fixture-port" && controller.tab === "board"
              && controller.flash.engine === "esptool" && controller.flash.firmware === "chosen-firmware",
              "Escolhas e navegação são preservadas");
        client.flashProposalResolved({ command: "stale command" }, -1);
        client.serialIdentified({ device: "/fixture-port", success: true, identity: { chip: "old" } });
        client.serialFilesResolved({ device: "/fixture-port", success: true, action: "list", entries: [] });
        check(!controller.flash.found && !controller.identity.found && !controller.files.listed,
              "Resultados tardios não restauram ações do core anterior");
        beforeReconnect = calls;
        Qt.callLater(checkSurface);
    }
    function checkSurface() {
        const error = find(panel, "embeddedOperationError");
        check(error.visible && error.message === controller.errorText && !error.codeFont
              && error.width <= panel.width, "Causa legível aparece na faixa existente");
        client.connected = true;
        client.statusChanged();
        check(calls === beforeReconnect, "Recuperação após aceite não repete operação");
        controller.refresh();
        check(calls === beforeReconnect + 3 && controller.errorText === "",
              "Somente novo gesto atualiza dados após reconexão");
        client.requestFailed("serial.list", "causa específica do core");
        check(!controller.portsBusy && controller.errorText === "causa específica do core",
              "Erro do backend mantém sua causa");
        client.probesResolved([], true, "", "");
        check(controller.errorText === "causa específica do core",
              "Resposta de outra consulta não apaga a falha da porta");
        client.serialPortsResolved([], "diagnóstico da consulta");
        check(controller.portsMeasured && controller.boardVerdict === "Nenhum dispositivo listado",
              "Lista vazia recebida é distinta de consulta ainda não feita");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
