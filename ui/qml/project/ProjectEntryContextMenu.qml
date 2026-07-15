import QtQuick

Item {
    id: root

    property real menuX: 0
    property real menuY: 0
    property bool runnableScript: false

    signal dismissRequested()
    signal createFileRequested()
    signal createDirectoryRequested()
    signal runScriptRequested()
    signal renameRequested()
    signal deleteRequested()

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: root.menuX
        y: root.menuY
        width: 168
        height: entryMenuColumn.height + 2 * Theme.spacingSmall
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderSoft
        border.width: 1

        Column {
            id: entryMenuColumn

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            spacing: 2

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryCreateFileHover.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Adicionar arquivo")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryCreateFileHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.createFileRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryCreateDirectoryHover.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Adicionar pasta")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryCreateDirectoryHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.createDirectoryRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.borderSoft
            }

            Rectangle {
                width: parent.width
                height: root.runnableScript ? 26 : 0
                visible: root.runnableScript
                radius: Theme.radius
                color: entryRunHover.containsMouse
                       ? Theme.surface2 : "transparent"

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "run"
                        size: 14
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: qsTr("Executar script")
                        color: Theme.textPrimary
                        font.pixelSize: 12
                    }
                }

                MouseArea {
                    id: entryRunHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.runScriptRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: root.runnableScript ? 1 : 0
                visible: root.runnableScript
                color: Theme.borderSoft
            }

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryRenameHover.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Renomear")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryRenameHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.renameRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryDeleteHover.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Excluir")
                    color: Theme.errorSoft
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryDeleteHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.deleteRequested()
                }
            }
        }
    }
}
