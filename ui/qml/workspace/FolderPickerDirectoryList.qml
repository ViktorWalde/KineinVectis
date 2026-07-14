pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var controller

    radius: Theme.radius
    color: Theme.background0
    border.color: Theme.borderSoft
    border.width: 1

    Text {
        anchors.centerIn: parent
        visible: root.controller.loading
        text: qsTr("Carregando...")
        color: Theme.textMuted
        font.pixelSize: 12
    }

    Text {
        anchors.centerIn: parent
        visible: !root.controller.loading && root.controller.entriesModel.count === 0
        text: qsTr("Nenhum subdiretorio")
        color: Theme.textMuted
        font.pixelSize: 12
    }

    ListView {
        id: folderList

        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        visible: !root.controller.loading && root.controller.entriesModel.count > 0
        clip: true
        model: root.controller.entriesModel

        delegate: Rectangle {
            id: folderRow

            required property string name
            required property string path

            width: folderList.width
            height: 28
            radius: Theme.radius
            color: root.controller.selectedPath === folderRow.path
                   ? Theme.surfaceSelected
                   : (folderMouse.containsMouse ? Theme.surface2 : "transparent")

            Row {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 16
                    text: ">"
                    color: Theme.accent
                    font.pixelSize: 12
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 20
                    text: folderRow.name
                    color: root.controller.selectedPath === folderRow.path
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 12
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                id: folderMouse

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.controller.selectedPath = folderRow.path
                onDoubleClicked: root.controller.browsePath(folderRow.path)
            }
        }
    }
}
