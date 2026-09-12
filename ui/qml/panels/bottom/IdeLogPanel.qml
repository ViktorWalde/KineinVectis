pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel


    // B2 (DocsPublic/roadmaps/24): barra de rolagem. `parent: panel` é OBRIGATÓRIO — um filho
    // declarado dentro de um ListView vira filho do contentItem e ROLARIA
    // junto com a lista. O ListView segue sendo a fonte da verdade.
    VerticalScrollBar {
        id: scrollBar_panel

        parent: panel
        anchors.right: panel.right
        anchors.top: panel.top
        anchors.bottom: panel.bottom

        contentSize: panel.contentHeight
        viewportSize: panel.height
        position: panel.contentY

        onMoveRequested: function(position) {
            panel.contentY = position;
        }
    }
    property var logLinesModel: []

    clip: true
    model: logLinesModel
    onCountChanged: positionViewAtEnd()

    delegate: Text {
        required property string modelData

        width: panel.width
        text: modelData
        color: Theme.textSecondary
        font.family: Theme.monoFont
        font.pixelSize: 11
        wrapMode: Text.WrapAnywhere
    }
}
