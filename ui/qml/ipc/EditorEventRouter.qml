import QtQuick

Item {
    id: root

    property var coreClient: null
    property var editorController: null

    visible: false

    Connections {
        target: root.coreClient

        function onFileLoaded(path, content) {
            root.editorController.handleFileLoaded(path, content);
        }

        // M4.3: o core foi recuperado de um crash — o LSP subiu do zero.
        // Re-sincroniza o arquivo ativo (didOpen + semantic tokens), então
        // highlighting e diagnósticos do arquivo atual voltam sozinhos.
        function onRecovered() {
            root.editorController.refreshSemanticTokens();
        }

        // M4.3b: um servidor LSP reiniciou (auto por timeouts ou comando) —
        // o servidor novo não conhece o arquivo aberto; re-sincroniza (mesmo
        // caminho do recovered): semantic_tokens faz didOpen de novo.
        function onLspRestarted(language) {
            root.editorController.refreshSemanticTokens();
        }

        // Etapa 4 (roadmap 30): apos um cmake.configure o core FECHOU os
        // documentos C/C++ que o servidor conhecia — reenviar didOpen sem
        // fechar seria inerte (curto-circuito por hash no lsp/sync). Aqui a UI
        // reabre o arquivo ativo com o BUFFER real, e o clangd recompila com as
        // flags novas. Sem isto, o arquivo fica sem diagnostico ate o usuario
        // digitar.
        function onLspDocumentsClosed(language, count) {
            root.editorController.refreshSemanticTokens();
        }

        function onFileSaved(path) {
            root.editorController.handleFileSaved(path);
        }

        function onFilesChanged(changes) {
            root.editorController.handleExternalChanges(changes);
        }

        function onFileSaveFailed(path, message) {
            root.editorController.handleFileSaveFailed(path, message);
        }

        function onFileWatchFailed(message) {
            root.editorController.handleFileWatchFailed(message);
        }

        function onFileFormatted(path, text, changed) {
            root.editorController.handleFormatResolved(path, text, changed);
        }

        function onSessionRestored(files, activeFile) {
            root.editorController.restoreSession(files, activeFile);
        }

        // M-S1 (DocsPublic/seguranca/23): rascunhos não salvos recuperados de um crash.
        function onDraftsRecovered(drafts) {
            root.editorController.restoreDrafts(drafts);
        }

        function onLspDefinitionResolved(path, line, column) {
            root.editorController.openDiagnostic(path, line, column);
        }

        function onLspHoverResolved(content) {
            root.editorController.handleHoverResolved(content);
        }

        function onLspSemanticTokensResolved(path, version, tokens) {
            root.editorController.handleSemanticTokensResolved(path, version, tokens);
        }

        function onSyntaxTreeResolved(path, version, language, hasErrors,
                                      highlights, foldingRanges, outline, locals) {
            root.editorController.handleSyntaxTreeResolved(
                path, version, language, hasErrors, highlights,
                foldingRanges, outline, locals);
        }

        function onLspSwitchSourceHeaderResolved(path) {
            root.editorController.handleSwitchSourceHeader(path);
        }

        function onLspCompletionResolved(items, isIncomplete) {
            root.editorController.handleCompletionResolved(items, isIncomplete);
        }

        function onLspReferencesResolved(references) {
            root.editorController.handleReferencesResolved(references);
        }

        function onLspCodeActionsResolved(actions) {
            root.editorController.handleCodeActionsResolved(actions);
        }

        function onLspWorkspaceEditPreviewResolved(transactionId, title, files, edits) {
            root.editorController.handleWorkspaceEditPreview(transactionId, title,
                                                             files, edits);
        }

        function onLspWorkspaceEditApplied(files, title, edits) {
            root.editorController.handleWorkspaceEditApplied(files);
        }

        function onLspWorkspaceEditCancelled(transactionId) {
            root.editorController.handleWorkspaceEditCancelled();
        }

        function onRequestFailed(method, message) {
            if (method === "lsp.hover" || method === "lsp.completion"
                    || method === "lsp.rename" || method === "lsp.codeActions"
                    || method === "lsp.applyCodeAction"
                    || method === "lsp.workspaceEdit.apply"
                    || method === "lsp.workspaceEdit.cancel"
                    || method === "format.text") {
                root.editorController.handleRequestFailed(method, message);
            }
        }
    }
}
