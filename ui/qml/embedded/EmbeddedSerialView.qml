pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Conexão e ações por dispositivo; as medidas e as operações pertencem ao core.
Item {
    id: root
    property var controller: null
    implicitHeight: content.implicitHeight

    Column {
        id: content
        width: parent.width
        spacing: Theme.spacingMedium

        Item {
            width: parent.width
            height: permissions.implicitHeight
            Text {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Conexões seriais")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.DemiBold
            }
            KvButton {
                id: permissions
                anchors.right: parent.right
                text: root.controller && root.controller.access.busy ? qsTr("Verificando…") : qsTr("Permissões")
                compact: true
                enabled: root.controller !== null && !root.controller.access.busy
                onClicked: root.controller.access.diagnose("")
            }
        }

        Text {
            width: parent.width
            text: qsTr("Selecione a porta para gravar. Identificar reinicia a placa; o monitor abre em uma aba de terminal.")
            wrapMode: Text.WordWrap
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeBody
        }

        Repeater {
            model: root.controller ? root.controller.ports : []
            Rectangle {
                id: portCard
                required property var modelData
                readonly property bool selected: root.controller.selectedPort === modelData.device
                width: content.width
                implicitHeight: portContent.implicitHeight + 2 * Theme.spacingMedium
                radius: Theme.radiusLarge
                color: Theme.surface1
                border.width: 1
                border.color: selected ? Theme.accentDim : Theme.borderSoft

                Column {
                    id: portContent
                    x: Theme.spacingMedium
                    y: Theme.spacingMedium
                    width: parent.width - 2 * Theme.spacingMedium
                    spacing: Theme.spacingSmall

                    Text {
                        width: parent.width
                        text: portCard.modelData.device
                        color: portCard.selected ? Theme.accent : Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: Theme.fontSizeMedium
                        font.weight: Font.DemiBold
                        elide: Text.ElideMiddle
                    }
                    Text {
                        width: parent.width
                        text: root.controller.portSummary(portCard.modelData)
                        color: Theme.textSecondary
                        font.pixelSize: Theme.fontSizeSmall
                        wrapMode: Text.WrapAnywhere
                    }
                    Text {
                        width: parent.width
                        visible: text !== ""
                        text: portCard.modelData.family || ""
                        color: Theme.textMuted
                        font.pixelSize: Theme.fontSizeCaption
                        wrapMode: Text.WordWrap
                    }

                    Flow {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        KvToggleChip {
                            objectName: "embeddedSelectPort"
                            labelText: portCard.selected ? qsTr("Porta selecionada") : qsTr("Selecionar porta")
                            codeFont: false
                            outlined: true
                            height: 28
                            active: portCard.selected
                            enabled: portCard.modelData.access.readableWritable
                            tooltip: qsTr("Usar esta porta para gravar e executar na placa")
                            onToggled: root.controller.selectPort(portCard.modelData.device)
                        }
                        KvButton {
                            objectName: "embeddedIdentify"
                            text: qsTr("Identificar")
                            iconName: "search"
                            compact: true
                            enabled: portCard.modelData.access.readableWritable && !root.controller.identity.busy
                            tooltip: qsTr("Ler o chip pelo esptool; reinicia a placa")
                            onClicked: root.controller.identity.identify(portCard.modelData.device)
                        }
                        KvButton {
                            objectName: "embeddedMonitor"
                            text: qsTr("Monitor serial")
                            iconName: "terminal"
                            compact: true
                            enabled: portCard.modelData.access.readableWritable && !root.controller.identity.busy
                            onClicked: root.controller.openMonitor(portCard.modelData.device)
                        }
                        KvButton {
                            objectName: "embeddedFiles"
                            text: qsTr("Arquivos na placa")
                            iconName: "folder"
                            compact: true
                            visible: root.controller.projectMicroPython
                            enabled: portCard.modelData.access.readableWritable && !root.controller.files.busy
                            tooltip: qsTr("Acessar arquivos pelo mpremote; interrompe o programa da placa")
                            onClicked: root.controller.files.list(portCard.modelData.device, "")
                        }
                    }
                    Text {
                        width: parent.width
                        visible: !portCard.modelData.access.readableWritable
                        text: qsTr("Sem acesso: %1").arg(portCard.modelData.access.hint || portCard.modelData.access.mode)
                        color: Theme.errorSoft
                        font.pixelSize: Theme.fontSizeCaption
                        wrapMode: Text.WordWrap
                    }
                    Text {
                        width: parent.width
                        visible: root.controller.modemManagerWarns(portCard.modelData)
                        text: qsTr("O ModemManager pode ocupar esta porta. Abra Permissões para verificar a configuração.")
                        color: Theme.warningSoft
                        font.pixelSize: Theme.fontSizeCaption
                        wrapMode: Text.WordWrap
                    }
                }
            }
        }
        KvVerdict {
            width: parent.width
            visible: root.controller && !root.controller.portFound
            busy: root.controller !== null && root.controller.portsBusy
            busyText: qsTr("Procurando dispositivos…")
            neutral: true
            message: root.controller ? (root.controller.portsHint || qsTr("Conecte uma placa por USB e atualize os dispositivos.")) : ""
        }
        Text {
            width: parent.width
            visible: root.controller && root.controller.projectMicroPython
            text: root.controller && root.controller.selectedPort !== ""
                  ? qsTr("Executar MicroPython usa %1.").arg(root.controller.selectedPort)
                  : qsTr("MicroPython: sem seleção, o mpremote escolhe a primeira porta disponível.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            wrapMode: Text.WordWrap
        }
    }
}
