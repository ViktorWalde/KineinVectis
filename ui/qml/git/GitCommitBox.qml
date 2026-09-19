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

    signal commitRequested(string message)
    signal commitAndPushRequested(string message)
    signal amendToggled()

    readonly property bool canCommit: commitInput.text.trim() !== "" && (stagedCount > 0 || amend)

    implicitHeight: historyVisible ? 0 : coluna.implicitHeight
    visible: !historyVisible

    function clearMessage() {
        commitInput.text = "";
    }

    Column {
        id: coluna

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

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            KvToggleChip {
                anchors.verticalCenter: parent.verticalCenter
                labelText: qsTr("Amend")
                active: root.amend
                onToggled: root.amendToggled()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.amend
                text: qsTr("reescreve o último commit")
                color: Theme.warningSoft
                font.pixelSize: 10
            }

            Item { width: parent.width - x - botoes.width; height: 1 }

            Row {
                id: botoes

                spacing: Theme.spacingSmall

                KvButton {
                    compact: true
                    text: qsTr("Commit e Push")
                    enabled: root.canCommit && !root.remoteRunning
                    onClicked: {
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
