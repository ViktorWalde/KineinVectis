import QtQuick
import KineinVectis

// O pedido de resize do terminal (16.3, roadmaps/59 §7.1.3): o
// TerminalResizeController e o TerminalMetrics REAIS. As larguras sao
// "N celulas e meia", para a conta de colunas ser exata.
Item {
    id: root

    property var sent: []
    property int stage: 0
    property int failures: 0

    TerminalMetrics {
        id: metrics
        fontFamily: "monospace"
        fontPixelSize: 13
    }
    TerminalResizeController {
        id: resizeController
        metrics: metrics
        onResizeRequested: (cols, rows) => root.sent = root.sent.concat([cols])
    }

    function widthFor(cols) { return (cols + 0.5) * metrics.cellWidth; }
    readonly property real viewportHeight: 10.5 * metrics.cellHeight

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label + " " + JSON.stringify(root.sent)); }
    }

    Timer {
        interval: 80
        running: true
        repeat: true
        onTriggered: {
            if (root.stage === 0) {
                resizeController.measure(root.widthFor(121), root.viewportHeight);
            } else if (root.stage === 1) {
                root.check(JSON.stringify(root.sent) === "[121]", "o primeiro tamanho sai");
                // Um passo intermediario e a volta ao tamanho que o core ja' tem,
                // dentro da janela de coalescencia: nada sai.
                resizeController.measure(root.widthFor(133), root.viewportHeight);
                resizeController.measure(root.widthFor(121), root.viewportHeight);
            } else if (root.stage === 2) {
                root.check(JSON.stringify(root.sent) === "[121]", "o intermediario nao sai depois da volta");
                resizeController.measure(root.widthFor(80), root.viewportHeight);
                resizeController.measure(root.widthFor(46), root.viewportHeight);
            } else if (root.stage === 3) {
                root.check(JSON.stringify(root.sent) === "[121,46]", "sai so' o ultimo tamanho");
                resizeController.reset();
                resizeController.measure(root.widthFor(46), root.viewportHeight);
            } else {
                root.check(JSON.stringify(root.sent) === "[121,46,46]", "outra sessao pede de novo");
                root.check(resizeController.lastRequestedRows === 10, "linhas: " + resizeController.lastRequestedRows);
                Qt.exit(root.failures === 0 ? 0 : 1);
            }
            root.stage++;
        }
    }
}
