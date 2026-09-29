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
    }

    function pathRenamed(from, to) {
        tree.entryRenameVisible = false;
        tree.entryRenameError = "";
        tree.tabsRenameRequested(from, to);
        tree.selectEntry(to, tree.entryRenameKind);
        tree.listDirRequested(tree.parentDir(to));
        tree.focusTreeRequested();
    }

    function pathDeleted(path) {
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
        if (method === "fs.rename") {
            tree.entryRenameError = message;
            tree.entryRenameVisible = true;
        }
        if (method === "fs.delete") {
            tree.entryDeleteError = message;
            tree.entryDeleteVisible = true;
        }
    }
}
