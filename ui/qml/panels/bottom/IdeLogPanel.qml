pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel

    FlickableScrollBar {
        id: scrollBar_panel

        view: panel
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
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WrapAnywhere
    }
}
