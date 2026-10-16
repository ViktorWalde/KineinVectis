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
                                       cdbStaleBecause, preset, presetChoices, presetChoicesFile) {
            root.projectHealthController.handleCmakeStatus(configured, cdbStale,
                                                           cdbStaleBecause, preset,
                                                           presetChoices, presetChoicesFile);
        }

        function onCmakeConfigureFinished(success) {
            root.projectHealthController.handleCmakeFinished(success);
        }

        function onFileSaved(path) {
            // Auto-setup: salvar CMakeLists/Presets pela
            // IDE reconfigura sozinho (job na aba Jobs, sem roubar foco).
            // 16.4b: reconfigurar executa o CMake do projeto; so' com confianca.
            if (root.coreClient.workspaceBuildSystems.indexOf("cmake") < 0
                    || !root.coreClient.workspaceTrusted) {
                return;
            }
            if (path.endsWith("/CMakeLists.txt")
                    || path.endsWith("/CMakePresets.json")) {
                root.coreClient.cmakeConfigure();
            }
        }

        function onWorkspaceTrustedChanged() {
            root.projectHealthController.handleTrust(root.coreClient.workspaceTrusted);
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

        function onPathCopied(from, to) {
            root.projectTree.handlePathCopied(from, to);
        }

        function onPathBatchTransferred(result) {
            root.projectTree.fileClipboard.batchTransferred(result);
        }

        function onPathCopyFailed(to, message) {
            root.projectTree.handleCopyFailed(to, message);
        }

        function onPathDeleted(path) {
            root.projectTree.handlePathDeleted(path);
        }

        function onFilesChanged(changes) {
            root.projectTree.handleExternalChanges(changes);
        }

        function onWorkspaceBrowseListed(path, parent, entries, places) {
            root.folderPicker.setListing(path, parent, entries, places);
        }

        function onWorkspaceFolderCreated(path) {
            root.folderPicker.selectAfterRefresh(path);
            root.coreClient.browseWorkspaceFolders(root.folderPicker.currentPath);
        }

        // As versoes das ferramentas dependem do projeto (o rustup escolhe a
        // toolchain pelo diretorio): o core as mede de novo na raiz dele
        // (16.1, F4; roadmaps/59 §7.1.1).
        function onWorkspaceChanged() {
            root.folderPicker.close();
            root.workspaceController.handleWorkspaceChanged();
            root.coreClient.detectTools();
        }

        function onRecentWorkspacesResolved(workspaces) {
            root.recentWorkspacesController.handleResolved(workspaces);
        }

        function onToolsListed(tools) {
            root.workspaceController.toolsList = tools;
        }

        // Cada ferramenta detectada entra na lista NA HORA (pente-fino
        // 2026-09-18: o sinal existia e ninguem o ouvia; a tela so' via a
        // lista pronta no fim da varredura).
        function onEnvironmentTool(tool) {
            root.workspaceController.handleToolDetected(tool);
        }

        function onRequestFailed(method, message) {
            root.recentWorkspacesController.handleRequestFailed(method, message);
            if (method === "workspace.open" || method === "workspace.browse"
                    || method === "workspace.createFolder"
                    || method === "workspace.createProject") {
                root.folderPicker.showError(message);
            }
            if (method === "fs.createFile" || method === "fs.createDirectory"
                    || method === "fs.rename"
                    || method === "fs.delete" || method === "fs.trash") {
                root.projectTree.handleRequestFailed(method, message);
            }
            if (method === "cargo.metadata") {
                root.projectHealthController.handleCargoMetadataFailed(message);
            }
        }
    }
}
