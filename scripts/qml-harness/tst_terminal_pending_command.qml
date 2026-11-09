import QtQuick
import KineinVectis

// INF-02 (62 §3.4): the line the core holds until the shell's first prompt.
// Real controller, routers and TerminalPanel; the transport records requests.
Item {
    id: root
    width: 480
    height: 320
    property int failures: 0
    property var inputs: []
    property var pendingActions: []
    property var band: null
    property var viewport: null
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
    function open(command) {
        runtime.runInNewTerminal(command);
        return client.sequence;
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
        function terminalOpen(cwd, command) { return ++sequence; }
        function terminalInput(id, data) { root.inputs.push(id + "|" + data); }
        function terminalPendingCommand(id, action) { root.pendingActions.push(id + "|" + action); }
    }
    RuntimeController { id: runtime; workspaceRoot: "/test"; terminalActive: true }
    RunConfigController { id: configs }
    RuntimeRequestRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    RuntimeEventRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    TerminalPanel {
        id: panel
        anchors.fill: parent
        runtimeController: runtime
        sessionId: runtime.activeTerminalId
        terminalActive: true
        sessionAvailable: runtime.activeTerminalId !== ""
    }

    Component.onCompleted: {
        band = find(panel, "terminalPendingCommand");
        viewport = find(panel, "terminalViewport");
        band.graceMs = 30;
        client.terminalOpened("t1", "bash", open("make run"), "waiting");
        check(runtime.activeTerminalId === "t1" && band.command === "make run",
              "A sessão ativa expõe a linha guardada");
        check(!band.visible, "Um prompt rápido não pisca a faixa");
        firstGrace.start();
    }
    Timer {
        id: firstGrace
        interval: 120
        onTriggered: {
            root.check(root.band.visible && root.band.height > 0, "Depois da carência a faixa aparece");
            root.check(root.find(root.band, "terminalPendingCommandText").text === "make run", "A faixa mostra a linha");
            root.check(root.viewport.y >= root.band.y + root.band.height && root.viewport.height > 0,
                       "A faixa não cobre o grid");
            root.find(root.band, "terminalPendingCommandSend").clicked();
            root.check(root.pendingActions.length === 1 && root.pendingActions[0] === "t1|send",
                       "Enviar agora pede ao core, para esta sessão");
            client.terminalCommandSettled("t1", "sent", "request");
            root.check(!root.band.visible && runtime.pendingCommandFor("t1") === "", "O envio encerra a espera");
            runtime.sendPendingCommand("t1");
            runtime.discardPendingCommand("t1");
            root.check(root.pendingActions.length === 1, "Sem linha guardada, nada é pedido ao core");

            client.terminalOpened("t2", "bash", root.open("ssh pi@host"), "waiting");
            runtime.selectTerminal("t1");
            root.check(root.band.command === "", "Outra sessão não mostra a linha de t2");
            runtime.selectTerminal("t2");
            root.check(root.band.command === "ssh pi@host", "A linha volta com a sessão dona");
            secondGrace.start();
        }
    }
    Timer {
        id: secondGrace
        interval: 120
        onTriggered: {
            root.check(root.band.visible, "Faixa da segunda sessão");
            root.find(root.band, "terminalPendingCommandDiscard").clicked();
            root.check(root.pendingActions[1] === "t2|discard", "Descartar pede ao core");
            client.terminalCommandSettled("t2", "discarded", "request");
            root.check(!root.band.visible && runtime.shellErrorText === "", "Descarte pedido não é erro");

            client.terminalOpened("t3", "bash", root.open("cargo run"), "waiting");
            client.terminalCommandSettled("t3", "discarded", "exit");
            const error = root.find(panel, "terminalCommandError");
            root.check(runtime.pendingCommandFor("t3") === "" && runtime.shellErrorText.indexOf("encerrou") >= 0
                       && error.visible && error.message === runtime.shellErrorText,
                       "Saída antes do prompt aparece como comando não enviado");

            client.terminalOpened("t4", "bash", root.open("ls"), "waiting");
            client.terminalClosed("t4", 0);
            root.check(runtime.pendingCommandFor("t4") === "", "Fechar a sessão descarta a espera");

            client.terminalOpened("t5", "bash", root.open("pwd"), "waiting");
            client.requestFailed("terminal.pendingCommand", "sessão de terminal inexistente ou encerrada", "INVALID_REQUEST");
            root.check(runtime.shellErrorText === "sessão de terminal inexistente ou encerrada",
                       "Falha do pedido fica visível");
            runtime.workspaceRoot = "/other";
            root.check(runtime.pendingCommandFor("t5") === "", "Trocar de projeto descarta a espera");
            client.terminalOpened("t6", "bash", root.open("date"), "waiting");
            client.connected = false;
            client.statusChanged();
            runtime.handleCoreDisconnected();
            root.check(runtime.pendingCommandFor("t6") === "", "Queda do core descarta a espera");
            root.check(root.inputs.length === 0, "A UI nunca digita a linha");
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
