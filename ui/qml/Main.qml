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
    title: core.workspaceName !== ""
           ? core.workspaceName + " — Kinein Vectis"
           : "Kinein Vectis"
    color: Theme.frame
    WindowBackdrop { anchors.fill: parent }

    WindowChromeController {
        id: windowChromeController

        window: root
    }

    // UMA JANELA POR PASTA. Enquanto este workspace esta' aberto, esta janela
    // responde por ele: quem digitar `kinein <a mesma pasta>` no terminal
    // desiste de abrir a segunda e pede que esta suba.
    //
    // POR QUE NAO E' SO' CONFORTO: nao ha' lock de workspace, e duas janelas
    // na mesma pasta sao duas donas do `.kinein/` — a ultima a fechar apaga o
    // que a outra gravou, sem erro e sem aviso.
    SingleInstanceGuard {
        workspacePath: core.workspaceRoot

        onActivationRequested: function (token) {
            // MELHOR ESFORCO, E DITO COMO TAL. No Wayland quem decide e' o
            // compositor: sem um token de ativacao valido o pedido vira
            // "janela pronta" na barra de tarefas, e nao foco. O que a IDE
            // garante em qualquer ambiente e' o que importa — a segunda
            // janela nao abriu, e o terminal disse por que.
            root.show();
            root.raise();
            root.requestActivate();
        }
    }

    CoreClient {
        id: core
    }

    AppDomains {
        id: domains

        coreClient: core
        workspaceHost: shellWorkspaceHost
        shellOverlays: overlays
        folderPicker: folderPickerDialog
        workspaceUiResetter: uiResetter
        hostWidth: root.width
        hostHeight: root.height
    }

    Connections {
        target: domains.editorController

        // Trocar de aba re-aponta o diff da gutter (e o blame) para o ativo.
        function onCurrentTabChanged() {
            domains.gitController.requestDiffFor(domains.editorController.currentFilePath());
            domains.gitController.requestBlameFor(domains.editorController.currentFilePath());
            domains.diagnosticsController.setActivePath(domains.editorController.currentFilePath());
            domains.indexController.setActivePath(domains.editorController.currentFilePath());
            domains.coverageController.setActivePath(domains.editorController.currentFilePath());
            // F3: o explorer segue o arquivo ativo.
            domains.projectTree.revealPath(domains.editorController.currentFilePath());
        }
    }

    WorkspaceUiResetter {
        id: uiResetter

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
        bottomPanelHost: shellWorkspaceHost
    }

    FolderPickerDialog {
        id: folderPickerDialog

        anchors.fill: parent
        homePath: core.homeDir
        recentProjects: domains.recentWorkspacesController.visibleWorkspaces
        onBrowseRequested: function(path) {
            core.browseWorkspaceFolders(path);
        }
        onOpenRequested: function(path) {
            core.openWorkspace(path);
        }
        // Escolha de pasta para outro fim (o SDK do kit, 2026-09-17): o
        // caminho volta ao dono que pediu, sem abrir workspace.
        onFolderPicked: function(purpose, path) {
            domains.toolchainController.handlePickedPath(purpose, path);
        }
        onCreateFolderRequested: function(parent, name) {
            core.createWorkspaceFolder(parent, name);
        }
        onCreateProjectRequested: function(parent, name, templateId) {
            core.createWorkspaceProject(parent, name, templateId);
        }
    }

    Component.onCompleted: {
        core.start();
    }

    Connections {
        target: core

        function onFormatCapabilitiesListed(formatters) {
            domains.editorController.applyFormatCapabilities(formatters);
        }

        function onRunCapabilitiesListed(runnable, debuggable) {
            domains.projectTree.applyRunCapabilities(runnable, debuggable);
        }

        function onConnectedChanged() {
            if (core.connected && domains.workspaceController.toolsList.length === 0) {
                core.detectTools();
            }
            if (core.connected) {
                core.settingsGet();
                domains.recentWorkspacesController.listRequested();
                // Os catalogos de formatters e do Executar/Depurar sao
                // estaticos: pedir uma vez por conexao basta. A UI nao mantem
                // lista propria (0.61.0; run.capabilities 0.107.0).
                core.formatCapabilities();
                core.runCapabilities();
            }
        }

        function onWorkspaceChanged() {
            core.settingsGet();
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
        workspaceOpen: core.workspaceRoot !== ""
    }

    ShellHeaderHost {
        id: header

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: core
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
        remoteController: domains.remoteController
        grafanaController: domains.grafanaController
        embeddedController: domains.embeddedController
        setupController: domains.setupController
        containerController: domains.containerController
        configActionController: domains.configActionController
        toolchainController: domains.toolchainController
        pythonController: domains.pythonController
        recentWorkspacesController: domains.recentWorkspacesController
        gitController: domains.gitController
        windowMaximized: windowChromeController.maximized
        windowEdgesFlush: windowChromeController.maximized
                          || root.visibility === Window.FullScreen
        onConfigMenuRequested: function(menuX, menuY) {
            const pos = header.mapToItem(overlays, menuX, menuY);
            domains.runConfigController.openConfigMenu(pos.x, pos.y);
        }
        onToolchainMenuRequested: function(menuX, menuY) {
            const pos = header.mapToItem(overlays, menuX, menuY);
            domains.toolchainController.openMenu(pos.x, pos.y, true);
        }
        onAppMenuRequested: function(key, menuX, menuY, items) {
            if (key === "") {
                overlays.closeAppMenu();
                return;
            }
            const pos = header.mapToItem(overlays, menuX, menuY);
            overlays.openAppMenu(pos.x, pos.y, items);
        }
        onAboutRequested: overlays.openAboutDialog()
        onManualRequested: overlays.openManualDialog()
        onMinimizeRequested: windowChromeController.minimize()
        onMaximizeRestoreRequested: windowChromeController.toggleMaximized()
        onCloseWindowRequested: windowChromeController.closeWindow()
        onMoveWindowRequested: windowChromeController.startSystemMove()
    }

    ShellWorkspaceHost {
        id: shellWorkspaceHost

        settingsController: domains.settingsController
        onShellMenuRequested: function(menuX, menuY, items) {
            const pos = shellWorkspaceHost.mapToItem(overlays, menuX, menuY);
            overlays.openAppMenu(pos.x, pos.y, items);
        }

        anchors.top: header.bottom
        anchors.bottom: statusBar.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: Theme.panelGap
        anchors.bottomMargin: Theme.panelGap
        shellController: domains.shellController
        indexController: domains.indexController
        workspaceController: domains.workspaceController
        projectHealthController: domains.projectHealthController
        projectTree: domains.projectTree
        editorController: domains.editorController
        jobsController: domains.jobsController
        activeJobController: domains.activeJobController
        runtimeController: domains.runtimeController
        debugController: domains.debugController
        gitController: domains.gitController
        coverageController: domains.coverageController
        diagnosticsController: domains.diagnosticsController
        searchController: domains.searchController
        recentWorkspacesController: domains.recentWorkspacesController
        containerController: domains.containerController
        grafanaController: domains.grafanaController
        dataSourceController: domains.dataSourceController
        embeddedController: domains.embeddedController
        remoteController: domains.remoteController
        toolchainController: domains.toolchainController
        workspaceOpen: core.workspaceRoot !== ""
        workspaceRoot: core.workspaceRoot
        workspaceName: core.workspaceName
        workspaceKind: core.workspaceKind
        workspaceBuildSystems: core.workspaceBuildSystems
        testing: core.testing
        terminalActive: core.terminalActive
        running: core.running
        logLinesModel: core.logLines
        toolsList: domains.workspaceController.toolsList
        onListDirRequested: function(path) {
            core.listDir(path);
        }
        onReadFileRequested: function(path) {
            core.readFile(path);
        }
        onOpenWorkspacePathRequested: path => core.openWorkspace(path)
        onCloseWorkspaceRequested: core.closeWorkspace()
        onToolsDetectionRequested: core.detectTools()
        // A faixa de saude do projeto: cada alvo e' um gesto de um clique.
        onHealthActionRequested: function(target) {
            if (target === "scan") {
                core.scanEnvironment();
            } else if (target === "cmakeConfigure") {
                domains.shellController.showTab("jobs");
                core.cmakeConfigure();
            } else if (target === "cargoMetadata") {
                core.cargoMetadata();
            } else if (target === "pythonEnvironment") {
                domains.shellController.showTab("jobs");
                domains.pythonController.createEnvironment();
            } else if (target === "pythonStubs") {
                domains.shellController.showTab("jobs");
                domains.pythonController.installStubs();
            } else if (target !== "") {
                domains.shellController.showTab(target);
            }
        }
        onCreateProjectRequested: function(templateId) {
            folderPickerDialog.openCreateProject(core.homeDir, templateId);
        }
        onSettingsRequested: domains.settingsController.openDialog()
        // F5: o proximo passo de um problema. "Acoes" abre o arquivo na
        // linha e pede as code actions (o Alt+Enter); o resto e' a mesma
        // acao da faixa de saude (configurar, ferramentas).
        onProblemNextStepRequested: function(kind, target, file, line, column) {
            if (kind === "codeActions") {
                domains.editorController.openDiagnostic(file, line, column);
                domains.editorController.requestCodeActions();
            } else if (kind === "health") {
                shellWorkspaceHost.healthActionRequested(target);
            }
        }
    }

    ShellStatusHost {
        id: statusBar

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: core
        shellController: domains.shellController
        indexController: domains.indexController
        activeJobController: domains.activeJobController
        lspStatusController: domains.lspStatusController
        editorController: domains.editorController
        remoteController: domains.remoteController
    }

    ShellOverlays {
        id: overlays

        // Overlay global acima do header (z=100) e de toda a workspace. O z
        // interno de um popup não escapa do stacking context do pai.
        z: 1000
        appMenuPassThroughTop: header.height
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
        toolWindows: shellWorkspaceHost.toolWindows
        libraryController: domains.libraryController
        dataSourceController: domains.dataSourceController
        remoteController: domains.remoteController
        grafanaController: domains.grafanaController
        embeddedController: domains.embeddedController
        setupController: domains.setupController
        containerController: domains.containerController
        configActionController: domains.configActionController
        toolchainController: domains.toolchainController
        pythonController: domains.pythonController
        onAppMenuActionRequested: function(action) {
            header.executeMenuAction(action);
        }
        onAppMenuDismissed: header.closeAppMenu()
    }

    KvFloatingLayer {
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
