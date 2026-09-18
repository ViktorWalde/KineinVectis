pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os recentes da tela inicial (Etapa 2 F7): o ultimo aberto em destaque
// (Enter abre), os de caminho ausente ocultos com desfazer. A logica e'
// do RecentWorkspacesController; aqui so' a forma.
Rectangle {
    id: root

    required property var controller

    readonly property var shown: controller.visibleWorkspaces.slice(0, 4)

    height: (shown.length === 0 ? 86 : 54 + shown.length * 36)
            + (controller.hiddenMissingCount > 0 ? 32 : 0)
            + (controller.errorText === "" ? 0 : 18)
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingXSmall

        Row {
            width: parent.width
            height: 26

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Workspaces recentes")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizePanelTitle
                font.bold: true
            }

            Item {
                width: parent.width - x - clearButton.width
                height: 1
            }

            KvButton {
                id: clearButton

                compact: true
                visible: root.controller.workspaces.length > 0
                text: qsTr("Limpar")
                onClicked: root.controller.clearAll()
            }
        }

        Text {
            visible: root.shown.length === 0
            width: parent.width
            text: qsTr("Os projetos abertos com sucesso aparecerão aqui.")
            color: Theme.textMuted
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        Repeater {
            model: root.shown

            delegate: Rectangle {
                id: workspaceRow

                required property var modelData
                required property int index

                readonly property bool highlighted: index === root.controller.highlightedIndex

                width: parent.width
                height: 34
                radius: Theme.radius
                color: highlighted ? Theme.surfaceSelected
                       : openArea.containsMouse && workspaceRow.modelData.available
                       ? Theme.surface2 : "transparent"
                border.width: highlighted ? 1 : 0
                border.color: Theme.borderStrong
                opacity: workspaceRow.modelData.available ? 1.0 : 0.72

                KvIcon {
                    id: workspaceIcon

                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    name: "project"
                    size: 16
                    disabled: !workspaceRow.modelData.available
                    active: workspaceRow.modelData.pinned
                }

                Column {
                    anchors.left: workspaceIcon.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.right: pinButton.left
                    anchors.rightMargin: Theme.spacingSmall
                                         + (enterHint.visible ? enterHint.width + Theme.spacingSmall : 0)
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 0

                    Text {
                        width: parent.width
                        text: workspaceRow.modelData.name
                              + (workspaceRow.modelData.available
                                 ? "" : qsTr(" — caminho ausente"))
                        color: workspaceRow.modelData.available
                               ? Theme.textPrimary : Theme.warningSoft
                        font.pixelSize: 11
                        font.bold: workspaceRow.modelData.pinned
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: workspaceRow.modelData.root
                        color: Theme.textMuted
                        font.pixelSize: 9
                        elide: Text.ElideMiddle
                    }
                }

                Text {
                    id: enterHint

                    anchors.right: pinButton.left
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    visible: workspaceRow.highlighted && workspaceRow.modelData.available
                    text: qsTr("Enter abre")
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                KvButton {
                    id: pinButton

                    anchors.right: removeButton.left
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    text: workspaceRow.modelData.pinned ? qsTr("Soltar") : qsTr("Fixar")
                    selected: workspaceRow.modelData.pinned
                    onClicked: root.controller.togglePinned(workspaceRow.modelData.root)
                }

                KvIconButton {
                    id: removeButton

                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    iconName: "close"
                    tooltip: qsTr("Remover dos workspaces recentes")
                    onClicked: root.controller.removeWorkspace(workspaceRow.modelData.root)
                }

                MouseArea {
                    id: openArea

                    anchors.left: parent.left
                    anchors.right: pinButton.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    enabled: workspaceRow.modelData.available
                    hoverEnabled: true
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: root.controller.openWorkspace(workspaceRow.modelData.root)
                }
            }
        }

        Row {
            visible: root.controller.hiddenMissingCount > 0
            width: parent.width
            height: 28
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.controller.hiddenMissingCount === 1
                      ? qsTr("1 recente sem caminho foi ocultado")
                      : qsTr("%1 recentes sem caminho foram ocultados")
                        .arg(root.controller.hiddenMissingCount)
                color: Theme.textMuted
                font.pixelSize: 10
            }

            KvButton {
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                text: qsTr("Desfazer")
                onClicked: root.controller.restoreMissing()
            }
        }

        Text {
            visible: root.controller.errorText !== ""
            width: parent.width
            text: root.controller.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
            elide: Text.ElideRight
        }
    }
}
