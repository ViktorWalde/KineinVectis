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
        anchors.left: parent.left
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
                anchors.left: parent.left
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
                    font.pixelSize: Theme.fontSizeSmall
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

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        height: 28
        spacing: Theme.spacingSmall

        KvTextField {
            id: newBranchInput

            width: parent.width - createBranchButton.width - Theme.spacingSmall
            height: parent.height
            pixelSize: Theme.fontSizeCaption
            placeholder: qsTr("nova branch")
            onAccepted: root.branchCreateRequested(newBranchInput.text)
        }

        KvButton {
            id: createBranchButton

            height: parent.height
            compact: true
            primary: true
            enabled: newBranchInput.text.trim() !== ""
            text: qsTr("Criar")
            onClicked: root.branchCreateRequested(newBranchInput.text)
        }
    }
}

