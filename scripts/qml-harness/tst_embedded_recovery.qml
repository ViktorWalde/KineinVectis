import QtQuick
import KineinVectis

// Regression: a blocked device remains selectable and offers visible recovery.
// The core owns the diagnosis; informational steps never launch a command.
Item {
    id: root
    width: 400
    height: 680
    property int failures: 0
    property var requests: []
    property var commands: []
    property int refreshes: 0
    EmbeddedController { id: controller }
    EmbeddedPanel { id: panel; anchors.fill: parent; controller: controller }
    Connections {
        target: controller.access
        function onDiagnoseRequested(device) { root.requests.push(device); }
        function onCommandRequested(command) { root.commands.push(command); }
    }
    Connections {
        target: controller
        function onSerialListRequested() { root.refreshes++; }
    }
    function check(ok, message) {
        if (!ok) { failures++; console.error(message); }
    }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
    function port(ready) {
        return { device: "/dev/ttyUSB0", vid: "10c4", pid: "ea60", product: "CP2102",
            access: { readableWritable: ready, state: ready ? "ready" : "sessionRefreshRequired",
                mode: "crw-rw----", group: "dialout", hint: ready ? "" : "Sua conta já está cadastrada; atualize a sessão do sistema." } };
    }
    function channel(ok, command) {
        return { kind: "serial", device: "/dev/ttyUSB0", ok: ok,
            detail: "crw-rw---- · grupo dialout", problem: ok ? "" : "diagnóstico do core",
            fix: ok ? undefined : { steps: [{ explanation: "Salve o trabalho e entre novamente na sessão do sistema.", command: command }], sourceUrl: "https://docs.espressif.com/", checkedOn: "2026-10-20" } };
    }
    function checkPreparation() {
        check(find(panel, "embeddedSelectPort").enabled, "Seleção não exige abrir a porta");
        find(panel, "embeddedSelectPort").toggled();
        check(controller.selectedPort === "/dev/ttyUSB0", "Destino bloqueado pode ser escolhido");
        check(!find(panel, "embeddedIdentify").enabled && !find(panel, "embeddedMonitor").enabled,
              "Operações não passam sem acesso");
        find(panel, "embeddedPrepareAccess").clicked();
        check(requests.join(",") === "/dev/ttyUSB0" && controller.access.busy, "Preparo solicita diagnóstico da porta");
        controller.access.handleChannels([channel(false, ""), { kind: "probe", ok: false, detail: "outra preparação" }]);
        Qt.callLater(checkSessionInstructions);
    }
    function checkSessionInstructions() {
        check(controller.access.visibleChannels.length === 1 && controller.access.problems === 1,
              "Preparo de serial não mistura sondas sem relação");
        const command = find(panel, "embeddedAccessCommand");
        check(command && !command.visible && commands.length === 0, "Instrução de sessão não tem comando executável");
        const details = find(panel, "embeddedAccessDetails");
        check(details.text === "Detalhes técnicos", "Evidência técnica começa recolhida");
        details.clicked();
        check(details.text === "Ocultar detalhes técnicos", "Evidência pode ser consultada");
        controller.errorText = "recusa antiga do monitor";
        find(panel, "embeddedRecheckAccess").clicked();
        check(controller.errorText === "", "Reverificação limpa recusa antiga antes da nova medição");
        controller.access.recheck();
        check(refreshes === 1 && requests.length === 2 && controller.portsBusy && controller.access.busy,
              "Rever acesso atualiza portas e diagnóstico uma vez");
        controller.handleSerialPorts([port(true)], "");
        controller.access.handleChannels([channel(true, "")]);
        Qt.callLater(checkReady);
    }
    function checkReady() {
        check(find(panel, "embeddedIdentify").enabled && find(panel, "embeddedMonitor").enabled,
              "Novo veredito de acesso libera operações");
        check(!find(panel, "embeddedPrepareAccess").visible && controller.access.problems === 0,
              "Preparação pendente desaparece após sucesso");
        controller.access.diagnose("/dev/ttyUSB0");
        controller.access.handleChannels([channel(false, "id")]);
        root.width = 300;
        Qt.callLater(checkCommand);
    }
    function checkCommand() {
        const command = find(panel, "embeddedAccessCommand");
        check(command.visible && commands.length === 0, "Comando fica disponível sem execução automática");
        check(command.text === "Executar no terminal", "Botão informa execução, sem prometer apenas cópia");
        const position = command.mapToItem(panel, 0, 0);
        check(position.x >= 0 && position.x + command.width <= panel.width, "Recuperação cabe no painel estreito");
        command.clicked();
        check(commands.join(",") === "id", "Clique explícito envia somente o comando do core");
        controller.access.diagnose("/dev/ttyUSB0");
        controller.access.handleFailed("serial.access", "consulta indisponível");
        check(!controller.access.busy && controller.access.errorText === "consulta indisponível", "Falha encerra espera e permite repetir");
        controller.refresh();
        check(!controller.access.diagnosed && controller.access.errorText === "", "Atualização geral remove diagnóstico antigo");
        controller.access.diagnose("/dev/ttyUSB0");
        const sent = requests.length;
        controller.refresh();
        controller.access.diagnose("/dev/ttyUSB0");
        check(controller.access.busy && requests.length === sent, "Consulta antiga impede sobreposição durante limpeza");
        controller.access.handleChannels([channel(false, "id")]);
        check(!controller.access.busy && !controller.access.diagnosed, "Resposta antiga não restaura diagnóstico limpo");
        controller.access.diagnose("/dev/ttyUSB0");
        check(requests.length === sent + 1, "Novo pedido pode ser feito após descarte");
        controller.access.clear();
        controller.access.handleFailed("serial.access", "falha antiga");
        check(!controller.access.busy && controller.access.errorText === "", "Erro antigo também é descartado");
        controller.access.diagnose("/dev/ttyUSB0");
        controller.access.clear(true);
        check(!controller.access.busy && !controller.access.discardReply, "Perda do transporte não deixa espera sem resposta");
        controller.access.handleChannels([channel(false, "id")]);
        check(!controller.access.diagnosed, "Resposta sem consulta vigente é ignorada");
        controller.access.diagnose("/dev/ttyUSB0");
        check(controller.access.busy, "Nova conexão permite consultar novamente");
        Qt.exit(failures === 0 ? 0 : 1);
    }
    Component.onCompleted: {
        controller.tab = "board";
        controller.handleSerialPorts([port(false)], "");
        Qt.callLater(checkPreparation);
    }
}
