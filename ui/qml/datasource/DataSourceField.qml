pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Um campo de texto com rotulo, do jeito que este painel usa seis vezes.
//
// POR QUE EXISTE. Sao seis campos com a mesma moldura, o mesmo foco e o mesmo
// placeholder. Repetir isso seis vezes no formulario e' como as derivacoes
// duplicadas que o 16o gate passou a caçar (`DocsPublic/roadmaps/39` §5): uma delas
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
        font.pixelSize: Theme.fontSizeCaption
    }

    KvTextField {
        id: caixa

        anchors.top: rotulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        height: 26
        pixelSize: Theme.fontSizeSmall
        readOnly: root.readOnlyField
        echoMode: root.secret ? TextInput.Password : TextInput.Normal
        inputMethodHints: root.numeric ? Qt.ImhDigitsOnly : Qt.ImhNone
        placeholder: root.placeholder
        text: root.value
        onEdited: (text) => root.edited(text)
        onAccepted: root.accepted()
    }
}
