import QtQuick

Item {
    id: root

    property var coreClient: null
    property var runtimeController: null

    visible: false

    Connections {
        target: root.coreClient

        function onTerminalData(data) {
            root.runtimeController.handleTerminalData(data);
        }

        function onTerminalClosed(exitCode) {
            root.runtimeController.handleTerminalClosed();
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
