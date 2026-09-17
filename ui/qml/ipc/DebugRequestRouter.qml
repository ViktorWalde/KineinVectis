import QtQuick

// Espelho do DebugEventRouter: aquele traz os eventos do adaptador (parado,
// frames, variaveis), este leva ao core o que o DebugController pede.
Item {
    id: root

    property var coreClient: null
    property var debugController: null

    visible: false

    Connections {
        target: root.debugController

        function onStartRequested(program, connect) {
            root.coreClient.debugStart(program, connect);
        }

        function onStopRequested() {
            root.coreClient.debugStop();
        }

        function onContinueRequested() {
            root.coreClient.debugContinue();
        }

        function onNextRequested() {
            root.coreClient.debugNext();
        }

        function onStepInRequested() {
            root.coreClient.debugStepIn();
        }

        function onStepOutRequested() {
            root.coreClient.debugStepOut();
        }

        function onPauseRequested() {
            root.coreClient.debugPause();
        }

        function onSetBreakpointsRequested(file, breakpoints) {
            root.coreClient.debugSetBreakpoints(file, breakpoints);
        }

        function onEvaluateRequested(expression, frameId) {
            root.coreClient.debugEvaluate(expression, frameId);
        }

        function onStackTraceRequested() {
            root.coreClient.debugStackTrace();
        }

        function onFrameVariablesRequested(frameId) {
            root.coreClient.debugVariablesForFrame(frameId);
        }

        function onVariablesByRefRequested(ref) {
            root.coreClient.debugVariablesForRef(ref);
        }
    }
}
