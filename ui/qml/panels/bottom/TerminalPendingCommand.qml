import QtQuick
import KineinVectis

// INF-02 (62 §3.4): o comando que a IDE pediu e o core guarda ate' o primeiro
// prompt do shell. A faixa so' aparece depois de `graceMs` (o prompt costuma
// chegar antes) e da' a saida manual a um shell que nunca marca o prompt,
// como um ~/.bashrc que troca de shell. Enquanto espera, o terminal segue
// aceitando teclas: e' assim que a pessoa responde a um assistente do shell.
Item {
    id: root

    property string command: ""
    property int graceMs: 1000
    property bool graceElapsed: false
    readonly property bool showsBand: command !== "" && graceElapsed
    signal sendRequested()
    signal discardRequested()

    onCommandChanged: {
        graceElapsed = false;
        if (command !== "") grace.restart();
        else grace.stop();
    }

    Timer {
        id: grace
        interval: root.graceMs
        repeat: false
        onTriggered: root.graceElapsed = true
    }

    implicitHeight: showsBand ? body.implicitHeight + 2 * Theme.spacingSmall : 0
    visible: showsBand

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius
        color: Theme.textMuted
        opacity: 0.18
    }

    Column {
        id: body

        x: Theme.spacingSmall
        y: Theme.spacingSmall
        width: parent.width - 2 * Theme.spacingSmall
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            textFormat: Text.PlainText
            wrapMode: Text.WordWrap
            text: qsTr("Aguardando o prompt do shell para enviar o comando")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeSmall
            font.weight: Font.DemiBold
        }
        Text {
            objectName: "terminalPendingCommandText"
            width: parent.width
            textFormat: Text.PlainText
            wrapMode: Text.Wrap
            text: root.command
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeCaption
        }
        Row {
            spacing: Theme.spacingSmall

            KvButton {
                objectName: "terminalPendingCommandSend"
                text: qsTr("Enviar agora")
                primary: true
                compact: true
                onClicked: root.sendRequested()
            }
            KvButton {
                objectName: "terminalPendingCommandDiscard"
                text: qsTr("Descartar")
                compact: true
                onClicked: root.discardRequested()
            }
        }
    }
}
