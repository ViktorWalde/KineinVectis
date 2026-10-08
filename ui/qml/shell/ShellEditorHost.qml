import QtQuick
import KineinVectis

// Composition host do editor: liga o EditorPane (visual burro) aos controllers
// que ele precisa. Nasceu em 2026-09-03 do ShellWorkspaceHost, que lia 92
// propriedades do editorController para repassa-las ao painel — a ARCHITECTURE
// §4 regra 8 manda dividir composition root POR AREA, fazendo a contagem de
// arquivos crescer em vez do tamanho. Mesmo padrao do ShellHeaderHost.
//
// O layout (largura, altura, visibilidade) continua sendo do ShellWorkspaceHost:
// ele depende dos irmaos (banner de saude, painel inferior) e por isso nao desce.
Item {
    id: root

    property var dataSourceController: null
    property var editorController
    property var shellController
    property var debugController
    property var gitController
    property var coverageController
    property var diagnosticsController
    property var projectTree
    property var urlDecoder: Clipboard

    property bool workspaceOpen: false
    property var indexController: null

    property alias editorSurface: editorPane.editorSurface

    ProjectTreeRules {
        id: treeRules
    }

    function internalDroppedFile(event) {
        if (!workspaceOpen || projectTree === null) return "";
        const paths = ProjectDragRules.internalPaths(event);
        if (paths.length !== 1 || !paths[0].startsWith(shellController.workspaceRoot + "/"))
            return "";
        const row = projectTree.rowIndexForPath(paths[0]);
        return row >= 0 && treeRules.isFile(projectTree.entriesModel.get(row).kind)
               ? paths[0] : "";
    }

    function localDroppedFile(event) {
        if (!workspaceOpen || !event.hasUrls || urlDecoder === null) return "";
        const paths = urlDecoder.localFilePathsFromUrls(event.urls);
        return paths.length === 1 ? paths[0] : "";
    }

    function focusCreateDialog() {
        overlayHost.focusCreateDialog();
    }

    function openRenameDialogWithName(name) {
        overlayHost.openRenameDialogWithName(name);
    }

    function openGoToLineDialog(prefill) {
        overlayHost.openGoToLineDialog(prefill);
    }

    // O ciclo de foco (Ctrl+F6) entra aqui: o texto do editor.
    function focusArea() {
        root.editorController.focusEditor();
    }

    function focusFindBar() {
        overlayHost.focusFindBar();
    }

    function focusSymbols(query) {
        editorPane.focusSymbols(query);
    }

    // Os args de tab/revisao existem so para os bindings reavaliarem
    // quando a aba ativa ou os breakpoints mudam (funcoes nao notificam).
    function currentFileBreakpoints(currentTab, revision) {
        return root.debugController.breakpointLinesFor(
            root.editorController.currentFilePath());
    }

    function currentFileExecutionLine(currentTab, stoppedFile, stoppedLine) {
        if (stoppedFile === ""
                || stoppedFile !== root.editorController.currentFilePath()) {
            return 0;
        }
        return stoppedLine;
    }

    // A §6 da especificacao do preview diz que o HOST compoe a previa, e nao o
    // EditorController: o estado de modo e' apresentacao, e o documento so'
    // fornece identidade e conteudo. E' por isso que ele nasce aqui, ao lado do
    // painel, e nao na lista de dominios do AppDomains.
    MarkdownPreviewController {
        id: markdownPreview

        editorController: root.editorController
    }

    // O arquivo ativo vai ao dono dos consoles (passo 13a): dali saem o
    // cabecalho do console e o "localizar" da janela do Banco.
    Binding {
        target: root.dataSourceController ? root.dataSourceController.consoles : null
        property: "activePath"
        value: root.editorController.currentFilePath()
        when: root.dataSourceController !== null
    }

    DataSourceConsoleBanner {
        id: consoleBanner
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        profile: root.dataSourceController === null ? null : root.dataSourceController.profileByName(
            root.dataSourceController.consoles.activeConnection)
        onPreviewRequested: {
            const surface = editorPane.editorSurface;
            root.dataSourceController.consoles.runFromEditor(root.editorController.currentFilePath(), surface.text,
                surface.cursorPosition, surface.selectionStart, surface.selectionEnd, true);
        }
        onHistoryRequested: (x, y) => {
            const point = consoleBanner.mapToItem(root, x, y);
            historyMenu.menuX = point.x;
            historyMenu.menuY = point.y;
            root.dataSourceController.history.open(root.dataSourceController.consoles.activeConnection);
            root.historyOpen = true;
        }
    }

    // O HISTORICO da conexao do console (passo 13b), por cima do editor.
    property bool historyOpen: false
    DataSourceHistoryMenu {
        id: historyMenu
        anchors.fill: parent
        z: 100
        visible: root.historyOpen && root.dataSourceController !== null
        history: root.dataSourceController ? root.dataSourceController.history : null
        onDismissRequested: root.historyOpen = false
        onStatementChosen: sql => root.dataSourceController.consoles.open(root.dataSourceController.history.name, sql)
    }

    EditorPane {
        id: editorPane

        anchors.top: consoleBanner.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        workspaceOpen: root.workspaceOpen
        filesModel: root.editorController.filesModel
        fileLabels: root.dataSourceController === null ? ({}) : root.dataSourceController.consoles.labels()
        fileCount: root.editorController.filesModel.count
        currentTab: root.editorController.currentTab
        completionVisible: root.editorController.completionVisible
        usagesVisible: root.editorController.usagesVisible
        hoverVisible: root.editorController.hoverVisible
        actionsVisible: root.editorController.actionsVisible
        breakpointLines: root.currentFileBreakpoints(
            root.editorController.currentTab,
            root.debugController.breakpointsRevision)
        executionLine: root.currentFileExecutionLine(
            root.editorController.currentTab,
            root.debugController.currentFile,
            root.debugController.currentLine)
        // A lampada da calha e' o Alt+Enter na linha do cursor.
        onCodeActionsRequested: function(line) {
            if (!root.editorController.currentReadOnly)
                root.editorController.requestCodeActions();
        }
        onGutterLineClicked: function(line) {
            if (!root.editorController.currentReadOnly)
                root.debugController.toggleBreakpoint(
                    root.editorController.currentFilePath(), line);
        }
        diffLineKinds: root.gitController.diffLineKinds
        diffRevision: root.gitController.diffRevision
        coverageLineKinds: root.coverageController.lineKinds
        coverageRevision: root.coverageController.revision
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
        readOnlyExternal: root.editorController.currentReadOnly
        externalPreviewError: root.editorController.externalPreview.errorMessage
        watchError: root.editorController.watchError
        outlineItems: root.editorController.syntaxOutline
        symbols: root.indexController ? root.indexController.symbols : null
        outlineWidth: root.shellController.outlineWidth
        outlineCollapsed: root.shellController.effectiveOutlineCollapsed
        markdownAvailable: markdownPreview.available
        previewMode: markdownPreview.mode
        previewWidth: markdownPreview.splitWidth
        currentFilePath: root.editorController.currentFilePath()
        currentDocId: root.editorController.currentDocId
        workspaceRoot: root.shellController.workspaceRoot
        onPreviewModeSelected: function(mode) {
            markdownPreview.setMode(mode);
        }
        onPreviewResizeRequested: function(delta) {
            markdownPreview.resizeSplit(delta);
        }
        onPreviewLocalFileRequested: function(path) {
            // O caminho ja' passou pela politica (dentro do projeto, sem
            // esquema estranho). Abrir e' o mesmo gesto de sempre; a linha 1
            // existe porque este caminho pede posicao.
            root.editorController.openDiagnostic(path, 1, 1);
        }
        onPreviewWebUrlRequested: function(url) {
            Qt.openUrlExternally(url);
        }
        onTabSelected: function(docId) {
            root.editorController.selectDocument(docId);
        }
        onTabCloseRequested: function(docId) {
            root.editorController.closeDocument(docId);
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
        onUsagesDismissRequested: root.editorController.usagesVisible = false
        onHoverDismissRequested: root.editorController.hoverVisible = false
        onIndentRequested: root.editorController.textEditing.indentSelection()
        onUnindentRequested: root.editorController.textEditing.unindentSelection()
        onNewlineRequested: root.editorController.textEditing.insertNewline()
        onCloserBraceRequested: root.editorController.textEditing.insertCloserBrace()
        onSmartHomeRequested: function(extendSelection) {
            root.editorController.textEditing.smartHome(extendSelection);
        }
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

    DropArea {
        id: editorFileDrop
        anchors.fill: editorPane
        z: 2
        onEntered: function(drag) {
            drag.accepted = root.internalDroppedFile(drag) !== ""
                            || root.localDroppedFile(drag) !== "";
        }
        onDropped: function(drop) {
            const path = root.internalDroppedFile(drop) || root.localDroppedFile(drop);
            if (path === "") { drop.accepted = false; return; }
            drop.accept(Qt.CopyAction);
            if (path.startsWith(root.shellController.workspaceRoot + "/"))
                root.editorController.openDiagnostic(path, 1, 1);
            else
                root.editorController.externalPreview.open(path);
        }
    }

    // O que FLUTUA sobre o editor tem host proprio, e le' os controllers
    // direto — sem passar por trinta propriedades do painel.
    // Ver ShellEditorOverlayHost.qml.
    ShellEditorOverlayHost {
        id: overlayHost

        anchors.fill: editorPane
        contentTop: editorPane.overlayTop

        editorController: root.editorController
        projectTree: root.projectTree
        shellController: root.shellController
        editorSurface: editorPane.editorSurface
    }

    // O diff/o commit escolhido na janela do Git abre AQUI, sobre o editor,
    // como uma aba de visualizacao (E3-3); o x devolve o editor intacto.
    GitViewerPane {
        anchors.fill: editorPane
        z: 20
        inspector: root.gitController ? root.gitController.inspector : null
        workspaceRoot: root.editorController.workspaceRoot
        onActiveChanged: if (active) forceActiveFocus()
    }
}
