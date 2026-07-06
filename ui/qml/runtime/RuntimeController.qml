import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property bool running: false
    property bool terminalActive: false
    property bool terminalPanelVisible: false
    property string terminalText: ""
    property alias runModel: runItemsModel

    signal showTabRequested(string tab)
    signal terminalOpenRequested()
    signal terminalInputRequested(string data)
    signal runStartRequested(string command)
    signal runStopRequested()
    signal runStdinRequested(string data)
    signal focusTerminalInputRequested()
    signal clearTerminalInputRequested()
    signal clearRunInputRequested()

    visible: false

    ListModel {
        id: runItemsModel
    }

    function clear() {
        runItemsModel.clear();
        terminalText = "";
    }

    function appendRunLine(line, kind) {
        runItemsModel.append({ line: line, kind: kind });
        if (runItemsModel.count > 2000) {
            runItemsModel.remove(0);
        }
    }

    function appendTerminalData(data) {
        terminalText += data;
        if (terminalText.length > 100000) {
            terminalText = terminalText.substring(terminalText.length - 80000);
        }
    }

    function openTerminalPanel() {
        if (workspaceRoot === "") {
            return;
        }
        const alreadyVisible = terminalPanelVisible;
        showTabRequested("terminal");
        if (alreadyVisible && !terminalActive) {
            terminalOpenRequested();
        }
        focusTerminalInputRequested();
    }

    function submitShellInput(text) {
        if (!terminalActive) {
            terminalOpenRequested();
            return;
        }
        terminalInputRequested(text + "\n");
        clearTerminalInputRequested();
    }

    function startRun(command) {
        if (workspaceRoot === "" || running) {
            return;
        }
        showTabRequested("run");
        runStartRequested(command);
    }

    function stopRun() {
        if (running) {
            runStopRequested();
        }
    }

    function submitRunInput(text) {
        if (text === "") {
            return;
        }
        clearRunInputRequested();
        if (running) {
            appendRunLine("> " + text, "stdin");
            runStdinRequested(text + "\n");
            return;
        }
        startRun(text);
    }

    function handleTerminalData(data) {
        appendTerminalData(data);
    }

    function handleTerminalClosed() {
        appendTerminalData(qsTr("\n== sessao encerrada — Enter para reabrir ==\n"));
    }

    function handleRunStarted(command) {
        appendRunLine("$ " + command, "command");
    }

    function handleRunOutput(line, stream) {
        appendRunLine(line, stream);
    }

    function handleRunFinished(success, exitCode) {
        appendRunLine(success
                      ? qsTr("== processo finalizado (codigo %1) ==").arg(exitCode)
                      : qsTr("== processo encerrou com falha (codigo %1) ==")
                            .arg(exitCode),
                      success ? "info" : "stderr");
    }

    function handleRequestFailed(method, message) {
        if (method === "run.start" || method === "run.stdin"
                || method === "run.stop") {
            appendRunLine(message, "stderr");
        }
    }
}
