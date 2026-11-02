import QtQuick

// Evidence belongs to the preview snapshot; edits and context changes retire it.
Item {
    id: root

    property var documentController: null
    property var surfaceBridge: null
    property var evidence: null
    property var files: []
    property bool obsolete: false

    visible: false

    Connections {
        target: root.surfaceBridge !== null && root.surfaceBridge.editorSurface !== undefined
                ? root.surfaceBridge.editorSurface : null
        function onTextChanged() {
            if (root.evidence !== null && root.surfaceBridge.loadingText !== true
                    && root.documentController !== null) {
                root.documentChanged(root.documentController.currentFilePath(), root.surfaceBridge.text());
            }
        }
    }

    Connections {
        target: root.documentController === null ? null : root.documentController.filesModel
        function onDataChanged() {
            if (root.evidence !== null && !root.otherBuffersClean()) {
                root.invalidate(qsTr("Salve as outras abas modificadas e consulte as ações novamente."));
            }
        }
    }

    function reset() {
        evidence = null;
        files = [];
        obsolete = false;
    }

    function invalidate(reason) {
        if (evidence === null) return;
        obsolete = true;
        evidence = { status: "notVerified", reason: reason };
    }

    function otherBuffersClean() {
        if (documentController === null) return true;
        const model = documentController.filesModel;
        for (let i = 0; i < model.count; ++i) {
            if (i !== documentController.currentTab && model.get(i).modified) return false;
        }
        return true;
    }

    function receive(previewFiles, result) {
        files = previewFiles;
        evidence = result === undefined || result === null || result.status === undefined ? null : result;
        obsolete = evidence !== null && evidence.obsolete === true;
        if (evidence !== null && !otherBuffersClean()) {
            invalidate(qsTr("Salve as outras abas modificadas e consulte as ações novamente."));
        }
        if (evidence !== null && documentController !== null && surfaceBridge !== null) {
            documentChanged(documentController.currentFilePath(), surfaceBridge.text());
        }
    }

    function documentChanged(path, content) {
        for (let i = 0; i < files.length; ++i) {
            if (files[i].path === path && files[i].beforeContent !== content) {
                invalidate(qsTr("Documento alterado; consulte as ações novamente."));
                return;
            }
        }
    }

    function canApply() {
        if (evidence === null) return true;
        if (documentController !== null && surfaceBridge !== null) {
            documentChanged(documentController.currentFilePath(), surfaceBridge.text());
        }
        if (!otherBuffersClean()) {
            invalidate(qsTr("Salve as outras abas modificadas e consulte as ações novamente."));
        }
        return !obsolete;
    }
}
