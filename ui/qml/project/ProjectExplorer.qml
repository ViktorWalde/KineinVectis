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
    signal scriptRunRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)

    // `revision` existe so' para o binding reavaliar quando o git muda.
    // Arquivo SEM estado de git nao e' um estado de git: cai na cor normal da
    // arvore, e por isso este caso fica aqui e nao no StatusColors.
    function gitFileColor(path, revision) {
        const kind = gitKinds[path];
        return kind === undefined ? Theme.textSecondary : StatusColors.gitKind(kind);
    }

    function treeIconName(name, kind, expanded) {
        if (kind === "directory") {
            return expanded ? "tree-folder-open" : "tree-folder-closed";
        }
        const lowerName = name.toLowerCase();
        if (lowerName.endsWith(".c") || lowerName.endsWith(".h")) {
            return "tree-file-c";
        }
        if (lowerName.endsWith(".cc") || lowerName.endsWith(".cpp")
                || lowerName.endsWith(".cxx") || lowerName.endsWith(".c++")
                || lowerName.endsWith(".hh") || lowerName.endsWith(".hpp")
                || lowerName.endsWith(".hxx") || lowerName.endsWith(".h++")
                || lowerName.endsWith(".ipp")) {
            return "tree-file-cpp";
        }
        if (lowerName.endsWith(".rs")) {
            return "tree-file-rust";
        }
        return "file";
    }

    function isRunnableScript(name, kind) {
        if (kind !== "file") return false;
        const lower = name.toLowerCase();
        return lower.endsWith(".sh") || lower.endsWith(".bash")
                || lower.endsWith(".zsh");
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
                              ? (treeRow.expanded ? "▾" : "▸") : ""
                        color: treeRow.kind === "directory"
                               ? Theme.accent : Theme.textMuted
                        font.pixelSize: Theme.fontSizeTree
                    }

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        size: 20
                        name: root.treeIconName(treeRow.name, treeRow.kind,
                                                treeRow.expanded)
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.max(0, treeRow.width - parent.x - x - 30)
                        text: treeRow.name
                        color: treeRow.kind === "directory"
                               ? Theme.textPrimary
                               : root.gitFileColor(treeRow.path,
                                                   root.gitRevision)
                        font.pixelSize: Theme.fontSizeTree
                        elide: Text.ElideRight
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

                KvIconButton {
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    width: 22
                    height: 22
                    z: 2
                    visible: root.isRunnableScript(treeRow.name, treeRow.kind)
                             && (entryArea.containsMouse
                                 || treeRow.path === root.selectedPath)
                    enabled: visible
                    iconName: "run"
                    iconSize: 13
                    primary: true
                    tooltip: qsTr("Executar script")
                    onClicked: root.scriptRunRequested(treeRow.path)
                }
            }
        }
    }
}
