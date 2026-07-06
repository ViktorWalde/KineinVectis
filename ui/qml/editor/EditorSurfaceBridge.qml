import QtQuick

Item {
    id: root

    property var editorSurface: null
    property bool loadingText: false

    visible: false

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
