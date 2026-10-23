import QtQuick

// Espelho do DebugEventRouter: aquele traz os eventos do adaptador (parado,
// frames, variaveis), este leva ao core o que o DebugController pede.
Item {
    id: root

    property var coreClient: null
    property var debugController: null
    // A sonda escolhida em Gravar acompanha a depuracao probe-rs (52 §14.36).
    property var embeddedController: null

    visible: false

    Connections {
        target: root.debugController

        function onStartRequested(program, connect) {
            const flash = root.embeddedController ? root.embeddedController.flash : null;
            const attach = connect !== undefined && connect !== null && Object.keys(connect).length > 0;
            const probe = flash && flash.probeDestination && !attach ? flash.probe : "";
            // Sem programa explicito, o alvo escolhido em Gravar (o core recusa
            // "mais de um executavel" quando o build tem varios).
            const chosen = probe !== "" && program === "" ? flash.targetArtifact : "";
            root.coreClient.debugStart(chosen !== "" ? chosen : program, connect, probe);
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
