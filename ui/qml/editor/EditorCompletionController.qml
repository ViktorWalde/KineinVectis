import QtQuick

Item {
    id: root

    property var surfaceBridge: null
    property var documentController: null
    property var textController: null
    // Índice sintático local produzido pelo Tree-sitter já integrado. Ele
    // fornece uma primeira lista enquanto clangd/rust-analyzer inicializa ou
    // indexa o projeto; o LSP continua sendo a autoridade semântica final.
    property var localItems: []
    property alias completionModel: completionItemsModel
    property int index: 0
    property int prefixStart: -1
    property var allItems: []
    // O servidor marcou a lista como incompleta (ex.: std::c no clangd,
    // que devolve centenas): ao digitar mais, REPEDIR ao servidor em vez
    // de filtrar o cache — senão itens fora dos primeiros N somem.
    property bool lastIncomplete: false
    property bool fallbackVisible: false
    property bool serverRequestPending: false
    property bool refreshQueued: false
    // D1 (docs/roadmaps/24): estado do popup em property PRÓPRIA — NÃO no `visible`
    // do Item. `Item.visible` LÊ a visibilidade EFETIVA (explicitVisible &&
    // pai efetivamente visível); como este controller vive dentro do
    // EditorController (Item invisível, pois é controller e não UI), gravar
    // `visible = true` guardava explicitVisible mas a LEITURA devolvia
    // sempre false e nem emitia visibleChanged — o popup nunca abria.
    property bool popupVisible: false

    signal completionRequested(string path, string content, int line, int column)

    visible: false

    ListModel {
        id: completionItemsModel
    }

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && documentController !== null && textController !== null;
    }

    function clear() {
        popupVisible = false;
        index = 0;
        prefixStart = -1;
        allItems = [];
        lastIncomplete = false;
        fallbackVisible = false;
        serverRequestPending = false;
        refreshQueued = false;
        completionItemsModel.clear();
    }

    function dismiss() {
        popupVisible = false;
        prefixStart = -1;
    }

    function move(delta) {
        index = Math.max(0, Math.min(index + delta, completionItemsModel.count - 1));
    }

    function requestCompletion() {
        if (!ready()) {
            return;
        }
        const path = documentController.currentFilePath();
        if (path === "") {
            return;
        }
        prefixStart = textController.wordStartAt(surfaceBridge.editorSurface.cursorPosition);
        if (!popupVisible || fallbackVisible) {
            showLocalFallback();
        }
        if (serverRequestPending) {
            refreshQueued = true;
            return;
        }
        const position = textController.cursorLineColumn();
        serverRequestPending = true;
        completionRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function showLocalFallback() {
        const seen = {};
        const items = [];
        // Não varre um índice sintático arbitrariamente grande na thread da
        // UI: a prévia precisa continuar instantânea em arquivos extensos.
        const scanLimit = Math.min(localItems.length, 2048);
        for (let i = 0; i < scanLimit && items.length < 256; i++) {
            const local = localItems[i];
            const name = local.name !== undefined && local.name !== null
                    ? String(local.name) : "";
            if (name === "" || seen[name] === true) {
                continue;
            }
            // Referências completam lacunas quando a query da gramática não
            // classificou a declaração como definição.
            if (local.kind !== "definition" && local.kind !== "reference") {
                continue;
            }
            seen[name] = true;
            items.push({
                label: name,
                insertText: name,
                detail: qsTr("símbolo local — análise sintática"),
                kind: local.kind === "definition" ? "local" : "reference"
            });
        }
        if (items.length === 0) {
            return;
        }
        allItems = items;
        lastIncomplete = false;
        fallbackVisible = true;
        refilter();
    }

    // D1 (docs/roadmaps/24): match FUZZY por subsequência (estilo VS Code). O filtro
    // só-prefixo antigo escondia matches legítimos do servidor (ex.:
    // `into_iter` ao digitar `iter`), deixando o popup vazio. Preserva a
    // ORDEM do servidor (ele já ranqueia), então os melhores ficam no topo.
    function fuzzyMatch(text, needle) {
        if (needle === "") {
            return true;
        }
        const haystack = text.toLowerCase();
        const query = needle.toLowerCase();
        let j = 0;
        for (let i = 0; i < haystack.length && j < query.length; i++) {
            if (haystack.charAt(i) === query.charAt(j)) {
                j++;
            }
        }
        return j === query.length;
    }

    function refilter() {
        if (!ready() || prefixStart < 0
                || prefixStart > surfaceBridge.editorSurface.cursorPosition) {
            popupVisible = false;
            return;
        }
        const prefix = surfaceBridge.text().substring(
                    prefixStart, surfaceBridge.editorSurface.cursorPosition);
        completionItemsModel.clear();
        for (let i = 0; i < allItems.length; i++) {
            const item = allItems[i];
            const insertText = item.insertText !== undefined
                    ? item.insertText : item.label;
            const label = item.label !== undefined ? item.label : insertText;
            if (fuzzyMatch(insertText, prefix) || fuzzyMatch(label, prefix)) {
                completionItemsModel.append({
                    label: label,
                    insertText: insertText,
                    detail: item.detail !== undefined ? item.detail : "",
                    kind: item.kind !== undefined ? item.kind : ""
                });
            }
        }
        index = 0;
        popupVisible = completionItemsModel.count > 0;
    }

    function accept() {
        if (!ready() || !popupVisible || index < 0
                || index >= completionItemsModel.count) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const insertText = completionItemsModel.get(index).insertText;
        const start = prefixStart >= 0 ? prefixStart : surface.cursorPosition;
        const end = surface.cursorPosition;
        dismiss();
        surface.remove(start, end);
        surface.insert(start, insertText);
        surface.cursorPosition = start + insertText.length;
        surfaceBridge.focusEditor();
    }

    function handleTextEdited() {
        if (popupVisible && (lastIncomplete || fallbackVisible)) {
            // Lista incompleta: refiltra o cache já (resposta instantânea)
            // e REPEDE ao servidor com o prefixo maior (traz os itens que
            // não couberam no primeiro lote, ex.: cout ao digitar "cou").
            refilter();
            completionDebounce.restart();
        } else if (popupVisible) {
            refilter();
        } else {
            completionDebounce.restart();
        }
    }

    function handleResolved(items, isIncomplete) {
        serverRequestPending = false;
        if (items.length > 0 || !fallbackVisible) {
            allItems = items;
            lastIncomplete = isIncomplete === true;
            fallbackVisible = false;
            refilter();
        }
        if (refreshQueued) {
            refreshQueued = false;
            completionRefresh.restart();
        }
    }

    function handleFailed() {
        serverRequestPending = false;
        refreshQueued = false;
        if (!fallbackVisible) {
            dismiss();
        }
    }

    Timer {
        id: completionDebounce

        interval: 120
        repeat: false
        onTriggered: {
            if (!root.ready() || root.documentController.currentTab < 0
                    || !root.surfaceBridge.editorSurface.editorActiveFocus) {
                return;
            }
            // Popup aberto com lista incompleta: repede ao servidor com o
            // prefixo atual (handleTextEdited só reinicia o timer visível
            // quando lastIncomplete).
            if (root.popupVisible) {
                root.requestCompletion();
                return;
            }
            // Auto-open: só depois de caractere de palavra, "." ou ":".
            const position = root.surfaceBridge.editorSurface.cursorPosition;
            if (position <= 0) {
                return;
            }
            const previous = root.surfaceBridge.editorSurface.text.charAt(
                        position - 1);
            const willOpen = root.textController.isWordChar(previous)
                    || previous === "." || previous === ":";
            if (willOpen) {
                root.requestCompletion();
            }
        }
    }

    Timer {
        id: completionRefresh

        interval: 0
        repeat: false
        onTriggered: root.requestCompletion()
    }
}
