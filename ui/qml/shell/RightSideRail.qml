import QtQuick
import KineinVectis

// O trilho da DIREITA (0.3.9, pedido do autor: "que o trilho da esquerda e
// da direita sejam usaveis e que o usuario possa organizar"): as areas que o
// usuario arrastou para este lado (ou que nascem nele, como os Simbolos). E'
// parceiro de arrasto do trilho da esquerda: um icone solto aqui passa para
// a direita, e daqui para la' volta para a esquerda.
SideRail {
    id: root

    property var shellController: null
    property var railEntries: null

    primary: false
    entries: root.railEntries !== null ? root.railEntries.rightEntries : []
    order: root.shellController !== null ? root.shellController.savedOrder("railRight") : []
    onEntryMoved: (id, i, ids) => root.shellController.moveInBar("railRight", ids, id, i)
    onEntryTransferred: (id, i, ids) => root.shellController.moveRailEntryToSide(id, "left", i, ids)
    onActivated: id => root.railEntries.activate(id)
}
