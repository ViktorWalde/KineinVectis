import QtQuick
import KineinVectis

// 16.7, F10 (roadmaps/59 §7.1.7): o Ctrl+B num cabecalho do sistema
// (`/usr/include/...`) abre a aba somente leitura do fs.readExternal e leva o
// cursor ao destino; dentro do projeto, continua o fs.read. O
// EditorRequestRouter, o EditorDocumentController e o
// EditorExternalPreviewController REAIS; os sinais do editor falso sao
// gerados dos handlers do proprio router.
Item {
    id: root
    width: 640
    height: 300
    property int failures: 0
    property var reads: []
    property var externalReads: []

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    TextEdit {
        id: surface
        width: parent.width
        height: parent.height
        property string path: ""
        function setFilePath(path) { surface.path = path; }
        function focusEditor() { surface.forceActiveFocus(); }
    }
    EditorSurfaceBridge { id: bridge; editorSurface: surface }
    EditorDocumentController {
        id: documents
        workspaceRoot: "/p"
        surfaceBridge: bridge
        // No EditorController de verdade, o pedido sobe por ele.
        onReadFileRequested: path => fakeEditor.readFileRequested(path)
    }
    EditorExternalPreviewController {
        id: preview
        workspaceRoot: "/p"
        documentController: documents
    }
    QtObject {
        id: fakeEditor
        property var externalPreview: preview
        property var language: fakeLanguage
        property var highlight: fakeHighlight
        signal readFileRequested(var path)
        signal writeFileRequested(var path, var content, var expectedContent)
        signal draftSaveRequested(var path, var content)
        signal draftClearRequested(var path)
        signal indentRequested(var path, var version, var line, var column, var trigger)
        signal formatRequested(var path, var content)
        signal saveSessionRequested(var files, var activeFile)
        signal fileChangedNotificationRequested(var path, var content)
        signal completionRequested(var path, var content, var line, var column)
    }
    QtObject {
        id: fakeLanguage
        signal codeActionsRequested(var path, var content, var line, var column)
        signal codeActionApplyRequested(var path, var content, var actionIndex)
        signal workspaceEditApplyRequested(var transactionId)
        signal workspaceEditCancelRequested(var transactionId)
        signal switchSourceHeaderRequested(var path, var content)
        signal definitionRequested(var path, var content, var line, var column)
        signal hoverRequested(var path, var content, var line, var column)
        signal referencesRequested(var path, var content, var line, var column)
        signal renameRequested(var path, var content, var line, var column, var newName)
    }
    QtObject {
        id: fakeHighlight
        signal semanticTokensRequested(var path, var content, var version)
        signal syntaxTreeRequested(var path, var content, var version)
    }
    QtObject {
        id: fakeCore
        function readFile(path) { root.reads.push(path); }
        function readExternalFile(path) { root.externalReads.push(path); }
    }
    EditorRequestRouter {
        coreClient: fakeCore
        editorController: fakeEditor
    }

    Component.onCompleted: {
        const header = "/usr/include/c++/16/iostream";
        documents.openDiagnostic(header, 3, 5);
        check(root.externalReads.length === 1 && root.externalReads[0] === header,
              "fora da raiz vai pelo fs.readExternal: " + JSON.stringify(root.externalReads));
        check(root.reads.length === 0, "fora da raiz nao vai pelo fs.read: " + JSON.stringify(root.reads));
        preview.handleLoaded(header, "// 1\n// 2\nextern ostream cout;\n");
        check(documents.currentFilePath() === header, "a aba do cabecalho e' a atual: " + documents.currentFilePath());
        check(surface.text.indexOf("extern ostream cout") >= 0, "o conteudo chegou");
        check(surface.cursorPosition === 10 + 4, "o cursor na linha 3, coluna 5: " + surface.cursorPosition);

        // Um destino guardado de OUTRO arquivo nao move o cursor do que chegou.
        documents.openDiagnostic("/usr/include/a.h", 2, 1);
        preview.open("/tmp/solto.txt");
        preview.handleLoaded("/tmp/solto.txt", "x\ny\n");
        check(documents.currentFilePath() === "/tmp/solto.txt" && surface.cursorPosition === 0,
              "o destino de outro arquivo fica guardado: " + surface.cursorPosition);

        const externalBefore = root.externalReads.length;
        documents.openDiagnostic("src/a.cpp", 1, 1);
        check(root.reads.length === 1 && root.reads[0] === "/p/src/a.cpp",
              "dentro da raiz continua o fs.read: " + JSON.stringify(root.reads));
        check(root.externalReads.length === externalBefore, "dentro da raiz nao vai pelo externo");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
