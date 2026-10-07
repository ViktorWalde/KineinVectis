import QtQuick

// Append a prepared instruction using native editing, after opening its file.
// Existing tabs keep their unsaved buffer. One insert remains undoable.
Item {
    id: root
    visible: false
    property var documentController: null
    property var surfaceBridge: null
    readonly property string workspaceRoot: root.documentController ? root.documentController.workspaceRoot : ""
    property var pending: []
    onWorkspaceRootChanged: root.cancel()
    signal readFileRequested(string path)

    function cancel() { root.pending = []; }

    function contextIsCurrent(operation) { return operation === undefined || operation === null; }

    function indexFor(path) {
        const files = root.documentController.openFilesModel;
        for (let index = 0; index < files.count; index++) {
            if (files.get(index).path === path) return index;
        }
        return -1;
    }

    function request(path, text, operation) {
        const index = root.indexFor(path);
        if (index >= 0) { root.apply(path, text, operation); return; }
        const waiting = root.pending.some(item => item.path === path);
        root.pending = root.pending.concat([{ path: path, text: text, operation: operation }]);
        if (!waiting) root.readFileRequested(path);
    }

    function loaded(path) {
        const ready = root.pending.filter(item => item.path === path);
        root.pending = root.pending.filter(item => item.path !== path);
        for (const item of ready) root.apply(item.path, item.text, item.operation);
    }

    function apply(path, text, operation) {
        if (operation && !root.contextIsCurrent(operation)) return;
        const index = root.indexFor(path);
        if (index < 0 || root.documentController.openFilesModel.get(index).readOnly === true) return;
        root.documentController.selectDocument(root.documentController.openFilesModel.get(index).docId);
        if (!root.surfaceBridge.ready()) return;
        const surface = root.surfaceBridge.editorSurface;
        if (text !== "") {
            const end = surface.text.length;
            const prefix = end === 0 ? "" : (surface.text.endsWith("\n") ? "\n" : "\n\n");
            surface.insert(end, prefix + text + "\n");
            root.documentController.storeCurrentEditor();
            root.documentController.markCurrentModified(surface.text);
            surface.select(end + prefix.length, end + prefix.length + text.length);
        }
        root.surfaceBridge.focusEditor();
    }
}
