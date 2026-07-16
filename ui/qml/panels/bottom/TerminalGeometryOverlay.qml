pragma ComponentBehavior: Bound
import QtQuick

// R0 item 5 (docs/roadmaps/26): overlay de diagnostico da geometria do terminal.
//
// Ligado so por KINEIN_TERMINAL_DEBUG_GEOMETRY=1. Existe para responder UMA
// pergunta que numero nao respondeu: o cursor esta onde a celula diz que ele
// esta, ou nao?
//
//   - se o retangulo do cursor coincide com a celula e o GLIFO e que esta fora
//     dela  → o problema e a rasterizacao do Text (R2/R3);
//   - se o retangulo do cursor esta fora da celula → o problema e a posicao do
//     cursor (volta ao R1);
//   - se nao aparece cursor nenhum → a aplicacao escondeu o cursor VT (?25l) e
//     desenha o proprio; o que se ve torto e um span, nao o cursor.
//
// O R1 zerou o erro aritmetico (0.000px em toda coluna) e o sintoma visual NAO
// mudou. Logo a divergencia esta depois das coordenadas. Isto mostra onde.
//
// Nao loga texto do terminal, prompt nem bytes: so metricas e estado do cursor.
Item {
    id: root

    property TerminalMetrics metrics: null
    property var cursor: ({ row: 0, col: 0, visible: false })
    property int gridCols: 0
    property int gridRows: 0
    // Retangulo REAL do cursor desenhado, medido do item de verdade — nao
    // recalculado aqui. Se a gente recalculasse, o overlay concordaria consigo
    // mesmo e nao provaria nada.
    property rect cursorRect: Qt.rect(0, 0, 0, 0)

    // Cores fixas de diagnostico, de proposito fora do Theme: isto nao e UI de
    // produto e nao deve seguir tema nenhum.
    readonly property color corCelula: "#00ff00"
    readonly property color corBaseline: "#ff00ff"
    readonly property color corCursor: "#00ffff"
    readonly property color corAscent: "#ffff00"

    // Grade de celulas: retangulo logico de cada celula visivel.
    Repeater {
        model: root.metrics ? root.gridRows : 0

        delegate: Item {
            id: linhaGuia
            required property int index

            Repeater {
                model: root.gridCols

                delegate: Rectangle {
                    id: celulaGuia
                    required property int index

                    x: root.metrics ? root.metrics.xForColumn(celulaGuia.index) : 0
                    y: root.metrics ? root.metrics.yForRow(linhaGuia.index) : 0
                    width: root.metrics ? root.metrics.cellWidth : 0
                    height: root.metrics ? root.metrics.cellHeight : 0
                    color: "transparent"
                    border.width: 1
                    border.color: root.corCelula
                    opacity: 0.22
                }
            }

            // Baseline da linha: onde o glifo DEVERIA assentar.
            Rectangle {
                x: 0
                y: (root.metrics ? root.metrics.yForRow(linhaGuia.index) : 0)
                   + (root.metrics ? root.metrics.cellBaseline : 0)
                width: root.metrics
                       ? root.metrics.widthForCells(root.gridCols) : 0
                height: 1
                color: root.corBaseline
                opacity: 0.55
            }

            // Topo do ascent: a caixa que o glifo ocupa dentro da celula.
            Rectangle {
                x: 0
                y: (root.metrics ? root.metrics.yForRow(linhaGuia.index) : 0)
                   + (root.metrics ? root.metrics.glyphTop : 0)
                width: root.metrics
                       ? root.metrics.widthForCells(root.gridCols) : 0
                height: 1
                color: root.corAscent
                opacity: 0.4
            }
        }
    }

    // Celula onde o core DIZ que o cursor esta (preenchida).
    Rectangle {
        visible: root.metrics && root.cursor && root.cursor.visible === true
        x: root.metrics ? root.metrics.xForColumn(root.cursor.col || 0) : 0
        y: root.metrics ? root.metrics.yForRow(root.cursor.row || 0) : 0
        width: root.metrics ? root.metrics.cellWidth : 0
        height: root.metrics ? root.metrics.cellHeight : 0
        color: root.corCursor
        opacity: 0.18
        border.width: 1
        border.color: root.corCursor
    }

    // Retangulo EFETIVO do cursor desenhado. Se este nao coincidir com o de
    // cima, o cursor divergiu da celula.
    Rectangle {
        visible: root.cursorRect.width > 0
        x: root.cursorRect.x
        y: root.cursorRect.y
        width: root.cursorRect.width
        height: root.cursorRect.height
        color: "transparent"
        border.width: 1
        border.color: "#ff0000"
    }

    // Leitura numerica, canto superior direito.
    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        width: leitura.implicitWidth + 8
        height: leitura.implicitHeight + 8
        color: "#000000"
        opacity: 0.82

        Text {
            id: leitura
            anchors.centerIn: parent
            color: "#ffffff"
            font.family: "monospace"
            font.pixelSize: 10
            text: !root.metrics ? "sem metrics" :
                  "DPR " + root.metrics.dpr.toFixed(3) + "\n"
                + "glyphAdvance " + root.metrics.glyphAdvance.toFixed(4) + "\n"
                + "cell " + root.metrics.cellWidthPx + "x"
                  + root.metrics.cellHeightPx + " px fis\n"
                + "cell " + root.metrics.cellWidth.toFixed(4) + "x"
                  + root.metrics.cellHeight.toFixed(4) + " log\n"
                + "ascent " + root.metrics.ascent.toFixed(3)
                  + "  descent " + root.metrics.descent.toFixed(3) + "\n"
                + "leading " + root.metrics.leading.toFixed(3)
                  + "  baseline " + root.metrics.cellBaselinePx + "px\n"
                + "glyphTop " + root.metrics.glyphTop.toFixed(3)
                  + "  glyphLeft " + root.metrics.glyphLeft.toFixed(3) + "\n"
                + "cursor celula " + (root.cursor ? root.cursor.col : "?")
                  + "," + (root.cursor ? root.cursor.row : "?")
                  + "  vis " + (root.cursor ? root.cursor.visible : "?")
                  + "  " + (root.cursor ? root.cursor.shape : "?") + "\n"
                + "cursor rect " + root.cursorRect.x.toFixed(2) + ","
                  + root.cursorRect.y.toFixed(2) + " "
                  + root.cursorRect.width.toFixed(2) + "x"
                  + root.cursorRect.height.toFixed(2) + "\n"
                + "esperado    " + (root.metrics
                    ? root.metrics.xForColumn(root.cursor ? (root.cursor.col || 0) : 0).toFixed(2)
                    : "?") + ","
                  + (root.metrics
                    ? root.metrics.yForRow(root.cursor ? (root.cursor.row || 0) : 0).toFixed(2)
                    : "?") + "\n"
                + "legenda: verde=celula magenta=baseline\n"
                + "         amarelo=ascent ciano=celula do cursor\n"
                + "         vermelho=cursor desenhado"
        }
    }

    // R0 item 6: uma linha por mudanca de estado do cursor. Metricas e cursor
    // apenas — nada de conteudo.
    property string ultimoEstado: ""
    onCursorRectChanged: root.registrar()
    onCursorChanged: root.registrar()

    function registrar() {
        if (!metrics || !cursor) {
            return;
        }
        const estado = "KINEIN_TERM_GEOM"
            + " dpr=" + metrics.dpr.toFixed(3)
            + " cell_px=" + metrics.cellWidthPx + "x" + metrics.cellHeightPx
            + " cell_log=" + metrics.cellWidth.toFixed(4)
              + "x" + metrics.cellHeight.toFixed(4)
            + " baseline_px=" + metrics.cellBaselinePx
            + " cursor_cell=" + (cursor.col || 0) + "," + (cursor.row || 0)
            + " cursor_vis=" + (cursor.visible === true)
            + " cursor_shape=" + (cursor.shape !== undefined ? cursor.shape : "?")
            + " rect=" + cursorRect.x.toFixed(2) + "," + cursorRect.y.toFixed(2)
              + "," + cursorRect.width.toFixed(2) + "x" + cursorRect.height.toFixed(2)
            + " esperado=" + metrics.xForColumn(cursor.col || 0).toFixed(2)
              + "," + metrics.yForRow(cursor.row || 0).toFixed(2);
        if (estado !== ultimoEstado) {
            ultimoEstado = estado;
            // console.warn, nao console.log: a categoria de debug do QML vem
            // DESLIGADA por padrao no Qt do Fedora, entao console.log nao
            // apareceria e a instrumentacao seria inutil. Ainda exige
            // QT_ASSUME_STDERR_HAS_CONSOLE=1 para o qml escrever no terminal.
            console.warn(estado);
        }
    }
}
