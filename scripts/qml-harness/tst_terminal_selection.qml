import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 100
    height: 100

    // R1: a seleção não converte pixel→célula por conta própria; ela consome a
    // mesma instância de métricas que o texto e o cursor. Por isso o harness
    // deixou de fixar uma geometria falsa (10x20) e passou a derivar as
    // coordenadas da grade real — assim ele testa a grade que a IDE desenha.
    TerminalMetrics {
        id: metrics
        fontFamily: "monospace"
        fontPixelSize: 13
        devicePixelRatio: 1.0
    }

    // Um ponto dentro da célula (col,row): canto + um quarto, longe da borda.
    function pt(col, row) {
        return {
            "x": metrics.xForColumn(col) + metrics.cellWidth / 4,
            "y": metrics.yForRow(row) + metrics.cellHeight / 4
        };
    }

    TerminalSelectionController {
        id: selection
        metrics: metrics
        lines: [
            [{ "text": "primeira" }],
            [{ "text": "segunda" }],
            [{ "text": "terceira" }],
            [
                { "text": "A", "cells": 1 },
                { "text": "界", "cells": 2 },
                { "text": "B", "cells": 1 }
            ]
        ]
    }

    Component.onCompleted: {
        let failures = 0;

        // (col 2, linha 0) até (col 3, linha 2).
        let a = root.pt(2, 0), b = root.pt(3, 2);
        selection.begin(a.x, a.y);
        selection.update(b.x, b.y);
        selection.finish();
        if (!selection.hasSelection) failures += 1;
        if (selection.selectedText() !== "imeira\nsegunda\nter") failures += 2;

        // Seleção para trás na mesma linha: (col 4 → col 1) da linha 1.
        a = root.pt(4, 1); b = root.pt(1, 1);
        selection.begin(a.x, a.y);
        selection.update(b.x, b.y);
        selection.finish();
        if (selection.selectedText() !== "egu") failures += 4;

        selection.clear();
        if (selection.hasSelection || selection.selecting) failures += 8;

        // Linha 3 é "A界B": o 界 ocupa as colunas 1 e 2, o B a coluna 3.
        a = root.pt(3, 3); b = root.pt(4, 3);
        selection.begin(a.x, a.y);
        selection.update(b.x, b.y);
        selection.finish();
        if (selection.selectedText() !== "B") failures += 16;

        // Tocar qualquer coluna do glifo largo seleciona o glifo inteiro.
        a = root.pt(1, 3); b = root.pt(2, 3);
        selection.begin(a.x, a.y);
        selection.update(b.x, b.y);
        selection.finish();
        if (selection.selectedText() !== "界") failures += 32;

        Qt.exit(failures);
    }
}
