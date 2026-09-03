import QtQuick

// Dono dos controllers de dominio e do IPC dos dois sentidos (EventRouter +
// RequestRouter). Existe porque o composition root cresce LINEARMENTE com o
// numero de dominios quando nao ha registro/contribuicao — e registro dinamico e
// a maquina que este projeto recusou (ver docs/arquitetura/27-modulos-por-dominio.md).
//
// A saida e dividir a COMPOSICAO por area: aqui os dominios, no Main.qml a janela
// e os hosts visuais. Assim o que cresce quando um dominio novo nasce e a
// CONTAGEM de arquivos, nao o tamanho de nenhum (ARCHITECTURE.md §4 regra 8).
//
// Nao ha logica aqui: so instanciacao e fiacao. Cruzamento entre dominios que
// dependa dos HOSTS entra por propriedade declarada, nunca por id global.
Item {
    id: root

    property var coreClient: null
    property var workspaceHost: null
    property var shellOverlays: null
    property var folderPicker: null
    property var workspaceUiResetter: null
    // Dimensoes da janela, para quem posiciona popup pela area util.
    property real hostWidth: 0
    property real hostHeight: 0

    readonly property alias workspaceController: workspaceController
    readonly property alias recentWorkspacesController: recentWorkspacesController
    readonly property alias projectHealthController: projectHealthController
    readonly property alias shellController: shellController
    readonly property alias jobsController: jobsController
    readonly property alias diagnosticsController: diagnosticsController
    readonly property alias settingsController: settingsController
    readonly property alias configActionController: configActionController
    readonly property alias toolchainController: toolchainController
    readonly property alias libraryController: libraryController
    readonly property alias runtimeController: runtimeController
    readonly property alias runConfigController: runConfigController
    readonly property alias debugController: debugController
    readonly property alias gitController: gitController
    readonly property alias searchController: searchController
    readonly property alias searchEverywhereController: searchEverywhereController
    readonly property alias commandDispatcher: commandDispatcher
    readonly property alias editorController: editorController
    readonly property alias projectTree: projectTree

    visible: false

    WorkspaceController {
        id: workspaceController

        workspaceRoot: root.coreClient.workspaceRoot
        onClearWorkspaceUiRequested: root.workspaceUiResetter.clear()
    }

    RecentWorkspacesController {
        id: recentWorkspacesController

        onListRequested: root.coreClient.listRecentWorkspaces()
        onOpenRequested: function(rootPath) {
            root.coreClient.openWorkspace(rootPath);
        }
        onPinRequested: function(rootPath, pinned) {
            root.coreClient.pinRecentWorkspace(rootPath, pinned);
        }
        onRemoveRequested: function(rootPath) {
            root.coreClient.removeRecentWorkspace(rootPath);
        }
        onClearRequested: root.coreClient.clearRecentWorkspaces()
    }

    ProjectHealthController {
        id: projectHealthController

        workspaceRoot: root.coreClient.workspaceRoot
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        toolsList: workspaceController.toolsList
        scanningEnvironment: root.coreClient.scanningEnvironment
        onAutoConfigureRequested: root.coreClient.cmakeConfigure()
    }

    ShellController {
        id: shellController

        workspaceRoot: root.coreClient.workspaceRoot
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        homeDir: root.coreClient.homeDir
        toolsCount: workspaceController.toolsList.length
        onFolderOpenRequested: function(path) {
            root.folderPicker.open(path);
        }
        onToolsDetectionRequested: root.coreClient.detectTools()
        onLayoutSaveRequested: function(values) {
            settingsController.setGlobal(values);
        }
    }

    JobsController {
        id: jobsController

        workspaceRoot: root.coreClient.workspaceRoot
        building: root.coreClient.building
        testing: root.coreClient.testing
        analyzing: root.coreClient.analyzing
        onRunBuildRequested: buildSystem => root.coreClient.runBuild(buildSystem)
        onRunTestsRequested: buildSystem => root.coreClient.runTests("", buildSystem)
        onRunQualityRequested: root.coreClient.runQuality()
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
    }

    DiagnosticsController {
        id: diagnosticsController
    }

    SettingsController {
        id: settingsController

        onGetRequested: root.coreClient.settingsGet()
        onSetRequested: function(scope, values) {
            root.coreClient.settingsSet(scope, values);
        }
    }

    // Toolchain (roadmap 30, etapa 5): qual executavel cumpre cada papel neste
    // projeto. Sem escolha, tudo e automatico e o PATH continua decidindo.
    ToolchainController {
        id: toolchainController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // Bibliotecas C/C++ (roadmaps/35, etapa 19/20): o catalogo curado. Nao
    // escreve arquivo — produz plano, e quem escreve e o configaction.
    LibraryController {
        id: libraryController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // Configuration Actions (roadmap 30, etapa 2): dominio proprio, nasce nas
    // quatro camadas. Quem abre o dialogo e a paleta/atalho; o host visual dele
    // e o ShellOverlays, como os demais dialogos.
    ConfigActionController {
        id: configActionController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // Configuracoes salvas saem do RuntimeController: "guardar como rodar um
    // programa" nao e "manter uma sessao de terminal".
    RunConfigController {
        id: runConfigController

        onRunConfigDialogOpenRequested: function(name, command) {
            root.shellOverlays.openRunConfigDialogWith(name, command);
        }
    }

    RuntimeController {
        id: runtimeController

        workspaceRoot: root.coreClient.workspaceRoot
        running: root.coreClient.running
        terminalActive: root.coreClient.terminalActive
        terminalPanelVisible: shellController.showBottomPanel
                              && shellController.bottomTab === "terminal"
        // Pedido ao core mora no RuntimeRequestRouter. Aqui fica so fiacao de
        // controller para HOST/shell, que nao e IPC.
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onFocusTerminalInputRequested: root.workspaceHost.focusTerminalInput()
        onClearTerminalInputRequested: root.workspaceHost.clearTerminalInput()
        onClearRunInputRequested: root.workspaceHost.clearRunInput()
    }

    DebugController {
        id: debugController

        workspaceRoot: root.coreClient.workspaceRoot
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

        workspaceRoot: root.coreClient.workspaceRoot
        // Toda a fiacao de pedido — inclusive a guarda de arquivo sujo — mora
        // no GitRequestRouter.
    }

    // Busca e substituicao NO PROJETO (painel de baixo). A operacao destrutiva
    // mora aqui; a caixa modal e outro dono.
    SearchController {
        id: searchController

        workspaceRoot: root.coreClient.workspaceRoot
        // Pedido ao core (inclusive a guarda de replace) mora no
        // SearchRequestRouter.
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onFocusSearchInputRequested: root.workspaceHost.focusSearchInput()
        onFocusReplaceInputRequested: root.workspaceHost.focusSearchReplaceInput()
    }

    // Search Everywhere: a caixa modal que acha arquivo, simbolo e comando.
    // Separada em 2026-09-02 — efemera, sem escrita, teclado-primeiro.
    SearchEverywhereController {
        id: searchEverywhereController

        workspaceRoot: root.coreClient.workspaceRoot
        recentFiles: editorController.recentFiles
        hasActiveEditorFile: editorController.currentTab >= 0
        onResetAndFocusEverywhereRequested: root.shellOverlays.resetSearchEverywhereAndFocus()
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

        coreClient: root.coreClient
        debugController: debugController
        editorController: editorController
        gitController: gitController
        jobsController: jobsController
        projectTree: projectTree
        runtimeController: runtimeController
        settingsController: settingsController
        searchController: searchController
        searchEverywhereController: searchEverywhereController
        configActionController: configActionController
        libraryController: libraryController
        onOpenWorkspaceRequested: shellController.requestOpenFolder()
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
    }

    EditorController {
        id: editorController

        workspaceRoot: root.coreClient.workspaceRoot
        editorSurface: root.workspaceHost.editorSurface
        diagnosticsController: diagnosticsController
        settingsController: settingsController
        // Pedido ao core mora no EditorRequestRouter. O que fica aqui e fiacao
        // de controller para HOST — nao e IPC, e so o Main.qml enxerga os dois.
        onGoToLineDialogOpenRequested: function(prefill) {
            root.workspaceHost.openGoToLineDialog(prefill);
        }
        onFindBarOpenRequested: root.workspaceHost.focusFindBar()
    }

    // O dialogo de rename e aberto pela CAMADA DE LINGUAGEM, nao pelo editor:
    // o gesto nasce de um `textDocument/rename`. Escutar no lugar errado nao
    // quebra build — deixa de funcionar em silencio (ARCHITECTURE.md §8).
    Connections {
        target: editorController.language

        function onRenameDialogOpenRequested(currentName) {
            root.workspaceHost.openRenameDialogWithName(currentName);
        }
    }

    ProjectTreeController {
        id: projectTree

        workspaceRoot: root.coreClient.workspaceRoot
        hostWidth: root.hostWidth
        hostHeight: root.hostHeight
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
        onCreateDialogFocusRequested: root.workspaceHost.focusCreateDialog()
        onEntryRenameDialogOpenRequested: function(name) {
            root.shellOverlays.openEntryRenameWithName(name);
        }
        onFocusEditorRequested: editorController.focusEditor()
    }

    AppRouters {
        domains: root
    }
}
