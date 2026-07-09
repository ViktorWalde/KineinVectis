pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Item {
    id: root

    property var filesModel
    property int fileCount: 0
    property int currentIndex: -1

    signal tabSelected(int index)
    signal tabCloseRequested(int index)
    signal saveRequested()

    height: fileCount > 0 ? 30 : 0
    visible: fileCount > 0

    Row {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingSmall

        Repeater {
            model: root.filesModel

            delegate: Rectangle {
                id: tabDelegate

                required property int index
                required property string name
                required property bool modified

                width: tabLabel.width + closeLabel.width
                       + 3 * Theme.spacingSmall
                height: 26
                radius: Theme.radius
                color: index === root.currentIndex
                       ? Theme.surface2 : Theme.surface1
                border.color: index === root.currentIndex
                              ? Theme.accent : Theme.borderSoft
                border.width: 1

                Text {
                    id: tabLabel

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: (tabDelegate.modified ? "● " : "") + tabDelegate.name
                    color: tabDelegate.index === root.currentIndex
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 12
                }

                Text {
                    id: closeLabel

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingSmall
                    text: "×"
                    color: closeArea.containsMouse
                           ? Theme.errorSoft : Theme.textMuted
                    font.pixelSize: 14

                    MouseArea {
                        id: closeArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.tabCloseRequested(tabDelegate.index)
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    anchors.rightMargin: closeLabel.width + Theme.spacingSmall
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tabSelected(tabDelegate.index)
                }
            }
        }
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        width: saveText.width + 2 * Theme.spacingMedium
        height: 24
        radius: Theme.radius
        visible: root.currentIndex >= 0
        color: saveArea.pressed ? Theme.accentDim : Theme.accent

        Text {
            id: saveText

            anchors.centerIn: parent
            text: qsTr("Salvar")
            color: Theme.background0
            font.pixelSize: 11
            font.bold: true
        }

        MouseArea {
            id: saveArea

            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.saveRequested()
        }
    }
}
