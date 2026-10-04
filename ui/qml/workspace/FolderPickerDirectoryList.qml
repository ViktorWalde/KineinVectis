pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As pastas da pasta atual (0.3.9): linhas largas com o icone de pasta, e a
// pasta de PROJETO marcada com o icone do arquivo que a faz projeto e o nome
// do ecossistema (0.147.0) — da' para achar o projeto sem abrir cada pasta.
// Um clique seleciona, dois entram; no teclado, setas, Enter e Backspace.
FocusScope {
    id: root

    property var controller

    signal pathTypingRequested(string seed)

    function focusList() {
        folderList.forceActiveFocus();
    }

    Text {
        anchors.centerIn: parent
        visible: root.controller.loading
        text: qsTr("Carregando…")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeBody
    }

    Text {
        anchors.centerIn: parent
        width: parent.width - 2 * Theme.spacingLarge
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        visible: !root.controller.loading && root.controller.entriesModel.count === 0
        text: root.controller.hiddenCount > 0
              ? qsTr("Só há pastas ocultas aqui. O olho na barra as mostra.")
              : qsTr("Nenhuma subpasta. Dá para escolher esta mesma pasta.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeBody
    }

    ListView {
        id: folderList

        anchors.fill: parent
        visible: !root.controller.loading && root.controller.entriesModel.count > 0
        clip: true
        focus: true
        boundsBehavior: Flickable.StopAtBounds
        model: root.controller.entriesModel
        currentIndex: root.controller.selectedIndex()
        highlightFollowsCurrentItem: false
        onCurrentIndexChanged: if (currentIndex >= 0) positionViewAtIndex(currentIndex, ListView.Contain)

        Keys.onUpPressed: root.controller.moveSelection(-1)
        Keys.onDownPressed: root.controller.moveSelection(1)
        Keys.onReturnPressed: root.controller.enterSelected()
        Keys.onEnterPressed: root.controller.enterSelected()
        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Backspace && root.controller.parentPath !== "") {
                root.controller.browsePath(root.controller.parentPath);
                event.accepted = true;
            } else if (event.key === Qt.Key_L && (event.modifiers & Qt.ControlModifier)) {
                root.pathTypingRequested("");
                event.accepted = true;
            } else if (event.text === "/" || event.text === "~") {
                root.pathTypingRequested(event.text);
                event.accepted = true;
            }
        }

        delegate: Rectangle {
            id: folderRow

            required property string name
            required property string path
            required property string kind
            required property string buildSystems
            readonly property bool hybrid: ProjectKindNames.isHybrid(folderRow.buildSystems)
            readonly property bool selected: root.controller.selectedPath === folderRow.path
            readonly property bool hidden: root.controller.isHidden(folderRow.name)

            width: folderList.width
            height: 34
            radius: Theme.radius
            color: folderRow.selected ? Theme.surfaceSelected
                   : (rowHover.hovered ? Theme.surface2 : "transparent")
            border.color: folderRow.selected && folderList.activeFocus ? Theme.accentDim
                                                                       : "transparent"
            border.width: 1

            Behavior on color {
                ColorAnimation { duration: Theme.motionFast }
            }

            // O destaque da linha continua aceso com o mouse sobre o selo.
            HoverHandler {
                id: rowHover
            }

            KvFileIcon {
                id: folderIcon

                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                directory: true
                size: 18
                opacity: folderRow.hidden ? 0.55 : 1.0
            }

            Text {
                anchors.left: folderIcon.right
                anchors.leftMargin: Theme.spacingMedium
                anchors.right: badge.visible ? badge.left : parent.right
                anchors.rightMargin: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                text: folderRow.name
                color: folderRow.selected ? Theme.textPrimary
                       : (folderRow.hidden ? Theme.textMuted : Theme.textSecondary)
                font.pixelSize: Theme.fontSizeTree
                elide: Text.ElideRight
            }

            // O ecossistema da pasta. Hibrida (duas linguagens ou mais): o
            // selo de camadas em ambar, e as linguagens no tooltip
            // (2026-10-03, pedido do autor: antes so' o primeiro marcador).
            Rectangle {
                id: badge

                z: 1
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                visible: folderRow.kind !== ""
                width: badgeRow.width + 2 * Theme.spacingSmall
                height: 22
                radius: height / 2
                color: Theme.background2
                border.color: folderRow.hybrid ? Theme.accentDim : Theme.borderSoft
                border.width: 1

                Row {
                    id: badgeRow

                    anchors.centerIn: parent
                    spacing: Theme.spacingXSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: folderRow.hybrid
                        name: "layers"
                        size: 14
                        active: true
                    }

                    KvFileIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !folderRow.hybrid
                        fileName: ProjectKindNames.marker(folderRow.kind)
                        size: 14
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: folderRow.hybrid
                              ? ProjectKindNames.languages(folderRow.buildSystems).map(function(entry) {
                                    return entry.language;
                                }).join(" + ")
                              : ProjectKindNames.label(folderRow.kind)
                        color: Theme.textSecondary
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }

                // Por cima da linha so' para o pairar: o clique (NoButton)
                // segue para a linha, que escolhe e abre.
                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: folderRow.hybrid
                    acceptedButtons: Qt.NoButton
                    onContainsMouseChanged: {
                        if (containsMouse) TooltipController.showFor(badge, qsTr("Projeto híbrido: %1")
                                                                      .arg(ProjectKindNames.hybridDescription(folderRow.buildSystems)), "bottom");
                        else TooltipController.hideFor(badge);
                    }
                }
            }

            MouseArea {
                id: folderMouse

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    folderList.forceActiveFocus();
                    root.controller.selectedPath = folderRow.path;
                }
                onDoubleClicked: root.controller.browsePath(folderRow.path)
            }
        }
    }
}
