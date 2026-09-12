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

    AppDomains {
        id: domains

        coreClient: coreClient
        workspaceHost: workspaceHost
        shellOverlays: shellOverlays
        folderPicker: folderPicker
        workspaceUiResetter: workspaceUiResetter
        hostWidth: root.width
        hostHeight: root.height
    }

    Connections {
        target: domains.editorController

        // Troca de aba (ou abertura/fechamento de arquivo) re-aponta o
        // diff da gutter — e o blame, quando ligado — para o arquivo
        // ativo.
        function onCurrentTabChanged() {
            domains.gitController.requestDiffFor(domains.editorController.currentFilePath());
            domains.gitController.requestBlameFor(domains.editorController.currentFilePath());
            domains.diagnosticsController.setActivePath(domains.editorController.currentFilePath());
            domains.indexController.setActivePath(domains.editorController.currentFilePath());
        }
    }

    WorkspaceUiResetter {
        id: workspaceUiResetter

        debugController: domains.debugController
        gitController: domains.gitController
        diagnosticsController: domains.diagnosticsController
        projectTree: domains.projectTree
        editorController: domains.editorController
        jobsController: domains.jobsController
        searchController: domains.searchController
        searchEverywhereController: domains.searchEverywhereController
        runtimeController: domains.runtimeController
        runConfigController: domains.runConfigController
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
            domains.editorController.applyFormatCapabilities(formatters);
        }

        function onConnectedChanged() {
            if (coreClient.connected && domains.workspaceController.toolsList.length === 0) {
                coreClient.detectTools();
            }
            if (coreClient.connected) {
                coreClient.settingsGet();
                domains.recentWorkspacesController.listRequested();
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
        target: domains.settingsController

        function onResolved() {
            domains.shellController.applySettings(domains.settingsController);
        }
    }

    GlobalShortcuts {
        // `shellController` VOLTA A SER LIGADO. Ele foi retirado daqui em
        // 414972d, com a justificativa medida de que era "recebida e nunca
        // usada" — e naquele momento era verdade. Depois o Ctrl+O
        // (`workspace.open`) passou a chama-lo, e a chamada caiu num `null`
        // sem barulho: o menu "Abrir workspace..." funcionava, a tecla nao.
        shellController: domains.shellController
        debugController: domains.debugController
        editorController: domains.editorController
        jobsController: domains.jobsController
        runtimeController: domains.runtimeController
        searchController: domains.searchController
        searchEverywhereController: domains.searchEverywhereController
    }

    EnvironmentShortcuts {
        settingsController: domains.settingsController
        configActionController: domains.configActionController
        libraryController: domains.libraryController
        dataSourceController: domains.dataSourceController
        grafanaController: domains.grafanaController
        embeddedController: domains.embeddedController
        setupController: domains.setupController
        containerController: domains.containerController
    }

    ShellHeaderHost {
        id: header

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: coreClient
        shellController: domains.shellController
        jobsController: domains.jobsController
        runtimeController: domains.runtimeController
        runConfigController: domains.runConfigController
        debugController: domains.debugController
        editorController: domains.editorController
        projectTree: domains.projectTree
        searchController: domains.searchController
        searchEverywhereController: domains.searchEverywhereController
        settingsController: domains.settingsController
        libraryController: domains.libraryController
        dataSourceController: domains.dataSourceController
        grafanaController: domains.grafanaController
        embeddedController: domains.embeddedController
        setupController: domains.setupController
        containerController: domains.containerController
        configActionController: domains.configActionController
        toolchainController: domains.toolchainController
        recentWorkspacesController: domains.recentWorkspacesController
        windowMaximized: windowChromeController.maximized
        onConfigMenuRequested: function(menuX, menuY) {
            const pos = header.mapToItem(shellOverlays, menuX, menuY);
            domains.runConfigController.openConfigMenu(pos.x, pos.y);
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
        shellController: domains.shellController
        workspaceController: domains.workspaceController
        projectHealthController: domains.projectHealthController
        projectTree: domains.projectTree
        editorController: domains.editorController
        jobsController: domains.jobsController
        runtimeController: domains.runtimeController
        debugController: domains.debugController
        gitController: domains.gitController
        diagnosticsController: domains.diagnosticsController
        searchController: domains.searchController
        recentWorkspacesController: domains.recentWorkspacesController
        containerController: domains.containerController
        grafanaController: domains.grafanaController
        workspaceOpen: coreClient.workspaceRoot !== ""
        workspaceRoot: coreClient.workspaceRoot
        workspaceName: coreClient.workspaceName
        workspaceKind: coreClient.workspaceKind
        workspaceBuildSystems: coreClient.workspaceBuildSystems
        testing: coreClient.testing
        terminalActive: coreClient.terminalActive
        running: coreClient.running
        logLinesModel: coreClient.logLines
        toolsList: domains.workspaceController.toolsList
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
            domains.shellController.showTab("jobs");
            coreClient.cmakeConfigure();
        }
        onCargoMetadataRequested: coreClient.cargoMetadata()
        onCreateProjectRequested: function(templateId) {
            folderPicker.openCreateProject(coreClient.homeDir, templateId);
        }
        onSettingsRequested: domains.settingsController.openDialog()
    }

    ShellStatusHost {
        id: statusBar

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: coreClient
        shellController: domains.shellController
        gitController: domains.gitController
        toolchainController: domains.toolchainController
        indexController: domains.indexController
        onToolchainMenuRequested: function(menuX, menuY) {
            const pos = statusBar.mapToItem(shellOverlays, menuX, menuY);
            domains.toolchainController.openMenu(pos.x, pos.y);
        }
    }

    ShellOverlays {
        id: shellOverlays

        // Overlay global acima do header (z=100) e de toda a workspace. O z
        // interno de um popup não escapa do stacking context do pai.
        z: 1000
        hostWidth: root.width
        hostHeight: root.height
        searchEverywhereController: domains.searchEverywhereController
        projectTree: domains.projectTree
        editorController: domains.editorController
        shellController: domains.shellController
        runtimeController: domains.runtimeController
        runConfigController: domains.runConfigController
        gitController: domains.gitController
        settingsController: domains.settingsController
        libraryController: domains.libraryController
        dataSourceController: domains.dataSourceController
        grafanaController: domains.grafanaController
        embeddedController: domains.embeddedController
        setupController: domains.setupController
        containerController: domains.containerController
        configActionController: domains.configActionController
        toolchainController: domains.toolchainController
        simController: domains.simController
        simRunController: domains.simRunController
        simSystemController: domains.simSystemController
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
