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
    property real charWidth: 0
    property real lineHeight: 0
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

        Column {
            width: root.contentWidth
            spacing: 0

            Repeater {
                model: root.lines

                delegate: Row {
                    id: lineRow
                    required property var modelData
                    spacing: 0
                    height: root.lineHeight

                    Repeater {
                        model: lineRow.modelData

                        delegate: Rectangle {
                            id: spanCell
                            required property var modelData
                            height: root.lineHeight
                            width: root.spanCells(spanCell.modelData)
                                   * root.charWidth
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
        }

        Rectangle {
            id: cursorBar

            // O core resolve DECSCUSR para uma forma concreta. A UI somente
            // converte a célula VT em pixels, igual no Terminal e KV Context.
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
            x: Math.floor(root.cursor.col * root.charWidth)
            y: Math.floor(root.cursor.row * root.lineHeight)
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
                x: selectionLayer.selected.c1 * root.charWidth
                y: selectionLayer.selected.r1 * root.lineHeight
                height: root.lineHeight
                width: selectionLayer.selected.r1 === selectionLayer.selected.r2
                       ? (selectionLayer.selected.c2 - selectionLayer.selected.c1)
                         * root.charWidth
                       : root.contentWidth
                         - selectionLayer.selected.c1 * root.charWidth
            }

            Rectangle {
                visible: selectionLayer.selected.r2 > selectionLayer.selected.r1 + 1
                color: Theme.accent
                opacity: 0.28
                x: 0
                y: (selectionLayer.selected.r1 + 1) * root.lineHeight
                width: root.contentWidth
                height: (selectionLayer.selected.r2 - selectionLayer.selected.r1 - 1)
                        * root.lineHeight
            }

            Rectangle {
                visible: selectionLayer.selected.r2 > selectionLayer.selected.r1
                color: Theme.accent
                opacity: 0.28
                x: 0
                y: selectionLayer.selected.r2 * root.lineHeight
                width: selectionLayer.selected.c2 * root.charWidth
                height: root.lineHeight
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
