import QtQuick
import "../../ui/qml/panels/bottom"

// R0 item 5/6 (DocsPublic/roadmaps/26): o overlay de geometria mede o cursor REAL e
// aponta divergencia; ele nao pode concordar consigo mesmo.
//
// Por que testar instrumentacao: um overlay que calcula o proprio "esperado" e
// o proprio "obtido" sempre bate, e prova nada. O contrato aqui e que
// `cursorRect` venha de FORA (medido do item desenhado) e seja confrontado com
// o que a metrica diz. Este harness fixa isso.
//
// Tambem protege o item 6: o log carrega SO metrica e estado do cursor. Se
// alguem acrescentar texto do terminal ali, isto quebra.
Item {
    id: root
    width: 400
    height: 200

    property var logged: []

    TerminalMetrics {
        id: metrics
        fontFamily: "monospace"
        fontPixelSize: 13
        devicePixelRatio: 1.0
    }

    TerminalGeometryOverlay {
        id: overlay
        anchors.fill: parent
        metrics: metrics
        gridCols: 80
        gridRows: 24
        cursor: ({ "row": 3, "col": 10, "visible": true, "shape": "bar" })
    }

    Component.onCompleted: {
        let failures = 0;

        // O overlay expoe o esperado a partir da metrica unica.
        const esperadoX = metrics.xForColumn(10);
        const esperadoY = metrics.yForRow(3);

        // Caso alinhado: o cursor desenhado cai na celula. O overlay tem que
        // refletir isso sem inventar.
        overlay.cursorRect = Qt.rect(esperadoX, esperadoY, 2, metrics.cellHeight);
        if (Math.abs(overlay.cursorRect.x - esperadoX) > 1e-9) failures += 1;
        if (overlay.ultimoEstado === "") failures += 2;
        if (overlay.ultimoEstado.indexOf("KINEIN_TERM_GEOM") !== 0) failures += 4;

        // O log carrega metrica e cursor — e NADA de conteudo. Estes campos
        // sao o contrato do item 6.
        for (const campo of ["dpr=", "cell_px=", "cell_log=", "baseline_px=",
                             "cursor_cell=", "cursor_vis=", "cursor_shape=",
                             "rect=", "esperado="]) {
            if (overlay.ultimoEstado.indexOf(campo) < 0) {
                failures += 8;
                break;
            }
        }

        // Caso divergente: o cursor desenhado fora da celula tem que produzir
        // um estado DIFERENTE. Se o overlay recalculasse `rect` a partir da
        // metrica, este teste falharia — que e o ponto.
        const antes = overlay.ultimoEstado;
        overlay.cursorRect = Qt.rect(esperadoX + 7, esperadoY + 3, 2,
                                     metrics.cellHeight);
        if (overlay.ultimoEstado === antes) failures += 16;
        if (overlay.ultimoEstado.indexOf("esperado=" + esperadoX.toFixed(2)) < 0) {
            failures += 32;
        }

        // Estado repetido nao gera linha nova (item 6: uma por MUDANCA).
        const estavel = overlay.ultimoEstado;
        overlay.cursorRect = Qt.rect(esperadoX + 7, esperadoY + 3, 2,
                                     metrics.cellHeight);
        if (overlay.ultimoEstado !== estavel) failures += 64;

        // Cursor escondido pela aplicacao (?25l, que o claude emite) tem que
        // ser observavel — e uma das tres saidas que o overlay precisa separar.
        overlay.cursor = ({ "row": 3, "col": 10, "visible": false, "shape": "bar" });
        overlay.cursorRect = Qt.rect(0, 0, 0, 0);
        if (overlay.ultimoEstado.indexOf("cursor_vis=false") < 0) failures += 128;

        // Sem metrica nao quebra nem loga lixo.
        overlay.metrics = null;
        overlay.cursorRect = Qt.rect(1, 1, 1, 1);
        if (overlay.ultimoEstado.indexOf("cursor_vis=false") < 0) failures += 256;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
