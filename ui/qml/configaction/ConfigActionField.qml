pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Um parametro de acao: rotulo, o que ele FAZ, e os valores que o projeto
// ja' oferece.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). Pedido do autor: *"mostrar o que
// aquilo faz ou nao no projeto, e ter uma descricao explicando... e o usuario
// clicar no campo e, em vez de digitar manualmente, ter a opcao de selecionar
// visualmente a leitura que a IDE faz"*.
//
// Eram duas faltas e ele juntou as duas com razao: o campo `visibility` com
// placeholder `PRIVATE` nao dizia o que muda se virar `PUBLIC`, e o campo
// `target` pedia que se digitasse um nome que a IDE ja' sabe de cor.
//
// As sugestoes sao CHIPS e nao um combo fechado: o campo continua livre,
// porque nem todo alvo aparece na leitura (nome montado por variavel no
// CMake, por exemplo). Sugerir sem impedir.
Item {
    id: root

    property var param: null
    property string value: ""

    signal edited(string text)

    readonly property var sugestoes: root.param && root.param.suggestions
                                     ? root.param.suggestions : []

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                width: 110
                text: root.param
                      ? root.param.label + (root.param.required ? " *" : "")
                      : ""
                color: Theme.textSecondary
                font.pixelSize: 11
                elide: Text.ElideRight
            }

            Rectangle {
                width: parent.width - 110 - Theme.spacingSmall
                height: 26
                radius: Theme.radius
                color: Theme.background0
                border.color: entrada.activeFocus ? Theme.accent : Theme.borderSoft
                border.width: 1

                TextInput {
                    id: entrada

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    selectedTextColor: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                    clip: true
                    selectByMouse: true
                    text: root.value

                    onTextEdited: root.edited(text)
                }

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    visible: entrada.text === ""
                    text: root.param ? root.param.placeholder : ""
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                }
            }
        }

        // O QUE ESTE CAMPO FAZ. Sem isto, `visibility` era uma palavra.
        Text {
            x: 110 + Theme.spacingSmall
            width: parent.width - 110 - Theme.spacingSmall
            visible: root.param !== null && root.param.description !== undefined
                     && root.param.description !== ""
            wrapMode: Text.WordWrap
            text: root.param ? root.param.description : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        // O QUE O PROJETO OFERECE. Clicar preenche.
        Flow {
            x: 110 + Theme.spacingSmall
            width: parent.width - 110 - Theme.spacingSmall
            visible: root.sugestoes.length > 0
            spacing: Theme.spacingXSmall

            Repeater {
                model: root.sugestoes

                delegate: KvToggleChip {
                    id: chipValor

                    required property var modelData

                    width: Math.min(200, medida.width + 2 * Theme.spacingSmall)
                    height: 20
                    labelText: String(chipValor.modelData)
                    active: String(chipValor.modelData) === root.value
                    onToggled: root.edited(String(chipValor.modelData))

                    TextMetrics {
                        id: medida

                        font.family: Theme.monoFont
                        font.pixelSize: 11
                        text: String(chipValor.modelData)
                    }
                }
            }
        }
    }
}
