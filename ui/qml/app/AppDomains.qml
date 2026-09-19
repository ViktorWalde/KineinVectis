import QtQuick

// Dono dos controllers de dominio e do IPC dos dois sentidos (EventRouter +
// RequestRouter). Existe porque o composition root cresce LINEARMENTE com o
// numero de dominios quando nao ha registro/contribuicao — e registro dinamico e
// a maquina que este projeto recusou (ver DocsPublic/arquitetura/27-modulos-por-dominio.md).
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
    readonly property alias toolchainController: environment.toolchainController
    readonly property alias dataSourceController: environment.dataSourceController
    readonly property alias grafanaController: environment.grafanaController
    readonly property alias embeddedController: environment.embeddedController
    readonly property alias setupController: environment.setupController
    readonly property alias containerController: environment.containerController
    readonly property alias remoteController: environment.remoteController
    readonly property alias libraryController: environment.libraryController
    readonly property alias runtimeController: runtimeController
    readonly property alias runConfigController: runConfigController
    readonly property alias debugController: debugController
    readonly property alias gitController: gitController
    readonly property alias coverageController: coverageController
    readonly property alias activeJobController: activeJobController
    readonly property alias lspStatusController: lspStatusController
    readonly property alias searchController: searchController
    readonly property alias searchEverywhereController: searchEverywhereController
    readonly property alias indexController: indexController
    readonly property alias pythonController: pythonController
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
        onOpenRequested: rootPath => root.coreClient.openWorkspace(rootPath)
        onPinRequested: (rootPath, pinned) => root.coreClient.pinRecentWorkspace(rootPath, pinned)
        onRemoveRequested: rootPath => root.coreClient.removeRecentWorkspace(rootPath)
        onClearRequested: root.coreClient.clearRecentWorkspaces()
    }

    ProjectHealthController {
        id: projectHealthController

        workspaceRoot: root.coreClient.workspaceRoot
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        toolsList: workspaceController.toolsList
        scanningEnvironment: root.coreClient.scanningEnvironment
        pythonNeedsEnvironment: pythonController.needsEnvironment
        pythonCreating: pythonController.creating
        pythonMessage: pythonController.bannerMessage()
        pythonActionLabel: pythonController.actionLabel()
        pythonNeedsStubs: pythonController.needsStubs
        pythonInstallingStubs: pythonController.installingStubs
        pythonStubsMessage: pythonController.stubsMessage()
        onAutoConfigureRequested: root.coreClient.cmakeConfigure()
    }

    ShellController {
        id: shellController

        workspaceRoot: root.coreClient.workspaceRoot
        workspaceKind: root.coreClient.workspaceKind
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
        homeDir: root.coreClient.homeDir
        toolsCount: workspaceController.toolsList.length
        onFolderOpenRequested: path => root.folderPicker.open(path)
        onToolsDetectionRequested: root.coreClient.detectTools()
        onLayoutSaveRequested: values => settingsController.setGlobal(values)
    }

    JobsController {
        id: jobsController

        workspaceRoot: root.coreClient.workspaceRoot
        building: root.coreClient.building
        testing: root.coreClient.testing
        analyzing: root.coreClient.analyzing
        onRunBuildRequested: buildSystem => root.coreClient.runBuild(buildSystem)
        onRunTestsRequested: buildSystem => root.coreClient.runTests("", buildSystem)
        onRunOneTestRequested: (testId, buildSystem) => root.coreClient.runTests("", buildSystem, testId)
        onDiscoverTestsRequested: buildSystem => root.coreClient.discoverTests(buildSystem)
        onRunQualityRequested: root.coreClient.runQuality()
        onRunCoverageRequested: root.coreClient.coverageRun()
        onShowTabRequested: tab => shellController.showTab(tab)
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

    // Os donos do "Ambiente do projeto" — toolchain, bibliotecas, banco,
    // observabilidade, embarcados, containers, instalar ferramentas — moram
    // juntos em AppEnvironmentDomains: sairam daqui em 2026-09-12, quando o
    // dominio `container` nasceu e este arquivo estava em 390/400. Corte por
    // RESPONSABILIDADE (a Frente 1 do arquitetura/27: <X>Domain na UI), nao
    // por tamanho: os seis tem a mesma forma e o menu ja' os agrupa.
    AppEnvironmentDomains {
        id: environment

        coreClient: root.coreClient
        folderPicker: root.folderPicker
    }

    // O indice do projeto INTEIRO (pilar 0 do roadmaps/42, 2026-09-12): os
    // totais que o core manda ao construir; a busca por nome mora no
    // SearchEverywhere.
    IndexController {
        id: indexController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // O ambiente Python do projeto (bloco B do roadmaps/41): o interpretador
    // que o core resolveu e o botao de criar o .venv. So' pergunta em projeto
    // Python.
    PythonController {
        id: pythonController

        workspaceRoot: root.coreClient.workspaceRoot
        workspaceBuildSystems: root.coreClient.workspaceBuildSystems
    }

    // Configuration Actions (roadmap 30, etapa 2): dominio proprio, nasce nas
    // quatro camadas. Quem abre o dialogo e a paleta/atalho; o host visual dele
    // e o ShellOverlays, como os demais dialogos.
    ConfigActionController {
        id: configActionController

        workspaceRoot: root.coreClient.workspaceRoot
        // A lista e' UMA: acoes e bibliotecas lado a lado. Ver o cabecalho de
        // `libraryController` no ConfigActionController.
        libraryController: environment.libraryController
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
        terminalPanelVisible: shellController.tabActive("terminal")
        // A porta escolhida no painel de Embarcados e' o `device` do Executar
        // (MicroPython na placa). Composicao, nao IPC: o controller so' le.
        serialDevice: environment.embeddedController.selectedPort
        // Pedido ao core mora no RuntimeRequestRouter. Aqui fica so fiacao de
        // controller para HOST/shell, que nao e IPC.
        onShowTabRequested: tab => shellController.showTab(tab)
        onFocusTerminalInputRequested: root.workspaceHost.focusTerminalInput()
        onClearTerminalInputRequested: root.workspaceHost.clearTerminalInput()
    }

    // Gravar (E4) e' configuracao de execucao: rodar agora e' o run.start de
    // um comando explicito; salvar vira a configuracao ativa. Fiacao entre o
    // painel de Embarcados e os donos de execucao — composicao, nao IPC.
    Connections {
        target: environment.embeddedController.flash

        function onRunRequested(command) {
            runtimeController.startRun(command);
        }

        function onSaveRequested(name, command) {
            runConfigController.saveRunConfigRequested("", name, command);
        }
    }

    // Arquivos na placa (C2): "enviar o arquivo aberto" pergunta ao editor
    // qual e' NO CLIQUE; o baixado abre no editor. Fiacao entre dois donos.
    Connections {
        target: environment.embeddedController.files

        function onUploadCurrentRequested() {
            environment.embeddedController.files.upload(editorController.currentFilePath(), "");
        }

        function onOpenLocalRequested(local) {
            editorController.openDiagnostic(local, 1, 1);
        }
    }

    // A cobertura dos testes (D8, 2026-09-17): o resumo e as linhas do
    // arquivo ativo para a calha do editor.
    CoverageController {
        id: coverageController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // F2 da Etapa 2: o job em curso e os servidores de linguagem (status bar).
    ActiveJobController {
        id: activeJobController

        jobsModel: jobsController.jobsModel
        onCancelRequested: jobId => root.coreClient.cancelJob(jobId)
    }

    LspStatusController {
        id: lspStatusController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    DebugController {
        id: debugController

        workspaceRoot: root.coreClient.workspaceRoot
        // Pedido ao core mora no DebugRequestRouter. Aqui fica so fiacao de
        // controller para HOST/editor, que nao e IPC.
        onShowTabRequested: tab => shellController.showTab(tab)
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
        onShowTabRequested: tab => shellController.showTab(tab)
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

    // A medicao headless (KINEIN_STARTUP_COMMANDS) passa pelo mesmo dispatcher.
    StartupCommands {
        coreClient: root.coreClient
        commandDispatcher: commandDispatcher
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
        libraryController: environment.libraryController
        dataSourceController: environment.dataSourceController
        grafanaController: environment.grafanaController
        embeddedController: environment.embeddedController
        setupController: environment.setupController
        containerController: environment.containerController
        remoteController: environment.remoteController
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
        onDebugScriptRequested: function(path) {
            debugController.startDebug(path);
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
