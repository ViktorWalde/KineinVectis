import QtQuick
import KineinVectis

Item {
    id: panel

    property string terminalText: ""
    property bool terminalActive: false
    property bool workspaceAvailable: false

    signal openRequested()
    signal inputSubmitted(string text)

    function focusInput() {
        shellInput.forceActiveFocus();
    }

    function clearInput() {
        shellInput.text = "";
    }

    onVisibleChanged: {
        if (!visible) {
            return;
        }
        if (!terminalActive && workspaceAvailable) {
            openRequested();
        }
        shellInput.forceActiveFocus();
    }

    Flickable {
        id: shellFlick

        anchors.top: parent.top
        anchors.bottom: shellInputBox.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        contentWidth: width
        contentHeight: shellText.contentHeight + 8
        boundsBehavior: Flickable.StopAtBounds
        onContentHeightChanged: {
            if (contentHeight > height) {
                contentY = contentHeight - height;
            }
        }

        TextEdit {
            id: shellText

            width: shellFlick.width
            text: panel.terminalText
            readOnly: true
            color: Theme.textSecondary
            selectionColor: Theme.accentDim
            selectedTextColor: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            wrapMode: TextEdit.WrapAnywhere
            selectByMouse: true
        }

        Text {
            anchors.centerIn: parent
            visible: panel.terminalText === ""
            text: qsTr("Seu shell ($SHELL) abre aqui na raiz do"
                       + " workspace (Alt+F12).")
            color: Theme.textMuted
            font.pixelSize: 11
        }
    }

    Rectangle {
        id: shellInputBox

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 24
        radius: Theme.radius
        color: Theme.background0
        border.color: shellInput.activeFocus ? Theme.accent : Theme.borderSoft
        border.width: 1

        TextInput {
            id: shellInput

            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            verticalAlignment: TextInput.AlignVCenter
            color: Theme.textPrimary
            selectionColor: Theme.accentDim
            selectedTextColor: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            clip: true
            selectByMouse: true
            onAccepted: panel.inputSubmitted(text)
        }
    }
}
