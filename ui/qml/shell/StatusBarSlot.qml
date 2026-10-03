import QtQuick
import KineinVectis

// Um lugar da barra de status (0.3.9): cria a peca da chave e a deixa
// ARRASTAR para outro lugar da mesma faixa. A peca nasce por createObject com
// `bar` e `strip` ja' entregues (um Loader criaria antes e as leituras dariam
// TypeError). O arrasto e' um DragHandler: so' toma o gesto depois do limiar,
// e ate' la' o clique segue para a peca (o IDE, o remoto, o ✕ do job, a dica
// do LSP continuam funcionando).
Item {
    id: root

    required property string modelData
    property var parts: ({})
    property ReorderController reorder: null
    // `statusBar`, nao `bar`: no Qt 6.4 `bar: bar` no delegado lia a propria
    // propriedade (o nome tapava o id da barra) e virava laco de binding.
    property var statusBar: null
    property Item strip: null
    readonly property string reorderKey: root.modelData
    property var part: null

    anchors.verticalCenter: parent.verticalCenter
    width: root.part !== null ? root.part.width : 0
    height: root.part !== null ? root.part.height : 0
    visible: root.part !== null && root.part.shown === true
    opacity: root.reorder !== null ? root.reorder.opacityFor(root.modelData) : 1.0

    Component.onCompleted: {
        root.part = root.parts[root.modelData].createObject(root, { bar: root.statusBar, strip: root.strip });
    }
    onXChanged: if (root.part !== null && root.part.slotX !== undefined) root.part.slotX = root.x

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
