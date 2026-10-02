import QtQuick
import KineinVectis

// A area de clique de um item que se pode ARRASTAR para reordenar (0.3.9).
// Clique continua clique (`tapped`, com o botao): o arrasto so' comeca
// depois de 6 px com o botao esquerdo, e entao a barra nao rola nem recebe o
// clique de soltura. Esc cancela.
MouseArea {
    id: root

    property ReorderController reorder: null
    property string reorderKey: ""

    signal tapped(var mouse)

    property point pressPoint: Qt.point(0, 0)
    property bool dragging: false
    // O clique que o Qt emite depois de soltar um arrasto nao e' clique.
    property bool swallowClick: false

    acceptedButtons: Qt.LeftButton | Qt.RightButton
    hoverEnabled: true
    preventStealing: root.dragging
    cursorShape: root.dragging ? Qt.ClosedHandCursor : Qt.PointingHandCursor

    onPressed: function(mouse) {
        root.pressPoint = Qt.point(mouse.x, mouse.y);
        root.dragging = false;
        root.swallowClick = false;
    }

    onPositionChanged: function(mouse) {
        if (!(mouse.buttons & Qt.LeftButton) || root.reorder === null || root.reorderKey === "") {
            return;
        }
        if (!root.dragging) {
            const dx = mouse.x - root.pressPoint.x;
            const dy = mouse.y - root.pressPoint.y;
            if (Math.abs(dx) < 6 && Math.abs(dy) < 6) return;
            root.dragging = true;
            root.reorder.begin(root.reorderKey);
        }
        const pos = root.mapToItem(root.reorder.container, mouse.x, mouse.y);
        root.reorder.update(pos.x, pos.y);
    }

    onReleased: function(mouse) {
        if (root.dragging) {
            root.dragging = false;
            root.swallowClick = true;
            root.reorder.finish();
        }
    }

    onCanceled: {
        if (root.dragging) {
            root.dragging = false;
            root.reorder.cancel();
        }
    }

    onClicked: function(mouse) {
        if (root.swallowClick) {
            root.swallowClick = false;
            return;
        }
        root.tapped(mouse);
    }

    Keys.onEscapePressed: function(event) {
        if (root.dragging) {
            root.dragging = false;
            root.reorder.cancel();
            event.accepted = true;
        }
    }
}
