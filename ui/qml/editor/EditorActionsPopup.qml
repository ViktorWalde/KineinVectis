pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var itemsModel
    property int actionCount: 0
    property int currentIndex: 0
    property real maxAvailableWidth: 420

    signal actionActivated(int index)
    signal dismissRequested()

    width: Math.min(420, maxAvailableWidth)
    height: Math.min(224, Math.max(1, actionCount) * 24 + 2 * Theme.spacingSmall)
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accentDim
    border.width: 1
    clip: true

    Text {
        anchors.centerIn: parent
        visible: root.actionCount === 0
        text: qsTr("Nenhuma ação disponível aqui")
        color: Theme.textMuted
        font.pixelSize: 11
    }

    ListView {
        id: actionsList

        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        visible: root.actionCount > 0
        clip: true
        model: root.itemsModel
        currentIndex: root.currentIndex
        onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)

        delegate: Rectangle {
            id: actionDelegate

            required property int index
            required property string title
            required property string kind

            width: actionsList.width
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
                    width: 64
                    text: actionDelegate.kind
                    color: Theme.accent
                    font.pixelSize: 9
                    elide: Text.ElideRight
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 64 - Theme.spacingSmall
                    text: actionDelegate.title
                    color: Theme.textPrimary
                    font.pixelSize: 12
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.actionActivated(actionDelegate.index)
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        visible: root.actionCount === 0
        onClicked: root.dismissRequested()
    }
}
