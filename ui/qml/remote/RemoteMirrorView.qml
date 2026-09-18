pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O workspace ESPELHADO (P6 fatia 2, 2026-09-18): abrir uma pasta do alvo
// como espelho local, e — quando o workspace aberto E' um espelho — de quem
// ele e' e os dois gestos de sincronia. Burro: recebe, pede por sinal.
Item {
    id: root

    property string openPath: ""
    property var mirror: null
    property bool isMirror: false
    property bool canOpen: false
    property bool syncing: false
    property string syncMessage: ""

    signal openPathEdited(string text)
    signal openFolderRequested()
    signal syncRequested(string direction)

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: parent.width - abrir.width - parent.spacing
                label: qsTr("Pasta no alvo (abre como espelho local)")
                placeholder: "/home/pi/projeto"
                value: root.openPath
                onEdited: text => root.openPathEdited(text)
                onAccepted: root.openFolderRequested()
            }

            KvButton {
                id: abrir

                anchors.bottom: parent.bottom
                text: qsTr("Abrir espelho")
                compact: true
                enabled: root.canOpen && root.openPath.trim() !== ""
                onClicked: root.openFolderRequested()
            }
        }

        Rectangle {
            width: parent.width
            visible: root.isMirror
            height: espelho.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.surface2

            Column {
                id: espelho

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.margins: Theme.spacingSmall
                spacing: Theme.spacingXSmall

                Text {
                    width: parent.width
                    wrapMode: Text.WrapAnywhere
                    text: root.isMirror
                          ? qsTr("Este workspace é um espelho de %1:%2 — salvar empurra o arquivo; "
                                 + "o que mudar no alvo só aparece ao Puxar.").arg(root.mirror.name).arg(root.mirror.path)
                          : ""
                    color: Theme.textSecondary
                    font.pixelSize: 10
                }

                Row {
                    spacing: Theme.spacingSmall

                    KvButton {
                        text: qsTr("Puxar do alvo")
                        compact: true
                        enabled: !root.syncing
                        onClicked: root.syncRequested("pull")
                    }

                    KvButton {
                        text: qsTr("Empurrar tudo")
                        compact: true
                        enabled: !root.syncing
                        onClicked: root.syncRequested("push")
                    }
                }
            }
        }

        Text {
            width: parent.width
            visible: root.syncing || root.syncMessage !== ""
            wrapMode: Text.WordWrap
            text: root.syncing ? qsTr("Sincronizando (rsync)...") : root.syncMessage
            color: Theme.textMuted
            font.pixelSize: 10
        }
    }
}
