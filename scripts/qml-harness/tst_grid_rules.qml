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
            || rules.cellOf(null, columns[0], 0) !== undefined) failures += 2;

        // largura natural: piso para colunas curtas, teto para as longas
        if (rules.naturalWidth(columns[0], rows, 0) !== rules.minWidth) failures += 4;
        const longa = [{ key: "t", label: "t" }];
        const linhaLonga = [{ t: "x".repeat(200) }];
        if (rules.naturalWidth(longa[0], linhaLonga, 0) !== rules.maxWidth) failures += 8;
        if (rules.naturalWidth(columns[1], rows, 1) !== 16 * rules.charWidth + rules.padding) failures += 16;

        // sobra distribuida por igual quando cabe; naturais quando nao cabe (a grade rola)
        const cabe = rules.columnWidths(columns, rows, 400);
        if (cabe.length !== 2 || cabe[0] + cabe[1] > 400 || cabe[0] + cabe[1] < 398) failures += 32;
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
