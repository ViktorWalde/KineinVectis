import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var coreClient: null
    property var shellController: null
    property var jobsController: null
    property var runtimeController: null
    property var runConfigController: null
    property var debugController: null
    property var editorController: null
    property var projectTree: null
    property var searchController: null
    property var searchEverywhereController: null
    property var settingsController: null
    property var libraryController: null
    property var dataSourceController: null
    property var remoteController: null
    property var grafanaController: null
    property var embeddedController: null
    property var setupController: null
    property var containerController: null
    property var configActionController: null
    property var toolchainController: null
    property var pythonController: null
    property var recentWorkspacesController: null
    property var gitController: null
    property bool windowMaximized: false
    property bool windowEdgesFlush: false
    // Qual menu da barra principal esta' aberto (project | build | ""): o
    // widget correspondente fica marcado enquanto o popup esta' na tela.
    property string headerMenu: ""

    signal configMenuRequested(real menuX, real menuY)
    signal toolchainMenuRequested(real menuX, real menuY)
    signal appMenuRequested(string key, real menuX, real menuY, var items)
    signal aboutRequested()
    signal manualRequested()
    signal minimizeRequested()
    signal maximizeRestoreRequested()
    signal closeWindowRequested()
    signal moveWindowRequested()

    // A MOLDURA de cima em UMA linha (0.3.9, pedido do autor: "aproveitar
    // melhor o espaco para codigo", modelo da JetBrains): o ☰ com o menu
    // recolhido, a barra de ferramentas logo depois e os controles da janela
    // no fim. Eram duas faixas (84 px); sao 44.
    height: 38
    z: 100
    radius: root.windowEdgesFlush ? 0 : Theme.radiusLarge
    // Transparente: a moldura (e o veu ambar) e' o WindowBackdrop.
    color: "transparent"

    function executeMenuAction(action) {
        const recentPrefix = "workspace.recent.open:";
        if (action.indexOf(recentPrefix) === 0) {
            const index = Number(action.substring(recentPrefix.length));
            if (index >= 0 && index < root.recentWorkspacesController.workspaces.length) {
                root.recentWorkspacesController.openWorkspace(
                    root.recentWorkspacesController.workspaces[index].root);
            }
            return;
        }
        const bottomAction = /^bottom\.(pin|unpin):(.+)$/.exec(action);
        if (bottomAction !== null) {
            root.shellController.setBottomPinned(bottomAction[2], bottomAction[1] === "pin");
            return;
        }
        switch (action) {
        case "rail.restore": root.shellController.restoreRail(); break;
        case "view.areas": root.shellController.showTab("areas"); break;
        case "view.focusMode": root.shellController.focusMode.toggle(); break;
        case "view.returnToEditor": root.editorController.focusEditor(); break;
        case "view.cycleFocus": root.shellController.focusCycleRequested(1); break;
        case "view.cycleFocusBack": root.shellController.focusCycleRequested(-1); break;
        case "workspace.open": root.shellController.requestOpenFolder(); break;
        case "workspace.createProject": root.shellController.requestFolder("createProject"); break;
        case "workspace.recent.clear": root.recentWorkspacesController.clearAll(); break;
        case "workspace.close": root.coreClient.closeWorkspace(); break;
        case "project.createFile": root.projectTree.openCreateDialog("file"); break;
        case "project.createDirectory": root.projectTree.openCreateDialog("directory"); break;
        case "editor.save": root.editorController.saveCurrentFile(); break;
        case "editor.saveAll": root.editorController.saveAllFiles(); break;
        case "editor.find": root.editorController.openFind(); break;
        case "editor.replace": root.editorController.openFindReplace(); break;
        case "editor.gotoLine": root.editorController.openGoToLine(); break;
        case "editor.format": root.editorController.formatCurrentFile(); break;
        case "fs.replace": root.searchController.openReplacePanel(); break;
        case "settings.open": root.settingsController.openDialog(); break;
        case "view.project": root.shellController.toggleExplorer(); break;
        case "view.terminal": root.runtimeController.openTerminalPanel(); break;
        case "view.tools": root.shellController.toggleBottomTab("tools"); break;
        case "view.git": root.shellController.toggleBottomTab("git"); break;
        case "search.everywhere": root.searchEverywhereController.openSearchEverywhere(); break;
        case "search.recent": root.searchEverywhereController.openRecentFiles(); break;
        case "search.documentSymbols":
            root.searchEverywhereController.openSearchEverywhere();
            root.searchEverywhereController.runSymbolSearch("@");
            break;
        case "lsp.rename": root.editorController.openRenameDialog(); break;
        case "lsp.codeActions": root.editorController.requestCodeActions(); break;
        case "lsp.switchSourceHeader": root.editorController.requestSwitchSourceHeader(); break;
        case "lsp.restart": root.coreClient.lspRestart(""); break;
        case "cmake.configure": root.coreClient.cmakeConfigure(); break;
        case "build.run": root.jobsController.startBuild(); break;
        case "build.run.cargo": root.jobsController.startBuild("cargo"); break;
        case "build.run.cmake": root.jobsController.startBuild("cmake"); break;
        case "test.run": root.jobsController.startTests(); break;
        case "test.run.cargo": root.jobsController.startTests("cargo"); break;
        case "test.run.cmake": root.jobsController.startTests("cmake"); break;
        case "test.run.python": root.jobsController.startTests("python"); break;
        case "quality.run": root.jobsController.startQuality(); break;
        case "coverage.run": root.jobsController.startCoverage(); break;
        case "run.start": root.runtimeController.startRun(""); break;
        case "run.stop": root.runtimeController.stopRun(); break;
        case "debug.start": root.debugController.startDebug(); break;
        case "debug.stop": root.debugController.stopDebug(); break;
        case "tools.detect": root.coreClient.detectTools(); break;
        case "library.list": root.libraryController.open(); break;
        case "datasource.list": root.dataSourceController.open(); break;
        case "remote.list": root.remoteController.open(); break;
        case "grafana.get": root.grafanaController.open(); break;
        case "probe.list": root.embeddedController.open(); break;
        case "setup.list": root.setupController.open(); break;
        case "container.list": root.containerController.open(); break;
        case "configAction.list": root.configActionController.openDialog(); break;
        // Vindo do menu Ambiente ou do "⋯" dos chips, sem chip para ancorar:
        // o popover abre embaixo do cabecalho, no canto esquerdo (0.3.8 F3).
        case "toolchain.get":
            root.toolchainController.openMenu(Theme.spacingLarge, root.contextMenuY(), true);
            break;
        case "python.context":
            root.pythonController.openMenu(Theme.spacingLarge, root.contextMenuY());
            break;
        case "help.manual": root.manualRequested(); break;
        case "help.about": root.aboutRequested(); break;
        case "app.quit": Qt.quit(); break;
        }
    }

    AppMenuBar {
        id: appMenuBar

        y: 0
        width: parent.width
        workspaceOpen: root.coreClient.workspaceRoot !== ""
        hasActiveFile: root.editorController.currentTab >= 0
        coreConnected: root.coreClient.connected
        running: root.coreClient.running
        debugging: root.coreClient.debugging
        workspaceName: root.coreClient.workspaceName
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        recentWorkspaces: root.recentWorkspacesController.workspaces
        windowMaximized: root.windowMaximized
        onActionRequested: function(action) {
            root.executeMenuAction(action);
        }
        onMenuRequested: function(key, menuX, menuY, items) {
            root.appMenuRequested(key, menuX, menuY, items);
        }
        onMinimizeRequested: root.minimizeRequested()
        onMaximizeRestoreRequested: root.maximizeRestoreRequested()
        onCloseWindowRequested: root.closeWindowRequested()
        onMoveWindowRequested: root.moveWindowRequested()
    }

    function contextMenuY() {
        return root.height + Theme.spacingXSmall;
    }

    // O "⋯" dos chips: so' os contextos que nao couberam, cada um abrindo o dono.
    function contextOverflowItems() {
        const items = [];
        if (headerBar.toolchainHidden) {
            items.push({ label: qsTr("Toolchain: %1").arg(headerBar.toolchainSummary),
                         action: "toolchain.get", enabled: true });
        }
        if (headerBar.pythonHidden) {
            items.push({ label: qsTr("Python: %1").arg(headerBar.pythonSummary),
                         action: "python.context", enabled: true });
        }
        return items;
    }

    function closeAppMenu() {
        appMenuBar.closeMenu();
        headerMenu = "";
    }

    TopHeaderBar {
        id: headerBar

        // ACIMA da AppMenuBar (z 100): na mesma linha, a area de arrastar a
        // janela dela cobria os widgets e nenhum recebia clique (0.3.9). O
        // vazio desta barra nao trata mouse, entao o arrasto continua passando.
        z: 101
        // POSICAO FIXA, logo depois do ☰ (2026-10-03, pedido do autor): com
        // os menus abertos a barra SOME por um momento — eles ocupam o lugar
        // dela — e volta igual ao fechar; antes, ela era EMPURRADA para a
        // direita pelos menus e mudava de lugar a cada clique no ☰.
        x: appMenuBar.collapsedEndX + Theme.spacingSmall
        opacity: appMenuBar.menuExpanded ? 0 : 1
        visible: opacity > 0
        enabled: !appMenuBar.menuExpanded
        Behavior on opacity {
            NumberAnimation { duration: Theme.motionFast }
        }
        y: 0
        width: Math.max(0, appMenuBar.controlsX - Theme.spacingSmall - x)
        height: root.height
        workspaceOpen: root.coreClient.workspaceRoot !== ""
        coreConnected: root.coreClient.connected
        workspaceName: root.coreClient.workspaceName
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        activeConfigId: root.runConfigController.activeConfigId
        activeConfigName: root.runConfigController.activeConfigName
        configMenuOpen: root.runConfigController.configMenuVisible
        projectMenuOpen: root.headerMenu === "project"
        actionsMenuOpen: root.headerMenu === "build"
        building: root.coreClient.building
        testing: root.coreClient.testing
        analyzing: root.coreClient.analyzing
        running: root.coreClient.running
        debugging: root.coreClient.debugging
        gitBranchLabel: root.gitController ? root.gitController.branchLabel : ""
        gitAheadCount: root.gitController ? root.gitController.aheadCount : 0
        gitBehindCount: root.gitController ? root.gitController.behindCount : 0
        gitChangeCount: root.gitController ? root.gitController.changeCount : 0
        gitPanelActive: root.shellController.tabActive("git")
        // So' com papel de toolchain (Cargo, CMake, Make): num Python puro o
        // resumo seria "nenhuma detectada", verdadeiro e inutil.
        toolchainSummary: root.toolchainController.summaryRoles(
                              root.coreClient.workspaceBuildSystems).length > 0
                          && root.coreClient.workspaceBuildSystems.length > 0
                          ? root.toolchainController.summary(root.coreClient.workspaceBuildSystems) : ""
        toolchainMenuOpen: root.toolchainController.menuVisible
        order: root.shellController.savedOrder("header")
        onWidgetMoved: (key, dropIndex, visibleKeys) => root.shellController.moveInBar(
                           "header", visibleKeys, key, dropIndex)
        onToolchainMenuRequested: function(menuX, menuY) {
            root.toolchainMenuRequested(menuX, menuY + headerBar.y);
        }
        pythonSummary: root.pythonController !== null ? root.pythonController.contextLabel() : ""
        pythonMenuOpen: root.pythonController !== null && root.pythonController.menuVisible
        // Os overlays cobrem a janela inteira: a coordenada da janela serve.
        onContextOverflowRequested: function(menuX, menuY) {
            root.headerMenu = "context";
            root.appMenuRequested("context", menuX, menuY + headerBar.y,
                                  root.contextOverflowItems());
        }
        onPythonMenuRequested: function(menuX, menuY) {
            const pos = headerBar.mapToItem(null, menuX, menuY);
            root.pythonController.openMenu(pos.x, pos.y);
        }
        onOpenWorkspaceRequested: root.shellController.requestOpenFolder()
        onGitPanelRequested: root.shellController.toggleBottomTab("git")
        // O branch da barra: a aba Git com o menu de branches aberto.
        onGitBranchMenuRequested: {
            root.shellController.showTab("git");
            root.gitController.openBranchMenu();
        }
        onRunRequested: root.runtimeController.startRun("")
        onStopRunRequested: root.runtimeController.stopRun()
        onDebugRequested: root.debugController.startDebug()
        onStopDebugRequested: root.debugController.stopDebug()
        onConfigMenuRequested: function(menuX, menuY) {
            root.configMenuRequested(menuX, menuY + headerBar.y);
        }
        // Os dois menus da barra usam o MESMO popup dos menus de aplicacao
        // (e as mesmas listas: o de projeto e' o comeco de "Arquivo", o de
        // acoes e' "Build" inteiro) — um dono para o que cada um oferece.
        onProjectMenuRequested: function(menuX, menuY) {
            root.headerMenu = "project";
            root.appMenuRequested("project", menuX, menuY + headerBar.y,
                                  appMenuBar.projectMenuItems());
        }
        onActionsMenuRequested: function(menuX, menuY) {
            root.headerMenu = "build";
            root.appMenuRequested("build", menuX, menuY + headerBar.y,
                                  appMenuBar.menuItems("build"));
        }
    }
}
