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
        font.pixelSize: Theme.fontSizeSmall
    }

    KvTextField {
        id: campo

        anchors.left: rotulo.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        height: parent.height
        placeholder: root.placeholder
        text: root.value
    }
}
