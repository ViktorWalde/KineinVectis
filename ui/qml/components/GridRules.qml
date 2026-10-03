import QtQuick
import "KvLists.js" as KvLists

// As regras PURAS da grade comum (Etapa 2 F8, 2026-09-18): largura de cada
// coluna a partir do que ha' para mostrar, e o texto de uma celula. Sem
// tela, para o harness medir.
QtObject {
    id: rules

    readonly property int minWidth: 56
    readonly property int maxWidth: 320
    // Largura de um caractere: a grade a MEDE na fonte que desenha
    // (FontMetrics, 2026-10-03); 7 e' so' o valor do harness sem tela.
    property real charWidth: 7
    readonly property int padding: 10

    // Uma celula: `null`/`undefined` viram "null" (a grade os poe em italico).
    function cellText(value) {
        return value === null || value === undefined ? "null" : String(value);
    }

    function isNull(value) {
        return value === null || value === undefined;
    }

    // A celula de uma linha numa coluna: a linha pode ser um objeto (chave
    // da coluna) ou uma lista (posicao da coluna). A lista que vem do C++
    // nao passa em Array.isArray, e trata-la como objeto deixou o Resultado
    // do Banco todo em "null" (2026-10-03, achado na tela real): KvLists.
    function cellOf(row, column, index) {
        if (row === null || row === undefined) {
            return undefined;
        }
        return KvLists.isList(row) ? row[index] : row[column.key];
    }

    // Largura natural de uma coluna: o maior texto (rotulo incluido) em
    // caracteres, com piso e teto — depois `columnWidths` distribui a sobra.
    function naturalWidth(column, rows, index) {
        let chars = String(column.label !== undefined ? column.label : column.key).length;
        for (let r = 0; r < rows.length; r++) {
            chars = Math.max(chars, cellText(cellOf(rows[r], column, index)).length);
        }
        return Math.max(minWidth, Math.min(maxWidth, Math.ceil(chars * charWidth) + padding));
    }

    // A largura em que a grade inteira cabe sem rolar (colunas naturais e os
    // vaos de 1 px): quem a hospeda pode se alargar ate' ela.
    function naturalTotal(columns, rows) {
        let total = 0;
        for (let i = 0; i < columns.length; i++) total += naturalWidth(columns[i], rows, i) + 1;
        return total;
    }

    // Ate' onde uma coluna larga ENCOLHE para as outras caberem.
    readonly property int shrinkFloor: 120

    // Uma coluna de NUMEROS (toda celula nao nula e' numero): a grade a
    // alinha a direita, como as grades de banco fazem — os digitos se leem
    // por casa decimal.
    function isNumericColumn(rows, column, index) {
        let seen = 0;
        for (let r = 0; r < rows.length; r++) {
            const value = cellOf(rows[r], column, index);
            if (isNull(value)) continue;
            if (!/^-?\d+(\.\d+)?([eE][-+]?\d+)?$/.test(String(value).trim())) return false;
            seen++;
        }
        return seen > 0;
    }

    // As larguras finais (revistas em 2026-10-03, pedido do autor: "melhorar
    // o dimensionamento da tabela"):
    //   - cada coluna na largura NATURAL (o maior texto, medido);
    //   - a que a pessoa arrastou (`overrides[indice]`) fica como ela deixou;
    //   - nao cabe: as mais largas (nao arrastadas) encolhem ate'
    //     `shrinkFloor`, a mais larga primeiro; ainda nao cabe: a grade ROLA;
    //   - sobra espaco: vai para a ULTIMA coluna. Antes a sobra ia por igual a
    //     todas, e um `id` ganhava 240 px vazios enquanto o `email` cortava.
    function columnWidths(columns, rows, available, overrides) {
        const fixed = overrides !== undefined && overrides !== null ? overrides : {};
        const widths = columns.map((column, index) => fixed[index] !== undefined
                                   ? fixed[index] : naturalWidth(column, rows, index));
        let total = widths.reduce((sum, width) => sum + width, 0);
        if (columns.length === 0) {
            return widths;
        }
        for (let step = 0; total > available && step < 64; step++) {
            let widest = -1;
            for (let i = 0; i < widths.length; i++) {
                if (fixed[i] === undefined && widths[i] > shrinkFloor
                        && (widest < 0 || widths[i] > widths[widest])) widest = i;
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
        widths[widths.length - 1] += Math.floor(available - total);
        return widths;
    }
}
