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
    property var searchController
    property var assistantController
    property alias editorSurface: editorPane.editorSurface
    property bool workspaceOpen: false
    property string workspaceRoot: ""
    property string workspaceName: ""
    property string workspaceKind: ""
    property bool testing: false
    property bool terminalActive: false
    property bool running: false
    property var logLinesModel

    signal listDirRequested(string path)
    signal readFileRequested(string path)
    signal closeWorkspaceRequested()
    signal toolsDetectionRequested()
    signal environmentScanRequested()

    function focusSearchInput() {
        bottomPanel.focusSearchInput();
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

    ShellLayout {
        anchors.fill: parent

        SideRail {
            id: sideBar

            height: parent.height
            workspaceOpen: root.workspaceOpen
            explorerActive: root.shellController.showExplorer
            toolsActive: root.shellController.showBottomPanel
                         && root.shellController.bottomTab === "tools"
            logsActive: root.shellController.showBottomPanel
                        && root.shellController.bottomTab === "logs"
            assistantActive: root.shellController.showAssistant
            onExplorerToggled: root.shellController.toggleExplorer()
            onToolsRequested: root.shellController.toggleBottomTab("tools")
            onLogsRequested: root.shellController.toggleBottomTab("logs")
            onAssistantToggled: root.shellController.toggleAssistant()
        }

        ProjectExplorer {
            id: explorerPanel

            width: visible ? 260 : 0
            height: parent.height
            visible: root.workspaceOpen && root.shellController.showExplorer
            workspaceName: root.workspaceName
            workspaceKindLabel: root.shellController.kindLabel(
                                    root.workspaceKind)
            selectedPath: root.projectTree.selectedPath
            entriesModel: root.projectTree.entriesModel
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
            onContextMenuRequested: function(path, kind, name, sceneX, sceneY) {
                root.projectTree.openEntryMenu(path, kind, name, sceneX, sceneY);
            }
        }

        Column {
            width: parent.width - sideBar.width - Theme.panelGap
                   - (explorerPanel.visible
                      ? explorerPanel.width + Theme.panelGap : 0)
                   - (assistantPanel.visible
                      ? assistantPanel.width + Theme.panelGap : 0)
            height: parent.height
            spacing: Theme.panelGap

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
                    } else if (target !== "") {
                        root.shellController.showTab(target);
                    }
                }
                onDismissRequested: root.projectHealthController.dismiss()
            }

            EditorPane {
                id: editorPane

                width: parent.width
                height: parent.height
                        - (healthBanner.visible
                        ? healthBanner.height + Theme.panelGap : 0)
                        - (bottomPanel.visible
                        ? bottomPanel.height + Theme.panelGap : 0)
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
                usagesModel: root.editorController.usagesModel
                usageCount: root.editorController.usagesModel.count
                createDialogVisible: root.projectTree.createDialogVisible
                createDialogKind: root.projectTree.createDialogKind
                createDialogParentDisplayPath: root.shellController.relativeToRoot(
                                                   root.projectTree.createDialogParentPath)
                createDialogError: root.projectTree.createDialogError
                renameDialogVisible: root.editorController.renameDialogVisible
                renameError: root.editorController.renameError
                onTabSelected: function(index) {
                    root.editorController.selectTab(index);
                }
                onTabCloseRequested: function(index) {
                    root.editorController.closeTab(index);
                }
                onSaveRequested: root.editorController.saveCurrentFile()
                onTextEdited: function(text) {
                    root.editorController.handleTextEdited(text);
                }
                onCompletionMoveRequested: function(delta) {
                    root.editorController.moveCompletion(delta);
                }
                onCompletionAcceptRequested: root.editorController.acceptCompletion()
                onCompletionDismissRequested: root.editorController.completionVisible = false
                onUsagesDismissRequested: root.editorController.usagesVisible = false
                onHoverDismissRequested: root.editorController.hoverVisible = false
                onIndentRequested: root.editorController.indentEditorSelection()
                onUnindentRequested: root.editorController.unindentEditorSelection()
                onNewlineRequested: root.editorController.insertEditorNewline()
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
            }

            BottomPanelHost {
                id: bottomPanel

                width: parent.width
                height: 170
                open: root.shellController.showBottomPanel
                activeTab: root.shellController.bottomTab
                problemCount: root.jobsController.problemsModel.count
                buildOutputModel: root.jobsController.buildOutputModel
                jobsModel: root.jobsController.jobsModel
                testModel: root.jobsController.testModel
                testSummary: root.jobsController.testSummary
                testing: root.testing
                problemsModel: root.jobsController.problemsModel
                terminalText: root.runtimeController.terminalText
                terminalActive: root.terminalActive
                workspaceAvailable: root.workspaceOpen
                runModel: root.runtimeController.runModel
                running: root.running
                searchModel: root.searchController.searchModel
                searchCaseSensitive: root.searchController.caseSensitive
                searching: root.searchController.searching
                searchTruncated: root.searchController.searchTruncated
                logLinesModel: root.logLinesModel
                toolsList: root.workspaceController.toolsList
                onTabRequested: function(tab) {
                    root.shellController.toggleBottomTab(tab);
                }
                onRefreshToolsRequested: root.toolsDetectionRequested()
                onProblemOpenRequested: function(file, line, column) {
                    root.editorController.openDiagnostic(file, line, column);
                }
                onTerminalOpenRequested: root.runtimeController.openTerminalPanel()
                onTerminalInputSubmitted: function(text) {
                    root.runtimeController.submitShellInput(text);
                }
                onRunInputSubmitted: function(text) {
                    root.runtimeController.submitRunInput(text);
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
            }
        }

        AssistantPanel {
            id: assistantPanel

            width: visible ? 300 : 0
            height: parent.height
            visible: root.shellController.showAssistant
            messagesModel: root.assistantController.messagesModel
            onCloseRequested: root.shellController.closeAssistant()
            onMessageSubmitted: function(body) {
                root.assistantController.sendMessage(body);
            }
        }
    }
}
