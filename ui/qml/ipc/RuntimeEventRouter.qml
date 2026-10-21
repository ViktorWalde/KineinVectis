import QtQuick

Item {
    id: root

    property var coreClient: null
    property var runtimeController: null
    property var runConfigController: null

    visible: false

    Connections {
        target: root.coreClient

        function onTerminalOpened(id, shell, requestId) {
            root.runtimeController.handleTerminalOpened(id, shell, "", requestId);
        }

        function onTerminalOpenFailed(requestId, message) {
            root.runtimeController.handleTerminalOpenFailed(requestId, message);
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

        function onRunConfigMutationFinished(requestId) {
            root.runConfigController.handleMutationFinished(requestId);
        }

        function onRunConfigMutationFailed(requestId, message) {
            root.runConfigController.handleMutationFailed(requestId, message);
        }

        function onRunStarted(command, terminalId, executionKey, workspace) {
            root.runtimeController.handleRunStarted(command, terminalId, executionKey, workspace);
        }

        function onStatusChanged() {
            if (!root.coreClient.connected) {
                root.runtimeController.handleCoreDisconnected();
                root.runConfigController.handleCoreDisconnected();
            }
        }

        function onRunFinished(success, exitCode) {
            root.runtimeController.handleRunFinished(success, exitCode);
        }

        // `run.script` entrou em 2026-09-17: a recusa do core ("mpremote nao
        // esta' nesta maquina", porta invalida) ia so' para o log da IDE, e a
        // aba que o Executar acabou de abrir ficava muda.
        function onRequestFailed(method, message) {
            if (method === "runConfig.list") root.runConfigController.handleReadFailed(message);
            if (method === "terminal.selectAll" || method === "terminal.copySelection") {
                root.runtimeController.terminalSelectionFailed();
            }
            if (method === "run.start" || method === "run.script" || method === "run.stop"
                    || method === "terminal.close") {
                root.runtimeController.handleRequestFailed(method, message);
            }
        }
    }
}
