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
    // O gesto concreto para a UNICA falha que tem um (0.133.0): a chave.
    property bool canCopyId: false
    // Uma linha composta pelo core e ARMADA: visivel antes de rodar.
    property string armedCommand: ""
    property string armedName: ""

    signal copyIdRequested()
    signal runArmedRequested()
    signal disarmRequested()

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

        KvVerdict {
            width: parent.width
            busy: root.probing
            busyText: qsTr("Sondando o alvo (ssh, até 5 s)...")
            ok: root.probeOk
            message: root.probeOk
                     ? root.probeArch + " · " + root.probeKernel + "\n" + root.toolsLine()
                     : root.probeMessage
        }

        // A sonda disse "o alvo recusou a chave". Ate' 0.132.0 isso terminava
        // numa frase mandando o autor ir ao terminal sozinho — o defeito 14 da
        // especificacao. Agora o gesto fica AQUI, onde a causa foi explicada.
        KvButton {
            visible: root.canCopyId
            compact: true
            text: qsTr("Copiar minha chave (ssh-copy-id)")
            onClicked: root.copyIdRequested()
        }

        // A linha ARMADA. Ela aparece ANTES de rodar, por exigencia da secao 10
        // da especificacao: geracao/copia de chave nunca acontece em silencio.
        Column {
            width: parent.width
            visible: root.armedCommand !== ""
            spacing: Theme.spacingXSmall

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                text: qsTr("%1 — a IDE não digita senha: o ssh vai pedir no terminal, "
                           + "e o host key você aceita uma vez.").arg(root.armedName)
                color: Theme.textSecondary
                font.pixelSize: 10
            }

            Text {
                width: parent.width
                wrapMode: Text.WrapAnywhere
                text: "$ " + root.armedCommand
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 10
            }

            Row {
                spacing: Theme.spacingSmall

                KvButton {
                    compact: true
                    text: qsTr("Rodar no terminal")
                    onClicked: root.runArmedRequested()
                }

                KvButton {
                    compact: true
                    text: qsTr("Cancelar")
                    onClicked: root.disarmRequested()
                }
            }
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
