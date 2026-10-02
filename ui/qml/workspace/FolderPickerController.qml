import QtQuick

Item {
    id: root

    property string currentPath: ""
    property string parentPath: ""
    property string selectedPath: ""
    property string homePath: "/"
    property string errorText: ""
    property bool loading: false
    property string createMode: ""
    // "Criar Projeto" escolhe a LINGUAGEM e depois o ECOSSISTEMA (53 §13.0
    // item 6). Os dois comecam vazios: a tela inicial nao escolhe por quem
    // vai criar. O catalogo e' o dono do que existe.
    property string createLanguage: ""
    property string createTemplate: ""
    readonly property ProjectTemplateCatalog templateCatalog: ProjectTemplateCatalog {}
    // No "Criar projeto" o navegador de pastas fica recolhido; "Alterar
    // local..." o abre (0.3.8).
    property bool createBrowsing: false
    // O dono de "o que estou criando": o cartao e o painel so' perguntam.
    readonly property bool creatingProject: createMode === "project"
    // "Nova pasta" e' uma linha do navegador, independente do projeto: criar
    // a pasta do local nao desfaz o projeto que se esta' montando.
    property bool creatingFolder: false
    property string folderName: ""
    readonly property bool canCreateProject: createTemplate !== "" && createName.trim() !== ""
    // O navegador amplo (0.3.9, retorno do autor: "encolhido"; modelo do
    // seletor da JetBrains). Locais vem do core (0.147.0); ocultas ficam
    // fora ate' pedir; voltar/avancar e' o historico desta abertura.
    property var places: []
    property bool showHidden: false
    property int hiddenCount: 0
    property var backStack: []
    property var forwardStack: []
    property string historyMove: ""
    // O caminho como migalhas: "/" e cada pasta ate' a atual, clicaveis.
    readonly property var crumbs: {
        const result = [{ label: "/", path: "/" }];
        const parts = currentPath.split("/").filter(part => part !== "");
        let path = "";
        for (const part of parts) {
            path += "/" + part;
            result.push({ label: part, path: path });
        }
        return result;
    }
    property bool editingPath: false
    property var listing: []
    property string pendingSelectionPath: ""
    property string createName: ""
    property alias entriesModel: entryModel
    // Para que a pasta escolhida serve (2026-09-17): "workspace" abre o
    // projeto (o de sempre); qualquer outro proposito — "kitPath", o
    // SDK/sysroot do painel de Embarcados — devolve o caminho a quem pediu
    // por `folderPicked`, sem abrir nada. Um picker, dois usos: o mesmo
    // navegador de pastas do core (fs.browse), nem um segundo dialogo.
    property string purpose: "workspace"

    signal browseRequested(string path)
    signal openRequested(string path)
    signal folderPicked(string purpose, string path)
    signal createFolderRequested(string parent, string name)
    signal createProjectRequested(string parent, string name, string templateId)
    signal pathFocusRequested()
    signal createFocusRequested()

    visible: false

    ListModel {
        id: entryModel
    }

    function open(startPath) {
        openFor("workspace", startPath);
    }

    function openFor(newPurpose, startPath) {
        purpose = newPurpose === undefined || newPurpose === "" ? "workspace" : newPurpose;
        const path = startPath !== "" ? startPath : "/";
        errorText = "";
        selectedPath = path;
        createMode = "";
        creatingFolder = false;
        pendingSelectionPath = "";
        backStack = [];
        forwardStack = [];
        // A primeira listagem desta abertura nao empilha a pasta da anterior.
        historyMove = "open";
        editingPath = false;
        browsePath(path);
        pathFocusRequested();
    }

    function close() {
        loading = false;
        errorText = "";
        createMode = "";
        createName = "";
        creatingFolder = false;
    }

    function browsePath(path) {
        const cleanPath = path.trim();
        if (cleanPath === "") {
            return;
        }
        loading = true;
        errorText = "";
        browseRequested(cleanPath);
    }

    function setListing(path, parent, entries, newPlaces) {
        loading = false;
        recordHistory(path);
        currentPath = path;
        parentPath = parent;
        selectedPath = pendingSelectionPath !== "" ? pendingSelectionPath : path;
        pendingSelectionPath = "";
        errorText = "";
        editingPath = false;
        // Listas do C++ chegam como QVariantList (nao e' Array do JS): so'
        // `length` e indice, nada de for..of nem filter.
        if (newPlaces !== undefined && newPlaces !== null && newPlaces.length > 0) {
            places = newPlaces;
        }
        listing = entries;
        rebuildEntries();
    }

    // O historico so' muda quando a listagem chega: um caminho que falhou
    // nao entra nem tira nada das pilhas.
    function recordHistory(path) {
        const previous = currentPath;
        if (historyMove === "back") {
            backStack = backStack.slice(0, -1);
            forwardStack = forwardStack.concat([previous]);
        } else if (historyMove === "forward") {
            forwardStack = forwardStack.slice(0, -1);
            backStack = backStack.concat([previous]);
        } else if (historyMove === "" && previous !== "" && previous !== path) {
            backStack = backStack.concat([previous]);
            forwardStack = [];
        }
        historyMove = "";
    }

    function goBack() {
        if (backStack.length === 0) return;
        historyMove = "back";
        browsePath(backStack[backStack.length - 1]);
    }

    function goForward() {
        if (forwardStack.length === 0) return;
        historyMove = "forward";
        browsePath(forwardStack[forwardStack.length - 1]);
    }

    function isHidden(name) {
        return name.startsWith(".");
    }

    function rebuildEntries() {
        entryModel.clear();
        let hidden = 0;
        for (let i = 0; i < listing.length; i++) {
            const entry = listing[i];
            if (isHidden(entry.name)) {
                hidden += 1;
                if (!showHidden) continue;
            }
            entryModel.append({
                name: entry.name,
                path: entry.path,
                kind: entry.kind !== undefined ? entry.kind : ""
            });
        }
        hiddenCount = hidden;
    }

    function setShowHidden(show) {
        showHidden = show;
        rebuildEntries();
    }

    // Teclado na lista: setas andam, Enter entra, Backspace sobe.
    function selectedIndex() {
        for (let i = 0; i < entryModel.count; i++) {
            if (entryModel.get(i).path === selectedPath) return i;
        }
        return -1;
    }

    function moveSelection(delta) {
        if (entryModel.count === 0) return;
        const index = Math.max(0, Math.min(entryModel.count - 1, selectedIndex() + delta));
        selectedPath = entryModel.get(index).path;
    }

    function enterSelected() {
        if (selectedPath !== "" && selectedPath !== currentPath) browsePath(selectedPath);
    }

    // "Usar esta pasta" do local do projeto novo: a selecionada (ou a atual)
    // vira o local e o navegador recolhe.
    function useAsLocation() {
        createBrowsing = false;
        if (selectedPath !== "" && selectedPath !== currentPath) browsePath(selectedPath);
    }

    function showError(message) {
        loading = false;
        historyMove = "";
        errorText = message;
    }

    function openSelected() {
        const path = selectedPath !== "" ? selectedPath : currentPath;
        if (path === "") {
            return;
        }
        if (purpose === "workspace") {
            openRequested(path);
        } else {
            folderPicked(purpose, path);
        }
    }

    function beginCreateFolder() {
        creatingFolder = true;
        folderName = "";
        errorText = "";
    }

    function cancelCreateFolder() {
        creatingFolder = false;
        errorText = "";
    }

    function submitCreateFolder() {
        const name = folderName.trim();
        if (name === "") {
            errorText = qsTr("Informe um nome.");
            return;
        }
        createFolderRequested(currentPath, name);
    }

    // Escolher a linguagem seleciona o primeiro ecossistema dela.
    function chooseLanguage(key) {
        createLanguage = key;
        createTemplate = templateCatalog.defaultTemplate(key);
        errorText = "";
    }

    // O template ja' traz a linguagem (o comando da paleta, ou quem abrir com
    // um template); vazio deixa os dois por escolher.
    function chooseTemplate(templateId) {
        createLanguage = templateCatalog.languageOf(templateId);
        createTemplate = createLanguage !== "" ? templateId : "";
        errorText = "";
    }

    function beginCreateProject() {
        createMode = "project";
        createBrowsing = false;
        errorText = "";
        createName = "";
        createFocusRequested();
    }

    function submitCreate() {
        const name = createName.trim();
        if (name === "") {
            errorText = qsTr("Informe um nome.");
            return;
        }
        if (createTemplate === "") {
            errorText = qsTr("Escolha a linguagem do projeto.");
            return;
        }
        createProjectRequested(currentPath, name, createTemplate);
    }

    // A pasta nova foi criada: a linha fecha e ela vem selecionada.
    function selectAfterRefresh(path) {
        creatingFolder = false;
        pendingSelectionPath = path;
    }
}
