import QtQuick

Item {
    id: root

    property var coreClient: null
    property var editorController: null
    property var jobsController: null
    property var projectTree: null
    property var runtimeController: null
    property var debugController: null
    property var gitController: null
    property var settingsController: null
    property var searchController: null
    property var searchEverywhereController: null
    property var configActionController: null
    property var libraryController: null
    property var dataSourceController: null
    property var grafanaController: null
    property var setupController: null

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
        } else if (commandId === "fs.replace") {
            searchController.openReplacePanel();
        } else if (commandId === "editor.find") {
            // D1b: busca NO ARQUIVO (UI pura, sem RPC) — o fs.search acima
            // é a busca no PROJETO, que roda ripgrep no core.
            editorController.openFind();
        } else if (commandId === "editor.replace") {
            editorController.openFindReplace();
        } else if (commandId === "fs.findFiles" || commandId === "command.list") {
            searchEverywhereController.openSearchEverywhere();
        } else if (commandId === "cargo.check") {
            showTabRequested("problems");
            coreClient.cargoCheck();
        } else if (commandId === "cargo.metadata") {
            coreClient.cargoMetadata();
        } else if (commandId === "cmake.configure") {
            showTabRequested("jobs");
            coreClient.cmakeConfigure();
        } else if (commandId === "run.start") {
            runtimeController.startRun("");
        } else if (commandId === "debug.start") {
            debugController.startDebug();
        } else if (commandId === "git.status") {
            gitController.refresh();
        } else if (commandId === "git.fileDiff") {
            gitController.openDiffDialog(editorController.currentFilePath());
        } else if (commandId === "git.commit") {
            showTabRequested("git");
            gitController.showChanges();
        } else if (commandId === "git.blame") {
            gitController.toggleBlame(editorController.currentFilePath());
        } else if (commandId === "git.log") {
            showTabRequested("git");
            gitController.openHistory();
        } else if (commandId === "git.branches") {
            showTabRequested("git");
            gitController.openBranchMenu();
        } else if (commandId === "git.pull") {
            gitController.startRemote("pull");
        } else if (commandId === "git.push") {
            gitController.startRemote("push");
        } else if (commandId === "git.stash") {
            showTabRequested("git");
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
        } else if (commandId === "lsp.switchSourceHeader") {
            editorController.requestSwitchSourceHeader();
        } else if (commandId === "lsp.restart") {
            coreClient.lspRestart("");
        } else if (commandId === "configAction.list") {
            configActionController.openDialog();
        } else if (commandId === "library.list") {
            libraryController.open();
        } else if (commandId === "datasource.list") {
            dataSourceController.open();
        } else if (commandId === "grafana.get") {
            grafanaController.open();
        } else if (commandId === "setup.list") {
            setupController.open();
        } else if (commandId === "settings.get") {
            settingsController.openDialog();
        } else if (commandId === "fs.createFile") {
            projectTree.openCreateDialog("file");
        } else if (commandId === "fs.createDirectory") {
            projectTree.openCreateDialog("directory");
        }
    }
}
