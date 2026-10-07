import QtQuick
import KineinVectis

Item {
    id: root
    property var queries: []
    property var opens: []
    property var appends: []
    DataSourceController {
        id: bankController
        workspaceRoot: "/p"
        onQueryRequested: (name, password, sql, confirmed, maxRows, context, confirmation) =>
            root.queries.push({ name: name, sql: sql, confirmed: confirmed, context: context })
    }
    DataSourceTree {
        id: tree
        profiles: bankController.profiles
    }
    DatabaseTreeActions { id: actions; controller: bankController; treeModel: tree }
    Connections {
        target: actions
        function onConsoleStatementRequested(name, text) { bankController.consoles.open(name, text); }
        function onTableDataRequested(connection, engine, schema, table, readSql) { bankController.consoles.tableData(connection, engine, schema, table, readSql); }
    }
    Connections {
        target: bankController.consoles
        function onConsoleRequested(name, context) { root.opens.push(context); }
        function onAppendRequested(path, text) { root.appends.push({ path: path, text: text }); }
    }
    function check(ok, label) {
        if (!ok) console.error("FALHOU: " + label);
        return ok ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        failures += check(DataSourceKinds.statementName("mongo", "select") === "find"
            && DataSourceKinds.statementName("mongo", "insert") === "insertOne"
            && DataSourceKinds.statementName("sqlite", "update") === "UPDATE", "rótulos usam o comando de cada motor");
        const profile = Object.assign(bankController.emptyDraft(), { name: "loja", engine: "sqlite", database: "/p/a.db" });
        bankController.handleList([profile]);
        const path = "/p/.kinein/consoles/v1/loja.sql";
        bankController.consoles.catalogue([{ name: "loja", paths: [path] }], "/p");
        const statements = { select: 'SELECT * FROM "main"."x" LIMIT 200;', insert: 'INSERT INTO "x" VALUES (<value>);', update: 'UPDATE "x" SET "v" = <value> WHERE <condition>;', clear: 'DELETE FROM "main"."x";', remove: 'DROP TABLE "main"."x";' };
        tree.structures = { [profile.name]: { schemas: [{ name: "main", tables: [{ name: "x", kind: "table", columns: [], statements: statements }] }], collections: [] } };
        tree.toggle(tree.key(["c", "loja"]));
        const row = tree.rows[1];
        failures += check(row.readSql === statements.select, "leitura vem intacta do core");
        actions.showRow(row.key);
        failures += check(actions.entries().filter(item => item.action.indexOf("database.template.") === 0).length === 3, "modelos disponíveis por objeto");
        actions.activateMenu("database.template.update");
        failures += check(root.queries.length === 0 && root.opens.length === 1, "modelo só pede abertura, sem consulta");
        bankController.consoles.handleResolved(Object.assign({}, root.opens[0], { path: path }));
        failures += check(root.appends.length === 1 && root.appends[0].text === statements.update, "texto do core chega ao editor sem reconstrução");
        actions.dispatch("database.console", row);
        bankController.consoles.handleResolved(Object.assign({}, root.opens[1], { path: path }));
        failures += check(root.appends[1].text === statements.select && root.queries.length === 0, "abrir console da tabela prepara SELECT");
        actions.dispatch("database.data", row);
        failures += check(root.queries[0].sql === statements.select && root.queries[0].confirmed === false, "ver dados reaproveita a mesma instrução");
        actions.showRow(row.key);
        actions.activateMenu("database.clear");
        failures += check(root.queries[1].sql === statements.clear && root.queries[1].confirmed === false, "esvaziar usa consulta normal não confirmada");
        bankController.queries.fail("confirmação", "WRITE_CONFIRMATION_REQUIRED", Object.assign({ name: "loja" }, root.queries[1].context));
        failures += check(bankController.impact.open && bankController.impact.sql === statements.clear, "recusa do core abre impacto existente");
        bankController.impact.cancel();
        failures += check(root.queries.length === 2, "cancelar impacto não envia escrita");
        actions.dispatch("database.template.insert", row);
        const stale = root.opens[2];
        bankController.handleList([Object.assign({}, profile, { readOnly: true })]);
        bankController.consoles.handleResolved(Object.assign({}, stale, { path: path }));
        failures += check(root.appends.length === 2, "modelo de perfil antigo descartado");
        actions.showRow(row.key);
        failures += check(actions.entries().filter(item => item.action === "database.clear" || item.action === "database.remove").every(item => !item.enabled), "somente leitura desabilita ações de escrita");
        actions.activateMenu("database.remove");
        failures += check(root.queries.length === 2, "ação desabilitada não despacha");
        bankController.consoles.tableData("loja", "sqlite", "main", "x", "");
        failures += check(root.queries.length === 2 && bankController.queryStatus.indexOf("estrutura") >= 0, "catálogo antigo não gera SQL em QML");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
