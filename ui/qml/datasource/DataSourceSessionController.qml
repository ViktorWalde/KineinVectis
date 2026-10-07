pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Intencao de desconectar correlacionada; somente o core confirma o encerramento.
// Estado publico por destino, sem possuir perfil, buffer, driver ou senha.
QtObject {
    id: root
    property var dataSourceController: null
    property string workspaceRoot: ""
    property int generation: 0
    property int serial: 0
    property var states: DataSourceMap.copy()

    signal requested(string name, var context)

    onWorkspaceRootChanged: {
        root.generation += 1;
        root.states = DataSourceMap.copy();
    }
    readonly property Connections profileChanges: Connections {
        target: root.dataSourceController
        function onProfilesChanged() { root.validate(); }
    }

    function current(operation) {
        return operation && operation.expectedContext.workspace === root.workspaceRoot
            && root.dataSourceController.queries.profileKey(operation.expectedContext.profile)
                === root.dataSourceController.queries.profileKey(root.dataSourceController.profileByName(operation.name));
    }

    function busy(name) {
        const state = DataSourceMap.get(root.states, name);
        return !!state && state.pending === true;
    }

    function forget(name) {
        const states = DataSourceMap.copy(root.states);
        delete states[name];
        root.states = states;
    }

    function validate() {
        for (const name of Object.keys(root.states)) {
            if (!root.current(root.states[name])) root.forget(name);
        }
    }

    function begin(name) {
        const controller = root.dataSourceController;
        const profile = controller.profileByName(name);
        if (!profile || root.workspaceRoot === "" || root.busy(name)) return;
        root.serial += 1;
        const context = { clientContext: "disconnect." + String(root.generation) + ":" + String(root.serial),
            expectedContext: { workspace: root.workspaceRoot, profile: Object.assign({}, profile) } };
        root.states = DataSourceMap.copy(root.states, { [name]: Object.assign({ name: name,
            pending: true, text: qsTr("desconectando…") }, context) });
        controller.catalog.detach(name, false);
        controller.consoles.discard(name);
        if (controller.lastQuery && controller.lastQuery.name === name) controller.queries.invalidate(name);
        if (controller.secrets.ownerName === name || controller.secrets.pending && controller.secrets.pending.name === name) {
            controller.clearSecret();
            controller.secretRequired = false;
        }
        if (controller.odbc.pending && controller.odbc.pending.name === name) controller.odbc.cancel();
        root.requested(name, context);
    }

    function matches(event) {
        if (!event) return false;
        const state = DataSourceMap.get(root.states, event.name);
        return root.current(state) && state.pending && state.clientContext === event.clientContext
            && (!state.jobId || !event.jobId || state.jobId === event.jobId);
    }

    function accepted(event) {
        if (!root.matches(event)) return;
        root.states = DataSourceMap.copy(root.states, { [event.name]: Object.assign({}, root.states[event.name], { jobId: event.jobId }) });
    }

    function finished(event) {
        if (!root.matches(event)) return;
        if (event.success === true) root.dataSourceController.catalog.detach(event.name, true);
        root.states = DataSourceMap.copy(root.states, { [event.name]: Object.assign({}, root.states[event.name], {
            pending: false, text: event.success === true ? qsTr("desconectado") : event.message }) });
    }

    function failed(message, operation) {
        if (!root.matches(operation)) return;
        root.finished(Object.assign({}, operation, { success: false, message: message }));
    }
}
