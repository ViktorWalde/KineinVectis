pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A caixa de COMMIT da HUD do Git (2026-09-18): a mensagem, o chip Amend
// (reescreve o ultimo commit — pedido explicito, nunca padrao), Commit e
// Commit e Push, e o que o git respondeu sobre isso. Burro: estado por
// property, intencao por signal.
Item {
    id: root

    property bool historyVisible: false
    property string errorText: ""
    property int stagedCount: 0
    property bool amend: false
    property bool remoteRunning: false
    // O HEAD ja' foi enviado (ahead 0): um amend reescreve historia publica.
    property bool headPushed: false
    property string branchLabel: ""
    // "Commit e Push" pede o segundo clique: o primeiro so' arma.
    property bool pushArmed: false

    signal commitRequested(string message)
    signal commitAndPushRequested(string message)
    signal amendToggled()

    // O MINIMO que esta caixa precisa (53 §4.4): os botoes nao quebram; o
    // resto se rearranja. O GitWindow soma as margens e o declara ao shell.
    readonly property real minimumWidth: buttonRow.implicitWidth
    readonly property bool compactFooter: width < amendChip.implicitWidth + Theme.spacingSmall
                                                  + buttonRow.implicitWidth
    readonly property bool canCommit: commitInput.text.trim() !== "" && (stagedCount > 0 || amend)

    implicitHeight: historyVisible ? 0 : column.implicitHeight
    visible: !historyVisible

    function clearMessage() {
        commitInput.text = "";
        pushArmed = false;
    }

    // Mexer na mensagem desarma o push: o que se confirma e' o texto atual.
    onAmendChanged: pushArmed = false

    Column {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            visible: root.errorText !== ""
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
            wrapMode: Text.WordWrap
        }

        Rectangle {
            width: parent.width
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: commitInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: commitInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                font.pixelSize: 12
                clip: true
                selectByMouse: true
                onTextEdited: root.pushArmed = false
                onAccepted: {
                    if (root.canCommit) {
                        root.commitRequested(commitInput.text);
                        commitInput.text = "";
                    }
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: commitInput.text === ""
                    text: root.amend ? qsTr("Nova mensagem do último commit…") : qsTr("Mensagem do commit…")
                    color: Theme.textMuted
                    font.pixelSize: 12
                }
            }
        }

        // Os avisos tem linha propria e quebram: na mesma linha dos botoes,
        // o texto do amend empurrava o "Commit" para fora do painel.
        Text {
            width: parent.width
            visible: root.amend || root.pushArmed
            text: root.amend
                  ? (root.headPushed
                     ? qsTr("reescreve um commit JÁ ENVIADO — vai exigir push forçado")
                     : qsTr("reescreve o último commit"))
                  : qsTr("vai enviar para origin/%1 — clique de novo").arg(root.branchLabel)
            color: root.amend && root.headPushed ? Theme.errorSoft : Theme.warningSoft
            font.pixelSize: 10
            wrapMode: Text.WordWrap
        }

        // Amend a esquerda e os botoes a direita; sem largura para os dois
        // numa linha (53 §4.4: perto do minimo o conteudo se rearranja), o
        // Amend sobe para a linha de cima.
        Item {
            id: footer

            width: parent.width
            height: root.compactFooter ? amendChip.height + Theme.spacingXSmall + buttonRow.height
                                       : Math.max(amendChip.height, buttonRow.height)

            KvToggleChip {
                id: amendChip

                y: root.compactFooter ? 0 : (footer.height - height) / 2
                labelText: qsTr("Amend")
                active: root.amend
                onToggled: root.amendToggled()
            }

            Row {
                id: buttonRow

                anchors.right: parent.right
                y: root.compactFooter ? amendChip.height + Theme.spacingXSmall : 0
                spacing: Theme.spacingSmall

                KvButton {
                    compact: true
                    primary: root.pushArmed
                    text: root.pushArmed ? qsTr("Confirmar Commit e Push") : qsTr("Commit e Push")
                    enabled: root.canCommit && !root.remoteRunning
                    onClicked: {
                        if (!root.pushArmed) {
                            root.pushArmed = true;
                            return;
                        }
                        root.pushArmed = false;
                        root.commitAndPushRequested(commitInput.text);
                        commitInput.text = "";
                    }
                }

                KvButton {
                    compact: true
                    primary: true
                    text: root.stagedCount > 0 && !root.amend
                          ? qsTr("Commit (%1)").arg(root.stagedCount) : qsTr("Commit")
                    enabled: root.canCommit
                    onClicked: {
                        root.commitRequested(commitInput.text);
                        commitInput.text = "";
                    }
                }
            }
        }
    }
}
