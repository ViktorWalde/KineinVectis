import QtQuick
import "../../ui/qml/components"

// As regras puras da grade comum (Etapa 2 F8): largura por conteudo com
// piso e teto, a sobra distribuida, `null`, linha como objeto ou array; e a
// ordenacao do passo 14 da 0.3.9 (59 §5.4.1).
Item {
    id: root

    GridRules { id: rules }

    Component.onCompleted: {
        let failures = 0;
        const columns = [{ key: "id", label: "id" }, { key: "name", label: "nome" }];
        const rows = [{ id: 1, name: "alpha-beta-gamma" }, { id: 22, name: null }];

        // celulas
        if (rules.cellText(null) !== "null" || rules.cellText(undefined) !== "null"
            || rules.cellText(7) !== "7" || !rules.isNull(null) || rules.isNull("")) failures += 1;
        if (rules.cellOf(rows[0], columns[1], 1) !== "alpha-beta-gamma"
            || rules.cellOf(["a", "b"], columns[1], 1) !== "b"
            // Uma lista vinda do C++ so' tem `length` e indices.
            || rules.cellOf({ 0: "x", 1: "y", length: 2 }, columns[1], 1) !== "y"
            || rules.cellOf(null, columns[0], 0) !== undefined) failures += 2;

        // a grade inteira sem rolar: as naturais e um vao de 1 px por coluna
        if (rules.naturalTotal(columns, rows) !== rules.naturalWidth(columns[0], rows, 0)
                + rules.naturalWidth(columns[1], rows, 1) + 2
            || rules.naturalTotal([], rows) !== 0) failures += 1024;

        // largura natural: piso para colunas curtas, teto para as longas
        if (rules.naturalWidth(columns[0], rows, 0) !== rules.minWidth) failures += 4;
        const longa = [{ key: "t", label: "t" }];
        const linhaLonga = [{ t: "x".repeat(200) }];
        if (rules.naturalWidth(longa[0], linhaLonga, 0) !== rules.maxWidth) failures += 8;
        if (rules.naturalWidth(columns[1], rows, 1) !== 16 * rules.charWidth + rules.padding) failures += 16;

        // sobra vai para a ULTIMA coluna (2026-10-03); naturais quando nao cabe (a grade rola)
        const fits = rules.columnWidths(columns, rows, 400);
        if (fits.length !== 2 || fits[0] + fits[1] > 400 || fits[0] + fits[1] < 398) failures += 32;
        if (fits[0] !== rules.naturalWidth(columns[0], rows, 0)) failures += 2048;
        // a coluna arrastada fica como a pessoa deixou, e nao encolhe para caber
        const dragged = rules.columnWidths(columns, rows, 100, { 1: 300 });
        if (dragged[1] !== 300) failures += 4096;
        // numeros (inteiro, decimal, negativo, expoente) com null no meio; texto nao
        const numbers = [{ n: "1", t: "a" }, { n: null, t: "b" }, { n: "-2.5e3", t: "3" }];
        if (!rules.isNumericColumn(numbers, { key: "n" }, 0) || rules.isNumericColumn(numbers, { key: "t" }, 1)
            || rules.isNumericColumn([{ n: null }], { key: "n" }, 0)) failures += 8192;
        const naoCabe = rules.columnWidths(columns, rows, 80);
        if (naoCabe[0] !== rules.minWidth || naoCabe[1] !== rules.shrinkFloor) failures += 64;
        if (rules.columnWidths([], [], 400).length !== 0) failures += 128;
        // a coluna larga encolhe ate' o piso para as outras caberem; abaixo do piso, rola
        const largas = [{ key: "a" }, { key: "b" }];
        const linhasLargas = [{ a: "x".repeat(100), b: "y".repeat(100) }];
        const encolhe = rules.columnWidths(largas, linhasLargas, 400);
        if (encolhe[0] + encolhe[1] > 400 || encolhe[0] < rules.shrinkFloor || encolhe[1] < rules.shrinkFloor) failures += 256;
        const rola = rules.columnWidths(largas, linhasLargas, 100);
        if (rola[0] !== rules.shrinkFloor || rola[1] !== rules.shrinkFloor) failures += 512;

        // ordenar: o ciclo do clique, e a ordem estavel com NULL por ultimo
        const cycle = rules.nextSort(-1, "", 2);
        const descending = rules.nextSort(2, "asc", 2);
        const back = rules.nextSort(2, "desc", 2);
        const other = rules.nextSort(2, "desc", 0);
        if (cycle.column !== 2 || cycle.direction !== "asc" || descending.direction !== "desc"
            || back.column !== -1 || back.direction !== "" || other.column !== 0
            || other.direction !== "asc") failures += 16384;
        // numero como numero ("10" depois de "9"); empate fica na ordem original
        const sortRows = [{ n: "10", t: "b" }, { n: null, t: "a" }, { n: "9", t: "b" },
                          { n: "-1", t: null }, { n: "9", t: "c" }];
        const byNumber = rules.sortedOrder(sortRows, { key: "n" }, 0, "asc", true);
        if (JSON.stringify(byNumber) !== "[3,2,4,0,1]") failures += 32768;
        // decrescente inverte os valores, mas o NULL continua por ultimo e o
        // empate (os dois "9") continua na ordem original
        const byNumberDescending = rules.sortedOrder(sortRows, { key: "n" }, 0, "desc", true);
        if (JSON.stringify(byNumberDescending) !== "[0,2,4,3,1]") failures += 65536;
        // texto compara como texto ("10" antes de "9"); sem direcao, a ordem original
        const asText = rules.sortedOrder([{ n: "9" }, { n: "10" }], { key: "n" }, 0, "asc", false);
        const textDescending = rules.sortedOrder(sortRows, { key: "t" }, 1, "desc", false);
        if (JSON.stringify(asText) !== "[1,0]" || JSON.stringify(textDescending) !== "[4,0,2,1,3]"
            || JSON.stringify(rules.sortedOrder(sortRows, { key: "n" }, 0, "", true)) !== "[0,1,2,3,4]"
            || rules.sortedOrder([], { key: "n" }, 0, "asc", true).length !== 0) failures += 131072;
        // dois NULL no decrescente: continuam na ordem original entre si
        const twoNulls = rules.sortedOrder([{ n: null }, { n: "1" }, { n: null }], { key: "n" }, 0, "desc", true);
        if (JSON.stringify(twoNulls) !== "[1,0,2]") failures += 524288;
        // linha como lista (a que vem do C++), pela posicao da coluna
        if (JSON.stringify(rules.sortedOrder([["b"], ["a"]], { key: "x" }, 0, "asc", false)) !== "[1,0]")
            failures += 262144;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
