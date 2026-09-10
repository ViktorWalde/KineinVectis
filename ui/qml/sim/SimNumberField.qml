pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Um campo numerico com rotulo em cima. Comeca vazio e nao ganha padrao.
Column {
    id: root

    property string label: ""
    property string text: ""

    signal edited(string texto)

    spacing: 2

    Text {
        text: root.label
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus - 1
    }

    Rectangle {
        width: 120
        height: 26
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
        // Vazio nao e' erro: e' "ainda nao".
        border.color: root.text.trim() === ""
                      ? Theme.borderSoft
                      : (isFinite(Number(root.text)) ? Theme.borderStrong : Theme.warningSoft)

        TextInput {
            id: campo

            anchors.fill: parent
            anchors.margins: Theme.spacingXSmall
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            text: root.text
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
            selectByMouse: true

            onTextEdited: root.edited(campo.text)
        }
    }
}
