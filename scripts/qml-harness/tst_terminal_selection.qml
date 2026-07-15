import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 100
    height: 100

    TerminalSelectionController {
        id: selection
        charWidth: 10
        lineHeight: 20
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

        selection.begin(20, 1);
        selection.update(30, 41);
        selection.finish();
        if (!selection.hasSelection) failures += 1;
        if (selection.selectedText() !== "imeira\nsegunda\nter") failures += 2;

        selection.begin(40, 21);
        selection.update(10, 21);
        selection.finish();
        if (selection.selectedText() !== "egu") failures += 4;

        selection.clear();
        if (selection.hasSelection || selection.selecting) failures += 8;

        selection.begin(30, 61);
        selection.update(40, 61);
        selection.finish();
        if (selection.selectedText() !== "B") failures += 16;

        selection.begin(10, 61);
        selection.update(20, 61);
        selection.finish();
        if (selection.selectedText() !== "界") failures += 32;

        Qt.exit(failures);
    }
}
