pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel

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
        font.pixelSize: 11
        wrapMode: Text.WrapAnywhere
    }
}
