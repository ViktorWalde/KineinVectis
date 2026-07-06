import QtQuick

Item {
    id: root

    property var coreClient: null
    property var folderPicker: null
    property var projectTree: null
    property var searchController: null
    property var workspaceController: null

    visible: false

    Connections {
        target: root.coreClient

        function onStatusChanged() {
            if (root.coreClient.connected && !root.searchController.hasCommands) {
                root.coreClient.listCommands();
            }
        }

        function onDirListed(path, entries) {
            root.projectTree.setDirectoryListing(path, entries);
        }

        function onFileCreated(path) {
            root.projectTree.handleFileCreated(path);
        }

        function onDirectoryCreated(path) {
            root.projectTree.handleDirectoryCreated(path);
        }

        function onPathRenamed(from, to) {
            root.projectTree.handlePathRenamed(from, to);
        }

        function onPathDeleted(path) {
            root.projectTree.handlePathDeleted(path);
        }

        function onWorkspaceBrowseListed(path, parent, entries) {
            root.folderPicker.setListing(path, parent, entries);
        }

        function onWorkspaceFolderCreated(path) {
            root.folderPicker.selectAfterRefresh(path);
            root.coreClient.browseWorkspaceFolders(root.folderPicker.currentPath);
        }

        function onWorkspaceChanged() {
            root.folderPicker.close();
            root.workspaceController.handleWorkspaceChanged();
        }

        function onToolsListed(tools) {
            root.workspaceController.toolsList = tools;
        }

        function onRequestFailed(method, message) {
            if (method === "workspace.open" || method === "workspace.browse"
                    || method === "workspace.createFolder"
                    || method === "workspace.createProject") {
                root.folderPicker.showError(message);
            }
            if (method === "fs.createFile" || method === "fs.createDirectory"
                    || method === "fs.rename" || method === "fs.delete") {
                root.projectTree.handleRequestFailed(method, message);
            }
        }
    }
}
