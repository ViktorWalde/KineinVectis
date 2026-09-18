import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property real hostWidth: 0
    property real hostHeight: 0
    property alias entriesModel: treeModel
    property string selectedPath: ""
    property string selectedKind: ""
    property bool createDialogVisible: false
    property string createDialogKind: "file"
    property string createDialogParentPath: ""
    property string createDialogError: ""
    property bool entryMenuVisible: false
    property real entryMenuX: 0
    property real entryMenuY: 0
    property string entryMenuPath: ""
    property string entryMenuKind: ""
    property string entryMenuName: ""
    property bool entryMenuRunnable: false
    property bool entryMenuDebuggable: false
    property bool entryRenameVisible: false
    property string entryRenamePath: ""
    property string entryRenameKind: ""
    property string entryRenameError: ""
    property bool entryDeleteVisible: false
    property string entryDeletePath: ""
    property string entryDeleteKind: ""
    property string entryDeleteName: ""
    property string entryDeleteError: ""

    signal listDirRequested(string path)
    signal createFileRequested(string path)
    signal createDirectoryRequested(string path)
    signal readFileRequested(string path)
    signal renamePathRequested(string from, string to)
    signal deletePathRequested(string path)
    signal runScriptRequested(string path)
    signal debugScriptRequested(string path)
    signal tabsRenameRequested(string from, string to)
    signal tabsCloseRequested(string path)
    signal createDialogFocusRequested()
    signal entryRenameDialogOpenRequested(string name)
    signal focusEditorRequested()

    visible: false

    ListModel {
        id: treeModel
    }

    function baseName(path) {
        return path.substring(path.lastIndexOf("/") + 1);
    }

    function parentDir(path) {
        const slash = path.lastIndexOf("/");
        return slash > 0 ? path.substring(0, slash) : path;
    }

    // O que "Executar" e "Depurar" aceitam vem do CORE (`run.capabilities`,
    // 0.107.0): antes do catalogo chegar, NADA e' executavel — uma lista
    // escrita a mao aqui divergiu da do core por construcao (era o defeito do
    // format.capabilities em 0.60). O ProjectExplorer le `runnableExtensions`
    // para o icone da linha; a decisao mora aqui.
    property var runnableExtensions: []
    property var debuggableExtensions: []

    function applyRunCapabilities(runnable, debuggable) {
        runnableExtensions = runnable === undefined || runnable === null ? [] : runnable;
        debuggableExtensions = debuggable === undefined || debuggable === null ? [] : debuggable;
    }

    function extensionOf(path) {
        const nome = String(path);
        const ponto = nome.lastIndexOf(".");
        return ponto < 0 ? "" : nome.substring(ponto + 1).toLowerCase();
    }

    function isRunnableScript(path, kind) {
        if (kind !== "file") {
            return false;
        }
        return runnableExtensions.indexOf(extensionOf(path)) >= 0;
    }

    function isDebuggableScript(path, kind) {
        return isRunnableScript(path, kind) && debuggableExtensions.indexOf(extensionOf(path)) >= 0;
    }

    function clear() {
        treeModel.clear();
        selectedPath = "";
        selectedKind = "";
        createDialogVisible = false;
        createDialogError = "";
        createDialogParentPath = "";
        entryMenuVisible = false;
        entryMenuPath = "";
        entryMenuRunnable = false;
        entryRenameVisible = false;
        entryRenameError = "";
        entryDeleteVisible = false;
        entryDeleteError = "";
    }

    function selectEntry(path, kind) {
        selectedPath = path;
        selectedKind = kind;
    }

    function rowIndexForPath(path) {
        for (let i = 0; i < treeModel.count; i++) {
            if (treeModel.get(i).path === path) {
                return i;
            }
        }
        return -1;
    }

    function collapseRow(index) {
        const depth = treeModel.get(index).depth;
        while (index + 1 < treeModel.count
               && treeModel.get(index + 1).depth > depth) {
            treeModel.remove(index + 1);
        }
        treeModel.setProperty(index, "expanded", false);
    }

    function insertEntries(startIndex, entries, depth, parentPath) {
        for (let i = 0; i < entries.length; i++) {
            treeModel.insert(startIndex + i, {
                path: parentPath + "/" + entries[i].name,
                name: entries[i].name,
                kind: entries[i].kind,
                depth: depth,
                expanded: false
            });
        }
    }

    function setDirectoryListing(path, entries) {
        if (path === workspaceRoot) {
            treeModel.clear();
            insertEntries(0, entries, 0, path);
            advanceReveal();
            return;
        }
        const index = rowIndexForPath(path);
        if (index < 0) {
            return;
        }
        if (treeModel.get(index).expanded) {
            collapseRow(index);
        }
        treeModel.setProperty(index, "expanded", true);
        insertEntries(index + 1, entries, treeModel.get(index).depth + 1, path);
        listingArrived(path);
        advanceReveal();
    }

    // O explorer segue o arquivo ativo (F3): dono proprio, abaixo.
    function revealPath(path) {
        reveal.revealPath(path);
    }

    function advanceReveal() {
        reveal.advance();
    }

    function listingArrived(path) {
        reveal.listed(path);
    }

    ProjectTreeRevealController {
        id: reveal

        tree: root
    }

    function toggleDirectory(path, index, expanded) {
        if (expanded) {
            collapseRow(index);
        } else {
            root.listDirRequested(path);
        }
    }

    function selectedCreateParent() {
        if (selectedPath === "") {
            return workspaceRoot;
        }
        return selectedKind === "directory" ? selectedPath : parentDir(selectedPath);
    }

    function openCreateDialog(kind) {
        if (workspaceRoot === "") {
            return;
        }
        openCreateDialogAt(kind, selectedCreateParent());
    }

    function openCreateDialogAt(kind, parentPath) {
        createDialogKind = kind;
        createDialogParentPath = parentPath;
        createDialogError = "";
        createDialogVisible = true;
        createDialogFocusRequested();
    }

    function openEntryCreate(kind) {
        if (entryMenuPath === "" || workspaceRoot === "") {
            return;
        }
        entryMenuVisible = false;
        const parentPath = entryMenuKind === "directory"
                ? entryMenuPath : parentDir(entryMenuPath);
        openCreateDialogAt(kind, parentPath);
    }

    function confirmCreateEntry(name) {
        const trimmed = name.trim();
        if (trimmed === "") {
            createDialogError = qsTr("Informe um nome.");
            return;
        }
        if (trimmed.indexOf("/") >= 0) {
            createDialogError = qsTr("Use apenas um nome, sem barras.");
            return;
        }
        const path = createDialogParentPath + "/" + trimmed;
        if (createDialogKind === "directory") {
            createDirectoryRequested(path);
        } else {
            createFileRequested(path);
        }
    }

    function openEntryMenu(path, kind, name, sceneX, sceneY) {
        entryMenuPath = path;
        entryMenuKind = kind;
        entryMenuName = name;
        entryMenuRunnable = isRunnableScript(path, kind);
        entryMenuDebuggable = isDebuggableScript(path, kind);
        entryMenuX = Math.max(0, Math.min(sceneX, hostWidth - 172));
        entryMenuY = Math.max(0, Math.min(sceneY, hostHeight - 190));
        entryMenuVisible = true;
    }

    function runScript(path) {
        if (!isRunnableScript(path, "file") || workspaceRoot === "") {
            return;
        }
        entryMenuVisible = false;
        runScriptRequested(path);
    }

    function runEntryScript() {
        if (!entryMenuRunnable) {
            return;
        }
        runScript(entryMenuPath);
    }

    function debugEntryScript() {
        if (!entryMenuDebuggable || workspaceRoot === "") {
            return;
        }
        entryMenuVisible = false;
        debugScriptRequested(entryMenuPath);
    }

    function openEntryRename() {
        if (entryMenuPath === "" || workspaceRoot === "") {
            return;
        }
        entryMenuVisible = false;
        entryRenamePath = entryMenuPath;
        entryRenameKind = entryMenuKind;
        entryRenameError = "";
        entryRenameVisible = true;
        entryRenameDialogOpenRequested(entryMenuName);
    }

    function confirmEntryRename(name) {
        const trimmed = name.trim();
        if (trimmed === "") {
            entryRenameError = qsTr("Informe um nome.");
            return;
        }
        if (trimmed.indexOf("/") >= 0) {
            entryRenameError = qsTr("Use apenas um nome, sem barras.");
            return;
        }
        if (trimmed === baseName(entryRenamePath)) {
            entryRenameVisible = false;
            focusEditorRequested();
            return;
        }
        renamePathRequested(entryRenamePath, parentDir(entryRenamePath) + "/" + trimmed);
    }

    function openEntryDelete() {
        if (entryMenuPath === "" || workspaceRoot === "") {
            return;
        }
        entryMenuVisible = false;
        entryDeletePath = entryMenuPath;
        entryDeleteKind = entryMenuKind;
        entryDeleteName = entryMenuName;
        entryDeleteError = "";
        entryDeleteVisible = true;
    }

    function confirmEntryDelete() {
        entryDeleteError = "";
        deletePathRequested(entryDeletePath);
    }

    function handleFileCreated(path) {
        createDialogVisible = false;
        createDialogError = "";
        selectedPath = path;
        selectedKind = "file";
        listDirRequested(parentDir(path));
        readFileRequested(path);
    }

    function handleDirectoryCreated(path) {
        createDialogVisible = false;
        createDialogError = "";
        selectedPath = path;
        selectedKind = "directory";
        listDirRequested(parentDir(path));
    }

    function handlePathRenamed(from, to) {
        entryRenameVisible = false;
        entryRenameError = "";
        tabsRenameRequested(from, to);
        selectedPath = to;
        listDirRequested(parentDir(to));
        focusEditorRequested();
    }

    function handlePathDeleted(path) {
        entryDeleteVisible = false;
        entryDeleteError = "";
        tabsCloseRequested(path);
        if (selectedPath === path || selectedPath.indexOf(path + "/") === 0) {
            selectedPath = "";
            selectedKind = "";
        }
        listDirRequested(parentDir(path));
        focusEditorRequested();
    }

    function handleExternalChanges(changes) {
        const directories = {};
        for (let i = 0; i < changes.length; i++) {
            const directory = parentDir(changes[i].path);
            if (directory === workspaceRoot) {
                directories[directory] = true;
                continue;
            }
            const index = rowIndexForPath(directory);
            if (index >= 0 && treeModel.get(index).kind === "directory"
                    && treeModel.get(index).expanded) {
                directories[directory] = true;
            }
        }
        for (const directory in directories) {
            listDirRequested(directory);
        }
    }

    function handleRequestFailed(method, message) {
        if (method === "fs.createFile" || method === "fs.createDirectory") {
            createDialogError = message;
            createDialogVisible = true;
        }
        if (method === "fs.rename") {
            entryRenameError = message;
            entryRenameVisible = true;
        }
        if (method === "fs.delete") {
            entryDeleteError = message;
            entryDeleteVisible = true;
        }
    }
}
