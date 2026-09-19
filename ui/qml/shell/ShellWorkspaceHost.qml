import QtQuick
import KineinVectis

Item {
    id: root

    property var shellController
    property var workspaceController
    property var projectHealthController
    property var projectTree
    property var editorController
    property var indexController: null
    property var jobsController
    property var activeJobController: null
    property var runtimeController
    property var debugController
    property var gitController
    property var coverageController
    property var diagnosticsController
    property var searchController
    property var recentWorkspacesController
    property var containerController
    property var grafanaController
    property var dataSourceController
    property var embeddedController: null
    property alias editorSurface: editorPaneHost.editorSurface
    property bool workspaceOpen: false
    property string workspaceRoot: ""
    property string workspaceName: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property bool testing: false
    property bool terminalActive: false
    property bool running: false
    property var logLinesModel
    property var toolsList: []
    property bool scanningEnvironment: false

    signal listDirRequested(string path)
    signal readFileRequested(string path)
    signal closeWorkspaceRequested()
    signal toolsDetectionRequested()
    // A acao da faixa de saude vai INTEIRA para quem tem os controllers
    // (Main.qml): "scan", "cmakeConfigure", "cargoMetadata",
    // "pythonEnvironment", ou o nome de uma aba.
    signal healthActionRequested(string target)
    signal problemNextStepRequested(string kind, string target, string file, int line, int column)
    signal createProjectRequested(string templateId)
    signal settingsRequested()

    onWidthChanged: {
        if (root.shellController) root.shellController.updateViewport(width, height);
    }
    onHeightChanged: {
        if (root.shellController) root.shellController.updateViewport(width, height);
    }
    Component.onCompleted: root.shellController.updateViewport(width, height)

    function focusSearchInput() {
        bottomPanel.focusSearchInput();
    }

    function focusSearchReplaceInput() {
        bottomPanel.focusSearchReplaceInput();
    }

    function clearSearchInput() {
        bottomPanel.clearSearchInput();
    }

    function focusTerminalInput() {
        bottomPanel.focusTerminalInput();
    }

    function clearTerminalInput() {
        bottomPanel.clearTerminalInput();
    }


    function focusCreateDialog() {
        editorPaneHost.focusCreateDialog();
    }

    function openRenameDialogWithName(name) {
        editorPaneHost.openRenameDialogWithName(name);
    }

    function openGoToLineDialog(prefill) {
        editorPaneHost.openGoToLineDialog(prefill);
    }

    function focusFindBar() {
        editorPaneHost.focusFindBar();
    }

    function focusSymbols(query) {
        editorPaneHost.focusSymbols(query);
    }

    ShellLayout {
        anchors.fill: parent

        SideRail {
            id: sideBar

            height: parent.height
            expanded: root.shellController.railExpanded
            onExpandedToggled: root.shellController.toggleRail()
            workspaceOpen: root.workspaceOpen
            explorerActive: root.shellController.effectiveShowExplorer
            gitActive: root.shellController.gitWindowVisible
            embeddedActive: root.embeddedController !== undefined && root.embeddedController !== null
                            && root.embeddedController.panelVisible
            toolsActive: root.shellController.showBottomPanel
                         && root.shellController.bottomTab === "tools"
            onExplorerToggled: root.shellController.toggleExplorer()
            onGitRequested: root.shellController.toggleBottomTab("git")
            onEmbeddedRequested: root.embeddedController.open()
            onToolsRequested: root.shellController.toggleBottomTab("tools")
            containersActive: root.containerController !== undefined && root.containerController !== null
                              && root.containerController.panelVisible
            observabilityActive: root.grafanaController !== undefined && root.grafanaController !== null
                                 && root.grafanaController.panelVisible
            databaseActive: root.dataSourceController !== undefined && root.dataSourceController !== null
                            && root.dataSourceController.panelVisible
            onDatabaseRequested: root.dataSourceController.open()
            onContainersRequested: root.containerController.open()
            onObservabilityRequested: root.grafanaController.open()
        }

        // O slot a esquerda: o explorer OU a janela do Git (E3-3).
        ShellLeftWindowHost {
            id: explorerPanel

            width: visible ? root.shellController.explorerWidth : 0
            height: parent.height
            visible: root.workspaceOpen
                     && (root.shellController.effectiveShowExplorer
                         || root.shellController.gitWindowVisible)
            shellController: root.shellController
            projectTree: root.projectTree
            gitController: root.gitController
            workspaceName: root.workspaceName
            workspaceRoot: root.workspaceRoot
            onListDirRequested: function(path) { root.listDirRequested(path); }
            onReadFileRequested: function(path) { root.readFileRequested(path); }
            onCloseWorkspaceRequested: root.closeWorkspaceRequested()
        }

        Column {
            id: centerColumn

            width: visible
                   ? Math.max(0, parent.width - sideBar.width - Theme.panelGap
                              - (explorerPanel.visible
                                 ? explorerPanel.width + Theme.panelGap : 0))
                   : 0
            height: parent.height
            spacing: Theme.panelGap

            ProjectHealthBanner {
                id: healthBanner

                width: parent.width
                active: root.projectHealthController.active
                status: root.projectHealthController.status
                message: root.projectHealthController.message
                actionLabel: root.projectHealthController.actionLabel
                onActionRequested: root.healthActionRequested(root.projectHealthController.actionTarget)
                onDismissRequested: root.projectHealthController.dismiss()
            }

            StartScreen {
                width: parent.width
                height: visible
                        ? parent.height - (bottomPanel.visible
                          ? bottomPanel.height + Theme.panelGap : 0) : 0
                visible: !root.workspaceOpen
                tools: root.toolsList
                recentWorkspacesController: root.recentWorkspacesController
                scanning: root.scanningEnvironment
                onOpenWorkspaceRequested: root.shellController.requestOpenFolder()
                onNewProjectRequested: function(templateId) {
                    root.createProjectRequested(templateId);
                }
                onSettingsRequested: root.settingsRequested()
                onDetectToolsRequested: root.toolsDetectionRequested()
                onToolsPanelRequested: root.shellController.showTab("tools")
            }

            ShellEditorHost {
                id: editorPaneHost

                width: parent.width
                visible: root.workspaceOpen
                height: visible ? parent.height
                        - (healthBanner.visible
                        ? healthBanner.height + Theme.panelGap : 0)
                        - (bottomPanel.visible
                        ? bottomPanel.height + Theme.panelGap : 0) : 0

                workspaceOpen: root.workspaceOpen
                editorController: root.editorController
                indexController: root.indexController
                shellController: root.shellController
                debugController: root.debugController
                gitController: root.gitController
                coverageController: root.coverageController
                diagnosticsController: root.diagnosticsController
                projectTree: root.projectTree
            }

            BottomPanelHost {
                id: bottomPanel

                width: parent.width
                height: root.shellController.bottomPanelHeight
                open: root.shellController.showBottomPanel
                activeTab: root.shellController.bottomTab
                problemCount: root.jobsController.problemsModel.count
                gitController: root.gitController
                testsBadge: root.jobsController.testsBadge
                testsOk: root.jobsController.testsFailed === 0
                jobsRunning: root.activeJobController ? root.activeJobController.runningCount : 0
                buildOutputModel: root.jobsController.buildOutputModel
                jobsModel: root.jobsController.jobsModel
                jobsController: root.jobsController
                testing: root.testing
                problemsModel: root.jobsController.problemsModel
                terminalRender: root.runtimeController.terminalRender
                terminalActive: root.terminalActive
                workspaceAvailable: root.workspaceOpen
                debugController: root.debugController
                running: root.running
                terminalsModel: root.runtimeController.terminalsModel
                activeTerminalId: root.runtimeController.activeTerminalId
                runTerminalId: root.runtimeController.runTerminalId
                searchModel: root.searchController.searchModel
                searchCaseSensitive: root.searchController.caseSensitive
                searching: root.searchController.searching
                searchTruncated: root.searchController.searchTruncated
                searchReplaceMode: root.searchController.replaceMode
                searchReplacing: root.searchController.replacing
                searchReplaceError: root.searchController.replaceError
                searchReplaceSummary: root.searchController.replaceSummary
                logLinesModel: root.logLinesModel
                toolsList: root.workspaceController.toolsList
                onHideRequested: root.shellController.showBottomPanel = false
                onTabRequested: function(tab) {
                    // A aba Terminal abre uma sessao se nao ha' nenhuma: o
                    // dono disso e' o RuntimeController.
                    if (tab === "terminal" && !root.shellController.tabActive("terminal")) {
                        root.runtimeController.openTerminalPanel();
                        return;
                    }
                    root.shellController.toggleBottomTab(tab);
                }
                onRefreshToolsRequested: root.toolsDetectionRequested()
                onProblemOpenRequested: function(file, line, column) {
                    root.editorController.openDiagnostic(file, line, column);
                }
                onProblemNextStepRequested: (kind, target, file, line, column) =>
                    root.problemNextStepRequested(kind, target, file, line, column)
                onTerminalOpenRequested: root.runtimeController.openTerminalPanel()
                onTerminalKeyPressed: function(data) {
                    root.runtimeController.sendTerminalKey(data);
                }
                onTerminalResizeRequested: function(cols, rows) {
                    root.runtimeController.resizeTerminal(cols, rows);
                }
                onTerminalScrollRequested: function(offset) {
                    root.runtimeController.scrollTerminal(offset);
                }
                onTerminalWheelRequested: function(col, row, lines, modifiers) {
                    root.runtimeController.wheelTerminal(col, row, lines,
                                                         modifiers);
                }
                onTerminalSelectRequested: function(id) {
                    root.runtimeController.selectTerminal(id);
                }
                onTerminalNewRequested: root.runtimeController.newTerminal()
                onTerminalCloseTabRequested: function(id) {
                    root.runtimeController.closeTerminal(id);
                }
                onSearchRequested: function(query) {
                    root.searchController.runSearch(query);
                }
                onSearchCaseSensitivityToggleRequested: function(query) {
                    root.searchController.toggleCaseAndRun(query);
                }
                onSearchResultOpenRequested: function(path, line, column) {
                    root.editorController.openDiagnostic(path, line, column);
                }
                onSearchReplaceRequested: function(query, replacement) {
                    root.searchController.runReplace(query, replacement);
                }
            }
        }

    }

    // Alcas de redimensionamento em overlay sobre os vaos do layout
    // (KVSplitter minimo da fatia C1; limites da spec no ShellController).
    PanelSplitter {
        visible: explorerPanel.visible
        x: explorerPanel.x + explorerPanel.width
        width: Theme.panelGap
        height: parent.height
        onDragged: function(delta) {
            root.shellController.resizeExplorer(delta);
        }
    }


    PanelSplitter {
        visible: bottomPanel.visible
        horizontal: true
        x: centerColumn.x
        y: centerColumn.y + bottomPanel.y - Theme.panelGap
        width: centerColumn.width
        height: Theme.panelGap
        onDragged: function(delta) {
            root.shellController.resizeBottomPanel(-delta);
        }
    }
}
