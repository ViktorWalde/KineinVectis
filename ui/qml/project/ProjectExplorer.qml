import QtQuick

Rectangle {
    id: root

    property string workspaceName: ""
    property string workspaceKindLabel: ""
    property string selectedPath: ""
    property var entriesModel

    signal createFileRequested()
    signal createDirectoryRequested()
    signal refreshRequested()
    signal closeRequested()
    signal entrySelected(string path, string kind)
    signal directoryToggleRequested(string path, int index, bool expanded)
    signal fileOpenRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)

    implicitWidth: 260
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.workspaceName
                color: Theme.textPrimary
                font.pixelSize: 13
                font.bold: true
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: kindText.width + 10
                height: 16
                radius: 8
                color: Theme.accentDim

                Text {
                    id: kindText

                    anchors.centerIn: parent
                    text: root.workspaceKindLabel
                    color: Theme.accent
                    font.pixelSize: 9
                    font.bold: true
                }
            }

            Item {
                width: parent.width - x - refreshChip.width
                       - newFileChip.width - newFolderChip.width
                       - closeProjectChip.width - 3 * Theme.spacingSmall
                height: 1
            }

            Rectangle {
                id: newFileChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                radius: Theme.radius
                color: newFileArea.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "+"
                    color: Theme.textSecondary
                    font.pixelSize: 15
                    font.bold: true
                }

                MouseArea {
                    id: newFileArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.createFileRequested()
                }
            }

            Rectangle {
                id: newFolderChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                radius: Theme.radius
                color: newFolderArea.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "▣"
                    color: Theme.textSecondary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: newFolderArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.createDirectoryRequested()
                }
            }

            Rectangle {
                id: refreshChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                radius: Theme.radius
                color: refreshArea.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "⟳"
                    color: Theme.textSecondary
                    font.pixelSize: 13
                }

                MouseArea {
                    id: refreshArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.refreshRequested()
                }
            }

            Rectangle {
                id: closeProjectChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                radius: Theme.radius
                color: closeProjectArea.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "x"
                    color: closeProjectArea.containsMouse
                           ? Theme.errorSoft : Theme.textSecondary
                    font.pixelSize: 13
                }

                MouseArea {
                    id: closeProjectArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.closeRequested()
                }
            }
        }

        ListView {
            id: explorerView

            width: parent.width
            height: parent.height - y
            clip: true
            model: root.entriesModel

            delegate: Rectangle {
                id: treeRow

                required property int index
                required property string path
                required property string name
                required property string kind
                required property int depth
                required property bool expanded

                width: explorerView.width
                height: 24
                radius: Theme.radius
                color: treeRow.path === root.selectedPath
                       ? Theme.accentDim
                       : (entryArea.containsMouse ? Theme.surface2 : "transparent")

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall + treeRow.depth * 14
                    spacing: Theme.spacingSmall

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 12
                        text: treeRow.kind === "directory"
                              ? (treeRow.expanded ? "▾" : "▸") : "·"
                        color: treeRow.kind === "directory"
                               ? Theme.accent : Theme.textMuted
                        font.pixelSize: 12
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: treeRow.name
                        color: treeRow.kind === "directory"
                               ? Theme.textPrimary : Theme.textSecondary
                        font.pixelSize: 12
                    }
                }

                MouseArea {
                    id: entryArea

                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    cursorShape: Qt.PointingHandCursor
                    onClicked: function(mouse) {
                        root.entrySelected(treeRow.path, treeRow.kind);
                        if (mouse.button === Qt.RightButton) {
                            const pt = entryArea.mapToItem(null, mouse.x, mouse.y);
                            root.contextMenuRequested(treeRow.path, treeRow.kind,
                                                      treeRow.name, pt.x, pt.y);
                            return;
                        }
                        if (treeRow.kind === "directory") {
                            root.directoryToggleRequested(treeRow.path, treeRow.index,
                                                          treeRow.expanded);
                        } else if (treeRow.kind === "file") {
                            root.fileOpenRequested(treeRow.path);
                        }
                    }
                }
            }
        }
    }
}
