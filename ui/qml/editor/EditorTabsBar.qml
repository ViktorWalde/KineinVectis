pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As abas do editor (Etapa 2, F3 do roadmaps/43, 2026-09-18): a aba ATIVA e'
// obvia — fundo do editor, borda de acento em cima, texto primario — e as
// inativas ficam no fundo do painel (o "islands" da referencia: a aba ativa
// e o editor sao a mesma superficie). O arquivo modificado mostra "●" no
// lugar do ✕ ate' o mouse chegar; o botao "Salvar" amarelo permanente saiu:
// salvar e' Ctrl+S ou o autosave (decisao do autor, 2026-09-18).
Item {
    id: root

    property var filesModel
    property int fileCount: 0
    property int currentIndex: -1

    readonly property real tabSpacing: 2
    readonly property real minimumTabWidth: 84
    readonly property real maximumTabWidth: 220
    readonly property real tabChromeWidth: 16 + 24 + 3 * Theme.spacingSmall
    readonly property real naturalTabsWidth: calculateNaturalTabsWidth()
    readonly property bool tabsCompressed: naturalTabsWidth > width
    readonly property real compactTabWidth: compactWidthFor(width, fileCount)

    // O DOCUMENTO, nao a posicao (V5): o delegate tem o `docId` do modelo, e e'
    // ele que atravessa. Assim nenhuma operacao de dominio depende de onde a
    // aba esta' na fila.
    signal tabSelected(int docId)
    signal tabCloseRequested(int docId)

    height: fileCount > 0 ? 36 : 0
    visible: fileCount > 0

    function naturalTabWidthFor(name) {
        return Math.min(root.maximumTabWidth,
                        Math.max(root.minimumTabWidth,
                                 Math.ceil(tabFontMetrics.advanceWidth(name))
                                 + root.tabChromeWidth));
    }

    function calculateNaturalTabsWidth() {
        if (!root.filesModel || root.fileCount <= 0) {
            return 0;
        }
        let total = (root.fileCount - 1) * root.tabSpacing;
        for (let i = 0; i < root.fileCount; ++i) {
            total += root.naturalTabWidthFor(root.filesModel.get(i).name);
        }
        return total;
    }

    function compactWidthFor(availableWidth, count) {
        if (count <= 0) {
            return root.maximumTabWidth;
        }
        const gaps = Math.max(0, count - 1) * root.tabSpacing;
        return Math.max(root.minimumTabWidth,
                        Math.floor((Math.max(0, availableWidth) - gaps) / count));
    }

    function tabWidthFor(name) {
        return root.tabsCompressed ? root.compactTabWidth
                                   : root.naturalTabWidthFor(name);
    }

    FontMetrics {
        id: tabFontMetrics

        font.pixelSize: 12
    }

    ListView {
        id: tabList

        anchors.fill: parent
        anchors.bottomMargin: 4
        orientation: ListView.Horizontal
        spacing: root.tabSpacing
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        interactive: contentWidth > width
        model: root.filesModel
        currentIndex: root.currentIndex
        highlightMoveDuration: 0

        onCurrentIndexChanged: {
            if (currentIndex >= 0) {
                positionViewAtIndex(currentIndex, ListView.Contain);
            }
        }
        onWidthChanged: {
            if (currentIndex >= 0) {
                positionViewAtIndex(currentIndex, ListView.Contain);
            }
        }
        onContentWidthChanged: {
            if (currentIndex >= 0) {
                positionViewAtIndex(currentIndex, ListView.Contain);
            }
        }

        delegate: Rectangle {
            id: tabDelegate

            required property int index
            required property int docId
            required property string name
            required property bool modified

            readonly property bool active: index === root.currentIndex

            width: root.tabWidthFor(name)
            height: 32
            radius: Theme.radius
            color: active ? Theme.background0 : (tabArea.containsMouse ? Theme.surface2 : Theme.surface1)

            // A borda de acento em cima: a marca da aba ativa.
            Rectangle {
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: Theme.radius
                anchors.rightMargin: Theme.radius
                height: 2
                radius: 1
                visible: tabDelegate.active
                color: Theme.accent
            }

            KvFileIcon {
                id: tabFileIcon

                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                size: 16
                fileName: tabDelegate.name
            }

            Text {
                id: tabLabel

                anchors.verticalCenter: parent.verticalCenter
                anchors.left: tabFileIcon.right
                anchors.leftMargin: Theme.spacingXSmall
                anchors.right: closeButton.left
                anchors.rightMargin: Theme.spacingXSmall
                text: tabDelegate.name
                color: tabDelegate.active ? Theme.textPrimary : Theme.textSecondary
                elide: Text.ElideRight
                font.pixelSize: 12
                font.weight: tabDelegate.active ? Font.DemiBold : Font.Normal
            }

            // Modificado: "●" onde o ✕ fica; o ✕ volta ao pairar.
            Text {
                anchors.centerIn: closeButton
                visible: tabDelegate.modified && !closeArea.containsMouse
                text: "●"
                color: Theme.accent
                font.pixelSize: 11
            }

            KvIconButton {
                id: closeButton

                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: 2
                width: 24
                height: 24
                opacity: tabDelegate.modified && !closeArea.containsMouse ? 0 : 1
                iconName: "close"
                iconSize: 13
                danger: true
                tooltip: tabDelegate.modified ? qsTr("Fechar aba (não salvo)") : qsTr("Fechar aba")
                onClicked: root.tabCloseRequested(tabDelegate.docId)

                MouseArea {
                    id: closeArea

                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                }
            }

            MouseArea {
                id: tabArea

                anchors.fill: parent
                anchors.rightMargin: closeButton.width + Theme.spacingSmall
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onContainsMouseChanged: {
                    if (containsMouse && tabLabel.truncated) {
                        TooltipController.showFor(tabDelegate,
                                                  tabDelegate.name,
                                                  "bottom");
                    } else {
                        TooltipController.hideFor(tabDelegate);
                    }
                }
                onClicked: root.tabSelected(tabDelegate.docId)
            }
        }
    }
}
