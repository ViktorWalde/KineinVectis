import QtQuick
import KineinVectis

Item {
    id: root
    width: 360
    height: 240
    property int failures: 0
    property int calls: 0
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
        function runStart(command, device) { return ++root.calls; }
        function runScript(path, device) { return ++root.calls; }
        function runStop() { return ++root.calls; }
        function terminalClose(id) { return ++root.calls; }
    }
    RuntimeController { id: runtime; workspaceRoot: "/fixture" }
    RunConfigController { id: configs }
    RuntimeRequestRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    TerminalPanel { id: panel; anchors.fill: parent; runtimeController: runtime }
    Component.onCompleted: {
        runtime.startRun("fake-command");
        check(calls === 0 && !runtime.runPending && runtime.runErrorText !== "",
              "Recusa sem conexão libera espera e mostra a causa");
        runtime.startScript("main.py");
        check(calls === 0 && !runtime.runPending, "Script desconectado também termina a espera");
        client.connected = true;
        runtime.startRun("fake-command");
        check(calls === 1 && runtime.runPending && runtime.runErrorText === "",
              "Nova tentativa conectada limpa erro e faz um pedido");
        runtime.handleRunRequestFailed(root.calls, "run.start", "A porta está em uso por outro monitor.");
        check(!runtime.runPending, "Recusa real do core termina a espera");
        Qt.callLater(checkSurface);
    }
    function checkSurface() {
        const error = find(panel, "terminalCommandError");
        const viewport = find(panel, "terminalViewport");
        check(error.visible && error.message === "A porta está em uso por outro monitor.",
              "A causa do core aparece na faixa do Terminal");
        check(viewport.y >= error.height && viewport.height > 0,
              "Faixa não sobrepõe o grid");
        runtime.handleCoreDisconnected();
        check(runtime.runErrorText === "A porta está em uso por outro monitor.",
              "Desconexão após recusa preserva a causa específica");
        runtime.startRun("fake-command");
        runtime.handleRunStarted("fake-command", "run-1", "key-1", "/fixture", root.calls);
        runtime.terminalActive = true;
        runtime.terminalActive = false;
        runtime.handleCoreDisconnected();
        check(runtime.runErrorText.indexOf("não pôde ser confirmado") >= 0,
              "Queda após aceite informa estado desconhecido mesmo após zerar terminalActive");
        runtime.startRun("fake-command");
        runtime.handleRunStarted("fake-command", "run-2", "key-2", "/fixture", root.calls);
        runtime.handleTerminalClosed("run-2", 0);
        runtime.handleRunFinished(true, 0);
        check(runtime.runErrorText === "", "Conclusão positiva não vira erro vermelho");
        runtime.handleCoreDisconnected();
        check(runtime.runErrorText === "", "Queda após conclusão não inventa execução interrompida");
        runtime.startRun("fake-command");
        runtime.handleRunStarted("fake-command", "run-3", "key-3", "/fixture", root.calls);
        runtime.handleTerminalOpened("shell-1", "sh", "Shell");
        runtime.closeTerminal("run-3");
        runtime.closeTerminal("shell-1");
        runtime.handleRequestFailed("terminal.close", "falha ao fechar shell-1");
        runtime.handleTerminalClosed("run-3", 0);
        check(runtime.indexOfTerminal("run-3") < 0 && runtime.runErrorText === "falha ao fechar shell-1",
              "Recusa de uma aba preserva o fechamento independente da outra");
        runtime.workspaceRoot = "/other";
        check(runtime.runErrorText === "", "Novo contexto limpa erro anterior");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
