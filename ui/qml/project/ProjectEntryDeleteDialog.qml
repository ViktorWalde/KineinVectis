import QtQuick
import KineinVectis

Item {
    id: root

    property string entryKind: "file"
    property string entryName: ""
    property string errorText: ""
    property bool pending: false
    property real maxAvailableWidth: 380

    signal confirmRequested()
    signal permanentRequested()
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
        border.color: Theme.borderSoft
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
                      ? qsTr("Remover pasta") : qsTr("Remover arquivo")
                color: Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            Text {
                width: parent.width
                text: qsTr("Mover \"%1\" para a lixeira? Você poderá recuperar "
                           + "o item pelo gerenciador de arquivos.").arg(root.entryName)
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

            Text {
                width: parent.width
                visible: root.pending
                text: qsTr("Removendo…")
                color: Theme.textSecondary
                font.pixelSize: 10
            }

            Row {
                anchors.right: parent.right
                spacing: Theme.spacingSmall

                KvButton {
                    text: qsTr("Cancelar")
                    compact: true
                    enabled: !root.pending
                    onClicked: root.cancelRequested()
                }

                KvButton {
                    text: qsTr("Mover para a lixeira")
                    primary: true
                    compact: true
                    enabled: !root.pending
                    onClicked: root.confirmRequested()
                }
            }

            KvButton {
                anchors.right: parent.right
                text: qsTr("Excluir permanentemente")
                compact: true
                danger: true
                tooltip: qsTr("Apaga do disco sem possibilidade de recuperar pela lixeira")
                enabled: !root.pending
                onClicked: root.permanentRequested()
            }
        }
    }
}
