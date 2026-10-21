import QtQuick
import KineinVectis

Item {
    id: root
    width: 480
    height: 600
    property int failures: 0
    property double firstRequest: -1
    function check(condition, message) {
        if (!condition) { failures++; console.error("FALHAS: " + message); }
    }
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        property int writes: 0
        property int refreshes: 0
        signal statusChanged()
        signal terminalOpened(string id, string shell, double requestId)
        signal terminalOpenFailed(double requestId, string message)
        signal terminalRender(var render)
        signal terminalSelectionCopied(string id, string selectionId, string text, bool valid)
        signal terminalClosed(string id, int exitCode)
        signal runConfigsResolved(var configs, string activeId)
        signal runConfigMutationFinished(double requestId)
        signal runConfigMutationFailed(double requestId, string message)
        signal runStarted(string command, string terminalId, string executionKey, string workspace)
        signal runFinished(bool success, int exitCode)
        signal requestFailed(string method, string message, string code)
        function runConfigSave(id, name, command) { writes++; return ++sequence; }
        function runConfigDelete(id) { writes++; return ++sequence; }
        function runConfigSetActive(id) { writes++; return ++sequence; }
        function runConfigList() { refreshes++; }
    }
    RuntimeController { id: runtime; runConfigErrorText: configs.errorText }
    RunConfigController {
        id: configs
        onRunConfigDialogOpenRequested: (name, command) => editor.openWith(name, command)
        onMutationFailed: runtime.showTabRequested("terminal")
    }
    RuntimeRequestRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    RuntimeEventRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    RunConfigDialog {
        id: editor
        visible: configs.runConfigDialogVisible
        busy: configs.mutationPending
        errorText: configs.errorText
    }
    Component.onCompleted: {
        configs.handleRunConfigs([{id:"cfg-1",name:"Old",command:"echo old"}],"cfg-1");
        configs.openEditConfigDialog();
        editor.openWith("New", "echo new");
        const nameField = find(editor, "runConfigName");
        const commandField = find(editor, "runConfigCommand");
        const cancel = find(editor, "runConfigCancel");
        const save = find(editor, "runConfigSave");
        // Offscreen checks focus intent; Xvfb covers the actual key events/window.
        nameField.forceActiveFocus();
        let tab = {key: Qt.Key_Tab, modifiers: Qt.NoModifier, accepted: false};
        nameField.keyPressed(tab);
        check(tab.accepted && commandField.focus, "Tab segue do nome ao comando");
        let back = {key: Qt.Key_Backtab, modifiers: Qt.ShiftModifier, accepted: false};
        commandField.keyPressed(back);
        check(back.accepted && nameField.focus, "Shift+Tab volta ao nome");
        nameField.keyPressed({key: Qt.Key_Backtab, modifiers: Qt.ShiftModifier, accepted: false});
        check(save.focus, "voltar do nome chega ao salvar habilitado");
        commandField.keyPressed({key: Qt.Key_Tab, modifiers: Qt.NoModifier, accepted: false});
        check(cancel.focus && cancel.KeyNavigation.tab === save && save.KeyNavigation.tab === nameField,
              "botoes completam o ciclo de foco dentro do editor");
        nameField.keyPressed({key: Qt.Key_Left, modifiers: Qt.NoModifier, accepted: false});
        check(cancel.focus, "teclas de edicao nao desviam o foco");
        configs.confirmConfigDialog(editor.currentName(), editor.currentCommand());
        const first = client.sequence;
        firstRequest = first;
        check(configs.mutationPending && configs.runConfigDialogVisible, "clique nao confirma salvamento");
        configs.confirmConfigDialog("duplicate", "echo duplicate");
        check(client.writes === 1, "clique repetido nao envia outra mutacao");
        client.runConfigsResolved([{id:"cfg-1",name:"Old",command:"echo old"}],"cfg-1");
        client.runConfigMutationFinished(first + 30);
        client.runConfigMutationFailed(first + 30,"falha alheia");
        check(configs.mutationPending && configs.runConfigDialogVisible && configs.errorText === "",
              "snapshot ou resposta alheia nao confirmam pedido");
        client.runConfigMutationFailed(first,"arquivo invalido: /tmp/runconfigs.json <literal>");
        check(!configs.mutationPending && configs.runConfigDialogVisible && configs.errorText.indexOf("<literal>") >= 0,
              "falha correspondente preserva editor e causa");
        check(editor.currentName() === "New" && editor.currentCommand() === "echo new", "resposta nao substitui campos");
        afterError.start();
    }
    Timer { id: afterError; interval: 60; onTriggered: root.finish() }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (let index = 0; index < item.children.length; index++) {
            const found = find(item.children[index], name);
            if (found) return found;
        }
        return null;
    }
    function finish() {
        const first = firstRequest;
        const band = find(editor, "runConfigError");
        check(band && band.visible && band.implicitHeight > 0 && band.message === configs.errorText,
              "erro e renderizado no editor real");
        configs.confirmConfigDialog("New","echo new");
        client.runConfigMutationFinished(first);
        check(configs.mutationPending, "confirmacao antiga nao conclui tentativa nova");
        client.runConfigsResolved([{id:"cfg-1",name:"New",command:"echo new"}],"cfg-1");
        client.runConfigMutationFinished(client.sequence);
        check(!configs.mutationPending && !configs.runConfigDialogVisible && configs.activeConfigName === "New",
              "somente confirmacao correspondente fecha editor");
        configs.openEditConfigDialog();
        configs.confirmConfigDialog("Keep","echo keep");
        const disconnected = client.sequence;
        client.connected = false;
        client.statusChanged();
        check(!configs.mutationPending && configs.runConfigDialogVisible && configs.errorText.indexOf("confirmado") >= 0,
              "desconexao libera espera sem afirmar se salvou");
        configs.confirmConfigDialog("Keep","echo keep");
        check(client.sequence === disconnected && !configs.mutationPending, "desconectado nao envia nem fica aguardando");
        client.connected = true;
        client.statusChanged();
        check(client.sequence === disconnected, "reconectar nao repete salvamento");
        configs.refreshRequested();
        check(client.refreshes === 1, "verificar lista usa caminho existente");
        client.requestFailed("runConfig.list","nao foi possivel ler","INVALID_REQUEST");
        check(configs.errorText === "nao foi possivel ler" && runtime.runConfigErrorText === configs.errorText,
              "erro de leitura chega ao veredito existente");
        runtime.runErrorText = "falha independente da execucao";
        configs.handleRunConfigs([], "");
        check(configs.errorText === "" && runtime.runConfigErrorText === "" && runtime.runErrorText !== "",
              "lista valida limpa somente a falha de configuracao");
        configs.saveConfiguration("Flash","echo flash");
        check(configs.mutationPending, "salvamento vindo de embarcados usa a mesma confirmacao");
        configs.handleRunConfigs([{id:"cfg-old",name:"Other project",command:"echo other"}], "cfg-old");
        configs.clear();
        check(configs.runConfigsModel.count === 0 && configs.activeConfigId === "" && configs.activeConfigName === ""
              && configs.editingConfigId === "", "trocar workspace remove o perfil e a lista anteriores mesmo se a proxima leitura falhar");
        client.runConfigMutationFinished(client.sequence);
        check(!configs.runConfigDialogVisible && configs.errorText === "", "resposta do workspace anterior nao restaura editor");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
