pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Terminal profissional (D2, docs/roadmaps/24): renderiza o GRID vindo do core
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
    // Rolagem explícita do histórico (barra, snap-to-bottom).
    signal scrollRequested(int offset)
    // Gesto de roda cru; `modifiers` são os do Qt e só o CoreClient os traduz
    // para o contrato. Quem decide o destino é o core (protocolo 0.60.0).
    signal wheelRequested(int col, int row, int lines, int modifiers)

    // D2.2 (docs/roadmaps/24): scrollback sintetico; estado/coalescencia vivem no
    // controller dedicado para serem testados fora da superficie visual.
    property alias scrollOffset: scrollController.scrollOffset
    readonly property int pendingScrollOffset:
        scrollController.pendingScrollOffset
    readonly property int awaitingScrollOffset:
        scrollController.awaitingScrollOffset
    readonly property string renderedSessionId:
        scrollController.renderedSessionId

    focus: true

    readonly property var lines: (render && render.lines) ? render.lines : []
    readonly property var cursor: (render && render.cursor)
            ? render.cursor : ({ row: 0, col: 0, visible: false })
    readonly property bool alternateScreen: render
                                                   && render.alternateScreen === true
    readonly property bool applicationCursor: render
                                                    && render.applicationCursor === true
    readonly property bool bracketedPaste: render
                                                  && render.bracketedPaste === true

    // B2 (docs/roadmaps/24): quantas linhas cabem na tela e quanto histórico existe. O
    // core manda os dois no render (protocolo 0.43.0) — sem `scrollbackMax` a
    // UI não tem como desenhar uma barra proporcional nem saber se há o que
    // rolar (0 = terminal recém-aberto, "não rolar" é o correto).
    readonly property int gridRows: (render && render.rows) ? render.rows : 0
    readonly property int gridCols: (render && render.cols) ? render.cols : 0
    readonly property int scrollbackMax: scrollController.scrollbackMax
    readonly property bool scrollIndicatorVisible: terminalViewport.scrollIndicatorVisible
    readonly property bool scrollIndicatorScrollable: terminalViewport.scrollIndicatorScrollable
    readonly property real terminalContentWidth: terminalViewport.contentWidth
    onRenderChanged: {
        const nextSessionId = render && render.id !== undefined
                ? String(render.id) : "";
        if (nextSessionId !== renderedSessionId) {
            resizeFlush.stop();
            pendingCols = 0;
            pendingRows = 0;
            lastRequestedCols = 0;
            lastRequestedRows = 0;
            selectionController.clear();
            Qt.callLater(panel.recomputeSize);
        }
        scrollController.handleRender(render);
    }

    function queueScroll(next) {
        scrollController.queueScroll(next);
    }

    // R1 (docs/roadmaps/26 §4.6): a ÚNICA fonte de métricas de célula. Antes o
    // painel media aqui (`TextMetrics.advanceWidth`/`height`) e cada consumidor
    // arredondava do seu jeito — o cursor com `Math.floor`, o texto sem
    // arredondar. Com a célula fracionária isso divergia até 0,95px, variando
    // com a coluna. Agora a grade cai em pixel físico inteiro e todo mundo
    // deriva daqui.
    TerminalMetrics {
        id: cellMetrics

        fontFamily: Theme.monoFont
        fontPixelSize: Theme.fontSizeTerminal
        devicePixelRatio: Screen.devicePixelRatio
    }

    readonly property TerminalMetrics metrics: cellMetrics
    readonly property real charWidth: cellMetrics.cellWidth
    readonly property real lineHeight: cellMetrics.cellHeight
    property int pendingCols: 0
    property int pendingRows: 0
    property int lastRequestedCols: 0
    property int lastRequestedRows: 0

    TerminalSelectionController {
        id: selectionController

        lines: panel.lines
        metrics: cellMetrics
    }

    TerminalScrollController {
        id: scrollController

        onScrollRequested: function(offset) {
            panel.scrollRequested(offset);
        }
    }

    TerminalInputController {
        id: inputController

        terminalActive: panel.terminalActive
        applicationCursor: panel.applicationCursor
        onOpenRequested: panel.openRequested()
        onCopyRequested: panel.copySelection()
        onPasteRequested: panel.paste()
        onDataRequested: function(data) {
            panel.snapToBottom();
            panel.keyPressed(data);
        }
    }

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
        if (charWidth <= 0 || lineHeight <= 0
                || terminalContentWidth <= 0
                || terminalViewport.contentHeight <= 0) {
            return;
        }
        // R1.4: o resize consome a mesma instância que o texto e o cursor. Se
        // ele contasse células por conta própria, o grid pedido ao core poderia
        // não ser o grid desenhado.
        const cols = Math.max(2, cellMetrics.columnsIn(terminalContentWidth));
        const rows = Math.max(2, cellMetrics.rowsIn(
            terminalViewport.contentHeight));
        // Math.floor/Math.max chegam ao cache AOT como double. Comparar pela
        // distancia inteira evita -Wfloat-equal no C++ gerado pelo Qt.
        if (Math.abs(cols - lastRequestedCols) < 0.5
                && Math.abs(rows - lastRequestedRows) < 0.5) {
            return;
        }
        pendingCols = cols;
        pendingRows = rows;
        if (!resizeFlush.running) {
            resizeFlush.start();
        }
    }

    // Splitters e maximize podem gerar dezenas de eventos de geometria por
    // segundo. Um terminal nativo tambem coalesce resize: evita serializar um
    // grid completo para cada pixel durante o arrasto e preserva fluidez.
    Timer {
        id: resizeFlush

        interval: 33
        repeat: false
        onTriggered: {
            if (panel.pendingCols <= 0 || panel.pendingRows <= 0) {
                return;
            }
            panel.lastRequestedCols = panel.pendingCols;
            panel.lastRequestedRows = panel.pendingRows;
            panel.resizeRequested(panel.pendingCols, panel.pendingRows);
        }
    }

    function copySelection() {
        terminalViewport.copySelection();
    }

    function paste() {
        const text = Clipboard.text();
        if (text !== "") {
            snapToBottom();
            const data = panel.bracketedPaste
                    ? "\x1b[200~" + text + "\x1b[201~" : text;
            panel.keyPressed(data);
        }
    }

    function snapToBottom() {
        // O controller emite o retorno ao vivo sincronamente; o byte de input
        // só é enviado depois pelo chamador.
        scrollController.snapToBottom();
    }

    Keys.onPressed: function(event) {
        inputController.handleKey(event);
    }

    TerminalViewport {
        id: terminalViewport

        anchors.fill: parent
        lines: panel.lines
        cursor: panel.cursor
        selectionController: selectionController
        metrics: cellMetrics
        terminalActive: panel.terminalActive
        scrollOffset: panel.scrollOffset
        scrollbackMax: panel.scrollbackMax
        gridRows: panel.gridRows
        gridCols: panel.gridCols
        emptyText: panel.emptyText
        onContentWidthChanged: panel.recomputeSize()
        onContentHeightChanged: panel.recomputeSize()
        onFocusRequested: panel.forceActiveFocus()
        onPasteRequested: panel.paste()
        onScrollPositionRequested: function(offset) {
            if (offset !== panel.scrollOffset) panel.queueScroll(offset);
        }
    }

    // A roda fica na composicao do terminal, como na implementacao D2
    // validada ao vivo. O viewport desenha; este nivel coordena input e não
    // perde o gesto para camadas internas de selecao/barra.
    //
    // O gesto é REPORTADO, não interpretado: a UI diz "rodou N linhas na célula
    // (col,row) com estes modificadores" e o core decide se isso vira relatório
    // à aplicação, cursor keys ou rolagem do histórico. A UI não tem como saber
    // — o modo VT é estado do terminal.
    WheelHandler {
        target: null
        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
        onWheel: function(event) {
            const lines = scrollController.linesFromWheel(
                    event.angleDelta.y, event.pixelDelta.y, panel.lineHeight);
            if (lines === 0) {
                return;
            }
            const cell = terminalViewport.cellAt(event.x, event.y);
            // O ponteiro pode estar sobre a barra ou a margem: a célula
            // reportada tem que existir na grade.
            const col = Math.max(0, Math.min(
                Math.max(0, panel.gridCols - 1), cell.col));
            const row = Math.max(0, Math.min(
                Math.max(0, panel.gridRows - 1), cell.row));
            panel.wheelRequested(col, row, lines, event.modifiers);
            event.accepted = true;
        }
    }
}
