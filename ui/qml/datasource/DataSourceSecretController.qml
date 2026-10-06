import QtQuick
import KineinVectis

// Um segredo pertence ao projeto e ao perfil completo que o recebeu.
// A chave guarda somente campos publicos, nunca a senha.
QtObject {
    id: root

    property var dataSourceController: null
    property string workspaceRoot: ""
    property string value: ""
    property string ownerName: ""
    property string ownerKey: ""
    property var pending: null

    onWorkspaceRootChanged: clear()
    readonly property Connections profileChanges: Connections {
        target: root.dataSourceController
        function onDraftChanged() { root.validateContext(); }
        function onProfilesChanged() { root.validateContext(); }
    }
    onValueChanged: {
        if (root.value === "") {
            root.ownerName = "";
            root.ownerKey = "";
        } else {
            root.ownerName = root.dataSourceController && root.dataSourceController.draft
                ? root.dataSourceController.draft.name : "";
            root.ownerKey = root.key(root.dataSourceController ? root.dataSourceController.draft : null);
        }
    }

    function key(profile) {
        return profile === null || profile === undefined ? "" : JSON.stringify(DataSourceKinds.cloneProfile(profile));
    }

    function forName(name) {
        if (root.dataSourceController === null || root.value === "" || name !== root.ownerName
                || root.ownerKey !== root.key(root.dataSourceController.draft)) return "";
        const profile = root.dataSourceController.profileByName(name);
        return root.key(profile) === root.ownerKey ? root.value : "";
    }

    function validateContext() {
        if (root.pending !== null && !root.pendingCurrent()) root.pending = null;
        if (root.value !== "" && (root.dataSourceController === null || root.ownerKey !== root.key(root.dataSourceController.draft)
                || root.ownerKey !== root.key(root.dataSourceController.profileByName(root.ownerName)))) clear();
    }

    function pendingCurrent() {
        return root.pending !== null && root.dataSourceController !== null
            && root.pending.expectedContext.workspace === root.workspaceRoot
            && root.key(root.pending.expectedContext.profile) === root.key(root.dataSourceController.profileByName(root.pending.name))
            && root.key(root.pending.expectedContext.profile) === root.key(root.dataSourceController.draft);
    }

    // Copia apenas a intenção pública. Selecionar limpa o segredo e os pedidos anteriores.
    function request(method, operation) {
        if (!operation || !operation.expectedContext || root.dataSourceController === null
                || operation.expectedContext.workspace !== root.workspaceRoot
                || root.key(operation.expectedContext.profile) !== root.key(root.dataSourceController.profileByName(operation.name))) return;
        const saved = Object.assign({}, operation, { method: method });
        if (root.dataSourceController.selectedName !== operation.name) root.dataSourceController.select(operation.name);
        root.pending = saved;
        root.dataSourceController.panelVisible = true;
        root.dataSourceController.secretRequired = true;
        root.dataSourceController.testMessage = qsTr("O servidor pediu a senha para continuar esta operação.");
    }

    function takePending() {
        const operation = root.pendingCurrent() ? root.pending : null;
        root.pending = null;
        return operation;
    }

    function clear() {
        root.pending = null;
        root.ownerKey = "";
        root.ownerName = "";
        root.value = "";
    }
}
