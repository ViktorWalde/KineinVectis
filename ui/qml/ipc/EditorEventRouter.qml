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

        function onFileSaved(path) {
            root.editorController.handleFileSaved(path);
        }

        function onLspDefinitionResolved(path, line, column) {
            root.editorController.openDiagnostic(path, line, column);
        }

        function onLspHoverResolved(content) {
            root.editorController.handleHoverResolved(content);
        }

        function onLspSemanticTokensResolved(tokens) {
            root.editorController.handleSemanticTokensResolved(tokens);
        }

        function onLspCompletionResolved(items) {
            root.editorController.handleCompletionResolved(items);
        }

        function onLspReferencesResolved(references) {
            root.editorController.handleReferencesResolved(references);
        }

        function onLspRenameApplied(files, edits) {
            root.editorController.handleRenameApplied(files);
        }

        function onRequestFailed(method, message) {
            if (method === "lsp.hover" || method === "lsp.completion"
                    || method === "lsp.rename") {
                root.editorController.handleRequestFailed(method, message);
            }
        }
    }
}
