import QtQuick

// O PEDIDO DE RESIZE do terminal: quantas colunas e linhas cabem, coalescido
// (16.3, roadmaps/59 §7.1.3). Fora da superficie visual para ser testado sem
// ela, como o TerminalScrollController.
//
// O DEFEITO QUE TROUXE ISTO PARA CA' (medido em 2026-10-16). Ao alargar, as
// linhas nao refluiam: o painel calculava as colunas por uma propriedade
// DERIVADA da largura do viewport, lida dentro do sinal de mudanca dessa
// largura — antes de a derivada ser reavaliada. Todo pedido saia com a
// largura ANTERIOR: estreitando a janela, 40 colunas onde cabiam ~50; voltando
// a' mesma largura, 124 onde cabiam 115 (o passo intermediario, com o slot da
// esquerda ainda estreito), e a linha longa ficava cortada na borda. Por isso
// `measure` recebe a largura e a altura de quem as tem AGORA.
//
// R1.4: as celulas saem da MESMA instancia de metricas que o texto e o cursor;
// contadas a parte, o grid pedido ao core podia nao ser o grid desenhado.
Item {
    id: root

    property var metrics: null
    property int pendingCols: 0
    property int pendingRows: 0
    property int lastRequestedCols: 0
    property int lastRequestedRows: 0

    signal resizeRequested(int cols, int rows)

    visible: false

    // Splitters e maximize geram dezenas de eventos de geometria por segundo.
    // Um terminal nativo tambem coalesce: evita serializar um grid completo
    // para cada pixel durante o arrasto.
    Timer {
        id: flush

        interval: 33
        repeat: false
        onTriggered: root.flushPending()
    }

    function measure(width, height) {
        if (!root.metrics || root.metrics.cellWidth <= 0 || root.metrics.cellHeight <= 0
                || width <= 0 || height <= 0) {
            return;
        }
        const cols = Math.max(2, root.metrics.columnsIn(width));
        const rows = Math.max(2, root.metrics.rowsIn(height));
        // Math.floor/Math.max chegam ao cache AOT como double. Comparar pela
        // distancia inteira evita -Wfloat-equal no C++ gerado pelo Qt.
        if (Math.abs(cols - root.lastRequestedCols) < 0.5
                && Math.abs(rows - root.lastRequestedRows) < 0.5) {
            // Voltou ao que o core ja' tem: um tamanho intermediario ainda
            // pendente nao pode sair depois deste.
            root.dropPending();
            return;
        }
        root.pendingCols = cols;
        root.pendingRows = rows;
        if (!flush.running) {
            flush.start();
        }
    }

    function flushPending() {
        if (root.pendingCols <= 0 || root.pendingRows <= 0) {
            return;
        }
        root.lastRequestedCols = root.pendingCols;
        root.lastRequestedRows = root.pendingRows;
        root.dropPending();
        root.resizeRequested(root.lastRequestedCols, root.lastRequestedRows);
    }

    function dropPending() {
        flush.stop();
        root.pendingCols = 0;
        root.pendingRows = 0;
    }

    // Outra sessao: o grid dela no core e' desconhecido; o proximo `measure`
    // pede de novo.
    function reset() {
        root.dropPending();
        root.lastRequestedCols = 0;
        root.lastRequestedRows = 0;
    }
}
