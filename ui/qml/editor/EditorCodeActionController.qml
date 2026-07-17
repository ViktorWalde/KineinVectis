import QtQuick

// Code actions do LSP e o preview/apply de workspace edits — o fluxo de
// REFACTORING do editor. Extraido do EditorController em 2026-07-17 (Fase 1.1):
// pedir uma code action, escolher uma na lista, e o servidor devolve um
// workspace edit que e' PREVIEWADO antes de aplicar (ou cancelar). E' um estado
// e uma logica proprios, nao fachada — por isso viram dono proprio, ao contrario
// da edicao de linha, que so' delega ao textController.
//
// Nao fala com o coreClient direto: emite os mesmos signals que o
// EditorController ja emitia, e o EditorRequestRouter continua sendo quem
// despacha. As dependencias de volta (arquivo atual, foco, rename) derivam
// todas de surfaceBridge/documentController, injetados como nos outros
// sub-controllers.
QtObject {
    id: root

    required property var surfaceBridge
    required property var textController
    required property var documentController

    // Popups concorrentes (completion, hover) sao do COORDENADOR: pedir uma code
    // action deve fecha-los, mas quem sabe que eles existem e' o EditorController.
    signal dismissConcurrentPopups()
    signal codeActionsRequested(string path, string content, int line, int column)
    signal codeActionApplyRequested(string path, string content, int actionIndex)
    signal workspaceEditApplyRequested(string transactionId)
    signal workspaceEditCancelRequested(string transactionId)

    property ListModel actionsModel: ListModel {}
    property bool actionsVisible: false
    property int actionsIndex: 0

    property bool workspaceEditPreviewVisible: false
    property string workspaceEditTransactionId: ""
    property string workspaceEditTitle: ""
    property var workspaceEditFiles: []
    property int workspaceEditCount: 0
    property string workspaceEditError: ""

    function editorReady() {
        return root.surfaceBridge.ready();
    }

    function currentFilePath() {
        return root.documentController.currentFilePath();
    }

    function editableFileOpen() {
        return currentFilePath() !== "" && editorReady();
    }

    function requestCodeActions() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        root.dismissConcurrentPopups();
        const position = root.textController.cursorLineColumn();
        root.codeActionsRequested(path, root.surfaceBridge.text(), position.line, position.column);
    }

    function handleCodeActionsResolved(actions) {
        root.actionsModel.clear();
        for (let i = 0; i < actions.length; i++) {
            root.actionsModel.append({
                title: actions[i].title,
                kind: actions[i].kind !== undefined ? actions[i].kind : ""
            });
        }
        root.actionsIndex = 0;
        root.actionsVisible = true;
    }

    function moveActions(delta) {
        if (root.actionsModel.count === 0) {
            return;
        }
        const next = root.actionsIndex + delta;
        root.actionsIndex = Math.max(0, Math.min(root.actionsModel.count - 1, next));
    }

    function applyCodeAction(index) {
        const path = currentFilePath();
        if (!root.actionsVisible || path === "" || !editorReady()) {
            return;
        }
        if (index < 0 || index >= root.actionsModel.count) {
            dismissActions();
            return;
        }
        root.actionsVisible = false;
        root.codeActionApplyRequested(path, root.surfaceBridge.text(), index);
    }

    function applySelectedAction() {
        applyCodeAction(root.actionsIndex);
    }

    function dismissActions() {
        root.actionsVisible = false;
        root.actionsModel.clear();
    }

    function handleWorkspaceEditPreview(transactionId, title, files, edits) {
        root.workspaceEditTransactionId = transactionId;
        root.workspaceEditTitle = title;
        root.workspaceEditFiles = files;
        root.workspaceEditCount = edits;
        root.workspaceEditError = "";
        root.workspaceEditPreviewVisible = true;
    }

    function applyWorkspaceEdit() {
        if (root.workspaceEditTransactionId === "") {
            return;
        }
        root.workspaceEditError = "";
        root.workspaceEditApplyRequested(root.workspaceEditTransactionId);
    }

    function cancelWorkspaceEdit() {
        if (root.workspaceEditTransactionId === "") {
            resetWorkspaceEditPreview();
            root.surfaceBridge.focusEditor();
            return;
        }
        root.workspaceEditCancelRequested(root.workspaceEditTransactionId);
    }

    function handleWorkspaceEditApplied(files) {
        resetWorkspaceEditPreview();
        root.documentController.handleRenameApplied(files);
        root.surfaceBridge.focusEditor();
    }

    function handleWorkspaceEditCancelled() {
        resetWorkspaceEditPreview();
        root.surfaceBridge.focusEditor();
    }

    // Falha ao APLICAR o workspace edit: mostra o erro e mantem o preview aberto
    // enquanto houver transacao (o usuario decide retry ou cancela).
    function handleApplyFailed(message) {
        root.workspaceEditError = message;
        root.workspaceEditPreviewVisible = root.workspaceEditTransactionId !== "";
    }

    function resetWorkspaceEditPreview() {
        root.workspaceEditPreviewVisible = false;
        root.workspaceEditTransactionId = "";
        root.workspaceEditTitle = "";
        root.workspaceEditFiles = [];
        root.workspaceEditCount = 0;
        root.workspaceEditError = "";
    }
}
