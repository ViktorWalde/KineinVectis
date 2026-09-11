import QtQuick
import KineinVectis

// Uma linha rotulo + campo do kit (chip, alvo, sysroot).
//
// Tres campos com a mesma forma num painel so' e' a duplicacao que o
// `verificar-qml-duplicacao.sh` persegue; um dono para a forma, tres usos.
// O campo nao aplica nada sozinho: quem manda o kit ao core e' o painel, num
// gesto so', porque `toolchain.setKit` e' uma escrita e nao tres.
Item {
    id: root

    property string labelText: ""
    property string value: ""
    property string placeholder: ""
    readonly property string text: campo.text

    width: parent ? parent.width : 320
    height: 26

    onValueChanged: campo.text = root.value

    Text {
        id: rotulo

        width: 78
        anchors.verticalCenter: parent.verticalCenter
        text: root.labelText
        color: Theme.textSecondary
        font.pixelSize: 11
    }

    Rectangle {
        anchors.left: rotulo.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        height: parent.height
        radius: Theme.radius
        color: Theme.background0
        border.width: 1
        border.color: campo.activeFocus ? Theme.accent : Theme.borderSoft

        TextInput {
            id: campo

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
        }

        Text {
            anchors.fill: campo
            verticalAlignment: Text.AlignVCenter
            text: root.placeholder
            visible: campo.text === "" && !campo.activeFocus
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 12
        }
    }
}
