import QtQuick

// SALVAR O QUE ESTA' ABERTO (extraido do EditorDocumentController em
// 2026-09-25, fatia V5).
//
// Saiu de la' porque a catraca de arquitetura disse nao: o controller passou de
// 400 linhas ao ganhar a identidade por documento da V5. Salvar e' uma
// responsabilidade inteira — o arquivo atual, todos os modificados, e a regra
// do SNAPSHOT que impede um salvar-tudo assincrono de sobrescrever digitacao
// nova — e por isso foi ela que saiu.
//
// Recebe o `documentController` como dono, do mesmo jeito que o
// `EditorFormatController` ja' fazia: o estado (modelo, buffer, pendencias)
// continua num lugar so'.
Item {
    id: root

    property var documentController: null
    // O conflito externo tem dono proprio; o salvamento so' o avisa do que
    // aconteceu com o arquivo que ele mesmo escreveu.
    property var externalController: null

    signal writeFileRequested(string path, string content, string expectedContent)

    visible: false

    function ready() {
        return documentController !== null && documentController.surfaceReady();
    }

    function notePendingSave(path, content) {
        const saves = documentController.pendingSaves;
        saves[path] = content;
        documentController.pendingSaves = saves;
    }

    function saveCurrentFile() {
        if (!ready() || documentController.currentTab < 0) {
            return;
        }
        documentController.storeCurrentEditor();
        const model = documentController.filesModel;
        const document = model.get(documentController.currentTab);
        const text = documentController.surfaceBridge.text();
        notePendingSave(document.path, text);
        root.writeFileRequested(document.path, text, document.savedContent);
    }

    function modifiedDocuments() {
        if (documentController === null) {
            return [];
        }
        documentController.storeCurrentEditor();
        const model = documentController.filesModel;
        const result = [];
        for (let i = 0; i < model.count; i++) {
            const document = model.get(i);
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
        if (documentController === null) {
            return false;
        }
        documentController.storeCurrentEditor();
        const model = documentController.filesModel;
        for (let i = 0; i < model.count; i++) {
            const document = model.get(i);
            if (document.path !== path || document.externalDeleted === true) {
                continue;
            }
            if (document.content !== localSnapshot) {
                return false;
            }
            model.setProperty(i, "content", contentToSave);
            model.setProperty(i, "modified", contentToSave !== document.savedContent);
            if (i === documentController.currentTab && ready()) {
                const surface = documentController.surfaceBridge;
                const cursor = Math.min(surface.editorSurface.cursorPosition,
                                        contentToSave.length);
                surface.setText(contentToSave);
                surface.editorSurface.cursorPosition = cursor;
            }
            notePendingSave(path, contentToSave);
            root.writeFileRequested(path, contentToSave, document.savedContent);
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

    // --- A volta do disco ----------------------------------------------
    //
    // Escrever e' metade; a outra e' o que o core responde. As duas moram
    // juntas porque a pendencia registrada aqui e' o que a resposta consome.

    function takePendingSave(path) {
        const saves = documentController.pendingSaves;
        const noted = saves[path];
        delete saves[path];
        documentController.pendingSaves = saves;
        return noted;
    }

    function handleFileSaved(path) {
        const noted = takePendingSave(path);
        const model = documentController.filesModel;
        for (let i = 0; i < model.count; i++) {
            if (model.get(i).path !== path) {
                continue;
            }
            // O buffer LOCAL vence: o autor pode ter digitado entre o pedido
            // de escrita e a resposta, e esse texto nao pode sumir.
            const localContent = i === documentController.currentTab && ready()
                    ? documentController.surfaceBridge.text() : model.get(i).content;
            const snapshot = noted !== undefined ? noted : model.get(i).content;
            model.setProperty(i, "savedContent", snapshot);
            model.setProperty(i, "content", localContent);
            model.setProperty(i, "modified", localContent !== snapshot);
            if (externalController !== null) {
                externalController.clearAt(i);
            }
        }
    }

    function handleFileSaveFailed(path, message) {
        takePendingSave(path);
        const model = documentController.filesModel;
        for (let i = 0; i < model.count; i++) {
            if (model.get(i).path === path) {
                if (externalController !== null) {
                    externalController.markConflictAt(i, message);
                    externalController.queueRead(path, message);
                }
                return;
            }
        }
    }
}
