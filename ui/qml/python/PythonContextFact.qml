import QtQuick
import KineinVectis

// Um fato do contexto Python: rotulo a esquerda, valor a direita, e um
// detalhe opcional em fonte de codigo embaixo.
Column {
    id: root

    property string label: ""
    property string value: ""
    property string detail: ""
    property bool code: false

    spacing: 2

    Item {
        width: root.width
        height: 24

        Text {
            id: labelText

            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: root.label
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeBody
        }

        Text {
            anchors.left: labelText.right
            anchors.leftMargin: Theme.spacingMedium
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            horizontalAlignment: Text.AlignRight
            text: root.value
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.family: root.code ? Theme.monoFont : Theme.uiFont
            elide: Text.ElideMiddle
        }
    }

    Text {
        width: root.width
        visible: root.detail !== ""
        text: root.detail
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        font.family: Theme.monoFont
        wrapMode: Text.WrapAnywhere
    }
}
