pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A caixa de COMMIT: a mensagem de erro do git e a linha de commit.
//
// Saiu do GitPanel.qml em 2026-09-03 (etapa 17 do roadmaps/34). As duas ficam
// juntas porque sao a mesma responsabilidade — o que o usuario ESCREVE e o que
// o git respondeu sobre isso —, e porque a mensagem de erro se ancora na linha
// de commit: separa-las criaria dependencia entre dois arquivos para nada.
Item {
    id: root

    property bool historyVisible: false
    property string errorText: ""
    property int stagedCount: 0

    signal commitRequested(string message)

    function clearMessage() {
        commitInput.text = "";
    }

    Text {
        anchors.bottom: commitBar.top
        anchors.bottomMargin: 2 * Theme.spacingSmall + 2
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.errorText !== "" && !root.historyVisible
        text: root.errorText
        color: Theme.errorSoft
        font.pixelSize: 10
        elide: Text.ElideRight
    }

    Row {
        id: commitBar

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 30
        spacing: Theme.spacingSmall
        visible: !root.historyVisible

        Rectangle {
            width: parent.width - commitButton.width - Theme.spacingSmall
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
                    root.commitRequested(commitInput.text);
                    commitInput.text = "";
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: commitInput.text === ""
                    text: qsTr("Mensagem do commit...")
                    color: Theme.textMuted
                    font.pixelSize: 12
                }
            }
        }

        Rectangle {
            id: commitButton

            readonly property bool commitEnabled: root.stagedCount > 0
                && commitInput.text.trim() !== ""

            anchors.verticalCenter: parent.verticalCenter
            width: commitLabel.width + 2 * Theme.spacingMedium
            height: 30
            radius: Theme.radius
            opacity: commitEnabled ? 1.0 : 0.5
            color: commitEnabled
                   ? (commitButtonArea.pressed ? Theme.accentDim : Theme.accent)
                   : Theme.surface1
            border.color: commitEnabled ? "transparent" : Theme.borderSoft
            border.width: commitEnabled ? 0 : 1

            Text {
                id: commitLabel

                anchors.centerIn: parent
                text: root.stagedCount > 0
                      ? qsTr("Commit (%1)").arg(root.stagedCount)
                      : qsTr("Commit")
                color: commitButton.commitEnabled
                       ? Theme.background0 : Theme.textMuted
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: commitButtonArea

                anchors.fill: parent
                cursorShape: commitButton.commitEnabled
                             ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: {
                    if (commitButton.commitEnabled) {
                        root.commitRequested(commitInput.text);
                        commitInput.text = "";
                    }
                }
            }
        }
    }
}
