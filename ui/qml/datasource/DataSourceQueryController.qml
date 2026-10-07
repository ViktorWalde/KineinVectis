pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Uma resposta vale somente para o pedido e o destino que continuam ativos.
QtObject {
    id: root
    property var dataSourceController: null
    property string workspaceRoot: ""
    property int generation: 0
    property int serial: 0
    property var lastQuery: null
    property bool querying: false
    property bool writeConfirmationRequired: false
    property var columns: []
    property var rows: []
    property string status: ""
    property string pendingDatabase: ""

    signal requested(string name, string sql, bool confirmed, int maxRows, var context, var confirmation)
    signal databaseCreated(var profile)
    signal secretNeeded(var operation)

    onWorkspaceRootChanged: invalidate()
    readonly property Connections profileChanges: Connections {
        target: root.dataSourceController
        function onProfilesChanged() { root.validate(); }
        function onDraftChanged() { root.validate(); }
    }

    function profileKey(profile) {
        return profile ? JSON.stringify(DataSourceKinds.cloneProfile(profile)) : "";
    }

    function current() {
        if (root.lastQuery === null || root.dataSourceController === null) return false;
        return root.lastQuery.expectedContext.workspace === root.workspaceRoot
            && root.lastQuery.profileKey === root.profileKey(root.dataSourceController.profileByName(root.lastQuery.name));
    }

    function validate() {
        if (root.lastQuery === null) return;
        const controller = root.dataSourceController;
        const editing = controller.selectedName === root.lastQuery.name || controller.draft.name === root.lastQuery.name;
        if (!root.current() || editing && root.profileKey(controller.draft) !== root.lastQuery.profileKey) root.invalidate();
    }

    function invalidate() {
        if (root.dataSourceController !== null && root.dataSourceController.previews) root.dataSourceController.previews.discard();
        root.generation += 1;
        root.lastQuery = null;
        root.querying = false;
        root.writeConfirmationRequired = false;
        root.columns = [];
        root.rows = [];
        root.status = "";
        root.pendingDatabase = "";
        if (root.dataSourceController !== null) {
            root.dataSourceController.impact.cancel();
            root.dataSourceController.odbc.cancel();
        }
    }

    function begin(name, text, confirmed, maxRows, confirmation, database, preview) {
        if (name === "" || text.trim() === "") return;
        const profile = root.dataSourceController.profileByName(name);
        if (profile === null) {
            root.status = qsTr("Salve a conexão antes de executar.");
            return;
        }
        if (root.dataSourceController.draft.name === name && root.profileKey(root.dataSourceController.draft) !== root.profileKey(profile)) {
            root.status = qsTr("Salve as alterações da conexão antes de executar.");
            return;
        }
        root.dataSourceController.impact.cancel();
        root.dataSourceController.odbc.cancel();
        root.dataSourceController.previews.discard();
        root.serial += 1;
        const context = { clientContext: String(root.generation) + ":" + String(root.serial),
                          expectedContext: { workspace: root.workspaceRoot, profile: Object.assign({}, profile) }, preview: preview === true };
        root.lastQuery = Object.assign({}, context, { name: name, sql: text, confirmWrite: confirmed === true,
            maxRows: maxRows || 0, profileKey: root.profileKey(profile), confirmation: confirmation || null, database: database || "" });
        root.pendingDatabase = database || "";
        root.querying = true;
        root.writeConfirmationRequired = false;
        root.status = "";
        root.requested(name, text, confirmed === true, maxRows || 0, context, confirmation || ({}));
    }

    function matches(event) {
        return root.current() && event.name === root.lastQuery.name && event.clientContext === root.lastQuery.clientContext;
    }

    function accepted(operation) {
        if (!root.matches(operation)) {
            if (operation.preview === true) root.dataSourceController.previews.cancelExecutionRequested(operation.jobId);
            return;
        }
        root.lastQuery = Object.assign({}, root.lastQuery, { jobId: operation.jobId });
    }

    function handleOutcome(outcome) {
        if (!root.matches(outcome) || root.lastQuery.finished === true) return;
        root.dataSourceController.previews.finished(outcome);
        if (outcome.confirmationSql !== undefined) {
            if (outcome.confirmationSql !== root.lastQuery.sql) return;
            root.fail(outcome.message || "", "WRITE_CONFIRMATION_REQUIRED", outcome);
            return;
        }
        root.lastQuery = Object.assign({}, root.lastQuery, { finished: true });
        root.querying = false;
        root.columns = outcome.success === true ? (outcome.columns || []) : [];
        root.rows = outcome.success === true ? (outcome.rows || []) : [];
        root.dataSourceController.secretRequired = outcome.secretRequired === true;
        if (outcome.success === true) {
            const profile = root.lastQuery.expectedContext.profile;
            const wrote = outcome.previewOutcome !== undefined ? outcome.previewOutcome === "committed"
                : outcome.access === "write" || outcome.affected !== undefined && outcome.affected !== null;
            root.lastQuery = Object.assign({}, root.lastQuery, { wrote: wrote });
            root.status = outcome.previewOutcome !== undefined ? outcome.message : DataSourceKinds.querySummary(outcome, profile.engine || "postgres");
            if (root.pendingDatabase !== "") {
                const created = DataSourceKinds.cloneProfile(profile);
                created.name = profile.name + "-" + root.pendingDatabase;
                created.database = root.pendingDatabase;
                root.pendingDatabase = "";
                root.databaseCreated(created);
            }
        } else {
            root.lastQuery = Object.assign({}, root.lastQuery, { failed: true });
            root.pendingDatabase = "";
            root.status = outcome.message || qsTr("A consulta falhou.");
            if (outcome.secretRequired === true) root.secretNeeded(Object.assign({}, root.lastQuery));
        }
        if (outcome.catalogUpdate === "reload") root.dataSourceController.catalog.invalidate(outcome.name);
    }

    function fail(message, code, operation) {
        if (!operation || !root.matches(operation)) return;
        root.querying = false;
        root.status = message;
        if (code === "DRIVER_APPROVAL_REQUIRED") return;
        if (code === "WRITE_CONFIRMATION_REQUIRED") {
            root.writeConfirmationRequired = true;
            root.dataSourceController.impact.begin(root.lastQuery.name, root.lastQuery.sql);
        } else {
            root.lastQuery = Object.assign({}, root.lastQuery, { failed: true });
            root.columns = [];
            root.rows = [];
            root.pendingDatabase = "";
            root.dataSourceController.secretRequired = code === "SECRET_REQUIRED";
            if (code === "SECRET_REQUIRED") root.secretNeeded(Object.assign({}, root.lastQuery));
        }
    }
}
