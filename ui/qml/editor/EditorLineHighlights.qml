import QtQuick
import KineinVectis

// C4: as faixas horizontais que marcam LINHA dentro do texto.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Sao duas marcas com regra de
// precedencia entre si — a linha do cursor SOME durante selecao, e a linha de
// execucao do debugger, mais forte, desenha por cima — que estavam soltas no
// meio do Flickable do EditorTextSurface, entre a rolagem e o TextEdit. Juntas
// elas formam uma area visual propria: "onde estou" e "onde o programa parou".
//
// Este componente nao sabe rolar, nao sabe texto e nao sabe sarjeta: recebe as
// coordenadas ja calculadas e pinta. O POSICIONAMENTO fica no pai, como no
// corte do GitPanel (docs/roadmaps/38 §2.2).
Item {
    id: root

    // false enquanto nao ha arquivo aberto: nenhuma faixa aparece.
    property bool active: false
    property real cursorY: 0
    property real cursorHeight: 0
    property bool hasSelection: false
    // 0 = o debugger nao esta parado neste arquivo.
    property int executionLine: 0
    property real lineHeight: 1

    Rectangle {
        visible: root.active && !root.hasSelection
        y: root.cursorY
        height: root.cursorHeight
        width: root.width
        color: Theme.surfaceSelected
    }

    Rectangle {
        visible: root.executionLine > 0
        y: (root.executionLine - 1) * root.lineHeight
        width: root.width
        height: root.lineHeight
        color: Theme.accentDim
        opacity: 0.35
    }
}
