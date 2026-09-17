pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Gravar como configuracao de execucao (E4): o motor, a PREVIA da linha com
// as evidencias e os avisos, e os dois gestos — rodar agora ou salvar como
// configuracao. Dono proprio pela mesma regra das outras views do painel.
//
// Burro: le de `controller.flash`. A porta e' a escolhida no painel
// (`controller.selectedPort`); a flash lida pela identidade so' entra na
// checagem quando foi lida DESTA porta.
Item {
    id: root

    property var controller: null
    readonly property var flash: controller ? controller.flash : null

    implicitHeight: coluna.implicitHeight

    function flashBytesDaPorta() {
        if (!root.controller || !root.controller.identity.found) return 0;
        if (root.controller.identity.device !== root.controller.selectedPort) return 0;
        const bytes = root.controller.identity.identity.flashSizeBytes;
        return bytes === undefined ? 0 : bytes;
    }

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Gravar")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            // O motor: vazio = o que o modelo do projeto sugere.
            Repeater {
                model: root.flash ? root.flash.engines : []

                KvToggleChip {
                    required property string modelData

                    anchors.verticalCenter: parent.verticalCenter
                    labelText: modelData
                    active: root.flash && root.flash.engine === modelData
                    tooltip: qsTr("Usar %1 como motor; solto, o modelo do projeto decide").arg(modelData)
                    onToggled: root.flash.selectEngine(modelData)
                }
            }

            KvButton {
                anchors.verticalCenter: parent.verticalCenter
                text: root.flash && root.flash.busy ? qsTr("montando…") : qsTr("Prévia")
                compact: true
                enabled: root.flash !== null && !root.flash.busy
                onClicked: root.flash.propose(root.controller.selectedPort, root.flashBytesDaPorta())
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.flash && !root.flash.found && root.flash.errorText === ""
            text: qsTr("a linha do motor vem do modelo do projeto (receita do build, ELF/UF2) e da porta escolhida; nada roda sem o seu clique")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        // A linha, como vai rodar — editavel depois de salva, como qualquer
        // configuracao de execucao.
        Text {
            width: parent.width
            wrapMode: Text.WrapAnywhere
            visible: root.flash && root.flash.found
            text: root.flash && root.flash.found ? root.flash.proposal.command : ""
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
        }

        Repeater {
            model: root.flash ? root.flash.sourceLines() : []

            Text {
                required property string modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                text: "· " + modelData
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }

        Repeater {
            model: root.flash ? root.flash.warningLines() : []

            Text {
                required property string modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                text: qsTr("aviso: %1").arg(modelData)
                color: Theme.warningSoft
                font.pixelSize: 10
            }
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.flash && root.flash.found

            KvButton {
                text: qsTr("Gravar agora")
                compact: true
                primary: true
                onClicked: root.flash.run()
            }

            KvButton {
                text: root.flash && root.flash.found
                      ? qsTr("Salvar como \"%1\"").arg(root.flash.proposal.name) : ""
                compact: true
                onClicked: root.flash.save()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("salva, vira a configuração ativa: o botão Executar grava")
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.flash && root.flash.errorText !== ""
            text: root.flash ? root.flash.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }
    }
}
