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

        function onStartRequested(program) {
            root.coreClient.debugStart(program);
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

        function onSetBreakpointsRequested(file, lines) {
            root.coreClient.debugSetBreakpoints(file, lines);
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
