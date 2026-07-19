import QtQuick

// Modo "imagem" dos .md: cobre a area do editor com o markdown RENDERIZADO,
// no mesmo padrao visual do Manual da IDE (DocumentationDialog). O chip no
// canto alterna Codigo <-> Imagem e so aparece em arquivo .md.
//
// Visual puro: quem decide se o arquivo esta em preview e o
// EditorMarkdownModeController; quem e dono do texto e o editor (o binding
// de `sourceText` e vivo, entao o preview acompanha o buffer).
//
// Limitacao registrada da v1: imagens com caminho relativo dentro do .md nao
// resolvem (sem baseUrl do arquivo); texto, titulos, listas, tabelas e codigo
// renderizam. Suficiente para a leitura de documentacao pedida pelo autor.
Item {
    id: root

    property var mode: null
    property string filePath: ""
    property string sourceText: ""
    readonly property bool isMarkdown: root.mode !== null && root.filePath !== ""
                                       && root.mode.isMarkdownPath(root.filePath)
    readonly property bool showing: root.mode !== null && root.isMarkdown
                                    && root.mode.isPreview(root.filePath,
                                                          root.mode.revision)

    Rectangle {
        anchors.fill: parent
        visible: root.showing
        color: Theme.backgroundEditor
        radius: Theme.radius

        Flickable {
            id: previewFlick

            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            clip: true
            contentWidth: width
            contentHeight: previewText.paintedHeight + 2 * Theme.spacingLarge
            boundsBehavior: Flickable.StopAtBounds

            TextEdit {
                id: previewText

                x: Theme.spacingLarge
                y: Theme.spacingSmall
                width: previewFlick.width - 2 * Theme.spacingLarge
                readOnly: true
                selectByMouse: true
                text: root.showing ? root.sourceText : ""
                textFormat: TextEdit.MarkdownText
                wrapMode: TextEdit.Wrap
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.uiFont
                font.pixelSize: 13
            }

            VerticalScrollBar {
                anchors.top: parent.top
                anchors.right: parent.right
                height: previewFlick.height
                contentSize: previewFlick.contentHeight
                viewportSize: previewFlick.height
                position: previewFlick.contentY
                onMoveRequested: function(position) {
                    previewFlick.contentY = position;
                }
            }
        }
    }

    // O alternador fica por cima das duas vistas, sempre no canto.
    Rectangle {
        id: modeChip

        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        width: modeChipText.width + 2 * Theme.spacingSmall
        height: 22
        radius: Theme.radius
        visible: root.isMarkdown
        color: modeChipArea.containsMouse ? Theme.surfaceSelected : Theme.surface2
        border.width: 1
        border.color: root.showing ? Theme.accent : Theme.borderSoft

        Text {
            id: modeChipText

            anchors.centerIn: parent
            text: root.showing ? qsTr("Código") : qsTr("Imagem")
            color: root.showing ? Theme.accent : Theme.textSecondary
            font.pixelSize: 11
        }

        MouseArea {
            id: modeChipArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.mode.toggle(root.filePath)
        }
    }
}
