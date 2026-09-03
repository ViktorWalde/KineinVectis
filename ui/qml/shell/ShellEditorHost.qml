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
    property var diagnosticsController
    property var projectTree

    property bool workspaceOpen: false

    property alias editorSurface: editorPane.editorSurface

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
        breadcrumbPath: root.currentFileBreadcrumb(
            root.editorController.currentTab)
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
        onSaveRequested: root.editorController.saveCurrentFile()
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
}
