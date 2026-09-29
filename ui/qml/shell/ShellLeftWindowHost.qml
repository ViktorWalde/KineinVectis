import QtQuick
import KineinVectis

// O SLOT a esquerda do trilho (E3-3, roadmaps/44 §4.1): UM lugar, duas
// janelas — o explorer do projeto ou a janela do Git em pe' — como a
// referencia alterna Project/Commit. Quem decide qual esta' visivel e' o
// ShellController (leftWindow); a largura e o splitter sao os do explorer.
// O explorer saiu do ShellWorkspaceHost como estava.
Item {
    id: root

    property var shellController
    property var projectTree
    property var gitController
    property string workspaceName: ""
    property string workspaceRoot: ""

    signal listDirRequested(string path)
    signal readFileRequested(string path)
    signal closeWorkspaceRequested()

    ProjectExplorer {
        id: explorerPanel

        anchors.fill: parent
        visible: root.shellController.effectiveShowExplorer
        workspaceName: root.workspaceName
        selectedPaths: root.projectTree.selectedPaths
        entriesModel: root.projectTree.entriesModel
        projectTree: root.projectTree
        gitKinds: root.gitController.gitKinds
        gitRevision: root.gitController.revision
        onCreateFileRequested: root.projectTree.openCreateDialog("file")
        onCreateDirectoryRequested: root.projectTree.openCreateDialog("directory")
        onRefreshRequested: root.listDirRequested(root.workspaceRoot)
        onCloseRequested: root.closeWorkspaceRequested()
        onEntrySelected: function(path, kind, modifiers) {
            root.projectTree.selectEntry(path, kind, modifiers);
        }
        onSelectAllRequested: root.projectTree.selectAllEntries()
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

    Connections {
        target: root.projectTree
        function onFocusTreeRequested() { explorerPanel.focusTree(); }
    }

    GitWindow {
        anchors.fill: parent
        visible: root.shellController.gitWindowVisible
        gitController: root.gitController
        // Abrir o arquivo pela lista: o editor volta (o visualizador fecha).
        onOpenRequested: function(absPath) {
            root.gitController.inspector.clear();
            root.readFileRequested(absPath);
        }
        onCloseRequested: root.shellController.toggleGitWindow()
    }
}
