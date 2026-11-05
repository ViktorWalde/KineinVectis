import QtQuick
import KineinVectis

Item {
    id: root
    width: 640
    height: 300
    property int failures: 0
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
        workspaceRoot: "/project"
        surfaceBridge: bridge
    }
    function check(ok, reason) {
        if (!ok) { failures++; console.error("GUARD: " + reason); }
    }
    Component.onCompleted: {
        const path = "/project/main.c";
        const disk = "int main(void) { return 0; }\n";
        documents.handleFileLoaded(path, disk);
        const identity = documents.currentDocId;
        surface.insert(surface.text.length, "/* não salvo ação */");
        documents.markCurrentModified(surface.text);
        const draft = surface.text;
        documents.handleFileLoaded(path, disk);
        check(surface.text === draft && documents.currentIsModified(), "resposta ordinária não descarta buffer ativo");
        documents.handleFileLoaded("/project/other.c", "int other(void) { return 1; }\n");
        documents.handleFileLoaded(path, disk);
        check(surface.text === draft && documents.currentIsModified(), "retornar pela árvore conserva buffer inativo");
        check(documents.currentDocId === identity && documents.openFilesModel.count === 2, "retorno conserva identidade sem duplicar aba");
        const row = documents.openFilesModel.get(documents.currentTab);
        check(row.savedContent === disk && row.content === draft, "baseline salvo permanece separado do texto editado");
        documents.handleFileLoaded(path, disk + "/* resposta de disco posterior */");
        check(surface.text === draft && documents.currentIsModified(), "nova resposta não vira recarga implícita");
        // Uma recarga pedida explicitamente conserva seu contrato de descarte.
        documents.pendingReloads = { "/project/main.c": true };
        documents.handleFileLoaded(path, disk);
        check(surface.text === disk && !documents.currentIsModified(), "recarga explícita continua aplicando disco");
        console.log("reopen buffer: " + failures + " falhas");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
