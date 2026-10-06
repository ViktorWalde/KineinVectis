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

    function isDestructive(statement) {
        return statement.severity === "destructive";
    }

    readonly property bool destructive: root.severity === "destructive" || root.errorText !== ""
    // O nome que se digita: o alvo da primeira instrucao destrutiva (sem
    // aspas nem esquema); sem medida, o nome da conexao.
    readonly property string confirmName: {
        const first = root.statements.find(s => root.isDestructive(s) && s.targets.length > 0);
        if (first === undefined) return root.name;
        const target = String(first.targets[0]).replace(/"/g, "");
        // No Mongo o ponto pertence ao nome da colecao, nao a um esquema.
        if (first.kind.indexOf("mongo") === 0 || first.kind === "dropCollection") return target;
        return target.substring(target.lastIndexOf(".") + 1);
    }
    readonly property bool canRun: root.open && !root.measuring
                                   && (!root.destructive || root.typed.trim() === root.confirmName)

    signal impactRequested(string name, string sql)
    signal runConfirmed(string name, string sql)

    function begin(name, sql) {
        root.name = name;
        root.sql = sql;
        root.severity = "";
        root.statements = [];
        root.errorText = "";
        root.typed = "";
        root.measuring = true;
        root.open = true;
        root.impactRequested(name, sql);
    }

    // Um evento de outra pergunta (a pessoa ja' cancelou e pediu outra) nao
    // vale para esta.
    function handleMeasured(event) {
        if (!root.open || event.name !== root.name || event.sql !== root.sql) return;
        root.measuring = false;
        root.severity = event.severity;
        root.statements = KvLists.listOf(event.statements).filter(s => s.severity !== "read");
    }

    // Sem medida (senha, rede): o painel continua, mas como DESTRUTIVO — sem
    // saber o tamanho, pede o nome da conexao.
    function handleFailed(message) {
        if (!root.open) return;
        root.measuring = false;
        root.errorText = message;
    }

    function confirm() {
        if (!root.canRun) return;
        root.open = false;
        root.runConfirmed(root.name, root.sql);
    }

    function cancel() {
        root.open = false;
    }

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
            return qsTr("O core não conseguiu determinar o impacto desta instrução");
        }
    }
}
