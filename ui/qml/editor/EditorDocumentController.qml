import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property var surfaceBridge: null
    property alias filesModel: openFilesModel
    property int currentTab: -1
    property var pendingReloads: ({})
    property var pendingSaves: ({})
    property alias recentFiles: recent.paths
    // Conflito externo tem dono proprio; ver EditorExternalChangeController.qml.
    property alias currentExternalConflict: external.currentConflict
    property alias currentExternalDeleted: external.currentDeleted
    property alias currentExternalMessage: external.currentMessage

    signal readFileRequested(string path)
    signal writeFileRequested(string path, string content, string expectedContent)
    signal currentDocumentChanged()

    visible: false

    ListModel {
        id: openFilesModel
    }

    // Regras puras (sem estado, sem UI); ver PathRules.qml.
    PathRules {
        id: pathRules
    }

    EditorRecentFiles {
        id: recent

        rules: pathRules
    }

    EditorJumpController {
        id: jump

        filesModel: openFilesModel
        surfaceBridge: root.surfaceBridge
        workspaceRoot: root.workspaceRoot

        onTabSelectionRequested: index => root.selectTab(index)
        onReadFileRequested: path => root.readFileRequested(path)
    }

    EditorExternalChangeController {
        id: external

        filesModel: openFilesModel
        currentTab: root.currentTab
        surfaceBridge: root.surfaceBridge
        pendingSaves: root.pendingSaves

        onReadFileRequested: path => root.readFileRequested(path)
        onDocumentChanged: root.currentDocumentChanged()
    }

    function surfaceReady() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function editorText() {
        return surfaceReady() ? surfaceBridge.text() : "";
    }

    function relativeToRoot(path) {
        return pathRules.relativeTo(workspaceRoot, path);
    }

    function clear() {
        openFilesModel.clear();
        jump.reset();
        pendingReloads = {};
        pendingSaves = {};
        recent.reset();
        currentTab = -1;
        external.reset();
        if (surfaceBridge !== null) {
            surfaceBridge.setText("");
            surfaceBridge.setPath("");
        }
    }

    function storeCurrentEditor() {
        if (surfaceReady() && currentTab >= 0 && currentTab < openFilesModel.count) {
            openFilesModel.setProperty(currentTab, "content", surfaceBridge.text());
        }
    }

    function markCurrentModified(text) {
        if (currentTab < 0 || currentTab >= openFilesModel.count) {
            return false;
        }
        const saved = openFilesModel.get(currentTab).savedContent;
        openFilesModel.setProperty(currentTab, "modified", text !== saved);
        return true;
    }

    // M-S1 (DocsPublic/seguranca/23): true se a aba atual tem alterações não salvas.
    function currentIsModified() {
        return currentTab >= 0 && currentTab < openFilesModel.count
                && openFilesModel.get(currentTab).modified === true;
    }

    // M-S1: sobrepõe o buffer de `path` com o rascunho recuperado, mantendo o
    // savedContent do disco (aba fica modificada, salvar/reverter corretos).
    function applyDraftOverlay(path, draftContent) {
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                openFilesModel.setProperty(i, "content", draftContent);
                openFilesModel.setProperty(i, "modified",
                        draftContent !== openFilesModel.get(i).savedContent);
                if (i === currentTab && surfaceReady()) {
                    surfaceBridge.setText(draftContent);
                }
                return;
            }
        }
    }

    function selectTab(index) {
        if (index === currentTab) {
            if (index >= 0 && index < openFilesModel.count) {
                recent.touch(openFilesModel.get(index).path);
            }
            return;
        }
        storeCurrentEditor();
        currentTab = index;
        if (index >= 0 && index < openFilesModel.count) {
            recent.touch(openFilesModel.get(index).path);
        }
        external.syncCurrent();
        if (surfaceBridge !== null) {
            surfaceBridge.setText(index >= 0 ? openFilesModel.get(index).content : "");
            surfaceBridge.setPath(index >= 0 ? openFilesModel.get(index).path : "");
        }
        currentDocumentChanged();
    }

    function closeTab(index) {
        openFilesModel.remove(index);
        if (openFilesModel.count === 0) {
            currentTab = -1;
            external.syncCurrent();
            if (surfaceBridge !== null) {
                surfaceBridge.setText("");
                surfaceBridge.setPath("");
            }
            currentDocumentChanged();
            return;
        }
        const next = Math.min(index, openFilesModel.count - 1);
        currentTab = -1;
        selectTab(next);
    }

    function saveCurrentFile() {
        if (!surfaceReady() || currentTab < 0) {
            return;
        }
        storeCurrentEditor();
        const document = openFilesModel.get(currentTab);
        const saves = pendingSaves;
        saves[document.path] = surfaceBridge.text();
        pendingSaves = saves;
        writeFileRequested(document.path, surfaceBridge.text(), document.savedContent);
    }

    function modifiedDocuments() {
        storeCurrentEditor();
        const result = [];
        for (let i = 0; i < openFilesModel.count; i++) {
            const document = openFilesModel.get(i);
            if (document.modified === true && document.externalDeleted !== true) {
                result.push({
                    path: document.path,
                    content: document.content,
                    savedContent: document.savedContent
                });
            }
        }
        return result;
    }

    // Salva somente se o buffer ainda for o snapshot que iniciou a operacao.
    // Isso impede um format/save-all assincrono de sobrescrever digitacao nova.
    function saveDocumentSnapshot(path, localSnapshot, contentToSave) {
        storeCurrentEditor();
        for (let i = 0; i < openFilesModel.count; i++) {
            const document = openFilesModel.get(i);
            if (document.path !== path || document.externalDeleted === true) {
                continue;
            }
            if (document.content !== localSnapshot) {
                return false;
            }
            openFilesModel.setProperty(i, "content", contentToSave);
            openFilesModel.setProperty(i, "modified",
                                       contentToSave !== document.savedContent);
            if (i === currentTab && surfaceReady()) {
                const cursor = Math.min(surfaceBridge.editorSurface.cursorPosition,
                                        contentToSave.length);
                surfaceBridge.setText(contentToSave);
                surfaceBridge.editorSurface.cursorPosition = cursor;
            }
            const saves = pendingSaves;
            saves[path] = contentToSave;
            pendingSaves = saves;
            writeFileRequested(path, contentToSave, document.savedContent);
            return true;
        }
        return false;
    }

    function saveAllFiles() {
        const modified = modifiedDocuments();
        for (let i = 0; i < modified.length; i++) {
            saveDocumentSnapshot(modified[i].path, modified[i].content,
                                 modified[i].content);
        }
    }

    function handleExternalChanges(changes) {
        storeCurrentEditor();
        external.noteChanges(changes);
    }

    function applyPathRenameToTabs(from, to) {
        for (let i = 0; i < openFilesModel.count; i++) {
            const current = openFilesModel.get(i).path;
            const updated = pathRules.renamed(current, from, to);
            if (updated === current) {
                continue;
            }
            openFilesModel.setProperty(i, "path", updated);
            openFilesModel.setProperty(i, "name", pathRules.baseName(updated));
            if (i === currentTab && surfaceBridge !== null) {
                surfaceBridge.setPath(updated);
            }
        }
        recent.applyRename(from, to);
    }

    function closeTabsUnderPath(path) {
        for (let i = openFilesModel.count - 1; i >= 0; i--) {
            if (pathRules.isUnder(openFilesModel.get(i).path, path)) {
                closeTab(i);
            }
        }
    }

    function openDiagnostic(file, line, column) {
        jump.request(file, line, column);
    }

    function jumpToPending() {
        jump.applyPending();
    }

    function currentFilePath() {
        if (currentTab < 0 || currentTab >= openFilesModel.count) {
            return "";
        }
        return openFilesModel.get(currentTab).path;
    }

    function handleFileLoaded(path, content) {
        if (pendingReloads[path] === true) {
            const reloads = pendingReloads;
            delete reloads[path];
            pendingReloads = reloads;
            for (let i = 0; i < openFilesModel.count; i++) {
                if (openFilesModel.get(i).path === path) {
                    openFilesModel.setProperty(i, "content", content);
                    openFilesModel.setProperty(i, "savedContent", content);
                    openFilesModel.setProperty(i, "modified", false);
                    if (i === currentTab && surfaceReady()) {
                        const cursor = Math.min(surfaceBridge.editorSurface.cursorPosition,
                                                content.length);
                        surfaceBridge.setText(content);
                        surfaceBridge.editorSurface.cursorPosition = cursor;
                    }
                    return;
                }
            }
            return;
        }
        if (external.isPendingRead(path)) {
            external.applyExternalRead(path, content);
            return;
        }
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                openFilesModel.setProperty(i, "content", content);
                openFilesModel.setProperty(i, "savedContent", content);
                openFilesModel.setProperty(i, "modified", false);
                selectTab(i);
                if (surfaceBridge !== null) {
                    surfaceBridge.setText(content);
                    surfaceBridge.setPath(path);
                }
                if (jump.hasPendingFor(path)) {
                    jump.applyPending();
                }
                currentDocumentChanged();
                return;
            }
        }
        openFilesModel.append({
            path: path,
            name: pathRules.baseName(path),
            content: content,
            savedContent: content,
            modified: false,
            externalContent: "",
            externalConflict: false,
            externalDeleted: false,
            externalMessage: ""
        });
        selectTab(openFilesModel.count - 1);
        if (jump.hasPendingFor(path)) {
            jump.applyPending();
        }
        currentDocumentChanged();
    }

    function handleFileSaved(path) {
        const saves = pendingSaves;
        const savedContent = saves[path];
        delete saves[path];
        pendingSaves = saves;
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                const localContent = i === currentTab && surfaceReady()
                        ? surfaceBridge.text() : openFilesModel.get(i).content;
                const snapshot = savedContent !== undefined
                        ? savedContent : openFilesModel.get(i).content;
                openFilesModel.setProperty(i, "savedContent", snapshot);
                openFilesModel.setProperty(i, "content", localContent);
                openFilesModel.setProperty(i, "modified", localContent !== snapshot);
                external.clearAt(i);
            }
        }
    }

    function handleFileSaveFailed(path, message) {
        const saves = pendingSaves;
        delete saves[path];
        pendingSaves = saves;
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                external.markConflictAt(i, message);
                external.queueRead(path, message);
                return;
            }
        }
    }

    function reloadExternalCurrent() {
        return external.reloadCurrent();
    }

    function keepLocalAfterExternalChange() {
        external.keepLocal();
    }

    function handleRenameApplied(files) {
        const reloads = {};
        for (let i = 0; i < openFilesModel.count; i++) {
            const path = openFilesModel.get(i).path;
            if (files.indexOf(path) >= 0) {
                reloads[path] = true;
            }
        }
        pendingReloads = reloads;
        for (const path in reloads) {
            readFileRequested(path);
        }
    }
}
