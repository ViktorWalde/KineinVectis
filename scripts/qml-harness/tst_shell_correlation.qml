import QtQuick
import KineinVectis

// The real controllers and both routers, with a transport returning request ids.
// No shell is opened; input is recorded to verify its exact destination.
Item {
    id: root
    width: 360
    height: 240
    property int failures: 0
    property var inputs: []
    property int timedOutRequest: -1
    property int freshRequest: -1
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
        property int sequence: 0
        property bool available: true
        property bool connected: true
        signal statusChanged()
        signal terminalOpened(string id, string shell, double requestId)
        signal terminalOpenFailed(double requestId, string message)
        signal terminalRender(var render)
        signal terminalSelectionCopied(string id, string selectionId, string text, bool valid)
        signal terminalClosed(string id, int exitCode)
        signal runConfigsResolved(var configs, string activeId)
        signal runStarted(string command, string terminalId, string executionKey, string workspace)
        signal runFinished(bool success, int exitCode)
        signal requestFailed(string method, string message, string code)
        function terminalOpen(cwd) { return available ? ++sequence : -1; }
        function terminalInput(id, data) { root.inputs.push({ id: id, data: data }); }
    }
    RuntimeController { id: runtime; workspaceRoot: "/test"; shellTimeoutMs: 80 }
    RunConfigController { id: configs }
    RuntimeRequestRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    RuntimeEventRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    TerminalPanel { id: panel; anchors.fill: parent; runtimeController: runtime }

    Component.onCompleted: {
        const otherRequest = client.terminalOpen("");
        runtime.runInNewTerminal("printf first");
        const requested = client.sequence;
        runtime.handleTerminalOpened("monitor", "tio", "Monitor serial");
        client.terminalOpened("other-shell", "sh", otherRequest);
        client.terminalOpenFailed(otherRequest, "falha de outro pedido");
        check(inputs.length === 0 && runtime.pendingShellInput === "printf first",
              "Monitor, shell e falha não relacionados não consomem a linha");
        runtime.runInNewTerminal("printf second");
        check(client.sequence === requested && runtime.pendingShellInput === "printf first"
              && runtime.shellErrorText !== "", "Segundo clique é recusado sem trocar o primeiro comando");
        client.terminalOpened("requested", "sh", requested);
        client.terminalOpened("requested", "sh", requested);
        check(inputs.length === 1 && inputs[0].id === "requested" && inputs[0].data === "printf first\n",
              "Somente a resposta correspondente recebe o comando, uma vez");
        runtime.runInNewTerminal("printf second");
        check(runtime.shellErrorText === "", "Nova tentativa limpa o erro anterior");
        client.terminalOpenFailed(client.sequence, "limite de sessões atingido");
        check(runtime.pendingShellInput === "" && runtime.shellErrorText === "limite de sessões atingido",
              "Falha correspondente libera espera e preserva a causa");
        runtime.handleCoreDisconnected();
        check(runtime.shellErrorText === "limite de sessões atingido",
              "Desconexão após a recusa preserva a causa já recebida");
        Qt.callLater(root.checkErrorPanel);
    }
    function checkErrorPanel() {
        const error = find(panel, "terminalCommandError");
        const viewport = find(panel, "terminalViewport");
        check(error.visible && error.message === "limite de sessões atingido", "Erro aparece no painel existente");
        check(viewport.y >= error.height && viewport.height > 0 && viewport.height < panel.height,
              "Mensagem não sobrepõe o grid e deixa área útil");
        client.available = false;
        runtime.runInNewTerminal("printf unavailable");
        check(runtime.pendingShellInput === "" && runtime.shellErrorText.indexOf("indisponível") >= 0,
              "Pedido que não sai do cliente não fica aguardando");
        client.available = true;
        runtime.runInNewTerminal("printf timeout");
        timedOutRequest = client.sequence;
        afterTimeout.start();
    }
    Timer {
        id: afterTimeout
        interval: 180
        onTriggered: {
            root.check(runtime.pendingShellInput === "" && runtime.shellErrorText !== "", "Timeout descarta a linha");
            runtime.runInNewTerminal("printf fresh");
            root.freshRequest = client.sequence;
            client.terminalOpened("late", "sh", root.timedOutRequest);
            root.check(root.inputs.length === 1 && runtime.pendingShellInput === "printf fresh",
                       "Resposta após timeout não rouba o pedido novo");
            runtime.workspaceRoot = "/another";
            client.terminalOpened("old-workspace", "sh", root.freshRequest);
            root.check(root.inputs.length === 1 && runtime.pendingShellInput === "", "Troca de projeto cancela a linha");
            runtime.runInNewTerminal("printf disconnected");
            const disconnected = client.sequence;
            client.connected = false;
            client.statusChanged();
            client.terminalOpened("old-core", "sh", disconnected);
            root.check(root.inputs.length === 1 && runtime.pendingShellInput === "", "Queda do core cancela a linha");
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
