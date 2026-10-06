import QtQuick
import "KvLists.js" as KvLists

// A CONFIRMACAO DE UMA ESCRITA, com o impacto medido (2026-10-03, pedido do
// autor: "quando for um delete muito destrutivo, aparecer um painel
// perguntando se o usuario quer executar, exibindo o comando e a
// consequencia" — porque "ja' ocorreu e ocorre do desenvolvedor apagar o
// banco de dados inteiro sem ter essa intencao").
//
//   console (Ctrl+Enter) ─▶ datasource.query ─▶ recusa WRITE_CONFIRMATION_REQUIRED
//        ─▶ begin(): o painel abre, "medindo…" ─▶ datasource.impact (job de
//           contagens SO' DE LEITURA) ─▶ handleMeasured(): o que cada
//           instrucao faz, com as linhas
//        ─▶ Executar: escrita comum, um clique; DESTRUTIVA, so' depois de
//           digitar o nome do que some (o modelo do "apagar repositorio")
//        ─▶ runConfirmed ─▶ datasource.query com confirmWrite
//
// Medir e' de graca e nao muda nada; a escrita so' sai daqui confirmada.
// Puro sobre o que o core mediu: o painel (SqlImpactDialog) so' desenha.
QtObject {
    id: root

    property var dataSourceController: null

    property bool open: false
    property bool measuring: false
    property string name: ""
    property string sql: ""
    // "write" | "destructive"; "" enquanto mede.
    property string severity: ""
    property var statements: []
    property string errorText: ""
    // O que a pessoa digitou para confirmar a destrutiva.
    property string typed: ""

    property string typedConnection: ""
    property string targetName: ""
    property bool requiresConnection: false
    property string clientContext: ""
    property string queryContext: ""
    property int serial: 0
    property var expectedContext: null

    function isDestructive(statement) {
        return statement.severity === "destructive";
    }

    readonly property bool destructive: root.severity === "destructive" || root.errorText !== "" || root.requiresConnection
    readonly property string confirmName: root.targetName === "" ? root.name : root.targetName
    readonly property bool canRun: root.open && !root.measuring && root.contextCurrent()
        && (!root.destructive || root.typed === root.confirmName)
        && (!root.requiresConnection || root.typedConnection.trim() === root.name)

    signal impactRequested(string name, string sql, var context)
    signal runConfirmed(string name, string sql, var confirmation)

    function contextCurrent() {
        if (root.dataSourceController === null) return true;
        const query = root.dataSourceController.lastQuery;
        return root.dataSourceController.queries.current() && query !== null
            && query.name === root.name && query.sql === root.sql && query.clientContext === root.queryContext;
    }

    function begin(name, sql) {
        const query = root.dataSourceController ? root.dataSourceController.lastQuery : null;
        if (root.dataSourceController !== null && (query === null || query.name !== name || query.sql !== sql)) return;
        root.name = name;
        root.sql = sql;
        root.severity = "";
        root.statements = [];
        root.errorText = "";
        root.typed = "";
        root.typedConnection = "";
        root.targetName = "";
        root.requiresConnection = false;
        root.expectedContext = query ? query.expectedContext : null;
        root.queryContext = query ? query.clientContext : "standalone";
        root.serial += 1;
        root.clientContext = root.queryContext + ".impact." + String(root.serial);
        root.measuring = true;
        root.open = true;
        root.impactRequested(name, sql, { clientContext: root.clientContext, expectedContext: root.expectedContext });
    }

    function handleMeasured(event) {
        if (!root.open || !root.contextCurrent() || event.name !== root.name || event.sql !== root.sql
                || event.clientContext !== root.clientContext) return;
        root.measuring = false;
        root.severity = event.severity;
        root.targetName = event.confirmationTarget || "";
        root.requiresConnection = event.requiresConnection === true;
        root.statements = KvLists.listOf(event.statements).filter(s => s.severity !== "read");
    }

    function handleFailed(message, operation, code) {
        if (!root.open || !root.contextCurrent() || !operation || operation.name !== root.name
                || operation.clientContext !== root.clientContext) return;
        root.measuring = false;
        if (code === "SECRET_REQUIRED" && root.dataSourceController !== null) {
            root.cancel();
            root.dataSourceController.secrets.request("query", Object.assign({}, root.dataSourceController.lastQuery));
            return;
        }
        root.errorText = message;
        root.targetName = root.name;
        root.requiresConnection = root.expectedContext !== null && root.expectedContext.profile.production === true;
    }

    function confirm() {
        if (!root.canRun) return;
        root.open = false;
        const confirmation = root.destructive || root.requiresConnection
            ? { connection: root.typedConnection.trim(), target: root.typed } : ({});
        root.runConfirmed(root.name, root.sql, confirmation);
    }

    function cancel() { root.open = false; }

    function rows(count, one, many) {
        return count === 1 ? one : many.arg(count);
    }

    function joined(list) {
        return KvLists.listOf(list).join(", ");
    }

    // O que a instrucao faz, em uma frase, com o numero que o core contou.
    function describe(s) {
        const target = root.joined(s.targets);
        const known = s.rows !== undefined && s.rows !== null;
        const n = known ? s.rows : 0;
        const filtered = s.filter !== undefined && s.filter !== "";
        const total = s.totalRows !== undefined && s.totalRows !== null ? s.totalRows : -1;
        const unknown = qsTr(" (não deu para contar%1)").arg(s.note ? ": " + s.note : "");
        switch (s.kind) {
        case "delete":
        case "update": {
            const verb = s.kind === "delete" ? qsTr("Apaga") : qsTr("Altera");
            if (!known) return qsTr("%1 linhas de %2%3").arg(verb).arg(target).arg(unknown);
            if (!filtered) return qsTr("%1 TODAS as linhas de %2: %3").arg(verb).arg(target)
                                  .arg(root.rows(n, qsTr("1 linha"), qsTr("%1 linhas")));
            if (total >= 0 && n === total && total > 0) {
                return qsTr("%1 TODAS as %2 linhas de %3 — o WHERE pega a tabela inteira").arg(verb).arg(n).arg(target);
            }
            return qsTr("%1 %2 de %3").arg(verb)
                .arg(root.rows(n, qsTr("1 linha"), qsTr("%1 linhas"))).arg(target)
                + (total >= 0 ? qsTr(" (a tabela tem %1)").arg(total) : "");
        }
        // MongoDB (0.155.0): documentos e colecoes, na mesma forma.
        case "mongoDelete":
        case "mongoUpdate": {
            const verb = s.kind === "mongoDelete" ? qsTr("Apaga") : qsTr("Altera");
            const docs = root.rows(n, qsTr("1 documento"), qsTr("%1 documentos"));
            if (!known) return qsTr("%1 documentos de %2%3").arg(verb).arg(target).arg(unknown);
            if (!filtered && total > 0 && n === total) return qsTr("%1 TODOS os documentos de %2: %3").arg(verb).arg(target).arg(docs);
            if (total > 0 && n === total) {
                return qsTr("%1 TODOS os %2 documentos de %3 — o filtro pega a coleção inteira").arg(verb).arg(n).arg(target);
            }
            return qsTr("%1 %2 de %3").arg(verb).arg(docs).arg(target)
                + (total >= 0 ? qsTr(" (a coleção tem %1)").arg(total) : "");
        }
        case "mongoInsert":
            return qsTr("Insere %1 em %2").arg(root.rows(n, qsTr("1 documento"), qsTr("%1 documentos"))).arg(target);
        case "dropCollection":
            return qsTr("Remove a coleção %1").arg(target)
                + (known ? qsTr(" e %1 dela").arg(root.rows(n, qsTr("1 documento"), qsTr("%1 documentos"))) : unknown);
        case "truncate":
            return qsTr("Esvazia %1").arg(target) + (known ? qsTr(": %1 somem").arg(root.rows(n, qsTr("1 linha"), qsTr("%1 linhas"))) : unknown);
        case "dropTable":
            return qsTr("Remove a tabela %1").arg(target) + (known ? qsTr(" e %1 dela").arg(root.rows(n, qsTr("1 linha"), qsTr("%1 linhas"))) : unknown);
        case "dropColumn":
            return qsTr("Remove a coluna %1 de %2").arg(s.column).arg(target)
                + (known ? qsTr(": %1 somem").arg(root.rows(n, qsTr("1 valor"), qsTr("%1 valores"))) : unknown);
        case "dropSchema":
            return qsTr("Remove o esquema %1").arg(target) + (known ? qsTr(" e %1 dele").arg(root.rows(n, qsTr("1 tabela"), qsTr("%1 tabelas"))) : unknown);
        case "dropDatabase":
            return qsTr("Remove o banco %1 INTEIRO: todas as tabelas e todos os dados").arg(target);
        case "drop":
            return qsTr("Remove %1").arg(target || qsTr("um objeto do banco"));
        case "dropView":
            return qsTr("Remove a visão %1 (os dados das tabelas ficam)").arg(target);
        case "dropIndex":
            return qsTr("Remove o índice %1 (os dados ficam)").arg(target);
        case "replace":
            return qsTr("Substitui registros de %1, podendo apagar os anteriores").arg(target);
        case "insert":
            return qsTr("Insere linhas em %1").arg(target);
        case "create":
            return qsTr("Cria um objeto novo no banco");
        case "alter":
            return qsTr("Muda a estrutura de %1").arg(target);
        default:
            return s.note || qsTr("O core não conseguiu determinar o impacto desta instrução");
        }
    }
}
