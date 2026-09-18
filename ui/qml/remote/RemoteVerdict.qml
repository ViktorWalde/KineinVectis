pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O que a sonda MEDIU no alvo (arquitetura, kernel, o que ele tem), o
// desfecho do deploy e a ultima linha que a IDE compos ou rodou. Nada aqui
// e' deduzido: e' o que o `ssh` trouxe.
Item {
    id: root

    property bool probing: false
    property bool probeOk: false
    property string probeArch: ""
    property string probeKernel: ""
    property var probeTools: []
    property string probeMessage: ""
    property bool deploying: false
    property string deployMessage: ""
    property string lastCommand: ""
    property string lastOutcome: ""
    property string errorText: ""

    implicitHeight: coluna.implicitHeight

    function toolsLine() {
        return root.probeTools.map(function(t) {
            return t.id + (t.found ? " ✓" : " ✗");
        }).join("   ");
    }

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            visible: root.probing
            text: qsTr("Sondando o alvo (ssh, até 5 s)...")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Rectangle {
            width: parent.width
            visible: !root.probing && (root.probeOk || root.probeMessage !== "")
            height: veredito.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: root.probeOk ? Theme.successSoft : Theme.errorSoft
            opacity: 0.18
        }

        Text {
            id: veredito

            width: parent.width
            visible: !root.probing && (root.probeOk || root.probeMessage !== "")
            wrapMode: Text.WordWrap
            text: root.probeOk
                  ? root.probeArch + " · " + root.probeKernel + "\n" + root.toolsLine()
                  : root.probeMessage
            color: root.probeOk ? Theme.textPrimary : Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: root.deploying || root.deployMessage !== ""
            wrapMode: Text.WordWrap
            text: root.deploying ? qsTr("Enviando...") : root.deployMessage
            color: Theme.textSecondary
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: root.lastOutcome !== ""
            wrapMode: Text.WordWrap
            text: root.lastOutcome
            color: Theme.textSecondary
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: root.lastCommand !== ""
            wrapMode: Text.WrapAnywhere
            text: "$ " + root.lastCommand
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 9
        }

        Text {
            width: parent.width
            visible: root.errorText !== ""
            wrapMode: Text.WordWrap
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
        }
    }
}
