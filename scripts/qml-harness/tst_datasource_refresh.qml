import QtQuick
import KineinVectis

Item {
    id: root
    property var refreshes: []
    property int failures: 0
    DataSourceController { id: controller; workspaceRoot: "/p" }
    DataSourceTree {
        id: tree
        workspaceRoot: controller.workspaceRoot
        profiles: controller.profiles
        structures: controller.structures
        readingNames: controller.readingNames
    }
    Connections {
        target: controller
        function onIntrospectRequested(name, password, context) { root.refreshes.push({ name: name, context: context }); }
    }
    function check(ok, label) { if (!ok) { root.failures++; console.error("FALHOU: " + label); } }
    function schema(name) { return [{ name: "main", tables: [{ name: name, kind: "table", columns: [] }] }]; }
    function outcome(invalidated, success) {
        const query = controller.lastQuery;
        return { name: query.name, clientContext: query.clientContext, success: success !== false,
            columns: ["id"], rows: [["preservado"]], elapsedMs: 1, access: "write",
            catalogUpdate: invalidated ? "reload" : "none", message: success === false ? "lote falhou" : "" };
    }
    function complete(name, schemas, secret) {
        const operation = controller.catalog.pending["introspect:" + name];
        controller.handleIntrospected(name, !secret, schemas, [], secret ? "senha necessária" : "", secret === true, operation.clientContext);
    }
    Component.onCompleted: {
        const profile = Object.assign(DataSourceKinds.emptyProfile(), { name: "__proto__", engine: "sqlite", host: "", database: "/p/a.db" });
        const other = Object.assign({}, profile, { name: "outro", database: "/p/b.db" });
        controller.handleList([profile, other]);
        controller.select(profile.name);
        controller.introspectProfile(profile.name);
        root.complete(profile.name, root.schema("clientes"));
        tree.toggle(tree.key(["c", profile.name]));
        const selected = tree.rows.find(row => row.kind === "table").key;
        tree.select(selected);
        root.refreshes = [];
        controller.runOn(profile.name, "CREATE TABLE nova(id)", false);
        const created = root.outcome(true);
        controller.handleQueried(created);
        root.check(root.refreshes.length === 1 && root.refreshes[0].name === profile.name, "DDL relê somente sua conexão");
        if (root.refreshes.length === 0) { Qt.exit(1); return; }
        root.check(controller.queryRows[0][0] === "preservado" && !controller.querying, "releitura não apaga resultado nem reabre consulta");
        controller.handleQueried(created);
        root.check(Object.keys(controller.catalog.invalidated).length === 0, "evento terminal duplicado não enfileira outra leitura");
        controller.runOn(profile.name, "ALTER TABLE clientes ADD COLUMN extra", false);
        controller.handleQueried(root.outcome(true));
        controller.runOn(profile.name, "CREATE TABLE outra(id)", false);
        controller.handleQueried(root.outcome(true));
        root.check(root.refreshes.length === 1 && Object.keys(controller.catalog.invalidated).length === 1, "invalidações ocupadas agrupadas por conexão");
        root.complete(profile.name, root.schema("snapshot-antigo"));
        root.check(root.refreshes.length === 2 && controller.structures[profile.name].schemas[0].tables[0].name === "clientes", "snapshot anterior descartado e uma releitura posterior");
        root.complete(profile.name, root.schema("clientes"));
        root.check(tree.selectedKey === selected && tree.selectedRow.kind === "table", "seleção conserva identidade após releitura");
        controller.runOn(profile.name, "SELECT 'CREATE TABLE falsa'", false);
        controller.handleQueried(root.outcome(false));
        root.check(root.refreshes.length === 2, "resultado sem aviso não interpreta SQL na UI");
        controller.runOn(profile.name, "CREATE TABLE parcial(id); instrução inválida", true);
        controller.handleQueried(root.outcome(true, false));
        root.check(root.refreshes.length === 3 && controller.queryStatus === "lote falhou", "falha parcial relê catálogo e conserva erro da consulta");
        controller.runOn(profile.name, "CREATE TABLE posterior(id)", false);
        controller.handleQueried(root.outcome(true));
        const failedRead = controller.catalog.pending["introspect:" + profile.name];
        controller.handleFailed("datasource.introspect", "rede indisponível", "INTERNAL_ERROR", failedRead);
        root.check(root.refreshes.length === 4 && DataSourceMap.get(controller.readingNames, profile.name), "falha com invalidação posterior agenda só uma nova leitura");
        controller.runOn(profile.name, "CREATE TABLE depois(id)", false);
        controller.handleQueried(root.outcome(true));
        root.complete(profile.name, [], true);
        root.check(root.refreshes.length === 4 && Object.keys(controller.catalog.invalidated).length === 0, "credencial necessária consome fila sem retry automático");
        controller.runOn(other.name, "CREATE TABLE vizinha(id)", false);
        const stale = root.outcome(true);
        controller.handleList([profile, Object.assign({}, other, { database: "/p/changed.db" })]);
        controller.handleQueried(stale);
        root.check(root.refreshes.length === 4, "resultado de outro perfil não pede catálogo novo");
        controller.select(profile.name);
        controller.introspectProfile(profile.name);
        controller.runOn(profile.name, "CREATE TABLE fila(id)", false);
        controller.handleQueried(root.outcome(true));
        const beforeWorkspace = root.refreshes.length;
        controller.workspaceRoot = "/other";
        root.check(Object.keys(controller.catalog.invalidated).length === 0 && root.refreshes.length === beforeWorkspace, "workspace descarta releitura pendente");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
