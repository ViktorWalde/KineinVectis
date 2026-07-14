pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property string workspaceName: ""
    property string workspaceKindLabel: ""
    property string selectedPath: ""
    property var entriesModel
    // path absoluto -> kind do git (fatia M3.1); a revisão força rebind.
    property var gitKinds: ({})
    property int gitRevision: 0

    signal createFileRequested()
    signal createDirectoryRequested()
    signal refreshRequested()
    signal closeRequested()
    signal entrySelected(string path, string kind)
    signal directoryToggleRequested(string path, int index, bool expanded)
    signal fileOpenRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)

    function gitFileColor(path, revision) {
        const kind = gitKinds[path];
        if (kind === undefined) {
            return Theme.textSecondary;
        }
        if (kind === "conflicted") {
            return Theme.errorSoft;
        }
        if (kind === "untracked" || kind === "added") {
            return Theme.successSoft;
        }
        if (kind === "deleted") {
            return Theme.textDisabled;
        }
        return Theme.infoSoft;
    }

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

            KvIconButton {
                id: newFileChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "file"
                iconSize: 15
                tooltip: qsTr("Novo arquivo")
                onClicked: root.createFileRequested()
            }

            KvIconButton {
                id: newFolderChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "folder"
                iconSize: 15
                tooltip: qsTr("Nova pasta")
                onClicked: root.createDirectoryRequested()
            }

            KvIconButton {
                id: refreshChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "refresh"
                iconSize: 15
                tooltip: qsTr("Atualizar projeto")
                onClicked: root.refreshRequested()
            }

            KvIconButton {
                id: closeProjectChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "close"
                iconSize: 14
                danger: true
                tooltip: qsTr("Fechar workspace")
                onClicked: root.closeRequested()
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
                       ? Theme.surfaceSelected
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
                        font.pixelSize: Theme.fontSizeTree
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: treeRow.name
                        color: treeRow.kind === "directory"
                               ? Theme.textPrimary
                               : root.gitFileColor(treeRow.path,
                                                   root.gitRevision)
                        font.pixelSize: Theme.fontSizeTree
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
