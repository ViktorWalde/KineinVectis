import QtQuick

Item {
    id: root

    property var editorSurface: null
    property bool loadingText: false
    // "Ln:Col" do cursor para a status bar (fechamento da Etapa 2,
    // 2026-09-18): recalculado com 80 ms de folga depois do cursor parar —
    // contar quebras ate' o cursor e' O(n) e nao vale a cada tecla.
    property string cursorSummary: ""
    // O ultimo texto que a superficie TEVE. Ela emite `textChanged` tambem
    // quando so' o realce muda (no markdown, cada snapshot do Tree-sitter):
    // sem esta guarda, realce -> "edicao" -> reparse -> realce virava um laco
    // de ~10 pedidos/s ao core com o editor parado (IPC medido em 2026-10-02).
    property string lastText: ""

    visible: false

    // Ao fim de cada carga o texto carregado vira o "ultimo": a primeira
    // mudanca so' de realce depois de abrir tambem nao conta como edicao.
    onLoadingTextChanged: if (!loadingText) lastText = text()

    // Texto igual nao e' edicao.
    function acceptEdit(newText) {
        if (newText === lastText) return false;
        lastText = newText;
        return true;
    }

    Timer {
        id: cursorTimer

        interval: 80
        repeat: false
        onTriggered: root.cursorSummary = root.computeCursorSummary()
    }

    Connections {
        target: root.editorSurface

        function onCursorPositionChanged() { cursorTimer.restart(); }
        function onTextChanged() { if (!root.loadingText) cursorTimer.restart(); }
    }

    onEditorSurfaceChanged: cursorTimer.restart()

    function computeCursorSummary() {
        if (!ready()) {
            return "";
        }
        const content = editorSurface.text;
        const cursor = editorSurface.cursorPosition;
        let line = 1;
        let lineStart = 0;
        let offset = 0;
        while (offset < cursor) {
            const next = content.indexOf("\n", offset);
            if (next < 0 || next >= cursor) {
                break;
            }
            line++;
            lineStart = next + 1;
            offset = next + 1;
        }
        return line + ":" + (cursor - lineStart + 1);
    }

    function ready() {
        return editorSurface !== null && editorSurface !== undefined;
    }

    function text() {
        return ready() ? editorSurface.text : "";
    }

    function setText(text) {
        if (!ready()) {
            return;
        }
        loadingText = true;
        editorSurface.text = text;
        loadingText = false;
    }

    function setPath(path) {
        if (ready()) {
            editorSurface.setFilePath(path);
        }
    }

    function focusEditor() {
        if (ready()) {
            editorSurface.focusEditor();
        }
    }
}
