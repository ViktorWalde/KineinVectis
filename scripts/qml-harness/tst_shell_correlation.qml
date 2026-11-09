import QtQuick
import KineinVectis

// The real controllers and both routers, with a transport returning request ids.
// No shell is opened; requests are recorded to verify where the line goes. Since
// 0.205.0 (INF-02) the line travels in terminal.open and the UI never types it.
Item {
    id: root
    width: 360
    height: 240
    property int failures: 0
    property var inputs: []
    property var opens: []
    property var pendingActions: []
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
        signal terminalOpened(string id, string shell, double requestId, string commandState)
        signal terminalCommandSettled(string id, string state, string reason)
        signal terminalOpenFailed(double requestId, string message)
        signal terminalRender(var render)
        signal terminalSelectionCopied(string id, string selectionId, string text, bool valid)
        signal terminalClosed(string id, int exitCode)
        signal runConfigMutationFinished(double requestId)
        signal runConfigMutationFailed(double requestId, string message)
        signal runConfigsResolved(var configs, string activeId)
        signal runStarted(string command, string terminalId, string executionKey, string workspace, double requestId)
        signal runRequestFailed(double requestId, string method, string message)
        signal runStopFinished(double requestId)
        signal workspaceChanged()
        signal runFinished(bool success, int exitCode)
        signal requestFailed(string method, string message, string code)
        function terminalOpen(cwd, command) {
            if (!available) return -1;
            root.opens.push(command === undefined ? "" : command);
            return ++sequence;
        }
        function terminalPendingCommand(id, action) { root.pendingActions.push(id + "|" + action); }
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
        check(opens.length === 2 && opens[0] === "" && opens[1] === "printf first",
              "A linha vai no terminal.open pedido, e só nele");
        runtime.handleTerminalOpened("monitor", "tio", "Monitor serial");
        client.terminalOpened("other-shell", "sh", otherRequest, "");
        client.terminalOpenFailed(otherRequest, "falha de outro pedido");
        check(inputs.length === 0 && runtime.pendingShellInput === "printf first"
              && runtime.pendingCommandFor("other-shell") === "",
              "Monitor, shell e falha não relacionados não consomem a linha");
        runtime.runInNewTerminal("printf second");
        check(client.sequence === requested && runtime.pendingShellInput === "printf first"
              && runtime.shellErrorText !== "", "Segundo clique é recusado sem trocar o primeiro comando");
        client.terminalOpened("requested", "sh", requested, "waiting");
        client.terminalOpened("requested", "sh", requested, "waiting");
        check(inputs.length === 0 && runtime.pendingShellInput === ""
              && runtime.pendingCommandFor("requested") === "printf first" && pendingActions.length === 0,
              "Somente a resposta correspondente guarda o comando, uma vez, sem digitá-lo");
        client.terminalCommandSettled("requested", "sent", "prompt");
        check(runtime.pendingCommandFor("requested") === "", "O prompt encerra a espera");
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
            client.terminalOpened("late", "sh", root.timedOutRequest, "waiting");
            root.check(runtime.pendingShellInput === "printf fresh" && runtime.pendingCommandFor("late") === ""
                       && root.pendingActions.length === 1 && root.pendingActions[0] === "late|discard",
                       "Resposta após timeout não rouba o pedido novo e o core descarta a linha avisada como não enviada");
            const timeoutText = runtime.shellErrorText;
            client.terminalOpened("late-sent", "sh", root.timedOutRequest, "sent");
            root.check(runtime.shellErrorText !== timeoutText && runtime.shellErrorText.indexOf("depois do prazo") >= 0,
                       "Envio tardio por shell sem integração corrige o aviso");
            runtime.workspaceRoot = "/another";
            client.terminalOpened("old-workspace", "sh", root.freshRequest, "waiting");
            root.check(runtime.pendingShellInput === "" && runtime.pendingCommandFor("old-workspace") === ""
                       && root.pendingActions[root.pendingActions.length - 1] === "old-workspace|discard",
                       "Troca de projeto cancela a linha");
            runtime.runInNewTerminal("printf disconnected");
            const disconnected = client.sequence;
            client.connected = false;
            client.statusChanged();
            client.terminalOpened("old-core", "sh", disconnected, "waiting");
            root.check(runtime.pendingShellInput === "" && runtime.pendingCommandFor("old-core") === "",
                       "Queda do core cancela a linha");
            root.check(root.inputs.length === 0, "A UI nunca digita a linha");
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
