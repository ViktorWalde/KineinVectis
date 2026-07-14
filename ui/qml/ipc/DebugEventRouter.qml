import QtQuick

// Roteia eventos de debug do CoreClient para o DebugController.
Item {
    id: root

    property var coreClient: null
    property var debugController: null

    visible: false

    Connections {
        target: root.coreClient

        function onDebugStarted(program) {
            root.debugController.handleStarted(program);
        }

        function onDebugOutput(category, line) {
            root.debugController.handleOutput(category, line);
        }

        function onDebugStopped(reason, file, line, threadId) {
            root.debugController.handleStopped(reason, file, line);
        }

        function onDebugContinued() {
            root.debugController.handleContinued();
        }

        function onDebugFinished(exitCode) {
            root.debugController.handleFinished(exitCode);
        }

        function onDebugStackTraceResolved(frames) {
            root.debugController.handleStackTrace(frames);
        }

        function onDebugVariablesResolved(frameId, ref, variables) {
            root.debugController.handleVariables(frameId, ref, variables);
        }

        function onRequestFailed(method, message) {
            root.debugController.handleRequestFailed(method, message);
        }
    }
}
