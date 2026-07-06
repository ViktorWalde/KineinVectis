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

    signal readFileRequested(string path)
    signal writeFileRequested(string path, string content)
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
        currentTab = -1;
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

    function selectTab(index) {
        if (index === currentTab) {
            return;
        }
        storeCurrentEditor();
        currentTab = index;
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
        writeFileRequested(openFilesModel.get(currentTab).path, surfaceBridge.text());
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
            modified: false
        });
        selectTab(openFilesModel.count - 1);
        if (pendingJumpPath === path) {
            jumpToPending();
        }
        currentDocumentChanged();
    }

    function handleFileSaved(path) {
        for (let i = 0; i < openFilesModel.count; i++) {
            if (openFilesModel.get(i).path === path) {
                openFilesModel.setProperty(i, "savedContent",
                                           openFilesModel.get(i).content);
                openFilesModel.setProperty(i, "modified", false);
            }
        }
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
