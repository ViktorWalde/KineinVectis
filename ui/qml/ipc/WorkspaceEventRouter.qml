import QtQuick

Item {
    id: root

    property var coreClient: null
    property var folderPicker: null
    property var projectTree: null
    property var searchEverywhereController: null
    property var workspaceController: null
    property var projectHealthController: null
    property var recentWorkspacesController: null

    visible: false

    Connections {
        target: root.coreClient

        function onStatusChanged() {
            if (root.coreClient.connected && !root.searchEverywhereController.hasCommands) {
                root.coreClient.listCommands();
            }
        }

        function onCmakeStatusResolved(configured, hasCompileCommands, cdbStale,
                                       cdbStaleBecause, preset) {
            root.projectHealthController.handleCmakeStatus(configured, cdbStale,
                                                           cdbStaleBecause, preset);
        }

        function onCmakeConfigureFinished(success) {
            root.projectHealthController.handleCmakeFinished(success);
        }

        function onFileSaved(path) {
            // Auto-setup (radar DocsPrivate/diario/18): salvar CMakeLists/Presets pela
            // IDE reconfigura sozinho (job na aba Jobs, sem roubar foco).
            if (root.coreClient.workspaceBuildSystems.indexOf("cmake") < 0) {
                return;
            }
            if (path.endsWith("/CMakeLists.txt")
                    || path.endsWith("/CMakePresets.json")) {
                root.coreClient.cmakeConfigure();
            }
        }

        function onCargoMetadataResolved(packages) {
            root.projectHealthController.handleCargoMetadataResolved();
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

        function onFilesChanged(changes) {
            root.projectTree.handleExternalChanges(changes);
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

        function onRecentWorkspacesResolved(workspaces) {
            root.recentWorkspacesController.handleResolved(workspaces);
        }

        function onToolsListed(tools) {
            root.workspaceController.toolsList = tools;
        }

        function onRequestFailed(method, message) {
            root.recentWorkspacesController.handleRequestFailed(method, message);
            if (method === "workspace.open" || method === "workspace.browse"
                    || method === "workspace.createFolder"
                    || method === "workspace.createProject") {
                root.folderPicker.showError(message);
            }
            if (method === "fs.createFile" || method === "fs.createDirectory"
                    || method === "fs.rename" || method === "fs.delete") {
                root.projectTree.handleRequestFailed(method, message);
            }
            if (method === "cargo.metadata") {
                root.projectHealthController.handleCargoMetadataFailed(message);
            }
        }
    }
}
