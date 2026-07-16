import QtQuick
import KineinVectis

// Alca de redimensionamento entre regioes (KVSplitter minimo da fatia C1 de
// docs/roadmaps/20). Emite deltas incrementais: como a alca acompanha a borda que
// move, o offset em relacao ao ponto de press e o delta a aplicar.
MouseArea {
    id: root

    property bool horizontal: false

    signal dragged(real delta)
    signal resetRequested()

    property real pressOrigin: 0

    cursorShape: horizontal ? Qt.SplitVCursor : Qt.SplitHCursor
    hoverEnabled: true
    preventStealing: true

    onPressed: function(mouse) {
        pressOrigin = horizontal ? mouse.y : mouse.x;
    }
    onPositionChanged: function(mouse) {
        if (!pressed) {
            return;
        }
        const position = horizontal ? mouse.y : mouse.x;
        root.dragged(position - pressOrigin);
    }
    onDoubleClicked: root.resetRequested()

    Rectangle {
        anchors.centerIn: parent
        width: root.horizontal ? parent.width : 2
        height: root.horizontal ? 2 : parent.height
        color: root.containsMouse || root.pressed ? Theme.accent : "transparent"
    }
}
