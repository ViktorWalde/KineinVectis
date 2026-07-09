import QtQuick

Item {
    id: root

    property string entryKind: "file"
    property string entryName: ""
    property string errorText: ""
    property real maxAvailableWidth: 380

    signal confirmRequested()
    signal cancelRequested()

    MouseArea {
        anchors.fill: parent
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(380, root.maxAvailableWidth)
        height: entryDeleteColumn.height + 2 * Theme.spacingMedium
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.errorSoft
        border.width: 1

        Column {
            id: entryDeleteColumn

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Text {
                text: root.entryKind === "directory"
                      ? qsTr("Excluir pasta") : qsTr("Excluir arquivo")
                color: Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            Text {
                width: parent.width
                text: root.entryKind === "directory"
                      ? qsTr("A pasta \"%1\" e todo o seu conteudo serao removidos "
                             + "do disco. Esta acao nao pode ser desfeita.")
                        .arg(root.entryName)
                      : qsTr("O arquivo \"%1\" sera removido do disco. "
                             + "Esta acao nao pode ser desfeita.")
                        .arg(root.entryName)
                color: Theme.textSecondary
                font.pixelSize: 11
                wrapMode: Text.WordWrap
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: 10
                wrapMode: Text.WordWrap
            }

            Row {
                anchors.right: parent.right
                spacing: Theme.spacingSmall

                Rectangle {
                    width: entryDeleteCancelText.width + 2 * Theme.spacingMedium
                    height: 24
                    radius: Theme.radius
                    color: entryDeleteCancelArea.containsMouse
                           ? Theme.surface2 : Theme.surface1
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: entryDeleteCancelText

                        anchors.centerIn: parent
                        text: qsTr("Cancelar")
                        color: Theme.textSecondary
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: entryDeleteCancelArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.cancelRequested()
                    }
                }

                Rectangle {
                    width: entryDeleteConfirmText.width + 2 * Theme.spacingMedium
                    height: 24
                    radius: Theme.radius
                    color: entryDeleteConfirmArea.pressed
                           ? Qt.darker(Theme.errorSoft, 1.4) : Theme.errorSoft

                    Text {
                        id: entryDeleteConfirmText

                        anchors.centerIn: parent
                        text: qsTr("Excluir")
                        color: Theme.background0
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: entryDeleteConfirmArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.confirmRequested()
                    }
                }
            }
        }
    }
}
