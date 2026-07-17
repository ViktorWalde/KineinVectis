import QtQuick
import QtQuick.Window
import KineinVectis

Window {
    id: root
    width: 1280
    height: 800
    minimumWidth: 800
    minimumHeight: 500
    visible: true
    flags: Qt.Window | Qt.FramelessWindowHint
    title: coreClient.workspaceName !== ""
           ? coreClient.workspaceName + " — Kinein Vectis"
           : "Kinein Vectis"
    color: Theme.background0

    WindowChromeController {
        id: windowChromeController

        window: root
    }

    CoreClient {
        id: coreClient
    }
    WorkspaceController {
        id: workspaceController

        workspaceRoot: coreClient.workspaceRoot
        onClearWorkspaceUiRequested: workspaceUiResetter.clear()
    }

    RecentWorkspacesController {
        id: recentWorkspacesController

        onListRequested: coreClient.listRecentWorkspaces()
        onOpenRequested: function(rootPath) {
            coreClient.openWorkspace(rootPath);
        }
        onPinRequested: function(rootPath, pinned) {
            coreClient.pinRecentWorkspace(rootPath, pinned);
        }
        onRemoveRequested: function(rootPath) {
            coreClient.removeRecentWorkspace(rootPath);
        }
        onClearRequested: coreClient.clearRecentWorkspaces()
    }

    ProjectHealthController {
        id: projectHealthController

        workspaceRoot: coreClient.workspaceRoot
        workspaceKind: coreClient.workspaceKind
        workspaceBuildSystems: coreClient.workspaceBuildSystems
        toolsList: workspaceController.toolsList
        scanningEnvironment: coreClient.scanningEnvironment
        onAutoConfigureRequested: coreClient.cmakeConfigure()
    }

    ShellController {
        id: shellController

        workspaceRoot: coreClient.workspaceRoot
        workspaceKind: coreClient.workspaceKind
        workspaceBuildSystems: coreClient.workspaceBuildSystems
        homeDir: coreClient.homeDir
        toolsCount: workspaceController.toolsList.length
        onFolderOpenRequested: function(path) {
            folderPicker.open(path);
        }
        onToolsDetectionRequested: coreClient.detectTools()
        onLayoutSaveRequested: function(values) {
            settingsController.setGlobal(values);
        }
    }

    JobsController {
        id: jobsController

        workspaceRoot: coreClient.workspaceRoot
        building: coreClient.building
        testing: coreClient.testing
        analyzing: coreClient.analyzing
        onRunBuildRequested: buildSystem => coreClient.runBuild(buildSystem)
        onRunTestsRequested: buildSystem => coreClient.runTests("", buildSystem)
        onRunQualityRequested: coreClient.runQuality()
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
    }

    DiagnosticsController {
        id: diagnosticsController
    }

    SettingsController {
        id: settingsController

        onGetRequested: coreClient.settingsGet()
        onSetRequested: function(scope, values) {
            coreClient.settingsSet(scope, values);
        }
    }

    RuntimeController {
        id: runtimeController

        workspaceRoot: coreClient.workspaceRoot
        running: coreClient.running
        terminalActive: coreClient.terminalActive
        terminalPanelVisible: shellController.showBottomPanel
                              && shellController.bottomTab === "terminal"
        // Pedido ao core mora no RuntimeRequestRouter. Aqui fica so fiacao de
        // controller para HOST/shell, que nao e IPC.
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onRunConfigDialogOpenRequested: function(name, command) {
            shellOverlays.openRunConfigDialogWith(name, command);
        }
        onFocusTerminalInputRequested: workspaceHost.focusTerminalInput()
        onClearTerminalInputRequested: workspaceHost.clearTerminalInput()
        onClearRunInputRequested: workspaceHost.clearRunInput()
    }

    DebugController {
        id: debugController

        workspaceRoot: coreClient.workspaceRoot
        // Pedido ao core mora no DebugRequestRouter. Aqui fica so fiacao de
        // controller para HOST/editor, que nao e IPC.
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onOpenAtRequested: function(file, line) {
            editorController.openDiagnostic(file, line, 1);
        }
    }

    GitController {
        id: gitController

        workspaceRoot: coreClient.workspaceRoot
        // Toda a fiacao de pedido — inclusive a guarda de arquivo sujo — mora
        // no GitRequestRouter.
    }

    Connections {
        target: editorController

        // Troca de aba (ou abertura/fechamento de arquivo) re-aponta o
        // diff da gutter — e o blame, quando ligado — para o arquivo
        // ativo.
        function onCurrentTabChanged() {
            gitController.requestDiffFor(editorController.currentFilePath());
            gitController.requestBlameFor(editorController.currentFilePath());
            diagnosticsController.setActivePath(editorController.currentFilePath());
        }
    }

    SearchController {
        id: searchController

        workspaceRoot: coreClient.workspaceRoot
        recentFiles: editorController.recentFiles
        hasActiveEditorFile: editorController.currentTab >= 0
        // Pedido ao core (inclusive a guarda de replace e os symbols, que
        // precisam do editor) mora no SearchRequestRouter.
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onFocusSearchInputRequested: workspaceHost.focusSearchInput()
        onFocusReplaceInputRequested: workspaceHost.focusSearchReplaceInput()
        onResetAndFocusEverywhereRequested: shellOverlays.resetSearchEverywhereAndFocus()
        onOpenAtRequested: function(path, line, column) {
            editorController.openDiagnostic(path, line, column);
        }
        onCommandAccepted: function(commandId) {
            commandDispatcher.execute(commandId);
        }
        onFocusEditorRequested: editorController.focusEditor()
    }

    CommandDispatcher {
        id: commandDispatcher

        coreClient: coreClient
        debugController: debugController
        editorController: editorController
        gitController: gitController
        jobsController: jobsController
        projectTree: projectTree
        runtimeController: runtimeController
        settingsController: settingsController
        searchController: searchController
        onOpenWorkspaceRequested: shellController.requestOpenFolder()
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
    }

    EditorController {
        id: editorController

        workspaceRoot: coreClient.workspaceRoot
        editorSurface: workspaceHost.editorSurface
        diagnosticsController: diagnosticsController
        settingsController: settingsController
        // Pedido ao core mora no EditorRequestRouter. O que fica aqui e fiacao
        // de controller para HOST — nao e IPC, e so o Main.qml enxerga os dois.
        onRenameDialogOpenRequested: function(currentName) {
            workspaceHost.openRenameDialogWithName(currentName);
        }
        onGoToLineDialogOpenRequested: function(prefill) {
            workspaceHost.openGoToLineDialog(prefill);
        }
        onFindBarOpenRequested: workspaceHost.focusFindBar()
    }

    ProjectTreeController {
        id: projectTree

        workspaceRoot: coreClient.workspaceRoot
        hostWidth: root.width
        hostHeight: root.height
        // Pedido ao core mora no ProjectTreeRequestRouter. O que a arvore pede a
        // OUTROS dominios e composicao e fica aqui.
        onRunScriptRequested: function(path) {
            runtimeController.startScript(path);
        }
        onTabsRenameRequested: function(from, to) {
            editorController.applyPathRenameToTabs(from, to);
        }
        onTabsCloseRequested: function(path) {
            editorController.closeTabsUnderPath(path);
        }
        onCreateDialogFocusRequested: workspaceHost.focusCreateDialog()
        onEntryRenameDialogOpenRequested: function(name) {
            shellOverlays.openEntryRenameWithName(name);
        }
        onFocusEditorRequested: editorController.focusEditor()
    }

    WorkspaceUiResetter {
        id: workspaceUiResetter

        debugController: debugController
        gitController: gitController
        diagnosticsController: diagnosticsController
        shellController: shellController
        projectTree: projectTree
        editorController: editorController
        jobsController: jobsController
        searchController: searchController
        runtimeController: runtimeController
        bottomPanelHost: workspaceHost
    }

    FolderPickerDialog {
        id: folderPicker

        anchors.fill: parent
        homePath: coreClient.homeDir
        onBrowseRequested: function(path) {
            coreClient.browseWorkspaceFolders(path);
        }
        onOpenRequested: function(path) {
            coreClient.openWorkspace(path);
        }
        onCreateFolderRequested: function(parent, name) {
            coreClient.createWorkspaceFolder(parent, name);
        }
        onCreateProjectRequested: function(parent, name, templateId) {
            coreClient.createWorkspaceProject(parent, name, templateId);
        }
    }

    Component.onCompleted: {
        coreClient.start();
    }

    Connections {
        target: coreClient

        function onFormatCapabilitiesListed(formatters) {
            editorController.applyFormatCapabilities(formatters);
        }

        function onConnectedChanged() {
            if (coreClient.connected && workspaceController.toolsList.length === 0) {
                coreClient.detectTools();
            }
            if (coreClient.connected) {
                coreClient.settingsGet();
                recentWorkspacesController.listRequested();
                // O catalogo de formatters e estatico: pedir uma vez por
                // conexao basta. A UI nao mantem lista propria (0.61.0).
                coreClient.formatCapabilities();
            }
        }

        function onWorkspaceChanged() {
            coreClient.settingsGet();
        }
    }


    Connections {
        target: settingsController

        function onResolved() {
            shellController.applySettings(settingsController);
        }
    }

    WorkspaceEventRouter {
        coreClient: coreClient
        folderPicker: folderPicker
        projectTree: projectTree
        searchController: searchController
        workspaceController: workspaceController
        projectHealthController: projectHealthController
        recentWorkspacesController: recentWorkspacesController
    }

    EditorEventRouter {
        coreClient: coreClient
        editorController: editorController
    }

    EditorRequestRouter {
        coreClient: coreClient
        editorController: editorController
    }

    JobsEventRouter {
        coreClient: coreClient
        jobsController: jobsController
        diagnosticsController: diagnosticsController
    }

    SettingsEventRouter {
        coreClient: coreClient
        settingsController: settingsController
    }

    SearchEventRouter {
        coreClient: coreClient
        searchController: searchController
    }

    SearchRequestRouter {
        coreClient: coreClient
        searchController: searchController
        editorController: editorController
    }

    ProjectTreeRequestRouter {
        coreClient: coreClient
        projectTree: projectTree
    }

    RuntimeEventRouter {
        coreClient: coreClient
        runtimeController: runtimeController
    }

    RuntimeRequestRouter {
        coreClient: coreClient
        runtimeController: runtimeController
    }

    DebugEventRouter {
        coreClient: coreClient
        debugController: debugController
    }

    DebugRequestRouter {
        coreClient: coreClient
        debugController: debugController
    }

    GitEventRouter {
        coreClient: coreClient
        gitController: gitController
    }

    GitRequestRouter {
        coreClient: coreClient
        gitController: gitController
        editorController: editorController
    }

    GlobalShortcuts {
        debugController: debugController
        editorController: editorController
        jobsController: jobsController
        runtimeController: runtimeController
        searchController: searchController
        settingsController: settingsController
    }

    ShellHeaderHost {
        id: header

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: coreClient
        shellController: shellController
        jobsController: jobsController
        runtimeController: runtimeController
        debugController: debugController
        editorController: editorController
        projectTree: projectTree
        searchController: searchController
        settingsController: settingsController
        recentWorkspacesController: recentWorkspacesController
        windowMaximized: windowChromeController.maximized
        onConfigMenuRequested: function(menuX, menuY) {
            const pos = header.mapToItem(shellOverlays, menuX, menuY);
            runtimeController.openConfigMenu(pos.x, pos.y);
        }
        onAppMenuRequested: function(key, menuX, menuY, items) {
            if (key === "") {
                shellOverlays.closeAppMenu();
                return;
            }
            const pos = header.mapToItem(shellOverlays, menuX, menuY);
            shellOverlays.openAppMenu(pos.x, pos.y, items);
        }
        onAboutRequested: shellOverlays.openAboutDialog()
        onManualRequested: shellOverlays.openManualDialog()
        onMinimizeRequested: windowChromeController.minimize()
        onMaximizeRestoreRequested: windowChromeController.toggleMaximized()
        onCloseWindowRequested: windowChromeController.closeWindow()
        onMoveWindowRequested: windowChromeController.startSystemMove()
    }

    ShellWorkspaceHost {
        id: workspaceHost

        anchors.top: header.bottom
        anchors.bottom: statusBar.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.panelGap
        shellController: shellController
        workspaceController: workspaceController
        projectHealthController: projectHealthController
        projectTree: projectTree
        editorController: editorController
        jobsController: jobsController
        runtimeController: runtimeController
        debugController: debugController
        gitController: gitController
        diagnosticsController: diagnosticsController
        searchController: searchController
        recentWorkspacesController: recentWorkspacesController
        workspaceOpen: coreClient.workspaceRoot !== ""
        workspaceRoot: coreClient.workspaceRoot
        workspaceName: coreClient.workspaceName
        workspaceKind: coreClient.workspaceKind
        workspaceBuildSystems: coreClient.workspaceBuildSystems
        testing: coreClient.testing
        terminalActive: coreClient.terminalActive
        running: coreClient.running
        logLinesModel: coreClient.logLines
        toolsList: workspaceController.toolsList
        scanningEnvironment: coreClient.scanningEnvironment
        onListDirRequested: function(path) {
            coreClient.listDir(path);
        }
        onReadFileRequested: function(path) {
            coreClient.readFile(path);
        }
        onCloseWorkspaceRequested: coreClient.closeWorkspace()
        onToolsDetectionRequested: coreClient.detectTools()
        onEnvironmentScanRequested: coreClient.scanEnvironment()
        onCmakeConfigureRequested: {
            shellController.showTab("jobs");
            coreClient.cmakeConfigure();
        }
        onCargoMetadataRequested: coreClient.cargoMetadata()
        onCreateProjectRequested: function(templateId) {
            folderPicker.openCreateProject(coreClient.homeDir, templateId);
        }
        onSettingsRequested: settingsController.openDialog()
    }

    ShellStatusHost {
        id: statusBar

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: coreClient
        shellController: shellController
        gitController: gitController
    }

    ShellOverlays {
        id: shellOverlays

        // Overlay global acima do header (z=100) e de toda a workspace. O z
        // interno de um popup não escapa do stacking context do pai.
        z: 1000
        hostWidth: root.width
        hostHeight: root.height
        searchController: searchController
        projectTree: projectTree
        editorController: editorController
        shellController: shellController
        runtimeController: runtimeController
        gitController: gitController
        settingsController: settingsController
        onAppMenuActionRequested: function(action) {
            header.executeMenuAction(action);
        }
        onAppMenuDismissed: header.closeAppMenu()
    }

    KvTooltipHost {
        anchors.fill: parent
        z: 10000
    }

    WindowResizeHandles {
        anchors.fill: parent
        z: 20000
        resizeEnabled: !windowChromeController.maximized
                       && root.visibility !== Window.FullScreen
        onResizeRequested: edges => windowChromeController.startSystemResize(edges)
    }

}
