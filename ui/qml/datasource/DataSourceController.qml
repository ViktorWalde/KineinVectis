pragma ComponentBehavior: Bound
import QtQuick

// Estado das FONTES DE DADOS (etapa 26 do roadmaps/35).
//
// Guarda o que o core respondeu e o que o autor esta' editando. NAO decide
// nada: quem valida perfil, quem sabe de onde vem a senha e quem conversa com
// o servidor e' o core.
//
// A SENHA VIVE AQUI E SO' AQUI, em `sessionPassword`, e some sozinha:
//   - ao trocar de perfil selecionado;
//   - ao fechar o painel;
//   - ao trocar de workspace.
// Ela nunca vai para o perfil (que e' o que o core persiste) e nunca aparece
// no log do cliente, que redige por nome de campo. Ver `DocsPublic/seguranca/40`.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
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
    // O core disse que PEDIR A SENHA resolve. A UI abre o campo por este
    // campo, nunca lendo `testMessage` — a mensagem do servidor e' localizada.
    property bool secretRequired: false

    // A estrutura lida do banco. Vazia ate' o autor pedir: ler catalogo custa
    // tres consultas pela rede, e fazer isso sozinho ao abrir o painel seria
    // gastar a conexao de quem so' queria conferir a porta.
    property var schemas: []
    // A SEGUNDA FORMA, para os motores sem esquema fixo. Nunca preenchida ao
    // mesmo tempo que `schemas`: o core manda uma OU outra, e a tela escolhe a
    // visao pelo motor do perfil.
    property var collections: []
    property bool reading: false

    // Senha da sessao. Nunca persistida, nunca enviada ao `save`.
    property string sessionPassword: ""

    // A consulta (0.121.0): o texto, o resultado como tabela de texto, e o
    // pedido de confirmacao quando a instrucao escreve — pelo CODIGO
    // `WRITE_CONFIRMATION_REQUIRED`, nunca lendo a mensagem.
    property string sql: ""
    property bool querying: false
    property bool writeConfirmationRequired: false
    property var queryColumns: []
    property var queryRows: []
    property string queryStatus: ""

    signal listRequested()
    signal saveRequested(var profile)
    signal removeRequested(string name)
    signal testRequested(string name, string password)
    signal introspectRequested(string name, string password)
    signal queryRequested(string name, string password, string sql, bool confirmWrite)

    visible: false

    onWorkspaceRootChanged: {
        profiles = [];
        selectedName = "";
        draft = emptyDraft();
        clearSecret();
        clearVerdict();
        errorText = "";
        if (workspaceRoot !== "") {
            listRequested();
        }
    }

    // O padrao e' o caso comum do autor, nao o mais defensivo da IDE: um
    // PostgreSQL local por socket unix com `peer` conecta sem senha nenhuma
    // (DocsPublic/seguranca/40 §7). Por isso host de socket e `automatic`.
    function emptyDraft() {
        return {
            engine: "postgres",
            name: "",
            host: "/var/run/postgresql",
            port: 5432,
            database: "postgres",
            user: "",
            secretSource: "automatic",
            secretVariable: "",
            tls: "disable",
            caFile: ""
        };
    }

    function open() {
        panelVisible = true;
        if (profiles.length === 0) {
            listRequested();
        }
    }

    function close() {
        panelVisible = false;
        clearSecret();
    }

    function clearSecret() {
        sessionPassword = "";
    }

    function clearVerdict() {
        clearQuery();
        schemas = [];
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

    // Copia rasa com os campos que o protocolo aceita. Copiar campo a campo
    // (e nao o objeto inteiro) garante que nada que o core mande a mais entre
    // no `save` — o perfil tem `deny_unknown_fields`, e um campo extra seria
    // recusado.
    function cloneProfile(source) {
        return {
            engine: source.engine || "postgres",
            name: source.name,
            host: source.host,
            port: source.port,
            database: source.database,
            user: source.user,
            secretSource: source.secretSource || "automatic",
            secretVariable: source.secretVariable || "",
            tls: source.tls || "disable",
            caFile: source.caFile || ""
        };
    }

    function editDraft(field, value) {
        const atualizado = cloneProfile(draft);
        atualizado[field] = value;
        draft = atualizado;
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
        clearVerdict();
        testing = true;
        testRequested(draft.name, sessionPassword);
    }

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

    function introspect() {
        if (draft.name === "") {
            return;
        }
        schemas = [];
        collections = [];
        reading = true;
        testMessage = "";
        introspectRequested(draft.name, sessionPassword);
    }

    function handleIntrospected(name, ok, newSchemas, newCollections, message, needsSecret) {
        reading = false;
        testedName = name;
        if (ok) {
            schemas = newSchemas;
            collections = newCollections;
            testMessage = "";
            secretRequired = false;
        } else {
            schemas = [];
            collections = [];
            testMessage = message;
            secretRequired = needsSecret;
        }
    }

    // `true` quando o perfil em edicao fala de DOCUMENTO, e nao de tabela.
    // Um so' dono desta derivacao: o painel e o host leem daqui.
    readonly property bool documentEngine:
        root.draft ? root.draft.engine === "mongo" : false

    function handleTested(name, ok, version, message, needsSecret) {
        testing = false;
        testedName = name;
        testOk = ok;
        serverVersion = version;
        testMessage = message;
        secretRequired = needsSecret;
    }

    function clearQuery() {
        querying = false;
        writeConfirmationRequired = false;
        queryColumns = [];
        queryRows = [];
        queryStatus = "";
    }

    // Executar o que esta' no editor. `confirmWrite` so' vai `true` quando o
    // autor respondeu ao pedido de confirmacao — o core recusa o resto.
    function runQuery(confirmWrite) {
        if (draft.name === "" || sql.trim() === "") {
            return;
        }
        querying = true;
        writeConfirmationRequired = false;
        queryStatus = "";
        queryRequested(draft.name, sessionPassword, sql, confirmWrite === true);
    }

    function handleQueried(outcome) {
        querying = false;
        if (outcome.success === true) {
            queryColumns = outcome.columns || [];
            queryRows = outcome.rows || [];
            secretRequired = false;
            if (outcome.affected !== undefined && outcome.affected !== null) {
                queryStatus = qsTr("%1 linha(s) afetada(s) em %2 ms").arg(outcome.affected).arg(outcome.elapsedMs);
            } else {
                queryStatus = qsTr("%1 linha(s)%2 em %3 ms").arg(outcome.rowCount)
                    .arg(outcome.truncated ? qsTr(" — teto atingido") : "").arg(outcome.elapsedMs);
            }
        } else {
            queryColumns = [];
            queryRows = [];
            queryStatus = outcome.message || qsTr("a consulta falhou");
            secretRequired = outcome.secretRequired === true;
        }
    }

    function handleFailed(method, message, code) {
        if (method.indexOf("datasource.") === 0) {
            testing = false;
            reading = false;
            querying = false;
            if (code === "WRITE_CONFIRMATION_REQUIRED") {
                writeConfirmationRequired = true;
                queryStatus = message;
            } else if (code === "SECRET_REQUIRED" && method === "datasource.query") {
                secretRequired = true;
                queryStatus = message;
            } else {
                errorText = message;
            }
        }
    }
}
