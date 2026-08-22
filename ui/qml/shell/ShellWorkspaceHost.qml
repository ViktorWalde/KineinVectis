import QtQuick
import KineinVectis

Item {
    id: root

    property var shellController
    property var workspaceController
    property var projectHealthController
    property var projectTree
    property var projectTreeGestures
    property var markdownMode
    property var editorController
    property var jobsController
    property var compileContextController
    property var runtimeController
    property var debugController
    property var gitController
    property var diagnosticsController
    property var searchController
    property var recentWorkspacesController
    property alias editorSurface: editorPane.editorSurface
    property bool workspaceOpen: false
    property string workspaceRoot: ""
    property string workspaceName: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property bool terminalActive: false
    property var logLinesModel
    property var toolsList: []
    property bool scanningEnvironment: false

    signal listDirRequested(string path)
    signal readFileRequested(string path)
    signal closeWorkspaceRequested()
    signal toolsDetectionRequested()
    signal environmentScanRequested()
    signal cmakeConfigureRequested()
    signal cargoMetadataRequested()
    signal createProjectRequested(string templateId)
    signal settingsRequested()

    onWidthChanged: {
        if (root.shellController) root.shellController.updateViewport(width, height);
    }
    onHeightChanged: {
        if (root.shellController) root.shellController.updateViewport(width, height);
    }
    Component.onCompleted: root.shellController.updateViewport(width, height)

    // Os args de tab/revisao existem so para os bindings reavaliarem
    // quando a aba ativa ou os breakpoints mudam (funcoes nao notificam).
    function currentFileBreakpoints(currentTab, revision) {
        return debugController.breakpointLinesFor(
            editorController.currentFilePath());
    }

    function currentFileExecutionLine(currentTab, stoppedFile, stoppedLine) {
        if (stoppedFile === ""
                || stoppedFile !== editorController.currentFilePath()) {
            return 0;
        }
        return stoppedLine;
    }

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
        editorPane.focusCreateDialog();
    }

    function openRenameDialogWithName(name) {
        editorPane.openRenameDialogWithName(name);
    }

    function openGoToLineDialog(prefill) {
        editorPane.openGoToLineDialog(prefill);
    }

    function focusFindBar() {
        editorPane.focusFindBar();
    }

    function focusProjectTree() {
        explorerPanel.focusTree();
    }

    ShellLayout {
        anchors.fill: parent

        SideRail {
            id: sideBar

            height: parent.height
            workspaceOpen: root.workspaceOpen
            explorerActive: root.shellController.effectiveShowExplorer
            bottomOpen: root.shellController.showBottomPanel
            bottomTab: root.shellController.bottomTab
            onExplorerToggled: root.shellController.toggleExplorer()
            onSearchRequested: root.searchController.openSearchPanel()
            onTabToggleRequested: function(tab) {
                root.shellController.toggleBottomTab(tab);
            }
            // Rail JetBrains: clicar no ativo ESCONDE; senao abre materializando
            // uma sessao se nao houver (openTerminalPanel, mesmo do Alt+F12).
            onTerminalRequested: {
                if (root.shellController.showBottomPanel
                        && root.shellController.bottomTab === "terminal") {
                    root.shellController.toggleBottomTab("terminal");
                } else {
                    root.runtimeController.openTerminalPanel();
                }
            }
        }

        // A fiacao da arvore mora no ProjectExplorerHost. Aqui fica so o que e
        // deste host: geometria, dados de outros dominios (git) e o que sobe.
        ProjectExplorerHost {
            id: explorerPanel

            width: visible ? root.shellController.explorerWidth : 0
            height: parent.height
            visible: root.workspaceOpen
                     && root.shellController.effectiveShowExplorer
            workspaceName: root.workspaceName
            workspaceKindLabel: root.shellController.kindLabel(
                                    root.workspaceKind,
                                    root.workspaceBuildSystems)
            projectTree: root.projectTree
            gestures: root.projectTreeGestures
            gitKinds: root.gitController.gitKinds
            gitRevision: root.gitController.revision
            onCloseRequested: root.closeWorkspaceRequested()
            onListDirRequested: function(path) {
                root.listDirRequested(path);
            }
            onReadFileRequested: function(path) {
                root.readFileRequested(path);
            }
            onFocusEditorRequested: root.editorController.focusEditor()
        }

        Column {
            id: centerColumn

            width: visible
                   ? Math.max(0, parent.width - sideBar.width - Theme.seamWidth
                              - (explorerPanel.visible
                                 ? explorerPanel.width + Theme.seamWidth : 0))
                   : 0
            height: parent.height
            spacing: Theme.seamWidth

            ProjectHealthBanner {
                id: healthBanner

                width: parent.width
                active: root.projectHealthController.active
                status: root.projectHealthController.status
                message: root.projectHealthController.message
                actionLabel: root.projectHealthController.actionLabel
                onActionRequested: {
                    const target = root.projectHealthController.actionTarget;
                    if (target === "scan") {
                        root.environmentScanRequested();
                    } else if (target === "cmakeConfigure") {
                        root.cmakeConfigureRequested();
                    } else if (target === "cargoMetadata") {
                        root.cargoMetadataRequested();
                    } else if (target !== "") {
                        root.shellController.showTab(target);
                    }
                }
                onDismissRequested: root.projectHealthController.dismiss()
            }

            StartScreen {
                width: parent.width
                height: visible
                        ? parent.height - (bottomPanel.visible
                          ? bottomPanel.height + Theme.seamWidth : 0) : 0
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

            EditorPane {
                id: editorPane

                width: parent.width
                visible: root.workspaceOpen
                height: visible ? parent.height
                        - (healthBanner.visible
                        ? healthBanner.height + Theme.seamWidth : 0)
                        - (bottomPanel.visible
                        ? bottomPanel.height + Theme.seamWidth : 0) : 0
                workspaceOpen: root.workspaceOpen
                filesModel: root.editorController.filesModel
                fileCount: root.editorController.filesModel.count
                currentTab: root.editorController.currentTab
                completionVisible: root.editorController.completionVisible
                usagesVisible: root.editorController.usagesVisible
                hoverVisible: root.editorController.hoverVisible
                hoverText: root.editorController.hoverText
                completionModel: root.editorController.completionModel
                completionCount: root.editorController.completionModel.count
                completionIndex: root.editorController.completionIndex
                actionsVisible: root.editorController.actionsVisible
                actionsModel: root.editorController.actionsModel
                actionCount: root.editorController.actionsModel.count
                actionsIndex: root.editorController.actionsIndex
                usagesModel: root.editorController.usagesModel
                usageCount: root.editorController.usagesModel.count
                createDialogVisible: root.projectTree.createDialogVisible
                createDialogKind: root.projectTree.createDialogKind
                createDialogParentDisplayPath: root.shellController.relativeToRoot(
                                                   root.projectTree.createDialogParentPath)
                createDialogError: root.projectTree.createDialogError
                renameDialogVisible: root.editorController.renameDialogVisible
                renameError: root.editorController.renameError
                workspaceEditPreviewVisible: root.editorController.workspaceEditPreviewVisible
                workspaceEditTitle: root.editorController.workspaceEditTitle
                workspaceEditFiles: root.editorController.workspaceEditFiles
                workspaceEditCount: root.editorController.workspaceEditCount
                workspaceEditError: root.editorController.workspaceEditError
                goToLineDialogVisible: root.editorController.goToLineVisible
                findBarVisible: root.editorController.findBarVisible
                findReplaceMode: root.editorController.findReplaceMode
                findQuery: root.editorController.findQuery
                findReplacement: root.editorController.findReplacement
                findCaseSensitive: root.editorController.findCaseSensitive
                findWholeWord: root.editorController.findWholeWord
                findUseRegex: root.editorController.findUseRegex
                findInvalidRegex: root.editorController.findInvalidRegex
                findMatchCount: root.editorController.findMatchCount
                findCurrentDisplay: root.editorController.findCurrentDisplay
                breakpointLines: root.currentFileBreakpoints(
                    root.editorController.currentTab,
                    root.debugController.breakpointsRevision)
                executionLine: root.currentFileExecutionLine(
                    root.editorController.currentTab,
                    root.debugController.currentFile,
                    root.debugController.currentLine)
                onGutterLineClicked: function(line) {
                    root.debugController.toggleBreakpoint(
                        root.editorController.currentFilePath(), line);
                }
                // currentTab no gatilho: funcao nao notifica; a troca de aba sim.
                breadcrumbPath: root.editorController.currentTab < 0 ? ""
                    : root.shellController.relativeToRoot(
                          root.editorController.currentFilePath())
                markdownMode: root.markdownMode
                diffLineKinds: root.gitController.diffLineKinds
                diffRevision: root.gitController.diffRevision
                blameActive: root.gitController.blameVisible
                blameLineAnnotations: root.gitController.blameLineAnnotations
                blameRevision: root.gitController.blameRevision
                diagnosticSpans: root.diagnosticsController.editorSpansList
                diagnosticByLine: root.diagnosticsController.gutterMap
                diagnosticRevision: root.diagnosticsController.revision
                autoCloseEnabled: root.editorController.autoCloseEnabled
                externalConflict: root.editorController.externalConflict
                externalDeleted: root.editorController.externalDeleted
                externalMessage: root.editorController.externalMessage
                watchError: root.editorController.watchError
                outlineItems: root.editorController.syntaxOutline
                outlineWidth: root.shellController.outlineWidth
                outlineCollapsed: root.shellController.outlineCollapsed
                onTabSelected: function(index) {
                    root.editorController.selectTab(index);
                }
                onTabCloseRequested: function(index) {
                    root.editorController.closeTab(index);
                }
                onTextEdited: function(text) {
                    root.editorController.handleTextEdited(text);
                }
                onCompletionMoveRequested: function(delta) {
                    root.editorController.moveCompletion(delta);
                }
                onCompletionAcceptRequested: root.editorController.acceptCompletion()
                onCompletionDismissRequested: root.editorController.completionVisible = false
                onActionsMoveRequested: function(delta) {
                    root.editorController.moveActions(delta);
                }
                onActionsAcceptRequested: root.editorController.applySelectedAction()
                onActionsDismissRequested: root.editorController.dismissActions()
                onActionActivated: function(index) {
                    root.editorController.applyCodeAction(index);
                }
                onUsagesDismissRequested: root.editorController.usagesVisible = false
                onHoverDismissRequested: root.editorController.hoverVisible = false
                onIndentRequested: root.editorController.indentEditorSelection()
                onUnindentRequested: root.editorController.unindentEditorSelection()
                onNewlineRequested: root.editorController.insertEditorNewline()
                onCloserBraceRequested: root.editorController.insertEditorCloserBrace()
                onSmartHomeRequested: function(extendSelection) {
                    root.editorController.editorSmartHome(extendSelection);
                }
                onCompletionActivated: function(index) {
                    root.editorController.completionIndex = index;
                    root.editorController.acceptCompletion();
                }
                onUsageOpenRequested: function(path, line, column) {
                    root.editorController.openDiagnostic(path, line, column);
                }
                onCreateConfirmRequested: function(name) {
                    root.projectTree.confirmCreateEntry(name);
                }
                onCreateCancelRequested: {
                    root.projectTree.createDialogVisible = false;
                    root.editorController.focusEditor();
                }
                onRenameConfirmRequested: function(name) {
                    root.editorController.confirmRename(name);
                }
                onRenameCancelRequested: {
                    root.editorController.renameDialogVisible = false;
                    root.editorController.focusEditor();
                }
                onWorkspaceEditApplyRequested: root.editorController.applyWorkspaceEdit()
                onWorkspaceEditCancelRequested: root.editorController.cancelWorkspaceEdit()
                onGoToLineConfirmRequested: function(value) {
                    root.editorController.confirmGoToLine(value);
                }
                onGoToLineCancelRequested: root.editorController.cancelGoToLine()
                onFindQueryEdited: function(text) {
                    root.editorController.setFindQuery(text);
                }
                onFindReplacementEdited: function(text) {
                    root.editorController.setFindReplacement(text);
                }
                onFindNextRequested: root.editorController.findNext()
                onFindPreviousRequested: root.editorController.findPrevious()
                onFindReplaceRequested: root.editorController.replaceFindCurrent()
                onFindReplaceAllRequested: root.editorController.replaceFindAll()
                onFindCaseToggleRequested: root.editorController.toggleFindCase()
                onFindWholeWordToggleRequested:
                    root.editorController.toggleFindWholeWord()
                onFindRegexToggleRequested: root.editorController.toggleFindRegex()
                onFindCloseRequested: root.editorController.closeFind()
                onExternalReloadRequested: root.editorController.reloadExternalFile()
                onExternalKeepLocalRequested: root.editorController.keepLocalFile()
                onWatchErrorDismissRequested: root.editorController.dismissWatchError()
                onOutlineOpenRequested: function(line, column) {
                    root.editorController.openOutlineItem(line, column);
                }
                onOutlineResizeRequested: function(delta) {
                    root.shellController.resizeOutline(delta);
                }
                onOutlineResetRequested: root.shellController.resetOutlineWidth()
                onOutlineToggleRequested: root.shellController.toggleOutline()
            }

            BottomPanelHost {
                id: bottomPanel

                width: parent.width
                height: root.shellController.bottomPanelHeight
                open: root.shellController.showBottomPanel
                activeTab: root.shellController.bottomTab
                coverage: root.jobsController.coverage
                compileContext: root.compileContextController
                buildOutputModel: root.jobsController.buildOutputModel
                jobsModel: root.jobsController.jobsModel
                testModel: root.jobsController.testModel
                testSummary: root.jobsController.testSummary
                testing: root.jobsController.testing
                problemsModel: root.jobsController.problemsModel
                terminalRender: root.runtimeController.terminalRender
                terminalActive: root.terminalActive
                workspaceAvailable: root.workspaceOpen
                runModel: root.runtimeController.runModel
                debugOutputModel: root.debugController.outputModel
                debugSessionActive: root.debugController.sessionActive
                debugPaused: root.debugController.paused
                debugFramesModel: root.debugController.framesModel
                debugVariablesModel: root.debugController.variablesModel
                debugCurrentFrameIndex: root.debugController.currentFrameIndex
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
                running: root.runtimeController.running
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
                onHideRequested: root.shellController.toggleBottomTab(
                                     root.shellController.bottomTab)
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
                onDebugContinueRequested: root.debugController.continueDebug()
                onDebugPauseRequested: root.debugController.pauseDebug()
                onDebugStepOverRequested: root.debugController.stepOver()
                onDebugStepIntoRequested: root.debugController.stepInto()
                onDebugStepOutRequested: root.debugController.stepOutOf()
                onDebugStopRequested: root.debugController.stopDebug()
                onDebugFrameActivated: function(index) {
                    root.debugController.selectFrame(index, true);
                }
                onDebugVariableToggled: function(index) {
                    root.debugController.toggleVariable(index);
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

    // Alcas de redimensionamento montadas SOBRE o divisor de 1px (§4.2):
    // area de arrasto invisivel de `splitterGrip` px centrada no divisor;
    // so a linha de hover do PanelSplitter pinta. Limites no ShellController.
    PanelSplitter {
        visible: explorerPanel.visible
        x: explorerPanel.x + explorerPanel.width - Theme.splitterGrip / 2
        width: Theme.splitterGrip
        height: parent.height
        onDragged: function(delta) {
            root.shellController.resizeExplorer(delta);
        }
    }


    PanelSplitter {
        visible: bottomPanel.visible
        horizontal: true
        x: centerColumn.x
        y: centerColumn.y + bottomPanel.y - Theme.splitterGrip / 2
        width: centerColumn.width
        height: Theme.splitterGrip
        onDragged: function(delta) {
            root.shellController.resizeBottomPanel(-delta);
        }
    }
}
