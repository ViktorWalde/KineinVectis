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

    // O que o depurador mostra (P3): escopos, memoria e disassembly — filho
    // do DebugController; as variaveis de um escopo vao pelo mesmo
    // debug.variables { ref } do pai.
    Connections {
        target: root.debugController ? root.debugController.inspect : null

        function onScopesRequested(frameId) {
            root.coreClient.debugScopes(frameId);
        }

        function onScopeVariablesRequested(ref) {
            root.coreClient.debugVariablesForRef(ref);
        }

        function onReadMemoryRequested(memoryReference, count, offset) {
            root.coreClient.debugReadMemory(memoryReference, count, offset);
        }

        function onDisassembleRequested(memoryReference, instructionCount, instructionOffset) {
            root.coreClient.debugDisassemble(memoryReference, instructionCount, instructionOffset);
        }
    }
}
