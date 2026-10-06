import QtQuick

// O que responde nesta maquina, e um banco onde nao havia (0.124.0, pedido
// do autor no teste da Etapa 2). Filho do DataSourceController — dono
// proprio, como o `identity` do EmbeddedController: os pedidos saem daqui
// por sinal e as respostas voltam para ca'. NAO decide nada: quem sonda a
// porta, lista o container e le o cabecalho do SQLite e' o core.
Item {
    id: root

    property var dataSourceController: null
    property string workspaceRoot: ""
    property int generation: 0
    property int serial: 0
    property var pendingDestroy: null
    onWorkspaceRootChanged: { root.generation += 1; root.pendingDestroy = null; root.destroying = false; }
    readonly property Connections profileChanges: Connections {
        target: root.dataSourceController
        function onProfilesChanged() { root.validateDestroy(); }
        function onDraftChanged() { root.validateDestroy(); }
    }

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
    // Remover o perfil e, com `data`, o que ele aponta (0.129.0).
    signal destroyRequested(string name, bool data, var context, var confirmation)
    // O catalogo depois de uma remocao imediata ou do job — o pai relista.
    signal profilesChanged(var profiles)
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

    // A remocao em curso: o comando do job e o desfecho.
    property bool destroying: false
    property string destroyMessage: ""
    property bool destroyOk: false
    property string destroyNote: ""

    // `destroyProfile`, nao `destroy`: todo objeto QML ja' tem um `destroy()`.
    function destroyProfile(name, data, confirmation) {
        const profile = root.dataSourceController ? root.dataSourceController.profileByName(name) : null;
        if (profile === null) { root.destroyMessage = qsTr("A conexão já foi removida."); return; }
        root.serial += 1;
        const context = { clientContext: "destroy." + String(root.generation) + ":" + String(root.serial),
            expectedContext: { workspace: root.workspaceRoot, profile: Object.assign({}, profile) } };
        root.pendingDestroy = Object.assign({ name: name, data: data === true, confirmation: confirmation || ({}) }, context);
        destroying = true;
        destroyOk = false;
        destroyMessage = "";
        destroyNote = "";
        destroyRequested(name, data === true, context, confirmation || ({}));
    }

    function destroyCurrent(token) {
        return root.pendingDestroy !== null && token === root.pendingDestroy.clientContext
            && root.pendingDestroy.expectedContext.workspace === root.workspaceRoot
            && root.dataSourceController.queries.profileKey(root.pendingDestroy.expectedContext.profile)
                === root.dataSourceController.queries.profileKey(root.dataSourceController.profileByName(root.pendingDestroy.name));
    }

    function validateDestroy() {
        if (root.pendingDestroy === null) return;
        const draft = root.dataSourceController.draft;
        if (root.destroyCurrent(root.pendingDestroy.clientContext)
                && (root.dataSourceController.selectedName !== root.pendingDestroy.name && draft.name !== root.pendingDestroy.name
                    || root.dataSourceController.queries.profileKey(draft)
                    === root.dataSourceController.queries.profileKey(root.pendingDestroy.expectedContext.profile))) return;
        root.pendingDestroy = null;
        root.destroying = false;
        root.destroyMessage = "";
        root.destroyOk = false;
        root.destroyNote = "";
    }

    function handleDestroyResolved(profiles, immediate, jobId, command, note, token) {
        if (!root.destroyCurrent(token)) return;
        destroyNote = note === undefined ? "" : note;
        if (immediate) {
            root.pendingDestroy = null;
            destroying = false;
            destroyOk = true;
            destroyMessage = qsTr("removido");
            profilesChanged(profiles);
            discover();
        } else if (jobId === "") {
            root.pendingDestroy = null;
            destroying = false;
        } else {
            destroyMessage = qsTr("rodando: %1").arg(command);
        }
    }

    function handleDestroyed(success, message, profiles, token) {
        if (!root.destroyCurrent(token)) return;
        root.pendingDestroy = null;
        destroying = false;
        destroyOk = success;
        destroyMessage = message;
        if (success) {
            if (profiles !== undefined && profiles !== null) profilesChanged(profiles);
            discover();
        }
    }

    function handleFailed(method, message, code, operation) {
        if (method === "datasource.discover") {
            discovering = false;
            hint = message;
        } else if (method === "datasource.create") {
            creating = false;
            createOk = false;
            createMessage = message;
        } else if (method === "datasource.destroy") {
            if (!operation || !root.destroyCurrent(operation.clientContext)) return;
            if (code === "SECRET_REQUIRED") root.dataSourceController.secrets.request("destroy", Object.assign({}, root.pendingDestroy));
            destroying = false;
            destroyOk = false;
            destroyMessage = message;
        }
    }
}
