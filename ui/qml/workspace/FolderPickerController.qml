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
    property string pendingSelectionPath: ""
    property string pathDraft: ""
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
        pendingSelectionPath = "";
        pathDraft = path;
        browsePath(path);
        pathFocusRequested();
    }

    function close() {
        loading = false;
        errorText = "";
        createMode = "";
        createName = "";
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

    function setListing(path, parent, entries) {
        loading = false;
        currentPath = path;
        parentPath = parent;
        selectedPath = pendingSelectionPath !== "" ? pendingSelectionPath : path;
        pendingSelectionPath = "";
        errorText = "";
        pathDraft = path;

        entryModel.clear();
        for (let i = 0; i < entries.length; i++) {
            entryModel.append({
                name: entries[i].name,
                path: entries[i].path
            });
        }
    }

    function showError(message) {
        loading = false;
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
        createMode = "folder";
        errorText = "";
        createName = "";
        createFocusRequested();
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
        errorText = "";
        createName = "";
        createFocusRequested();
    }

    function cancelCreate() {
        createMode = "";
        errorText = "";
        createName = "";
    }

    function submitCreate() {
        const name = createName.trim();
        if (name === "") {
            errorText = qsTr("Informe um nome.");
            return;
        }
        if (createMode === "folder") {
            createFolderRequested(currentPath, name);
        } else if (createMode === "project") {
            if (createTemplate === "") {
                errorText = qsTr("Escolha a linguagem do projeto.");
                return;
            }
            createProjectRequested(currentPath, name, createTemplate);
        }
    }

    function selectAfterRefresh(path) {
        pendingSelectionPath = path;
    }
}
