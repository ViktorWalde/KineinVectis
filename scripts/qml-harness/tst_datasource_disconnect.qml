pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Dois destinos reais do controller: encerrar um nao descarta o outro.
Item {
    id: root
    property int failures: 0
    property var requests: []
    property var readings: []
    property var queries: []

    DataSourceController {
        id: bank
        workspaceRoot: "/project"
        onIntrospectRequested: (name, password, context) => root.readings.push({ name: name, context: context })
        onQueryRequested: (name, password, sql, confirmed, maxRows, context, confirmation) => root.queries.push({ name: name, context: context })
    }
    Connections {
        target: bank.sessions
        function onRequested(name, context) { root.requests.push(Object.assign({ name: name }, context)); }
    }
    DataSourceTree {
        id: tree
        workspaceRoot: bank.workspaceRoot
        profiles: bank.profiles
        structures: bank.structures
        readingNames: bank.readingNames
        sessionStates: bank.sessions.states
    }
    DatabaseTreeActions { id: actions; controller: bank; treeModel: tree }

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }
    function loaded(name) {
        bank.introspectProfile(name);
        const request = root.readings.pop();
        bank.handleIntrospected(name, true, [{ name: "main", tables: [{ name: "items", columns: [], kind: "table" }] }], [], "", false, request.context.clientContext);
    }
    Component.onCompleted: {
        const profile = name => Object.assign(DataSourceKinds.emptyProfile(), { name: name, engine: "sqlite", host: "", database: "/" + name + ".sqlite" });
        bank.handleList([profile("__proto__"), profile("other")]);
        root.loaded("__proto__");
        root.loaded("other");
        bank.select("__proto__");
        bank.sessionPassword = "segredo-de-prova";
        bank.runOn("__proto__", "SELECT 1", false);
        const query = root.queries.pop();
        bank.handleQueried({ name: "__proto__", clientContext: query.context.clientContext, success: true, columns: ["id"], rows: [["preservado"]] });
        tree.toggle(tree.key(["c", "__proto__"]));
        tree.select(tree.rows.find(row => row.kind === "table" && row.connection === "__proto__").key);
        bank.introspectProfile("__proto__");
        const old = root.readings.pop();
        bank.catalog.invalidate("__proto__");
        bank.introspectProfile("other");
        const other = root.readings.pop();
        const consolePath = "/project/.kinein/consoles/v1/bank.sql";
        const otherConsolePath = "/project/.kinein/consoles/v1/other.sql";
        bank.consoles.catalogue([{ name: "__proto__", paths: [consolePath] },
            { name: "other", paths: [otherConsolePath] }], bank.workspaceRoot);
        bank.consoles.runFromEditor(consolePath, "SELECT 1", 0, 0, 0);
        const oldStatement = Object.assign({}, bank.consoles.pendingStatement);
        const saved = JSON.stringify(bank.profiles);
        const draft = JSON.stringify(bank.draft);
        actions.showRow(tree.key(["c", "__proto__"]));
        actions.activateMenu("database.disconnect");
        const closing = root.requests.pop();
        root.check(!!closing && bank.sessions.busy("__proto__") && !bank.sessions.busy("other"), "menu encaminha desconexao de um destino");
        root.check(!actions.menuOpen && bank.sessionPassword === "", "menu fecha e segredo sai");
        root.check(JSON.stringify(bank.profiles) === saved && JSON.stringify(bank.draft) === draft, "perfil e rascunho permanecem");
        root.check(bank.structures["__proto__"] !== undefined, "catalogo aguarda encerramento real");
        root.check(bank.lastQuery === null && bank.queryRows.length === 0, "consulta anterior descartada sem retry");
        bank.runOn("__proto__", "SELECT 2", false);
        bank.introspectProfile("__proto__");
        bank.sessions.begin("__proto__");
        root.check(root.queries.length === 0 && root.readings.length === 0 && root.requests.length === 0, "pedidos bloqueados sem duplicar desconexao");
        bank.handleIntrospected("__proto__", true, [], [], "", true, old.context.clientContext);
        bank.handleQueried({ name: "__proto__", clientContext: query.context.clientContext, success: true, rows: [["antigo"]], catalogUpdate: "reload" });
        root.check(root.readings.length === 0 && bank.secrets.pending === null, "respostas antigas nao restauram nem pedem senha");
        bank.handleIntrospected("other", true, [{ name: "main", tables: [] }], [], "", false, other.context.clientContext);
        root.check(bank.structures.other.schemas[0].tables.length === 0, "outro pedido continua recebendo resposta");
        bank.sessions.finished(Object.assign({}, closing, { clientContext: "antigo", success: true }));
        root.check(bank.sessions.busy("__proto__"), "token errado ignorado");
        bank.sessions.accepted(Object.assign({}, closing, { jobId: "close-job" }));
        bank.sessions.finished(Object.assign({}, closing, { jobId: "wrong-job", success: true }));
        root.check(bank.sessions.busy("__proto__"), "job errado ignorado");
        bank.sessions.finished(Object.assign({}, closing, { jobId: "close-job", success: true }));
        root.check(!bank.sessions.busy("__proto__") && DataSourceMap.get(bank.structures, "__proto__") === undefined, "somente evento correspondente retira catalogo");
        bank.consoles.handleStatement(Object.assign({}, oldStatement, { statement: "SELECT 1" }));
        root.check(root.queries.length === 0 && bank.lastQuery === null,
            "extracao anterior nao reconecta depois do encerramento");
        root.check(tree.selectedRow.kind === "connection" && tree.selectedRow.detail.indexOf("desconectado") >= 0, "arvore retorna a conexao com estado");
        root.check(JSON.stringify(bank.profiles) === saved && JSON.stringify(bank.draft) === draft && bank.structures.other !== undefined, "perfil e outro catalogo intactos");
        bank.sessions.accepted(Object.assign({}, closing, { jobId: "late-job" }));
        root.check(!bank.sessions.busy("__proto__"), "aceite tardio nao reabre estado terminal");
        root.loaded("__proto__");
        root.check(DataSourceMap.get(bank.sessions.states, "__proto__") === undefined, "leitura explicita reabre pelo caminho atual");
        bank.sessions.begin("__proto__");
        const failed = root.requests.pop();
        bank.consoles.runFromEditor(consolePath, "SELECT 2", 0, 0, 0);
        root.check(bank.consoles.pendingStatement === null, "extracao bloqueada durante encerramento");
        bank.sessions.failed("falha de prova", failed);
        root.check(bank.structures["__proto__"] !== undefined && !bank.sessions.busy("__proto__"), "erro conserva catalogo e libera novo pedido");
        bank.consoles.runFromEditor(otherConsolePath, "SELECT 7", 0, 0, 0);
        const neighborStatement = Object.assign({}, bank.consoles.pendingStatement);
        bank.sessions.begin("__proto__");
        const stale = root.requests.pop();
        bank.consoles.handleStatement(Object.assign({}, neighborStatement, { statement: "SELECT 7" }));
        root.check(root.queries.length === 1 && root.queries[0].name === "other",
            "desconectar conserva a extracao pendente do outro destino");
        bank.handleList([Object.assign({}, profile("__proto__"), { database: "/changed" }), profile("other")]);
        bank.sessions.finished(Object.assign({}, stale, { success: true }));
        root.check(DataSourceMap.get(bank.sessions.states, "__proto__") === undefined, "perfil alterado descarta evento antigo");
        bank.sessions.begin("other");
        const previousWorkspace = root.requests.pop();
        bank.workspaceRoot = "/next";
        bank.sessions.finished(Object.assign({}, previousWorkspace, { success: true }));
        root.check(Object.keys(bank.sessions.states).length === 0 && bank.profiles.length === 0, "workspace antigo nao altera novo estado");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
