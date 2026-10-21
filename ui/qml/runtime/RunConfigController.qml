import QtQuick

// Configuracoes de execucao salvas: modelo, qual esta ativa, e o estado do
// dialogo e do menu.
//
// Saiu do RuntimeController porque sao responsabilidades diferentes que estavam
// no mesmo arquivo (ARCHITECTURE.md §6: nunca misturar). "Guardar como rodar um
// programa" nao e "manter uma sessao de terminal": este controller nao conhece
// PTY, id de sessao nem render — e o RuntimeController nao precisa saber o que e
// uma configuracao salva.
//
// O RuntimeController continua dono de: terminais e a execucao em si
// (run.start/stop/stdin), que compartilham a aba Terminal.
Item {
    id: root

    property alias runConfigsModel: runConfigsListModel
    property string activeConfigId: ""
    property string activeConfigName: ""
    property bool runConfigDialogVisible: false
    property string editingConfigId: ""
    property bool configMenuVisible: false
    property real configMenuX: 0
    property real configMenuY: 0
    property bool mutationPending: false
    property double mutationRequestId: -1
    property bool closeEditorAfterSave: false
    property string errorText: ""

    signal mutationFailed(string message)
    signal refreshRequested()
    signal saveRunConfigRequested(string id, string name, string command)
    signal deleteRunConfigRequested(string id)
    signal setActiveRunConfigRequested(string id)
    signal runConfigDialogOpenRequested(string name, string command)

    visible: false

    ListModel {
        id: runConfigsListModel
    }

    // Trocar de workspace fecha dialogo e menu: eles apontam para configuracoes
    // do workspace anterior.
    function clear() {
        runConfigsListModel.clear();
        activeConfigId = "";
        activeConfigName = "";
        editingConfigId = "";
        configMenuVisible = false;
        runConfigDialogVisible = false;
        mutationPending = false;
        mutationRequestId = -1;
        errorText = "";
    }

    function handleRunConfigs(configs, activeId) {
        errorText = "";
        runConfigsListModel.clear();
        activeConfigId = activeId !== undefined ? activeId : "";
        activeConfigName = "";
        for (let i = 0; i < configs.length; i++) {
            runConfigsListModel.append({
                id: configs[i].id,
                name: configs[i].name,
                command: configs[i].command
            });
            if (configs[i].id === activeConfigId) {
                activeConfigName = configs[i].name;
            }
        }
        if (activeConfigName === "") {
            activeConfigId = "";
        }
    }

    function activeConfigCommand() {
        for (let i = 0; i < runConfigsListModel.count; i++) {
            if (runConfigsListModel.get(i).id === activeConfigId) {
                return runConfigsListModel.get(i).command;
            }
        }
        return "";
    }

    function openConfigMenu(x, y) {
        configMenuX = x;
        configMenuY = y;
        configMenuVisible = true;
    }

    function closeConfigMenu() {
        configMenuVisible = false;
    }

    function chooseConfig(id) {
        if (!beginMutation(false)) return;
        configMenuVisible = false;
        setActiveRunConfigRequested(id);
    }

    function openNewConfigDialog() {
        if (mutationPending) return;
        errorText = "";
        configMenuVisible = false;
        editingConfigId = "";
        runConfigDialogVisible = true;
        runConfigDialogOpenRequested("", "");
    }

    function openEditConfigDialog() {
        if (activeConfigId === "" || mutationPending) {
            return;
        }
        errorText = "";
        configMenuVisible = false;
        editingConfigId = activeConfigId;
        runConfigDialogVisible = true;
        runConfigDialogOpenRequested(activeConfigName, activeConfigCommand());
    }

    function confirmConfigDialog(name, command) {
        if (name === "" || command === "" || !beginMutation(true)) return;
        saveRunConfigRequested(editingConfigId, name, command);
    }

    function cancelConfigDialog() {
        runConfigDialogVisible = false;
    }

    function deleteActiveConfig() {
        configMenuVisible = false;
        if (activeConfigId !== "" && beginMutation(false)) {
            deleteRunConfigRequested(activeConfigId);
        }
    }
    function saveConfiguration(name, command) {
        if (name === "" || command === "" || !beginMutation(false)) return;
        saveRunConfigRequested("", name, command);
    }

    function beginMutation(closeEditor) {
        if (mutationPending) return false;
        mutationPending = true;
        mutationRequestId = -1;
        closeEditorAfterSave = closeEditor;
        errorText = "";
        return true;
    }

    function bindMutation(requestId) {
        if (!mutationPending) return;
        mutationRequestId = requestId;
        if (requestId < 0) handleMutationFailed(requestId, qsTr("O serviço da IDE está indisponível. Tente novamente após reconectar."));
    }

    function handleMutationFinished(requestId) {
        if (!mutationPending || requestId !== mutationRequestId) return;
        mutationPending = false;
        mutationRequestId = -1;
        if (closeEditorAfterSave) runConfigDialogVisible = false;
    }

    function handleMutationFailed(requestId, message) {
        if (!mutationPending || requestId !== mutationRequestId) return;
        mutationPending = false;
        mutationRequestId = -1;
        errorText = message;
        mutationFailed(message);
    }

    function handleReadFailed(message) {
        errorText = message;
        mutationFailed(message);
    }

    function handleCoreDisconnected() {
        if (mutationPending) handleMutationFailed(mutationRequestId,
            qsTr("A conexão com o serviço da IDE foi interrompida. O salvamento não pôde ser confirmado; atualize a lista antes de tentar novamente."));
    }

}
