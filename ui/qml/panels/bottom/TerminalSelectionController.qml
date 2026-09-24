import QtQuick

// Selecao visivel local e identidade da selecao completa mantida no core.
// O texto completo so atravessa este controller no gesto de copiar.
//
// R1.4 (DocsPublic/roadmaps/26): a conversão pixel→célula NÃO mora aqui. Ela é uma
// só, no TerminalMetrics, e texto, cursor, seleção, mouse e resize consomem a
// mesma instância. Quando cada consumidor fazia a própria conta, o cursor e o
// texto divergiam até 0,95px (§4.6).
Item {
    id: root

    // Regras puras de trecho; ver TerminalSpanRules.qml.
    TerminalSpanRules {
        id: spanRules
    }

    property var lines: []
    property TerminalMetrics metrics: null
    property bool selecting: false
    property bool hasSelection: false
    property int anchorRow: 0
    property int anchorCol: 0
    property int headRow: 0
    property int headCol: 0
    readonly property bool hasSelectableContent: lastContentRow() >= 0
    property string selectedSnapshot: ""
    property var runtimeController: null
    property string sessionId: ""
    property bool available: true
    property string allToken: ""
    property bool allConfirmed: false
    property int gestureSequence: 0
    property bool copyPending: false
    signal textReady(string text)

    onSessionIdChanged: clear()
    onAvailableChanged: if (!available) clear()

    visible: false
    // Coordenadas de viewport nao sao identidade do buffer. Invalidar evita
    // copiar texto diferente quando output/scroll/reflow muda a selecao.
    onLinesChanged: {
        if (allToken === "" && hasSelection && selectedText() !== selectedSnapshot) clear();
    }

    Connections {
        target: root.runtimeController
        function onTerminalSelectionCopied(id, selectionId, text, valid) {
            root.receiveCopy(id, selectionId, text, valid);
        }
        function onTerminalSelectionFailed() { root.clear(); }
    }

    function selectAll() {
        clear();
        if (!available || sessionId === "" || !runtimeController) return;
        gestureSequence += 1;
        allToken = String(gestureSequence);
        hasSelection = true;
        runtimeController.terminalSelectAllRequested(sessionId, allToken);
        selectionTimeout.restart();
    }

    function handleCoreSelection(token) {
        if (allToken === "") return;
        if (token === allToken) {
            allConfirmed = true;
            if (!copyPending) selectionTimeout.stop();
        } else if (allConfirmed) clear();
    }

    function copySelection() {
        if (!hasSelection || copyPending) return;
        if (allToken !== "") {
            copyPending = true;
            runtimeController.terminalCopySelectionRequested(sessionId, allToken);
            selectionTimeout.restart();
            return;
        }
        const text = selectedText();
        if (text !== "") textReady(text);
    }

    function receiveCopy(id, token, text, valid) {
        if (!available || id !== sessionId || token !== allToken || !copyPending) return;
        copyPending = false;
        selectionTimeout.stop();
        if (valid && text !== "") textReady(text);
        if (!valid) clear();
    }

    Timer {
        id: selectionTimeout
        interval: 3000
        onTriggered: root.clear()
    }

    function cellAt(x, y) {
        if (!metrics || metrics.cellWidth <= 0 || metrics.cellHeight <= 0) {
            return { "row": 0, "col": 0 };
        }
        return {
            "row": metrics.rowAt(y),
            "col": metrics.columnAt(x)
        };
    }

    function lineText(row) {
        if (row < 0 || row >= lines.length) return "";
        const spans = lines[row];
        let text = "";
        for (let index = 0; index < spans.length; index++) {
            text += spans[index].text;
        }
        return text;
    }

    function lineCells(row) {
        if (row < 0 || row >= lines.length) return 0;
        const spans = lines[row];
        let cells = 0;
        for (let index = 0; index < spans.length; index++) {
            cells += spanRules.cells(spans[index]);
        }
        return cells;
    }

    function lastContentRow() {
        for (let row = lines.length - 1; row >= 0; row--) {
            if (lineCells(row) > 0) return row;
        }
        return -1;
    }

    function lineTextBetween(row, startCol, endCol) {
        if (row < 0 || row >= lines.length) return "";
        const spans = lines[row];
        const first = Math.max(0, startCol);
        const last = Math.max(first, endCol);
        let text = "";
        let spanStart = 0;
        for (let index = 0; index < spans.length; index++) {
            const span = spans[index];
            const cells = spanRules.cells(span);
            const spanEnd = spanStart + cells;
            const overlapStart = Math.max(first, spanStart);
            const overlapEnd = Math.min(last, spanEnd);
            if (overlapStart < overlapEnd) {
                const glyphs = Array.from(String(
                    span.text !== undefined ? span.text : ""));
                if (glyphs.length === cells) {
                    text += glyphs.slice(overlapStart - spanStart,
                                         overlapEnd - spanStart).join("");
                } else {
                    // O core isola células largas/combinações: tocar qualquer
                    // coluna do glifo seleciona o glifo inteiro.
                    text += glyphs.join("");
                }
            }
            spanStart = spanEnd;
            if (spanStart >= last) break;
        }
        return text;
    }

    function range() {
        if (allToken !== "") {
            const last = Math.max(0, lastContentRow());
            return { "r1": 0, "c1": 0, "r2": last, "c2": lineCells(last) };
        }
        let r1 = anchorRow;
        let c1 = anchorCol;
        let r2 = headRow;
        let c2 = headCol;
        if (r2 < r1 || (r2 === r1 && c2 < c1)) {
            r1 = headRow;
            c1 = headCol;
            r2 = anchorRow;
            c2 = anchorCol;
        }
        return { "r1": r1, "c1": c1, "r2": r2, "c2": c2 };
    }

    function selectedText() {
        if (!hasSelection || allToken !== "") return "";
        const selected = range();
        if (selected.r1 === selected.r2) {
            return lineTextBetween(selected.r1, selected.c1, selected.c2);
        }
        let text = lineTextBetween(selected.r1, selected.c1,
                                   lineCells(selected.r1));
        for (let row = selected.r1 + 1; row < selected.r2; row++) {
            text += "\n" + lineText(row);
        }
        return text + "\n" + lineTextBetween(selected.r2, 0, selected.c2);
    }

    function begin(x, y) {
        clear();
        const cell = cellAt(x, y);
        anchorRow = cell.row;
        anchorCol = cell.col;
        headRow = cell.row;
        headCol = cell.col;
        selecting = true;
        hasSelection = false;
        selectedSnapshot = "";
    }

    function update(x, y) {
        if (!selecting) return;
        const cell = cellAt(x, y);
        headRow = cell.row;
        headCol = cell.col;
        hasSelection = headRow !== anchorRow || headCol !== anchorCol;
        selectedSnapshot = selectedText();
    }

    function finish() {
        selecting = false;
        selectedSnapshot = selectedText();
    }

    function selectVisible() {
        clear();
        const lastRow = lastContentRow();
        if (lastRow < 0) {
            clear();
            return;
        }
        anchorRow = 0;
        anchorCol = 0;
        headRow = lastRow;
        headCol = lineCells(headRow);
        selecting = false;
        hasSelection = true;
        selectedSnapshot = selectedText();
    }

    function clear() {
        selectionTimeout.stop();
        allToken = "";
        allConfirmed = false;
        copyPending = false;
        selecting = false;
        hasSelection = false;
        selectedSnapshot = "";
    }
}
