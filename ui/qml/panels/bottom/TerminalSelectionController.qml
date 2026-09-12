import QtQuick

// Estado local de seleção da grade VT. Não conhece IPC nem clipboard.
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

    visible: false

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
        if (!hasSelection) return "";
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
        const cell = cellAt(x, y);
        anchorRow = cell.row;
        anchorCol = cell.col;
        headRow = cell.row;
        headCol = cell.col;
        selecting = true;
        hasSelection = false;
    }

    function update(x, y) {
        if (!selecting) return;
        const cell = cellAt(x, y);
        headRow = cell.row;
        headCol = cell.col;
        hasSelection = headRow !== anchorRow || headCol !== anchorCol;
    }

    function finish() {
        selecting = false;
    }

    function clear() {
        selecting = false;
        hasSelection = false;
    }
}
