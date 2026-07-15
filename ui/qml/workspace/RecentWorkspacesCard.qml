pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var workspaces: []
    property string errorText: ""

    signal openRequested(string rootPath)
    signal pinRequested(string rootPath)
    signal removeRequested(string rootPath)
    signal clearRequested()

    readonly property int visibleCount: Math.min(workspaces.length, 4)

    height: (workspaces.length === 0 ? 86 : 54 + visibleCount * 36)
            + (errorText === "" ? 0 : 18)
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
                visible: root.workspaces.length > 0
                text: qsTr("Limpar")
                onClicked: root.clearRequested()
            }
        }

        Text {
            visible: root.workspaces.length === 0
            width: parent.width
            text: qsTr("Os projetos abertos com sucesso aparecerão aqui.")
            color: Theme.textMuted
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        Repeater {
            model: root.workspaces.slice(0, 4)

            delegate: Rectangle {
                id: workspaceRow

                required property var modelData

                width: parent.width
                height: 34
                radius: Theme.radius
                color: openArea.containsMouse && workspaceRow.modelData.available
                       ? Theme.surface2 : "transparent"
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

                KvButton {
                    id: pinButton

                    anchors.right: removeButton.left
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    text: workspaceRow.modelData.pinned ? qsTr("Soltar") : qsTr("Fixar")
                    selected: workspaceRow.modelData.pinned
                    onClicked: root.pinRequested(workspaceRow.modelData.root)
                }

                KvIconButton {
                    id: removeButton

                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    iconName: "close"
                    tooltip: qsTr("Remover dos workspaces recentes")
                    onClicked: root.removeRequested(workspaceRow.modelData.root)
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
                    onClicked: root.openRequested(workspaceRow.modelData.root)
                }
            }
        }

        Text {
            visible: root.errorText !== ""
            width: parent.width
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
            elide: Text.ElideRight
        }
    }
}
