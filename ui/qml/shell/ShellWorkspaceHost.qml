import QtQuick
import KineinVectis

Item {
    id: root

    property var shellController
    property var workspaceController
    property var projectHealthController
    property var projectTree
    property var editorController
    property var jobsController
    property var runtimeController
    property var debugController
    property var gitController
    property var diagnosticsController
    property var searchController
    property var recentWorkspacesController
    property var containerController
    property var grafanaController
    property var dataSourceController
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

    function clearRunInput() {
        bottomPanel.clearRunInput();
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

    ShellLayout {
        anchors.fill: parent

        SideRail {
            id: sideBar

            height: parent.height
            workspaceOpen: root.workspaceOpen
            explorerActive: root.shellController.effectiveShowExplorer
            searchActive: root.shellController.showBottomPanel
                          && root.shellController.bottomTab === "search"
            gitActive: root.shellController.showBottomPanel
                       && root.shellController.bottomTab === "git"
            buildActive: root.shellController.showBottomPanel
                         && (root.shellController.bottomTab === "build"
                             || root.shellController.bottomTab === "jobs")
            debugActive: root.shellController.showBottomPanel
                         && root.shellController.bottomTab === "debug"
            toolsActive: root.shellController.showBottomPanel
                         && root.shellController.bottomTab === "tools"
            onExplorerToggled: root.shellController.toggleExplorer()
            onSearchRequested: root.searchController.openSearchPanel()
            onGitRequested: root.shellController.toggleBottomTab("git")
            onBuildRequested: root.shellController.toggleBottomTab("build")
            onDebugRequested: root.shellController.toggleBottomTab("debug")
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

        ProjectExplorer {
            id: explorerPanel

            width: visible ? root.shellController.explorerWidth : 0
            height: parent.height
            visible: root.workspaceOpen
                     && root.shellController.effectiveShowExplorer
            workspaceName: root.workspaceName
            workspaceKindLabel: root.shellController.kindLabel(
                                    root.workspaceKind,
                                    root.workspaceBuildSystems)
            selectedPath: root.projectTree.selectedPath
            entriesModel: root.projectTree.entriesModel
            gitKinds: root.gitController.gitKinds
            gitRevision: root.gitController.revision
            onCreateFileRequested: root.projectTree.openCreateDialog("file")
            onCreateDirectoryRequested: root.projectTree.openCreateDialog("directory")
            onRefreshRequested: root.listDirRequested(root.workspaceRoot)
            onCloseRequested: root.closeWorkspaceRequested()
            onEntrySelected: function(path, kind) {
                root.projectTree.selectEntry(path, kind);
            }
            onDirectoryToggleRequested: function(path, index, expanded) {
                root.projectTree.toggleDirectory(path, index, expanded);
            }
            onFileOpenRequested: function(path) {
                root.readFileRequested(path);
            }
            onScriptRunRequested: function(path) {
                root.projectTree.runScript(path);
            }
            onContextMenuRequested: function(path, kind, name, sceneX, sceneY) {
                root.projectTree.openEntryMenu(path, kind, name, sceneX, sceneY);
            }
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
                recentWorkspaces: root.recentWorkspacesController.workspaces
                recentWorkspacesError: root.recentWorkspacesController.errorText
                scanning: root.scanningEnvironment
                onOpenWorkspaceRequested: root.shellController.requestOpenFolder()
                onRecentWorkspaceOpenRequested: function(rootPath) {
                    root.recentWorkspacesController.openWorkspace(rootPath);
                }
                onRecentWorkspacePinRequested: function(rootPath) {
                    root.recentWorkspacesController.togglePinned(rootPath);
                }
                onRecentWorkspaceRemoveRequested: function(rootPath) {
                    root.recentWorkspacesController.removeWorkspace(rootPath);
                }
                onRecentWorkspacesClearRequested: root.recentWorkspacesController.clearAll()
                onNewProjectRequested: function(templateId) {
                    root.createProjectRequested(templateId);
                }
                onSettingsRequested: root.settingsRequested()
                onDetectToolsRequested: root.toolsDetectionRequested()
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
                shellController: root.shellController
                debugController: root.debugController
                gitController: root.gitController
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
                buildOutputModel: root.jobsController.buildOutputModel
                jobsModel: root.jobsController.jobsModel
                testModel: root.jobsController.testModel
                testOutputModel: root.jobsController.testOutputModel
                testSummary: root.jobsController.testSummary
                testing: root.testing
                problemsModel: root.jobsController.problemsModel
                terminalRender: root.runtimeController.terminalRender
                terminalActive: root.terminalActive
                workspaceAvailable: root.workspaceOpen
                runModel: root.runtimeController.runModel
                debugController: root.debugController
                gitChangesModel: root.gitController.changesModel
                gitRepo: root.gitController.repo
                gitStagedCount: root.gitController.stagedCount
                gitErrorText: root.gitController.lastMutationError
                gitHistoryModel: root.gitController.historyModel
                gitHistoryVisible: root.gitController.historyVisible
                gitHistoryLoading: root.gitController.historyLoading
                gitBranchLabel: root.gitController.branchLabel
                gitBranchesModel: root.gitController.branchesModel
                gitBranchMenuVisible: root.gitController.branchMenuVisible
                gitRemoteOperationRunning: root.gitController.remoteOperationRunning
                running: root.running
                terminalSession: root.runtimeController.terminalSession
                terminalsModel: root.runtimeController.terminalsModel
                activeTerminalId: root.runtimeController.activeTerminalId
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
                onTabRequested: function(tab) {
                    root.shellController.toggleBottomTab(tab);
                }
                onTerminalSessionRequested: function(session) {
                    root.runtimeController.setTerminalSession(session);
                }
                onClearSessionRequested: root.runtimeController.clearActiveSession()
                onRefreshToolsRequested: root.toolsDetectionRequested()
                onProblemOpenRequested: function(file, line, column) {
                    root.editorController.openDiagnostic(file, line, column);
                }
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
                onRunInputSubmitted: function(text) {
                    root.runtimeController.submitRunInput(text);
                }
                onGitStageToggleRequested: function(index) {
                    root.gitController.toggleStaged(index);
                }
                onGitChangesViewRequested: root.gitController.showChanges()
                onGitHistoryViewRequested: root.gitController.openHistory()
                onGitHistoryRefreshRequested: root.gitController.refreshHistory()
                onGitCommitActivated: function(sha, shortSha, summary) {
                    root.gitController.openCommitDiff(sha, shortSha, summary);
                }
                onGitBranchMenuRequested: root.gitController.openBranchMenu()
                onGitBranchCheckoutRequested: function(branch) {
                    root.gitController.checkoutBranch(branch);
                }
                onGitBranchCreateRequested: function(name) {
                    root.gitController.createBranch(name);
                }
                onGitRemoteRequested: function(operation) {
                    root.gitController.startRemote(operation);
                }
                onGitStashRequested: function(action) {
                    root.gitController.stashRequested(action, "");
                }
                onGitDiffRequested: function(absPath) {
                    root.gitController.openDiffDialog(absPath);
                }
                onGitDiscardRequested: function(index) {
                    root.gitController.openDiscardDialog(index);
                }
                onGitOpenRequested: function(absPath) {
                    root.readFileRequested(absPath);
                }
                onGitCommitRequested: function(message) {
                    root.gitController.commit(message);
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
