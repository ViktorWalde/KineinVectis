import QtQuick
// Carrega o EditorCompletionController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

Item {
    id: root
    width: 100; height: 100

    // Fakes minimos com a mesma superficie que o controller usa.
    QtObject {
        id: fakeSurface
        property string text: "let x = St"
        property int cursorPosition: 10
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
        function wordStartAt(pos) { return 8; }              // inicio de "St"
        function cursorLineColumn() { return { line: 1, column: 11 }; }
        function isWordChar(c) { return /[A-Za-z0-9_]/.test(c); }
    }

    // O controller REAL, dentro de um pai INVISIVEL — exatamente como
    // ele vive dentro do EditorController na aplicacao.
    Item {
        id: invisibleParent
        visible: false

        EditorCompletionController {
            id: completion
            surfaceBridge: fakeBridge
            documentController: fakeDocs
            textController: fakeText
        }
    }

    Component.onCompleted: {
        let falhas = 0;

        // 1) Servidor responde (como a sonda ja provava). Prefixo digitado = "St".
        completion.prefixStart = 8;
        completion.handleResolved([
            { label: "String", insertText: "String", detail: "struct", kind: "struct" },
            { label: "Struct", insertText: "Struct", detail: "kw", kind: "kw" },
            { label: "into_iter", insertText: "into_iter", detail: "fn", kind: "fn" }
        ], false);

        // O popup TEM que estar visivel e com os matches fuzzy de "St".
        if (completion.popupVisible !== true) falhas += 1;      // o bug D1
        if (completion.completionModel.count !== 2) falhas += 2; // String + Struct

        // 2) dismiss() fecha.
        completion.dismiss();
        if (completion.popupVisible !== false) falhas += 4;

        // 3) Sem itens -> nao abre.
        completion.prefixStart = 8;
        completion.handleResolved([], false);
        if (completion.popupVisible !== false) falhas += 8;

        Qt.exit(falhas);
    }
}
