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

    property var editorController
    property var shellController
    property var debugController
    property var gitController
    property var coverageController
    property var diagnosticsController
    property var projectTree

    property bool workspaceOpen: false
    property var indexController: null

    property alias editorSurface: editorPane.editorSurface

    function focusCreateDialog() {
        overlayHost.focusCreateDialog();
    }

    function openRenameDialogWithName(name) {
        overlayHost.openRenameDialogWithName(name);
    }

    function openGoToLineDialog(prefill) {
        overlayHost.openGoToLineDialog(prefill);
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

    function currentFileBreadcrumb(currentTab) {
        if (currentTab < 0) {
            return "";
        }
        return root.shellController.relativeToRoot(
            root.editorController.currentFilePath());
    }

    function currentFileExecutionLine(currentTab, stoppedFile, stoppedLine) {
        if (stoppedFile === ""
                || stoppedFile !== root.editorController.currentFilePath()) {
            return 0;
        }
        return stoppedLine;
    }

    EditorPane {
        id: editorPane

        anchors.fill: parent

        workspaceOpen: root.workspaceOpen
        filesModel: root.editorController.filesModel
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
            root.editorController.requestCodeActions();
        }
        onGutterLineClicked: function(line) {
            root.debugController.toggleBreakpoint(
                root.editorController.currentFilePath(), line);
        }
        breadcrumbPath: root.currentFileBreadcrumb(
            root.editorController.currentTab)
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
        watchError: root.editorController.watchError
        outlineItems: root.editorController.syntaxOutline
        symbols: root.indexController ? root.indexController.symbols : null
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
        onUsagesDismissRequested: root.editorController.usagesVisible = false
        onHoverDismissRequested: root.editorController.hoverVisible = false
        onIndentRequested: root.editorController.indentEditorSelection()
        onUnindentRequested: root.editorController.unindentEditorSelection()
        onNewlineRequested: root.editorController.insertEditorNewline()
        onCloserBraceRequested: root.editorController.insertEditorCloserBrace()
        onSmartHomeRequested: function(extendSelection) {
            root.editorController.editorSmartHome(extendSelection);
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
