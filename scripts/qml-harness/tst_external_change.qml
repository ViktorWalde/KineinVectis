import QtQuick
// Carrega o controller real: conflito externo, auto-reload e save seguro.
import "../../ui/qml/editor"

Item {
    id: root
    width: 100
    height: 100

    property string readPath: ""
    property string savedPath: ""
    property string savedContent: ""
    property string expectedContent: ""

    QtObject {
        id: fakeEditor
        property int cursorPosition: 0
    }

    QtObject {
        id: fakeBridge
        property var editorSurface: fakeEditor
        property string currentText: ""
        property string currentPath: ""
        function ready() { return true; }
        function text() { return currentText; }
        function setText(text) { currentText = text; }
        function setPath(path) { currentPath = path; }
        function focusEditor() {}
    }

    Item {
        visible: false

        EditorDocumentController {
            id: documents
            workspaceRoot: "/tmp/ws"
            surfaceBridge: fakeBridge
            onReadFileRequested: function(path) { root.readPath = path; }
            onWriteFileRequested: function(path, content, expected) {
                root.savedPath = path;
                root.savedContent = content;
                root.expectedContent = expected;
            }
        }
    }

    Component.onCompleted: {
        let failures = 0;
        const path = "/tmp/ws/main.rs";

        documents.handleFileLoaded(path, "disk v1\n");
        fakeBridge.currentText = "buffer local\n";
        documents.markCurrentModified(fakeBridge.currentText);
        documents.handleExternalChanges([{ path: path, kind: "modified" }]);
        if (root.readPath !== path) failures += 1;
        documents.handleFileLoaded(path, "disk v2\n");
        if (!documents.currentExternalConflict) failures += 2;
        if (fakeBridge.currentText !== "buffer local\n") failures += 4;

        // Escolha explícita de manter o local atualiza a base do compare-save.
        documents.keepLocalAfterExternalChange();
        documents.saveCurrentFile();
        if (root.savedContent !== "buffer local\n"
                || root.expectedContent !== "disk v2\n") failures += 8;
        documents.handleFileSaved(path);

        // Sem edição local, uma alteração externa recarrega automaticamente.
        documents.handleExternalChanges([{ path: path, kind: "modified" }]);
        documents.handleFileLoaded(path, "disk v3\n");
        if (fakeBridge.currentText !== "disk v3\n"
                || documents.currentExternalConflict) failures += 16;

        // Remoção jamais descarta o buffer em memória.
        documents.handleExternalChanges([{ path: path, kind: "deleted" }]);
        if (!documents.currentExternalConflict || !documents.currentExternalDeleted) {
            failures += 32;
        }
        if (fakeBridge.currentText !== "disk v3\n") failures += 64;

        Qt.exit(failures);
    }
}
