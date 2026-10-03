import QtQuick
import KineinVectis

// O SLOT a esquerda do trilho (E3-3, roadmaps/44 §4.1): UM lugar, tres
// janelas — o explorer do projeto, a janela do Git em pe' e (2026-10-03) a do
// Banco — como a referencia alterna Project/Commit/Database. Quem decide qual esta' visivel e' o
// ShellController (leftWindow); a largura e o splitter sao os do explorer.
// O explorer saiu do ShellWorkspaceHost como estava.
Item {
    id: root

    property var shellController
    property var projectTree
    property var gitController
    property var dataSourceController: null
    property var editorController: null
    property var containerController: null
    property string workspaceName: ""
    property string workspaceRoot: ""
    // O slot da direita (ShellRightDock): a janela cujo icone esta' no trilho
    // da direita muda de PAI para la' — a mesma instancia, o mesmo estado.
    property Item rightSlot: null

    function slotOf(name) {
        return root.rightSlot !== null && root.shellController.rightWindow === name ? root.rightSlot : root;
    }

    function minimumOf(name) {
        if (name === "git") return gitWindow.minimumWidth;
        if (name === "database") return databaseWindow.minimumWidth;
        return name === "containers" ? containersWindow.minimumWidth : 220;
    }

    // O minimo DECLARADO por quem esta' no slot (53 §4.4); o shell o usa como
    // piso da largura. O explorer se vira em 220.
    readonly property real minimumWidth: root.minimumOf(root.shellController.leftWindow)
    readonly property real rightMinimumWidth: root.minimumOf(root.shellController.rightWindow)

    signal listDirRequested(string path)
    signal readFileRequested(string path)
    signal closeWorkspaceRequested()

    ProjectExplorer {
        id: explorerPanel

        parent: root.slotOf("explorer")
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
        onCopyRequested: root.projectTree.fileClipboard.copySelection(false)
        onCutRequested: root.projectTree.fileClipboard.copySelection(true)
        onPasteRequested: root.projectTree.fileClipboard.openPaste()
        onFilesDropped: function(paths, destination, copy, external) {
            if (external) root.projectTree.fileClipboard.openImport(paths, destination);
            else root.projectTree.fileClipboard.openTransfer(paths, !copy,
                                                             destination, false);
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

    Connections {
        target: root.projectTree
        function onFocusTreeRequested() { explorerPanel.focusTree(); }
    }

    // O ciclo de foco (Ctrl+F6) entra aqui: na janela que esta' no slot.
    // O slot da direita (ShellRightDock) pede o dele por `focusSlot("right")`.
    function focusArea() {
        root.focusSlot("left");
    }

    function focusSlot(side) {
        const name = root.shellController.docks.windowOn(side);
        if (name === "git") gitWindow.forceActiveFocus();
        else if (name === "database") databaseWindow.forceActiveFocus();
        else if (name === "containers") containersWindow.forceActiveFocus();
        else explorerPanel.focusTree();
    }

    GitWindow {
        id: gitWindow

        parent: root.slotOf("git")
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

    DatabaseWindow {
        id: databaseWindow

        parent: root.slotOf("database")
        anchors.fill: parent
        visible: root.shellController.databaseWindowVisible
        controller: root.dataSourceController
        onVisibleChanged: if (visible) root.dataSourceController.refreshCatalog()
        onConsoleRequested: name => root.dataSourceController.consoles.open(name)
        onTableDataRequested: (connection, engine, schema, table) =>
            root.dataSourceController.consoles.tableData(connection, engine, schema, table)
        onEditRequested: name => {
            root.dataSourceController.select(name);
            root.dataSourceController.open();
        }
        onNewRequested: {
            root.dataSourceController.startNew();
            root.dataSourceController.open();
        }
        onCandidateChosen: index => {
            root.dataSourceController.discovery.adopt(index);
            root.dataSourceController.open();
        }
        onCloseRequested: root.shellController.toggleDockWindow("database")
        onWidenRequested: width => root.shellController.docks.widen("database", width)
    }

    // Os Containers (2026-10-03): janela acoplada como o Banco, e a unica que
    // abre sem projeto (ShellDocks.needsProject). Toda abertura pergunta ao
    // motor de novo — container sobe e cai fora da IDE.
    ContainersWindow {
        id: containersWindow

        parent: root.slotOf("containers")
        anchors.fill: parent
        visible: root.shellController.containersWindowVisible
        controller: root.containerController
        onVisibleChanged: if (visible && root.containerController && !root.containerController.listBusy) root.containerController.refresh()
        onCloseRequested: root.shellController.toggleDockWindow("containers")
    }

    // Ctrl+Alt+W, o menu e a paleta pedem a janela pelo controller.
    Connections {
        target: root.containerController

        function onWindowRequested() {
            root.shellController.showDockWindow("containers");
        }
    }

    // Ctrl+Enter num console do Banco (`.kinein/consoles/`) executa a
    // instrucao sob o cursor — ou a selecao — na conexao do arquivo. Fora de
    // um console o atalho fica desligado e a tecla segue para o editor.
    Shortcut {
        sequences: ["Ctrl+Return", "Ctrl+Enter"]
        enabled: root.editorController !== null && root.dataSourceController !== null
                 && root.dataSourceController.consoles.isConsole(root.editorController.currentFilePath())
        onActivated: {
            const surface = root.editorController.editorSurface;
            root.dataSourceController.consoles.runFromEditor(root.editorController.currentFilePath(), surface.text,
                                                            surface.cursorPosition, surface.selectionStart,
                                                            surface.selectionEnd);
        }
    }

    // Executar no console (Ctrl+Enter) ou abrir uma tabela traz a janela do
    // Banco com a secao de dados aberta, mesmo que ela estivesse fechada.
    Connections {
        target: root.dataSourceController ? root.dataSourceController.consoles : null

        function onResultsRequested() {
            root.shellController.showDockWindow("database");
            databaseWindow.showResults();
        }
    }
}
