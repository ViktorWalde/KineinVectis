import QtQuick
import KineinVectis

// HISTORICO DE CONSULTAS (passo 13b, roadmaps/59 §5.2.1): o controlador e o
// menu REAIS, com a ponte falsa.
//
// O que se prova: abrir pede a lista da conexao e mostra "Carregando…"; uma
// resposta de outra conexao nao entra; a linha e' a instrucao numa linha e
// curta, com a hora e o aviso da que falhou; escolher insere o texto e fecha;
// limpar pede confirmacao antes do pedido; a falha do historico nao mexe na
// consulta em andamento; trocar de projeto esquece a lista.
//
// MUTACOES QUE PROVAM O GATE: limpar sem confirmar; o `handleListed` sem
// conferir o nome; a falha do historico caindo no ramo generico do controller.
Item {
    id: root

    width: 600
    height: 400

    property int failures: 0
    property var listed: []
    property var cleared: []
    property var chosen: []
    property int dismissed: 0

    DataSourceController {
        id: bank

        workspaceRoot: "/projeto"
    }

    Connections {
        target: bank.history
        function onListRequested(name) { root.listed.push(name); }
        function onClearRequested(name) { root.cleared.push(name); }
    }

    DataSourceHistoryMenu {
        id: menu

        anchors.fill: parent
        visible: false
        history: bank.history
        onStatementChosen: sql => root.chosen.push(sql)
        onDismissRequested: root.dismissed += 1
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            root.failures += 1;
        }
    }

    function actions() {
        return menu.entries().map(item => item.action + (item.enabled ? "" : "(off)")).join(" ");
    }

    Component.onCompleted: {
        const now = new Date();
        const earlier = new Date(now.getFullYear() - 1, 0, 2, 9, 5);
        bank.history.open("estacao");
        root.check(JSON.stringify(root.listed) === JSON.stringify(["estacao"]) && bank.history.loading,
                   "abrir pede a lista da conexao");
        root.check(menu.entries().length === 1 && menu.entries()[0].enabled === false, "carregando: " + root.actions());

        const entries = [{ sql: "SELECT *\n  FROM leituras\n WHERE id = 1", at: Math.floor(now.getTime() / 1000), outcome: "ok", rows: 1 },
                         { sql: "SELECT " + "x".repeat(100), at: Math.floor(earlier.getTime() / 1000), outcome: "failed" }];
        bank.history.handleListed("oficina", entries);
        root.check(bank.history.loading && bank.history.entries.length === 0, "resposta de outra conexao entrou");
        bank.history.handleListed("estacao", entries);
        const items = menu.entries();
        root.check(items.length === 4 && items[0].label === "SELECT * FROM leituras WHERE id = 1",
                   "linha numa linha: " + JSON.stringify(items[0]));
        root.check(items[1].label.length === 72 && items[1].label.endsWith("…") && items[1].icon === "warning",
                   "longa cortada e falha avisada: " + JSON.stringify(items[1]));
        root.check(items[0].shortcut === Qt.formatDateTime(now, "HH:mm")
                   && items[1].shortcut === Qt.formatDateTime(earlier, "dd/MM HH:mm"), "hora de hoje e de antes");

        // escolher insere o texto inteiro e fecha
        menu.dispatch("history:0");
        root.check(root.chosen.length === 1 && root.chosen[0] === entries[0].sql && root.dismissed === 1,
                   "escolher insere a instrucao e fecha");

        // limpar pede confirmacao antes do pedido
        menu.dispatch("history.clear");
        root.check(root.cleared.length === 0 && root.actions() === "history.clear.confirm history.clear.cancel",
                   "limpar sem confirmar: " + root.actions());
        menu.dispatch("history.clear.cancel");
        root.check(root.cleared.length === 0 && !menu.confirming, "cancelar nao limpa");
        menu.dispatch("history.clear");
        menu.dispatch("history.clear.confirm");
        root.check(JSON.stringify(root.cleared) === JSON.stringify(["estacao"]) && root.dismissed === 2,
                   "confirmar pede a limpeza e fecha");
        bank.history.handleCleared("estacao");
        root.check(bank.history.entries.length === 0
                   && menu.entries()[0].label === qsTr("Nada executado nesta conexão ainda"), "lista vazia depois de limpar");

        // a falha do historico nao mexe na consulta em andamento
        bank.history.open("estacao");
        bank.queries.querying = true;
        bank.handleFailed("datasource.history", "historico ilegivel", "INTERNAL_ERROR");
        root.check(bank.queries.querying && bank.errorText === "" && bank.history.error === "historico ilegivel"
                   && !bank.history.loading, "falha do historico no lugar certo");

        // trocar de projeto esquece a lista
        bank.history.handleListed("estacao", entries);
        bank.workspaceRoot = "/outro";
        root.check(bank.history.name === "" && bank.history.entries.length === 0, "projeto novo esquece");

        if (root.failures !== 0) console.error("FALHAS " + root.failures);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
