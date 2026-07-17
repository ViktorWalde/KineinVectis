pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Renderer burro da grade VT e interações visuais locais (seleção/scroll).
Item {
    id: root

    property var lines: []
    property var cursor: ({ row: 0, col: 0, visible: false })
    property var selectionController
    property bool terminalActive: false
    property int scrollOffset: 0
    property int scrollbackMax: 0
    property int gridRows: 0
    property int gridCols: 0
    // R1.4: a MESMA instancia que o painel, a selecao e o resize usam.
    // Ninguem converte celula<->pixel por conta propria aqui.
    property TerminalMetrics metrics: null
    readonly property real charWidth: metrics ? metrics.cellWidth : 0
    readonly property real lineHeight: metrics ? metrics.cellHeight : 0
    property string emptyText: ""
    signal focusRequested()
    signal pasteRequested()
    signal scrollPositionRequested(int offset)

    readonly property bool scrollIndicatorVisible: terminalScrollBar.visible
    readonly property bool scrollIndicatorScrollable: terminalScrollBar.scrollable
    readonly property real contentWidth: Math.max(
            0, grid.width - terminalScrollBar.width - Theme.spacingXSmall)
    readonly property real contentHeight: grid.height

    function ansiColor(value) {
        if (typeof value === "string") return value;
        const number = value | 0;
        if (number < 16) return Theme.terminalPalette[number];
        if (number < 232) {
            const cube = number - 16;
            const convert = function(part) { return part === 0 ? 0 : 55 + part * 40; };
            return Qt.rgba(convert(Math.floor(cube / 36)) / 255,
                           convert(Math.floor((cube % 36) / 6)) / 255,
                           convert(cube % 6) / 255, 1);
        }
        const gray = (8 + (number - 232) * 10) / 255;
        return Qt.rgba(gray, gray, gray, 1);
    }

    function spanFg(span) {
        const fg = span.fg !== undefined ? ansiColor(span.fg) : Theme.textPrimary;
        const bg = span.bg !== undefined ? ansiColor(span.bg) : Theme.backgroundEditor;
        return span.inverse === true ? bg : fg;
    }

    function spanBg(span) {
        const fg = span.fg !== undefined ? ansiColor(span.fg) : Theme.textPrimary;
        const bg = span.bg !== undefined ? ansiColor(span.bg) : "transparent";
        return span.inverse === true ? fg : bg;
    }

    function spanCells(span) {
        if (span.cells !== undefined) {
            return Math.max(0, Number(span.cells));
        }
        // Compatibilidade defensiva com um frame 0.55 que ainda esteja na
        // fila durante a troca do core; 0.56 sempre informa células reais.
        return span.text !== undefined ? Array.from(String(span.text)).length : 0;
    }

    function copySelection() {
        const text = selectionController.selectedText();
        if (text !== "") Clipboard.setText(text);
    }

    // Célula da grade sob um ponto DESTE item. A grade tem margem própria, então
    // quem está fora do viewport não pode dividir por charWidth direto; o
    // mapeamento passa pelo `grid` para sobreviver a mudanças de layout.
    // Reusa o `cellAt` da seleção: uma conversão pixel→célula só, não duas.
    function cellAt(x, y) {
        const local = grid.mapFromItem(root, x, y);
        return selectionController.cellAt(local.x, local.y);
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.backgroundEditor

        Text {
            anchors.centerIn: parent
            visible: root.lines.length === 0
            text: root.emptyText
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeTerminal
        }
    }

    Item {
        id: grid

        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        clip: true

        // CAUSA-RAIZ do cursor "desalocado" (2026-07-16, docs/roadmaps/26 §4.7).
        //
        // Antes isto era um `Column` de `Row`s. O positioner do Qt Quick
        // DESCARTA filhos de largura zero — e uma linha vazia vem do core como
        // `[]` (o `build_line` corta o run final de espaços no estilo default),
        // virando um `Row` sem filhos, logo sem largura. Resultado medido: cada
        // linha vazia sumia do layout e TODO o texto abaixo subia uma linha,
        // enquanto o cursor ficava na posição correta da grade. O cursor não
        // estava errado; o texto é que escorregava para cima.
        //
        // Por isso as correções de sub-pixel (R1) não mudaram nada: o erro era
        // de LINHA INTEIRA. E por isso o Terminal comum passou no aceite e as
        // TUIs não: no shell as linhas vazias ficam abaixo do cursor, numa TUI
        // elas ficam acima.
        //
        // A correção é a regra do R1.4 levada até o fim: a linha é posicionada
        // pela MESMA métrica que o cursor, não por um positioner. Layout não
        // decide geometria de grade.
        Repeater {
            model: root.lines

            delegate: Row {
                id: lineRow
                required property var modelData
                required property int index
                spacing: 0
                y: root.metrics ? root.metrics.yForRow(lineRow.index) : 0
                height: root.lineHeight

                Repeater {
                    model: lineRow.modelData

                    delegate: Rectangle {
                        id: spanCell
                        required property var modelData
                        height: root.lineHeight
                        // `flush_span` no core descarta span de 0 células, então
                        // todo span tem largura >= 1 e o `Row` não o engole — o
                        // colapso só atingia a LINHA vazia, tratada acima.
                        width: root.metrics
                               ? root.metrics.widthForCells(
                                     root.spanCells(spanCell.modelData))
                               : 0
                        color: root.spanBg(spanCell.modelData)

                        Text {
                            id: spanText
                            anchors.fill: parent
                            text: spanCell.modelData.text
                            color: root.spanFg(spanCell.modelData)
                            font.family: Theme.monoFont
                            font.pixelSize: Theme.fontSizeTerminal
                            // ANSI bold continua semanticamente distinto,
                            // mas sem o peso excessivo do Font.Bold (700).
                            font.weight: spanCell.modelData.bold === true
                                         ? Font.Medium : Font.Normal
                            font.italic: spanCell.modelData.italic === true
                            font.underline: spanCell.modelData.underline === true
                            font.preferShaping: false
                            verticalAlignment: Text.AlignVCenter
                        }
                    }
                }
            }
        }

        Rectangle {
            id: cursorBar

            // O core resolve DECSCUSR para uma forma concreta. A UI somente
            // converte a célula VT em pixels, igual no Terminal e no Assistente.
            readonly property string shape: root.cursor.shape !== undefined
                    ? String(root.cursor.shape) : "bar"
            readonly property bool blinking: root.cursor.blinking !== false
            readonly property real thickness: Math.max(
                    2, Math.round(root.charWidth * 0.18))
            readonly property real baseOpacity: shape === "block"
                    ? 0.72 : 0.95
            property real blinkFactor: 1.0

            visible: root.cursor.visible && root.terminalActive
                     && root.scrollOffset === 0
            // R1: NADA de Math.floor aqui. A celula ja cai em pixel fisico
            // inteiro, entao xForColumn e exato e bate com onde o texto foi
            // desenhado. Era este floor, sobre uma celula fracionaria, que
            // deslocava o cursor ate 0,95px conforme a coluna (§4.6).
            x: root.metrics ? root.metrics.xForColumn(root.cursor.col) : 0
            y: (root.metrics ? root.metrics.yForRow(root.cursor.row) : 0)
               + (shape === "underline"
                    ? Math.max(0, root.lineHeight - thickness) : 0)
            width: shape === "block" || shape === "underline"
                   ? root.charWidth : thickness
            height: shape === "underline"
                    ? thickness : root.lineHeight
            radius: shape === "bar" ? 1 : 0
            color: Theme.accentActive
            opacity: baseOpacity * (blinking ? blinkFactor : 1.0)

            SequentialAnimation on blinkFactor {
                running: cursorBar.visible && cursorBar.blinking
                loops: Animation.Infinite
                NumberAnimation { from: 1.0; to: 0.29; duration: 520 }
                NumberAnimation { from: 0.29; to: 1.0; duration: 520 }
            }
        }

        Item {
            id: selectionLayer
            anchors.fill: parent
            visible: root.selectionController.hasSelection
                     || root.selectionController.selecting
            readonly property var selected: root.selectionController.range()

            Rectangle {
                color: Theme.accent
                opacity: 0.28
                x: root.metrics ? root.metrics.xForColumn(selectionLayer.selected.c1) : 0
                y: root.metrics ? root.metrics.yForRow(selectionLayer.selected.r1) : 0
                height: root.lineHeight
                width: !root.metrics
                       ? 0
                       : selectionLayer.selected.r1 === selectionLayer.selected.r2
                         ? root.metrics.widthForCells(
                               selectionLayer.selected.c2
                               - selectionLayer.selected.c1)
                         : root.contentWidth
                           - root.metrics.xForColumn(selectionLayer.selected.c1)
            }

            Rectangle {
                visible: selectionLayer.selected.r2 > selectionLayer.selected.r1 + 1
                color: Theme.accent
                opacity: 0.28
                x: 0
                y: root.metrics ? root.metrics.yForRow(selectionLayer.selected.r1 + 1) : 0
                width: root.contentWidth
                height: root.metrics
                        ? root.metrics.yForRow(selectionLayer.selected.r2
                                               - selectionLayer.selected.r1 - 1)
                        : 0
            }

            Rectangle {
                visible: selectionLayer.selected.r2 > selectionLayer.selected.r1
                color: Theme.accent
                opacity: 0.28
                x: 0
                y: root.metrics ? root.metrics.yForRow(selectionLayer.selected.r2) : 0
                width: root.metrics ? root.metrics.xForColumn(selectionLayer.selected.c2) : 0
                height: root.lineHeight
            }
        }

        // R0 item 5: overlay de geometria, so com
        // KINEIN_TERMINAL_DEBUG_GEOMETRY=1. Sem a env o Loader nao instancia
        // nada e o uso normal nao paga. Fica DENTRO de `grid` e depois da
        // Column, entao desenha sobre a mesma origem que o texto e o cursor.
        Loader {
            id: geometryOverlay

            anchors.fill: parent
            z: 2
            active: DebugFlags.terminalGeometry
            sourceComponent: TerminalGeometryOverlay {
                metrics: root.metrics
                cursor: root.cursor
                gridCols: root.gridCols
                gridRows: root.lines.length
                // Medido do item real: se o overlay recalculasse, ele
                // concordaria consigo mesmo e nao provaria nada.
                cursorRect: Qt.rect(cursorBar.x, cursorBar.y,
                                    cursorBar.visible ? cursorBar.width : 0,
                                    cursorBar.visible ? cursorBar.height : 0)
            }
        }

        MouseArea {
            anchors.fill: parent
            anchors.rightMargin: terminalScrollBar.width + Theme.spacingXSmall
            acceptedButtons: Qt.LeftButton | Qt.MiddleButton
            cursorShape: Qt.IBeamCursor

            onPressed: function(mouse) {
                root.focusRequested();
                if (mouse.button === Qt.MiddleButton) {
                    root.pasteRequested();
                } else {
                    root.selectionController.begin(mouse.x, mouse.y);
                }
            }
            onPositionChanged: function(mouse) {
                root.selectionController.update(mouse.x, mouse.y);
            }
            onReleased: root.selectionController.finish()
        }

        VerticalScrollBar {
            id: terminalScrollBar
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            z: 3
            contentSize: root.scrollbackMax + root.gridRows
            viewportSize: root.gridRows
            position: root.scrollbackMax - root.scrollOffset
            showWhenIdle: root.terminalActive

            onMoveRequested: function(position) {
                const offset = Math.round(root.scrollbackMax - position);
                root.scrollPositionRequested(Math.max(
                    0, Math.min(root.scrollbackMax, offset)));
            }
        }
    }

}
