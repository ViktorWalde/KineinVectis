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
    title: coreClient.workspaceName !== ""
           ? qsTr("%1 - Kinein Vectis").arg(coreClient.workspaceName)
           : qsTr("Kinein Vectis")
    color: Theme.background0

    CoreClient {
        id: coreClient
    }
    AssistantController {
        id: assistantController

        onProfilesRequested: coreClient.aiProfiles()
        onTerminalOpenRequested: function(profileId) {
            coreClient.aiTerminalOpen(profileId);
        }
        onTerminalInputRequested: function(id, data) {
            coreClient.terminalInput(id, data);
        }
        onTerminalResizeRequested: function(id, cols, rows) {
            coreClient.terminalResize(id, cols, rows);
        }
        onTerminalScrollRequested: function(id, offset) {
            coreClient.terminalScroll(id, offset);
        }
        onTerminalCloseRequested: function(id) {
            coreClient.terminalClose(id);
        }
        onProfilePreferenceRequested: function(profileId) {
            settingsController.setGlobal({ aiCliProfile: profileId });
        }
    }
    WorkspaceController {
        id: workspaceController

        workspaceRoot: coreClient.workspaceRoot
        onClearWorkspaceUiRequested: workspaceUiResetter.clear()
    }

    ProjectHealthController {
        id: projectHealthController

        workspaceRoot: coreClient.workspaceRoot
        workspaceKind: coreClient.workspaceKind
        toolsList: workspaceController.toolsList
        scanningEnvironment: coreClient.scanningEnvironment
        onAutoConfigureRequested: coreClient.cmakeConfigure()
    }

    ShellController {
        id: shellController

        workspaceRoot: coreClient.workspaceRoot
        workspaceKind: coreClient.workspaceKind
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
        onRunBuildRequested: coreClient.runBuild()
        onRunTestsRequested: coreClient.runTests("")
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
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onTerminalOpenRequested: coreClient.terminalOpen()
        onTerminalInputRequested: function(id, data) {
            coreClient.terminalInput(id, data);
        }
        onTerminalResizeRequested: function(id, cols, rows) {
            coreClient.terminalResize(id, cols, rows);
        }
        onTerminalScrollRequested: function(id, offset) {
            coreClient.terminalScroll(id, offset);
        }
        onTerminalCloseRequested: function(id) {
            coreClient.terminalClose(id);
        }
        onRunStartRequested: function(command) {
            coreClient.runStart(command);
        }
        onRunStopRequested: coreClient.runStop()
        onRunStdinRequested: function(data) {
            coreClient.runStdin(data);
        }
        onSaveRunConfigRequested: function(id, name, command) {
            coreClient.runConfigSave(id, name, command);
        }
        onDeleteRunConfigRequested: function(id) {
            coreClient.runConfigDelete(id);
        }
        onSetActiveRunConfigRequested: function(id) {
            coreClient.runConfigSetActive(id);
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
        onStartRequested: function(program) {
            coreClient.debugStart(program);
        }
        onStopRequested: coreClient.debugStop()
        onContinueRequested: coreClient.debugContinue()
        onNextRequested: coreClient.debugNext()
        onStepInRequested: coreClient.debugStepIn()
        onStepOutRequested: coreClient.debugStepOut()
        onPauseRequested: coreClient.debugPause()
        onSetBreakpointsRequested: function(file, lines) {
            coreClient.debugSetBreakpoints(file, lines);
        }
        onStackTraceRequested: coreClient.debugStackTrace()
        onFrameVariablesRequested: function(frameId) {
            coreClient.debugVariablesForFrame(frameId);
        }
        onVariablesByRefRequested: function(ref) {
            coreClient.debugVariablesForRef(ref);
        }
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
        onStatusRequested: coreClient.gitStatus()
        onBranchesRequested: coreClient.gitBranches()
        onCheckoutRequested: function(branch) {
            if (editorController.hasModifiedFiles()) {
                gitController.rejectDirtyOperation();
                return;
            }
            coreClient.gitCheckout(branch);
        }
        onBranchCreateRequested: function(name) {
            if (editorController.hasModifiedFiles()) {
                gitController.rejectDirtyOperation();
                return;
            }
            coreClient.gitCreateBranch(name, true);
        }
        onPullRequested: {
            if (editorController.hasModifiedFiles()) {
                gitController.rejectDirtyOperation();
            } else {
                coreClient.gitPull();
            }
        }
        onPushRequested: coreClient.gitPush()
        onStashRequested: function(action, message) {
            if (editorController.hasModifiedFiles()) {
                gitController.rejectDirtyOperation();
                return;
            }
            coreClient.gitStash(action, message);
        }
        onFileDiffRequested: function(path) {
            coreClient.gitFileDiff(path);
        }
        onStageRequested: function(paths) {
            coreClient.gitStage(paths);
        }
        onUnstageRequested: function(paths) {
            coreClient.gitUnstage(paths);
        }
        onDiscardRequested: function(paths) {
            coreClient.gitDiscard(paths);
        }
        onCommitRequested: function(message) {
            coreClient.gitCommit(message);
        }
        onBlameRequested: function(path) {
            coreClient.gitBlame(path);
        }
        onLogRequested: coreClient.gitLog()
        onCommitDiffRequested: function(sha) {
            coreClient.gitCommitDiff(sha);
        }
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
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onFocusSearchInputRequested: workspaceHost.focusSearchInput()
        onFocusReplaceInputRequested: workspaceHost.focusSearchReplaceInput()
        onResetAndFocusEverywhereRequested: shellOverlays.resetSearchEverywhereAndFocus()
        onSearchInFilesRequested: function(query, caseSensitive) {
            coreClient.searchInFiles(query, caseSensitive);
        }
        onReplaceInFilesRequested: function(query, replacement, caseSensitive) {
            if (editorController.hasModifiedFiles()) {
                searchController.rejectReplaceForDirtyEditors();
                return;
            }
            coreClient.replaceInFiles(query, replacement, caseSensitive);
        }
        onFindFilesRequested: function(query) {
            coreClient.findFiles(query);
        }
        onDocumentSymbolsRequested: {
            coreClient.requestDocumentSymbols(editorController.currentFilePath(),
                                              editorController.editorText());
        }
        onWorkspaceSymbolsRequested: function(query) {
            coreClient.requestWorkspaceSymbols(editorController.currentFilePath(),
                                               editorController.editorText(), query);
        }
        onOpenAtRequested: function(path, line, column) {
            editorController.openDiagnostic(path, line, column);
        }
        onListCommandsRequested: coreClient.listCommands()
        onReadFileRequested: function(path) {
            coreClient.readFile(path);
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
        onReadFileRequested: function(path) {
            coreClient.readFile(path);
        }
        onWriteFileRequested: function(path, content, expectedContent) {
            coreClient.writeFile(path, content, expectedContent);
        }
        onDraftSaveRequested: function(path, content) {
            coreClient.draftSave(path, content);
        }
        onDraftClearRequested: function(path) {
            coreClient.draftClear(path);
        }
        onFormatRequested: function(path, content) {
            coreClient.formatFile(path, content);
        }
        onCodeActionsRequested: function(path, content, line, column) {
            coreClient.requestCodeActions(path, content, line, column);
        }
        onCodeActionApplyRequested: function(path, content, actionIndex) {
            coreClient.applyCodeAction(path, content, actionIndex);
        }
        onWorkspaceEditApplyRequested: function(transactionId) {
            coreClient.applyWorkspaceEdit(transactionId);
        }
        onWorkspaceEditCancelRequested: function(transactionId) {
            coreClient.cancelWorkspaceEdit(transactionId);
        }
        onSaveSessionRequested: function(files, activeFile) {
            coreClient.saveSession(files, activeFile);
        }
        onFileChangedNotificationRequested: function(path, content) {
            coreClient.notifyFileChanged(path, content);
        }
        onSemanticTokensRequested: function(path, content) {
            coreClient.requestSemanticTokens(path, content);
        }
        onSyntaxTreeRequested: function(path, content, version) {
            coreClient.requestSyntaxTree(path, content, version);
        }
        onSwitchSourceHeaderRequested: function(path, content) {
            coreClient.requestSwitchSourceHeader(path, content);
        }
        onDefinitionRequested: function(path, content, line, column) {
            coreClient.requestDefinition(path, content, line, column);
        }
        onHoverRequested: function(path, content, line, column) {
            coreClient.requestHover(path, content, line, column);
        }
        onCompletionRequested: function(path, content, line, column) {
            coreClient.requestCompletion(path, content, line, column);
        }
        onReferencesRequested: function(path, content, line, column) {
            coreClient.requestReferences(path, content, line, column);
        }
        onRenameRequested: function(path, content, line, column, newName) {
            coreClient.requestRename(path, content, line, column, newName);
        }
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
        onListDirRequested: function(path) {
            coreClient.listDir(path);
        }
        onCreateFileRequested: function(path) {
            coreClient.createFile(path, "");
        }
        onCreateDirectoryRequested: function(path) {
            coreClient.createDirectory(path);
        }
        onReadFileRequested: function(path) {
            coreClient.readFile(path);
        }
        onRenamePathRequested: function(from, to) {
            coreClient.renamePath(from, to);
        }
        onDeletePathRequested: function(path) {
            coreClient.deletePath(path);
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
        assistantController: assistantController
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

        function onConnectedChanged() {
            if (coreClient.connected && workspaceController.toolsList.length === 0) {
                coreClient.detectTools();
            }
            if (coreClient.connected) {
                coreClient.settingsGet();
                assistantController.initialize();
            }
        }

        function onWorkspaceChanged() {
            coreClient.settingsGet();
            assistantController.initialize();
        }
    }

    AiBridgeEventRouter {
        coreClient: coreClient
        assistantController: assistantController
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
    }

    EditorEventRouter {
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

    RuntimeEventRouter {
        coreClient: coreClient
        runtimeController: runtimeController
    }

    DebugEventRouter {
        coreClient: coreClient
        debugController: debugController
    }

    GitEventRouter {
        coreClient: coreClient
        gitController: gitController
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
        assistantController: assistantController
        workspaceOpen: coreClient.workspaceRoot !== ""
        workspaceRoot: coreClient.workspaceRoot
        workspaceName: coreClient.workspaceName
        workspaceKind: coreClient.workspaceKind
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

}
