import QtQuick

// O que responde nesta maquina, e um banco onde nao havia (0.124.0, pedido
// do autor no teste da Etapa 2). Filho do DataSourceController — dono
// proprio, como o `identity` do EmbeddedController: os pedidos saem daqui
// por sinal e as respostas voltam para ca'. NAO decide nada: quem sonda a
// porta, lista o container e le o cabecalho do SQLite e' o core.
Item {
    id: root

    // Cada candidato: { kind, label, detail, running, profile }.
    property var candidates: []
    property bool discovering: false
    property string containerEngine: ""
    property string hint: ""

    // A criacao em curso: o comando que o core rodou (para a tela mostrar) e
    // o desfecho.
    property bool creating: false
    property string createCommand: ""
    property string createMessage: ""
    property bool createOk: false

    signal discoverRequested()
    signal createSqliteRequested(string name, string path)
    signal createServerRequested(string engine, string name, int port)
    // Um perfil pronto (adotado de um candidato ou recem-criado) para o pai
    // por no formulario e/ou selecionar.
    signal profileReady(var profile, bool saved)

    visible: false

    readonly property bool canServe: containerEngine !== ""

    function discover() {
        discovering = true;
        discoverRequested();
    }

    function handleDiscovered(newCandidates, engine, newHint) {
        candidates = newCandidates === undefined || newCandidates === null ? [] : newCandidates;
        containerEngine = engine === undefined ? "" : engine;
        hint = newHint === undefined ? "" : newHint;
        discovering = false;
    }

    // Adotar um candidato: o perfil dele vai para o formulario — nao e' salvo
    // ate' o autor mandar (e' ele quem confirma nome e banco).
    function adopt(index) {
        if (index < 0 || index >= candidates.length) {
            return;
        }
        profileReady(candidates[index].profile, false);
    }

    // A porta padrao de cada motor com servidor, para o formulario de criacao.
    readonly property var defaultPorts: ({ postgres: 5432, mongo: 27017 })

    function defaultPort(engine) {
        return defaultPorts[engine] || 5432;
    }

    function createSqlite(name, path) {
        if (name.trim() === "") {
            createMessage = qsTr("dê um nome ao banco");
            createOk = false;
            return;
        }
        creating = true;
        createOk = false;
        createMessage = "";
        createCommand = "";
        createSqliteRequested(name.trim(), path === undefined ? "" : path);
    }

    function createServer(engine, name, port) {
        if (name.trim() === "") {
            createMessage = qsTr("dê um nome ao banco");
            createOk = false;
            return;
        }
        creating = true;
        createOk = false;
        createMessage = "";
        createCommand = "";
        createServerRequested(engine, name.trim(), Number(port) > 0 ? Number(port) : defaultPort(engine));
    }

    // A resposta do create: SQLite vem pronto; o servidor vem como job e o
    // desfecho chega por handleCreated.
    function handleCreateResolved(profile, jobId, command) {
        createCommand = command === undefined ? "" : command;
        if (profile && profile.name !== undefined && profile.name !== "") {
            creating = false;
            createOk = true;
            createMessage = qsTr("%1 criado").arg(profile.database);
            profileReady(profile, true);
        } else if (jobId === "") {
            creating = false;
        }
    }

    function handleCreated(success, profile, message) {
        creating = false;
        createOk = success;
        createMessage = message;
        if (success && profile && profile.name !== undefined) {
            profileReady(profile, true);
            discover();
        }
    }

    function handleFailed(method, message) {
        if (method === "datasource.discover") {
            discovering = false;
            hint = message;
        } else if (method === "datasource.create") {
            creating = false;
            createOk = false;
            createMessage = message;
        }
    }
}
