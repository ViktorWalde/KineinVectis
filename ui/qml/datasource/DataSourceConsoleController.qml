import QtQuick

// O CONSOLE SQL NO EDITOR (2026-10-03, decisao do autor: "editar/criar
// comandos SQL no proprio campo que ja' e' usado para desenvolver codigo").
// O core identifica o arquivo de cada conexão em `.kinein/consoles/`
// (`.mongo` no MongoDB), que abre como aba comum do editor — realce, desfazer,
// salvar sozinho, tudo o que o editor ja' faz. Ctrl+Enter num desses
// arquivos executa a instrucao sob o cursor (ou a selecao) na conexao do
// arquivo; o resultado aparece na secao de dados da janela do Banco.
//
// A UI correlaciona os pedidos; vínculo e instrução são regras do core.
QtObject {
    id: root

    property var dataSourceController: null
    property string workspaceRoot: ""

    signal consoleRequested(string name, var context)
    signal openFileRequested(string path)
    signal appendRequested(string path, string text, var operation)
    signal resultsRequested()

    property var bindings: []
    // O arquivo ativo do editor (a casca o liga) e a conexao dele, se for um
    // console: o cabecalho do console e o "localizar" da arvore leem daqui.
    property string activePath: ""
    readonly property string activeConnection: root.connectionFor(root.activePath)
    property var pendingOpen: null
    property var pendingStatement: null
    property int generation: 0
    property int serial: 0

    signal statementRequested(var operation)
    onWorkspaceRootChanged: {
        root.generation += 1;
        root.bindings = [];
        root.pendingOpen = null;
        root.pendingStatement = null;
    }

    readonly property Connections profileChanges: Connections {
        target: root.dataSourceController
        function onDraftChanged() {
            if (!root.current(root.pendingStatement)) root.pendingStatement = null;
        }
        function onProfilesChanged() {
            if (!root.current(root.pendingOpen)) root.pendingOpen = null;
            if (!root.current(root.pendingStatement)) root.pendingStatement = null;
        }
    }

    function catalogue(bindings, workspace) {
        if (workspace === root.workspaceRoot) root.bindings = bindings;
    }

    // Rótulos são dados de apresentação, sem callback dinâmico no editor.
    function labels() {
        const result = Object.create(null);
        for (const binding of root.bindings) {
            for (const path of binding.paths) {
                const name = root.connectionFor(path);
                if (name !== "") result[path] = qsTr("Console · %1").arg(name);
            }
        }
        return result;
    }

    function isConsole(path) { return root.connectionFor(path) !== ""; }

    function discard(name) {
        if (root.pendingOpen !== null && root.pendingOpen.name === name) root.pendingOpen = null;
        if (root.pendingStatement !== null && root.pendingStatement.name === name) root.pendingStatement = null;
    }

    // Só igualdade com caminhos calculados pelo core. Colisão recusa por padrão.
    function connectionFor(path) {
        if (root.dataSourceController === null) return "";
        const matches = root.bindings.filter(item => item.paths.indexOf(path) >= 0
            && root.dataSourceController.profileByName(item.name) !== null);
        return matches.length === 1 ? matches[0].name : "";
    }

    function current(operation) {
        if (!operation || root.dataSourceController === null) return false;
        const controller = root.dataSourceController;
        const editing = controller.selectedName === operation.name || controller.draft.name === operation.name;
        return (!editing || controller.queries.profileKey(controller.draft) === controller.queries.profileKey(operation.expectedContext.profile))
            && operation.expectedContext.workspace === root.workspaceRoot
            && controller.queries.profileKey(operation.expectedContext.profile)
                === controller.queries.profileKey(controller.profileByName(operation.name));
    }

    function operation(name) {
        const profile = root.dataSourceController.profileByName(name);
        if (profile === null) return null;
        root.serial += 1;
        return { name: name, clientContext: "console." + String(root.generation) + ":" + String(root.serial),
            expectedContext: { workspace: root.workspaceRoot, profile: Object.assign({}, profile) } };
    }

    function matches(pending, response) {
        return root.current(pending) && response && response.name === pending.name
            && response.clientContext === pending.clientContext
            && response.expectedContext && response.expectedContext.workspace === root.workspaceRoot
            && root.dataSourceController.queries.profileKey(response.expectedContext.profile)
                === root.dataSourceController.queries.profileKey(pending.expectedContext.profile);
    }

    function open(name, text) {
        root.pendingOpen = root.operation(name);
        if (root.pendingOpen !== null && text) root.pendingOpen.text = text;
        if (root.pendingOpen !== null) root.consoleRequested(name, root.pendingOpen);
    }

    function handleResolved(response) {
        if (!root.matches(root.pendingOpen, response) || root.connectionFor(response.path) !== response.name) return;
        const operation = root.pendingOpen;
        const text = operation.text || "";
        root.pendingOpen = null;
        if (text !== "") root.appendRequested(response.path, text, operation);
        else root.openFileRequested(response.path);
    }

    // O core escolhe a instrução. Nenhuma regex da UI interpreta SQL.
    function runFromEditor(path, text, cursor, selectionStart, selectionEnd, preview) {
        const name = root.connectionFor(path);
        if (name === "") return false;
        if (root.dataSourceController.sessions.busy(name)) {
            root.dataSourceController.queryStatus = qsTr("A conexão está desconectando; aguarde o encerramento.");
            root.resultsRequested();
            return true;
        }
        const operation = root.operation(name);
        if (!root.current(operation)) {
            root.dataSourceController.queryStatus = qsTr("Salve as alterações da conexão antes de executar.");
            root.resultsRequested();
            return true;
        }
        root.pendingStatement = Object.assign({}, operation, { path: path, preview: preview === true });
        root.statementRequested(Object.assign({}, operation, {
            path: path, text: text, cursor: cursor, selectionStart: selectionStart, selectionEnd: selectionEnd }));
        return true;
    }

    function handleStatement(response) {
        const pending = root.pendingStatement;
        if (!root.matches(pending, response) || response.path !== pending.path
                || root.connectionFor(response.path) !== pending.name) return;
        root.pendingStatement = null;
        if (typeof response.statement !== "string" || response.statement.trim() === "") return;
        root.dataSourceController.runOn(pending.name, response.statement, false, 0, null, pending.preview);
        root.resultsRequested();
    }

    function fail(method, message, operation) {
        const pending = method === "datasource.console" ? root.pendingOpen : root.pendingStatement;
        if (!root.current(pending) || !operation || pending.clientContext !== operation.clientContext
                || pending.name !== operation.name) return;
        if (method === "datasource.console") {
            root.pendingOpen = null;
            root.dataSourceController.errorText = message;
        } else {
            root.pendingStatement = null;
            root.dataSourceController.queryStatus = message;
            root.resultsRequested();
        }
    }

    // Clique duplo numa tabela da arvore: as primeiras linhas dela.
    function tableData(connection, engine, schema, table, readSql) {
        if (!readSql) root.dataSourceController.queryStatus = qsTr("Leia a estrutura de novo para obter a instrução deste objeto.");
        else root.dataSourceController.runOn(connection, readSql, false, 200);
        root.resultsRequested();
    }
}
