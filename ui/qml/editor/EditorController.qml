import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property var editorSurface: null
    property alias filesModel: documents.filesModel
    property alias completionModel: completionController.completionModel
    property alias usagesModel: usagesItemsModel
    property alias currentTab: documents.currentTab
    property alias loadingEditorText: surfaceBridge.loadingText
    property alias completionVisible: completionController.visible
    property alias completionIndex: completionController.index
    property bool hoverVisible: false
    property string hoverText: ""
    property bool usagesVisible: false
    property bool renameDialogVisible: false
    property string renameError: ""

    signal readFileRequested(string path)
    signal writeFileRequested(string path, string content)
    signal fileChangedNotificationRequested(string path, string content)
    signal semanticTokensRequested(string path, string content)
    signal definitionRequested(string path, string content, int line, int column)
    signal hoverRequested(string path, string content, int line, int column)
    signal completionRequested(string path, string content, int line, int column)
    signal referencesRequested(string path, string content, int line, int column)
    signal renameRequested(string path, string content, int line, int column,
                           string newName)
    signal renameDialogOpenRequested(string currentName)

    visible: false

    ListModel {
        id: usagesItemsModel
    }

    EditorSurfaceBridge {
        id: surfaceBridge

        editorSurface: root.editorSurface
    }

    EditorDocumentController {
        id: documents

        workspaceRoot: root.workspaceRoot
        surfaceBridge: surfaceBridge
        onReadFileRequested: function(path) {
            root.readFileRequested(path);
        }
        onWriteFileRequested: function(path, content) {
            root.writeFileRequested(path, content);
        }
        onCurrentDocumentChanged: root.refreshSemanticTokens()
    }

    EditorTextController {
        id: textController

        surfaceBridge: surfaceBridge
    }

    EditorCompletionController {
        id: completionController

        surfaceBridge: surfaceBridge
        documentController: documents
        textController: textController
        onCompletionRequested: function(path, content, line, column) {
            root.completionRequested(path, content, line, column);
        }
    }

    function editorReady() {
        return surfaceBridge.ready();
    }

    function editorText() {
        return surfaceBridge.text();
    }

    function focusEditor() {
        surfaceBridge.focusEditor();
    }

    function clear() {
        documents.clear();
        completionController.clear();
        hoverText = "";
        hoverVisible = false;
        usagesVisible = false;
        usagesItemsModel.clear();
        renameDialogVisible = false;
        renameError = "";
    }

    function storeCurrentEditor() {
        documents.storeCurrentEditor();
    }

    function selectTab(index) {
        completionController.dismiss();
        documents.selectTab(index);
    }

    function closeTab(index) {
        documents.closeTab(index);
    }

    function saveCurrentFile() {
        documents.saveCurrentFile();
    }

    function applyPathRenameToTabs(from, to) {
        documents.applyPathRenameToTabs(from, to);
    }

    function closeTabsUnderPath(path) {
        documents.closeTabsUnderPath(path);
    }

    function openDiagnostic(file, line, column) {
        documents.openDiagnostic(file, line, column);
    }

    function currentFilePath() {
        return documents.currentFilePath();
    }

    function requestDefinition() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        hoverVisible = false;
        const position = textController.cursorLineColumn();
        definitionRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function requestHover() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        const position = textController.cursorLineColumn();
        hoverRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function requestCompletion() {
        hoverVisible = false;
        completionController.requestCompletion();
    }

    function acceptCompletion() {
        completionController.accept();
    }

    function moveCompletion(delta) {
        completionController.move(delta);
    }

    function refreshSemanticTokens() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        semanticTokensRequested(path, surfaceBridge.text());
    }

    function requestUsages() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        hoverVisible = false;
        const position = textController.cursorLineColumn();
        referencesRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function openRenameDialog() {
        if (currentFilePath() === "") {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        renameError = "";
        renameDialogVisible = true;
        renameDialogOpenRequested(textController.currentWord());
    }

    function confirmRename(name) {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        if (name === "") {
            renameError = qsTr("Informe um novo nome.");
            return;
        }
        for (let i = 0; i < filesModel.count; i++) {
            if (i !== currentTab && filesModel.get(i).modified) {
                renameError =
                        qsTr("Salve as outras abas modificadas antes de renomear.");
                return;
            }
        }
        const position = textController.cursorLineColumn();
        renameDialogVisible = false;
        focusEditor();
        renameRequested(path, surfaceBridge.text(), position.line, position.column, name);
    }

    function indentEditorSelection() {
        textController.indentSelection();
    }

    function unindentEditorSelection() {
        textController.unindentSelection();
    }

    function insertEditorNewline() {
        textController.insertNewline();
    }

    function handleTextEdited(text) {
        if (!surfaceBridge.loadingText && documents.markCurrentModified(text)) {
            hoverVisible = false;
            changeDebounce.restart();
            completionController.handleTextEdited();
        }
    }

    function handleFileLoaded(path, content) {
        documents.handleFileLoaded(path, content);
    }

    function handleFileSaved(path) {
        documents.handleFileSaved(path);
    }

    function handleHoverResolved(content) {
        const text = content.trim();
        if (text === "") {
            hoverVisible = false;
            hoverText = "";
            return;
        }
        hoverText = text;
        hoverVisible = true;
        hoverHideTimer.restart();
    }

    function handleSemanticTokensResolved(tokens) {
        if (editorReady()) {
            editorSurface.setSemanticTokens(tokens);
        }
    }

    function handleCompletionResolved(items) {
        completionController.handleResolved(items);
    }

    function handleReferencesResolved(references) {
        usagesItemsModel.clear();
        for (let i = 0; i < references.length; i++) {
            const usage = references[i];
            const line = usage.line !== undefined ? Number(usage.line) : 1;
            usagesItemsModel.append({
                path: usage.path,
                line: line,
                column: usage.column !== undefined ? Number(usage.column) : 1,
                display: documents.relativeToRoot(usage.path) + ":" + line
            });
        }
        usagesVisible = usagesItemsModel.count > 0;
    }

    function handleRenameApplied(files) {
        documents.handleRenameApplied(files);
    }

    function handleRequestFailed(method, message) {
        if (method === "lsp.hover") {
            hoverVisible = false;
        }
        if (method === "lsp.completion") {
            completionController.dismiss();
        }
        if (method === "lsp.rename") {
            renameError = message;
            renameDialogVisible = true;
        }
    }

    Timer {
        id: changeDebounce

        interval: 600
        repeat: false
        onTriggered: {
            const path = root.currentFilePath();
            if (path !== "" && root.editorReady()) {
                root.fileChangedNotificationRequested(path, surfaceBridge.text());
                root.refreshSemanticTokens();
            }
        }
    }

    Timer {
        id: hoverHideTimer

        interval: 9000
        repeat: false
        onTriggered: root.hoverVisible = false
    }
}
