import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property var surfaceBridge: null
    property alias filesModel: openDocuments.model
    // A ABA E' UM DOCUMENTO, NAO UMA POSICAO (V5, 2026-09-25). O que a IDE
    // guarda e' QUAL documento esta' na tela; o indice e' derivado dele, e
    // existe so' para o `ListView` saber qual linha pintar.
    property int currentDocId: 0
    // O indice e' DERIVADO do documento, e recalculado por SINAL — nunca por
    // binding, sob pena de laco. O porque esta' no EditorOpenDocuments.qml.
    property int currentTab: -1
    property bool currentReadOnly: false

    function refreshCurrentTab() {
        const index = openDocuments.indexOf(root.currentDocId);
        if (root.currentTab !== index) {
            root.currentTab = index;
        }
    }

    onCurrentDocIdChanged: root.refreshCurrentTab()

    Connections {
        target: openDocuments

        // Fechar uma aba de fundo nao troca o documento, mas muda a POSICAO
        // dele — e e' isso que o `ListView` precisa saber.
        function onIndexByDocIdChanged() {
            root.refreshCurrentTab();
        }
    }
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

    EditorOpenDocuments {
        id: openDocuments
    }

    EditorSaveController {
        id: saving

        documentController: root
        externalController: external
        rules: pathRules

        onWriteFileRequested: function(path, content, expectedContent) {
            root.writeFileRequested(path, content, expectedContent);
        }
    }

    // Atalho de leitura: o modelo continua sendo lido por indice em dezenas de
    // lugares deste arquivo, e isso e' detalhe de armazenamento, nao de dominio.
    readonly property var openFilesModel: openDocuments.model

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

        filesModel: root.openFilesModel
        surfaceBridge: root.surfaceBridge
        workspaceRoot: root.workspaceRoot

        onTabSelectionRequested: index => root.selectDocument(openDocuments.docIdAt(index))
        onReadFileRequested: path => root.readFileRequested(path)
    }

    EditorExternalChangeController {
        id: external

        filesModel: root.openFilesModel
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
        openDocuments.clear();
        jump.reset();
        pendingReloads = {};
        pendingSaves = {};
        recent.reset();
        currentDocId = 0;
        currentReadOnly = false;
        external.reset();
        if (surfaceBridge !== null) {
            surfaceBridge.setText("");
            surfaceBridge.setPath("");
        }
    }

    function storeCurrentEditor() {
        if (surfaceReady() && currentTab >= 0 && currentTab < openFilesModel.count
                && !currentReadOnly) {
            openFilesModel.setProperty(currentTab, "content", surfaceBridge.text());
        }
    }

    function markCurrentModified(text) {
        if (currentReadOnly || currentTab < 0 || currentTab >= openFilesModel.count) {
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
                if (openFilesModel.get(i).readOnly === true) return;
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

    // O DOMINIO FALA EM DOCUMENTO. Quem tem um indice na mao (o delegate do
    // `ListView`) traduz na fronteira, com `openDocuments.docIdAt`.
    function selectDocument(docId) {
        const index = openDocuments.indexOf(docId);
        if (index < 0) {
            // ID SEM DOCUMENTO NAO MEXE EM NADA. O harness pegou isto: a
            // primeira versao zerava o documento atual — ou seja, um id velho,
            // de uma aba ja' fechada, ESVAZIAVA a tela de quem estava
            // editando. Esvaziar tem dono proprio, o `clearCurrentDocument`.
            return false;
        }
        const path = openFilesModel.get(index).path;
        if (docId === currentDocId) {
            if (!currentReadOnly) recent.touch(path);
            return true;
        }
        storeCurrentEditor();
        // Troca nao e' edicao: o realce re-emite o texto ANTERIOR (40 §7.144).
        if (surfaceBridge !== null) surfaceBridge.loadingText = true;
        currentReadOnly = openFilesModel.get(index).readOnly === true;
        currentDocId = docId;
        if (!currentReadOnly) recent.touch(path);
        external.syncCurrent();
        if (surfaceBridge !== null) {
            surfaceBridge.setText(openFilesModel.get(index).content);
            surfaceBridge.loadingText = false;
            surfaceBridge.setPath(path);
        }
        currentDocumentChanged();
        return true;
    }

    // Nenhum documento na tela. O unico caminho para isso e' fechar o ultimo
    // ou limpar o workspace — nunca um id que ninguem reconhece.
    function clearCurrentDocument() {
        currentDocId = 0;
        currentReadOnly = false;
        external.syncCurrent();
        if (surfaceBridge !== null) {
            surfaceBridge.setText("");
            surfaceBridge.setPath("");
        }
        currentDocumentChanged();
    }

    function pathOfDocument(docId) {
        return openDocuments.pathOfDocument(docId);
    }

    function isReadOnlyDocument(docId) {
        const index = openDocuments.indexOf(docId);
        return index >= 0 && openFilesModel.get(index).readOnly === true;
    }

    function closeDocument(docId) {
        if (openDocuments.indexOf(docId) < 0) {
            return;
        }
        // O buffer do documento ATUAL vai para o modelo ANTES de qualquer
        // remocao. Sem isto, fechar uma aba de fundo passava por `selectTab`
        // com `currentTab` ja' invalidado e o texto da tela era descartado.
        storeCurrentEditor();
        const closingCurrent = docId === currentDocId;
        const successor = closingCurrent ? openDocuments.successorOf(docId) : currentDocId;
        openDocuments.remove(docId);
        if (!closingCurrent) {
            // Fechar OUTRA aba nao troca o documento na tela: so' a posicao
            // dele mudou, e posicao e' detalhe do `ListView`.
            return;
        }
        currentDocId = 0;
        if (successor === 0) {
            clearCurrentDocument();
            return;
        }
        selectDocument(successor);
    }

    // Salvar tem dono proprio; ver EditorSaveController.qml.
    function saveCurrentFile() {
        saving.saveCurrentFile();
    }

    function modifiedDocuments() {
        return saving.modifiedDocuments();
    }

    function saveDocumentSnapshot(path, localSnapshot, contentToSave) {
        return saving.saveDocumentSnapshot(path, localSnapshot, contentToSave);
    }

    function saveAllFiles() {
        saving.saveAllFiles();
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
        // Uma edição pode ocorrer enquanto a resposta de fs.delete está em trânsito.
        if (hasUnsavedUnderPath(path)) return;
        // Os IDS sao colhidos ANTES de fechar qualquer um: cada remocao
        // desloca os indices seguintes, e o laco por indice fechava a aba
        // errada quando duas abas vizinhas estavam sob o mesmo caminho.
        const ids = openDocuments.docIdsUnder(path, pathRules);
        for (let i = 0; i < ids.length; i++) {
            closeDocument(ids[i]);
        }
    }

    function hasUnsavedUnderPath(path) {
        return saving.hasUnsavedUnderPath(path);
    }

    function openDiagnostic(file, line, column) {
        jump.request(file, line, column);
    }

    function jumpToPending() {
        jump.applyPending();
    }

    function currentFilePath() {
        return openDocuments.pathOfDocument(currentDocId);
    }

    function handleFileLoaded(path, content) {
        if (pendingReloads[path] === true) {
            const reloads = pendingReloads;
            delete reloads[path];
            pendingReloads = reloads;
            const reloaded = openDocuments.applyDiskContent(path, content, false);
            // O CURSOR e' preservado: recarregar nao e' abrir, e o autor
            // continua onde estava.
            if (reloaded === currentDocId && reloaded !== 0 && surfaceReady()) {
                const cursor = Math.min(surfaceBridge.editorSurface.cursorPosition,
                                        content.length);
                surfaceBridge.setText(content);
                surfaceBridge.editorSurface.cursorPosition = cursor;
            }
            return;
        }
        if (external.isPendingRead(path)) {
            external.applyExternalRead(path, content);
            return;
        }
        const alreadyOpen = openDocuments.applyDiskContent(path, content, false);
        if (alreadyOpen !== 0) {
            selectDocument(alreadyOpen);
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
        const newDocId = openDocuments.addLoadedFile(
            path, pathRules.baseName(path), content, false);
        selectDocument(newDocId);
        if (jump.hasPendingFor(path)) {
            jump.applyPending();
        }
        currentDocumentChanged();
    }

    function handleExternalFileLoaded(path, content) {
        const docId = openDocuments.applyDiskContent(path, content, true);
        if (docId !== 0) {
            selectDocument(docId);
            if (surfaceBridge !== null) {
                surfaceBridge.setText(content);
                currentDocumentChanged();
            }
            return;
        }
        if (openDocuments.docIdForPath(path) !== 0) return;
        selectDocument(openDocuments.addLoadedFile(
            path, pathRules.baseName(path), content, true));
    }

    function handleFileSaved(path) {
        saving.handleFileSaved(path);
    }

    function handleFileSaveFailed(path, message) {
        saving.handleFileSaveFailed(path, message);
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
