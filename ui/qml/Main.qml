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
        onTerminalInputRequested: function(data) {
            coreClient.terminalInput(data);
        }
        onRunStartRequested: function(command) {
            coreClient.runStart(command);
        }
        onRunStopRequested: coreClient.runStop()
        onRunStdinRequested: function(data) {
            coreClient.runStdin(data);
        }
        onFocusTerminalInputRequested: workspaceHost.focusTerminalInput()
        onClearTerminalInputRequested: workspaceHost.clearTerminalInput()
        onClearRunInputRequested: workspaceHost.clearRunInput()
    }

    SearchController {
        id: searchController

        workspaceRoot: coreClient.workspaceRoot
        onShowTabRequested: function(tab) {
            shellController.showTab(tab);
        }
        onFocusSearchInputRequested: workspaceHost.focusSearchInput()
        onResetAndFocusEverywhereRequested: shellOverlays.resetSearchEverywhereAndFocus()
        onSearchInFilesRequested: function(query, caseSensitive) {
            coreClient.searchInFiles(query, caseSensitive);
        }
        onFindFilesRequested: function(query) {
            coreClient.findFiles(query);
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
        editorController: editorController
        jobsController: jobsController
        projectTree: projectTree
        runtimeController: runtimeController
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
        onReadFileRequested: function(path) {
            coreClient.readFile(path);
        }
        onWriteFileRequested: function(path, content) {
            coreClient.writeFile(path, content);
        }
        onFileChangedNotificationRequested: function(path, content) {
            coreClient.notifyFileChanged(path, content);
        }
        onSemanticTokensRequested: function(path, content) {
            coreClient.requestSemanticTokens(path, content);
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
        assistantController.initialize();
    }

    WorkspaceEventRouter {
        coreClient: coreClient
        folderPicker: folderPicker
        projectTree: projectTree
        searchController: searchController
        workspaceController: workspaceController
    }

    EditorEventRouter {
        coreClient: coreClient
        editorController: editorController
    }

    JobsEventRouter {
        coreClient: coreClient
        jobsController: jobsController
    }

    SearchEventRouter {
        coreClient: coreClient
        searchController: searchController
    }

    RuntimeEventRouter {
        coreClient: coreClient
        runtimeController: runtimeController
    }

    GlobalShortcuts {
        editorController: editorController
        jobsController: jobsController
        runtimeController: runtimeController
        searchController: searchController
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
        onListDirRequested: function(path) {
            coreClient.listDir(path);
        }
        onReadFileRequested: function(path) {
            coreClient.readFile(path);
        }
        onCloseWorkspaceRequested: coreClient.closeWorkspace()
        onToolsDetectionRequested: coreClient.detectTools()
        onEnvironmentScanRequested: coreClient.scanEnvironment()
    }

    ShellStatusHost {
        id: statusBar

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        coreClient: coreClient
        shellController: shellController
    }

    ShellOverlays {
        id: shellOverlays

        hostWidth: root.width
        hostHeight: root.height
        searchController: searchController
        projectTree: projectTree
        editorController: editorController
        shellController: shellController
    }

}
