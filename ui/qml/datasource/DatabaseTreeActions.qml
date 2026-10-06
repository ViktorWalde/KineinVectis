pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Ações da árvore. Só coordena os caminhos existentes; SQL e política ficam
// com seus donos. Um menu conserva a chave e o contexto, nunca o índice.
QtObject {
    id: root

    property var controller: null
    property var treeModel: null
    property bool menuOpen: false
    property string menuKind: ""
    property string menuKey: ""
    property string menuContext: ""
    readonly property var selectedRow: root.treeModel ? root.treeModel.selectedRow : null
    readonly property var menuRow: root.treeModel ? root.treeModel.rows.find(item => item.key === root.menuKey) || null : null
    readonly property string liveContext: root.contextFor(root.menuRow)
    onLiveContextChanged: if (root.menuOpen && root.menuKind === "row" && root.liveContext !== root.menuContext) root.menuOpen = false
    readonly property string workspace: root.controller ? root.controller.workspaceRoot : ""
    onWorkspaceChanged: root.menuOpen = false

    signal consoleRequested(string name)
    signal editRequested(string name)
    signal tableDataRequested(string connection, string engine, string schema, string table, string readSql)
    signal newRequested(string engine)
    signal discoveryRequested()

    function contextFor(row) {
        if (!row || !root.controller) return "";
        const profile = root.controller.profileByName(row.connection);
        if (!profile) return "";
        return JSON.stringify([root.controller.workspaceRoot, root.controller.queries.profileKey(profile)]);
    }

    function canRefresh(row) {
        return row !== null && root.contextFor(row) !== ""
            && DataSourceMap.get(root.controller.readingNames, row.connection) !== true;
    }

    function showRow(key) {
        root.menuOpen = false;
        root.treeModel.select(key);
        root.menuKind = "row";
        root.menuKey = key;
        root.menuContext = root.liveContext;
        root.menuOpen = root.menuContext !== "";
    }

    function showNew() {
        root.menuOpen = false;
        root.menuKind = "new";
        root.menuKey = "";
        root.menuOpen = true;
    }

    function entries() {
        if (root.menuKind === "new") {
            const items = ["postgres", "sqlite", "mongo", "odbc"].map(engine => ({
                label: DataSourceKinds.engineName(engine) + "…", action: "new." + engine,
                icon: DataSourceKinds.engineIcon(engine), iconColor: DataSourceKinds.engineColor(engine), enabled: true }));
            items.push({ separator: true, label: "", action: "", enabled: false });
            items.push({ label: qsTr("Desta máquina…"), action: "database.discover", icon: "search", enabled: true });
            return items;
        }
        const row = root.menuRow;
        if (!row || root.liveContext !== root.menuContext) return [];
        const items = [];
        if (DataSourceKinds.hasData(row.kind)) items.push({ label: qsTr("Ver dados"), action: "database.data", icon: "table", enabled: true });
        if (DataSourceKinds.isConnection(row.kind) || DataSourceKinds.hasData(row.kind)) {
            items.push({ label: qsTr("Abrir console"), action: "database.console", icon: "terminal", enabled: true });
        }
        items.push({ label: qsTr("Ler estrutura de novo"), action: "database.refresh", icon: "refresh", shortcut: "F5", enabled: root.canRefresh(row) });
        if (DataSourceKinds.isConnection(row.kind)) items.push({ label: qsTr("Editar conexão…"), action: "database.edit", icon: "settings", enabled: true });
        if (["connection", "schema", "table", "view", "collection", "timeseries", "column", "field"].indexOf(row.kind) >= 0) {
            items.push({ separator: true, label: "", action: "", enabled: false });
            items.push({ label: qsTr("Copiar nome"), action: "database.copy", icon: "copy", enabled: true });
        }
        return items;
    }

    function dispatch(action, row) {
        if (action.indexOf("new.") === 0) {
            const engine = action.slice(4);
            if (["postgres", "sqlite", "mongo", "odbc"].indexOf(engine) >= 0) root.newRequested(engine);
            return;
        }
        if (action === "database.discover") { root.discoveryRequested(); return; }
        if (action === "database.collapse") { root.treeModel.collapseAll(); return; }
        if (!row || root.contextFor(row) === "") return;
        if (action === "database.refresh" && root.canRefresh(row)) root.controller.introspectProfile(row.connection);
        else if (action === "database.console" && (DataSourceKinds.isConnection(row.kind) || DataSourceKinds.hasData(row.kind))) root.consoleRequested(row.connection);
        else if (action === "database.edit" && DataSourceKinds.isConnection(row.kind)) root.editRequested(row.connection);
        else if (action === "database.data" && DataSourceKinds.hasData(row.kind)) root.tableDataRequested(row.connection, row.engine, row.schema || "", row.table, row.readSql || "");
        else if (action === "database.copy") Clipboard.setText(row.name);
    }

    function activateMenu(action) {
        const row = root.menuRow;
        const allowed = root.menuOpen && root.entries().some(item => item.action === action && item.enabled);
        root.menuOpen = false;
        if (allowed) root.dispatch(action, row);
    }
}
