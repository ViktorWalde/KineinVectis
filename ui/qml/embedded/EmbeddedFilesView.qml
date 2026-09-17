pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os arquivos na placa (C2 do roadmaps/41 bloco C): a lista que o mpremote
// leu, os gestos por entrada e a confirmacao do que escreve. Dono proprio
// pela mesma regra do EmbeddedSerialView: o painel esta' no limite.
//
// Burro: le do controller filho (`controller.files`). O botao que PEDE a
// primeira lista esta' na linha da porta (EmbeddedSerialView, "Arquivos");
// aqui e' o resultado — vazio ate' o primeiro clique, porque cada `fs`
// interrompe o programa da placa.
Item {
    id: root

    property var controller: null
    readonly property var files: controller ? controller.files : null

    implicitHeight: coluna.implicitHeight
    visible: files !== null && files.device !== ""

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                width: parent.width - botoes.width - Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                elide: Text.ElideMiddle
                text: root.files ? qsTr("Arquivos na placa — %1  :/%2").arg(root.files.device).arg(root.files.path) : ""
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            Row {
                id: botoes

                spacing: Theme.spacingSmall

                KvButton {
                    text: qsTr("Acima")
                    compact: true
                    visible: root.files && root.files.path !== ""
                    enabled: root.files && !root.files.busy
                    onClicked: root.files.up()
                }

                KvButton {
                    text: qsTr("Atualizar")
                    compact: true
                    enabled: root.files && !root.files.busy
                    onClicked: root.files.refresh()
                }

                KvButton {
                    text: qsTr("Enviar arquivo aberto")
                    compact: true
                    enabled: root.files && !root.files.busy && !root.files.hasPending
                    onClicked: root.files.uploadCurrentRequested()
                }
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.files && root.files.busy
            text: qsTr("falando com o mpremote… (o programa da placa e' interrompido)")
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        // A confirmacao do que ESCREVE: um segundo clique, nunca um so'.
        Rectangle {
            width: parent.width
            height: confirmacao.implicitHeight + 2 * Theme.spacingSmall
            visible: root.files && root.files.hasPending
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: Theme.warningSoft

            Row {
                id: confirmacao

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingSmall

                Text {
                    width: parent.width - sim.width - nao.width - 2 * Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    wrapMode: Text.WordWrap
                    text: root.files ? root.files.pendingText() : ""
                    color: Theme.textPrimary
                    font.pixelSize: 11
                }

                KvButton {
                    id: sim

                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Sim, escrever")
                    compact: true
                    onClicked: root.files.confirmPending()
                }

                KvButton {
                    id: nao

                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Não")
                    compact: true
                    onClicked: root.files.cancelPending()
                }
            }
        }

        Text {
            width: parent.width
            visible: root.files && root.files.listed && root.files.entries.length === 0 && !root.files.busy
            text: qsTr("pasta vazia")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        Repeater {
            model: root.files ? root.files.entries : []

            delegate: Row {
                id: linha

                required property var modelData

                width: coluna.width
                spacing: Theme.spacingSmall

                Text {
                    width: parent.width - acoes.width - Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideMiddle
                    text: root.files ? root.files.entrySummary(linha.modelData) : ""
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }

                Row {
                    id: acoes

                    spacing: Theme.spacingSmall

                    KvButton {
                        text: qsTr("Abrir")
                        compact: true
                        visible: linha.modelData.directory === true
                        enabled: root.files && !root.files.busy
                        onClicked: root.files.enterDir(linha.modelData.name)
                    }

                    KvButton {
                        text: qsTr("Baixar")
                        compact: true
                        visible: linha.modelData.directory !== true
                        enabled: root.files && !root.files.busy
                        onClicked: root.files.download(linha.modelData.name)
                    }

                    KvButton {
                        text: qsTr("Apagar")
                        compact: true
                        visible: linha.modelData.directory !== true
                        enabled: root.files && !root.files.busy && !root.files.hasPending
                        onClicked: root.files.remove(linha.modelData.name)
                    }
                }
            }
        }

        // A recusa ou a falha, com a linha do mpremote.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.files && root.files.errorText !== ""
            text: root.files ? root.files.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }
    }
}
