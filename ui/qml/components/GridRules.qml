import QtQuick

// As regras PURAS da grade comum (Etapa 2 F8, 2026-09-18): largura de cada
// coluna a partir do que ha' para mostrar, e o texto de uma celula. Sem
// tela, para o harness medir.
QtObject {
    id: rules

    readonly property int minWidth: 56
    readonly property int maxWidth: 320
    readonly property int charWidth: 7
    readonly property int padding: 10

    // Uma celula: `null`/`undefined` viram "null" (a grade os poe em italico).
    function cellText(value) {
        return value === null || value === undefined ? "null" : String(value);
    }

    function isNull(value) {
        return value === null || value === undefined;
    }

    // A celula de uma linha numa coluna: a linha pode ser um objeto (chave
    // da coluna) ou um array (posicao da coluna).
    function cellOf(row, column, index) {
        if (Array.isArray(row)) {
            return row[index];
        }
        return row === null || row === undefined ? undefined : row[column.key];
    }

    // Largura natural de uma coluna: o maior texto (rotulo incluido) em
    // caracteres, com piso e teto — depois `columnWidths` distribui a sobra.
    function naturalWidth(column, rows, index) {
        let chars = String(column.label !== undefined ? column.label : column.key).length;
        for (let r = 0; r < rows.length; r++) {
            chars = Math.max(chars, cellText(cellOf(rows[r], column, index)).length);
        }
        return Math.max(minWidth, Math.min(maxWidth, chars * charWidth + padding));
    }

    // Ate' onde uma coluna larga ENCOLHE para as outras caberem.
    readonly property int shrinkFloor: 120

    // As larguras finais: a natural de cada uma; se a soma couber em
    // `available`, a sobra vai por igual a todas (a grade preenche a
    // moldura); se nao couber, as mais largas encolhem ate' `shrinkFloor`
    // (a mais larga primeiro) e, so' se ainda nao couber, a grade ROLA.
    function columnWidths(columns, rows, available) {
        const widths = columns.map((column, index) => naturalWidth(column, rows, index));
        let total = widths.reduce((sum, width) => sum + width, 0);
        if (columns.length === 0) {
            return widths;
        }
        for (let step = 0; total > available && step < 64; step++) {
            let widest = -1;
            for (let i = 0; i < widths.length; i++) {
                if (widths[i] > shrinkFloor && (widest < 0 || widths[i] > widths[widest])) widest = i;
            }
            if (widest < 0) {
                return widths;
            }
            const cut = Math.min(total - available, widths[widest] - shrinkFloor);
            widths[widest] -= cut;
            total -= cut;
        }
        if (total >= available) {
            return widths;
        }
        const extra = Math.floor((available - total) / columns.length);
        return widths.map(width => width + extra);
    }
}
