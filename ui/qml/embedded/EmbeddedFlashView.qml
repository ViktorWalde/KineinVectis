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
    // Os firmwares BAIXADOS (C5): o painel os filtra da lista de instalaveis
    // do ToolchainController — [{ id, label, version, firmware: {...} }].
    property var firmwares: []
    property bool enginesVisible: false

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
        spacing: Theme.spacingMedium

        Text {
            width: parent.width
            text: root.controller && root.controller.selectedPort !== ""
                  ? qsTr("Porta: %1").arg(root.controller.selectedPort)
                  : qsTr("Nenhuma porta selecionada. Escolha uma na aba Placa se o motor usar conexão serial.")
            wrapMode: Text.WrapAnywhere
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        Text {
            width: parent.width
            text: qsTr("Ferramenta de gravação")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
            font.bold: true
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall
            Text {
                text: root.flash && root.flash.engine !== "" ? root.flash.engine : qsTr("Automática · escolhida pelo projeto")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeSmall
            }
            KvButton {
                text: root.enginesVisible ? qsTr("Ocultar opções") : qsTr("Ferramenta e modo")
                compact: true
                onClicked: root.enginesVisible = !root.enginesVisible
            }
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.enginesVisible

            KvToggleChip {
                labelText: qsTr("Automática")
                codeFont: false
                height: 28
                active: root.flash && root.flash.engine === ""
                onToggled: root.flash.selectEngine("")
            }

            // O motor: vazio = o que o modelo do projeto sugere.
            Repeater {
                model: root.flash ? root.flash.allEngines : []

                KvToggleChip {
                    required property string modelData

                    labelText: modelData
                    codeFont: false
                    height: 28
                    active: root.flash && root.flash.engine === modelData
                    tooltip: qsTr("Usar %1 como motor; solto, o modelo do projeto decide").arg(modelData)
                    onToggled: root.flash.selectEngine(modelData)
                }
            }

        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.enginesVisible
            KvToggleChip {
                labelText: qsTr("Nativo")
                active: root.flash && root.flash.nativeMode
                onToggled: root.flash.selectMode(true)
            }
            KvToggleChip {
                labelText: qsTr("Comando manual")
                active: root.flash && !root.flash.nativeMode
                onToggled: root.flash.selectMode(false)
            }
        }

        KvTextField {
            objectName: "embeddedFlashBuildDirectory"
            width: parent.width
            visible: root.enginesVisible && root.flash && root.flash.nativeMode
            label: qsTr("Diretório do build (opcional)")
            placeholder: qsTr("Padrão do projeto")
            text: root.flash ? root.flash.buildDirectory : ""
            onEdited: (value) => root.flash.selectBuildDirectory(value)
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.flash && root.flash.nativeMode
                  ? qsTr("Modo nativo: vincula porta, ferramenta e arquivos à prévia e confere novamente antes de executar.")
                  : qsTr("Comando manual: executa shell, sem reserva serial nem revalidação nativa de destino e arquivos.")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        KvButton {
            objectName: "embeddedReviewCommand"
            text: root.flash && root.flash.busy ? qsTr("Preparando…")
                : root.flash && root.flash.found ? qsTr("Atualizar prévia") : qsTr("Gerar prévia de gravação")
            primary: root.flash !== null && !root.flash.found
            compact: true
            enabled: root.flash !== null && !root.flash.busy
                     && !root.controller.projectBusy && !root.controller.portsBusy
            onClicked: root.flash.propose(root.controller.selectedPort, root.flashBytesDaPorta())
        }

        // Um firmware baixado no lugar dos artefatos do build (C5): a linha
        // e' a da pagina da placa. Solto, grava-se o build do projeto.
        Flow {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.firmwares.length > 0

            Text {
                text: qsTr("Firmware baixado:")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
            }

            Repeater {
                model: root.firmwares

                KvToggleChip {
                    required property var modelData

                    labelText: modelData.firmware !== undefined ? modelData.firmware.board + " " + modelData.version : modelData.id
                    active: root.flash && root.flash.firmware === modelData.id
                    tooltip: qsTr("Gravar %1 (%2) no lugar do build; solto, grava o build do projeto").arg(modelData.label).arg(modelData.firmware !== undefined ? modelData.firmware.engine : "")
                    onToggled: root.flash.selectFirmware(modelData.id)
                }
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.flash && !root.flash.found && root.flash.errorText === ""
            text: qsTr("A prévia mostra o comando e os arquivos que serão usados. Gerar a prévia não grava nada na placa.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvInsetSurface {
            width: parent.width
            implicitHeight: commandPreview.implicitHeight + 2 * Theme.spacingMedium
            visible: root.flash && root.flash.found
            TextEdit {
                id: commandPreview
                objectName: "embeddedFlashCommand"
                x: Theme.spacingMedium
                y: Theme.spacingMedium
                width: parent.width - 2 * Theme.spacingMedium
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
                text: root.flash && root.flash.found ? root.flash.proposal.command : ""
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeSmall
            }
        }

        Repeater {
            model: root.flash ? root.flash.sourceLines() : []

            Text {
                required property string modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                text: "· " + modelData
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
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
                font.pixelSize: Theme.fontSizeCaption
            }
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.flash && root.flash.found

            KvButton {
                text: qsTr("Gravar na placa")
                compact: true
                primary: true
                onClicked: root.flash.run()
            }

            KvButton {
                text: qsTr("Salvar e ativar")
                tooltip: root.flash && root.flash.found ? root.flash.proposal.name : ""
                compact: true
                onClicked: root.flash.save()
            }

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                text: qsTr("Gravar substitui o firmware da placa. Salvar e ativar também passa a usar esta gravação no botão Executar.")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
            }
        }

        KvVerdict {
            objectName: "flashFailure"
            width: parent.width
            heading: qsTr("Não foi possível preparar a gravação")
            message: root.flash ? root.flash.errorText : ""
            codeFont: false
            animateChanges: true
        }
    }
}
