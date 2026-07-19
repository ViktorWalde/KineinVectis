import QtQuick

// Fiacao da arvore de projeto: liga o painel visual (`ProjectExplorer`) ao
// dono do estado (`ProjectTreeController`) e ao interprete de gesto
// (`ProjectTreeGestures`).
//
// Por que existe: essa fiacao morava no `ShellWorkspaceHost`, que ja esta em
// 537 linhas contra um limite de 400. A catraca de arquitetura so gira para um
// lado — arquivo em debito nao pode engordar, nem uma linha. Os gestos novos
// (arraste, teclado) nao cabiam la, e a regra da casa e "quem toca a area paga
// a dela": a fiacao da arvore tem dono proprio agora, e o host encolheu.
//
// Herda o painel em vez de embrulha-lo porque o `ShellWorkspaceHost` posiciona
// esse item pela geometria (`explorerPanel.x/width/visible`); embrulhar exigiria
// republicar as tres, sem ganho nenhum.
ProjectExplorer {
    id: root

    property var projectTree: null

    // Sobem para o `ShellWorkspaceHost`, que e quem fala com o core.
    signal listDirRequested(string path)
    signal readFileRequested(string path)

    selectedPath: root.projectTree ? root.projectTree.selectedPath : ""
    selectedIndex: root.projectTree ? root.projectTree.selectedIndex : -1
    entriesModel: root.projectTree ? root.projectTree.entriesModel : null
    moveError: root.projectTree ? root.projectTree.moveError : ""

    onRefreshRequested: root.listDirRequested(root.projectTree.workspaceRoot)
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

    // Gesto: quem decide e o ProjectTreeGestures; quem muta e o controller.
    onDropRequested: function(sourcePath, targetPath, targetKind) {
        root.gestures.dropOn(sourcePath, targetPath, targetKind);
    }
    onMoveSelectionRequested: function(delta) {
        root.gestures.moveSelection(delta);
    }
    onExpandSelectedRequested: root.gestures.expandSelected()
    onCollapseSelectedRequested: root.gestures.collapseSelected()
    onActivateSelectedRequested: root.gestures.activateSelected()
    onDeleteSelectedRequested: root.projectTree.deleteSelected()
}
