pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A seccao SISTEMA: o que a sonda MEDIU no alvo (§5.1 da especificacao).
//
// Antes isto vinha espremido numa linha so' dentro do veredito
// (`arch · kernel` mais uma fileira de ✓/✗). Ferramenta faltando e' informacao
// de DECISAO — sem `gdbserver` nao ha' depuracao remota — e merece espaco e o
// caminho de onde ela esta'.
Item {
    id: root

    property bool probed: false
    property string probeArch: ""
    property string probeKernel: ""
    property var probeTools: []

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            visible: !root.probed
            wrapMode: Text.WordWrap
            text: qsTr("Ainda não sondei este alvo. A sonda mede arquitetura, kernel e quais "
                       + "ferramentas ele tem — nada é instalado no alvo.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: root.probed
            text: root.probeArch + "  ·  " + root.probeKernel
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            elide: Text.ElideRight
        }

        Repeater {
            model: root.probed ? root.probeTools : []

            delegate: Row {
                id: ferramenta

                required property var modelData

                width: coluna.width
                spacing: Theme.spacingSmall

                Text {
                    width: 96
                    text: (ferramenta.modelData.found ? "✓  " : "✗  ") + ferramenta.modelData.id
                    color: ferramenta.modelData.found ? Theme.textPrimary : Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 10
                }

                Text {
                    width: ferramenta.width - 96 - ferramenta.spacing
                    text: ferramenta.modelData.found
                          ? (ferramenta.modelData.path || "")
                          : qsTr("não está no alvo")
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 9
                    elide: Text.ElideMiddle
                }
            }
        }
    }
}
