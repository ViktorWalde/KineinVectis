pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Um campo de texto com rotulo, do jeito que este painel usa seis vezes.
//
// POR QUE EXISTE. Sao seis campos com a mesma moldura, o mesmo foco e o mesmo
// placeholder. Repetir isso seis vezes no formulario e' como as derivacoes
// duplicadas que o 16o gate passou a caçar (`docs/roadmaps/39` §5): uma delas
// diverge, e a divergencia aparece como "aquele campo se comporta diferente".
//
// `secret: true` esconde o texto E impede o eco no `echoMode` — e' o campo da
// senha da sessao, que nunca sai daqui para o disco.
Item {
    id: root

    property string label: ""
    property string value: ""
    property string placeholder: ""
    property bool secret: false
    property bool numeric: false
    property bool readOnlyField: false

    signal edited(string text)
    signal accepted()

    implicitHeight: rotulo.height + 2 + caixa.height

    Text {
        id: rotulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: root.label
        color: Theme.textMuted
        font.pixelSize: 10
    }

    Rectangle {
        id: caixa

        anchors.top: rotulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        height: 22
        radius: Theme.radius
        color: root.readOnlyField ? Theme.background2 : Theme.surface2
        border.width: 1
        border.color: entrada.activeFocus ? Theme.accent : Theme.borderSoft

        TextInput {
            id: entrada

            anchors.fill: parent
            anchors.leftMargin: 4
            anchors.rightMargin: 4
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            selectByMouse: true
            readOnly: root.readOnlyField
            color: Theme.textPrimary
            selectionColor: Theme.accentDim
            selectedTextColor: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            echoMode: root.secret ? TextInput.Password : TextInput.Normal
            inputMethodHints: root.numeric ? Qt.ImhDigitsOnly : Qt.ImhNone
            text: root.value

            onTextEdited: root.edited(text)
            onAccepted: root.accepted()

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: entrada.text === ""
                text: root.placeholder
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }
    }
}
