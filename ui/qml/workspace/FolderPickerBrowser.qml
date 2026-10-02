import QtQuick
import KineinVectis

// O navegador de pastas amplo (0.3.9, retorno do autor: "encolhido"; modelo
// do seletor da JetBrains): locais a esquerda, a barra com o caminho em cima,
// a lista ocupando o resto e uma linha de estado embaixo. Uma ilha de cantos
// redondos dentro do cartao, como os paineis da IDE.
Rectangle {
    id: root

    property var controller
    property var recentProjects: []

    radius: Theme.radiusLarge
    color: Theme.background0
    border.color: Theme.borderSoft
    border.width: 1

    function focusList() {
        directoryList.focusList();
    }

    FolderPickerPlaces {
        id: places

        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.margins: 1
        // Estreito, a coluna de locais cede: a lista e' o que importa.
        width: root.width >= 640 ? 200 : 0
        visible: width > 0
        controller: root.controller
        recentProjects: root.recentProjects
    }

    Item {
        id: main

        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: places.visible ? places.right : parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall

        FolderPickerPathBar {
            id: pathBar

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            controller: root.controller
            onPathEditingFinished: directoryList.focusList()
        }

        FolderPickerNewFolderRow {
            id: folderPanel

            anchors.top: pathBar.bottom
            anchors.topMargin: Theme.spacingSmall
            anchors.left: parent.left
            anchors.right: parent.right
            controller: root.controller
        }

        FolderPickerDirectoryList {
            id: directoryList

            anchors.top: folderPanel.visible ? folderPanel.bottom : pathBar.bottom
            anchors.topMargin: Theme.spacingSmall
            anchors.bottom: statusLine.top
            anchors.bottomMargin: Theme.spacingXSmall
            anchors.left: parent.left
            anchors.right: parent.right
            controller: root.controller
            onPathTypingRequested: function(seed) {
                pathBar.beginEditing(seed === "" ? undefined : seed);
            }
        }

        Text {
            id: statusLine

            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            leftPadding: Theme.spacingSmall
            text: {
                const count = root.controller.entriesModel.count;
                const folders = count === 1 ? qsTr("1 pasta") : qsTr("%1 pastas").arg(count);
                const hidden = root.controller.hiddenCount;
                if (hidden === 0 || root.controller.showHidden) return folders;
                return folders + " · " + (hidden === 1 ? qsTr("1 oculta")
                                                       : qsTr("%1 ocultas").arg(hidden));
            }
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }
    }
}
