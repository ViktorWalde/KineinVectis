import QtQuick

// Roteia eventos de debug do CoreClient para o DebugController.
Item {
    id: root

    property var coreClient: null
    property var debugController: null

    visible: false

    Connections {
        target: root.coreClient

        function onDebugStarted(program, attached) {
            root.debugController.handleStarted(program, attached);
        }

        function onDebugEvaluateResolved(expression, value, typeName, ref) {
            root.debugController.handleEvaluated(expression, value, typeName, ref);
        }

        function onDebugEvaluateFailed(expression, message) {
            root.debugController.handleEvaluateFailed(expression, message);
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
            root.debugController.inspect.handleVariables(ref, variables);
        }

        function onDebugScopesResolved(frameId, scopes) {
            root.debugController.inspect.handleScopes(frameId, scopes);
        }

        function onDebugMemoryResolved(memory) {
            root.debugController.inspect.handleMemory(memory);
        }

        function onDebugDisassemblyResolved(instructions) {
            root.debugController.inspect.handleDisassembly(instructions);
        }

        function onRequestFailed(method, message) {
            root.debugController.handleRequestFailed(method, message);
            root.debugController.inspect.handleFailed(method, message);
        }
    }
}
