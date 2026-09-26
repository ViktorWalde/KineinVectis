import QtQuick

// Espelho do EditorEventRouter. Os dois sentidos do IPC do editor sempre
// existiram, mas so um tinha casa: o EventRouter traz o que o core MANDA
// (fileLoaded, recovered...), e o que o controller PEDE ao core morava solto no
// Main.qml. Era metade do peso do composition root — o bloco do
// EditorController sozinho tinha 72 linhas, quase todas fiacao de pedido.
//
// Aqui entra SO pedido ao core. Fiacao de controller para host (abrir dialogo,
// focar find bar) nao e IPC e continua no Main.qml, onde os dois se enxergam.
Item {
    id: root

    property var coreClient: null
    property var editorController: null

    visible: false

    Connections {
        target: root.editorController

        function onReadFileRequested(path) {
            root.coreClient.readFile(path);
        }

        function onWriteFileRequested(path, content, expectedContent) {
            root.coreClient.writeFile(path, content, expectedContent);
        }

        function onDraftSaveRequested(path, content) {
            root.coreClient.draftSave(path, content);
        }

        function onDraftClearRequested(path) {
            root.coreClient.draftClear(path);
        }

        // E1: o fallback local ja' esta' na tela; isto pergunta a' gramatica.
        // `version` e' a mesma do `syntaxTree.update`, para o core poder dizer
        // se respondeu sobre a arvore certa.
        function onIndentRequested(path, version, line, column, trigger) {
            root.coreClient.requestSyntaxIndent(path, version, line, column, trigger);
        }

        function onFormatRequested(path, content) {
            root.coreClient.formatFile(path, content);
        }

        function onSaveSessionRequested(files, activeFile) {
            root.coreClient.saveSession(files, activeFile);
        }

        function onFileChangedNotificationRequested(path, content) {
            root.coreClient.notifyFileChanged(path, content);
        }

        function onCompletionRequested(path, content, line, column) {
            root.coreClient.requestCompletion(path, content, line, column);
        }

    }

    // Segundo bloco, e a fronteira e' de DONO, nao de conveniencia: desde
    // 2026-09-02 quem emite os pedidos de linguagem e o
    // `EditorLanguageController`, nao o `EditorController`. Escutar no objeto
    // errado nao quebra o build — o handler simplesmente nunca dispara, que e a
    // falha silenciosa da ARCHITECTURE.md §8. Ligar aqui, explicitamente, e o
    // que o `RuntimeRequestRouter` ja fazia com o `runConfigController`.
    Connections {
        target: root.editorController.language

        function onCodeActionsRequested(path, content, line, column) {
            root.coreClient.requestCodeActions(path, content, line, column);
        }

        function onCodeActionApplyRequested(path, content, actionIndex) {
            root.coreClient.applyCodeAction(path, content, actionIndex);
        }

        function onWorkspaceEditApplyRequested(transactionId) {
            root.coreClient.applyWorkspaceEdit(transactionId);
        }

        function onWorkspaceEditCancelRequested(transactionId) {
            root.coreClient.cancelWorkspaceEdit(transactionId);
        }

        function onSwitchSourceHeaderRequested(path, content) {
            root.coreClient.requestSwitchSourceHeader(path, content);
        }

        function onDefinitionRequested(path, content, line, column) {
            root.coreClient.requestDefinition(path, content, line, column);
        }

        function onHoverRequested(path, content, line, column) {
            root.coreClient.requestHover(path, content, line, column);
        }

        function onReferencesRequested(path, content, line, column) {
            root.coreClient.requestReferences(path, content, line, column);
        }

        function onRenameRequested(path, content, line, column, newName) {
            root.coreClient.requestRename(path, content, line, column, newName);
        }
    }

    // Terceiro bloco: o REALCE tem dono proprio desde 2026-09-02. Tres blocos
    // sao tres donos, e essa e a informacao que o arquivo carrega.
    Connections {
        target: root.editorController.highlight

        function onSemanticTokensRequested(path, content, version) {
            root.coreClient.requestSemanticTokens(path, content, version);
        }

        function onSyntaxTreeRequested(path, content, version) {
            root.coreClient.requestSyntaxTree(path, content, version);
        }
    }
}
