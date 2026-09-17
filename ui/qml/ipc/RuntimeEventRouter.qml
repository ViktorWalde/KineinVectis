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

        function onTerminalClosed(id, exitCode) {
            root.runtimeController.handleTerminalClosed(id);
        }

        function onRunConfigsResolved(configs, activeId) {
            root.runConfigController.handleRunConfigs(configs, activeId);
        }

        function onRunStarted(command) {
            root.runtimeController.handleRunStarted(command);
        }

        function onRunOutput(line, stream) {
            root.runtimeController.handleRunOutput(line, stream);
        }

        function onRunFinished(success, exitCode) {
            root.runtimeController.handleRunFinished(success, exitCode);
        }

        // `run.script` entrou em 2026-09-17: a recusa do core ("mpremote nao
        // esta' nesta maquina", porta invalida) ia so' para o log da IDE, e a
        // aba que o Executar acabou de abrir ficava muda.
        function onRequestFailed(method, message) {
            if (method === "run.start" || method === "run.script"
                    || method === "run.stdin" || method === "run.stop") {
                root.runtimeController.handleRequestFailed(method, message);
            }
        }
    }
}
