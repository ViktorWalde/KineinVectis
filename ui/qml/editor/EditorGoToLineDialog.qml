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
            font.pixelSize: 12
            font.bold: true
        }

        Rectangle {
            width: parent.width
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: goToLineInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: goToLineInput

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 12
                clip: true
                selectByMouse: true
                onAccepted: root.confirmRequested()
                Keys.onEscapePressed: root.cancelRequested()
            }
        }

        Text {
            width: parent.width
            text: qsTr("linha ou linha:coluna · Enter vai · Esc cancela")
            color: Theme.textMuted
            font.pixelSize: 10
            wrapMode: Text.WordWrap
        }
    }
}
