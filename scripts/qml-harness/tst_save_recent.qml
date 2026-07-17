import QtQuick
import "../../ui/qml/editor"
import "../../ui/qml/search"

Item {
    id: root

    property var saved: []
    property string openedRecent: ""

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
            onWriteFileRequested: function(path, content, expected) {
                const next = root.saved.slice();
                next.push({ path: path, content: content, expected: expected });
                root.saved = next;
            }
        }

        SearchController {
            id: search
            workspaceRoot: "/tmp/ws"
            recentFiles: documents.recentFiles
            onReadFileRequested: function(path) { root.openedRecent = path; }
        }
    }

    Component.onCompleted: {
        let failures = 0;
        documents.handleFileLoaded("/tmp/ws/a.rs", "a\n");
        fakeBridge.currentText = "a changed\n";
        documents.markCurrentModified(fakeBridge.currentText);
        documents.handleFileLoaded("/tmp/ws/b.rs", "b\n");
        fakeBridge.currentText = "b changed\n";
        documents.markCurrentModified(fakeBridge.currentText);

        documents.saveAllFiles();
        if (root.saved.length !== 2) failures += 1;
        if (root.saved[0].expected !== "a\n"
                || root.saved[1].expected !== "b\n") failures += 2;

        if (documents.recentFiles.length !== 2
                || documents.recentFiles[0] !== "/tmp/ws/b.rs") failures += 4;
        search.openRecentFiles();
        if (!search.everywhereVisible || !search.recentMode
                || search.everywhereModel.count !== 2) failures += 8;
        search.acceptSearchEverywhere();
        if (root.openedRecent !== "/tmp/ws/b.rs") failures += 16;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
