import QtQuick
import "../../ui/qml/components"

// As regras puras da grade comum (Etapa 2 F8): largura por conteudo com
// piso e teto, a sobra distribuida, `null`, linha como objeto ou array.
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

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
