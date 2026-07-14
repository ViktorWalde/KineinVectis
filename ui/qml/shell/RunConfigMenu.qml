pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Dropdown do seletor de run configs da Main Toolbar. Vive nos overlays
// (nao dentro do header) porque o header tem 44px e os paineis desenhados
// depois dele cobririam o menu — mesmo racional do ProjectEntryContextMenu.
Item {
    id: root

    property real menuX: 0
    property real menuY: 0
    property var configsModel
    property string activeConfigId: ""

    signal dismissRequested()
    signal configChosen(string id)
    signal newRequested()
    signal editRequested()
    signal deleteRequested()

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: Math.min(root.menuX, root.width - width - Theme.spacingMedium)
        y: root.menuY
        width: 240
        height: configMenuColumn.height + 2 * Theme.spacingSmall
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderSoft
        border.width: 1

        Column {
            id: configMenuColumn

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall

            Rectangle {
                width: parent.width
                height: 24
                radius: Theme.radiusXSmall
                color: automaticArea.containsMouse ? Theme.surface2
                       : (root.activeConfigId === ""
                          ? Theme.surfaceSelected : "transparent")

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Automático")
                    color: root.activeConfigId === ""
                           ? Theme.accent : Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: automaticArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.configChosen("")
                }
            }

            Repeater {
                model: root.configsModel

                delegate: Rectangle {
                    id: configEntry

                    required property string id
                    required property string name

                    width: configMenuColumn.width
                    height: 24
                    radius: Theme.radiusXSmall
                    color: configEntryArea.containsMouse ? Theme.surface2
                           : (root.activeConfigId === configEntry.id
                              ? Theme.surfaceSelected : "transparent")

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.spacingSmall
                        text: configEntry.name
                        color: root.activeConfigId === configEntry.id
                               ? Theme.accent : Theme.textPrimary
                        font.pixelSize: 12
                        elide: Text.ElideRight
                    }

                    MouseArea {
                        id: configEntryArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.configChosen(configEntry.id)
                    }
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.borderSoft
            }

            Rectangle {
                width: parent.width
                height: 24
                radius: Theme.radiusXSmall
                color: newConfigArea.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Nova configuração...")
                    color: Theme.textSecondary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: newConfigArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.newRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 24
                radius: Theme.radiusXSmall
                visible: root.activeConfigId !== ""
                color: editConfigArea.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Editar atual...")
                    color: Theme.textSecondary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: editConfigArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.editRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 24
                radius: Theme.radiusXSmall
                visible: root.activeConfigId !== ""
                color: deleteConfigArea.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Excluir atual")
                    color: Theme.errorSoft
                    font.pixelSize: 12
                }

                MouseArea {
                    id: deleteConfigArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.deleteRequested()
                }
            }
        }
    }
}
