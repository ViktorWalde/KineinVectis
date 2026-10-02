import QtQuick
import KineinVectis

// Um lugar da barra de status (0.3.9): carrega a peca da chave e a deixa
// ARRASTAR para outro lugar da mesma faixa. O arrasto e' um DragHandler: ele
// so' toma o gesto depois do limiar, e ate' la' o clique segue para a peca
// (o IDE, o remoto, o ✕ do job, a dica do LSP continuam funcionando).
Loader {
    id: root

    required property string modelData
    property var parts: ({})
    property ReorderController reorder: null
    readonly property string reorderKey: root.modelData

    anchors.verticalCenter: parent.verticalCenter
    sourceComponent: root.parts[root.modelData]
    visible: root.item !== null && root.item.shown === true
    opacity: root.reorder !== null ? root.reorder.opacityFor(root.modelData) : 1.0

    DragHandler {
        target: null
        acceptedButtons: Qt.LeftButton
        onActiveChanged: {
            if (active) {
                root.reorder.begin(root.reorderKey);
            } else if (root.reorder.active) {
                root.reorder.finish();
            }
        }
        onCentroidChanged: {
            if (!active) return;
            const pos = root.mapToItem(root.reorder.container, centroid.position.x, centroid.position.y);
            root.reorder.update(pos.x, pos.y);
        }
    }
}
