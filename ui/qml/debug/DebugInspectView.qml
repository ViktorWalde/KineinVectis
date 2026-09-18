pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O que o depurador de embarcado MOSTRA (P3): os escopos do frame (Locals,
// Registers, Peripherals do SVD) com as variaveis do escolhido, a memoria em
// hexadecimal e o disassembly. Burro: le do DebugInspectController
// (`debugController.inspect`) e pede por ele; nada roda sozinho — o escopo
// Peripherals e' caro e a memoria e' onde o usuario apontar.
Rectangle {
    id: root

    property var inspect: null
    property bool paused: false

    color: Theme.background1
    border.width: 1
    border.color: Theme.borderSoft
    radius: Theme.radius

    Column {
        id: coluna

        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingXSmall

        // --- Escopos --------------------------------------------------------
        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Escopos")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            KvButton {
                text: root.inspect && root.inspect.scopesBusy ? qsTr("lendo…") : qsTr("Listar")
                compact: true
                enabled: root.paused && root.inspect !== null && root.inspect.frameId >= 0 && !root.inspect.scopesBusy
                onClicked: root.inspect.requestScopes()
            }
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall

            Repeater {
                model: root.inspect ? root.inspect.scopes : []

                KvToggleChip {
                    required property var modelData

                    labelText: modelData.name + (modelData.expensive === true ? " ⏱" : "")
                    active: root.inspect && root.inspect.selectedScope === modelData.name
                    tooltip: modelData.expensive === true
                             ? qsTr("Escopo caro (o adaptador le todos os registradores): um clique por vez")
                             : qsTr("Mostrar as variaveis de %1").arg(modelData.name)
                    onToggled: root.inspect.selectScope(modelData.name)
                }
            }
        }

        ListView {
            id: variaveis

            width: parent.width
            height: Math.min(120, contentHeight)
            clip: true
            model: root.inspect ? root.inspect.scopeVariables : []

            delegate: Text {
                required property var modelData

                width: variaveis.width
                elide: Text.ElideRight
                text: modelData.name + " = " + modelData.value
                      + (modelData.type !== undefined ? "  : " + modelData.type : "")
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 10
            }
        }

        // --- Memoria e disassembly -----------------------------------------
        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Rectangle {
                width: parent.width - ler.width - desmontar.width - 2 * Theme.spacingSmall
                height: 24
                color: Theme.background0
                border.width: 1
                border.color: Theme.borderSoft
                radius: Theme.radius

                TextInput {
                    id: endereco

                    anchors.fill: parent
                    anchors.margins: 4
                    text: root.inspect ? root.inspect.memoryReference : ""
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                    clip: true
                    onAccepted: root.inspect.readMemory(text)

                    Text {
                        anchors.fill: parent
                        visible: endereco.text === ""
                        text: qsTr("endereco, ex.: 0x3ff00000")
                        color: Theme.textMuted
                        font.pixelSize: 11
                    }
                }
            }

            KvButton {
                id: ler

                text: root.inspect && root.inspect.memoryBusy ? qsTr("lendo…") : qsTr("Memoria")
                compact: true
                enabled: root.paused && root.inspect !== null && !root.inspect.memoryBusy
                onClicked: root.inspect.readMemory(endereco.text)
            }

            KvButton {
                id: desmontar

                text: root.inspect && root.inspect.disassemblyBusy ? qsTr("lendo…") : qsTr("Desmontar")
                compact: true
                enabled: root.paused && root.inspect !== null && !root.inspect.disassemblyBusy
                onClicked: root.inspect.disassemble(endereco.text)
            }
        }

        ListView {
            id: saida

            width: parent.width
            height: Math.max(0, coluna.height - y)
            clip: true
            // Memoria e instrucoes, a ultima leitura por cima.
            model: root.inspect
                   ? (root.inspect.instructions.length > 0 && root.inspect.disassemblyBusy === false
                      && root.inspect.memoryLines.length === 0
                      ? root.inspect.instructions.map(i => root.inspect.instructionLine(i))
                      : root.inspect.memoryLines.concat(root.inspect.instructions.map(i => root.inspect.instructionLine(i))))
                   : []

            delegate: Text {
                required property string modelData

                width: saida.width
                elide: Text.ElideRight
                text: modelData
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 10
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.inspect !== null && root.inspect.errorText !== ""
            text: root.inspect ? root.inspect.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }
    }
}
