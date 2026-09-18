import QtQuick

// A lista de recentes como a tela inicial a mostra (Etapa 2 F7,
// 2026-09-18): os de caminho ausente ficam OCULTOS (com desfazer que os
// traz de volta), o ultimo aberto vem em destaque e Enter o abre.
Item {
    id: root

    property var workspaces: []
    property string errorText: ""
    // Desfazer do "some da lista": true mostra de novo os de caminho ausente.
    property bool showMissing: false
    // Indice em `visibleWorkspaces`; -1 quando a lista esta vazia.
    property int highlightedIndex: -1

    readonly property var visibleWorkspaces: showMissing
        ? workspaces : workspaces.filter(entry => entry.available)
    readonly property int hiddenMissingCount: showMissing
        ? 0 : workspaces.length - visibleWorkspaces.length

    signal listRequested()
    signal openRequested(string rootPath)
    signal pinRequested(string rootPath, bool pinned)
    signal removeRequested(string rootPath)
    signal clearRequested()

    visible: false

    function handleResolved(items) {
        workspaces = items !== undefined && items !== null ? items : [];
        errorText = "";
        highlightedIndex = lastOpenedIndex();
    }

    function handleRequestFailed(method, message) {
        if (method.indexOf("workspace.recent.") === 0) {
            errorText = message;
        }
    }

    // O ultimo aberto (maior lastOpenedAt) entre os visiveis — nao o
    // primeiro da lista, porque os fixados vem antes.
    function lastOpenedIndex() {
        let best = -1;
        for (let index = 0; index < visibleWorkspaces.length; index++) {
            if (best < 0 || visibleWorkspaces[index].lastOpenedAt > visibleWorkspaces[best].lastOpenedAt) {
                best = index;
            }
        }
        return best;
    }

    function moveHighlight(delta) {
        if (visibleWorkspaces.length === 0) {
            highlightedIndex = -1;
            return;
        }
        const next = highlightedIndex < 0 ? lastOpenedIndex() : highlightedIndex + delta;
        highlightedIndex = Math.max(0, Math.min(visibleWorkspaces.length - 1, next));
    }

    function openHighlighted() {
        if (highlightedIndex >= 0 && highlightedIndex < visibleWorkspaces.length) {
            openWorkspace(visibleWorkspaces[highlightedIndex].root);
        }
    }

    function restoreMissing() {
        showMissing = true;
        highlightedIndex = lastOpenedIndex();
    }

    function entryFor(rootPath) {
        for (let index = 0; index < workspaces.length; index++) {
            if (workspaces[index].root === rootPath) {
                return workspaces[index];
            }
        }
        return null;
    }

    function openWorkspace(rootPath) {
        const entry = entryFor(rootPath);
        if (entry === null || !entry.available) {
            errorText = qsTr("Esse workspace não está mais disponível.");
            return;
        }
        errorText = "";
        openRequested(rootPath);
    }

    function togglePinned(rootPath) {
        const entry = entryFor(rootPath);
        if (entry === null) {
            return;
        }
        pinRequested(rootPath, !entry.pinned);
    }

    function removeWorkspace(rootPath) {
        if (entryFor(rootPath) !== null) {
            removeRequested(rootPath);
        }
    }

    function clearAll() {
        if (workspaces.length > 0) {
            clearRequested();
        }
    }
}
