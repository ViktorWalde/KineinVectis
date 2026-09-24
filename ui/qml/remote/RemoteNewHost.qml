pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// "Configurar servidor" SEM formulario (0.134.0): a pessoa cola a linha `ssh`
// que ja' usa e o core a interpreta. Ideia do `Remote-SSH: Add New SSH Host…`
// do VS Code, comparada na secao 3.1 da especificacao.
//
// O outro caminho — "usar o SSH que ja' funciona" — e' o RemoteDiscovery, acima
// deste. Os dois existem separados porque respondem perguntas diferentes: um e'
// "qual dos meus alvos", o outro e' "este alvo que ainda nao esta' em lugar
// nenhum".
//
// Componente burro: recebe por property, pede por signal. Ele NUNCA executa a
// linha — quem a le' e' o core, e ler nao e' gravar: o que volta e' um
// rascunho que a pessoa confere no formulario abaixo antes de salvar.
Item {
    id: root

    property string pasted: ""
    property bool canParse: false
    property var proposalSource: []
    property string errorText: ""

    signal pastedEdited(string text)
    signal parseRequested()

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 4

        Text {
            text: qsTr("Configurar um servidor novo")
            color: Theme.textPrimary
            font.pixelSize: 11
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: qsTr("Cole a linha que você já usaria no terminal. A IDE lê a linha — "
                       + "não a executa — e preenche o formulário abaixo para você conferir.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: parent.width - interpretar.width - parent.spacing
                label: qsTr("Comando ssh")
                placeholder: "ssh -p 2222 pi@192.168.0.42"
                value: root.pasted
                onEdited: text => root.pastedEdited(text)
            }

            KvButton {
                id: interpretar

                anchors.bottom: parent.bottom
                compact: true
                text: qsTr("Interpretar")
                enabled: root.canParse
                onClicked: root.parseRequested()
            }
        }

        // A PROCEDENCIA de cada campo. Sem isso, o formulario se preencheria
        // sozinho e a pessoa teria de adivinhar de onde veio cada valor.
        Repeater {
            model: root.proposalSource

            delegate: Text {
                required property var modelData

                width: coluna.width
                wrapMode: Text.WordWrap
                text: "· " + modelData
                color: Theme.textMuted
                font.pixelSize: 9
            }
        }

        Text {
            width: parent.width
            visible: root.errorText !== ""
            wrapMode: Text.WordWrap
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
        }
    }
}
