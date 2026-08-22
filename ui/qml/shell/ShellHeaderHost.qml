import QtQuick
import KineinVectis

// App Bar UNICA (F2, 2026-07-18): a AppMenuBar e a barra inteira (moldura,
// hamburguer, menus, drag, controles de janela) e o TopHeaderBar e um cluster
// de toolbar ancorado a direita, antes dos controles. As duas barras
// empilhadas (40+44px) viraram uma de 46px — LAYOUT_SYSTEM §4/§9/§10.
Item {
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
    property var settingsController: null
    property var recentWorkspacesController: null
    property bool windowMaximized: false

    signal configMenuRequested(real menuX, real menuY)
    signal appMenuRequested(string key, real menuX, real menuY, var items)
    signal aboutRequested()
    signal manualRequested()
    signal minimizeRequested()
    signal maximizeRestoreRequested()
    signal closeWindowRequested()
    signal moveWindowRequested()

    height: 46
    z: 100

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
        switch (action) {
        case "workspace.open": root.shellController.requestOpenFolder(); break;
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
        case "view.jobs": root.shellController.toggleBottomTab("jobs"); break;
        case "view.compileContext":
            root.shellController.toggleBottomTab("compileContext");
            break;
        case "view.logs": root.shellController.toggleBottomTab("logs"); break;
        case "view.git": root.shellController.toggleBottomTab("git"); break;
        case "search.everywhere": root.searchController.openSearchEverywhere(); break;
        case "search.recent": root.searchController.openRecentFiles(); break;
        case "search.documentSymbols":
            root.searchController.openSearchEverywhere();
            root.searchController.runSymbolSearch("@");
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
        case "quality.run": root.jobsController.startQuality(); break;
        case "audit.run": root.jobsController.startAudit(); break;
        case "memcheck.run": root.jobsController.startMemcheck(); break;
        case "run.start": root.runtimeController.startRun(""); break;
        case "run.stop": root.runtimeController.stopRun(); break;
        case "debug.start": root.debugController.startDebug(); break;
        case "debug.stop": root.debugController.stopDebug(); break;
        case "tools.detect": root.coreClient.detectTools(); break;
        case "help.manual": root.manualRequested(); break;
        case "help.about": root.aboutRequested(); break;
        case "app.quit": Qt.quit(); break;
        }
    }

    AppMenuBar {
        id: appMenuBar

        anchors.fill: parent
        // O drag region para antes do cluster de toolbar (irmao abaixo).
        reservedRight: headerToolbar.width + Theme.spacingSmall
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

    function closeAppMenu() {
        appMenuBar.closeMenu();
    }

    TopHeaderBar {
        id: headerToolbar

        anchors.right: parent.right
        anchors.rightMargin: appMenuBar.controlsWidth + Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        hostWidth: root.width
        workspaceOpen: root.coreClient.workspaceRoot !== ""
        coreConnected: root.coreClient.connected
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        activeConfigId: root.runConfigController.activeConfigId
        activeConfigName: root.runConfigController.activeConfigName
        configMenuOpen: root.runConfigController.configMenuVisible
        building: root.coreClient.building
        testing: root.coreClient.testing
        analyzing: root.coreClient.analyzing
        running: root.coreClient.running
        debugging: root.coreClient.debugging
        onOpenWorkspaceRequested: root.shellController.requestOpenFolder()
        onBuildRequested: buildSystem => root.jobsController.startBuild(buildSystem)
        onTestsRequested: buildSystem => root.jobsController.startTests(buildSystem)
        onQualityRequested: root.jobsController.startQuality()
        onRunRequested: root.runtimeController.startRun("")
        onStopRunRequested: root.runtimeController.stopRun()
        onDebugRequested: root.debugController.startDebug()
        onStopDebugRequested: root.debugController.stopDebug()
        onConfigureRequested: {
            root.shellController.showTab("jobs");
            root.coreClient.cmakeConfigure();
        }
        onConfigMenuRequested: function(menuX, menuY) {
            // O cluster nao ocupa a barra inteira: as coordenadas vem no
            // espaco DELE e precisam do mapeamento real (o antigo "+40"
            // era a costura das duas barras empilhadas — morreu com a fusao).
            const pos = headerToolbar.mapToItem(root, menuX, menuY);
            root.configMenuRequested(pos.x, pos.y);
        }
    }
}
