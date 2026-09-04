import QtQuick
import KineinVectis

// Composition host do que FLUTUA sobre o editor: barra de Find/Replace, os
// quatro popups (hover, completacao, acoes, usos) e os quatro dialogos
// (criar, renomear, previa de workspace edit, ir para linha).
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). O EditorPane tinha 460 linhas e,
// depois de sairem dele os widgets de verdade, quase nao desenhava mais nada:
// tres linhas de cartao e o resto REPASSE. Trinta propriedades e vinte e uma
// sinais existiam so' para levar um valor do controller ate' um popup e um
// clique de volta. Esse repasse E' a divida — cortar o painel em dois pedacos
// que continuam repassando seria o erro que a §4 regra 9 registra no caso
// AppDomains: "13 propriedades de pass-through, nada ficou mais claro".
//
// A saida que a §4 regra 8 manda usar: dividir a COMPOSICAO por area, fazendo
// a contagem de arquivos crescer. Aqui os overlays leem o controller
// DIRETAMENTE, e as trinta propriedades de travessia simplesmente deixam de
// existir — nao mudaram de arquivo, sumiram. Mesmo padrao do ShellEditorHost e
// do ShellHeaderHost.
//
// GEOMETRIA. Este host cobre exatamente a area do EditorPane. `contentTop`
// desce o que flutua no topo para baixo da faixa de conflito externo, que vive
// dentro do painel; os popups ancorados ao cursor usam
// `editorSurface.cursorPointIn(root)` e nao dependem disso.
Item {
    id: root

    property var editorController
    property var projectTree
    property var shellController
    // A superficie de texto do EditorPane, para posicionar pelo cursor.
    property var editorSurface

    // Onde termina o cabecalho do painel (abas, trilha, faixa de conflito).
    property real contentTop: 0

    readonly property real maxOverlayWidth: root.width - 4 * Theme.spacingSmall

    function focusCreateDialog() {
        createDialog.resetAndFocus();
    }

    function openRenameDialogWithName(name) {
        renameDialog.openWithName(name);
    }

    function openGoToLineDialog(prefill) {
        goToLineDialog.openWithValue(prefill);
    }

    function focusFindBar() {
        findBar.focusQuery();
    }

    // Canto superior esquerdo do cursor, em coordenadas DESTE host.
    function cursorPoint() {
        return editorSurface.cursorPointIn(root);
    }

    // X de um popup ancorado ao cursor, preso dentro da largura visivel.
    function popupX(popupWidth) {
        return Math.max(Theme.spacingSmall,
                        Math.min(cursorPoint().x,
                                 root.width - popupWidth - Theme.spacingSmall));
    }

    // Y de um popup ancorado ao cursor: ABAIXO da linha, ou acima quando nao
    // cabe embaixo. Sem esse giro, a completacao no fim do arquivo nasce fora
    // da tela.
    function popupY(popupHeight) {
        const point = cursorPoint();
        const below = point.y + editorSurface.cursorRectangle.height + 4;
        if (below + popupHeight > root.height - Theme.spacingSmall) {
            return Math.max(Theme.spacingSmall, point.y - popupHeight - 4);
        }
        return below;
    }

    // D1b (docs/roadmaps/24): flutua no canto superior direito (VS Code), acima
    // do texto mas abaixo dos popups de completacao/acoes.
    EditorFindBar {
        id: findBar

        visible: root.editorController.findBarVisible
                 && root.editorController.currentTab >= 0
        z: 22
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: root.contentTop + Theme.spacingSmall
        anchors.rightMargin: 2 * Theme.spacingSmall
        maxAvailableWidth: root.maxOverlayWidth
        replaceMode: root.editorController.findReplaceMode
        query: root.editorController.findQuery
        replacement: root.editorController.findReplacement
        caseSensitive: root.editorController.findCaseSensitive
        wholeWord: root.editorController.findWholeWord
        useRegex: root.editorController.findUseRegex
        invalidRegex: root.editorController.findInvalidRegex
        matchCount: root.editorController.findMatchCount
        currentMatch: root.editorController.findCurrentDisplay

        onQueryEdited: text => root.editorController.setFindQuery(text)
        onReplacementEdited: text => root.editorController.setFindReplacement(text)
        onFindNextRequested: root.editorController.findNext()
        onFindPreviousRequested: root.editorController.findPrevious()
        onReplaceRequested: root.editorController.replaceFindCurrent()
        onReplaceAllRequested: root.editorController.replaceFindAll()
        onCaseToggleRequested: root.editorController.toggleFindCase()
        onWholeWordToggleRequested: root.editorController.toggleFindWholeWord()
        onRegexToggleRequested: root.editorController.toggleFindRegex()
        onCloseRequested: root.editorController.closeFind()
    }

    EditorHoverPopup {
        visible: root.editorController.hoverVisible
                 && root.editorController.hoverText !== ""
        z: 20
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: root.contentTop + 2 * Theme.spacingSmall
        anchors.rightMargin: 2 * Theme.spacingSmall
        hoverText: root.editorController.hoverText
        maxAvailableWidth: root.maxOverlayWidth

        onDismissRequested: root.editorController.hoverVisible = false
    }

    EditorCompletionPopup {
        id: completionPopup

        visible: root.editorController.completionVisible
                 && root.editorController.currentTab >= 0
        z: 30
        itemsModel: root.editorController.completionModel
        completionCount: root.editorController.completionModel.count
        currentIndex: root.editorController.completionIndex
        maxAvailableWidth: root.maxOverlayWidth
        x: root.popupX(completionPopup.width)
        y: root.popupY(completionPopup.height)

        onCompletionActivated: index => {
            root.editorController.completionIndex = index;
            root.editorController.acceptCompletion();
        }
    }

    EditorActionsPopup {
        id: actionsPopup

        visible: root.editorController.actionsVisible
                 && root.editorController.currentTab >= 0
        z: 30
        itemsModel: root.editorController.actionsModel
        actionCount: root.editorController.actionsModel.count
        currentIndex: root.editorController.actionsIndex
        maxAvailableWidth: root.maxOverlayWidth
        x: root.popupX(actionsPopup.width)
        y: root.popupY(actionsPopup.height)

        onActionActivated: index => root.editorController.applyCodeAction(index)
        onDismissRequested: root.editorController.dismissActions()
    }

    EditorUsagesPopup {
        visible: root.editorController.usagesVisible
        z: 25
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: root.contentTop + 2 * Theme.spacingSmall
        anchors.rightMargin: 2 * Theme.spacingSmall
        itemsModel: root.editorController.usagesModel
        usageCount: root.editorController.usagesModel.count
        maxAvailableWidth: root.maxOverlayWidth

        onCloseRequested: root.editorController.usagesVisible = false
        onUsageOpenRequested: (path, line, column) =>
            root.editorController.openDiagnostic(path, line, column)
    }

    ProjectCreateDialog {
        id: createDialog

        visible: root.projectTree.createDialogVisible
        z: 40
        anchors.centerIn: parent
        dialogKind: root.projectTree.createDialogKind
        parentDisplayPath: root.shellController.relativeToRoot(
                               root.projectTree.createDialogParentPath)
        errorText: root.projectTree.createDialogError
        maxAvailableWidth: root.maxOverlayWidth

        onConfirmRequested: root.projectTree.confirmCreateEntry(
                                createDialog.currentName())
        onCancelRequested: {
            root.projectTree.createDialogVisible = false;
            root.editorController.focusEditor();
        }
    }

    SymbolRenameDialog {
        id: renameDialog

        visible: root.editorController.renameDialogVisible
        z: 40
        anchors.centerIn: parent
        errorText: root.editorController.renameError
        maxAvailableWidth: root.maxOverlayWidth

        onConfirmRequested: root.editorController.confirmRename(
                                renameDialog.currentName())
        onCancelRequested: {
            root.editorController.renameDialogVisible = false;
            root.editorController.focusEditor();
        }
    }

    EditorWorkspaceEditPreviewDialog {
        visible: root.editorController.workspaceEditPreviewVisible
        z: 45
        anchors.centerIn: parent
        operationTitle: root.editorController.workspaceEditTitle
        files: root.editorController.workspaceEditFiles
        editCount: root.editorController.workspaceEditCount
        errorText: root.editorController.workspaceEditError
        maxAvailableWidth: root.maxOverlayWidth
        maxAvailableHeight: root.height - 4 * Theme.spacingSmall

        onApplyRequested: root.editorController.applyWorkspaceEdit()
        onCancelRequested: root.editorController.cancelWorkspaceEdit()
    }

    EditorGoToLineDialog {
        id: goToLineDialog

        visible: root.editorController.goToLineVisible
        z: 40
        anchors.centerIn: parent
        maxAvailableWidth: root.maxOverlayWidth

        onConfirmRequested: root.editorController.confirmGoToLine(
                                goToLineDialog.currentValue())
        onCancelRequested: root.editorController.cancelGoToLine()
    }
}
