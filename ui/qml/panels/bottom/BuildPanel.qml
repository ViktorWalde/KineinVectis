pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel

    FlickableScrollBar {
        id: scrollBar_panel

        view: panel
    }

    ListModel {
        id: emptyOutputModel
    }

    property var outputModel: emptyOutputModel

    clip: true
    model: outputModel
    onCountChanged: positionViewAtEnd()

    delegate: Text {
        required property string line

        width: panel.width
        text: line
        color: Theme.textSecondary
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WrapAnywhere
    }
}
