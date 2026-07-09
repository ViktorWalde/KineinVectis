pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var itemsModel
    property int usageCount: 0
    property real maxAvailableWidth: 420

    signal closeRequested()
    signal usageOpenRequested(string path, int line, int column)

    width: Math.min(420, maxAvailableWidth)
    height: Math.min(260, usageCount * 22 + 34 + 2 * Theme.spacingSmall)
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accentDim
    border.width: 1
    clip: true

    Row {
        id: usagesHeader

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: 20

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Usos (%1)").arg(root.usageCount)
            color: Theme.textPrimary
            font.pixelSize: 11
            font.bold: true
        }

        Item {
            width: parent.width - x - usagesClose.width
            height: 1
        }

        Rectangle {
            id: usagesClose

            anchors.verticalCenter: parent.verticalCenter
            width: 18
            height: 18
            radius: Theme.radius
            color: usagesCloseArea.containsMouse ? Theme.surface2 : "transparent"

            Text {
                anchors.centerIn: parent
                text: "×"
                color: Theme.textSecondary
                font.pixelSize: 12
            }

            MouseArea {
                id: usagesCloseArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.closeRequested()
            }
        }
    }

    ListView {
        id: usagesList

        anchors.top: usagesHeader.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        anchors.topMargin: 2
        clip: true
        model: root.itemsModel

        delegate: Rectangle {
            id: usageDelegate

            required property string path
            required property int line
            required property int column
            required property string display

            width: usagesList.width
            height: 22
            radius: Theme.radius
            color: usageArea.containsMouse ? Theme.surface2 : "transparent"

            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                text: usageDelegate.display
                color: Theme.accent
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }

            MouseArea {
                id: usageArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.usageOpenRequested(usageDelegate.path,
                                                   usageDelegate.line,
                                                   usageDelegate.column)
            }
        }
    }
}
