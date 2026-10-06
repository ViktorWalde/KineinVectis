pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Estado de fontes de dados; regras/validacao sao do core. IPC pelos roteadores.
// sessionPassword vive so' em memoria, nunca no perfil/log (seguranca/40).
// Trocar perfil/projeto ou fechar painel limpa o segredo.
Item {
    id: root

    property string workspaceRoot: ""
    property var profiles: []
    property string selectedName: ""
    property bool panelVisible: false
    property string errorText: ""

    // O rascunho do formulario. Comeca no perfil selecionado; um perfil novo
    // comeca no padrao que conecta sem senha num Postgres local.
    property var draft: root.emptyDraft()

    // Veredito do ultimo teste. `testing` some quando o evento chega.
    property bool testing: false
    property string testedName: ""
    property bool testOk: false
    property string serverVersion: ""
    property string testMessage: ""
    // O core disse que PEDIR A SENHA resolve (nunca lendo `testMessage`).
    property bool secretRequired: false

    // A estrutura lida do banco. Vazia ate' o autor pedir: ler o catalogo
    // custa tres consultas pela rede.
    property var schemas: []
    // A SEGUNDA FORMA (motor sem esquema fixo): o core manda esta OU `schemas`.
    property var collections: []
    property bool reading: false
    // A estrutura POR CONEXAO, para a arvore da janela do Banco (2026-10-03):
    // nome -> { schemas, collections }; e quem esta' sendo lido agora.
    property var structures: ({})
    property var readingNames: ({})
    // A ultima consulta pedida por NOME (o console do editor): repetida tal
    // qual quando a escrita pede confirmacao.
    property alias lastQuery: queryController.lastQuery

    // Senha da sessao. Nunca persistida, nunca enviada ao `save`.
    property alias sessionPassword: secretController.value
    readonly property DataSourceSecretController secrets: DataSourceSecretController {
        id: secretController
        dataSourceController: root
        workspaceRoot: root.workspaceRoot
    }

    // A consulta (0.121.0): texto, resultado, e o pedido de confirmacao (so'
    // remocao desde o 0.156.0) pelo CODIGO `WRITE_CONFIRMATION_REQUIRED`.
    property alias querying: queryController.querying
    property alias writeConfirmationRequired: queryController.writeConfirmationRequired
    property alias queryColumns: queryController.columns
    property alias queryRows: queryController.rows
    property alias queryStatus: queryController.status

    signal listRequested()
    signal saveRequested(var profile)
    signal removeRequested(string name)
    signal testRequested(string name, string password, var context)
    signal introspectRequested(string name, string password, var context)
    signal queryRequested(string name, string password, string sql, bool confirmWrite, int maxRows, var context, var confirmation)
    // Menu, paleta e Ctrl+Alt+J pedem a JANELA do Banco (o shell a abre do lado do icone).
    signal windowRequested()

    // Descoberta e criacao (0.124.0): filho com dono proprio; adocao volta aqui.
    readonly property alias discovery: discoveryController
    // O console SQL no editor (2026-10-03) e a confirmacao de escrita com o
    // IMPACTO medido antes de rodar (0.150.0).
    readonly property DataSourceConsoleController consoles: DataSourceConsoleController {
        dataSourceController: root
        workspaceRoot: root.workspaceRoot
    }
    readonly property DataSourceImpactController impact: DataSourceImpactController {
        dataSourceController: root
        onRunConfirmed: (name, text, confirmation) => root.queries.begin(name, text, true, root.lastQuery ? root.lastQuery.maxRows : 0, confirmation, root.pendingDatabase)
    }

    readonly property DataSourceOdbcController odbc: DataSourceOdbcController {
        dataSourceController: root
        workspaceRoot: root.workspaceRoot
    }

    readonly property DataSourceQueryController queries: DataSourceQueryController {
        id: queryController
        dataSourceController: root
        workspaceRoot: root.workspaceRoot
        onRequested: (name, text, confirmed, maxRows, context, confirmation) => root.queryRequested(name, root.passwordFor(name), text, confirmed, maxRows, context, confirmation)
        onDatabaseCreated: profile => {
            root.draft = profile;
            root.selectedName = "";
            root.clearSecret();
            root.saveRequested(profile);
        }
        onSecretNeeded: operation => root.secrets.request("query", operation)
    }

    readonly property DataSourceCatalogController catalog: DataSourceCatalogController {
        dataSourceController: root
        workspaceRoot: root.workspaceRoot
        onTestRequested: (name, context) => root.testRequested(name, root.passwordFor(name), context)
        onIntrospectRequested: (name, context) => root.introspectRequested(name, root.passwordFor(name), context)
    }

    DataSourceDiscoveryController {
        id: discoveryController
        dataSourceController: root
        workspaceRoot: root.workspaceRoot

        onProfileReady: function(profile, saved) { root.adoptProfile(profile, saved); }
        onProfilesChanged: function(profiles) { root.handleList(profiles); root.startNew(); }
    }

    visible: false

    onWorkspaceRootChanged: {
        profiles = [];
        selectedName = "";
        draft = emptyDraft();
        clearSecret();
        clearVerdict();
        structures = ({});
        readingNames = ({});
        lastQuery = null;
        errorText = "";
        if (workspaceRoot !== "") {
            listRequested();
        }
    }

    // O padrao e' o caso comum do autor, nao o mais defensivo da IDE: um
    // PostgreSQL local por socket unix com `peer` conecta sem senha nenhuma
    // (DocsPublic/seguranca/40 §7). Por isso host de socket e `automatic`.
    function emptyDraft() { return DataSourceKinds.emptyProfile(); }

    // A lista do projeto e o que responde nesta maquina — PERGUNTA de novo
    // a cada abertura: um servidor sobe e cai fora da IDE.
    function refreshCatalog() {
        listRequested();
        discoveryController.discover();
        odbc.refresh();
    }

    function open() {
        panelVisible = true;
        refreshCatalog();
    }

    // Um perfil vindo da descoberta (vai ao formulario para o autor
    // confirmar) ou da criacao (ja' salvo: a lista e' relida e ele fica).
    function adoptProfile(profile, saved) {
        draft = cloneProfile(profile);
        selectedName = saved ? profile.name : "";
        clearSecret();
        clearVerdict();
        if (saved) {
            listRequested();
        }
    }

    // Um banco DENTRO do PostgreSQL do perfil em edicao: `CREATE DATABASE`
    // pelo caminho de escrita confirmada que ja' existe; quando o core
    // responder, o perfil clonado com o banco novo e' salvo (handleQueried).
    property alias pendingDatabase: queryController.pendingDatabase

    function createDatabaseOnServer(name) {
        const limpo = name.trim();
        if (draft.name === "" || !DataSourceKinds.isPostgres(draft.engine) || !/^[A-Za-z0-9_]+$/.test(limpo)) {
            errorText = qsTr("um banco novo pede um perfil PostgreSQL salvo e um nome só de letras, dígitos e _");
            return;
        }
        const sql = "CREATE DATABASE \"" + limpo + "\"";
        queries.begin(draft.name, sql, false, 0, null, limpo);
    }

    function close() {
        panelVisible = false;
        clearSecret();
    }

    function clearSecret() {
        secrets.clear();
    }

    function passwordFor(name) { return secrets.forName(name); }

    function retryWithSecret() {
        const operation = secrets.takePending();
        if (operation === null) return;
        secretRequired = false;
        if (operation.method === "query") queries.begin(operation.name, operation.sql, operation.confirmWrite,
            operation.maxRows, operation.confirmation, operation.database);
        else if (operation.method === "destroy") discovery.destroyProfile(operation.name, operation.data, operation.confirmation);
        else catalog.begin(operation.method, operation.name);
    }

    function clearVerdict() {
        clearQuery();
        schemas = [];
        collections = [];
        reading = false;
        testing = false;
        testedName = "";
        testOk = false;
        serverVersion = "";
        testMessage = "";
        secretRequired = false;
    }

    function profileByName(name) {
        return profiles.find(function(item) { return item.name === name; }) || null;
    }

    function select(name) {
        selectedName = name;
        const encontrado = profileByName(name);
        draft = encontrado === null ? emptyDraft() : cloneProfile(encontrado);
        clearSecret();
        clearVerdict();
    }

    function startNew() {
        selectedName = "";
        draft = emptyDraft();
        clearSecret();
        clearVerdict();
    }

    // Copia campo a campo: nada que o core mande a mais entra no `save` (o
    // perfil tem `deny_unknown_fields`, e um campo extra seria recusado).
    function cloneProfile(source) { return DataSourceKinds.cloneProfile(source); }

    function editDraft(field, value) {
        const updated = cloneProfile(draft);
        updated[field] = value;
        if (field === "engine" && value !== draft.engine) {
            DataSourceKinds.adoptDefaults(updated, draft.engine, value);
            clearSecret();
        }
        draft = updated;
    }

    // O core recusa `secretVariable` vazia? Nao — ele a normaliza para
    // ausente. Mandar a chave com "" e' valido e vira `None` la'.
    function save() {
        errorText = "";
        saveRequested(cloneProfile(draft));
    }

    function remove() {
        if (selectedName !== "") {
            removeRequested(selectedName);
        }
    }

    function test() {
        if (draft.name === "") {
            return;
        }
        testProfile(draft.name);
    }

    function testProfile(name) { catalog.begin("test", name); }

    function handleList(newProfiles) {
        profiles = newProfiles;
        errorText = "";
        // Salvar um perfil novo seleciona ele: e' o que o autor acabou de
        // fazer, e deixar a selecao no anterior faria "Remover" apagar a coisa
        // errada.
        if (selectedName === "" && draft.name !== "" && profileByName(draft.name) !== null) {
            selectedName = draft.name;
        }
        if (selectedName !== "" && profileByName(selectedName) === null) {
            startNew();
        }
    }

    function introspect() { catalog.begin("introspect", draft.name); }

    function introspectProfile(name) { catalog.begin("introspect", name); }

    // Executar `text` na conexao `name` (o console no editor).
    function runOn(name, text, confirmWrite, maxRows, confirmation) {
        queries.begin(name, text, confirmWrite, maxRows, confirmation);
    }

    function handleIntrospected(name, ok, schemas, collections, message, needsSecret, token) {
        catalog.introspected(name, ok, schemas, collections, message, needsSecret, token);
    }

    readonly property bool documentEngine: root.draft ? DataSourceKinds.isMongo(root.draft.engine) : false

    function handleTested(name, ok, version, message, needsSecret, token) {
        catalog.tested(name, ok, version, message, needsSecret, token);
    }

    function clearQuery() { queries.invalidate(); }

    function handleQueried(outcome) { queries.handleOutcome(outcome); }

    function handleFailed(method, message, code, operation) {
        if (method === "datasource.query") return queries.fail(message, code, operation);
        if (method === "datasource.test" || method === "datasource.introspect") return catalog.fail(method.substring(11), message, code, operation);
        if (method === "datasource.discover" || method === "datasource.create"
                || method === "datasource.destroy") {
            return;
        }
        if (method.indexOf("datasource.odbc.") === 0) return odbc.handleFailed(method, message);
        if (method === "datasource.impact") return impact.handleFailed(message, operation, code);
        if (method.indexOf("datasource.") === 0) {
            testing = false;
            reading = false;
            readingNames = ({});
            querying = false;
            if (code === "DRIVER_APPROVAL_REQUIRED") return;
            errorText = message;
        }
    }
}
