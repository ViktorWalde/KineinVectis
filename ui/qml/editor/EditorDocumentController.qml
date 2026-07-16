import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property var surfaceBridge: null
    property alias filesModel: openFilesModel
    property int currentTab: -1
    property string pendingJumpPath: ""
    property int pendingJumpLine: 0
    property int pendingJumpColumn: 0
    property var pendingReloads: ({})
    property var pendingExternalReads: ({})
    property var pendingSaves: ({})
    property var recentFiles: []
    property bool currentExternalConflict: false
    property bool currentExternalDeleted: false
    property string currentExternalMessage: ""

    signal readFileRequested(string path)
    signal writeFileRequested(string path, string content, string expectedContent)
    signal currentDocumentChanged()

    visible: false

    ListModel {
        id: openFilesModel
    }

    function surfaceReady() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function editorText() {
        return surfaceReady() ? surfaceBridge.text() : "";
    }

    function baseName(path) {
        return path.substring(path.lastIndexOf("/") + 1);
    }

    function relativeToRoot(path) {
        if (workspaceRoot !== "" && path.indexOf(workspaceRoot + "/") === 0) {
            return path.substring(workspaceRoot.length + 1);
        }
        return path;
    }

    function clear() {
        openFilesModel.clear();
        pendingJumpPath = "";
        pendingJumpLine = 0;
        pendingJumpColumn = 0;
        pendingReloads = {};
        pendingExternalReads = {};
        pendingSaves = {};
        recentFiles = [];
        currentTab = -1;
        syncCurrentExternalState();
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

    // M-S1 (docs/seguranca/23): true se a aba atual tem alterações não salvas.
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
                touchRecent(openFilesModel.get(index).path);
            }
            return;
        }
        storeCurrentEditor();
        currentTab = index;
        if (index >= 0 && index < openFilesModel.count) {
            touchRecent(openFilesModel.get(index).path);
        }
        syncCurrentExternalState();
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
            syncCurrentExternalState();
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

    function touchRecent(path) {
        if (path === "") {
            return;
        }
        const updated = [path];
        for (let i = 0; i < recentFiles.length && updated.length < 50; i++) {
            if (recentFiles[i] !== path) {
                updated.push(recentFiles[i]);
            }
        }
        recentFiles = updated;
    }

    function syncCurrentExternalState() {
        if (currentTab < 0 || currentTab >= openFilesModel.count) {
            currentExternalConflict = false;
            currentExternalDeleted = false;
            currentExternalMessage = "";
            return;
        }
        const document = openFilesModel.get(currentTab);
        currentExternalConflict = document.externalConflict === true;
        currentExternalDeleted = document.externalDeleted === true;
        currentExternalMessage = document.externalMessage || "";
    }

    function clearExternalState(index) {
        openFilesModel.setProperty(index, "externalContent", "");
        openFilesModel.setProperty(index, "externalConflict", false);
        openFilesModel.setProperty(index, "externalDeleted", false);
        openFilesModel.setProperty(index, "externalMessage", "");
        if (index === currentTab) {
            syncCurrentExternalState();
        }
    }

    function queueExternalRead(path, message) {
        const reads = pendingExternalReads;
        reads[path] = message || "";
        pendingExternalReads = reads;
        readFileRequested(path);
    }

    function handleExternalChanges(changes) {
        storeCurrentEditor();
        for (let changeIndex = 0; changeIndex < changes.length; changeIndex++) {
            const change = changes[changeIndex];
            for (let i = 0; i < openFilesModel.count; i++) {
                if (openFilesModel.get(i).path !== change.path) {
                    continue;
                }
                if (change.kind === "deleted") {
                    openFilesModel.setProperty(i, "externalConflict", true);
                    openFilesModel.setProperty(i, "externalDeleted", true);
                    openFilesModel.setProperty(i, "externalMessage",
                            qsTr("O arquivo foi removido fora da IDE; o buffer local foi preservado."));
                    openFilesModel.setProperty(i, "modified", true);
                    if (i === currentTab) {
                        syncCurrentExternalState();
                    }
                } else {
                    queueExternalRead(change.path, "");
                }
                break;
            }
        }
    }

    function applyPathRenameToTabs(from, to) {
        for (let i = 0; i < openFilesModel.count; i++) {
            const current = openFilesModel.get(i).path;
            let updated = "";
            if (current === from) {
                updated = to;
            } else if (current.indexOf(from + "/") === 0) {
                updated = to + current.substring(from.length);
            } else {
                continue;
            }
            openFilesModel.setProperty(i, "path", updated);
            openFilesModel.setProperty(i, "name", baseName(updated));
            if (i === currentTab && surfaceBridge !== null) {
                surfaceBridge.setPath(updated);
            }
        }
        const recent = [];
        for (let i = 0; i < recentFiles.length; i++) {
            const current = recentFiles[i];
            if (current === from) {
                recent.push(to);
            } else if (current.indexOf(from + "/") === 0) {
                recent.push(to + current.substring(from.length));
            } else {
                recent.push(current);
            }
        }
        recentFiles = recent;
    }

    function closeTabsUnderPath(path) {
        for (let i = openFilesModel.count - 1; i >= 0; i--) {
            const current = openFilesModel.get(i).path;
            if (current === path || current.indexOf(path + "/") === 0) {
                closeTab(i);
            }
        }
    }

    function openDiagnostic(file, line, column) {
        if (file === "") {
            return;
        }
        const path = file.startsWith("/") ? file : workspaceRoot + "/" + file;
        pendingJumpPath = path;
        pendingJumpLine = line;
        pendingJumpColumn = column;
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                selectTab(i);
                jumpToPending();
                return;
            }
        }
        readFileRequested(path);
    }

    function jumpToPending() {
        if (!surfaceReady() || pendingJumpPath === "") {
            return;
        }
        const text = surfaceBridge.text();
        let offset = 0;
        for (let currentLine = 1; currentLine < pendingJumpLine; currentLine++) {
            const next = text.indexOf("\n", offset);
            if (next < 0) {
                break;
            }
            offset = next + 1;
        }
        if (pendingJumpColumn > 0) {
            offset += pendingJumpColumn - 1;
        }
        surfaceBridge.editorSurface.cursorPosition = Math.min(offset, text.length);
        surfaceBridge.focusEditor();
        pendingJumpPath = "";
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
        if (pendingExternalReads[path] !== undefined) {
            const reads = pendingExternalReads;
            const failureMessage = reads[path];
            delete reads[path];
            pendingExternalReads = reads;
            for (let i = 0; i < openFilesModel.count; i++) {
                if (openFilesModel.get(i).path !== path) {
                    continue;
                }
                const document = openFilesModel.get(i);
                const pendingSave = pendingSaves[path];
                if ((pendingSave !== undefined && content === pendingSave)
                        || content === document.savedContent) {
                    clearExternalState(i);
                    return;
                }
                const localContent = i === currentTab && surfaceReady()
                        ? surfaceBridge.text() : document.content;
                if (localContent === document.savedContent) {
                    openFilesModel.setProperty(i, "content", content);
                    openFilesModel.setProperty(i, "savedContent", content);
                    openFilesModel.setProperty(i, "modified", false);
                    clearExternalState(i);
                    if (i === currentTab && surfaceReady()) {
                        const cursor = Math.min(
                                surfaceBridge.editorSurface.cursorPosition,
                                content.length);
                        surfaceBridge.setText(content);
                        surfaceBridge.editorSurface.cursorPosition = cursor;
                        currentDocumentChanged();
                    }
                    return;
                }
                openFilesModel.setProperty(i, "externalContent", content);
                openFilesModel.setProperty(i, "externalConflict", true);
                openFilesModel.setProperty(i, "externalDeleted", false);
                openFilesModel.setProperty(i, "externalMessage",
                        failureMessage !== "" ? failureMessage
                        : qsTr("O arquivo mudou no disco enquanto havia alterações locais."));
                if (i === currentTab) {
                    syncCurrentExternalState();
                }
                return;
            }
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
                if (pendingJumpPath === path) {
                    jumpToPending();
                }
                currentDocumentChanged();
                return;
            }
        }
        openFilesModel.append({
            path: path,
            name: baseName(path),
            content: content,
            savedContent: content,
            modified: false,
            externalContent: "",
            externalConflict: false,
            externalDeleted: false,
            externalMessage: ""
        });
        selectTab(openFilesModel.count - 1);
        if (pendingJumpPath === path) {
            jumpToPending();
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
                clearExternalState(i);
            }
        }
    }

    function handleFileSaveFailed(path, message) {
        const saves = pendingSaves;
        delete saves[path];
        pendingSaves = saves;
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                openFilesModel.setProperty(i, "externalConflict", true);
                openFilesModel.setProperty(i, "externalMessage", message);
                if (i === currentTab) {
                    syncCurrentExternalState();
                }
                queueExternalRead(path, message);
                return;
            }
        }
    }

    function reloadExternalCurrent() {
        if (currentTab < 0 || currentExternalDeleted) {
            return "";
        }
        const document = openFilesModel.get(currentTab);
        const content = document.externalContent;
        const path = document.path;
        openFilesModel.setProperty(currentTab, "content", content);
        openFilesModel.setProperty(currentTab, "savedContent", content);
        openFilesModel.setProperty(currentTab, "modified", false);
        clearExternalState(currentTab);
        if (surfaceReady()) {
            const cursor = Math.min(surfaceBridge.editorSurface.cursorPosition,
                                    content.length);
            surfaceBridge.setText(content);
            surfaceBridge.editorSurface.cursorPosition = cursor;
        }
        currentDocumentChanged();
        return path;
    }

    function keepLocalAfterExternalChange() {
        if (currentTab < 0) {
            return;
        }
        const document = openFilesModel.get(currentTab);
        if (!currentExternalDeleted) {
            openFilesModel.setProperty(currentTab, "savedContent",
                                       document.externalContent);
        }
        const localContent = surfaceReady() ? surfaceBridge.text() : document.content;
        openFilesModel.setProperty(currentTab, "content", localContent);
        openFilesModel.setProperty(currentTab, "modified", true);
        clearExternalState(currentTab);
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
