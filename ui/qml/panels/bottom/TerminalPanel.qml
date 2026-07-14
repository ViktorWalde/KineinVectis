pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Terminal profissional (D2, docs/24): renderiza o GRID vindo do core
// (event.terminal.render → { cols, rows, cursor, lines:[[span]] }) com cores,
// cursor e atributos, e captura o teclado CARACTERE-A-CARACTERE, mandando
// bytes crus pro PTY (incl. control chars). Calcula cols/rows do tamanho do
// painel e pede resize. Componente burro: estado por property, ações por sinal.
Item {
    id: panel

    property var render: ({})
    property bool terminalActive: false
    property bool workspaceAvailable: false
    property string emptyText: qsTr("Seu shell ($SHELL) abre aqui na raiz do workspace (Alt+F12).")

    signal openRequested()
    signal keyPressed(string data)
    signal resizeRequested(int cols, int rows)
    signal scrollRequested(int offset)

    // D2.2 (docs/24): scrollback e seleção para copiar.
    property int scrollOffset: 0
    property bool selecting: false
    property bool hasSelection: false
    property int selAnchorRow: 0
    property int selAnchorCol: 0
    property int selHeadRow: 0
    property int selHeadCol: 0
    // Coalescência de scroll: arrastar o polegar pode produzir centenas de
    // movimentos por segundo, mas cada terminal.scroll serializa um grid
    // completo. A posição visual é otimista e o core recebe no máximo um
    // pedido por frame.
    property int pendingScrollOffset: -1
    property int awaitingScrollOffset: -1
    property string renderedSessionId: ""

    focus: true

    readonly property var lines: (render && render.lines) ? render.lines : []
    readonly property var cursor: (render && render.cursor)
            ? render.cursor : ({ row: 0, col: 0, visible: false })

    // B2 (docs/24): quantas linhas cabem na tela e quanto histórico existe. O
    // core manda os dois no render (protocolo 0.43.0) — sem `scrollbackMax` a
    // UI não tem como desenhar uma barra proporcional nem saber se há o que
    // rolar (0 = terminal recém-aberto, "não rolar" é o correto).
    readonly property int gridRows: (render && render.rows) ? render.rows : 0
    readonly property int scrollbackMax: (render && render.scrollbackMax !== undefined)
            ? render.scrollbackMax : 0

    // O CORE é a fonte da verdade do offset: ele clampa o pedido ao histórico
    // real. A UI aplica local pra ter resposta imediata e reconcilia aqui.
    onRenderChanged: {
        const nextSessionId = render && render.id !== undefined
                ? String(render.id) : "";
        if (nextSessionId !== renderedSessionId) {
            scrollFlush.stop();
            pendingScrollOffset = -1;
            awaitingScrollOffset = -1;
            renderedSessionId = nextSessionId;
        }
        if (render && render.scrollback !== undefined) {
            const confirmed = Number(render.scrollback);
            if (confirmed === awaitingScrollOffset) {
                awaitingScrollOffset = -1;
            }
            // Um render de output pode cruzar com um arrasto já enviado. Não
            // deixa esse frame antigo puxar o polegar de volta.
            if (pendingScrollOffset < 0 && awaitingScrollOffset < 0) {
                scrollOffset = confirmed;
            }
        }
    }

    function queueScroll(next) {
        const clamped = Math.max(0, Math.min(panel.scrollbackMax, next));
        panel.scrollOffset = clamped;
        panel.pendingScrollOffset = clamped;
        if (!scrollFlush.running) {
            scrollFlush.start();
        }
    }

    // Rola `lines` linhas (positivo = pra cima/histórico). Clampado ao que
    // existe, então a barra nunca mente.
    function scrollBy(lines) {
        const next = Math.max(0, Math.min(panel.scrollbackMax, panel.scrollOffset + lines));
        if (next !== panel.scrollOffset) {
            panel.queueScroll(next);
        }
    }

    Timer {
        id: scrollFlush

        interval: 16
        repeat: false
        onTriggered: {
            if (panel.pendingScrollOffset < 0) {
                return;
            }
            const offset = panel.pendingScrollOffset;
            panel.pendingScrollOffset = -1;
            panel.awaitingScrollOffset = offset;
            panel.scrollRequested(offset);
        }
    }

    TextMetrics {
        id: metrics

        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeTerminal
        text: "M"
    }

    readonly property real charWidth: metrics.advanceWidth
    readonly property real lineHeight: metrics.height

    function focusInput() {
        panel.forceActiveFocus();
    }

    // Sem campo de linha no modo grid; mantido pela API do host (no-op).
    function clearInput() {
    }

    onVisibleChanged: {
        if (!visible) {
            return;
        }
        if (!terminalActive && workspaceAvailable) {
            openRequested();
        }
        panel.forceActiveFocus();
        recomputeSize();
    }

    function recomputeSize() {
        if (charWidth <= 0 || lineHeight <= 0 || grid.width <= 0 || grid.height <= 0) {
            return;
        }
        const cols = Math.max(2, Math.floor(grid.width / charWidth));
        const rows = Math.max(2, Math.floor(grid.height / lineHeight));
        panel.resizeRequested(cols, rows);
    }

    // Converte cor ANSI (índice 0–255 ou "#rrggbb") em cor Qt.
    function ansiColor(value) {
        if (typeof value === "string") {
            return value;
        }
        const n = value | 0;
        if (n < 16) {
            return Theme.terminalPalette[n];
        }
        if (n < 232) {
            const c = n - 16;
            const r = Math.floor(c / 36);
            const g = Math.floor((c % 36) / 6);
            const b = c % 6;
            const conv = function(x) { return x === 0 ? 0 : 55 + x * 40; };
            return Qt.rgba(conv(r) / 255, conv(g) / 255, conv(b) / 255, 1);
        }
        const gray = (8 + (n - 232) * 10) / 255;
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

    // Célula (row, col) sob o ponto (x, y) dentro da grade.
    function cellAt(x, y) {
        if (charWidth <= 0 || lineHeight <= 0) {
            return { "row": 0, "col": 0 };
        }
        const row = Math.max(0, Math.floor(y / lineHeight));
        const col = Math.max(0, Math.floor(x / charWidth));
        return { "row": row, "col": col };
    }

    // Texto puro de uma linha (concatena os spans).
    function lineText(row) {
        if (row < 0 || row >= lines.length) {
            return "";
        }
        const spans = lines[row];
        let text = "";
        for (let i = 0; i < spans.length; i++) {
            text += spans[i].text;
        }
        return text;
    }

    // Seleção normalizada em ordem de leitura {r1,c1,r2,c2}.
    function selectionRange() {
        let r1 = selAnchorRow;
        let c1 = selAnchorCol;
        let r2 = selHeadRow;
        let c2 = selHeadCol;
        if (r2 < r1 || (r2 === r1 && c2 < c1)) {
            r1 = selHeadRow; c1 = selHeadCol; r2 = selAnchorRow; c2 = selAnchorCol;
        }
        return { "r1": r1, "c1": c1, "r2": r2, "c2": c2 };
    }

    // Extrai o texto selecionado (seleção linear estilo xterm).
    function selectionText() {
        if (!hasSelection) {
            return "";
        }
        const s = selectionRange();
        if (s.r1 === s.r2) {
            return lineText(s.r1).substring(s.c1, s.c2);
        }
        let out = lineText(s.r1).substring(s.c1);
        for (let r = s.r1 + 1; r < s.r2; r++) {
            out += "\n" + lineText(r);
        }
        out += "\n" + lineText(s.r2).substring(0, s.c2);
        return out;
    }

    function copySelection() {
        const text = selectionText();
        if (text.length > 0) {
            Clipboard.setText(text);
        }
    }

    function paste() {
        const text = Clipboard.text();
        if (text.length > 0) {
            snapToBottom();
            panel.keyPressed(text);
        }
    }

    function snapToBottom() {
        // Input precisa chegar DEPOIS do retorno ao vivo, sem esperar o
        // coalescedor; assim o prompt/comando atual nunca fica fora da tela.
        const needsConfirmation = scrollOffset !== 0
                || pendingScrollOffset >= 0 || awaitingScrollOffset >= 0;
        scrollFlush.stop();
        pendingScrollOffset = -1;
        awaitingScrollOffset = -1;
        if (needsConfirmation) {
            scrollOffset = 0;
            panel.scrollRequested(0);
        }
    }

    function handleKey(event) {
        if (!panel.terminalActive) {
            panel.openRequested();
            event.accepted = true;
            return;
        }
        const ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
        const shift = (event.modifiers & Qt.ShiftModifier) !== 0;
        // Copiar/colar (Ctrl+Shift+C/V não conflitam com Ctrl+C/V do shell).
        if (ctrl && shift && event.key === Qt.Key_C) {
            panel.copySelection();
            event.accepted = true;
            return;
        }
        if (ctrl && shift && event.key === Qt.Key_V) {
            panel.paste();
            event.accepted = true;
            return;
        }
        let data = "";
        switch (event.key) {
        case Qt.Key_Return:
        case Qt.Key_Enter: data = "\r"; break;
        case Qt.Key_Backspace: data = "\x7f"; break;
        case Qt.Key_Tab: data = "\t"; break;
        case Qt.Key_Escape: data = "\x1b"; break;
        case Qt.Key_Up: data = "\x1b[A"; break;
        case Qt.Key_Down: data = "\x1b[B"; break;
        case Qt.Key_Right: data = "\x1b[C"; break;
        case Qt.Key_Left: data = "\x1b[D"; break;
        case Qt.Key_Home: data = "\x1b[H"; break;
        case Qt.Key_End: data = "\x1b[F"; break;
        case Qt.Key_PageUp: data = "\x1b[5~"; break;
        case Qt.Key_PageDown: data = "\x1b[6~"; break;
        case Qt.Key_Delete: data = "\x1b[3~"; break;
        default:
            if (ctrl && event.key >= Qt.Key_A && event.key <= Qt.Key_Z) {
                data = String.fromCharCode(event.key - Qt.Key_A + 1);
            } else if (event.text.length > 0) {
                data = event.text;
            }
        }
        if (data.length > 0) {
            panel.snapToBottom();
            panel.keyPressed(data);
            event.accepted = true;
        }
    }

    Keys.onPressed: function(event) {
        panel.handleKey(event);
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.backgroundEditor

        Text {
            anchors.centerIn: parent
            visible: panel.lines.length === 0
            text: panel.emptyText
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

        onWidthChanged: panel.recomputeSize()
        onHeightChanged: panel.recomputeSize()

        Column {
            spacing: 0

            Repeater {
                model: panel.lines

                delegate: Row {
                    id: lineRow

                    required property var modelData

                    spacing: 0
                    height: panel.lineHeight

                    Repeater {
                        model: lineRow.modelData

                        delegate: Rectangle {
                            id: spanCell

                            required property var modelData

                            height: panel.lineHeight
                            width: spanText.implicitWidth
                            color: panel.spanBg(spanCell.modelData)

                            Text {
                                id: spanText

                                text: spanCell.modelData.text
                                color: panel.spanFg(spanCell.modelData)
                                font.family: Theme.monoFont
                                font.pixelSize: Theme.fontSizeTerminal
                                font.bold: spanCell.modelData.bold === true
                                font.italic: spanCell.modelData.italic === true
                                font.underline: spanCell.modelData.underline === true
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            visible: panel.cursor.visible && panel.terminalActive
                     && panel.scrollOffset === 0
            x: panel.cursor.col * panel.charWidth
            y: panel.cursor.row * panel.lineHeight
            width: panel.charWidth
            height: panel.lineHeight
            color: Theme.accent
            opacity: 0.55
        }

        // Realce da seleção (linear, estilo xterm): até 3 retângulos.
        Item {
            id: selectionLayer

            anchors.fill: parent
            visible: panel.hasSelection || panel.selecting

            readonly property var range: panel.selectionRange()

            Rectangle {
                color: Theme.accent
                opacity: 0.28
                x: selectionLayer.range.c1 * panel.charWidth
                y: selectionLayer.range.r1 * panel.lineHeight
                height: panel.lineHeight
                width: selectionLayer.range.r1 === selectionLayer.range.r2
                        ? (selectionLayer.range.c2 - selectionLayer.range.c1) * panel.charWidth
                        : grid.width - selectionLayer.range.c1 * panel.charWidth
            }

            Rectangle {
                visible: selectionLayer.range.r2 > selectionLayer.range.r1 + 1
                color: Theme.accent
                opacity: 0.28
                x: 0
                y: (selectionLayer.range.r1 + 1) * panel.lineHeight
                width: grid.width
                height: (selectionLayer.range.r2 - selectionLayer.range.r1 - 1) * panel.lineHeight
            }

            Rectangle {
                visible: selectionLayer.range.r2 > selectionLayer.range.r1
                color: Theme.accent
                opacity: 0.28
                x: 0
                y: selectionLayer.range.r2 * panel.lineHeight
                width: selectionLayer.range.c2 * panel.charWidth
                height: panel.lineHeight
            }
        }

        MouseArea {
            id: gridMouse

            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.MiddleButton
            cursorShape: Qt.IBeamCursor

            onPressed: function(mouse) {
                panel.forceActiveFocus();
                if (mouse.button === Qt.MiddleButton) {
                    panel.paste();
                    return;
                }
                const cell = panel.cellAt(mouse.x, mouse.y);
                panel.selAnchorRow = cell.row;
                panel.selAnchorCol = cell.col;
                panel.selHeadRow = cell.row;
                panel.selHeadCol = cell.col;
                panel.selecting = true;
                panel.hasSelection = false;
            }

            onPositionChanged: function(mouse) {
                if (!panel.selecting) {
                    return;
                }
                const cell = panel.cellAt(mouse.x, mouse.y);
                panel.selHeadRow = cell.row;
                panel.selHeadCol = cell.col;
                panel.hasSelection = panel.selHeadRow !== panel.selAnchorRow
                        || panel.selHeadCol !== panel.selAnchorCol;
            }

            onReleased: function(mouse) {
                panel.selecting = false;
            }
        }

        // B2: barra de rolagem do terminal. SINTÉTICA — não há Flickable aqui:
        // a posição vem do scrollback do emulador (linhas), não de um contentY.
        // Eixo invertido: scrollOffset conta do FUNDO (0 = ao vivo), a barra
        // conta do TOPO.
        VerticalScrollBar {
            id: terminalScrollBar

            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom

            contentSize: panel.scrollbackMax + panel.gridRows
            viewportSize: panel.gridRows
            position: panel.scrollbackMax - panel.scrollOffset
            showWhenIdle: panel.terminalActive

            onMoveRequested: function(position) {
                const offset = Math.round(panel.scrollbackMax - position);
                const next = Math.max(0, Math.min(panel.scrollbackMax, offset));
                if (next !== panel.scrollOffset) {
                    panel.queueScroll(next);
                }
            }
        }
    }

    // B1 (docs/24): a roda do mouse não rolava o terminal. Era um `onWheel` num
    // MouseArea DENTRO do grid — e MouseArea perde o evento de roda quando
    // outro item segura o grab do ponteiro (ex.: durante o arrasto da seleção).
    // Um WheelHandler no root do painel recebe a roda sobre a área inteira,
    // independente de grab. Fica no root (não no grid) de propósito.
    WheelHandler {
        id: terminalWheel

        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad

        onWheel: function(event) {
            const magnitude = Math.max(1, Math.round(
                Math.abs(event.angleDelta.y) / 120 * 3));
            panel.scrollBy(event.angleDelta.y > 0 ? magnitude : -magnitude);
        }
    }
}
