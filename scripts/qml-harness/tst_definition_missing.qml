import QtQuick
import KineinVectis

// 16.7, F5 (roadmaps/59 §7.1.7): o Ctrl+B sem destino diz que nao ha'
// declaracao, no popup de hover, em vez de ficar mudo; com destino, abre. O
// EditorEventRouter REAL; o bridge falso declara os sinais que ele ouve
// (gerados dos proprios handlers).
Item {
    id: root
    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    QtObject {
        id: bridge
        signal fileLoaded(var path, var content)
        signal externalFileLoaded(var path, var content)
        signal externalFileLoadFailed(var path, var message)
        signal recovered()
        signal lspStatusChanged(var language, var status, var message)
        signal lspRestarted(var language)
        signal lspDocumentsClosed(var language, var count)
        signal fileSaved(var path)
        signal filesChanged(var changes)
        signal fileSaveFailed(var path, var message)
        signal fileWatchFailed(var message)
        signal fileFormatted(var path, var text, var changed)
        signal sessionRestored(var files, var activeFile)
        signal draftsRecovered(var drafts)
        signal lspDefinitionResolved(var path, var line, var column)
        signal lspHoverResolved(var content)
        signal lspSemanticTokensResolved(var path, var version, var tokens)
        signal syntaxTreeResolved(var path, var version, var language, var hasErrors, var highlights, var foldingRanges, var outline, var locals)
        signal syntaxIndentResolved(var path, var version, var language, var level)
        signal lspSwitchSourceHeaderResolved(var path)
        signal lspCompletionResolved(var items, var isIncomplete)
        signal lspReferencesResolved(var references)
        signal lspCodeActionsResolved(var actions)
        signal lspWorkspaceEditPreviewResolved(var transactionId, var title, var files, var edits)
        signal lspWorkspaceEditApplied(var files, var title, var edits)
        signal lspWorkspaceEditCancelled(var transactionId)
        signal requestFailed(var method, var message)
    }
    QtObject {
        id: editor
        property var hovers: []
        property var opened: []
        function handleHoverResolved(content) { editor.hovers = editor.hovers.concat([content]); }
        function openDiagnostic(path, line, column) { editor.opened = editor.opened.concat([path + ":" + line]); }
    }
    EditorEventRouter {
        coreClient: bridge
        editorController: editor
    }

    Component.onCompleted: {
        bridge.lspDefinitionResolved("", 1, 1);
        root.check(editor.opened.length === 0, "sem destino, nada abre");
        root.check(editor.hovers.length === 1 && editor.hovers[0].indexOf("Nenhuma declaração") >= 0,
            "sem destino, o aviso: " + JSON.stringify(editor.hovers));
        bridge.lspDefinitionResolved("/p/DataSourcePanel.qml", 18, 1);
        root.check(JSON.stringify(editor.opened) === '["/p/DataSourcePanel.qml:18"]', "com destino, abre");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
