import QtQuick
import KineinVectis

Item {
    id: panel

    ListModel {
        id: emptyOutputModel
    }

    property var outputModel: emptyOutputModel
    property bool running: false

    signal inputSubmitted(string text)

    function clearInput() {
        runInput.text = "";
    }

    function focusInput() {
        runInput.forceActiveFocus();
    }

    function lineColor(kind) {
        if (kind === "command") {
            return Theme.accent;
        }
        if (kind === "stderr") {
            return Theme.errorSoft;
        }
        if (kind === "stdin" || kind === "info") {
            return Theme.textMuted;
        }
        return Theme.textSecondary;
    }

    ListView {
        id: runOutputView

        anchors.top: parent.top
        anchors.bottom: runInputBox.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        model: panel.outputModel
        onCountChanged: positionViewAtEnd()

        Text {
            anchors.centerIn: parent
            visible: panel.outputModel.count === 0
            text: qsTr("Digite um comando e pressione Enter,"
                       + " ou use ▶ Iniciar (Shift+F10).")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Text {
            required property string line
            required property string kind

            width: runOutputView.width
            text: line
            color: panel.lineColor(kind)
            font.family: Theme.monoFont
            font.pixelSize: 11
            wrapMode: Text.WrapAnywhere
        }
    }

    Rectangle {
        id: runInputBox

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 24
        radius: Theme.radius
        color: Theme.background0
        border.color: runInput.activeFocus ? Theme.accent : Theme.borderSoft
        border.width: 1

        Text {
            id: runPrompt

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            text: panel.running ? ">" : "$"
            color: panel.running ? Theme.warningSoft : Theme.accent
            font.family: Theme.monoFont
            font.pixelSize: 11
            font.bold: true
        }

        TextInput {
            id: runInput

            anchors.left: runPrompt.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            verticalAlignment: TextInput.AlignVCenter
            color: Theme.textPrimary
            selectionColor: Theme.accentDim
            selectedTextColor: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            clip: true
            selectByMouse: true
            onAccepted: panel.inputSubmitted(text)

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: runInput.text === ""
                text: panel.running
                      ? qsTr("enviar para o stdin do processo")
                      : qsTr("comando no workspace (ex.: cargo test)")
                color: Theme.textMuted
                font.pixelSize: 11
            }
        }
    }
}
