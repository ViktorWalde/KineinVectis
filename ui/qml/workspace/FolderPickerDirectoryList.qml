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
            readonly property bool selected: root.controller.selectedPath === folderRow.path
            readonly property bool hidden: root.controller.isHidden(folderRow.name)

            width: folderList.width
            height: 34
            radius: Theme.radius
            color: folderRow.selected ? Theme.surfaceSelected
                   : (folderMouse.containsMouse ? Theme.surface2 : "transparent")
            border.color: folderRow.selected && folderList.activeFocus ? Theme.accentDim
                                                                       : "transparent"
            border.width: 1

            Behavior on color {
                ColorAnimation { duration: Theme.motionFast }
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

            Rectangle {
                id: badge

                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                visible: folderRow.kind !== ""
                width: badgeRow.width + 2 * Theme.spacingSmall
                height: 22
                radius: height / 2
                color: Theme.background2
                border.color: Theme.borderSoft
                border.width: 1

                Row {
                    id: badgeRow

                    anchors.centerIn: parent
                    spacing: Theme.spacingXSmall

                    KvFileIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        fileName: ProjectKindNames.marker(folderRow.kind)
                        size: 14
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: ProjectKindNames.label(folderRow.kind)
                        color: Theme.textSecondary
                        font.pixelSize: Theme.fontSizeSmall
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
