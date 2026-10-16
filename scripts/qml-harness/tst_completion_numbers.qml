import QtQuick
// 16.7, F6 (roadmaps/59 §7.1.7): o completar automatico nao abre depois de
// numero (`1` abria `Int16Array`), nem no `.` de `3.14`; palavra abre; o
// EditorCompletionController REAL.
import "../../ui/qml/editor"

Item {
    id: root
    width: 100; height: 100
    property int requests: 0
    property int stage: 0
    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    QtObject {
        id: fakeSurface
        property string text: ""
        property int cursorPosition: 0
        property bool editorActiveFocus: true
        function remove(a, b) {}
        function insert(p, t) {}
    }
    QtObject {
        id: fakeBridge
        property var editorSurface: fakeSurface
        function ready() { return true; }
        function text() { return fakeSurface.text; }
        function focusEditor() {}
    }
    QtObject {
        id: fakeDocs
        property int currentTab: 0
        function currentFilePath() { return "/tmp/x.rs"; }
    }
    QtObject {
        id: fakeText
        function wordStartAt(pos) { return pos; }
        function cursorLineColumn() { return { line: 1, column: 1 }; }
    }
    Item {
        visible: false
        EditorCompletionController {
            id: completion
            surfaceBridge: fakeBridge
            documentController: fakeDocs
            textController: fakeText
            onCompletionRequested: root.requests += 1
        }
    }

    // Cada caso: o texto ate' o cursor, e se o completar deve abrir.
    readonly property var cases: [
        ["naoExiste: 1", false],
        ["let pi = 3.", false],
        ["let pi = 3.14", false],
        ["mask = 0x1F", false],
        ["let x = St", true],
        ["obj.", true],
        ["vec2.", true],
        ["std::", true],
        ["valor2", true]
    ]

    Timer {
        interval: 200
        running: true
        repeat: true
        onTriggered: {
            if (root.stage > 0) {
                const previous = root.cases[root.stage - 1];
                root.check((root.requests > 0) === previous[1],
                           "`" + previous[0] + "` " + (previous[1] ? "abre" : "não abre") + ": " + root.requests);
            }
            if (root.stage >= root.cases.length) {
                Qt.exit(root.failures === 0 ? 0 : 1);
                return;
            }
            // Cada caso começa sem pedido pendente (o pendente segura o próximo).
            completion.clear();
            root.requests = 0;
            fakeSurface.text = root.cases[root.stage][0];
            fakeSurface.cursorPosition = fakeSurface.text.length;
            completion.handleTextEdited();
            root.stage++;
        }
    }
}
