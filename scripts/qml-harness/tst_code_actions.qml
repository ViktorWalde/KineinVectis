import QtQuick
// Carrega o EditorCodeActionController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// Cobre o fluxo de refactoring extraido do EditorController na Fase 1.1: pedir
// code actions, navegar/aplicar, e o ciclo preview -> apply/cancel/falha do
// workspace edit. O controller nasce COM sonda — a E1 (auto-close) nao tinha, e
// isso custou o bug do autocomplete passar despercebido.
Item {
    id: root
    width: 100; height: 100

    // Fakes minimos com a mesma superficie que o controller usa.
    QtObject {
        id: fakeBridge
        property bool readyState: true
        property string focoContagem: ""
        function ready() { return readyState; }
        function text() { return "conteudo do editor"; }
        function focusEditor() { focoContagem += "x"; }
    }

    QtObject {
        id: fakeDocs
        property string filePath: "/w/src/main.rs"
        property var renameAplicado: null
        function currentFilePath() { return filePath; }
        function handleRenameApplied(files) { renameAplicado = files; }
    }

    QtObject {
        id: fakeText
        function cursorLineColumn() { return { line: 4, column: 2 }; }
    }

    // Captura dos signals emitidos (o que o EditorRequestRouter receberia).
    property var ultimoCodeActionsRequest: null
    property var ultimoApplyRequest: null
    property string ultimoWorkspaceApply: ""
    property string ultimoWorkspaceCancel: ""
    property int dismissConcorrentes: 0

    Item {
        id: invisibleParent
        visible: false

        EditorCodeActionController {
            id: actions
            surfaceBridge: fakeBridge
            documentController: fakeDocs
            textController: fakeText
            onDismissConcurrentPopups: root.dismissConcorrentes += 1
            onCodeActionsRequested: (path, content, line, column) => {
                root.ultimoCodeActionsRequest = { path, content, line, column };
            }
            onCodeActionApplyRequested: (path, content, actionIndex) => {
                root.ultimoApplyRequest = { path, content, actionIndex };
            }
            onWorkspaceEditApplyRequested: (t) => root.ultimoWorkspaceApply = t
            onWorkspaceEditCancelRequested: (t) => root.ultimoWorkspaceCancel = t
        }
    }

    Component.onCompleted: {
        let falhas = 0;

        // 1) request: emite com posicao do cursor e fecha popups concorrentes.
        actions.requestCodeActions();
        if (root.dismissConcorrentes !== 1) falhas += 1;
        if (root.ultimoCodeActionsRequest === null
                || root.ultimoCodeActionsRequest.line !== 4
                || root.ultimoCodeActionsRequest.column !== 2) falhas += 2;

        // 1b) sem arquivo aberto NAO pede (nem fecha popups de novo).
        fakeDocs.filePath = "";
        root.ultimoCodeActionsRequest = null;
        actions.requestCodeActions();
        if (root.ultimoCodeActionsRequest !== null) falhas += 4;
        if (root.dismissConcorrentes !== 1) falhas += 8;
        fakeDocs.filePath = "/w/src/main.rs";

        // 2) resolved popula o modelo, zera o indice e abre.
        actions.handleCodeActionsResolved([
            { title: "Import symbol", kind: "quickfix" },
            { title: "Generate impl" }
        ]);
        if (actions.actionsModel.count !== 2) falhas += 16;
        if (!actions.actionsVisible || actions.actionsIndex !== 0) falhas += 32;

        // 3) navegacao satura nas bordas (nao da wrap).
        actions.moveActions(1);
        if (actions.actionsIndex !== 1) falhas += 64;
        actions.moveActions(1);
        if (actions.actionsIndex !== 1) falhas += 128; // ja no fim
        actions.moveActions(-5);
        if (actions.actionsIndex !== 0) falhas += 256;

        // 4) aplicar emite o indice certo e fecha a lista.
        actions.applySelectedAction();
        if (root.ultimoApplyRequest === null
                || root.ultimoApplyRequest.actionIndex !== 0) falhas += 512;
        if (actions.actionsVisible) falhas += 1024;

        // 4b) aplicar sem lista visivel nao emite.
        root.ultimoApplyRequest = null;
        actions.applyCodeAction(0);
        if (root.ultimoApplyRequest !== null) falhas += 2048;

        // 5) preview do workspace edit guarda o estado e abre.
        actions.handleWorkspaceEditPreview("tx-1", "Rename foo", ["/w/a.rs"], 3);
        if (!actions.workspaceEditPreviewVisible
                || actions.workspaceEditTransactionId !== "tx-1"
                || actions.workspaceEditCount !== 3) falhas += 4096;

        // 6) aplicar emite a transacao.
        actions.applyWorkspaceEdit();
        if (root.ultimoWorkspaceApply !== "tx-1") falhas += 8192;

        // 7) applied: reseta, delega rename ao documentController, refoca.
        actions.handleWorkspaceEditApplied(["/w/a.rs"]);
        if (actions.workspaceEditPreviewVisible
                || actions.workspaceEditTransactionId !== "") falhas += 16384;
        if (fakeDocs.renameAplicado === null) falhas += 32768;

        // 8) falha ao aplicar: mostra erro e mantem o preview enquanto ha
        // transacao (o usuario decide).
        actions.handleWorkspaceEditPreview("tx-2", "t", [], 1);
        actions.handleApplyFailed("conflito no disco");
        if (actions.workspaceEditError !== "conflito no disco") falhas += 65536;
        if (!actions.workspaceEditPreviewVisible) falhas += 131072;

        // 9) cancelar sem transacao reseta e refoca localmente (nao emite).
        actions.resetWorkspaceEditPreview();
        root.ultimoWorkspaceCancel = "";
        actions.cancelWorkspaceEdit();
        if (root.ultimoWorkspaceCancel !== "") falhas += 262144;

        if (falhas !== 0) console.error("FALHAS bitmask=" + falhas);
        Qt.exit(falhas === 0 ? 0 : 1);
    }
}
