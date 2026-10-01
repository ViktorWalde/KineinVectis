import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property var clipboard: null
    property real hostWidth: 0
    property real hostHeight: 0
    property alias entriesModel: treeModel
    property alias selectedPath: selection.selectedPath
    property alias selectedKind: selection.selectedKind
    property alias selectedPaths: selection.selectedPaths
    property alias fileClipboard: fileClipboard
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
    property string entryDeleteMethod: ""
    readonly property bool entryDeletePending: entryDeleteMethod !== ""

    signal listDirRequested(string path)
    signal createFileRequested(string path)
    signal createDirectoryRequested(string path)
    signal readFileRequested(string path)
    signal renamePathRequested(string from, string to)
    signal copyPathRequested(string from, string to)
    signal transferBatchRequested(string operation, var items)
    signal deletePathRequested(string path)
    signal trashPathRequested(string path)
    signal runScriptRequested(string path)
    signal debugScriptRequested(string path)
    signal tabsRenameRequested(string from, string to)
    signal tabsCloseRequested(string path)
    signal createDialogFocusRequested()
    signal entryRenameDialogOpenRequested(string name)
    signal focusTreeRequested()

    visible: false

    ListModel {
        id: treeModel
    }

    ProjectTreeSelection {
        id: selection
        model: treeModel
    }

    ProjectFileClipboard {
        id: fileClipboard
        tree: root
        clipboard: root.clipboard
        onCopyPathRequested: function(from, to) { root.copyPathRequested(from, to); }
        onMovePathRequested: function(from, to) { root.renamePathRequested(from, to); }
        onTransferBatchRequested: function(operation, items) {
            root.transferBatchRequested(operation, items);
        }
    }

    function baseName(path) {
        return path.substring(path.lastIndexOf("/") + 1);
    }

    function parentDir(path) {
        const slash = path.lastIndexOf("/");
        return slash > 0 ? path.substring(0, slash) : path;
    }

    function entryMenuDirectory() {
        return entryMenuKind === "directory" ? entryMenuPath : parentDir(entryMenuPath);
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
        selection.clear();
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
        entryDeleteMethod = "";
        fileClipboard.clear();
    }

    function selectEntry(path, kind, modifiers = Qt.NoModifier) {
        selection.select(path, kind, modifiers);
    }

    function selectAllEntries() {
        selection.selectAll();
    }

    function rowIndexForPath(path) {
        return selection.indexOf(path);
    }

    function collapseRow(index) {
        const depth = treeModel.get(index).depth;
        while (index + 1 < treeModel.count
               && treeModel.get(index + 1).depth > depth) {
            treeModel.remove(index + 1);
        }
        treeModel.setProperty(index, "expanded", false);
    }

    // Regras puras da arvore (o que e' da maquina; a ordem): ProjectTreeRules.
    ProjectTreeRules {
        id: rules
    }

    function insertEntries(startIndex, entries, depth, parentPath) {
        const ordered = rules.orderEntries(entries);
        for (let i = 0; i < ordered.length; i++) {
            treeModel.insert(startIndex + i, {
                path: parentPath + "/" + ordered[i].name,
                name: ordered[i].name,
                kind: ordered[i].kind,
                depth: depth,
                expanded: false,
                machine: depth === 0 && rules.isMachineEntry(ordered[i].name)
            });
        }
    }

    function setDirectoryListing(path, entries) {
        if (path === workspaceRoot) {
            treeModel.clear();
            insertEntries(0, entries, 0, path);
            selection.reconcile();
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
        selection.reconcile();
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
            selection.reconcile();
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
        if (workspaceRoot === "" || selectedPaths.length > 1) {
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
        const menuWidth = 244;
        entryMenuX = Math.max(0, Math.min(sceneX, hostWidth - menuWidth));
        entryMenuY = sceneY;
        entryMenuVisible = true;
    }

    function dismissEntryMenu() {
        entryMenuVisible = false;
        focusTreeRequested();
    }

    function runScript(path) {
        if (!isRunnableScript(path, "file") || workspaceRoot === "") {
            return;
        }
        entryMenuVisible = false;
        runScriptRequested(path);
        focusTreeRequested();
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
        focusTreeRequested();
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
            focusTreeRequested();
            return;
        }
        renamePathRequested(entryRenamePath, parentDir(entryRenamePath) + "/" + trimmed);
    }

    function openEntryDelete() {
        if (entryMenuPath === "" || workspaceRoot === "" || entryDeletePending) {
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
        if (entryDeletePending) return;
        entryDeleteError = "";
        deletePathRequested(entryDeletePath);
    }
    function confirmEntryTrash() {
        if (entryDeletePending) return;
        entryDeleteError = "";
        trashPathRequested(entryDeletePath);
    }

    function clearSelection() { selection.clear(); }
    function handleFileCreated(path) { results.fileCreated(path); }
    function handleDirectoryCreated(path) { results.directoryCreated(path); }
    function handlePathRenamed(from, to) {
        if (fileClipboard.pending && fileClipboard.cut
                && from === fileClipboard.source && to === fileClipboard.destination) {
            const kind = fileClipboard.sourceKind;
            fileClipboard.moved(from, to);
            results.pathRenamed(from, to, kind);
        } else {
            results.pathRenamed(from, to, entryRenameKind);
        }
    }
    function handlePathCopied(from, to) { fileClipboard.copied(from, to); }
    function handleCopyFailed(to, message) {
        fileClipboard.requestFailed("fs.copy", message, to);
    }
    function handlePathDeleted(path) { results.pathDeleted(path); }
    function handleExternalChanges(changes) { results.externalChanges(changes); }
    function handleRequestFailed(method, message) {
        if (!fileClipboard.requestFailed(method, message)) results.requestFailed(method, message);
    }

    ProjectTreeResults {
        id: results
        tree: root
    }
}
