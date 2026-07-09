pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var itemsModel
    property int completionCount: 0
    property int currentIndex: 0
    property real maxAvailableWidth: 400

    signal completionActivated(int index)

    width: Math.min(400, maxAvailableWidth)
    height: Math.min(224, completionCount * 24 + 2 * Theme.spacingSmall)
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accentDim
    border.width: 1
    clip: true

    ListView {
        id: completionList

        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        clip: true
        model: root.itemsModel
        currentIndex: root.currentIndex
        onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)

        delegate: Rectangle {
            id: completionDelegate

            required property int index
            required property string label
            required property string detail
            required property string kind

            width: completionList.width
            height: 24
            radius: Theme.radius
            color: index === root.currentIndex ? Theme.surface2 : "transparent"

            Row {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 52
                    text: completionDelegate.kind
                    color: Theme.accent
                    font.pixelSize: 9
                    elide: Text.ElideRight
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: completionDelegate.label
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - x
                    text: completionDelegate.detail
                    color: Theme.textMuted
                    font.pixelSize: 10
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.completionActivated(completionDelegate.index)
            }
        }
    }
}
