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

    function focusField() { field.forceActiveFocus(); }

    signal edited(string text)
    signal accepted()

    // Desde 2026-10-04 o rotulo MORA NO CAMPO e sobe ao focar ou ter texto
    // (o rotulo flutuante do KvTextField; pedido do autor: campos "mais
    // modernos"). Antes era um rotulo pequeno em cima de uma caixa de 26 px.
    implicitHeight: field.implicitHeight

    KvTextField {
        id: field

        activeFocusOnTab: true
        anchors.left: parent.left
        anchors.right: parent.right
        label: root.label
        placeholder: root.placeholder
        pixelSize: Theme.fontSizeSmall
        readOnly: root.readOnlyField
        echoMode: root.secret ? TextInput.Password : TextInput.Normal
        inputMethodHints: root.numeric ? Qt.ImhDigitsOnly : Qt.ImhNone
        text: root.value
        onEdited: (text) => root.edited(text)
        onAccepted: root.accepted()
    }
}
