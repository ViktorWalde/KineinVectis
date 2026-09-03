pragma ComponentBehavior: Bound
import QtQuick

// FORMATAÇÃO e a máquina de estados do FORMAT-ON-SAVE.
//
// # Por que isto é um dono, e não três `if` no meio do salvar
//
// Saiu do `EditorController.qml` em 2026-09-02, no mesmo corte que criou o
// `EditorLanguageController` e o `EditorHighlightController`. A razão não é
// tamanho: é que **format-on-save é assíncrono e tem estado pendente**, e
// estado pendente sem dono é onde bug mora.
//
// O gesto do usuário é um só (Ctrl+S), mas o caminho tem duas pernas:
//
// ```text
// Ctrl+S ─► formata ─► resolved ─► salva
//              └────► failed  ─► salva ASSIM MESMO
// ```
//
// A segunda perna é a que importa: se o formatter não existe, morreu ou deu
// timeout, o `format.text` volta em `requestFailed` — e o Ctrl+S do usuário
// **não pode ficar preso** esperando um servidor que não vem. Formatar é
// conveniência; salvar é dado. Por isso `pendingSaveAfterFormat` é consumido
// nos DOIS caminhos, e por isso ele precisa de um dono que os enxergue juntos.
//
// # "Salvar tudo" é uma fila, não um laço
//
// Com format-on-save ligado, `saveAllFiles` não pode formatar tudo de uma vez:
// cada `format.text` é um round-trip ao core, e a resposta chega por sinal.
// A fila (`pendingSaveAllQueue` + `pendingSaveAllItem`) processa um documento
// por vez e avança em `handleFormatResolved` — ou no `failed`, pela mesma razão
// da perna de cima. Reentrar no meio disso duplicaria escrita, e é por isso que
// `saveCurrentFile`/`saveAllFiles` recusam começar com fila em andamento.
//
// # A UI não decide o que é formatável
//
// O catálogo de formatters vem do core (`format.capabilities`, protocolo
// `0.61.0`). Até o `0.60` havia DUAS listas escritas à mão no QML — uma por
// linguagem (2 itens) e outra por extensão (9 itens) — que nem concordavam
// entre si, enquanto o core já era a autoridade em `format::formatter_for_path`.
// Duas fontes para a mesma verdade divergem por construção.
//
// Vazio até o catálogo chegar, e isso é o comportamento seguro: **não formatar
// por ainda não saber** nunca bloqueia o salvar.
Item {
    id: root

    property var surfaceBridge: null
    property var documentController: null
    property var completionController: null
    property var settingsController: null
    property var editorSurface: null

    // Extensões que o core declara saber formatar. Fonte única.
    property var formatterExtensions: []

    // Estado pendente do format-on-save. Público porque o `clear()` do
    // composition root precisa zerá-lo quando o workspace troca.
    property bool pendingSaveAfterFormat: false
    property var pendingSaveAllQueue: []
    property var pendingSaveAllItem: null

    signal formatRequested(string path, string content)
    // Pedido de esconder popups antes de reescrever o buffer: quem é dono do
    // hover é a camada de linguagem, e ela não é injetada aqui.
    signal dismissOverlaysRequested()

    visible: false

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && documentController !== null;
    }

    function currentPath() {
        return documentController === null ? "" : documentController.currentFilePath();
    }

    function applyFormatCapabilities(formatters) {
        const extensions = [];
        for (let index = 0; index < formatters.length; index++) {
            const lista = formatters[index].extensions || [];
            for (let interno = 0; interno < lista.length; interno++) {
                extensions.push(String(lista[interno]).toLowerCase());
            }
        }
        formatterExtensions = extensions;
    }

    function formatOnSaveEnabled() {
        return settingsController !== null && settingsController.formatOnSave;
    }

    // Única pergunta de formatabilidade da UI, respondida pelo catálogo do
    // core. Arquivo sem extensão (`Makefile`, `.bashrc`) nunca é formatável.
    function formattablePath(path) {
        const ponto = path.lastIndexOf(".");
        const barra = path.lastIndexOf("/");
        if (ponto <= barra + 1) {
            return false;
        }
        return formatterExtensions.indexOf(path.substring(ponto + 1).toLowerCase()) >= 0;
    }

    function busy() {
        return pendingSaveAllItem !== null || pendingSaveAllQueue.length > 0;
    }

    function saveCurrentFile() {
        if (busy()) {
            return;
        }
        if (formatOnSaveEnabled() && currentPath() !== "" && ready()
                && formattablePath(currentPath())) {
            pendingSaveAfterFormat = true;
            formatCurrentFile();
            return;
        }
        documentController.saveCurrentFile();
    }

    function saveAllFiles() {
        if (busy()) {
            return;
        }
        if (!formatOnSaveEnabled()) {
            documentController.saveAllFiles();
            return;
        }
        pendingSaveAllQueue = documentController.modifiedDocuments();
        continueSaveAll();
    }

    function hasModifiedFiles() {
        return documentController.modifiedDocuments().length > 0;
    }

    function continueSaveAll() {
        while (pendingSaveAllQueue.length > 0) {
            const item = pendingSaveAllQueue[0];
            pendingSaveAllQueue = pendingSaveAllQueue.slice(1);
            if (formattablePath(item.path)) {
                pendingSaveAllItem = item;
                formatRequested(item.path, item.content);
                return;
            }
            documentController.saveDocumentSnapshot(item.path, item.content, item.content);
        }
        pendingSaveAllItem = null;
    }

    function formatCurrentFile() {
        const path = currentPath();
        if (path === "" || !ready()) {
            return;
        }
        dismissOverlaysRequested();
        formatRequested(path, surfaceBridge.text());
    }

    function handleFormatResolved(path, text, changed) {
        if (pendingSaveAllItem !== null && pendingSaveAllItem.path === path) {
            const item = pendingSaveAllItem;
            pendingSaveAllItem = null;
            documentController.saveDocumentSnapshot(path, item.content, text);
            continueSaveAll();
            return;
        }
        // Reescrever o buffer move o cursor; preservá-lo (com clamp no fim do
        // texto novo) é a diferença entre "formatou" e "perdi meu lugar".
        if (ready() && path === currentPath() && changed && editorSurface !== null) {
            const cursor = editorSurface.cursorPosition;
            editorSurface.text = text;
            editorSurface.cursorPosition = Math.min(cursor, text.length);
        }
        if (pendingSaveAfterFormat) {
            pendingSaveAfterFormat = false;
            documentController.saveCurrentFile();
        }
    }

    // A perna do FRACASSO, e ela é obrigatória: formatter ausente ou lento não
    // pode engolir o Ctrl+S do usuário.
    function handleFormatFailed() {
        if (pendingSaveAfterFormat) {
            pendingSaveAfterFormat = false;
            documentController.saveCurrentFile();
            return;
        }
        if (pendingSaveAllItem !== null) {
            const item = pendingSaveAllItem;
            pendingSaveAllItem = null;
            documentController.saveDocumentSnapshot(item.path, item.content, item.content);
            continueSaveAll();
        }
    }

    function clear() {
        pendingSaveAfterFormat = false;
        pendingSaveAllQueue = [];
        pendingSaveAllItem = null;
    }
}
