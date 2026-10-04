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
    // A sonda rodou e nao alcancou o alvo: nao e' "ainda nao sondei".
    property bool failed: false
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
            text: root.failed
                  ? qsTr("A última sonda não alcançou o alvo. O que ele tem aparece aqui quando ela responder.")
                  : qsTr("Ainda não sondei este alvo. A sonda mede arquitetura, kernel e quais "
                         + "ferramentas ele tem — nada é instalado no alvo.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Text {
            width: parent.width
            visible: root.probed
            text: root.probeArch + "  ·  " + root.probeKernel
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeSmall
            elide: Text.ElideRight
        }

        Repeater {
            model: root.probed ? root.probeTools : []

            delegate: Row {
                id: tool

                required property var modelData

                width: coluna.width
                height: 22
                spacing: Theme.spacingSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: tool.modelData.found ? "check" : "close"
                    size: 13
                    success: tool.modelData.found
                    disabled: !tool.modelData.found
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 84
                    text: tool.modelData.id
                    color: tool.modelData.found ? Theme.textPrimary : Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeCaption
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: tool.width - 84 - 13 - 2 * tool.spacing
                    text: tool.modelData.found
                          ? (tool.modelData.path || "")
                          : qsTr("não está no alvo")
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeMicro
                    elide: Text.ElideMiddle
                }
            }
        }
    }
}
