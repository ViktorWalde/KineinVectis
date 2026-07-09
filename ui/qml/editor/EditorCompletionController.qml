import QtQuick

Item {
    id: root

    property var surfaceBridge: null
    property var documentController: null
    property var textController: null
    property alias completionModel: completionItemsModel
    property int index: 0
    property int prefixStart: -1
    property var allItems: []

    signal completionRequested(string path, string content, int line, int column)

    visible: false

    ListModel {
        id: completionItemsModel
    }

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && documentController !== null && textController !== null;
    }

    function clear() {
        visible = false;
        index = 0;
        prefixStart = -1;
        allItems = [];
        completionItemsModel.clear();
    }

    function dismiss() {
        visible = false;
        prefixStart = -1;
    }

    function move(delta) {
        index = Math.max(0, Math.min(index + delta, completionItemsModel.count - 1));
    }

    function requestCompletion() {
        if (!ready()) {
            return;
        }
        const path = documentController.currentFilePath();
        if (path === "") {
            return;
        }
        prefixStart = textController.wordStartAt(surfaceBridge.editorSurface.cursorPosition);
        const position = textController.cursorLineColumn();
        completionRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function refilter() {
        if (!ready() || prefixStart < 0
                || prefixStart > surfaceBridge.editorSurface.cursorPosition) {
            visible = false;
            return;
        }
        const prefix = surfaceBridge.text().substring(
                    prefixStart, surfaceBridge.editorSurface.cursorPosition).toLowerCase();
        completionItemsModel.clear();
        for (let i = 0; i < allItems.length; i++) {
            const item = allItems[i];
            const insertText = item.insertText !== undefined
                    ? item.insertText : item.label;
            const label = item.label !== undefined ? item.label : insertText;
            if (prefix === ""
                    || insertText.toLowerCase().indexOf(prefix) === 0
                    || label.toLowerCase().indexOf(prefix) === 0) {
                completionItemsModel.append({
                    label: label,
                    insertText: insertText,
                    detail: item.detail !== undefined ? item.detail : "",
                    kind: item.kind !== undefined ? item.kind : ""
                });
            }
        }
        index = 0;
        visible = completionItemsModel.count > 0;
    }

    function accept() {
        if (!ready() || !visible || index < 0 || index >= completionItemsModel.count) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const insertText = completionItemsModel.get(index).insertText;
        const start = prefixStart >= 0 ? prefixStart : surface.cursorPosition;
        const end = surface.cursorPosition;
        dismiss();
        surface.remove(start, end);
        surface.insert(start, insertText);
        surface.cursorPosition = start + insertText.length;
        surfaceBridge.focusEditor();
    }

    function handleTextEdited() {
        if (visible) {
            refilter();
        } else {
            completionDebounce.restart();
        }
    }

    function handleResolved(items) {
        allItems = items;
        refilter();
    }

    Timer {
        id: completionDebounce

        interval: 250
        repeat: false
        onTriggered: {
            if (!root.ready() || root.documentController.currentTab < 0
                    || !root.surfaceBridge.editorSurface.editorActiveFocus
                    || root.visible) {
                return;
            }
            const position = root.surfaceBridge.editorSurface.cursorPosition;
            if (position <= 0) {
                return;
            }
            const previous = root.surfaceBridge.editorSurface.text.charAt(
                        position - 1);
            if (root.textController.isWordChar(previous) || previous === "."
                    || previous === ":") {
                root.requestCompletion();
            }
        }
    }
}
