import QtQuick

// Estado da colagem no explorador; o clipboard e as operações de disco já
// pertencem, respectivamente, ao singleton Clipboard e ao core.
Item {
    id: root

    ProjectTreeRules {
        id: treeRules
    }

    property var tree: null
    property var clipboard: null
    property bool dialogVisible: false
    property bool pending: false
    property bool batchDialogVisible: false
    property bool batchPending: false
    property bool batchCut: false
    property bool batchImport: false
    property var batchEntries: []
    property var batchSourcePaths: []
    property var batchSubmitted: []
    property var batchSubmittedIndices: []
    property string batchDestinationDirectory: ""
    property string batchErrorText: ""
    property string source: ""
    property string sourceKind: ""
    property string destinationDirectory: ""
    property string destination: ""
    property string errorText: ""
    property bool cut: false
    property bool transferFromClipboard: false
    readonly property bool pasteAvailable: {
        if (clipboard === null) return false;
        const paths = clipboard.filePaths();
        return clipboard.filesAvailable && paths.length > 0 && paths.length <= 128
               && tree !== null && tree.workspaceRoot !== ""
               && tree.selectedPaths.length <= 1
               && paths.every(path => path.startsWith(tree.workspaceRoot + "/"));
    }

    signal copyPathRequested(string from, string to)
    signal movePathRequested(string from, string to)
    signal pasteOpened(string name)
    signal transferBatchRequested(string operation, var items)

    visible: false

    function clear() {
        dialogVisible = false;
        pending = false;
        batchDialogVisible = false;
        batchPending = false;
        batchCut = false;
        batchImport = false;
        batchEntries = [];
        batchSourcePaths = [];
        batchSubmitted = [];
        batchSubmittedIndices = [];
        batchDestinationDirectory = "";
        batchErrorText = "";
        source = "";
        sourceKind = "";
        destinationDirectory = "";
        destination = "";
        errorText = "";
        cut = false;
        transferFromClipboard = false;
    }

    function copySelection(asCut) {
        if (tree === null || clipboard === null || tree.selectedPaths.length === 0
                || tree.selectedPaths.length > 128) return false;
        clipboard.setFiles(tree.selectedPaths, asCut);
        tree.entryMenuVisible = false;
        tree.focusTreeRequested();
        return true;
    }

    function copySelectedPaths(relative) {
        if (tree === null || clipboard === null || tree.workspaceRoot === ""
                || tree.selectedPaths.length === 0
                || !tree.selectedPaths.every(path => path.startsWith(tree.workspaceRoot + "/")))
            return false;
        const paths = tree.selectedPaths.map(path =>
            relative ? path.substring(tree.workspaceRoot.length + 1) : path);
        clipboard.setText(paths.join("\n"));
        tree.entryMenuVisible = false;
        tree.focusTreeRequested();
        return true;
    }

    function openPaste() {
        if (!pasteAvailable || pending || batchPending) return false;
        const paths = clipboard.filePaths();
        const directory = paths.length === 1
                && tree.selectedPath === paths[0] && tree.selectedCreateParent() === paths[0]
                ? tree.parentDir(paths[0]) : tree.selectedCreateParent();
        return openTransfer(paths, clipboard.filesCut(), directory, true);
    }

    function openTransfer(paths, asCut, directory, fromClipboard) {
        if (tree === null || tree.workspaceRoot === "" || pending || batchPending
                || !treeRules.validPaths(paths)
                || (directory !== tree.workspaceRoot
                    && !directory.startsWith(tree.workspaceRoot + "/"))
                || !paths.every(path => path.startsWith(tree.workspaceRoot + "/"))) return false;
        transferFromClipboard = fromClipboard;
        if (paths.length > 1) {
            return openBatch(paths, asCut, directory, false);
        }
        source = paths[0];
        sourceKind = "";
        const row = tree.rowIndexForPath(source);
        if (row >= 0) sourceKind = tree.entriesModel.get(row).kind;
        destinationDirectory = directory;
        cut = asCut;
        destination = "";
        errorText = "";
        dialogVisible = true;
        tree.entryMenuVisible = false;
        pasteOpened(tree.baseName(source));
        return true;
    }

    function openImport(paths, directory) {
        if (tree === null || tree.workspaceRoot === "" || pending || batchPending
                || !treeRules.validPaths(paths)
                || !paths.every(path => path.startsWith("/"))) return false;
        transferFromClipboard = false;
        return openBatch(paths, false, directory, true);
    }

    function openBatch(paths, asCut, directory, importing) {
        if (directory !== tree.workspaceRoot
                && !directory.startsWith(tree.workspaceRoot + "/")) return false;
        batchSourcePaths = paths.slice();
        batchCut = asCut;
        batchImport = importing;
        batchDestinationDirectory = directory;
        batchEntries = paths.map(path => ({
            from: path, name: tree.baseName(path), included: true,
            status: "", error: ""
        }));
        batchErrorText = "";
        batchDialogVisible = true;
        tree.entryMenuVisible = false;
        return true;
    }

    function confirm(name) {
        if (!dialogVisible || pending) return;
        const trimmed = name.trim();
        if (trimmed === "" || trimmed === "." || trimmed === ".."
                || trimmed.indexOf("/") >= 0) {
            errorText = qsTr("Informe um nome sem barras.");
            return;
        }
        destination = destinationDirectory + "/" + trimmed;
        if (destination === source) {
            errorText = qsTr("Origem e destino são iguais. Escolha outro nome.");
            return;
        }
        errorText = "";
        pending = true;
        if (cut) movePathRequested(source, destination);
        else copyPathRequested(source, destination);
    }

    function copied(from, to) {
        if (!pending || cut || from !== source || to !== destination) return false;
        if (tree.workspaceRoot === "" || !to.startsWith(tree.workspaceRoot + "/")) {
            clear();
            return true;
        }
        const kind = sourceKind;
        clear();
        if (kind !== "") tree.selectEntry(to, kind);
        tree.listDirRequested(tree.parentDir(to));
        tree.focusTreeRequested();
        return true;
    }

    function moved(from, to) {
        if (!pending || !cut || from !== source || to !== destination) return false;
        if (transferFromClipboard) clipboard.clearCutFileIfMatches(from);
        clear();
        return true;
    }

    function requestFailed(method, message, to = "") {
        if (method === "fs.transferBatch" && batchPending) {
            batchPending = false;
            batchErrorText = message;
            batchDialogVisible = true;
            return true;
        }
        if (!pending || method !== (cut ? "fs.rename" : "fs.copy")) return false;
        if (to !== "" && to !== destination) return false;
        pending = false;
        errorText = message;
        dialogVisible = true;
        return true;
    }

    function confirmBatch(draftEntries) {
        if (!batchDialogVisible || batchPending || batchEntries.length === 0) return;
        if (draftEntries !== undefined) batchEntries = draftEntries.map(entry => ({
            from: entry.from, name: entry.name, included: entry.included,
            status: entry.status, error: entry.error
        }));
        const items = [];
        const indices = [];
        const destinations = {};
        for (let i = 0; i < batchEntries.length; i++) {
            const entry = batchEntries[i];
            if (!entry.included) continue;
            const name = entry.name.trim();
            if (name === "" || name === "." || name === ".." || name.indexOf("/") >= 0) {
                batchErrorText = qsTr("Corrija o nome do item %1.").arg(i + 1);
                return;
            }
            const to = batchDestinationDirectory + "/" + name;
            if (to === entry.from || destinations[to]) {
                batchErrorText = qsTr("Origem igual ao destino ou nomes repetidos no lote.");
                return;
            }
            destinations[to] = true;
            items.push({ from: entry.from, to: to });
            indices.push(i);
        }
        if (items.length === 0) {
            batchErrorText = qsTr("Inclua ao menos um item no lote.");
            return;
        }
        batchSubmitted = items;
        batchSubmittedIndices = indices;
        batchPending = true;
        batchErrorText = "";
        transferBatchRequested(batchImport ? "import" : batchCut ? "move" : "copy", items);
    }

    // Estado do item de fs.transferBatch; JobsPanel interpreta o estado de um
    // job inteiro. As duas respostas usam a mesma string, mas têm donos distintos.
    function batchItemSucceeded(status) {
        return status === "success";
    }

    function batchTransferred(result) {
        if (!batchPending || result.operation !== (batchImport ? "import" : batchCut ? "move" : "copy")
                || result.items.length !== batchSubmitted.length) return false;
        for (let i = 0; i < result.items.length; i++) {
            if (result.items[i].from !== batchSubmitted[i].from
                    || result.items[i].to !== batchSubmitted[i].to) return false;
        }
        const entries = batchEntries.slice();
        const refreshed = {};
        const moved = [];
        let failures = 0;
        for (let i = 0; i < result.items.length; i++) {
            const item = result.items[i];
            const index = batchSubmittedIndices[i];
            const entry = entries[index];
            const success = batchItemSucceeded(item.status);
            entries[index] = { from: entry.from, name: entry.name,
                               included: !success, status: item.status,
                               error: item.error || "" };
            if (success) {
                refreshed[tree.parentDir(item.to)] = true;
                if (batchCut) {
                    moved.push(item.from);
                    refreshed[tree.parentDir(item.from)] = true;
                    tree.tabsRenameRequested(item.from, item.to);
                }
            } else failures++;
        }
        batchPending = false;
        batchEntries = entries;
        if (batchCut && transferFromClipboard && moved.length > 0) {
            clipboard.removeCutFilesIfMatches(batchSourcePaths, moved);
            batchSourcePaths = batchSourcePaths.filter(path => moved.indexOf(path) < 0);
        }
        for (const directory in refreshed) tree.listDirRequested(directory);
        if (failures === 0) {
            clear();
            tree.focusTreeRequested();
        } else {
            batchErrorText = qsTr("%1 item(ns) não concluído(s). Ajuste ou pule e tente novamente.")
                             .arg(failures);
            batchDialogVisible = true;
        }
        return true;
    }
}
