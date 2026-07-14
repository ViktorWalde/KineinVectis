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

    height: fileCount > 0 ? 36 : 0
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

                width: tabLabel.width + closeButton.width
                       + 3 * Theme.spacingSmall
                height: 30
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
                    text: tabDelegate.name
                    color: tabDelegate.index === root.currentIndex
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 12
                }

                Rectangle {
                    anchors.left: tabLabel.left
                    anchors.leftMargin: -Theme.spacingSmall + 2
                    anchors.verticalCenter: parent.verticalCenter
                    width: 4
                    height: 4
                    radius: 2
                    visible: tabDelegate.modified
                    color: Theme.accent
                }

                KvIconButton {
                    id: closeButton

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.right: parent.right
                    anchors.rightMargin: 2
                    width: 24
                    height: 24
                    iconName: "close"
                    iconSize: 13
                    danger: true
                    tooltip: qsTr("Fechar aba")
                    onClicked: root.tabCloseRequested(tabDelegate.index)
                }

                MouseArea {
                    anchors.fill: parent
                    anchors.rightMargin: closeButton.width + Theme.spacingSmall
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.tabSelected(tabDelegate.index)
                }
            }
        }
    }

    KvButton {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        height: 28
        visible: root.currentIndex >= 0
        compact: true
        primary: true
        text: qsTr("Salvar")
        iconName: "file"
        onClicked: root.saveRequested()
    }
}
