import QtQuick
import KineinVectis

// A barra de rolagem de UMA lista (ou outro Flickable): pendura-se nela, a'
// direita, e liga as quatro pontas — tamanho, janela, posicao e o pedido de
// mover. Antes, cada lista repetia as nove linhas desta ligacao (pente fino
// da 0.3.9).
//
// No PAI certo: `parent` e' a propria lista, nao o contentItem dela — um
// filho declarado dentro de um ListView vira filho do contentItem e ROLARIA
// junto com a lista. A lista segue sendo a fonte da verdade da posicao.
VerticalScrollBar {
    id: root

    property Flickable view: null

    parent: root.view
    anchors.right: root.view !== null ? root.view.right : undefined
    anchors.top: root.view !== null ? root.view.top : undefined
    anchors.bottom: root.view !== null ? root.view.bottom : undefined
    contentSize: root.view !== null ? root.view.contentHeight : 0
    viewportSize: root.view !== null ? root.view.height : 0
    position: root.view !== null ? root.view.contentY : 0

    onMoveRequested: function(position) {
        if (root.view !== null) root.view.contentY = position;
    }
}
