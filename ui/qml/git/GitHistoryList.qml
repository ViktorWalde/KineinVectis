pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O HISTORICO de commits.
//
// Saiu do GitPanel.qml em 2026-09-03 (etapa 17). Mesma razao e mesma forma da
// GitChangesList: a raiz e' o ListView, e o posicionamento fica no GitPanel.
ListView {
    id: root

    property var historyModel
    property bool historyLoading: false
    property bool repo: false

    signal commitActivated(string sha, string shortSha, string summary)


    // B2 (DocsPublic/roadmaps/24): barra de rolagem. `parent: root` é OBRIGATÓRIO — um filho
    // declarado dentro de um ListView vira filho do contentItem e ROLARIA
    // junto com a lista. O ListView segue sendo a fonte da verdade.
    VerticalScrollBar {
        id: scrollBar

        parent: root
        anchors.right: root.right
        anchors.top: root.top
        anchors.bottom: root.bottom

        contentSize: root.contentHeight
        viewportSize: root.height
        position: root.contentY

        onMoveRequested: function(position) {
            root.contentY = position;
        }
    }
    clip: true
    model: root.historyModel

    Text {
        anchors.centerIn: parent
        visible: root.historyModel.count === 0
        text: root.historyLoading
              ? qsTr("Carregando histórico...")
              : (root.repo
                 ? qsTr("Sem commits ainda.")
                 : qsTr("Este workspace não é um repositório git."))
        color: Theme.textMuted
        font.pixelSize: 11
    }

    delegate: Rectangle {
        id: commitRowItem

        required property string sha
        required property string shortSha
        required property string author
        required property string age
        required property string summary

        width: root.width
        height: 24
        radius: Theme.radiusXSmall
        color: commitRowArea.containsMouse ? Theme.surface2 : "transparent"

        Text {
            id: commitShaText

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            text: commitRowItem.shortSha
            color: Theme.accent
            font.pixelSize: 11
            font.family: Theme.monoFont
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: commitShaText.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: commitMetaText.left
            anchors.rightMargin: Theme.spacingSmall
            text: commitRowItem.summary
            color: Theme.textPrimary
            font.pixelSize: 11
            elide: Text.ElideRight
        }

        Text {
            id: commitMetaText

            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            text: commitRowItem.author + ", " + commitRowItem.age
            color: Theme.textMuted
            font.pixelSize: 10
        }

        MouseArea {
            id: commitRowArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.commitActivated(commitRowItem.sha,
                                             commitRowItem.shortSha,
                                             commitRowItem.summary)
        }
    }
}

