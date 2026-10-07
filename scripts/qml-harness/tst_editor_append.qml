import QtQuick
import KineinVectis

Item {
    id: root
    width: 640
    height: 300
    property var reads: []
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
    }
    EditorAppendController {
        id: appendController
        documentController: documents
        surfaceBridge: bridge
        function contextIsCurrent(operation) { return operation.valid === true; }
        onReadFileRequested: path => root.reads.push(path)
    }
    function check(ok, label) {
        if (!ok) console.error("FALHOU: " + label);
        return ok ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        documents.handleFileLoaded("/p/a.sql", "SELECT 'original';");
        surface.insert(surface.text.length, "\n-- não salvo 😀");
        documents.storeCurrentEditor();
        documents.markCurrentModified(surface.text);
        const dirty = surface.text;
        appendController.request("/p/a.sql", "");
        failures += check(surface.text === dirty && root.reads.length === 0, "reabrir conserva texto sujo sem reler disco");
        appendController.request("/p/a.sql", "SELECT 'modelo';");
        failures += check(surface.text === dirty + "\n\nSELECT 'modelo';\n" && surface.selectedText === "SELECT 'modelo';", "acrescenta e seleciona apenas a instrução");
        failures += check(documents.openFilesModel.get(0).modified && documents.openFilesModel.get(0).savedContent === "SELECT 'original';", "rascunho não substitui savedContent");
        surface.undo();
        failures += check(surface.text === dirty, "um undo remove só a inserção");
        appendController.request("/p/b.sql", "SELECT 2;");
        appendController.request("/p/b.sql", "SELECT 3;");
        failures += check(root.reads.length === 1 && surface.text === dirty, "fila não duplica leitura nem toca outro arquivo");
        documents.handleFileLoaded("/p/b.sql", "-- B\n");
        appendController.loaded("/p/b.sql");
        failures += check(surface.text === "-- B\n\nSELECT 2;\n\nSELECT 3;\n" && surface.selectedText === "SELECT 3;", "modelos esperam carga e conservam ordem");
        appendController.loaded("/p/b.sql");
        failures += check(surface.text.indexOf("SELECT 2;") === surface.text.lastIndexOf("SELECT 2;"), "carga repetida não reinsere");
        const context = { valid: true };
        appendController.request("/p/stale.sql", "SELECT 'outro destino';", context);
        context.valid = false;
        documents.handleFileLoaded("/p/stale.sql", "-- contexto mudou\n");
        appendController.loaded("/p/stale.sql");
        failures += check(surface.text === "-- contexto mudou\n", "contexto é validado de novo após carga");
        appendController.request("/p/failed.sql", "SELECT 'falhou';");
        appendController.cancel();
        documents.handleFileLoaded("/p/failed.sql", "-- reaberto\n");
        appendController.loaded("/p/failed.sql");
        failures += check(surface.text === "-- reaberto\n", "leitura falha não deixa inserção para uma abertura futura");
        appendController.request("/p/old.sql", "SELECT 'antigo';");
        documents.workspaceRoot = "/other";
        documents.handleFileLoaded("/p/old.sql", "-- antigo\n");
        appendController.loaded("/p/old.sql");
        failures += check(surface.text === "-- antigo\n" && appendController.pending.length === 0, "troca de workspace descarta inserção pendente");
        documents.handleExternalFileLoaded("/external.sql", "-- externo");
        appendController.request("/external.sql", "DELETE FROM t;");
        failures += check(surface.text === "-- externo", "aba somente leitura não recebe modelo");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
