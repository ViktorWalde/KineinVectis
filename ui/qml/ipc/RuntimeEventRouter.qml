import QtQuick

Item {
    id: root

    property var coreClient: null
    property var runtimeController: null
    property var runConfigController: null

    visible: false

    Connections {
        target: root.coreClient

        function onTerminalOpened(id, shell) {
            root.runtimeController.handleTerminalOpened(id, shell);
        }

        function onTerminalRender(render) {
            root.runtimeController.handleTerminalRender(render);
        }

        function onTerminalSelectionCopied(id, selectionId, text, valid) {
            root.runtimeController.terminalSelectionCopied(id, selectionId, text, valid);
        }

        function onTerminalClosed(id, exitCode) {
            root.runtimeController.handleTerminalClosed(id, exitCode);
        }

        function onRunConfigsResolved(configs, activeId) {
            root.runConfigController.handleRunConfigs(configs, activeId);
        }

        function onRunStarted(command, terminalId) {
            root.runtimeController.handleRunStarted(command, terminalId);
        }

        function onRunFinished(success, exitCode) {
            root.runtimeController.handleRunFinished(success, exitCode);
        }

        // `run.script` entrou em 2026-09-17: a recusa do core ("mpremote nao
        // esta' nesta maquina", porta invalida) ia so' para o log da IDE, e a
        // aba que o Executar acabou de abrir ficava muda.
        function onRequestFailed(method, message) {
            if (method === "terminal.selectAll" || method === "terminal.copySelection") {
                root.runtimeController.terminalSelectionFailed();
            }
            if (method === "run.start" || method === "run.script" || method === "run.stop") {
                root.runtimeController.handleRequestFailed(method, message);
            }
        }
    }
}
