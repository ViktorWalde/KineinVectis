pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O menu de BRANCHES: trocar de branch e criar branch.
//
// Saiu do GitPanel.qml em 2026-09-03 (etapa 17 do roadmaps/34). O
// posicionamento (ancorado sob a barra de acoes) fica no GitPanel, que e' quem
// conhece os irmaos.
Rectangle {
    id: root

    property var branchesModel

    signal branchCheckoutRequested(string branch)
    signal branchCreateRequested(string name)

    width: Math.min(340, parent.width)
    height: 190
    z: 20
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accent
    border.width: 1

    ListView {
        id: branchesView

        anchors.top: parent.top
            anchors.right: parent.right
        anchors.bottom: newBranchRow.top
        anchors.margins: Theme.spacingSmall
        clip: true
        model: root.branchesModel

        delegate: Rectangle {
            id: branchRow

            required property string name
            required property bool current

            width: branchesView.width
            height: 24
            radius: Theme.radiusXSmall
            color: branchArea.containsMouse ? Theme.surface2 : "transparent"

            Row {
                anchors.verticalCenter: parent.verticalCenter
                            anchors.leftMargin: Theme.spacingSmall
                spacing: Theme.spacingXSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: branchRow.current
                    name: "check"
                    size: 14
                    success: true
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: branchRow.name
                    color: branchRow.current ? Theme.accent : Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }
            }

            MouseArea {
                id: branchArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: branchRow.current ? Qt.ArrowCursor : Qt.PointingHandCursor
                onClicked: {
                    if (!branchRow.current) {
                        root.branchCheckoutRequested(branchRow.name);
                    }
                }
            }
        }
    }

    Row {
        id: newBranchRow

            anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        height: 28
        spacing: Theme.spacingSmall

        Rectangle {
            width: parent.width - createBranchButton.width - Theme.spacingSmall
            height: parent.height
            radius: Theme.radiusXSmall
            color: Theme.background0
            border.color: newBranchInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: newBranchInput

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 10
                clip: true
                onAccepted: root.branchCreateRequested(text)

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: newBranchInput.text === ""
                    text: qsTr("nova branch")
                    color: Theme.textMuted
                    font.pixelSize: 10
                }
            }
        }

        Rectangle {
            id: createBranchButton

            width: createBranchLabel.width + 2 * Theme.spacingSmall
            height: parent.height
            radius: Theme.radiusXSmall
            color: createBranchArea.pressed ? Theme.accentDim : Theme.accent

            Text {
                id: createBranchLabel

                anchors.centerIn: parent
                text: qsTr("Criar")
                color: Theme.background0
                font.pixelSize: 10
                font.bold: true
            }

            MouseArea {
                id: createBranchArea

                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.branchCreateRequested(newBranchInput.text)
            }
        }
    }
}

