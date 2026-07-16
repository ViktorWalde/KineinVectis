import QtQuick
import KineinVectis

Column {
    id: root

    property var coreClient: null
    property var shellController: null
    property var jobsController: null
    property var runtimeController: null
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

    height: 84
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
        case "view.context": root.runtimeController.openContext(); break;
        case "view.tools": root.shellController.toggleBottomTab("tools"); break;
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

    function closeAppMenu() {
        appMenuBar.closeMenu();
    }

    TopHeaderBar {
        width: parent.width
        workspaceOpen: root.coreClient.workspaceRoot !== ""
        coreConnected: root.coreClient.connected
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        activeConfigId: root.runtimeController.activeConfigId
        activeConfigName: root.runtimeController.activeConfigName
        configMenuOpen: root.runtimeController.configMenuVisible
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
            root.configMenuRequested(menuX, menuY + 40);
        }
    }
}
