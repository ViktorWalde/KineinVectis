import QtQuick
import KineinVectis

// Valores e aliases do banco são dados. Detecta links realmente desenhados,
// inclusive no aviso vazio, sem abrir URL nem depender do textFormat interno.
Item {
    id: root
    width: 1800
    height: 300
    readonly property string heading: '<a href="https://example.invalid/column">column</a>'
    readonly property string value: '<a href="https://example.invalid/cell">value</a>'
    readonly property string message: '<a href="https://example.invalid/empty">empty</a>'

    KvDataGrid {
        id: grid
        width: root.width
        height: 100
        columns: [{ key: "value", label: root.heading }]
        rows: [{ value: root.value }]
    }
    KvDataGrid {
        id: emptyGrid
        y: 120
        width: root.width
        height: 100
        emptyText: root.message
    }
    KvTooltipHost { id: tooltipHost }

    function findText(item, raw) {
        if (item.text === raw && typeof item.linkAt === "function") return item;
        for (let index = 0; index < item.children.length; index++) {
            const match = root.findText(item.children[index], raw);
            if (match) return match;
        }
        return null;
    }
    function literal(item, raw, label) {
        const rendered = root.findText(item, raw);
        if (!rendered) {
            console.error("FALHOU: texto não renderizado — " + label);
            return 1;
        }
        for (let y = 0; y < rendered.height; y += 2) {
            for (let x = 0; x < rendered.width; x += 2) {
                if (rendered.linkAt(x, y) !== "") {
                    console.error("FALHOU: dados viraram link — " + label);
                    return 1;
                }
            }
        }
        return 0;
    }
    Component.onCompleted: {
        TooltipController.showFor(grid, root.value, "bottom");
        Qt.callLater(() => {
            const failures = root.literal(grid, root.heading, "coluna")
                + root.literal(grid, root.value, "célula")
                + root.literal(emptyGrid, root.message, "aviso vazio")
                + root.literal(tooltipHost, root.value, "dica da célula cortada");
            TooltipController.hideFor(grid);
            Qt.exit(failures === 0 ? 0 : 1);
        });
    }
}
