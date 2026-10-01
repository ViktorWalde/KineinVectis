import QtQuick

// Respostas de fs.* e do watcher voltam para a arvore e as abas por um lugar.
Item {
    id: root
    property var tree: null
    visible: false

    ProjectTreeRules { id: rules }

    function fileCreated(path) {
        tree.createDialogVisible = false;
        tree.createDialogError = "";
        tree.selectEntry(path, "file");
        tree.listDirRequested(tree.parentDir(path));
        tree.readFileRequested(path);
    }

    function directoryCreated(path) {
        tree.createDialogVisible = false;
        tree.createDialogError = "";
        tree.selectEntry(path, "directory");
        tree.listDirRequested(tree.parentDir(path));
        tree.focusTreeRequested();
    }

    function pathRenamed(from, to, kind) {
        tree.entryRenameVisible = false;
        tree.entryRenameError = "";
        tree.tabsRenameRequested(from, to);
        if (kind !== "") tree.selectEntry(to, kind);
        tree.listDirRequested(tree.parentDir(to));
        if (tree.parentDir(from) !== tree.parentDir(to)) {
            tree.listDirRequested(tree.parentDir(from));
        }
        tree.focusTreeRequested();
    }

    function pathDeleted(path) {
        if (!tree.entryDeletePending || path !== tree.entryDeletePath) return;
        tree.entryDeleteMethod = "";
        tree.entryDeleteVisible = false;
        tree.entryDeleteError = "";
        tree.tabsCloseRequested(path);
        if (tree.selectedPath === path || tree.selectedPath.indexOf(path + "/") === 0) {
            tree.clearSelection();
        }
        tree.listDirRequested(tree.parentDir(path));
        tree.focusTreeRequested();
    }

    function externalChanges(changes) {
        const directories = {};
        for (let i = 0; i < changes.length; i++) {
            const directory = tree.parentDir(changes[i].path);
            if (directory === tree.workspaceRoot) {
                directories[directory] = true;
                continue;
            }
            const index = tree.rowIndexForPath(directory);
            if (index >= 0 && rules.isDirectory(tree.entriesModel.get(index).kind)
                    && tree.entriesModel.get(index).expanded) {
                directories[directory] = true;
            }
        }
        for (const directory in directories) tree.listDirRequested(directory);
    }

    function requestFailed(method, message) {
        if (method === "fs.createFile" || method === "fs.createDirectory") {
            tree.createDialogError = message;
            tree.createDialogVisible = true;
        }
        if (method === "fs.rename" && tree.entryRenameVisible) {
            tree.entryRenameError = message;
            tree.entryRenameVisible = true;
        }
        if (method === "fs.delete" || method === "fs.trash") {
            if (!tree.entryDeletePending || method !== tree.entryDeleteMethod) return;
            tree.entryDeleteMethod = "";
            tree.entryDeleteError = message;
            tree.entryDeleteVisible = true;
        }
    }
}
