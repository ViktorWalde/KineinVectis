import QtQuick
import KineinVectis

// Identidade e correlação na UI; extração SQL é provada no core real.
Item {
    id: root
    property var statements: []
    property var opens: []
    property var queries: []
    property var files: []
    property int results: 0

    DataSourceController {
        id: controller
        workspaceRoot: "/p"
        onQueryRequested: (name, password, sql, confirmed, maxRows, context, confirmation) =>
            root.queries.push({ name: name, sql: sql, preview: context.preview })
    }
    Connections {
        target: controller.consoles
        function onStatementRequested(operation) { root.statements.push(operation); }
        function onConsoleRequested(name, context) { root.opens.push(context); }
        function onOpenFileRequested(path) { root.files.push(path); }
        function onResultsRequested() { root.results += 1; }
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    function response(operation, statement) { return Object.assign({}, operation, { statement: statement }); }
    Component.onCompleted: {
        let failures = 0;
        const first = Object.assign(controller.emptyDraft(), { name: "meu pg", engine: "sqlite", database: "/p/a.db" });
        const second = Object.assign({}, first, { name: "meu_pg", database: "/p/b.db" });
        controller.handleList([first, second]);
        const consoles = controller.consoles;
        const firstPath = "/p/.kinein/consoles/v1/first.sql";
        const secondPath = "/p/.kinein/consoles/v1/second.sql";
        consoles.catalogue([{ name: first.name, paths: [firstPath] }, { name: second.name, paths: [secondPath] }], "/p");
        failures += check(consoles.connectionFor(firstPath) === first.name && consoles.connectionFor(secondPath) === second.name,
            "binding do core separa nomes que colidiam");
        for (const path of [firstPath + "/child.sql", firstPath + ".bak", "/p/.kinein/consoles/meu_pg.sql", "/p/src/a.sql"])
            failures += check(!consoles.isConsole(path), "caminho exato: " + path);
        consoles.open(first.name);
        consoles.open(second.name);
        consoles.handleResolved(Object.assign({}, root.opens[0], { path: firstPath }));
        failures += check(root.files.length === 0, "abertura velha descartada");
        consoles.handleResolved(Object.assign({}, root.opens[1], { path: secondPath }));
        failures += check(root.files.length === 1 && root.files[0] === secondPath, "abertura atual");
        const text = "SELECT '; DELETE', '😀';";
        failures += check(consoles.runFromEditor(firstPath, text, 12, 12, 12, true), "console envia extração");
        const request = root.statements[0];
        failures += check(request.text === text && request.cursor === 12 && request.preview === undefined
            && request.expectedContext.profile.name === first.name && root.queries.length === 0, "texto não interpretado pela UI");
        consoles.handleStatement(response(request, "SELECT '; DELETE', '😀'"));
        failures += check(root.queries.length === 1 && root.queries[0].name === first.name
            && root.queries[0].sql === "SELECT '; DELETE', '😀'" && root.queries[0].preview === true, "resposta atual passa pela consulta existente");
        consoles.handleStatement(response(request, "DELETE FROM t"));
        failures += check(root.queries.length === 1, "resposta consumida uma vez");
        consoles.runFromEditor(firstPath, "SELECT 1", 0, 0, 0);
        consoles.runFromEditor(secondPath, "SELECT 2", 0, 0, 0);
        consoles.handleStatement(response(root.statements[1], "SELECT 1"));
        failures += check(root.queries.length === 1, "resposta de extração velha descartada");
        const current = root.statements[2];
        consoles.handleStatement(Object.assign(response(current, "SELECT 2"), { path: firstPath }));
        failures += check(root.queries.length === 1, "caminho trocado descartado");
        consoles.handleStatement(response(current, "SELECT 2"));
        failures += check(root.queries.length === 2, "extração mais recente");
        consoles.runFromEditor(firstPath, "SELECT 3", 0, 0, 0);
        controller.handleList([Object.assign({}, first, { database: "/p/other.db" }), second]);
        consoles.handleStatement(response(root.statements[3], "SELECT 3"));
        failures += check(root.queries.length === 2, "perfil mudou, não executa");
        consoles.runFromEditor(secondPath, "SELECT 4", 0, 0, 0);
        const pending = root.statements[4];
        consoles.fail("datasource.console.statement", "recusa", { name: second.name, clientContext: "outro" });
        failures += check(consoles.pendingStatement !== null, "falha velha não cancela atual");
        consoles.fail("datasource.console.statement", "recusa", pending);
        failures += check(consoles.pendingStatement === null && controller.queryStatus === "recusa", "recusa correlacionada");
        consoles.catalogue([{ name: first.name, paths: [firstPath] }, { name: second.name, paths: [firstPath] }], "/p");
        failures += check(consoles.connectionFor(firstPath) === "", "binding ambíguo recusado");
        controller.workspaceRoot = "/outro";
        consoles.handleStatement(response(pending, "DELETE FROM t"));
        failures += check(root.queries.length === 2 && consoles.bindings.length === 0, "projeto mudou, não executa");
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
