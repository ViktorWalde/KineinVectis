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
    property var remoteController: null
    property var grafanaController: null
    property var embeddedController: null
    property var setupController: null
    property var containerController: null

    // "open" ou "createProject": o seletor de pasta, com a intencao.
    signal folderRequested(string intent)
    signal showTabRequested(string tab)
    // `index.symbols`: a aba Simbolos a direita (E3-2) — do shell, nao do
    // core. `<id>=<arg>` (so' a medicao headless usa): o texto da busca, a
    // aba de um painel — a paleta nunca manda `=`.
    signal symbolsRequested(string query)

    visible: false

    // Diz que o id nao tem dono aqui. Ate' 2026-09-24 o `execute` terminava a
    // cadeia em silencio: comando errado na paleta, no menu ou no
    // `KINEIN_STARTUP_COMMANDS` simplesmente NAO ACONTECIA, e nada dizia por
    // que. A §6 da V3 pede "resultado observavel para ID desconhecido".
    signal unknownCommand(string id)

    // `true` quando alguem tratou; `false` quando ninguem tratou.
    function execute(rawId) {
        const eq = rawId.indexOf("=");
        const arg = eq > 0 ? rawId.substring(eq + 1) : "";
        const commandId = eq > 0 ? rawId.substring(0, eq) : rawId;
        switch (commandId) {
        case "workspace.open":
            folderRequested("open");
            return true;
        // Estava no catalogo da paleta ("New Project") sem dono aqui: escolher
        // nao fazia nada (achado da F0, 2026-10-01).
        case "workspace.createProject":
            folderRequested("createProject");
            return true;
        // O painel de areas do trilho: o shell trata "areas" como trata "git".
        case "view.areas":
            showTabRequested("areas");
            return true;
        case "view.focusMode":
            showTabRequested("focusMode");
            return true;
        case "view.cycleFocus":
            showTabRequested("focusNext");
            return true;
        case "view.cycleFocusBack":
            showTabRequested("focusPrevious");
            return true;
        case "view.returnToEditor":
            editorController.focusEditor();
            return true;
        case "workspace.close":
            coreClient.closeWorkspace();
            return true;
        case "tools.detect":
        case "tools.status":
            showTabRequested("tools");
            coreClient.detectTools();
            return true;
        case "build.run":
            jobsController.startBuild();
            return true;
        case "fs.search":
            searchController.openSearchPanel();
            return true;
        case "fs.replace":
            searchController.openReplacePanel();
            return true;
        // D1b: busca NO ARQUIVO (UI pura, sem RPC) — o fs.search acima e' a
        // busca no PROJETO, que roda ripgrep no core.
        case "editor.find":
            editorController.openFind();
            return true;
        case "editor.replace":
            editorController.openFindReplace();
            return true;
        case "fs.findFiles":
        case "command.list":
            searchEverywhereController.openSearchEverywhere();
            return true;
        case "index.symbols":
            symbolsRequested(arg);
            return true;
        case "cargo.check":
            showTabRequested("problems");
            coreClient.cargoCheck();
            return true;
        case "cargo.metadata":
            coreClient.cargoMetadata();
            return true;
        case "cmake.configure":
            showTabRequested("jobs");
            coreClient.cmakeConfigure();
            return true;
        case "run.start":
            runtimeController.startRun("");
            return true;
        case "debug.start":
            debugController.startDebug();
            return true;
        case "git.status":
            gitController.refresh();
            return true;
        case "git.fileDiff":
            gitController.showDiffOf(editorController.currentFilePath());
            return true;
        case "git.commit":
            showTabRequested("git");
            gitController.showChanges();
            return true;
        case "git.blame":
            gitController.toggleBlame(editorController.currentFilePath());
            return true;
        case "git.log":
            showTabRequested("git");
            gitController.openHistory();
            return true;
        case "git.branches":
            showTabRequested("git");
            gitController.openBranchMenu();
            return true;
        case "git.pull":
            gitController.startRemote("pull");
            return true;
        case "git.push":
            gitController.startRemote("push");
            return true;
        case "git.stash":
            showTabRequested("git");
            return true;
        case "run.stop":
            runtimeController.stopRun();
            return true;
        case "terminal.open":
            runtimeController.openTerminalPanel();
            return true;
        case "lsp.definition":
            editorController.requestDefinition();
            return true;
        case "lsp.hover":
            editorController.requestHover();
            return true;
        case "lsp.completion":
            editorController.requestCompletion();
            return true;
        case "lsp.references":
            editorController.requestUsages();
            return true;
        case "lsp.rename":
            editorController.openRenameDialog();
            return true;
        case "lsp.switchSourceHeader":
            editorController.requestSwitchSourceHeader();
            return true;
        case "lsp.restart":
            coreClient.lspRestart("");
            return true;
        case "configAction.list":
            configActionController.openDialog();
            return true;
        case "library.list":
            libraryController.open();
            return true;
        case "datasource.list":
            dataSourceController.open();
            return true;
        case "remote.list":
            remoteController.open();
            return true;
        case "grafana.get":
            grafanaController.open();
            return true;
        case "probe.list":
            embeddedController.open();
            if (arg !== "") {
                embeddedController.tab = arg;
            }
            return true;
        case "setup.list":
            setupController.open();
            return true;
        case "container.list":
            containerController.open();
            return true;
        case "settings.get":
            settingsController.openDialog();
            return true;
        case "fs.createFile":
            projectTree.openCreateDialog("file");
            return true;
        case "fs.createDirectory":
            projectTree.openCreateDialog("directory");
            return true;
        default:
            unknownCommand(commandId);
            return false;
        }
    }
}
