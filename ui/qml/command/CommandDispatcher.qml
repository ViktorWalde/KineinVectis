import QtQuick

Item {
    id: root

    property var coreClient: null
    property var editorController: null
    property var jobsController: null
    property var projectTree: null
    property var runtimeController: null
    property var searchController: null

    signal openWorkspaceRequested()
    signal showTabRequested(string tab)

    visible: false

    function execute(commandId) {
        if (commandId === "workspace.open") {
            openWorkspaceRequested();
        } else if (commandId === "workspace.close") {
            coreClient.closeWorkspace();
        } else if (commandId === "tools.detect" || commandId === "tools.status") {
            showTabRequested("tools");
            coreClient.detectTools();
        } else if (commandId === "build.run") {
            jobsController.startBuild();
        } else if (commandId === "fs.search") {
            searchController.openSearchPanel();
        } else if (commandId === "fs.findFiles" || commandId === "command.list") {
            searchController.openSearchEverywhere();
        } else if (commandId === "run.start") {
            runtimeController.startRun("");
        } else if (commandId === "run.stop") {
            runtimeController.stopRun();
        } else if (commandId === "terminal.open") {
            runtimeController.openTerminalPanel();
        } else if (commandId === "lsp.definition") {
            editorController.requestDefinition();
        } else if (commandId === "lsp.hover") {
            editorController.requestHover();
        } else if (commandId === "lsp.completion") {
            editorController.requestCompletion();
        } else if (commandId === "lsp.references") {
            editorController.requestUsages();
        } else if (commandId === "lsp.rename") {
            editorController.openRenameDialog();
        } else if (commandId === "fs.createFile") {
            projectTree.openCreateDialog("file");
        } else if (commandId === "fs.createDirectory") {
            projectTree.openCreateDialog("directory");
        }
    }
}
