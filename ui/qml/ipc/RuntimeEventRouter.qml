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

        function onRequestFailed(method, message) {
            if (method === "run.start" || method === "run.stdin"
                    || method === "run.stop") {
                root.runtimeController.handleRequestFailed(method, message);
            }
        }
    }
}
