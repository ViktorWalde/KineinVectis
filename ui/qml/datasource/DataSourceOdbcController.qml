pragma ComponentBehavior: Bound
import QtQuick

// Lista sem conectar; o consentimento so' sai no gesto explicito do dialogo.
QtObject {
    id: root

    property var dataSourceController: null
    property string workspaceRoot: ""
    property var sources: []
    property bool loading: false
    property string message: ""
    property var pending: null
    property bool authorizing: false
    readonly property bool open: root.pending !== null
    readonly property bool canAuthorize: root.open && !root.authorizing
        && root.pending.workspace === root.workspaceRoot
        && JSON.stringify(root.dataSourceController.profileByName(root.pending.name)) === root.pending.profile

    signal sourcesRequested()
    signal authorizeRequested(string name, string identity, string workspace)

    onWorkspaceRootChanged: {
        sources = [];
        loading = false;
        message = "";
        cancel();
    }

    function refresh() {
        loading = true;
        message = "";
        sourcesRequested();
    }

    function handleSources(entries) {
        sources = entries;
        loading = false;
        message = entries.length === 0 ? qsTr("Nenhum DSN registrado. Configure-o no unixODBC e atualize a lista.") : "";
    }

    function handleRequired(method, details) {
        if (details.workspace !== workspaceRoot) return;
        const profile = dataSourceController.profileByName(details.name);
        if (profile === null) return;
        const query = details.query || null;
        if (method === "datasource.query" && (query === null || dataSourceController.lastQuery === null
                || query.name !== dataSourceController.lastQuery.name || query.sql !== dataSourceController.lastQuery.sql)) return;
        pending = Object.assign({}, details, { method: method, query: query, profile: JSON.stringify(profile) });
        authorizing = false;
        message = "";
    }

    function confirm() {
        if (!canAuthorize) return;
        authorizing = true;
        authorizeRequested(pending.name, pending.identity, pending.workspace);
    }

    function cancel() {
        pending = null;
        authorizing = false;
    }

    function handleAuthorized(name, identity, workspace) {
        if (pending === null || !authorizing || pending.name !== name || pending.identity !== identity
                || pending.workspace !== workspace || workspace !== workspaceRoot) return;
        const operation = pending;
        const current = JSON.stringify(dataSourceController.profileByName(name));
        cancel();
        if (current !== operation.profile) return;
        if (operation.method === "datasource.test") dataSourceController.testProfile(name);
        else if (operation.method === "datasource.introspect") dataSourceController.introspectProfile(name);
        else if (operation.method === "datasource.query") {
            const query = operation.query;
            const last = dataSourceController.lastQuery;
            if (last !== null && last.name === query.name && last.sql === query.sql)
                dataSourceController.runOn(name, query.sql, query.confirmWrite, query.maxRows);
        }
    }

    function handleFailed(method, text) {
        if (method.indexOf("datasource.odbc.") !== 0) return;
        loading = false;
        authorizing = false;
        message = text;
    }
}
