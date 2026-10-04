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
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideRight
            }

            KvTextField {
                width: parent.width - 110 - Theme.spacingSmall
                height: 26
                placeholder: root.param ? root.param.placeholder : ""
                text: root.value
                onEdited: (text) => root.edited(text)
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
            font.pixelSize: Theme.fontSizeCaption
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

                    height: 20
                    labelText: String(chipValor.modelData)
                    active: String(chipValor.modelData) === root.value
                    onToggled: root.edited(String(chipValor.modelData))

                }
            }
        }
    }
}
