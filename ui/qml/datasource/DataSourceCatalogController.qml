pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Teste e catálogo têm pedidos públicos próprios, inclusive ao reabrir o projeto.
QtObject {
    id: root
    property var dataSourceController: null
    property string workspaceRoot: ""
    property int generation: 0
    property int serial: 0
    property int selectionGeneration: 0
    property var pending: ({})
    property var structureKeys: ({})
    property string lastTest: ""

    signal testRequested(string name, var context)
    signal introspectRequested(string name, var context)

    onWorkspaceRootChanged: reset()
    readonly property Connections profileChanges: Connections {
        target: root.dataSourceController
        function onProfilesChanged() { root.validate(); }
        function onDraftChanged() { root.validate(); }
        function onSelectedNameChanged() { root.selectionGeneration += 1; }
    }

    function key(profile) { return root.dataSourceController.queries.profileKey(profile); }

    function current(operation) {
        const controller = root.dataSourceController;
        return operation && operation.expectedContext.workspace === root.workspaceRoot
            && root.key(operation.expectedContext.profile) === root.key(controller.profileByName(operation.name))
            && (controller.selectedName !== operation.name && controller.draft.name !== operation.name
                || root.key(controller.draft) === root.key(operation.expectedContext.profile));
    }

    function reset() {
        root.generation += 1;
        root.pending = ({});
        root.structureKeys = ({});
        root.lastTest = "";
    }

    function validate() {
        const next = Object.assign({}, root.pending);
        for (const id of Object.keys(next)) {
            if (root.current(next[id])) continue;
            if (next[id].method === "test") root.dataSourceController.testing = false;
            else {
                if (root.dataSourceController.draft.name === next[id].name) root.dataSourceController.reading = false;
                root.dataSourceController.readingNames = Object.assign({}, root.dataSourceController.readingNames, { [next[id].name]: false });
            }
            delete next[id];
        }
        root.pending = next;
        const structures = Object.assign({}, root.dataSourceController.structures);
        for (const name of Object.keys(structures)) {
            if (root.structureKeys[name] !== root.key(root.dataSourceController.profileByName(name))) delete structures[name];
        }
        root.dataSourceController.structures = structures;
    }

    function begin(method, name) {
        const controller = root.dataSourceController;
        const profile = controller.profileByName(name);
        if (!profile || controller.draft.name === name && root.key(controller.draft) !== root.key(profile)) {
            controller.errorText = qsTr("Salve as alterações da conexão antes de conectar.");
            return;
        }
        root.serial += 1;
        const context = { clientContext: "catalog." + String(root.generation) + ":" + String(root.serial),
            expectedContext: { workspace: root.workspaceRoot, profile: Object.assign({}, profile) } };
        const operation = Object.assign({ method: method, name: name, selectionGeneration: root.selectionGeneration }, context);
        root.pending = Object.assign({}, root.pending, { [method + ":" + name]: operation });
        if (method === "test") {
            controller.clearVerdict();
            controller.testing = true;
            root.lastTest = context.clientContext;
            root.testRequested(name, context);
        } else {
            controller.readingNames = Object.assign({}, controller.readingNames, { [name]: true });
            if (controller.draft.name === name) controller.reading = true;
            root.introspectRequested(name, context);
        }
    }

    function matches(method, event) {
        if (!event) return false;
        const operation = root.pending[method + ":" + event.name];
        return root.current(operation) && operation.clientContext === event.clientContext;
    }

    function take(method, name, token) {
        const id = method + ":" + name;
        const operation = root.pending[id];
        if (!operation || token !== operation.clientContext || !root.current(operation)) return null;
        const next = Object.assign({}, root.pending);
        delete next[id];
        root.pending = next;
        return operation;
    }

    function tested(name, ok, version, message, needsSecret, token) {
        const operation = root.take("test", name, token);
        if (!operation || token !== root.lastTest || operation.selectionGeneration !== root.selectionGeneration) return;
        const controller = root.dataSourceController;
        controller.testing = false;
        controller.testedName = name;
        controller.testOk = ok;
        controller.serverVersion = version;
        controller.testMessage = message;
        controller.secretRequired = needsSecret;
        if (needsSecret) controller.secrets.request("test", operation);
    }

    function introspected(name, ok, schemas, collections, message, needsSecret, token) {
        const operation = root.take("introspect", name, token);
        if (!operation) return;
        const controller = root.dataSourceController;
        controller.readingNames = Object.assign({}, controller.readingNames, { [name]: false });
        root.structureKeys = Object.assign({}, root.structureKeys, { [name]: root.key(operation.expectedContext.profile) });
        controller.structures = Object.assign({}, controller.structures, { [name]: ok ? { schemas: schemas, collections: collections }
            : { schemas: [], collections: [], failed: message } });
        const sameSelection = operation.selectionGeneration === root.selectionGeneration;
        if (sameSelection && controller.draft.name === name) {
            controller.reading = false;
            controller.schemas = ok ? schemas : [];
            controller.collections = ok ? collections : [];
            controller.testMessage = message;
            controller.secretRequired = needsSecret;
        }
        if (needsSecret && sameSelection) controller.secrets.request("introspect", operation);
    }

    function fail(method, message, code, event) {
        if (!event) return;
        if (code === "DRIVER_APPROVAL_REQUIRED") return;
        const operation = root.take(method, event.name, event.clientContext);
        if (!operation) return;
        const controller = root.dataSourceController;
        const sameSelection = operation.selectionGeneration === root.selectionGeneration;
        if (method === "test" && !sameSelection) return;
        if (method === "test") controller.testing = false;
        else {
            if (controller.draft.name === event.name) controller.reading = false;
            controller.readingNames = Object.assign({}, controller.readingNames, { [event.name]: false });
        }
        if (sameSelection && controller.draft.name === event.name) {
            controller.testMessage = message;
            controller.secretRequired = code === "SECRET_REQUIRED";
        }
        if (code === "SECRET_REQUIRED" && sameSelection) controller.secrets.request(method, operation);
    }
}
