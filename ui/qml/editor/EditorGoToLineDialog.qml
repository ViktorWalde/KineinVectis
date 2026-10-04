import QtQuick
import KineinVectis

Rectangle {
    id: root

    property real maxAvailableWidth: 300

    signal confirmRequested()
    signal cancelRequested()

    width: Math.min(300, maxAvailableWidth)
    height: goToLineColumn.height + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accent
    border.width: 1

    function openWithValue(value) {
        goToLineInput.text = value;
        goToLineInput.forceActiveFocus();
        goToLineInput.selectAll();
    }

    function currentValue() {
        return goToLineInput.text.trim();
    }

    // A roda do mouse dentro da caixa nao atravessa para o codigo (o clique
    // passa: so' a roda e' segurada, KvBackdrop).
    KvBackdrop {
        acceptedButtons: Qt.NoButton
    }

    Column {
        id: goToLineColumn

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Ir para linha")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.bold: true
        }

        KvTextField {
            id: goToLineInput

            width: parent.width
            height: 30
            onAccepted: root.confirmRequested()
            Keys.onEscapePressed: root.cancelRequested()
        }

        Text {
            width: parent.width
            text: qsTr("linha ou linha:coluna · Enter vai · Esc cancela")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            wrapMode: Text.WordWrap
        }
    }
}
