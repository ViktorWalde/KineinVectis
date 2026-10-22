import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    function check(condition, message) {
        if (!condition) { failures++; console.error("FALHAS: " + message); }
    }
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        signal statusChanged()
        signal workspaceChanged()
        signal terminalOpened(string id, string shell, double requestId)
        signal terminalOpenFailed(double requestId, string message)
        signal terminalRender(var render)
        signal terminalSelectionCopied(string id, string selectionId, string text, bool valid)
        signal terminalClosed(string id, int exitCode)
        signal runConfigsResolved(var configs, string activeId)
        signal runConfigMutationFinished(double requestId)
        signal runConfigMutationFailed(double requestId, string message)
        signal runStarted(string command, string id, string key, string workspace, double requestId)
        signal runRequestFailed(double requestId, string method, string message)
        signal runStopFinished(double requestId)
        signal runFinished(bool success, int exitCode)
        signal requestFailed(string method, string message, string code)
        function runStart(command, device, flash) { return ++sequence; }
        function runScript(path, device) { return ++sequence; }
        function runStop() { return ++sequence; }
    }
    RuntimeController { id: runtime; workspaceRoot: "/project" }
    RunConfigController { id: configs }
    RuntimeRequestRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    RuntimeEventRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    HeaderRunWidget {
        id: header
        coreConnected: client.connected
        runPending: runtime.runPending
        runStopping: runtime.runStopping
        onStopRunRequested: runtime.stopRun()
    }
    Component.onCompleted: {
        runtime.startRun(""); const first = client.sequence;
        check(runtime.runPending, "pedido fica pendente antes do terminal");
        const action = find(header, "headerRunAction");
        check(action.objectName === "headerRunAction" && action.iconName === "stop"
              && action.tooltip.indexOf("preparação") >= 0, "header atual permite cancelar preparação");
        runtime.stopRun(); const stop = client.sequence;
        runtime.stopRun(); runtime.startRun("");
        check(client.sequence === stop && runtime.runStopping && !action.enabled, "parada e clique duplo não duplicam pedidos");
        client.runStopFinished(stop + 100);
        client.runRequestFailed(first + 100, "run.start", "unrelated");
        client.requestFailed("run.start", "generic", "INVALID_REQUEST");
        check(runtime.runPending && runtime.runStopping && runtime.runErrorText === "", "respostas alheias/genéricas não concluem pedidos");
        client.runStopFinished(stop);
        check(runtime.runPending && runtime.runStopping && !action.enabled,
              "aceite de Parar mantém cancelamento visível até a recusa da preparação");
        runtime.stopRun();
        check(client.sequence === stop, "repetir Parar após o aceite não envia outro pedido");
        runtime.startScript("main.py"); check(client.sequence === stop, "sem nova execução até a preparação terminar");
        client.runRequestFailed(first, "run.start", "cancelled");
        check(!runtime.runPending && runtime.runErrorText === "cancelled", "recusa correspondente encerra preparação");
        runtime.startScript("main.py"); const second = client.sequence;
        client.runStarted("old", "old-terminal", "old-key", "/project", first);
        client.runRequestFailed(first, "run.start", "old failure");
        check(runtime.runPending && runtime.terminalsModel.count === 0 && runtime.runErrorText === "", "respostas antigas não criam aba nem apagam pedido novo");
        client.runStarted("python main.py", "current", "script-key", "/project", second);
        client.runStarted("duplicate", "current", "script-key", "/project", second);
        check(!runtime.runPending && runtime.terminalsModel.count === 1 && runtime.runTerminalId === "current", "aceite correspondente cria somente uma aba");
        runtime.stopRun(); const runningStop = client.sequence;
        client.runStopFinished(runningStop);
        runtime.stopRun();
        check(runtime.runStopping && client.sequence === runningStop, "execução aceita espera saída real sem repetir Parar");
        client.terminalClosed("current", 0);
        check(!runtime.runStopping, "saída observada encerra o estado de parada");
        runtime.startRun(""); const reopening = client.sequence;
        configs.handleRunConfigs([{id:"saved",name:"Keep editor",command:"echo keep"}],"saved");
        configs.openEditConfigDialog();
        client.workspaceChanged();
        check(configs.runConfigDialogVisible && configs.editingConfigId === "saved"
              && configs.activeConfigId === "saved", "mesma raiz preserva editor e configurações no dono existente");
        client.runStarted("stale", "stale", "stale", "/project", reopening);
        check(!runtime.runPending && runtime.indexOfTerminal("stale") < 0
              && runtime.isFinishedRun("current"), "reabrir mesma raiz invalida espera e preserva abas existentes");
        runtime.startRun(""); const disconnected = client.sequence;
        client.connected = false; client.statusChanged();
        check(!runtime.runPending && !runtime.runStopping && runtime.runErrorText !== "", "queda libera esperas sem afirmar execução");
        client.connected = true; client.statusChanged();
        check(client.sequence === disconnected, "reconexão não repete operação");
        runtime.startRun("");
        client.runRequestFailed(disconnected, "run.start", "old connection");
        check(runtime.runPending, "resposta de conexão antiga não conclui tentativa nova");
        client.runRequestFailed(client.sequence, "run.start", "current refusal");
        check(!runtime.runPending && runtime.runErrorText === "current refusal", "recusa nova preserva causa");
        runtime.startRun(""); const delayed = client.sequence;
        runtime.stopRun(); const delayedStop = client.sequence;
        client.runRequestFailed(delayed, "run.start", "cancelled before stop reply");
        runtime.startRun("");
        check(!runtime.runPending && runtime.runStopping && client.sequence === delayedStop,
              "recusa antecipada ainda aguarda resposta da parada antes de outra execução");
        client.runStopFinished(delayedStop);
        check(!runtime.runStopping, "resposta atrasada de Parar libera a espera restante");
        Qt.exit(failures === 0 ? 0 : 1);
    }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
}
